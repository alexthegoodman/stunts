use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::fs;
use std::path::Path;
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use bytemuck::Contiguous;
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
use stunts_engine::editor::{Editor, Point, PolygonClickHandler, Viewport};
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
use crate::helpers::animations::{
    AnimationData, AnimationProperty, EasingType, KeyframeValue, UIKeyframe,
};
use crate::helpers::saved_state::Sequence;

use super::aside::tab_interface;
use super::keyframe_timeline::{create_timeline, TimelineConfig, TimelineState};
use super::properties_panel::properties_view;
use super::sequence_panel::sequence_panel;

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

    let state_cloned = Arc::clone(&editor_state);
    let state_cloned2 = Arc::clone(&editor_state);

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

                            let mut properties = Vec::new();

                            let mut position_keyframes = Vec::new();

                            position_keyframes.push(UIKeyframe {
                                id: Uuid::new_v4().to_string(),
                                time: Duration::from_secs(0),
                                value: KeyframeValue::Position([0, 0]),
                                easing: EasingType::EaseInOut,
                            });
                            position_keyframes.push(UIKeyframe {
                                id: Uuid::new_v4().to_string(),
                                time: Duration::from_millis(2500),
                                value: KeyframeValue::Position([10, 10]),
                                easing: EasingType::EaseInOut,
                            });
                            position_keyframes.push(UIKeyframe {
                                id: Uuid::new_v4().to_string(),
                                time: Duration::from_secs(5),
                                value: KeyframeValue::Position([20, 20]),
                                easing: EasingType::EaseInOut,
                            });
                            position_keyframes.push(UIKeyframe {
                                id: Uuid::new_v4().to_string(),
                                time: Duration::from_secs(15),
                                value: KeyframeValue::Position([20, 20]),
                                easing: EasingType::EaseInOut,
                            });
                            position_keyframes.push(UIKeyframe {
                                id: Uuid::new_v4().to_string(),
                                time: Duration::from_millis(17500),
                                value: KeyframeValue::Position([30, 30]),
                                easing: EasingType::EaseInOut,
                            });
                            position_keyframes.push(UIKeyframe {
                                id: Uuid::new_v4().to_string(),
                                time: Duration::from_secs(20),
                                value: KeyframeValue::Position([40, 40]),
                                easing: EasingType::EaseInOut,
                            });

                            let mut position_prop = AnimationProperty {
                                name: "Position".to_string(),
                                property_path: "position".to_string(),
                                children: Vec::new(),
                                keyframes: position_keyframes,
                                depth: 0,
                            };

                            let mut rotation_keyframes = Vec::new();

                            rotation_keyframes.push(UIKeyframe {
                                id: Uuid::new_v4().to_string(),
                                time: Duration::from_secs(0),
                                value: KeyframeValue::Rotation(0),
                                easing: EasingType::EaseInOut,
                            });
                            rotation_keyframes.push(UIKeyframe {
                                id: Uuid::new_v4().to_string(),
                                time: Duration::from_millis(2500),
                                value: KeyframeValue::Rotation(0),
                                easing: EasingType::EaseInOut,
                            });
                            rotation_keyframes.push(UIKeyframe {
                                id: Uuid::new_v4().to_string(),
                                time: Duration::from_secs(5),
                                value: KeyframeValue::Rotation(0),
                                easing: EasingType::EaseInOut,
                            });
                            rotation_keyframes.push(UIKeyframe {
                                id: Uuid::new_v4().to_string(),
                                time: Duration::from_secs(15),
                                value: KeyframeValue::Rotation(0),
                                easing: EasingType::EaseInOut,
                            });
                            rotation_keyframes.push(UIKeyframe {
                                id: Uuid::new_v4().to_string(),
                                time: Duration::from_millis(17500),
                                value: KeyframeValue::Rotation(0),
                                easing: EasingType::EaseInOut,
                            });
                            rotation_keyframes.push(UIKeyframe {
                                id: Uuid::new_v4().to_string(),
                                time: Duration::from_secs(20),
                                value: KeyframeValue::Rotation(0),
                                easing: EasingType::EaseInOut,
                            });

                            let mut rotation_prop = AnimationProperty {
                                name: "Rotation".to_string(),
                                property_path: "rotation".to_string(),
                                children: Vec::new(),
                                keyframes: rotation_keyframes,
                                depth: 0,
                            };

                            let mut scale_keyframes = Vec::new();

                            scale_keyframes.push(UIKeyframe {
                                id: Uuid::new_v4().to_string(),
                                time: Duration::from_secs(0),
                                value: KeyframeValue::Scale(100),
                                easing: EasingType::EaseInOut,
                            });
                            scale_keyframes.push(UIKeyframe {
                                id: Uuid::new_v4().to_string(),
                                time: Duration::from_millis(2500),
                                value: KeyframeValue::Scale(100),
                                easing: EasingType::EaseInOut,
                            });
                            scale_keyframes.push(UIKeyframe {
                                id: Uuid::new_v4().to_string(),
                                time: Duration::from_secs(5),
                                value: KeyframeValue::Scale(100),
                                easing: EasingType::EaseInOut,
                            });
                            scale_keyframes.push(UIKeyframe {
                                id: Uuid::new_v4().to_string(),
                                time: Duration::from_secs(15),
                                value: KeyframeValue::Scale(100),
                                easing: EasingType::EaseInOut,
                            });
                            scale_keyframes.push(UIKeyframe {
                                id: Uuid::new_v4().to_string(),
                                time: Duration::from_millis(17500),
                                value: KeyframeValue::Scale(100),
                                easing: EasingType::EaseInOut,
                            });
                            scale_keyframes.push(UIKeyframe {
                                id: Uuid::new_v4().to_string(),
                                time: Duration::from_secs(20),
                                value: KeyframeValue::Scale(100),
                                easing: EasingType::EaseInOut,
                            });

                            let mut scale_prop = AnimationProperty {
                                name: "Scale".to_string(),
                                property_path: "scale".to_string(),
                                children: Vec::new(),
                                keyframes: scale_keyframes,
                                depth: 0,
                            };

                            let mut perspective_x_keyframes = Vec::new();

                            perspective_x_keyframes.push(UIKeyframe {
                                id: Uuid::new_v4().to_string(),
                                time: Duration::from_secs(0),
                                value: KeyframeValue::PerspectiveX(0),
                                easing: EasingType::EaseInOut,
                            });
                            perspective_x_keyframes.push(UIKeyframe {
                                id: Uuid::new_v4().to_string(),
                                time: Duration::from_millis(2500),
                                value: KeyframeValue::PerspectiveX(0),
                                easing: EasingType::EaseInOut,
                            });
                            perspective_x_keyframes.push(UIKeyframe {
                                id: Uuid::new_v4().to_string(),
                                time: Duration::from_secs(5),
                                value: KeyframeValue::PerspectiveX(0),
                                easing: EasingType::EaseInOut,
                            });
                            perspective_x_keyframes.push(UIKeyframe {
                                id: Uuid::new_v4().to_string(),
                                time: Duration::from_secs(15),
                                value: KeyframeValue::PerspectiveX(0),
                                easing: EasingType::EaseInOut,
                            });
                            perspective_x_keyframes.push(UIKeyframe {
                                id: Uuid::new_v4().to_string(),
                                time: Duration::from_millis(17500),
                                value: KeyframeValue::PerspectiveX(0),
                                easing: EasingType::EaseInOut,
                            });
                            perspective_x_keyframes.push(UIKeyframe {
                                id: Uuid::new_v4().to_string(),
                                time: Duration::from_secs(20),
                                value: KeyframeValue::PerspectiveX(0),
                                easing: EasingType::EaseInOut,
                            });

                            let mut perspective_x_prop = AnimationProperty {
                                name: "Perspective X".to_string(),
                                property_path: "perspective_x".to_string(),
                                children: Vec::new(),
                                keyframes: perspective_x_keyframes,
                                depth: 0,
                            };

                            let mut perspective_y_keyframes = Vec::new();

                            perspective_y_keyframes.push(UIKeyframe {
                                id: Uuid::new_v4().to_string(),
                                time: Duration::from_secs(0),
                                value: KeyframeValue::PerspectiveY(0),
                                easing: EasingType::EaseInOut,
                            });
                            perspective_y_keyframes.push(UIKeyframe {
                                id: Uuid::new_v4().to_string(),
                                time: Duration::from_millis(2500),
                                value: KeyframeValue::PerspectiveY(0),
                                easing: EasingType::EaseInOut,
                            });
                            perspective_y_keyframes.push(UIKeyframe {
                                id: Uuid::new_v4().to_string(),
                                time: Duration::from_secs(5),
                                value: KeyframeValue::PerspectiveY(0),
                                easing: EasingType::EaseInOut,
                            });
                            perspective_y_keyframes.push(UIKeyframe {
                                id: Uuid::new_v4().to_string(),
                                time: Duration::from_secs(15),
                                value: KeyframeValue::PerspectiveY(0),
                                easing: EasingType::EaseInOut,
                            });
                            perspective_y_keyframes.push(UIKeyframe {
                                id: Uuid::new_v4().to_string(),
                                time: Duration::from_millis(17500),
                                value: KeyframeValue::PerspectiveY(0),
                                easing: EasingType::EaseInOut,
                            });
                            perspective_y_keyframes.push(UIKeyframe {
                                id: Uuid::new_v4().to_string(),
                                time: Duration::from_secs(20),
                                value: KeyframeValue::PerspectiveY(0),
                                easing: EasingType::EaseInOut,
                            });

                            let mut perspective_y_prop = AnimationProperty {
                                name: "Perspective Y".to_string(),
                                property_path: "perspective_y".to_string(),
                                children: Vec::new(),
                                keyframes: perspective_y_keyframes,
                                depth: 0,
                            };

                            let mut opacity_keyframes = Vec::new();

                            opacity_keyframes.push(UIKeyframe {
                                id: Uuid::new_v4().to_string(),
                                time: Duration::from_secs(0),
                                value: KeyframeValue::Opacity(100),
                                easing: EasingType::EaseInOut,
                            });
                            opacity_keyframes.push(UIKeyframe {
                                id: Uuid::new_v4().to_string(),
                                time: Duration::from_millis(2500),
                                value: KeyframeValue::Opacity(100),
                                easing: EasingType::EaseInOut,
                            });
                            opacity_keyframes.push(UIKeyframe {
                                id: Uuid::new_v4().to_string(),
                                time: Duration::from_secs(5),
                                value: KeyframeValue::Opacity(100),
                                easing: EasingType::EaseInOut,
                            });
                            opacity_keyframes.push(UIKeyframe {
                                id: Uuid::new_v4().to_string(),
                                time: Duration::from_secs(15),
                                value: KeyframeValue::Opacity(100),
                                easing: EasingType::EaseInOut,
                            });
                            opacity_keyframes.push(UIKeyframe {
                                id: Uuid::new_v4().to_string(),
                                time: Duration::from_millis(17500),
                                value: KeyframeValue::Opacity(100),
                                easing: EasingType::EaseInOut,
                            });
                            opacity_keyframes.push(UIKeyframe {
                                id: Uuid::new_v4().to_string(),
                                time: Duration::from_secs(20),
                                value: KeyframeValue::Opacity(100),
                                easing: EasingType::EaseInOut,
                            });

                            let mut opacity_prop = AnimationProperty {
                                name: "Opacity".to_string(),
                                property_path: "opacity".to_string(),
                                children: Vec::new(),
                                keyframes: opacity_keyframes,
                                depth: 0,
                            };

                            properties.push(position_prop);
                            properties.push(rotation_prop);
                            properties.push(scale_prop);
                            properties.push(perspective_x_prop);
                            properties.push(perspective_y_prop);
                            properties.push(opacity_prop);

                            animation_data.update(|c| {
                                *c = Some(AnimationData {
                                    id: Uuid::new_v4().to_string(),
                                    polygon_id: polygon_id.to_string(),
                                    duration: Duration::from_secs(20),
                                    properties: properties,
                                })
                            });
                        }

                        drop(editor_state);
                    }
                }) as Box<dyn FnMut(Uuid, PolygonConfig) + Send>,
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
        }
    });

    container((
        tab_interface(
            gpu_helper.clone(),
            editor_state.clone(),
            editor,
            viewport.clone(),
            // polygon_selected,
        ),
        dyn_container(
            move || sequence_selected.get(),
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
                    let state = TimelineState {
                        current_time: Duration::from_secs_f64(0.0),
                        zoom_level: 1.0,
                        scroll_offset: 0.0,
                        // selected_keyframes: Vec::new(),
                        property_expansions: im::HashMap::from_iter([
                            ("position".to_string(), true),
                            ("rotation".to_string(), true),
                        ]),
                        dragging: None,
                        hovered_keyframe: None,
                        selected_keyframes,
                    };

                    let config = TimelineConfig {
                        width: 1200.0,
                        height: 300.0,
                        header_height: 30.0,
                        property_width: 200.0,
                        row_height: 24.0,
                        // offset_x: 325.0,
                        // offset_y: 300.0,
                        offset_x: 0.0,
                        offset_y: 0.0,
                    };

                    let keyframe_timeline = create_timeline(state, config, animation_data);

                    h_stack((
                        properties_view(
                            state_cloned2.clone(),
                            gpu_cloned2.clone(),
                            editor_cloned4.clone(),
                            viewport_cloned2.clone(),
                            polygon_selected,
                            selected_polygon_id,
                            selected_polygon_data,
                        ),
                        keyframe_timeline,
                    ))
                    .into_any()
                } else {
                    empty().into_any()
                }
            },
        ),
    ))
}
