use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::fs;
use std::path::Path;
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use bytemuck::Contiguous;
use floem::common::{card_styles, simple_button};
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
use stunts_engine::editor::{string_to_f32, Editor, OnMouseUp, Point, PolygonClickHandler, Viewport};
use stunts_engine::polygon::{PolygonConfig, Stroke};
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

use crate::editor_state::EditorState;
use stunts_engine::animations::{
    Sequence, AnimationData, AnimationProperty, EasingType, KeyframeValue, UIKeyframe,
};
use crate::helpers::utilities::save_saved_state_raw;

use super::aside::tab_interface;
use super::inputs::styled_input;
use super::keyframe_timeline::{create_timeline, TimelineConfig, TimelineState};
use super::properties_panel::properties_view;
use super::sequence_panel::sequence_panel;

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
    let mut new_keyframes = Vec::new();
    new_keyframes.push(current_keyframe.to_owned());

    selected_keyframes.set(new_keyframes);

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

    let new_saved_state = last_saved_state.to_owned();

    save_saved_state_raw(new_saved_state);
}

pub fn app_view(
    editor_state: Arc<Mutex<EditorState>>,
    editor: std::sync::Arc<Mutex<Editor>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
) -> impl IntoView {
    let editor_cloned = Arc::clone(&editor);
    let editor_cloned2 = Arc::clone(&editor);
    let editor_cloned3 = Arc::clone(&editor);
    let editor_cloned4 = Arc::clone(&editor);
    let editor_cloned5 = Arc::clone(&editor);
    let editor_cloned6 = Arc::clone(&editor);
    let editor_cloned7 = Arc::clone(&editor);

    let state_cloned = Arc::clone(&editor_state);
    let state_cloned2 = Arc::clone(&editor_state);
    let state_cloned3 = Arc::clone(&editor_state);
    let state_cloned4 = Arc::clone(&editor_state);
    let state_cloned5 = Arc::clone(&editor_state);
    let state_cloned6 = Arc::clone(&editor_state);
    let state_cloned7 = Arc::clone(&editor_state);

    let gpu_cloned = Arc::clone(&gpu_helper);
    let gpu_cloned2 = Arc::clone(&gpu_helper);

    let viewport_cloned = Arc::clone(&viewport);
    let viewport_cloned2 = Arc::clone(&viewport);

    // set in sequence_panel
    let sequence_selected = create_rw_signal(false);
    let selected_sequence_id = create_rw_signal(String::new());
    let selected_sequence_data = create_rw_signal(Sequence {
        id: String::new(),
        active_polygons: Vec::new(),
        polygon_motion_paths: Vec::new(),
    });

    // set
    let polygon_selected = create_rw_signal(false);
    let selected_polygon_id = create_rw_signal(Uuid::nil());
    let selected_polygon_data = create_rw_signal(PolygonConfig {
        id: Uuid::nil(),
        name: String::new(),
        points: Vec::new(),
        dimensions: (100.0, 100.0),
        position: Point { x: 0.0, y: 0.0 },
        border_radius: 0.0,
        fill: [0.0, 0.0, 0.0, 1.0],
        stroke: Stroke {
            fill: [1.0, 1.0, 1.0, 1.0],
            thickness: 2.0,
        },
    });

    let animation_data: RwSignal<Option<AnimationData>> = create_rw_signal(None);
    let selected_keyframes: RwSignal<Vec<UIKeyframe>> = create_rw_signal(Vec::new());

    let polygon_selected_ref = Arc::new(Mutex::new(polygon_selected));
    let selected_polygon_id_ref = Arc::new(Mutex::new(selected_polygon_id));
    let selected_polygon_data_ref = Arc::new(Mutex::new(selected_polygon_data));
    let animation_data_ref = Arc::new(Mutex::new(animation_data));

    let editor_cloned2 = editor_cloned2.clone();

    let aside_width = 260.0;
    let quarters = (aside_width / 4.0) + (5.0 * 4.0);
    let thirds = (aside_width / 3.0) + (5.0 * 3.0);
    let halfs = (aside_width / 2.0) + (5.0 * 2.0);

    let handle_polygon_click: Arc<PolygonClickHandler> = Arc::new({
        let editor_state = editor_state.clone();
        let polygon_selected_ref = Arc::clone(&polygon_selected_ref);
        let selected_polygon_id_ref = Arc::clone(&selected_polygon_id_ref);
        let selected_polygon_data_ref = Arc::clone(&selected_polygon_data_ref);
        let animation_data_ref = Arc::clone(&animation_data_ref);

        move || {
            let editor_state = editor_state.clone();
            let polygon_selected_ref = polygon_selected_ref.clone();
            let selected_polygon_id_ref = selected_polygon_id_ref.clone();
            let selected_polygon_data_ref = selected_polygon_data_ref.clone();
            let animation_data_ref = animation_data_ref.clone();

            Some(
                Box::new(move |polygon_id: Uuid, polygon_data: PolygonConfig| {
                    // cannot lock editor here! probably because called from Editor
                    // {
                    //     let mut editor = new_editor.lock().unwrap();
                    //     // Update editor as needed
                    // }

                    if let Ok(mut polygon_selected) = polygon_selected_ref.lock() {
                        polygon_selected.update(|c| {
                            *c = true;
                        });
                    }
                    if let Ok(mut selected_polygon_id) = selected_polygon_id_ref.lock() {
                        selected_polygon_id.update(|c| {
                            *c = polygon_id;
                        });

                        let mut editor_state = editor_state.lock().unwrap();

                        editor_state.selected_polygon_id = polygon_id;
                        editor_state.polygon_selected = true;

                        drop(editor_state);
                    }
                    if let Ok(mut selected_polygon_data) = selected_polygon_data_ref.lock() {
                        selected_polygon_data.update(|c| {
                            *c = polygon_data;
                        });
                    }
                    if let Ok(mut animation_data) = animation_data_ref.lock() {
                        let editor_state = editor_state.lock().unwrap();
                        let saved_state = editor_state
                            .saved_state
                            .as_ref()
                            .expect("Couldn't get Saved State");

                        // let saved_sequence = saved_state
                        //     .sequences
                        //     .iter()
                        //     .find(|s| {
                        //         s.enter_motion_paths
                        //             .iter()
                        //             .any(|m| m.polygon_id == polygon_id.to_string())
                        //     })
                        //     .expect("Couldn't find matching sequence");
                        let saved_animation_data = saved_state
                            .sequences
                            .iter()
                            .flat_map(|s| s.polygon_motion_paths.iter())
                            .find(|p| p.polygon_id == polygon_id.to_string());

                        if let Some(polygon_animation_data) = saved_animation_data {
                            animation_data.update(|c| {
                                *c = Some(polygon_animation_data.clone());
                            });
                        } else {
                            // polygon is not saved animation data
                            // polygon_index,time,width,height,x,y,rotation,scale,perspective_x,perspective_y,opacity
                        }

                        drop(editor_state);
                    }
                }) as Box<dyn FnMut(Uuid, PolygonConfig) + Send>,
            )
        }
    });

    let on_mouse_up: Arc<OnMouseUp> = Arc::new({
        let editor_state = editor_state.clone();
        let polygon_selected_ref = Arc::clone(&polygon_selected_ref);
        let selected_polygon_id_ref = Arc::clone(&selected_polygon_id_ref);
        let selected_polygon_data_ref = Arc::clone(&selected_polygon_data_ref);
        let animation_data_ref = Arc::clone(&animation_data_ref);

        move || {
            let editor_state = editor_state.clone();
            let polygon_selected_ref = polygon_selected_ref.clone();
            let selected_polygon_id_ref = selected_polygon_id_ref.clone();
            let selected_polygon_data_ref = selected_polygon_data_ref.clone();
            let animation_data_ref = animation_data_ref.clone();

            Some(
                Box::new(move |poly_index: usize, point: Point| {
                    // cannot lock editor here! probably because called from Editor
                    // {
                    //     let mut editor = new_editor.lock().unwrap();
                    //     // Update editor as needed
                    // }

                    // let value = string_to_f32(&value).map_err(|_| "Couldn't convert string to f32").expect("Couldn't convert string to f32");

                    let mut current_animation_data = animation_data.get().expect("Couldn't get Animation Data");
                    let mut current_keyframe = selected_keyframes.get();

                    if let Some(current_keyframe) = current_keyframe.get_mut(0) {
                        // let mut current_keyframe = current_keyframe.get_mut(0).expect("Couldn't get Selected Keyframe");
                        let mut current_sequence = selected_sequence_data.get();
                        // let current_polygon = selected_polygon_data.read();
                        // let current_polygon = current_polygon.borrow();

                        // update keyframe
                        current_keyframe.value = KeyframeValue::Position([point.x as i32, point.y as i32]);

                        let mut editor_state = editor_state.lock().unwrap();

                        update_keyframe(
                            editor_state,
                            current_animation_data,
                            current_keyframe,
                            current_sequence,
                            selected_keyframes,
                            animation_data,
                            selected_sequence_data,
                            selected_sequence_id,
                            sequence_selected
                        );

                        // drop(editor_state);

                        println!("Keyframe updated!");
                    }                    

                    // let mut editor = editor_cloned7.lock().unwrap();
                    // editor.update_motion_paths(&selected_sequence_data.get());

                    // println!("Motion Paths updated!");

                    selected_sequence_data.get()
                }) as Box<dyn FnMut(usize, Point) -> Sequence + Send>,
            )
        }
    });

    // Use create_effect to set the handler only once
    create_effect({
        let handle_polygon_click = Arc::clone(&handle_polygon_click);
        let editor_cloned3 = Arc::clone(&editor_cloned3);
        move |_| {
            let mut editor = editor_cloned3.lock().unwrap();
            editor.handle_polygon_click = Some(Arc::clone(&handle_polygon_click));
            editor.on_mouse_up = Some(Arc::clone(&on_mouse_up));
        }
    });

    container((
        tab_interface(
            gpu_helper.clone(),
            editor_state.clone(),
            editor,
            viewport.clone(),
            sequence_selected,
            selected_sequence_id,
            selected_sequence_data,
            polygon_selected
        ),
        dyn_container(
            move || sequence_selected.get()  && !polygon_selected.get() && selected_keyframes.get().len() == 0,
            move |sequence_selected_real| {
                if sequence_selected_real {
                    h_stack((
                        sequence_panel(
                            state_cloned.clone(),
                            gpu_cloned.clone(),
                            editor_cloned3.clone(),
                            viewport_cloned.clone(),
                            sequence_selected,
                            selected_sequence_id,
                            selected_sequence_data,
                        ),
                        // keyframe_timeline,
                    ))
                    .into_any()
                } else {
                    empty().into_any()
                }
            },
        ),
        dyn_container(
            move || polygon_selected.get(),
            move |polygon_selected_real| {
                if polygon_selected_real {
                    let state_cloned3 = state_cloned3.clone();
                    let state_cloned4 = state_cloned4.clone();
                    let editor_cloned4 = editor_cloned4.clone();
                    let state_cloned6 = state_cloned6.clone();
                    let state_cloned7 = state_cloned7.clone();
                    let editor_cloned5 = editor_cloned5.clone();
                    let editor_cloned6 = editor_cloned6.clone();

                    let state = TimelineState {
                        current_time: Duration::from_secs_f64(0.0),
                        zoom_level: 1.0,
                        scroll_offset: 0.0,
                        // selected_keyframes: Vec::new(),
                        property_expansions: im::HashMap::from_iter([
                            ("position".to_string(), true),
                            ("rotation".to_string(), true),
                            ("scale".to_string(), true),
                            ("opacity".to_string(), true),
                        ]),
                        dragging: None,
                        hovered_keyframe: None,
                        selected_keyframes,
                    };

                    let config = TimelineConfig {
                        width: 1200.0,
                        height: 300.0,
                        header_height: 30.0,
                        property_width: 38.0,
                        row_height: 24.0,
                        // offset_x: 325.0,
                        // offset_y: 300.0,
                        offset_x: 0.0,
                        offset_y: 0.0,
                    };

                    let keyframe_timeline = create_timeline(state, config, animation_data);

                    h_stack((
                        dyn_container(
                            move || polygon_selected.get() && selected_keyframes.get().len() == 0,
                            move |polygon_selected_real| {
                                if polygon_selected_real {
                                    let state_cloned5 = state_cloned6.clone();
                                    let state_cloned6 = state_cloned7.clone();

                                    container(
                                        (v_stack((
                                            label(|| "Properties"),
                                            simple_button("Back to Sequence".to_string(), move |_| {
                                                polygon_selected.set(false);
                                            }),
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
                                                            editor_state.update_width(&value).expect("Couldn't update width");
                                                            // TODO: probably should update selected_polygon_data
                                                            // need to update active_polygons in saved_data
                                                            // TODO: on_debounce_stop?
                                                            let value = string_to_f32(&value).expect("Couldn't convert string");
                                                            let mut saved_state = editor_state.saved_state.as_mut().expect("Couldn't get Saved State");

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
                                                        }
                                                    }),
                                                    state_cloned5,
                                                    "width".to_string(),
                                                )
                                                .style(move |s| {
                                                    s.width(halfs).margin_right(5.0)
                                                }),
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
                                                            editor_state.update_height(&value).expect("Couldn't update height");
                                                            // TODO: probably should update selected_polygon_data
                                                            // need to update active_polygons in saved_data
                                                            // TODO: on_debounce_stop?
                                                            let value = string_to_f32(&value).expect("Couldn't convert string");
                                                            let mut saved_state = editor_state.saved_state.as_mut().expect("Couldn't get Saved State");

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
                                                    state_cloned6,
                                                    "height".to_string(),
                                                )
                                                .style(move |s| s.width(halfs)),
                                            ))
                                            .style(move |s| s.width(aside_width)),
                                        ))
                                        .style(|s| card_styles(s))),
                                    )
                                    .into_any()
                                } else {
                                    empty().into_any()
                                }
                            },
                        ),
                        dyn_container(
                            move || selected_keyframes.get(),
                            move |selected_keyframes_real| {
                                if let Some(selected_keyframe) = selected_keyframes_real.get(0) {
                                    let state_cloned3 = state_cloned3.clone();
                                    let state_cloned4 = state_cloned4.clone();
                                    let editor_cloned5 = editor_cloned5.clone();
                                    let editor_cloned6 = editor_cloned6.clone();

                                    match selected_keyframe.value {
                                        KeyframeValue::Position(position) => container(
                                            (v_stack((
                                                label(|| "Keyframe"),
                                                simple_button("Back to Properties".to_string(), move |_| {
                                                    selected_keyframes.set(Vec::new());
                                                }),
                                                h_stack((
                                                    styled_input(
                                                        "X:".to_string(),
                                                        &position[0].to_string(),
                                                        "Enter X",
                                                        Box::new({
                                                            move |mut editor_state: MutexGuard<'_, EditorState>, value| {
                                                                // update animation_data, selected_polygon_data, and selected_keyframes, and selected_sequence_data,
                                                                // then save merge animation_data with saved_data and save to file
                                                                // although perhaps polygon_data is not related to the keyframe data? no need to update here?

                                                                let value = string_to_f32(&value).map_err(|_| "Couldn't convert string to f32").expect("Couldn't convert string to f32");

                                                                let mut current_animation_data = animation_data.get().expect("Couldn't get Animation Data");
                                                                let mut current_keyframe = selected_keyframes.get();
                                                                let mut current_keyframe = current_keyframe.get_mut(0).expect("Couldn't get Selected Keyframe");
                                                                let mut current_sequence = selected_sequence_data.get();
                                                                // let current_polygon = selected_polygon_data.read();
                                                                // let current_polygon = current_polygon.borrow();

                                                                // update keyframe
                                                                current_keyframe.value = KeyframeValue::Position([value as i32, position[1]]);

                                                                update_keyframe(
                                                                    editor_state,
                                                                    current_animation_data,
                                                                    current_keyframe,
                                                                    current_sequence,
                                                                    selected_keyframes,
                                                                    animation_data,
                                                                    selected_sequence_data,
                                                                    selected_sequence_id,
                                                                    sequence_selected
                                                                );

                                                                println!("Keyframe updated!");

                                                                let mut editor = editor_cloned5.lock().unwrap();
                                                                editor.update_motion_paths(&selected_sequence_data.get());

                                                                println!("Motion Paths updated!");
                                                            }
                                                        }),
                                                        state_cloned3,
                                                        "x".to_string(),
                                                    )
                                                    .style(move |s| {
                                                        s.width(halfs).margin_right(5.0)
                                                    }),
                                                    styled_input(
                                                        "Y:".to_string(),
                                                        &position[1].to_string(),
                                                        "Enter Y",
                                                        Box::new({
                                                            move |mut editor_state, value| {
                                                                let value = string_to_f32(&value).map_err(|_| "Couldn't convert string to f32").expect("Couldn't convert string to f32");

                                                                let mut current_animation_data = animation_data.get().expect("Couldn't get Animation Data");
                                                                let mut current_keyframe = selected_keyframes.get();
                                                                let mut current_keyframe = current_keyframe.get_mut(0).expect("Couldn't get Selected Keyframe");
                                                                let mut current_sequence = selected_sequence_data.get();
                                                                // let current_polygon = selected_polygon_data.read();
                                                                // let current_polygon = current_polygon.borrow();

                                                                // update keyframe
                                                                current_keyframe.value = KeyframeValue::Position([position[0], value as i32]);

                                                                update_keyframe(
                                                                    editor_state,
                                                                    current_animation_data,
                                                                    current_keyframe,
                                                                    current_sequence,
                                                                    selected_keyframes,
                                                                    animation_data,
                                                                    selected_sequence_data,
                                                                    selected_sequence_id,
                                                                    sequence_selected
                                                                );

                                                                println!("Keyframe updated!");

                                                                let mut editor = editor_cloned6.lock().unwrap();
                                                                editor.update_motion_paths(&selected_sequence_data.get());

                                                                println!("Motion Paths updated!");
                                                            }
                                                        }),
                                                        state_cloned4,
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
                                            (v_stack((label(|| "Keyframe"),
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

                                                        let value = string_to_f32(&value).map_err(|_| "Couldn't convert string to f32").expect("Couldn't convert string to f32");

                                                        let mut current_animation_data = animation_data.get().expect("Couldn't get Animation Data");
                                                        let mut current_keyframe = selected_keyframes.get();
                                                        let mut current_keyframe = current_keyframe.get_mut(0).expect("Couldn't get Selected Keyframe");
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
                                                            sequence_selected
                                                        );
                                                    }
                                                }),
                                                state_cloned3,
                                                "rotation".to_string(),
                                            )))
                                                .style(|s| card_styles(s))),
                                        )
                                        .into_any(),
                                        KeyframeValue::Scale(scale) => container(
                                            (v_stack((label(|| "Keyframe"),
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

                                                        let value = string_to_f32(&value).map_err(|_| "Couldn't convert string to f32").expect("Couldn't convert string to f32");

                                                        let mut current_animation_data = animation_data.get().expect("Couldn't get Animation Data");
                                                        let mut current_keyframe = selected_keyframes.get();
                                                        let mut current_keyframe = current_keyframe.get_mut(0).expect("Couldn't get Selected Keyframe");
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
                                                            sequence_selected
                                                        );
                                                    }
                                                }),
                                                state_cloned3,
                                                "scale".to_string(),
                                            )))
                                                .style(|s| card_styles(s))),
                                        )
                                        .into_any(),
                                        KeyframeValue::Opacity(opacity) => container(
                                            (v_stack((label(|| "Keyframe"),
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

                                                        let value = string_to_f32(&value).map_err(|_| "Couldn't convert string to f32").expect("Couldn't convert string to f32");

                                                        let mut current_animation_data = animation_data.get().expect("Couldn't get Animation Data");
                                                        let mut current_keyframe = selected_keyframes.get();
                                                        let mut current_keyframe = current_keyframe.get_mut(0).expect("Couldn't get Selected Keyframe");
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
                                                            sequence_selected
                                                        );
                                                    }
                                                }),
                                                state_cloned3,
                                                "opacity".to_string(),
                                            )))
                                                .style(|s| card_styles(s))),
                                        )
                                        .into_any(),
                                        _ => empty().into_any(),
                                    }
                                } else {
                                    empty().into_any()
                                }
                            },
                        ),
                        v_stack((
                            simple_button("Play Sequence".to_string(), move |_| {
                                let mut editor = editor_cloned4.lock().unwrap();

                                if editor.is_playing {
                                    println!("Pause Sequence...");
                                    
                                    editor.is_playing = false;
                                    editor.start_playing_time = None;
                                } else {
                                    println!("Play Sequence...");

                                    let now = std::time::Instant::now();
                                    editor.start_playing_time = Some(now);

                                    editor.current_sequence_data = Some(selected_sequence_data.get());
                                    editor.is_playing = true;
                                }

                                // EventPropagation::Continue
                            }),
                            keyframe_timeline,
                        )).style(|s| s.margin_top(425.0))
                        
                    ))
                    .into_any()
                } else {
                    empty().into_any()
                }
            },
        ),
    ))
}
