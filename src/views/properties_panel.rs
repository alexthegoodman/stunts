use floem::common::card_styles;
use floem::common::simple_button;
use floem::common::small_button;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use stunts_engine::animations::ObjectType;
use stunts_engine::editor::color_to_wgpu;
use stunts_engine::editor::string_to_f32;
use stunts_engine::editor::wgpu_to_human;
use stunts_engine::editor::Editor;
use stunts_engine::editor::Viewport;
use stunts_engine::polygon::PolygonConfig;
use stunts_engine::st_image::StImageConfig;
use stunts_engine::text_due::TextRendererConfig;
use uuid::Uuid;
use wgpu::util::DeviceExt;

use floem::peniko::{Brush, Color};
use floem::reactive::{create_effect, create_rw_signal, create_signal, RwSignal, SignalRead};
use floem::reactive::{SignalGet, SignalUpdate};
use floem::text::Weight;
use floem::views::Decorators;
use floem::views::{container, dyn_container, empty, label};
use floem::views::{h_stack, v_stack};
use floem::GpuHelper;
use floem::IntoView;

use crate::editor_state::{self, EditorState};
use crate::helpers::utilities::save_saved_state_raw;

use super::color_pallete::rgb_view_debounced;
use super::inputs::create_dropdown;
use super::inputs::debounce_input;
use super::inputs::inline_dropdown;
use super::inputs::styled_input;
use super::inputs::DropdownOption;

pub fn properties_view(
    editor_state: Arc<Mutex<EditorState>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    editor: std::sync::Arc<Mutex<Editor>>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
    polygon_selected: RwSignal<bool>,
    selected_polygon_id: RwSignal<Uuid>,
    selected_polygon_data: RwSignal<PolygonConfig>,
    selected_sequence_id: RwSignal<String>,
) -> impl IntoView {
    let editor_cloned = Arc::clone(&editor);
    let editor_state2 = Arc::clone(&editor_state);
    let editor_state3 = Arc::clone(&editor_state);
    let editor_state4 = Arc::clone(&editor_state);
    let editor_state5 = Arc::clone(&editor_state);
    let editor_state6 = Arc::clone(&editor_state);
    let editor_state7 = Arc::clone(&editor_state);
    let editor_state8 = Arc::clone(&editor_state);
    let editor_state9 = Arc::clone(&editor_state);
    let editor_state10 = Arc::clone(&editor_state);
    let editor_state11 = Arc::clone(&editor_state);
    let editor_state12 = Arc::clone(&editor_state);
    let editor_state13 = Arc::clone(&editor_state);
    let editor_state14 = Arc::clone(&editor_state);
    let editor_state15 = Arc::clone(&editor_state);
    let editor_state16 = Arc::clone(&editor_state);
    let editor_state17 = Arc::clone(&editor_state);
    let editor_state18 = Arc::clone(&editor_state);
    let editor_state19 = Arc::clone(&editor_state);
    let editor_state20 = Arc::clone(&editor_state);

    let aside_width = 260.0;
    let quarters = (aside_width / 4.0) + (5.0 * 4.0);
    let thirds = (aside_width / 3.0) + (5.0 * 3.0);
    let halfs = (aside_width / 2.0) + (5.0 * 2.0);

    let back_active = RwSignal::new(false);

    v_stack((
        // label(|| "Properties"),
        simple_button("Back to Sequence".to_string(), move |_| {
            polygon_selected.set(false);
        }),
        v_stack((
            h_stack((
                debounce_input(
                    "Width:".to_string(),
                    &selected_polygon_data
                        .read()
                        .borrow()
                        .dimensions
                        .0
                        .to_string(),
                    "Enter width",
                    move |value| {
                        let mut editor_state = editor_state11.lock().unwrap();

                        // NOTE: editor_state actions are hooked into undo/redo as well as file save
                        editor_state
                            .update_width(&value, ObjectType::Polygon)
                            .expect("Couldn't update width");

                        drop(editor_state);

                        // TODO: should update selected_polygon_data?
                    },
                    editor_state,
                    "width".to_string(),
                    ObjectType::Polygon,
                )
                .style(move |s| s.width(halfs).margin_right(5.0)),
                debounce_input(
                    "Height:".to_string(),
                    &selected_polygon_data
                        .read()
                        .borrow()
                        .dimensions
                        .1
                        .to_string(),
                    "Enter height",
                    move |value| {
                        let mut editor_state = editor_state12.lock().unwrap();

                        editor_state
                            .update_height(&value, ObjectType::Polygon)
                            .expect("Couldn't update height");

                        drop(editor_state);
                    },
                    editor_state2,
                    "height".to_string(),
                    ObjectType::Polygon,
                )
                .style(move |s| s.width(halfs)),
            )),
            h_stack((
                debounce_input(
                    "Red:".to_string(),
                    &wgpu_to_human(selected_polygon_data.read().borrow().fill[0]).to_string(),
                    "0-255",
                    move |value| {
                        let mut editor_state = editor_state13.lock().unwrap();

                        editor_state
                            .update_red(&value)
                            .expect("Couldn't update red");

                        drop(editor_state);
                    },
                    editor_state3,
                    "red".to_string(),
                    ObjectType::Polygon,
                )
                .style(move |s| s.width(thirds).margin_right(5.0)),
                debounce_input(
                    "Green:".to_string(),
                    &wgpu_to_human(selected_polygon_data.read().borrow().fill[1]).to_string(),
                    "0-255",
                    move |value| {
                        let mut editor_state = editor_state14.lock().unwrap();

                        editor_state
                            .update_green(&value)
                            .expect("Couldn't update green");

                        drop(editor_state);
                    },
                    editor_state4,
                    "green".to_string(),
                    ObjectType::Polygon,
                )
                .style(move |s| s.width(thirds).margin_right(5.0)),
                debounce_input(
                    "Blue:".to_string(),
                    &wgpu_to_human(selected_polygon_data.read().borrow().fill[2]).to_string(),
                    "0-255",
                    move |value| {
                        let mut editor_state = editor_state15.lock().unwrap();

                        editor_state
                            .update_blue(&value)
                            .expect("Couldn't update blue");

                        drop(editor_state);
                    },
                    editor_state5,
                    "blue".to_string(),
                    ObjectType::Polygon,
                )
                .style(move |s| s.width(thirds)),
            ))
            .style(move |s| {
                s.width(aside_width)
                // .display(Display::Grid)
                // .grid_template_columns(vec![TrackSizingFunction::Repeat(
                //     // floem::taffy::GridTrackRepetition::Count(3),
                //     GridTrackRepetition::AutoFill,
                //     vec![MinMax::from(MinMax {
                //         min: MinTrackSizingFunction::Fixed(LengthPercentage::Length(100.0)),
                //         max: MaxTrackSizingFunction::Fraction(1.0),
                //     })],
                // )])
            }),
            debounce_input(
                "Border Radius:".to_string(),
                &selected_polygon_data
                    .read()
                    .borrow()
                    .border_radius
                    .to_string(),
                "Enter radius",
                move |value| {
                    let mut editor_state = editor_state16.lock().unwrap();

                    editor_state
                        .update_border_radius(&value)
                        .expect("Couldn't update border radius");

                    drop(editor_state);
                },
                editor_state6,
                "border_radius".to_string(),
                ObjectType::Polygon,
            ),
            label(|| "Stroke").style(|s| s.margin_bottom(5.0)),
            h_stack((
                debounce_input(
                    "Thickness:".to_string(),
                    &selected_polygon_data
                        .read()
                        .borrow()
                        .stroke
                        .thickness
                        .to_string(),
                    "Enter thickness",
                    move |value| {
                        let mut editor_state = editor_state17.lock().unwrap();

                        editor_state
                            .update_stroke_thickness(&value)
                            .expect("Couldn't update blue");

                        drop(editor_state);
                    },
                    editor_state7,
                    "stroke_thickness".to_string(),
                    ObjectType::Polygon,
                )
                .style(move |s| s.width(quarters).margin_right(5.0)),
                debounce_input(
                    "Red:".to_string(),
                    &wgpu_to_human(selected_polygon_data.read().borrow().stroke.fill[0])
                        .to_string(),
                    "Enter red",
                    move |value| {
                        let mut editor_state = editor_state18.lock().unwrap();

                        editor_state
                            .update_stroke_red(&value)
                            .expect("Couldn't update blue");

                        drop(editor_state);
                    },
                    editor_state10,
                    "stroke_red".to_string(),
                    ObjectType::Polygon,
                )
                .style(move |s| s.width(quarters).margin_right(5.0)),
                debounce_input(
                    "Green:".to_string(),
                    &wgpu_to_human(selected_polygon_data.read().borrow().stroke.fill[1])
                        .to_string(),
                    "Enter green",
                    move |value| {
                        let mut editor_state = editor_state19.lock().unwrap();

                        editor_state
                            .update_stroke_green(&value)
                            .expect("Couldn't update blue");

                        drop(editor_state);
                    },
                    editor_state8,
                    "stroke_green".to_string(),
                    ObjectType::Polygon,
                )
                .style(move |s| s.width(quarters).margin_right(5.0)),
                debounce_input(
                    "Blue:".to_string(),
                    &wgpu_to_human(selected_polygon_data.read().borrow().stroke.fill[2])
                        .to_string(),
                    "Enter blue",
                    move |value| {
                        let mut editor_state = editor_state20.lock().unwrap();

                        editor_state
                            .update_stroke_blue(&value)
                            .expect("Couldn't update blue");

                        drop(editor_state);
                    },
                    editor_state9,
                    "stroke_blue".to_string(),
                    ObjectType::Polygon,
                )
                .style(move |s| s.width(quarters)),
            )),
            label(|| "Path Settings").style(|s| s.margin_bottom(5.0)),
            // add curve settings (linear, bezier) and control points for bezier
        ))
        .style(move |s| s.width(aside_width)),
    ))
    // .style(|s| card_styles(s))
    .style(|s| {
        s.width(260.0)
            .height(800.0)
            .margin_left(0.0)
            .margin_top(20)
            .z_index(10)
    })
}

pub fn text_properties_view(
    editor_state: Arc<Mutex<EditorState>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    editor: std::sync::Arc<Mutex<Editor>>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
    text_selected: RwSignal<bool>,
    selected_text_id: RwSignal<Uuid>,
    selected_text_data: RwSignal<TextRendererConfig>,
    selected_sequence_id: RwSignal<String>,
) -> impl IntoView {
    let editor_cloned = Arc::clone(&editor);
    let editor_cloned2 = Arc::clone(&editor);
    let editor_cloned3 = Arc::clone(&editor);
    let editor_state2 = Arc::clone(&editor_state);
    let editor_state3 = Arc::clone(&editor_state);
    let editor_state4 = Arc::clone(&editor_state);
    let editor_state5 = Arc::clone(&editor_state);
    let editor_state6 = Arc::clone(&editor_state);
    let editor_state7 = Arc::clone(&editor_state);
    let editor_state8 = Arc::clone(&editor_state);

    let aside_width = 260.0;
    let quarters = (aside_width / 4.0) + (5.0 * 4.0);
    let thirds = (aside_width / 3.0) + (5.0 * 3.0);
    let halfs = (aside_width / 2.0) + (5.0 * 2.0);

    let back_active = RwSignal::new(false);
    let font_dropdown_options: RwSignal<Vec<DropdownOption>> = create_rw_signal(Vec::new());
    let selected_font_family = create_rw_signal("Aleo".to_string());
    let init_red = create_rw_signal(40);
    let init_green = create_rw_signal(40);
    let init_blue = create_rw_signal(40);
    let defaults_set = create_rw_signal(false);

    create_effect(move |_| {
        let editor = editor_cloned.lock().unwrap();

        let options: Vec<DropdownOption> = editor
            .font_manager
            .font_data
            .iter()
            .filter(|fd| fd.2 == "Regular")
            .map(|fd| DropdownOption {
                id: fd.0.clone(),
                label: fd.0.clone(),
            })
            .collect();

        font_dropdown_options.set(options);
    });

    create_effect(move |_| {
        let selected_data = selected_text_data.get();
        let selected_family = selected_data.font_family.clone();
        let selected_red = selected_data.color[0];
        let selected_green = selected_data.color[1];
        let selected_blue = selected_data.color[2];

        selected_font_family.set(selected_family);
        init_red.set(selected_red);
        init_green.set(selected_green);
        init_blue.set(selected_blue);

        defaults_set.set(true);
    });

    let on_font_selection = move |font_id: String| {
        // TODO: wrap up in editor_state for undo/redo

        println!("on_font_selection {:?}", font_id);

        // update editor's text_item, recall render text
        let mut editor = editor.lock().unwrap();

        editor.update_text_font_family(font_id.clone(), selected_text_id.get());

        drop(editor);

        // update selected_text_data
        let mut new_text_data = selected_text_data.get();
        new_text_data.font_family = font_id.clone();
        selected_text_data.set(new_text_data);

        // save to saved_state
        let mut editor_state = editor_state3.lock().unwrap();
        let mut saved_state = editor_state
            .record_state
            .saved_state
            .as_mut()
            .expect("Couldn't get Saved State");

        saved_state.sequences.iter_mut().for_each(|s| {
            if s.id == selected_sequence_id.get() {
                s.active_text_items.iter_mut().for_each(|t| {
                    if t.id == selected_text_id.get().to_string() {
                        t.font_family = font_id.clone();
                    }
                });
            }
        });

        save_saved_state_raw(saved_state.clone());

        editor_state.record_state.saved_state = Some(saved_state.clone());

        drop(editor_state);
    };

    let on_color_update = move |r: i32, g: i32, b: i32| {
        // TODO: wrap up in editor_state for undo/redo

        println!("Updating text color... {} {} {}", r, g, b);

        let mut editor = editor_cloned2.lock().unwrap();

        let color = [r, g, b, 255];

        editor.update_text_color(selected_text_id.get(), color);

        drop(editor);

        // update selected_text_data
        let mut new_text_data = selected_text_data.get();
        new_text_data.color = color;
        selected_text_data.set(new_text_data);

        // save to saved_state
        let mut editor_state = editor_state4.lock().unwrap();
        let mut saved_state = editor_state
            .record_state
            .saved_state
            .as_mut()
            .expect("Couldn't get Saved State");

        saved_state.sequences.iter_mut().for_each(|s| {
            if s.id == selected_sequence_id.get() {
                s.active_text_items.iter_mut().for_each(|t| {
                    if t.id == selected_text_id.get().to_string() {
                        t.color = color;
                    }
                });
            }
        });

        save_saved_state_raw(saved_state.clone());

        editor_state.record_state.saved_state = Some(saved_state.clone());

        drop(editor_state);

        println!("Text color updated!");
    };

    v_stack((
        // label(|| "Properties"),
        simple_button("Back to Sequence".to_string(), move |_| {
            text_selected.set(false);
        }),
        dyn_container(
            move || defaults_set.get(),
            move |defaults_are_set| {
                let on_font_selection = on_font_selection.clone();
                let on_color_update = on_color_update.clone();
                let editor_state = editor_state.clone();
                let editor_state2 = editor_state2.clone();
                let editor_state5 = editor_state5.clone();
                let editor_state6 = editor_state6.clone();
                let editor_cloned3 = editor_cloned3.clone();
                let editor_state7 = editor_state7.clone();
                let editor_state8 = editor_state8.clone();

                if defaults_are_set {
                    v_stack((
                        v_stack((h_stack((
                            debounce_input(
                                "Width:".to_string(),
                                &selected_text_data.read().borrow().dimensions.0.to_string(),
                                "Enter width",
                                move |value| {
                                    let mut editor_state = editor_state7.lock().unwrap();

                                    // NOTE: editor_state actions are hooked into undo/redo as well as file save
                                    editor_state
                                        .update_width(&value, ObjectType::TextItem)
                                        .expect("Couldn't update width");

                                    drop(editor_state);

                                    // TODO: should update selected_polygon_data?
                                },
                                editor_state,
                                "width".to_string(),
                                ObjectType::TextItem,
                            )
                            .style(move |s| s.width(halfs).margin_right(5.0)),
                            debounce_input(
                                "Height:".to_string(),
                                &selected_text_data.read().borrow().dimensions.1.to_string(),
                                "Enter height",
                                move |value| {
                                    let mut editor_state = editor_state8.lock().unwrap();

                                    editor_state
                                        .update_height(&value, ObjectType::TextItem)
                                        .expect("Couldn't update height");

                                    drop(editor_state);
                                },
                                editor_state2,
                                "height".to_string(),
                                ObjectType::TextItem,
                            )
                            .style(move |s| s.width(halfs)),
                        )),))
                        .style(move |s| s.width(aside_width)),
                        v_stack((
                            debounce_input(
                                "Font Size:".to_string(),
                                &selected_text_data.read().borrow().font_size.to_string(),
                                "Enter size",
                                move |value| {
                                    // TODO: wrap up in editor_state for undo/redo

                                    let value =
                                        string_to_f32(&value).expect("Couldn't convert string");

                                    let mut editor = editor_cloned3.lock().unwrap();

                                    editor.update_text_size(selected_text_id.get(), value as i32);

                                    drop(editor);

                                    let mut editor_state = editor_state5.lock().unwrap();

                                    let mut saved_state = editor_state
                                        .record_state
                                        .saved_state
                                        .as_mut()
                                        .expect("Couldn't get Saved State");

                                    saved_state.sequences.iter_mut().for_each(|s| {
                                        if s.id == selected_sequence_id.get() {
                                            s.active_text_items.iter_mut().for_each(|p| {
                                                if p.id == selected_text_id.get().to_string() {
                                                    p.font_size = value as i32;
                                                }
                                            });
                                        }
                                    });

                                    save_saved_state_raw(saved_state.clone());

                                    drop(editor_state);
                                },
                                editor_state6,
                                "font_size".to_string(),
                                ObjectType::TextItem,
                            )
                            .style(move |s| s.width(260.0)),
                            inline_dropdown(
                                "Select a font family".to_string(),
                                selected_font_family,
                                font_dropdown_options,
                                on_font_selection,
                            ),
                            rgb_view_debounced(on_color_update, init_red, init_green, init_blue),
                        )),
                    ))
                } else {
                    v_stack((empty(),))
                }
            },
        ),
    ))
    // .style(|s| card_styles(s))
    .style(|s| {
        s.width(260.0)
            .height(800.0)
            .margin_left(0.0)
            .margin_top(20)
            .z_index(10)
    })
}

pub fn image_properties_view(
    editor_state: Arc<Mutex<EditorState>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    editor: std::sync::Arc<Mutex<Editor>>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
    image_selected: RwSignal<bool>,
    selected_image_id: RwSignal<Uuid>,
    selected_image_data: RwSignal<StImageConfig>,
    selected_sequence_id: RwSignal<String>,
) -> impl IntoView {
    let editor_cloned = Arc::clone(&editor);
    let editor_state2 = Arc::clone(&editor_state);
    let editor_state3 = Arc::clone(&editor_state);
    let editor_state4 = Arc::clone(&editor_state);
    // let editor_state3 = Arc::clone(&editor_state);
    // let editor_state4 = Arc::clone(&editor_state);
    // let editor_state5 = Arc::clone(&editor_state);
    // let editor_state6 = Arc::clone(&editor_state);
    // let editor_state7 = Arc::clone(&editor_state);
    // let editor_state8 = Arc::clone(&editor_state);
    // let editor_state9 = Arc::clone(&editor_state);
    // let editor_state10 = Arc::clone(&editor_state);

    let aside_width = 260.0;
    let quarters = (aside_width / 4.0) + (5.0 * 4.0);
    let thirds = (aside_width / 3.0) + (5.0 * 3.0);
    let halfs = (aside_width / 2.0) + (5.0 * 2.0);

    let back_active = RwSignal::new(false);

    v_stack((
        // label(|| "Properties"),
        simple_button("Back to Sequence".to_string(), move |_| {
            image_selected.set(false);
        }),
        v_stack((h_stack((
            debounce_input(
                "Width:".to_string(),
                &selected_image_data.read().borrow().dimensions.0.to_string(),
                "Enter width",
                move |value| {
                    let mut editor_state = editor_state3.lock().unwrap();

                    // NOTE: editor_state actions are hooked into undo/redo as well as file save
                    editor_state
                        .update_width(&value, ObjectType::ImageItem)
                        .expect("Couldn't update width");

                    drop(editor_state);

                    // TODO: should update selected_polygon_data?
                },
                editor_state,
                "width".to_string(),
                ObjectType::ImageItem,
            )
            .style(move |s| s.width(halfs).margin_right(5.0)),
            debounce_input(
                "Height:".to_string(),
                &selected_image_data.read().borrow().dimensions.1.to_string(),
                "Enter height",
                move |value| {
                    let mut editor_state = editor_state4.lock().unwrap();

                    editor_state
                        .update_height(&value, ObjectType::ImageItem)
                        .expect("Couldn't update height");

                    drop(editor_state);
                },
                editor_state2,
                "height".to_string(),
                ObjectType::ImageItem,
            )
            .style(move |s| s.width(halfs)),
        )),))
        .style(move |s| s.width(aside_width)),
    ))
    // .style(|s| card_styles(s))
    .style(|s| {
        s.width(260.0)
            .height(800.0)
            .margin_left(0.0)
            .margin_top(20)
            .z_index(10)
    })
}
