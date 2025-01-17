use floem::common::card_styles;
use floem::common::simple_button;
use floem::common::small_button;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use stunts_engine::animations::AnimationData;
use stunts_engine::animations::KeyframeValue;
use stunts_engine::animations::Sequence;
use stunts_engine::animations::UIKeyframe;
use stunts_engine::editor::string_to_f32;
use stunts_engine::editor::Editor;
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
) -> impl IntoView {
    let editor_cloned = Arc::clone(&editor);
    let editor_state_cloned = Arc::clone(&editor_state);
    let editor_cloned2 = Arc::clone(&editor);
    let editor_state_cloned2 = Arc::clone(&editor_state);
    let editor_cloned3 = Arc::clone(&editor);
    let editor_state_cloned3 = Arc::clone(&editor_state);
    let editor_cloned4 = Arc::clone(&editor);
    let editor_state_cloned4 = Arc::clone(&editor_state);
    let editor_state_cloned5 = Arc::clone(&editor_state);
    let editor_state_cloned6 = Arc::clone(&editor_state);
    let editor_state_cloned7 = Arc::clone(&editor_state);

    let aside_width = 260.0;
    let quarters = (aside_width / 4.0) + (5.0 * 4.0);
    let thirds = (aside_width / 3.0) + (5.0 * 3.0);
    let halfs = (aside_width / 2.0) + (5.0 * 2.0);

    let back_active = RwSignal::new(false);

    match selected_keyframe.value {
        KeyframeValue::Position(position) => container(
            (v_stack((
                label(|| "Keyframe"),
                simple_button("Back to Properties".to_string(), move |_| {
                    selected_keyframes.set(Vec::new());
                }),
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

                            let mut editor = editor_cloned.lock().unwrap();
                            editor.update_motion_paths(&selected_sequence_data.get());

                            println!("Motion Paths updated!");
                        },
                        editor_state_cloned,
                        "x".to_string(),
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

                            let mut editor = editor_cloned2.lock().unwrap();
                            editor.update_motion_paths(&selected_sequence_data.get());

                            println!("Motion Paths updated!");
                        },
                        editor_state_cloned2,
                        "y".to_string(),
                    )
                    .style(move |s| s.width(halfs)),
                ))
                .style(move |s| s.width(aside_width)),
            ))
            .style(|s| card_styles(s))),
        )
        .into_any(),
        KeyframeValue::Rotation(rotation) => container(
            (v_stack((
                label(|| "Keyframe"),
                simple_button("Back to Properties".to_string(), move |_| {
                    selected_keyframes.set(Vec::new());
                }),
                styled_input(
                    "Rotation Degrees:".to_string(),
                    &rotation.to_string(),
                    "Enter Degrees",
                    Box::new({
                        move |mut editor_state: MutexGuard<'_, EditorState>, value| {
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
                        }
                    }),
                    editor_state_cloned3,
                    "rotation".to_string(),
                ),
            ))
            .style(|s| card_styles(s))),
        )
        .into_any(),
        KeyframeValue::Scale(scale) => container(
            (v_stack((
                label(|| "Keyframe"),
                simple_button("Back to Properties".to_string(), move |_| {
                    selected_keyframes.set(Vec::new());
                }),
                styled_input(
                    "Scale (100 default):".to_string(),
                    &scale.to_string(),
                    "Enter Scale",
                    Box::new({
                        move |mut editor_state: MutexGuard<'_, EditorState>, value| {
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
                        }
                    }),
                    editor_state_cloned4,
                    "scale".to_string(),
                ),
            ))
            .style(|s| card_styles(s))),
        )
        .into_any(),
        KeyframeValue::Opacity(opacity) => container(
            (v_stack((
                label(|| "Keyframe"),
                simple_button("Back to Properties".to_string(), move |_| {
                    selected_keyframes.set(Vec::new());
                }),
                styled_input(
                    "Opacity (default 100):".to_string(),
                    &opacity.to_string(),
                    "Enter Opacity",
                    Box::new({
                        move |mut editor_state: MutexGuard<'_, EditorState>, value| {
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
                        }
                    }),
                    editor_state_cloned5,
                    "opacity".to_string(),
                ),
            ))
            .style(|s| card_styles(s))),
        )
        .into_any(),
        _ => empty().into_any(),
    }
}
