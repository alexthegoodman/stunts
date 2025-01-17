use floem::common::card_styles;
use floem::common::simple_button;
use floem::common::small_button;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
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

use super::inputs::create_dropdown;
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
                styled_input(
                    "Width:".to_string(),
                    &selected_polygon_data
                        .read()
                        .borrow()
                        .dimensions
                        .0
                        .to_string(),
                    "Enter width",
                    Box::new({
                        move |mut editor_state, value| {
                            editor_state
                                .update_width(&value)
                                .expect("Couldn't update width");
                            // TODO: probably should update selected_polygon_data
                            // need to update active_polygons in saved_data
                            // TODO: on_debounce_stop?
                            let value = string_to_f32(&value).expect("Couldn't convert string");
                            let mut saved_state = editor_state
                                .saved_state
                                .as_mut()
                                .expect("Couldn't get Saved State");

                            saved_state.sequences.iter_mut().for_each(|s| {
                                if s.id == selected_sequence_id.get() {
                                    s.active_polygons.iter_mut().for_each(|p| {
                                        if p.id == selected_polygon_id.get().to_string() {
                                            p.dimensions = (value as i32, p.dimensions.1);
                                        }
                                    });
                                }
                            });

                            save_saved_state_raw(saved_state.clone());
                        }
                    }),
                    editor_state,
                    "width".to_string(),
                )
                .style(move |s| s.width(halfs).margin_right(5.0)),
                styled_input(
                    "Height:".to_string(),
                    &selected_polygon_data
                        .read()
                        .borrow()
                        .dimensions
                        .1
                        .to_string(),
                    "Enter height",
                    Box::new({
                        move |mut editor_state, value| {
                            editor_state
                                .update_height(&value)
                                .expect("Couldn't update height");
                            // TODO: probably should update selected_polygon_data
                            // need to update active_polygons in saved_data
                            // TODO: on_debounce_stop?
                            let value = string_to_f32(&value).expect("Couldn't convert string");
                            let mut saved_state = editor_state
                                .saved_state
                                .as_mut()
                                .expect("Couldn't get Saved State");

                            saved_state.sequences.iter_mut().for_each(|s| {
                                if s.id == selected_sequence_id.get() {
                                    s.active_polygons.iter_mut().for_each(|p| {
                                        if p.id == selected_polygon_id.get().to_string() {
                                            p.dimensions = (p.dimensions.0, value as i32);
                                        }
                                    });
                                }
                            });

                            save_saved_state_raw(saved_state.clone());

                            // TODO: editor_state.update_width?
                        }
                    }),
                    editor_state2,
                    "height".to_string(),
                )
                .style(move |s| s.width(halfs)),
            )),
            h_stack((
                styled_input(
                    "Red:".to_string(),
                    &wgpu_to_human(selected_polygon_data.read().borrow().fill[0]).to_string(),
                    "0-255",
                    Box::new({
                        move |mut editor_state, value| {
                            editor_state.update_red(&value);
                            // TODO: update saved state?
                        }
                    }),
                    editor_state3,
                    "red".to_string(),
                )
                .style(move |s| s.width(thirds).margin_right(5.0)),
                styled_input(
                    "Green:".to_string(),
                    &wgpu_to_human(selected_polygon_data.read().borrow().fill[1]).to_string(),
                    "0-255",
                    Box::new({
                        move |mut editor_state, value| {
                            editor_state.update_green(&value);
                        }
                    }),
                    editor_state4,
                    "green".to_string(),
                )
                .style(move |s| s.width(thirds).margin_right(5.0)),
                styled_input(
                    "Blue:".to_string(),
                    &wgpu_to_human(selected_polygon_data.read().borrow().fill[2]).to_string(),
                    "0-255",
                    Box::new({
                        move |mut editor_state, value| {
                            editor_state.update_blue(&value);
                        }
                    }),
                    editor_state5,
                    "blue".to_string(),
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
            styled_input(
                "Border Radius:".to_string(),
                &selected_polygon_data
                    .read()
                    .borrow()
                    .border_radius
                    .to_string(),
                "Enter radius",
                Box::new({
                    move |mut editor_state, value| {
                        editor_state.update_border_radius(&value);
                    }
                }),
                editor_state6,
                "border_radius".to_string(),
            ),
            label(|| "Stroke").style(|s| s.margin_bottom(5.0)),
            h_stack((
                styled_input(
                    "Thickness:".to_string(),
                    &selected_polygon_data
                        .read()
                        .borrow()
                        .stroke
                        .thickness
                        .to_string(),
                    "Enter thickness",
                    Box::new({
                        move |mut editor_state, value| {
                            editor_state.update_stroke_thickness(&value);
                        }
                    }),
                    editor_state7,
                    "stroke_thickness".to_string(),
                )
                .style(move |s| s.width(quarters).margin_right(5.0)),
                styled_input(
                    "Red:".to_string(),
                    &wgpu_to_human(selected_polygon_data.read().borrow().stroke.fill[0])
                        .to_string(),
                    "Enter red",
                    Box::new({
                        move |mut editor_state, value| {
                            editor_state.update_stroke_red(&value);
                        }
                    }),
                    editor_state10,
                    "stroke_red".to_string(),
                )
                .style(move |s| s.width(quarters).margin_right(5.0)),
                styled_input(
                    "Green:".to_string(),
                    &wgpu_to_human(selected_polygon_data.read().borrow().stroke.fill[1])
                        .to_string(),
                    "Enter green",
                    Box::new({
                        move |mut editor_state, value| {
                            editor_state.update_stroke_green(&value);
                        }
                    }),
                    editor_state8,
                    "stroke_green".to_string(),
                )
                .style(move |s| s.width(quarters).margin_right(5.0)),
                styled_input(
                    "Blue:".to_string(),
                    &wgpu_to_human(selected_polygon_data.read().borrow().stroke.fill[2])
                        .to_string(),
                    "Enter blue",
                    Box::new({
                        move |mut editor_state, value| {
                            editor_state.update_stroke_blue(&value);
                        }
                    }),
                    editor_state9,
                    "stroke_blue".to_string(),
                )
                .style(move |s| s.width(quarters)),
            )),
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
    let editor_state2 = Arc::clone(&editor_state);
    let editor_state3 = Arc::clone(&editor_state);
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
    let font_dropdown_options: RwSignal<Vec<DropdownOption>> = create_rw_signal(Vec::new());

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

    let on_font_selection = move |font_id: String| {
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

        editor_state.saved_state = Some(saved_state.clone());

        drop(editor_state);
    };

    v_stack((
        // label(|| "Properties"),
        simple_button("Back to Sequence".to_string(), move |_| {
            text_selected.set(false);
        }),
        v_stack((h_stack((
            styled_input(
                "Width:".to_string(),
                &selected_text_data.read().borrow().dimensions.0.to_string(),
                "Enter width",
                Box::new({
                    move |mut editor_state, value| {
                        editor_state
                            .update_width(&value)
                            .expect("Couldn't update width");
                        // TODO: probably should update selected_polygon_data
                        // need to update active_polygons in saved_data
                        // TODO: on_debounce_stop?
                        let value = string_to_f32(&value).expect("Couldn't convert string");
                        let mut saved_state = editor_state
                            .saved_state
                            .as_mut()
                            .expect("Couldn't get Saved State");

                        saved_state.sequences.iter_mut().for_each(|s| {
                            if s.id == selected_sequence_id.get() {
                                s.active_text_items.iter_mut().for_each(|p| {
                                    if p.id == selected_text_id.get().to_string() {
                                        p.dimensions = (value as i32, p.dimensions.1);
                                    }
                                });
                            }
                        });

                        save_saved_state_raw(saved_state.clone());

                        // TODO: editor_state.update_height?
                    }
                }),
                editor_state,
                "width".to_string(),
            )
            .style(move |s| s.width(halfs).margin_right(5.0)),
            styled_input(
                "Height:".to_string(),
                &selected_text_data.read().borrow().dimensions.1.to_string(),
                "Enter height",
                Box::new({
                    move |mut editor_state, value| {
                        editor_state
                            .update_height(&value)
                            .expect("Couldn't update height");
                        // TODO: probably should update selected_polygon_data
                        // need to update active_polygons in saved_data
                        // TODO: on_debounce_stop?
                        let value = string_to_f32(&value).expect("Couldn't convert string");
                        let mut saved_state = editor_state
                            .saved_state
                            .as_mut()
                            .expect("Couldn't get Saved State");

                        saved_state.sequences.iter_mut().for_each(|s| {
                            if s.id == selected_sequence_id.get() {
                                s.active_text_items.iter_mut().for_each(|p| {
                                    if p.id == selected_text_id.get().to_string() {
                                        p.dimensions = (p.dimensions.0, value as i32);
                                    }
                                });
                            }
                        });

                        save_saved_state_raw(saved_state.clone());

                        // TODO: editor_state.update_width?
                    }
                }),
                editor_state2,
                "height".to_string(),
            )
            .style(move |s| s.width(halfs)),
            h_stack((create_dropdown(
                selected_text_data.get().font_family.clone(),
                font_dropdown_options.get(),
                on_font_selection,
            ),)),
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
            styled_input(
                "Width:".to_string(),
                &selected_image_data.read().borrow().dimensions.0.to_string(),
                "Enter width",
                Box::new({
                    move |mut editor_state, value| {
                        editor_state
                            .update_width(&value)
                            .expect("Couldn't update width");
                        // TODO: probably should update selected_polygon_data
                        // need to update active_polygons in saved_data
                        // TODO: on_debounce_stop?
                        let value = string_to_f32(&value).expect("Couldn't convert string");
                        let mut saved_state = editor_state
                            .saved_state
                            .as_mut()
                            .expect("Couldn't get Saved State");

                        saved_state.sequences.iter_mut().for_each(|s| {
                            if s.id == selected_sequence_id.get() {
                                s.active_image_items.iter_mut().for_each(|p| {
                                    if p.id == selected_image_id.get().to_string() {
                                        p.dimensions = (value as u32, p.dimensions.1);
                                    }
                                });
                            }
                        });

                        save_saved_state_raw(saved_state.clone());

                        // TODO: editor_state.update_height?
                    }
                }),
                editor_state,
                "width".to_string(),
            )
            .style(move |s| s.width(halfs).margin_right(5.0)),
            styled_input(
                "Height:".to_string(),
                &selected_image_data.read().borrow().dimensions.1.to_string(),
                "Enter height",
                Box::new({
                    move |mut editor_state, value| {
                        editor_state
                            .update_height(&value)
                            .expect("Couldn't update height");
                        // TODO: probably should update selected_polygon_data
                        // need to update active_polygons in saved_data
                        // TODO: on_debounce_stop?
                        let value = string_to_f32(&value).expect("Couldn't convert string");
                        let mut saved_state = editor_state
                            .saved_state
                            .as_mut()
                            .expect("Couldn't get Saved State");

                        saved_state.sequences.iter_mut().for_each(|s| {
                            if s.id == selected_sequence_id.get() {
                                s.active_image_items.iter_mut().for_each(|p| {
                                    if p.id == selected_image_id.get().to_string() {
                                        p.dimensions = (p.dimensions.0, value as u32);
                                    }
                                });
                            }
                        });

                        save_saved_state_raw(saved_state.clone());

                        // TODO: editor_state.update_width?
                    }
                }),
                editor_state2,
                "height".to_string(),
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
