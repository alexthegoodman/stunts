use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex, MutexGuard};

use bytemuck::Contiguous;
use floem::common::input_styles;
use floem::event::{Event, EventListener, EventPropagation};
use floem::keyboard::{Key, KeyCode, NamedKey};
use floem::kurbo::Size;
use floem::peniko::Color;
use floem::reactive::{create_effect, create_rw_signal, create_signal, RwSignal, SignalRead};
use floem::style::{Background, CursorStyle, Transition};
use floem::taffy::AlignItems;
use floem::text::Weight;
use floem::views::editor::view;
use floem::views::{
    container, dyn_container, empty, label, scroll, stack, tab, text_input, virtual_stack,
    VirtualDirection, VirtualItemSize,
};
use floem::window::WindowConfig;
use floem_renderer::gpu_resources::{self, GpuResources};
use floem_winit::dpi::{LogicalSize, PhysicalSize};
use floem_winit::event::{ElementState, MouseButton};
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
) -> impl IntoView {
    let value = create_rw_signal(initial_value.to_string());

    let state_2 = Arc::clone(&editor_state);

    create_effect({
        let name = name.clone();
        move |_| {
            // need to value.set in undos defined in properties_panel
            let mut editor_state = editor_state.lock().unwrap();
            editor_state.register_signal(name.to_string(), value);
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
