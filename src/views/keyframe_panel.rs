use floem::common::card_styles;
use floem::common::simple_button;
use floem::common::small_button;
use floem::views::Checkbox;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use stunts_engine::animations::AnimationData;
use stunts_engine::animations::KeyframeValue;
use stunts_engine::animations::ObjectType;
use stunts_engine::animations::Sequence;
use stunts_engine::animations::UIKeyframe;
use stunts_engine::editor::string_to_f32;
use stunts_engine::editor::ControlPoint;
use stunts_engine::editor::CurveData;
use stunts_engine::editor::Editor;
use stunts_engine::editor::PathType;
use stunts_engine::editor::Viewport;
use stunts_engine::polygon::PolygonConfig;
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

use super::inputs::debounce_input;
use super::inputs::styled_input;

pub fn update_keyframe(
    mut editor_state: MutexGuard<EditorState>,
    mut current_animation_data: AnimationData,
    mut current_keyframe: &mut UIKeyframe,
    mut current_sequence: Sequence,
    selected_keyframes: RwSignal<Vec<UIKeyframe>>,
    animation_data: RwSignal<Option<AnimationData>>,
    selected_sequence_data: RwSignal<Sequence>,
    selected_sequence_id: RwSignal<String>,
    sequence_selected: RwSignal<bool>,
) {
    if let Some(selected_keyframe) = selected_keyframes.get().get(0) {
        if current_keyframe.id != selected_keyframe.id {
            let mut new_keyframes = Vec::new();
            new_keyframes.push(current_keyframe.to_owned());

            selected_keyframes.set(new_keyframes);
        }
    } else {
        let mut new_keyframes = Vec::new();
        new_keyframes.push(current_keyframe.to_owned());

        selected_keyframes.set(new_keyframes);
    }

    // update animation data
    current_animation_data.properties.iter_mut().for_each(|p| {
        p.keyframes.iter_mut().for_each(|mut k| {
            if k.id == current_keyframe.id {
                *k = current_keyframe.to_owned();
            }
        });
    });

    animation_data.set(Some(current_animation_data));

    // update sequence
    current_sequence
        .polygon_motion_paths
        .iter_mut()
        .for_each(|pm| {
            pm.properties.iter_mut().for_each(|p| {
                p.keyframes.iter_mut().for_each(|k| {
                    if k.id == current_keyframe.id {
                        *k = current_keyframe.to_owned();
                    }
                });
            });
        });

    selected_sequence_data.set(current_sequence);

    // sequence_selected.set(true);

    // save to file
    let last_saved_state = editor_state
        .record_state
        .saved_state
        .as_mut()
        .expect("Couldn't get Saved State");

    last_saved_state.sequences.iter_mut().for_each(|s| {
        if s.id == selected_sequence_id.get() {
            s.polygon_motion_paths.iter_mut().for_each(|pm| {
                pm.properties.iter_mut().for_each(|p| {
                    p.keyframes.iter_mut().for_each(|k| {
                        if k.id == current_keyframe.id {
                            *k = current_keyframe.to_owned();
                        }
                    });
                });
            });
        }
    });

    // TODO: probably perf hit with larger files, or does it get released?
    let new_saved_state = last_saved_state.to_owned();

    save_saved_state_raw(new_saved_state);
}

pub fn keyframe_properties_view(
    editor_state: Arc<Mutex<EditorState>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    editor: std::sync::Arc<Mutex<Editor>>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
    polygon_selected: RwSignal<bool>,
    selected_polygon_id: RwSignal<Uuid>,
    selected_polygon_data: RwSignal<PolygonConfig>,
    selected_sequence_id: RwSignal<String>,
    selected_keyframe: &UIKeyframe,
    selected_keyframes: RwSignal<Vec<UIKeyframe>>,
    animation_data: RwSignal<Option<AnimationData>>,
    sequence_selected: RwSignal<bool>,
    selected_sequence_data: RwSignal<Sequence>,
    object_type: ObjectType,
) -> impl IntoView {
    let editor_cloned = Arc::clone(&editor);
    let editor_cloned2 = Arc::clone(&editor);
    let editor_cloned3 = Arc::clone(&editor);
    let editor_cloned4 = Arc::clone(&editor);
    let editor_cloned5 = Arc::clone(&editor);
    let editor_cloned6 = Arc::clone(&editor);
    let editor_cloned7 = Arc::clone(&editor);
    let editor_state_cloned = Arc::clone(&editor_state);
    let editor_state_cloned2 = Arc::clone(&editor_state);
    let editor_state_cloned3 = Arc::clone(&editor_state);
    let editor_state_cloned4 = Arc::clone(&editor_state);
    let editor_state_cloned5 = Arc::clone(&editor_state);
    let editor_state_cloned6 = Arc::clone(&editor_state);
    let editor_state_cloned7 = Arc::clone(&editor_state);
    let editor_state_cloned8 = Arc::clone(&editor_state);
    let editor_state_cloned9 = Arc::clone(&editor_state);
    let editor_state_cloned10 = Arc::clone(&editor_state);
    let editor_state_cloned11 = Arc::clone(&editor_state);
    let editor_state_cloned12 = Arc::clone(&editor_state);
    let editor_state_cloned13 = Arc::clone(&editor_state);
    let editor_state_cloned14 = Arc::clone(&editor_state);
    let editor_state_cloned15 = Arc::clone(&editor_state);
    let editor_state_cloned16 = Arc::clone(&editor_state);
    let editor_state_cloned17 = Arc::clone(&editor_state);
    let editor_state_cloned18 = Arc::clone(&editor_state);
    let editor_state_cloned19 = Arc::clone(&editor_state);
    let editor_state_cloned20 = Arc::clone(&editor_state);

    let aside_width = 260.0;
    let quarters = (aside_width / 4.0) + (5.0 * 4.0);
    let thirds = (aside_width / 3.0) + (5.0 * 3.0);
    let halfs = (aside_width / 2.0) + (5.0 * 2.0);

    let back_active = RwSignal::new(false);
    let curve_is_checked = RwSignal::new(false);

    let selected_p_type = selected_keyframe.path_type.clone();

    let selected_path_type = create_rw_signal(selected_p_type);

    create_effect({
        move |_| {
            let is_set = match selected_path_type.get_untracked() {
                PathType::Linear => false,
                PathType::Bezier(curve_data) => true,
            };

            curve_is_checked.set(is_set);
        }
    });

    let object_type_cloned = object_type.clone();

    v_stack((
        match selected_keyframe.value {
            KeyframeValue::Position(position) => container(
                (v_stack((
                    label(|| "Keyframe").style(|s| s.font_size(14.0).margin_bottom(10)),
                    simple_button("Back to Properties".to_string(), move |_| {
                        selected_keyframes.set(Vec::new());
                    })
                    .style(|s| s.margin_bottom(5.0)),
                    v_stack((
                        Checkbox::new_labeled_rw(curve_is_checked, || "Curved Path"),
                        dyn_container(
                            move || curve_is_checked.get(),
                            move |curve_checked| {
                                let editor_cloned = editor_cloned.clone();
                                let editor_cloned2 = editor_cloned2.clone();
                                let editor_cloned3 = editor_cloned3.clone();
                                let editor_cloned4 = editor_cloned4.clone();
                                let editor_state_cloned8 = editor_state_cloned8.clone();
                                let editor_state_cloned9 = editor_state_cloned9.clone();
                                let editor_state_cloned10 = editor_state_cloned10.clone();
                                let editor_state_cloned11 = editor_state_cloned11.clone();
                                let editor_state_cloned12 = editor_state_cloned12.clone();
                                let editor_state_cloned13 = editor_state_cloned13.clone();
                                let editor_state_cloned14 = editor_state_cloned14.clone();
                                let editor_state_cloned15 = editor_state_cloned15.clone();

                                let curve_data: Option<CurveData> = match selected_path_type.get() {
                                    PathType::Linear => None,
                                    PathType::Bezier(curve_data) => Some(curve_data),
                                };

                                let c1 = if let Some(data) = curve_data.clone() {
                                    data.control_point1.expect("Couldn't get control point 1")
                                } else {
                                    ControlPoint { x: 0, y: 0 }
                                };

                                let c2 = if let Some(data) = curve_data.clone() {
                                    data.control_point2.expect("Couldn't get control point 2")
                                } else {
                                    ControlPoint { x: 0, y: 0 }
                                };

                                if curve_checked {
                                    v_stack((
                                        label(|| "Curve Control Points")
                                            .style(|s| s.font_size(10.0).margin_bottom(2.0)),
                                        h_stack((
                                            debounce_input(
                                                "X 1:".to_string(),
                                                &c1.x.to_string(),
                                                "Enter X",
                                                move |value| {
                                                    // update animation_data, selected_polygon_data, and selected_keyframes, and selected_sequence_data,
                                                    // then save merge animation_data with saved_data and save to file
                                                    // although perhaps polygon_data is not related to the keyframe data? no need to update here?

                                                    let value = string_to_f32(&value)
                                                        .map_err(|_| {
                                                            "Couldn't convert string to f32"
                                                        })
                                                        .expect("Couldn't convert string to f32");

                                                    let mut current_animation_data = animation_data
                                                        .get()
                                                        .expect("Couldn't get Animation Data");
                                                    let mut current_keyframe =
                                                        selected_keyframes.get();
                                                    let mut current_keyframe = current_keyframe
                                                        .get_mut(0)
                                                        .expect("Couldn't get Selected Keyframe");
                                                    let mut current_sequence =
                                                        selected_sequence_data.get();

                                                    // update keyframe
                                                    let mut current_path_type =
                                                        selected_path_type.get();

                                                    match current_path_type {
                                                        PathType::Linear => {
                                                            current_path_type =
                                                                PathType::Bezier(CurveData {
                                                                    control_point1: Some(
                                                                        ControlPoint {
                                                                            x: value as i32,
                                                                            y: 0,
                                                                        },
                                                                    ),
                                                                    control_point2: Some(
                                                                        ControlPoint { x: 0, y: 0 },
                                                                    ),
                                                                });
                                                        }
                                                        PathType::Bezier(curve_data) => {
                                                            let c1 =
                                                                curve_data.control_point1.expect(
                                                                    "Couldn't get control point 1",
                                                                );
                                                            let c2 =
                                                                curve_data.control_point2.expect(
                                                                    "Couldn't get control point 2",
                                                                );

                                                            current_path_type =
                                                                PathType::Bezier(CurveData {
                                                                    control_point1: Some(
                                                                        ControlPoint {
                                                                            x: value as i32,
                                                                            y: c1.y,
                                                                        },
                                                                    ),
                                                                    control_point2: Some(
                                                                        ControlPoint {
                                                                            x: c2.x,
                                                                            y: c2.y,
                                                                        },
                                                                    ),
                                                                });
                                                        }
                                                    }

                                                    selected_path_type
                                                        .set(current_path_type.clone());

                                                    current_keyframe.path_type = current_path_type;

                                                    let editor_state =
                                                        editor_state_cloned8.lock().unwrap();

                                                    update_keyframe(
                                                        editor_state,
                                                        current_animation_data,
                                                        current_keyframe,
                                                        current_sequence,
                                                        selected_keyframes,
                                                        animation_data,
                                                        selected_sequence_data,
                                                        selected_sequence_id,
                                                        sequence_selected,
                                                    );

                                                    println!("Keyframe updated!");

                                                    let mut editor = editor_cloned.lock().unwrap();
                                                    editor.update_motion_paths(
                                                        &selected_sequence_data.get(),
                                                    );

                                                    println!("Motion Paths updated!");
                                                },
                                                editor_state_cloned9,
                                                "x".to_string(),
                                                object_type.clone(),
                                            )
                                            .style(move |s| s.width(halfs).margin_right(5.0)),
                                            debounce_input(
                                                "Y 1:".to_string(),
                                                &c1.y.to_string(),
                                                "Enter Y",
                                                move |value| {
                                                    let value = string_to_f32(&value)
                                                        .map_err(|_| {
                                                            "Couldn't convert string to f32"
                                                        })
                                                        .expect("Couldn't convert string to f32");

                                                    let mut current_animation_data = animation_data
                                                        .get()
                                                        .expect("Couldn't get Animation Data");
                                                    let mut current_keyframe =
                                                        selected_keyframes.get();
                                                    let mut current_keyframe = current_keyframe
                                                        .get_mut(0)
                                                        .expect("Couldn't get Selected Keyframe");
                                                    let mut current_sequence =
                                                        selected_sequence_data.get();

                                                    let mut current_path_type =
                                                        selected_path_type.get();

                                                    match current_path_type {
                                                        PathType::Linear => {
                                                            current_path_type =
                                                                PathType::Bezier(CurveData {
                                                                    control_point1: Some(
                                                                        ControlPoint {
                                                                            x: 0,
                                                                            y: value as i32,
                                                                        },
                                                                    ),
                                                                    control_point2: Some(
                                                                        ControlPoint { x: 0, y: 0 },
                                                                    ),
                                                                });
                                                        }
                                                        PathType::Bezier(curve_data) => {
                                                            let c1 =
                                                                curve_data.control_point1.expect(
                                                                    "Couldn't get control point 1",
                                                                );
                                                            let c2 =
                                                                curve_data.control_point2.expect(
                                                                    "Couldn't get control point 2",
                                                                );

                                                            current_path_type =
                                                                PathType::Bezier(CurveData {
                                                                    control_point1: Some(
                                                                        ControlPoint {
                                                                            x: c1.x,
                                                                            y: value as i32,
                                                                        },
                                                                    ),
                                                                    control_point2: Some(
                                                                        ControlPoint {
                                                                            x: c2.x,
                                                                            y: c2.y,
                                                                        },
                                                                    ),
                                                                });
                                                        }
                                                    }

                                                    selected_path_type
                                                        .set(current_path_type.clone());

                                                    current_keyframe.path_type = current_path_type;

                                                    let editor_state =
                                                        editor_state_cloned10.lock().unwrap();

                                                    update_keyframe(
                                                        editor_state,
                                                        current_animation_data,
                                                        current_keyframe,
                                                        current_sequence,
                                                        selected_keyframes,
                                                        animation_data,
                                                        selected_sequence_data,
                                                        selected_sequence_id,
                                                        sequence_selected,
                                                    );

                                                    println!("Keyframe updated!");

                                                    let mut editor = editor_cloned2.lock().unwrap();
                                                    editor.update_motion_paths(
                                                        &selected_sequence_data.get(),
                                                    );

                                                    println!("Motion Paths updated!");
                                                },
                                                editor_state_cloned11,
                                                "y".to_string(),
                                                object_type.clone(),
                                            )
                                            .style(move |s| s.width(halfs)),
                                        ))
                                        .style(move |s| s.width(aside_width)),
                                        h_stack((
                                            debounce_input(
                                                "X 2:".to_string(),
                                                &c2.x.to_string(),
                                                "Enter X",
                                                move |value| {
                                                    // update animation_data, selected_polygon_data, and selected_keyframes, and selected_sequence_data,
                                                    // then save merge animation_data with saved_data and save to file
                                                    // although perhaps polygon_data is not related to the keyframe data? no need to update here?

                                                    let value = string_to_f32(&value)
                                                        .map_err(|_| {
                                                            "Couldn't convert string to f32"
                                                        })
                                                        .expect("Couldn't convert string to f32");

                                                    let mut current_animation_data = animation_data
                                                        .get()
                                                        .expect("Couldn't get Animation Data");
                                                    let mut current_keyframe =
                                                        selected_keyframes.get();
                                                    let mut current_keyframe = current_keyframe
                                                        .get_mut(0)
                                                        .expect("Couldn't get Selected Keyframe");
                                                    let mut current_sequence =
                                                        selected_sequence_data.get();

                                                    let mut current_path_type =
                                                        selected_path_type.get();

                                                    match current_path_type {
                                                        PathType::Linear => {
                                                            current_path_type =
                                                                PathType::Bezier(CurveData {
                                                                    control_point1: Some(
                                                                        ControlPoint { x: 0, y: 0 },
                                                                    ),
                                                                    control_point2: Some(
                                                                        ControlPoint {
                                                                            x: value as i32,
                                                                            y: 0,
                                                                        },
                                                                    ),
                                                                });
                                                        }
                                                        PathType::Bezier(curve_data) => {
                                                            let c1 =
                                                                curve_data.control_point1.expect(
                                                                    "Couldn't get control point 1",
                                                                );
                                                            let c2 =
                                                                curve_data.control_point2.expect(
                                                                    "Couldn't get control point 2",
                                                                );

                                                            current_path_type =
                                                                PathType::Bezier(CurveData {
                                                                    control_point1: Some(
                                                                        ControlPoint {
                                                                            x: c1.x,
                                                                            y: c1.y,
                                                                        },
                                                                    ),
                                                                    control_point2: Some(
                                                                        ControlPoint {
                                                                            x: value as i32,
                                                                            y: c2.y,
                                                                        },
                                                                    ),
                                                                });
                                                        }
                                                    }

                                                    selected_path_type
                                                        .set(current_path_type.clone());

                                                    current_keyframe.path_type = current_path_type;

                                                    let editor_state =
                                                        editor_state_cloned12.lock().unwrap();

                                                    update_keyframe(
                                                        editor_state,
                                                        current_animation_data,
                                                        current_keyframe,
                                                        current_sequence,
                                                        selected_keyframes,
                                                        animation_data,
                                                        selected_sequence_data,
                                                        selected_sequence_id,
                                                        sequence_selected,
                                                    );

                                                    println!("Keyframe updated!");

                                                    let mut editor = editor_cloned3.lock().unwrap();
                                                    editor.update_motion_paths(
                                                        &selected_sequence_data.get(),
                                                    );

                                                    println!("Motion Paths updated!");
                                                },
                                                editor_state_cloned13,
                                                "x".to_string(),
                                                object_type.clone(),
                                            )
                                            .style(move |s| s.width(halfs).margin_right(5.0)),
                                            debounce_input(
                                                "Y:".to_string(),
                                                &c2.y.to_string(),
                                                "Enter Y",
                                                move |value| {
                                                    let value = string_to_f32(&value)
                                                        .map_err(|_| {
                                                            "Couldn't convert string to f32"
                                                        })
                                                        .expect("Couldn't convert string to f32");

                                                    let mut current_animation_data = animation_data
                                                        .get()
                                                        .expect("Couldn't get Animation Data");
                                                    let mut current_keyframe =
                                                        selected_keyframes.get();
                                                    let mut current_keyframe = current_keyframe
                                                        .get_mut(0)
                                                        .expect("Couldn't get Selected Keyframe");
                                                    let mut current_sequence =
                                                        selected_sequence_data.get();

                                                    let mut current_path_type =
                                                        selected_path_type.get();

                                                    match current_path_type {
                                                        PathType::Linear => {
                                                            current_path_type =
                                                                PathType::Bezier(CurveData {
                                                                    control_point1: Some(
                                                                        ControlPoint { x: 0, y: 0 },
                                                                    ),
                                                                    control_point2: Some(
                                                                        ControlPoint {
                                                                            x: 0,
                                                                            y: value as i32,
                                                                        },
                                                                    ),
                                                                });
                                                        }
                                                        PathType::Bezier(curve_data) => {
                                                            let c1 =
                                                                curve_data.control_point1.expect(
                                                                    "Couldn't get control point 1",
                                                                );
                                                            let c2 =
                                                                curve_data.control_point2.expect(
                                                                    "Couldn't get control point 2",
                                                                );

                                                            current_path_type =
                                                                PathType::Bezier(CurveData {
                                                                    control_point1: Some(
                                                                        ControlPoint {
                                                                            x: c1.x,
                                                                            y: c1.y,
                                                                        },
                                                                    ),
                                                                    control_point2: Some(
                                                                        ControlPoint {
                                                                            x: c2.x,
                                                                            y: value as i32,
                                                                        },
                                                                    ),
                                                                });
                                                        }
                                                    }

                                                    selected_path_type
                                                        .set(current_path_type.clone());

                                                    current_keyframe.path_type = current_path_type;

                                                    let editor_state =
                                                        editor_state_cloned14.lock().unwrap();

                                                    update_keyframe(
                                                        editor_state,
                                                        current_animation_data,
                                                        current_keyframe,
                                                        current_sequence,
                                                        selected_keyframes,
                                                        animation_data,
                                                        selected_sequence_data,
                                                        selected_sequence_id,
                                                        sequence_selected,
                                                    );

                                                    println!("Keyframe updated!");

                                                    let mut editor = editor_cloned4.lock().unwrap();
                                                    editor.update_motion_paths(
                                                        &selected_sequence_data.get(),
                                                    );

                                                    println!("Motion Paths updated!");
                                                },
                                                editor_state_cloned15,
                                                "y".to_string(),
                                                object_type.clone(),
                                            )
                                            .style(move |s| s.width(halfs)),
                                        ))
                                        .style(move |s| s.width(aside_width)),
                                    ))
                                    .into_any()
                                } else {
                                    container((empty())).into_any()
                                }
                            },
                        ),
                    )),
                    h_stack((
                        debounce_input(
                            "X:".to_string(),
                            &position[0].to_string(),
                            "Enter X",
                            move |value| {
                                // update animation_data, selected_polygon_data, and selected_keyframes, and selected_sequence_data,
                                // then save merge animation_data with saved_data and save to file
                                // although perhaps polygon_data is not related to the keyframe data? no need to update here?

                                let value = string_to_f32(&value)
                                    .map_err(|_| "Couldn't convert string to f32")
                                    .expect("Couldn't convert string to f32");

                                let mut current_animation_data =
                                    animation_data.get().expect("Couldn't get Animation Data");
                                let mut current_keyframe = selected_keyframes.get();
                                let mut current_keyframe = current_keyframe
                                    .get_mut(0)
                                    .expect("Couldn't get Selected Keyframe");
                                let mut current_sequence = selected_sequence_data.get();
                                // let current_polygon = selected_polygon_data.read();
                                // let current_polygon = current_polygon.borrow();

                                // update keyframe
                                current_keyframe.value =
                                    KeyframeValue::Position([value as i32, position[1]]);

                                let editor_state = editor_state_cloned6.lock().unwrap();

                                update_keyframe(
                                    editor_state,
                                    current_animation_data,
                                    current_keyframe,
                                    current_sequence,
                                    selected_keyframes,
                                    animation_data,
                                    selected_sequence_data,
                                    selected_sequence_id,
                                    sequence_selected,
                                );

                                println!("Keyframe updated!");

                                let mut editor = editor_cloned5.lock().unwrap();
                                editor.update_motion_paths(&selected_sequence_data.get());

                                println!("Motion Paths updated!");
                            },
                            editor_state_cloned,
                            "x".to_string(),
                            object_type_cloned.clone(),
                        )
                        .style(move |s| s.width(halfs).margin_right(5.0)),
                        debounce_input(
                            "Y:".to_string(),
                            &position[1].to_string(),
                            "Enter Y",
                            move |value| {
                                let value = string_to_f32(&value)
                                    .map_err(|_| "Couldn't convert string to f32")
                                    .expect("Couldn't convert string to f32");

                                let mut current_animation_data =
                                    animation_data.get().expect("Couldn't get Animation Data");
                                let mut current_keyframe = selected_keyframes.get();
                                let mut current_keyframe = current_keyframe
                                    .get_mut(0)
                                    .expect("Couldn't get Selected Keyframe");
                                let mut current_sequence = selected_sequence_data.get();
                                // let current_polygon = selected_polygon_data.read();
                                // let current_polygon = current_polygon.borrow();

                                // update keyframe
                                current_keyframe.value =
                                    KeyframeValue::Position([position[0], value as i32]);

                                let editor_state = editor_state_cloned7.lock().unwrap();

                                update_keyframe(
                                    editor_state,
                                    current_animation_data,
                                    current_keyframe,
                                    current_sequence,
                                    selected_keyframes,
                                    animation_data,
                                    selected_sequence_data,
                                    selected_sequence_id,
                                    sequence_selected,
                                );

                                println!("Keyframe updated!");

                                let mut editor = editor_cloned6.lock().unwrap();
                                editor.update_motion_paths(&selected_sequence_data.get());

                                println!("Motion Paths updated!");
                            },
                            editor_state_cloned2,
                            "y".to_string(),
                            object_type_cloned.clone(),
                        )
                        .style(move |s| s.width(halfs)),
                    ))
                    .style(move |s| s.width(260.0)),
                ))),
            )
            .into_any(),
            KeyframeValue::Rotation(rotation) => container(
                (v_stack((
                    label(|| "Keyframe"),
                    simple_button("Back to Properties".to_string(), move |_| {
                        selected_keyframes.set(Vec::new());
                    }),
                    debounce_input(
                        "Rotation Degrees:".to_string(),
                        &rotation.to_string(),
                        "Enter Degrees",
                        move |value| {
                            // update animation_data, selected_polygon_data, and selected_keyframes, and selected_sequence_data,
                            // then save merge animation_data with saved_data and save to file
                            // although perhaps polygon_data is not related to the keyframe data? no need to update here?

                            let value = string_to_f32(&value)
                                .map_err(|_| "Couldn't convert string to f32")
                                .expect("Couldn't convert string to f32");

                            let mut current_animation_data =
                                animation_data.get().expect("Couldn't get Animation Data");
                            let mut current_keyframe = selected_keyframes.get();
                            let mut current_keyframe = current_keyframe
                                .get_mut(0)
                                .expect("Couldn't get Selected Keyframe");
                            let mut current_sequence = selected_sequence_data.get();
                            // let current_polygon = selected_polygon_data.read();
                            // let current_polygon = current_polygon.borrow();

                            // update keyframe
                            current_keyframe.value = KeyframeValue::Rotation(value as i32);

                            let editor_state = editor_state_cloned17.lock().unwrap();

                            update_keyframe(
                                editor_state,
                                current_animation_data,
                                current_keyframe,
                                current_sequence,
                                selected_keyframes,
                                animation_data,
                                selected_sequence_data,
                                selected_sequence_id,
                                sequence_selected,
                            );
                        },
                        editor_state_cloned3,
                        "rotation".to_string(),
                        object_type,
                    ),
                ))
                .style(move |s| s.width(260.0))),
            )
            .into_any(),
            KeyframeValue::Scale(scale) => container(
                (v_stack((
                    label(|| "Keyframe"),
                    simple_button("Back to Properties".to_string(), move |_| {
                        selected_keyframes.set(Vec::new());
                    }),
                    debounce_input(
                        "Scale (100 default):".to_string(),
                        &scale.to_string(),
                        "Enter Scale",
                        move |value| {
                            // update animation_data, selected_polygon_data, and selected_keyframes, and selected_sequence_data,
                            // then save merge animation_data with saved_data and save to file
                            // although perhaps polygon_data is not related to the keyframe data? no need to update here?

                            let value = string_to_f32(&value)
                                .map_err(|_| "Couldn't convert string to f32")
                                .expect("Couldn't convert string to f32");

                            let mut current_animation_data =
                                animation_data.get().expect("Couldn't get Animation Data");
                            let mut current_keyframe = selected_keyframes.get();
                            let mut current_keyframe = current_keyframe
                                .get_mut(0)
                                .expect("Couldn't get Selected Keyframe");
                            let mut current_sequence = selected_sequence_data.get();
                            // let current_polygon = selected_polygon_data.read();
                            // let current_polygon = current_polygon.borrow();

                            // update keyframe
                            current_keyframe.value = KeyframeValue::Scale(value as i32);

                            let editor_state = editor_state_cloned18.lock().unwrap();

                            update_keyframe(
                                editor_state,
                                current_animation_data,
                                current_keyframe,
                                current_sequence,
                                selected_keyframes,
                                animation_data,
                                selected_sequence_data,
                                selected_sequence_id,
                                sequence_selected,
                            );
                        },
                        editor_state_cloned4,
                        "scale".to_string(),
                        object_type,
                    ),
                ))
                .style(move |s| s.width(260.0))),
            )
            .into_any(),
            KeyframeValue::Opacity(opacity) => container(
                (v_stack((
                    label(|| "Keyframe"),
                    simple_button("Back to Properties".to_string(), move |_| {
                        selected_keyframes.set(Vec::new());
                    }),
                    debounce_input(
                        "Opacity (default 100):".to_string(),
                        &opacity.to_string(),
                        "Enter Opacity",
                        move |value| {
                            // update animation_data, selected_polygon_data, and selected_keyframes, and selected_sequence_data,
                            // then save merge animation_data with saved_data and save to file
                            // although perhaps polygon_data is not related to the keyframe data? no need to update here?

                            let value = string_to_f32(&value)
                                .map_err(|_| "Couldn't convert string to f32")
                                .expect("Couldn't convert string to f32");

                            let mut current_animation_data =
                                animation_data.get().expect("Couldn't get Animation Data");
                            let mut current_keyframe = selected_keyframes.get();
                            let mut current_keyframe = current_keyframe
                                .get_mut(0)
                                .expect("Couldn't get Selected Keyframe");
                            let mut current_sequence = selected_sequence_data.get();
                            // let current_polygon = selected_polygon_data.read();
                            // let current_polygon = current_polygon.borrow();

                            // update keyframe
                            current_keyframe.value = KeyframeValue::Opacity(value as i32);

                            let editor_state = editor_state_cloned19.lock().unwrap();

                            update_keyframe(
                                editor_state,
                                current_animation_data,
                                current_keyframe,
                                current_sequence,
                                selected_keyframes,
                                animation_data,
                                selected_sequence_data,
                                selected_sequence_id,
                                sequence_selected,
                            );
                        },
                        editor_state_cloned5,
                        "opacity".to_string(),
                        object_type,
                    ),
                ))
                .style(move |s| s.width(260.0))),
            )
            .into_any(),
            KeyframeValue::Zoom(zoom) => container(
                (v_stack((
                    label(|| "Keyframe"),
                    simple_button("Back to Properties".to_string(), move |_| {
                        selected_keyframes.set(Vec::new());
                    }),
                    debounce_input(
                        "Zoom (default 100):".to_string(),
                        &zoom.to_string(),
                        "Enter Zoom",
                        move |value| {
                            // update animation_data, selected_polygon_data, and selected_keyframes, and selected_sequence_data,
                            // then save merge animation_data with saved_data and save to file
                            // although perhaps polygon_data is not related to the keyframe data? no need to update here?

                            let value = string_to_f32(&value)
                                .map_err(|_| "Couldn't convert string to f32")
                                .expect("Couldn't convert string to f32");

                            let mut current_animation_data =
                                animation_data.get().expect("Couldn't get Animation Data");
                            let mut current_keyframe = selected_keyframes.get();
                            let mut current_keyframe = current_keyframe
                                .get_mut(0)
                                .expect("Couldn't get Selected Keyframe");
                            let mut current_sequence = selected_sequence_data.get();
                            // let current_polygon = selected_polygon_data.read();
                            // let current_polygon = current_polygon.borrow();

                            // update keyframe
                            current_keyframe.value = KeyframeValue::Zoom(value as i32);

                            let editor_state = editor_state_cloned20.lock().unwrap();

                            update_keyframe(
                                editor_state,
                                current_animation_data,
                                current_keyframe,
                                current_sequence,
                                selected_keyframes,
                                animation_data,
                                selected_sequence_data,
                                selected_sequence_id,
                                sequence_selected,
                            );
                        },
                        editor_state_cloned5,
                        "zoom".to_string(),
                        object_type,
                    ),
                ))
                .style(move |s| s.width(260.0))),
            )
            .into_any(),
            _ => empty().into_any(),
        },
        simple_button("Delete Keyframe".to_string(), move |_| {
            let mut current_keyframe = selected_keyframes.get();
            let mut current_keyframe = current_keyframe
                .get_mut(0)
                .expect("Couldn't get Selected Keyframe");
            let mut current_sequence = selected_sequence_data.get();

            selected_keyframes.set(Vec::new());

            // update animation data
            let mut current_animation_data =
                animation_data.get().expect("Couldn't get animation data");
            current_animation_data.properties.iter_mut().for_each(|p| {
                if let Some(index) = p.keyframes.iter().position(|k| k.id == current_keyframe.id) {
                    p.keyframes.swap_remove(index);
                }
            });

            animation_data.set(Some(current_animation_data.clone()));

            // update sequence
            current_sequence
                .polygon_motion_paths
                .iter_mut()
                .for_each(|pm| {
                    if pm.id == current_animation_data.id {
                        *pm = current_animation_data.clone();
                    }
                });

            selected_sequence_data.set(current_sequence.clone());

            let mut editor = editor_cloned7.lock().unwrap();

            editor.current_sequence_data = Some(current_sequence.clone());
            editor.update_motion_paths(&current_sequence);

            drop(editor);

            // save to file
            let mut editor_state = editor_state_cloned16.lock().unwrap();

            let last_saved_state = editor_state
                .record_state
                .saved_state
                .as_mut()
                .expect("Couldn't get Saved State");

            last_saved_state.sequences.iter_mut().for_each(|s| {
                if s.id == selected_sequence_id.get() {
                    s.polygon_motion_paths.iter_mut().for_each(|pm| {
                        if pm.id == current_animation_data.id {
                            *pm = current_animation_data.clone();
                        }
                    });
                }
            });

            // TODO: probably perf hit with larger files, or does it get released?
            let new_saved_state = last_saved_state.to_owned();

            save_saved_state_raw(new_saved_state);

            drop(editor_state);
        })
        .style(|s| s.color(Color::RED)),
    ))
    .style(|s| card_styles(s).width(300.0))
}
