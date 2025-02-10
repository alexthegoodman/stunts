use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex, MutexGuard};
use std::{fs, thread};

use bytemuck::Contiguous;
use floem::action::debounce_action;
use floem::common::{input_styles, simple_button, small_button};
use floem::event::{Event, EventListener, EventPropagation};
use floem::ext_event::create_signal_from_tokio_channel;
use floem::keyboard::{Key, KeyCode, NamedKey};
use floem::kurbo::Size;
use floem::peniko::Color;
use floem::reactive::{create_effect, create_rw_signal, create_signal, RwSignal, SignalRead};
use floem::style::{Background, CursorStyle, Transition};
use floem::taffy::{AlignItems, Position};
use floem::text::Weight;
use floem::views::dropdown::dropdown;
use floem::views::editor::view;
use floem::views::{
    container, dyn_container, empty, label, scroll, stack, tab, text_input, virtual_stack,
    VirtualDirection, VirtualItemSize,
};
use floem::window::WindowConfig;
use floem_renderer::gpu_resources::{self, GpuResources};
use floem_winit::dpi::{LogicalSize, PhysicalSize};
use floem_winit::event::{ElementState, MouseButton};
use stunts_engine::animations::{ObjectType, Sequence};
use stunts_engine::editor::{Editor, Viewport};
use tokio::runtime::Runtime;
use tokio::spawn;
use tokio::task::spawn_local;
use uuid::Uuid;
// use views::buttons::{nav_button, option_button, small_button};
// use winit::{event_loop, window};
use wgpu::util::DeviceExt;

use floem::context::PaintState;
// use floem::floem_reactive::SignalGet;
use floem::reactive::{SignalGet, SignalUpdate};
use floem::views::text;
use floem::views::Decorators;
use floem::views::{h_stack, svg, v_stack};
use floem::{
    views::{button, dropdown},
    IntoView,
};
use floem::{Application, CustomRenderCallback};
use floem::{GpuHelper, View, WindowHandle};

use floem::unit::{Auto, DurationUnitExt, Pct, UnitExt};
use std::time::Duration;

use crate::editor_state::EditorState;

pub fn styled_input(
    label_text: String,
    initial_value: &str,
    placeholder: &str,
    on_event_stop: Box<dyn Fn(MutexGuard<EditorState>, String) + 'static>,
    // mut values: HashMap<String, RwSignal<String>>,
    mut editor_state: Arc<Mutex<EditorState>>,
    name: String,
    signal_type: ObjectType,
) -> impl IntoView {
    let value = create_rw_signal(initial_value.to_string());

    let state_2 = Arc::clone(&editor_state);

    create_effect({
        let name = name.clone();

        move |_| {
            // need to value.set in undos defined in properties_panel
            let mut editor_state = editor_state.lock().unwrap();
            editor_state.register_signal(name.to_string(), value, signal_type.clone());
        }
    });

    v_stack((
        label(move || label_text.clone()).style(|s| s.font_size(10.0).margin_bottom(1.0)),
        text_input(value)
            .on_event_stop(EventListener::KeyUp, move |event: &Event| {
                if let Event::KeyUp(key_event) = event {
                    let editor_state = state_2.lock().unwrap();

                    // Handle keyboard shortcuts first
                    if editor_state.current_modifiers.control_key() {
                        match key_event.key.logical_key {
                            Key::Character(ref c) if c.to_lowercase() == "z" => {
                                // Don't trigger value update for Ctrl+Z
                                return;
                            }
                            Key::Character(ref c) if c.to_lowercase() == "y" => {
                                // Don't trigger value update for Ctrl+Y
                                return;
                            }
                            _ => {}
                        }
                    }

                    match key_event.key.logical_key {
                        // Ignore all control and navigation keys
                        Key::Named(NamedKey::ArrowUp)
                        | Key::Named(NamedKey::ArrowDown)
                        | Key::Named(NamedKey::ArrowLeft)
                        | Key::Named(NamedKey::ArrowRight)
                        | Key::Named(NamedKey::Enter)
                        | Key::Named(NamedKey::Tab)
                        | Key::Named(NamedKey::Escape)
                        | Key::Named(NamedKey::Home)
                        | Key::Named(NamedKey::End)
                        | Key::Named(NamedKey::PageUp)
                        | Key::Named(NamedKey::PageDown)
                        | Key::Named(NamedKey::Control)
                        | Key::Named(NamedKey::Shift)
                        | Key::Named(NamedKey::Alt)
                        | Key::Named(NamedKey::Meta) => {
                            // Ignore these keys
                            println!("Ignoring control/navigation key");
                            return;
                        }
                        // Only trigger value update for actual content changes
                        _ => {
                            println!("Content change detected: {:?}", key_event.key.logical_key);
                            let current_value = value.get();
                            on_event_stop(editor_state, current_value);
                        }
                    }
                }
            })
            .placeholder(placeholder)
            .style(|s| input_styles(s)),
    ))
    .style(|s| s.margin_bottom(10))
}

use tokio::sync::mpsc;
use tokio::time::sleep;

#[derive(Clone)]
struct DebounceMessage {
    pub value: String,
}

#[derive(Clone)]
struct ConfirmMessage {
    pub is_confirmed: bool,
}

pub fn debounce_input<F>(
    label_text: String,
    initial_value: &str,
    placeholder: &str,
    on_event_stop: F,
    mut editor_state: Arc<Mutex<EditorState>>,
    name: String,
    signal_type: ObjectType,
) -> impl IntoView
where
    F: Fn(String) + Clone + 'static,
{
    let value = create_rw_signal(initial_value.to_string());
    let filtered_value = create_rw_signal(initial_value.to_string());
    let state_2 = Arc::clone(&editor_state);
    let state_3 = Arc::clone(&editor_state);

    let signal_registered = create_rw_signal(false);

    debounce_action(filtered_value, Duration::from_millis(300), move || {
        println!("debounced action...");
        on_event_stop(filtered_value.get_untracked());
    });

    create_effect({
        let name = name.clone();

        move |_| {
            if !signal_registered.get() {
                let mut editor_state = editor_state.lock().unwrap();
                editor_state.register_signal(name.to_string(), value, signal_type.clone());
                signal_registered.set(true);
            }
        }
    });

    v_stack((
        label(move || label_text.clone()).style(|s| s.font_size(10.0).margin_bottom(1.0)),
        text_input(value)
            .on_event_stop(EventListener::KeyUp, move |event: &Event| {
                if let Event::KeyUp(key_event) = event {
                    let editor_state = state_2.lock().unwrap();

                    // Handle keyboard shortcuts first
                    if editor_state.current_modifiers.control_key() {
                        match key_event.key.logical_key {
                            Key::Character(ref c) if c.to_lowercase() == "z" => return,
                            Key::Character(ref c) if c.to_lowercase() == "y" => return,
                            _ => {}
                        }
                    }

                    drop(editor_state);

                    match key_event.key.logical_key {
                        // Ignore all control and navigation keys
                        Key::Named(NamedKey::ArrowUp)
                        | Key::Named(NamedKey::ArrowDown)
                        | Key::Named(NamedKey::ArrowLeft)
                        | Key::Named(NamedKey::ArrowRight)
                        | Key::Named(NamedKey::Enter)
                        | Key::Named(NamedKey::Tab)
                        | Key::Named(NamedKey::Escape)
                        | Key::Named(NamedKey::Home)
                        | Key::Named(NamedKey::End)
                        | Key::Named(NamedKey::PageUp)
                        | Key::Named(NamedKey::PageDown)
                        | Key::Named(NamedKey::Control)
                        | Key::Named(NamedKey::Shift)
                        | Key::Named(NamedKey::Alt)
                        | Key::Named(NamedKey::Meta) => {
                            return;
                        }
                        // Only trigger value update for actual content changes
                        _ => {
                            filtered_value.set(value.get());
                        }
                    }
                }
            })
            .placeholder(placeholder)
            .style(|s| input_styles(s)),
    ))
    .style(|s| s.margin_bottom(10))
}

// Define an option type for better ergonomics
#[derive(Clone)]
pub struct DropdownOption {
    pub id: String,
    pub label: String,
}

pub fn create_dropdown<F>(
    initial_selection: String,
    provided_options: Vec<DropdownOption>,
    on_selection: F,
) -> impl IntoView
where
    F: Fn(String) + Clone + 'static,
{
    let (selected, set_selected) = create_signal(initial_selection.clone());
    let (options, _set_options) = create_signal(provided_options);

    // Start with the default option
    let mut dropdown_options = vec![DropdownOption {
        id: initial_selection.clone(),
        label: "Make a selection".to_string(),
    }];

    // Add the file options
    dropdown_options.extend(options.get().into_iter().map(|file| DropdownOption {
        id: file.id.clone(),
        label: file.label.clone(),
    }));

    // Create the dropdown
    let dropdown = {
        let dropdown_2 = dropdown_options.clone();
        let set_selected = set_selected.clone();
        let on_selection = on_selection.clone();

        dropdown(
            move || {
                let selected = selected.get();
                let track = options.get();

                if options.get().len() > 0 {
                    dropdown_options
                        .clone()
                        .into_iter()
                        .find(|opt| opt.id == selected)
                        .expect("Couldn't find dropdown option")
                } else {
                    DropdownOption {
                        id: initial_selection.clone(),
                        label: "Make a selection".to_string(),
                    }
                }
            },
            // Main view (selected item)
            move |item: DropdownOption| {
                text(item.label.to_string())
                    .style(|s| {
                        s.background(Color::rgba(0.5, 0.5, 0.5, 1.0))
                            .padding_left(8)
                            .padding_right(8)
                            .width_full()
                    })
                    .into_any()
            },
            // Options iterator
            dropdown_2.clone(),
            // List item view
            move |item: DropdownOption| {
                text(item.label.to_string())
                    .style(|s| {
                        s.background(Color::rgba(0.5, 0.5, 0.5, 1.0))
                            .padding_left(8)
                            .padding_right(8)
                            .padding_top(2)
                            .padding_bottom(2)
                            .hover(|s| s.background(Color::rgba(0.5, 0.5, 0.5, 1.0)))
                            .width_full()
                            .cursor(CursorStyle::Pointer)
                    })
                    .into_any()
            },
        )
        .on_accept(move |new: DropdownOption| {
            set_selected.set(new.id.clone());
            on_selection(new.id.clone());
        })
        .style(|s| {
            s.width(200)
                .background(Color::rgba(0.5, 0.5, 0.5, 1.0))
                .border(1)
                .border_color(Color::rgba(0.5, 0.5, 0.5, 1.0))
                .border_radius(4)
                .position(Position::Relative)
                .class(dropdown::DropdownClass, |s| {
                    s.background(Color::rgba(0.5, 0.5, 0.5, 1.0))
                        .border(1)
                        .border_color(Color::rgba(0.5, 0.5, 0.5, 1.0))
                        .border_radius(4)
                        .z_index(999)
                        .position(Position::Absolute)
                        .height(300.0)
                })
                .class(dropdown::DropdownScrollClass, |s| s.height(300.0))
        })
    };

    dropdown
}

pub fn inline_dropdown<F>(
    button_text: String,
    label_text: RwSignal<String>,
    dropdown_options: RwSignal<Vec<DropdownOption>>,
    on_selection: F,
) -> impl IntoView
where
    F: Fn(String) + Clone + 'static,
{
    let dropdown_open = create_rw_signal(false);

    let dropdown_options_im: RwSignal<im::Vector<String>> = create_rw_signal(im::Vector::new());

    create_effect(move |_| {
        let options = dropdown_options.get();

        let new_options: im::Vector<String> = options.iter().map(|o| o.id.clone()).collect();

        dropdown_options_im.set(new_options);
    });

    v_stack((
        h_stack((
            simple_button(button_text, move |_| {
                dropdown_open.set(true);
            }),
            label(move || label_text.get()),
        )),
        dyn_container(
            move || dropdown_open.get(),
            move |is_dropdown_open| {
                let on_selection = on_selection.clone();

                if is_dropdown_open {
                    container((scroll({
                        virtual_stack(
                            VirtualDirection::Vertical,
                            VirtualItemSize::Fixed(Box::new(|| 30.0)),
                            move || dropdown_options_im.get(),
                            move |item| item.clone(),
                            move |item| {
                                let on_selection = on_selection.clone();

                                h_stack((simple_button(item.clone(), move |_| {
                                    on_selection(item.clone());
                                    dropdown_open.set(false);
                                }),))
                            },
                        )
                        .style(|s| {
                            s.flex_col()
                                .max_width(260.0)
                                .padding_vert(15.0)
                                .padding_horiz(20.0)
                                .background(Color::LIGHT_BLUE)
                        })
                    })
                    .style(|s| s.height(400.0)),))
                } else {
                    container((empty()))
                }
            },
        ),
    ))
}

pub fn play_sequence_button(
    // editor_state: Arc<Mutex<EditorState>>,
    editor: std::sync::Arc<Mutex<Editor>>,
    // viewport: std::sync::Arc<Mutex<Viewport>>,
    selected_sequence_data: RwSignal<Sequence>,
) -> impl IntoView {
    simple_button("Play Sequence".to_string(), move |_| {
        let mut editor = editor.lock().unwrap();

        if editor.is_playing {
            println!("Pause Sequence...");

            editor.is_playing = false;
            editor.start_playing_time = None;

            // should return objects to the startup positions and state
            editor.reset_sequence_objects();
        } else {
            println!("Play Sequence...");

            let now = std::time::Instant::now();
            editor.start_playing_time = Some(now);

            editor.current_sequence_data = Some(selected_sequence_data.get());
            editor.is_playing = true;
        }

        // EventPropagation::Continue
    })
}
