use floem::common::card_styles;
use floem::common::simple_button;
use floem::common::small_button;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;
use stunts_engine::animations::ObjectType;
use stunts_engine::animations::Sequence;
use stunts_engine::editor::color_to_wgpu;
use stunts_engine::editor::string_to_f32;
use stunts_engine::editor::wgpu_to_human;
use stunts_engine::editor::Editor;
use stunts_engine::editor::Viewport;
use stunts_engine::polygon::PolygonConfig;
use stunts_engine::st_image::StImageConfig;
use stunts_engine::st_video::StVideoConfig;
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

pub fn keyframe_tools(
    editor: Arc<Mutex<Editor>>,
    editor_state: Arc<Mutex<EditorState>>,
    selected_polygon_id: RwSignal<Uuid>,
    selected_sequence_id: RwSignal<String>,
    selected_sequence_data: RwSignal<Sequence>,
    object_type: ObjectType,
) -> impl IntoView {
    let editor_2 = Arc::clone(&editor);
    let editor_state2 = Arc::clone(&editor_state);
    let editor_state3 = Arc::clone(&editor_state);
    let editor_state4 = Arc::clone(&editor_state);
    let editor_state5 = Arc::clone(&editor_state);
    let editor_state6 = Arc::clone(&editor_state);

    let sequence_duration_input = create_rw_signal(String::new());
    let target_duration_signal = create_rw_signal(String::new());

    create_effect(move |_| {
        let sequence_data = selected_sequence_data.get();
        let current_duration = sequence_data
            .polygon_motion_paths
            .iter()
            .find(|pm| pm.polygon_id == selected_polygon_id.get().to_string())
            .map(|pm| pm.duration);

        if let Some(current_duration) = current_duration {
            sequence_duration_input.set(current_duration.as_secs().to_string());
        }
    });

    v_stack((
        label(|| "Update Keyframes").style(|s| s.margin_bottom(5.0)),
        v_stack((simple_button("Reverse Keyframes".to_string(), move |_| {
            let mut editor_state = editor_state6.lock().unwrap();

            let mut new_sequence = selected_sequence_data.get();

            for (i, animation) in new_sequence.polygon_motion_paths.iter_mut().enumerate() {
                if animation.polygon_id == selected_polygon_id.get().to_string() {
                    let target_animation = editor_state.reverse_keyframes(animation.clone());
                    *animation = target_animation.clone();
                }
            }

            selected_sequence_data.set(new_sequence.clone());

            let mut saved_state = editor_state
                .record_state
                .saved_state
                .as_mut()
                .expect("Couldn't get Saved State");

            saved_state.sequences.iter_mut().for_each(|s| {
                if s.id == selected_sequence_id.get() {
                    *s = new_sequence.clone()
                }
            });

            save_saved_state_raw(saved_state.clone());

            editor_state.record_state.saved_state = Some(saved_state.clone());

            drop(editor_state);

            let mut editor = editor_2.lock().unwrap();

            editor.update_motion_paths(&new_sequence);

            drop(editor);
        }),)),
        v_stack((
            debounce_input(
                "Target Duration".to_string(),
                &sequence_duration_input.get(),
                "Seconds",
                move |target_dur| {
                    target_duration_signal.set(target_dur);
                },
                editor_state4,
                "target_duration".to_string(),
                object_type,
            ),
            h_stack((
                simple_button("Shrink / Stretch".to_string(), move |_| {
                    // TODO: integrate with undo/redo
                    if target_duration_signal.get().len() < 1 {
                        return;
                    }

                    let mut editor_state = editor_state2.lock().unwrap();
                    let saved_state = editor_state
                        .record_state
                        .saved_state
                        .as_mut()
                        .expect("Couldn't get saved state");
                    let selected_sequence = saved_state
                        .sequences
                        .iter()
                        .find(|s| s.id == selected_sequence_id.get())
                        .expect("Couldn't get selected sequence");
                    let current_animation_path = selected_sequence
                        .polygon_motion_paths
                        .iter()
                        .find(|p| p.polygon_id == selected_polygon_id.get().to_string())
                        .expect("Couldn't find matching polygon");

                    let target_duration = string_to_f32(&target_duration_signal.get())
                        .expect("Couldn't get duration");

                    let motion_path = current_animation_path.clone();

                    drop(editor_state);

                    let mut editor_state = editor_state2.lock().unwrap();

                    let target_keyframes = editor_state
                        .scale_animation(motion_path, Duration::from_secs_f32(target_duration));

                    let saved_state = editor_state
                        .record_state
                        .saved_state
                        .as_mut()
                        .expect("Couldn't get saved state");

                    let mut new_sequence = selected_sequence_data.get();
                    let polygon_path = new_sequence
                        .polygon_motion_paths
                        .iter_mut()
                        .find(|pm| pm.polygon_id == selected_polygon_id.get().to_string())
                        .expect("Couldn't find amtching polygon path");
                    *polygon_path = target_keyframes.clone();

                    selected_sequence_data.set(new_sequence);

                    saved_state.sequences.iter_mut().for_each(|s| {
                        if s.id == selected_sequence_id.get() {
                            // s.duration_ms = target_duration as i32 * 1000; // no
                            let polygon_path = s
                                .polygon_motion_paths
                                .iter_mut()
                                .find(|pm| pm.polygon_id == selected_polygon_id.get().to_string())
                                .expect("Couldn't find amtching polygon path");
                            *polygon_path = target_keyframes.clone();
                        }
                    });

                    save_saved_state_raw(saved_state.clone());

                    editor_state.record_state.saved_state = Some(saved_state.clone());

                    drop(editor_state);
                }),
                simple_button("Cut / Extend".to_string(), move |_| {
                    if target_duration_signal.get().len() < 1 {
                        return;
                    }

                    // TODO: needs to actually cut off keyframes? and update motion paths?

                    let mut editor_state = editor_state3.lock().unwrap();

                    let target_duration = string_to_f32(&target_duration_signal.get())
                        .expect("Couldn't get duration");

                    let mut saved_state = editor_state
                        .record_state
                        .saved_state
                        .as_mut()
                        .expect("Couldn't get Saved State");

                    saved_state
                        .sequences
                        .iter_mut()
                        .filter(|s| s.id == selected_sequence_id.get())
                        .for_each(|s| {
                            s.duration_ms = target_duration as i32 * 1000;
                            // s.polygon_motion_paths.iter_mut().for_each(|pm| {
                            //     pm.duration = Duration::from_secs(target_duration as u64);
                            // });
                        });

                    save_saved_state_raw(saved_state.clone());

                    editor_state.record_state.saved_state = Some(saved_state.clone());

                    drop(editor_state);
                }),
            )),
        ))
        .style(|s| s.margin_bottom(5.0)),
    ))
}

pub fn properties_view(
    editor_state: Arc<Mutex<EditorState>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    editor: std::sync::Arc<Mutex<Editor>>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
    polygon_selected: RwSignal<bool>,
    selected_polygon_id: RwSignal<Uuid>,
    selected_polygon_data: RwSignal<PolygonConfig>,
    selected_sequence_id: RwSignal<String>,
    selected_sequence_data: RwSignal<Sequence>,
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
    let editor_state21 = Arc::clone(&editor_state);
    let editor_state22 = Arc::clone(&editor_state);
    let editor_state23 = Arc::clone(&editor_state);

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
            keyframe_tools(
                editor_cloned,
                editor_state22,
                selected_polygon_id,
                selected_sequence_id,
                selected_sequence_data,
                ObjectType::Polygon,
            ),
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
    selected_sequence_data: RwSignal<Sequence>,
) -> impl IntoView {
    let editor_cloned = Arc::clone(&editor);
    let editor_cloned2 = Arc::clone(&editor);
    let editor_cloned3 = Arc::clone(&editor);
    let editor_cloned4 = Arc::clone(&editor);
    let editor_cloned5 = Arc::clone(&editor);
    let editor_cloned6 = Arc::clone(&editor);
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
                let editor_cloned6 = editor_cloned6.clone();
                let editor_state = editor_state.clone();
                let editor_state2 = editor_state2.clone();
                let editor_state5 = editor_state5.clone();
                let editor_state6 = editor_state6.clone();
                let editor_cloned3 = editor_cloned3.clone();
                let editor_cloned4 = editor_cloned4.clone();
                let editor_state7 = editor_state7.clone();
                let editor_state8 = editor_state8.clone();
                let editor_state9 = editor_state9.clone();
                let editor_state10 = editor_state10.clone();
                let editor_state11 = editor_state11.clone();
                let editor_state12 = editor_state12.clone();
                let editor_state13 = editor_state13.clone();
                let editor_state14 = editor_state14.clone();
                let editor_state15 = editor_state15.clone();
                let editor_state16 = editor_state16.clone();
                let editor_state17 = editor_state17.clone();

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
                                "Text Content:".to_string(),
                                &selected_text_data.read().borrow().text.clone(),
                                "Enter content",
                                move |value| {
                                    // TODO: wrap up in editor_state for undo/redo

                                    let mut editor = editor_cloned4.lock().unwrap();

                                    editor
                                        .update_text_content(selected_text_id.get(), value.clone());

                                    drop(editor);

                                    let mut editor_state = editor_state10.lock().unwrap();

                                    let mut saved_state = editor_state
                                        .record_state
                                        .saved_state
                                        .as_mut()
                                        .expect("Couldn't get Saved State");

                                    saved_state.sequences.iter_mut().for_each(|s| {
                                        if s.id == selected_sequence_id.get() {
                                            s.active_text_items.iter_mut().for_each(|p| {
                                                if p.id == selected_text_id.get().to_string() {
                                                    p.text = value.clone();
                                                }
                                            });
                                        }
                                    });

                                    save_saved_state_raw(saved_state.clone());

                                    drop(editor_state);
                                },
                                editor_state11,
                                "text_content".to_string(),
                                ObjectType::TextItem,
                            )
                            .style(move |s| s.width(260.0)),
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
                        v_stack((
                            label(|| "Background Color").style(|s| s.margin_bottom(5.0)),
                            h_stack((
                                debounce_input(
                                    "Red:".to_string(),
                                    &selected_text_data.read().borrow().background_fill[0]
                                        .to_string(),
                                    "0-255",
                                    move |value| {
                                        let mut editor_state = editor_state12.lock().unwrap();

                                        editor_state
                                            .update_fill_red(&value, selected_sequence_id.get())
                                            .expect("Couldn't update red");

                                        drop(editor_state);
                                    },
                                    editor_state13,
                                    "red_fill".to_string(),
                                    ObjectType::TextItem,
                                )
                                .style(move |s| s.width(thirds).margin_right(5.0)),
                                debounce_input(
                                    "Green:".to_string(),
                                    &selected_text_data.read().borrow().background_fill[1]
                                        .to_string(),
                                    "0-255",
                                    move |value| {
                                        let mut editor_state = editor_state14.lock().unwrap();

                                        editor_state
                                            .update_fill_green(&value, selected_sequence_id.get())
                                            .expect("Couldn't update green");

                                        drop(editor_state);
                                    },
                                    editor_state15,
                                    "green_fill".to_string(),
                                    ObjectType::TextItem,
                                )
                                .style(move |s| s.width(thirds).margin_right(5.0)),
                                debounce_input(
                                    "Blue:".to_string(),
                                    &selected_text_data.read().borrow().background_fill[2]
                                        .to_string(),
                                    "0-255",
                                    move |value| {
                                        let mut editor_state = editor_state16.lock().unwrap();

                                        editor_state
                                            .update_fill_blue(&value, selected_sequence_id.get())
                                            .expect("Couldn't update blue");

                                        drop(editor_state);
                                    },
                                    editor_state17,
                                    "blue_fill".to_string(),
                                    ObjectType::TextItem,
                                )
                                .style(move |s| s.width(thirds)),
                            ))
                            .style(move |s| s.width(aside_width)),
                        )),
                        keyframe_tools(
                            editor_cloned6,
                            editor_state9,
                            selected_text_id,
                            selected_sequence_id,
                            selected_sequence_data,
                            ObjectType::TextItem,
                        ),
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
    selected_sequence_data: RwSignal<Sequence>,
) -> impl IntoView {
    let editor_cloned = Arc::clone(&editor);
    let editor_state2 = Arc::clone(&editor_state);
    let editor_state3 = Arc::clone(&editor_state);
    let editor_state4 = Arc::clone(&editor_state);
    let editor_state5 = Arc::clone(&editor_state);

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
        v_stack((
            h_stack((
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
            )),
            keyframe_tools(
                editor_cloned,
                editor_state5,
                selected_image_id,
                selected_sequence_id,
                selected_sequence_data,
                ObjectType::ImageItem,
            ),
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

pub fn video_properties_view(
    editor_state: Arc<Mutex<EditorState>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    editor: std::sync::Arc<Mutex<Editor>>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
    video_selected: RwSignal<bool>,
    selected_video_id: RwSignal<Uuid>,
    selected_video_data: RwSignal<StVideoConfig>,
    selected_sequence_id: RwSignal<String>,
    selected_sequence_data: RwSignal<Sequence>,
) -> impl IntoView {
    let editor_cloned = Arc::clone(&editor);
    let editor_state2 = Arc::clone(&editor_state);
    let editor_state3 = Arc::clone(&editor_state);
    let editor_state4 = Arc::clone(&editor_state);
    let editor_state5 = Arc::clone(&editor_state);

    let aside_width = 260.0;
    let quarters = (aside_width / 4.0) + (5.0 * 4.0);
    let thirds = (aside_width / 3.0) + (5.0 * 3.0);
    let halfs = (aside_width / 2.0) + (5.0 * 2.0);

    let back_active = RwSignal::new(false);

    v_stack((
        // label(|| "Properties"),
        simple_button("Back to Sequence".to_string(), move |_| {
            video_selected.set(false);
        }),
        v_stack((
            h_stack((
                debounce_input(
                    "Width:".to_string(),
                    &selected_video_data.read().borrow().dimensions.0.to_string(),
                    "Enter width",
                    move |value| {
                        let mut editor_state = editor_state3.lock().unwrap();

                        // NOTE: editor_state actions are hooked into undo/redo as well as file save
                        editor_state
                            .update_width(&value, ObjectType::VideoItem)
                            .expect("Couldn't update width");

                        drop(editor_state);

                        // TODO: should update selected_polygon_data?
                    },
                    editor_state,
                    "width".to_string(),
                    ObjectType::VideoItem,
                )
                .style(move |s| s.width(halfs).margin_right(5.0)),
                debounce_input(
                    "Height:".to_string(),
                    &selected_video_data.read().borrow().dimensions.1.to_string(),
                    "Enter height",
                    move |value| {
                        let mut editor_state = editor_state4.lock().unwrap();

                        editor_state
                            .update_height(&value, ObjectType::VideoItem)
                            .expect("Couldn't update height");

                        drop(editor_state);
                    },
                    editor_state2,
                    "height".to_string(),
                    ObjectType::VideoItem,
                )
                .style(move |s| s.width(halfs)),
            )),
            keyframe_tools(
                editor_cloned,
                editor_state5,
                selected_video_id,
                selected_sequence_id,
                selected_sequence_data,
                ObjectType::VideoItem,
            ),
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
