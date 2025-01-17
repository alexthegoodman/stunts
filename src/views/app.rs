use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::fs;
use std::path::Path;
use std::rc::{Rc, Weak};
use std::str::FromStr;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use bytemuck::Contiguous;
use floem::common::{card_styles, nav_button, simple_button};
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
use stunts_engine::editor::{
    string_to_f32, Editor, ImageItemClickHandler, OnHandleMouseUp, OnMouseUp, Point,
    PolygonClickHandler, TextItemClickHandler, Viewport, WindowSize,
};
use stunts_engine::polygon::{PolygonConfig, SavedPoint, Stroke};
use stunts_engine::st_image::StImageConfig;
use stunts_engine::text_due::TextRendererConfig;
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
use crate::helpers::saved_state::SavedState;
use crate::helpers::utilities::save_saved_state_raw;
use crate::views::keyframe_panel::update_keyframe;
use stunts_engine::animations::{
    AnimationData, AnimationProperty, EasingType, KeyframeValue, ObjectType, Sequence, UIKeyframe,
};

use super::aside::tab_interface;
use super::editor_settings::editor_settings;
use super::inputs::styled_input;
use super::keyframe_panel::keyframe_properties_view;
use super::keyframe_timeline::{create_timeline, TimelineConfig, TimelineState};
use super::project_browser::project_browser;
use super::properties_panel::{image_properties_view, properties_view, text_properties_view};
use super::sequence_panel::sequence_panel;

fn find_object_type(last_saved_state: &SavedState, object_id: &Uuid) -> Option<ObjectType> {
    // Check active polygons
    if last_saved_state.sequences.iter().any(|s| {
        s.active_polygons
            .iter()
            .any(|ap| ap.id == object_id.to_string())
    }) {
        return Some(ObjectType::Polygon);
    }

    // Check active images
    if last_saved_state.sequences.iter().any(|s| {
        s.active_image_items
            .iter()
            .any(|ai| ai.id == object_id.to_string())
    }) {
        return Some(ObjectType::ImageItem);
    }

    // Check active text
    if last_saved_state.sequences.iter().any(|s| {
        s.active_text_items
            .iter()
            .any(|at| at.id == object_id.to_string())
    }) {
        return Some(ObjectType::TextItem);
    }

    None
}

fn set_polygon_selected(
    editor_state: Arc<Mutex<EditorState>>,
    text_selected_ref: Arc<Mutex<RwSignal<bool>>>,
    polygon_selected_ref: Arc<Mutex<RwSignal<bool>>>,
    image_selected_ref: Arc<Mutex<RwSignal<bool>>>,
    selected_text_id_ref: Arc<Mutex<RwSignal<Uuid>>>,
    selected_polygon_id_ref: Arc<Mutex<RwSignal<Uuid>>>,
    selected_image_id_ref: Arc<Mutex<RwSignal<Uuid>>>,
    selected_polygon_data_ref: Arc<Mutex<RwSignal<PolygonConfig>>>,
    polygon_id: Uuid,
    polygon_data: PolygonConfig,
) {
    if let Ok(mut polygon_selected) = polygon_selected_ref.lock() {
        polygon_selected.update(|c| {
            *c = true;
        });
        if let Ok(mut text_selected) = text_selected_ref.lock() {
            text_selected.update(|c| {
                *c = false;
            });
        }
        if let Ok(mut image_selected) = image_selected_ref.lock() {
            image_selected.update(|c| {
                *c = false;
            });
        }
    }
    if let Ok(mut selected_polygon_id) = selected_polygon_id_ref.lock() {
        selected_polygon_id.update(|c| {
            *c = polygon_id;
        });
        if let Ok(mut selected_text_id) = selected_text_id_ref.lock() {
            selected_text_id.update(|c| {
                *c = Uuid::nil();
            });
        }
        if let Ok(mut selected_image_id) = selected_image_id_ref.lock() {
            selected_image_id.update(|c| {
                *c = Uuid::nil();
            });
        }

        let mut editor_state = editor_state.lock().unwrap();

        editor_state.selected_polygon_id = polygon_id;
        editor_state.polygon_selected = true;

        editor_state.selected_text_id = Uuid::nil();
        editor_state.text_selected = false;
        editor_state.selected_image_id = Uuid::nil();
        editor_state.image_selected = false;

        drop(editor_state);
    }
    if let Ok(mut selected_polygon_data) = selected_polygon_data_ref.lock() {
        selected_polygon_data.update(|c| {
            *c = polygon_data;
        });
        // no need to update stale data for other object types as it will be overwritten later
    }
}

fn set_image_selected(
    editor_state: Arc<Mutex<EditorState>>,
    text_selected_ref: Arc<Mutex<RwSignal<bool>>>,
    polygon_selected_ref: Arc<Mutex<RwSignal<bool>>>,
    image_selected_ref: Arc<Mutex<RwSignal<bool>>>,
    selected_text_id_ref: Arc<Mutex<RwSignal<Uuid>>>,
    selected_polygon_id_ref: Arc<Mutex<RwSignal<Uuid>>>,
    selected_image_id_ref: Arc<Mutex<RwSignal<Uuid>>>,
    selected_image_data_ref: Arc<Mutex<RwSignal<StImageConfig>>>,
    image_id: Uuid,
    image_data: StImageConfig,
) {
    if let Ok(mut image_selected) = image_selected_ref.lock() {
        image_selected.update(|c| {
            *c = true;
        });
        if let Ok(mut polygon_selected) = polygon_selected_ref.lock() {
            polygon_selected.update(|c| {
                *c = false;
            });
        }
        if let Ok(mut text_selected) = text_selected_ref.lock() {
            text_selected.update(|c| {
                *c = false;
            });
        }
    }
    if let Ok(mut selected_image_id) = selected_image_id_ref.lock() {
        selected_image_id.update(|c| {
            *c = image_id;
        });
        if let Ok(mut selected_polygon_id) = selected_polygon_id_ref.lock() {
            selected_polygon_id.update(|c| {
                *c = Uuid::nil();
            });
        }
        if let Ok(mut selected_text_id) = selected_text_id_ref.lock() {
            selected_text_id.update(|c| {
                *c = Uuid::nil();
            });
        }

        let mut editor_state = editor_state.lock().unwrap();

        editor_state.selected_image_id = image_id;
        editor_state.image_selected = true;

        editor_state.selected_text_id = Uuid::nil();
        editor_state.text_selected = false;
        editor_state.selected_polygon_id = Uuid::nil();
        editor_state.polygon_selected = false;

        drop(editor_state);
    }
    if let Ok(mut selected_image_data) = selected_image_data_ref.lock() {
        selected_image_data.update(|c| {
            *c = image_data;
        });
    }
}

fn set_text_selected(
    editor_state: Arc<Mutex<EditorState>>,
    text_selected_ref: Arc<Mutex<RwSignal<bool>>>,
    polygon_selected_ref: Arc<Mutex<RwSignal<bool>>>,
    image_selected_ref: Arc<Mutex<RwSignal<bool>>>,
    selected_text_id_ref: Arc<Mutex<RwSignal<Uuid>>>,
    selected_polygon_id_ref: Arc<Mutex<RwSignal<Uuid>>>,
    selected_image_id_ref: Arc<Mutex<RwSignal<Uuid>>>,
    selected_text_data_ref: Arc<Mutex<RwSignal<TextRendererConfig>>>,
    text_id: Uuid,
    text_data: TextRendererConfig,
) {
    if let Ok(mut text_selected) = text_selected_ref.lock() {
        text_selected.update(|c| {
            *c = true;
        });
        if let Ok(mut polygon_selected) = polygon_selected_ref.lock() {
            polygon_selected.update(|c| {
                *c = false;
            });
        }
        if let Ok(mut image_selected) = image_selected_ref.lock() {
            image_selected.update(|c| {
                *c = false;
            });
        }
    }
    if let Ok(mut selected_text_id) = selected_text_id_ref.lock() {
        selected_text_id.update(|c| {
            *c = text_id;
        });
        if let Ok(mut selected_polygon_id) = selected_polygon_id_ref.lock() {
            selected_polygon_id.update(|c| {
                *c = Uuid::nil();
            });
        }
        if let Ok(mut selected_image_id) = selected_image_id_ref.lock() {
            selected_image_id.update(|c| {
                *c = Uuid::nil();
            });
        }

        let mut editor_state = editor_state.lock().unwrap();

        editor_state.selected_text_id = text_id;
        editor_state.text_selected = true;

        editor_state.selected_polygon_id = Uuid::nil();
        editor_state.polygon_selected = false;
        editor_state.selected_image_id = Uuid::nil();
        editor_state.image_selected = false;

        drop(editor_state);
    }
    if let Ok(mut selected_text_data) = selected_text_data_ref.lock() {
        selected_text_data.update(|c| {
            *c = text_data;
        });
    }
}

pub fn project_view(
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
    let editor_cloned8 = Arc::clone(&editor);
    let editor_cloned9 = Arc::clone(&editor);

    let state_cloned = Arc::clone(&editor_state);
    let state_cloned2 = Arc::clone(&editor_state);
    let state_cloned3 = Arc::clone(&editor_state);
    let state_cloned4 = Arc::clone(&editor_state);
    let state_cloned5 = Arc::clone(&editor_state);
    let state_cloned6 = Arc::clone(&editor_state);
    let state_cloned7 = Arc::clone(&editor_state);
    let state_cloned8 = Arc::clone(&editor_state);
    let state_cloned9 = Arc::clone(&editor_state);

    let gpu_cloned = Arc::clone(&gpu_helper);
    let gpu_cloned2 = Arc::clone(&gpu_helper);
    let gpu_cloned3 = Arc::clone(&gpu_helper);
    let gpu_cloned4 = Arc::clone(&gpu_helper);
    let gpu_cloned5 = Arc::clone(&gpu_helper);

    let viewport_cloned = Arc::clone(&viewport);
    let viewport_cloned2 = Arc::clone(&viewport);
    let viewport_cloned3 = Arc::clone(&viewport);
    let viewport_cloned4 = Arc::clone(&viewport);
    let viewport_cloned5 = Arc::clone(&viewport);

    // set in sequence_panel
    let sequence_selected = create_rw_signal(false);
    let selected_sequence_id = create_rw_signal(String::new());
    let selected_sequence_data: RwSignal<Sequence> = create_rw_signal(Sequence {
        id: String::new(),
        active_polygons: Vec::new(),
        polygon_motion_paths: Vec::new(),
        active_text_items: Vec::new(),
        active_image_items: Vec::new(),
    });

    // set
    let polygon_selected: RwSignal<bool> = create_rw_signal(false);
    let selected_polygon_id: RwSignal<Uuid> = create_rw_signal(Uuid::nil());
    let selected_polygon_data: RwSignal<PolygonConfig> = create_rw_signal(PolygonConfig {
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

    let image_selected: RwSignal<bool> = create_rw_signal(false);
    let selected_image_id: RwSignal<Uuid> = create_rw_signal(Uuid::nil());
    let selected_image_data: RwSignal<StImageConfig> = create_rw_signal(StImageConfig {
        id: String::new(),
        name: String::new(),
        path: String::new(),
        dimensions: (100, 100),
        position: Point { x: 0.0, y: 0.0 },
    });

    let text_selected: RwSignal<bool> = create_rw_signal(false);
    let selected_text_id: RwSignal<Uuid> = create_rw_signal(Uuid::nil());
    let selected_text_data: RwSignal<TextRendererConfig> = create_rw_signal(TextRendererConfig {
        id: Uuid::nil(),
        name: String::new(),
        text: String::new(),
        font_family: "Aleo".to_string(),
        dimensions: (100.0, 100.0),
        position: Point { x: 0.0, y: 0.0 },
    });

    let animation_data: RwSignal<Option<AnimationData>> = create_rw_signal(None);
    let selected_keyframes: RwSignal<Vec<UIKeyframe>> = create_rw_signal(Vec::new());

    let image_selected_ref = Arc::new(Mutex::new(image_selected));
    let selected_image_id_ref = Arc::new(Mutex::new(selected_image_id));
    let selected_image_data_ref = Arc::new(Mutex::new(selected_image_data));

    let text_selected_ref = Arc::new(Mutex::new(text_selected));
    let selected_text_id_ref = Arc::new(Mutex::new(selected_text_id));
    let selected_text_data_ref = Arc::new(Mutex::new(selected_text_data));

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
        let text_selected_ref = Arc::clone(&text_selected_ref);
        let image_selected_ref = Arc::clone(&image_selected_ref);
        let selected_polygon_id_ref = Arc::clone(&selected_polygon_id_ref);
        let selected_polygon_data_ref = Arc::clone(&selected_polygon_data_ref);
        let selected_text_id_ref = Arc::clone(&selected_text_id_ref);
        let selected_text_data_ref = Arc::clone(&selected_text_data_ref);
        let selected_image_id_ref = Arc::clone(&selected_image_id_ref);
        let selected_image_data_ref = Arc::clone(&selected_image_data_ref);
        let animation_data_ref = Arc::clone(&animation_data_ref);

        move || {
            let editor_state = editor_state.clone();
            let polygon_selected_ref = polygon_selected_ref.clone();
            let text_selected_ref = text_selected_ref.clone();
            let image_selected_ref = image_selected_ref.clone();
            let selected_polygon_id_ref = selected_polygon_id_ref.clone();
            let selected_polygon_data_ref = selected_polygon_data_ref.clone();
            let selected_text_id_ref = selected_text_id_ref.clone();
            let selected_text_data_ref = selected_text_data_ref.clone();
            let selected_image_id_ref = selected_image_id_ref.clone();
            let selected_image_data_ref = selected_image_data_ref.clone();
            let animation_data_ref = animation_data_ref.clone();

            Some(
                Box::new(move |polygon_id: Uuid, polygon_data: PolygonConfig| {
                    // cannot lock editor here! probably because called from Editor
                    // {
                    //     let mut editor = new_editor.lock().unwrap();
                    //     // Update editor as needed
                    // }

                    set_polygon_selected(
                        editor_state.clone(),
                        text_selected_ref.clone(),
                        polygon_selected_ref.clone(),
                        image_selected_ref.clone(),
                        selected_text_id_ref.clone(),
                        selected_polygon_id_ref.clone(),
                        selected_image_id_ref.clone(),
                        selected_polygon_data_ref.clone(),
                        polygon_id,
                        polygon_data,
                    );

                    if let Ok(mut animation_data) = animation_data_ref.lock() {
                        let editor_state = editor_state.lock().unwrap();
                        let saved_state = editor_state
                            .saved_state
                            .as_ref()
                            .expect("Couldn't get Saved State");

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

    let handle_image_click: Arc<ImageItemClickHandler> = Arc::new({
        let editor_state = editor_state.clone();
        let polygon_selected_ref = Arc::clone(&polygon_selected_ref);
        let text_selected_ref = Arc::clone(&text_selected_ref);
        let image_selected_ref = Arc::clone(&image_selected_ref);
        let selected_polygon_id_ref = Arc::clone(&selected_polygon_id_ref);
        let selected_polygon_data_ref = Arc::clone(&selected_polygon_data_ref);
        let selected_text_id_ref = Arc::clone(&selected_text_id_ref);
        let selected_text_data_ref = Arc::clone(&selected_text_data_ref);
        let selected_image_id_ref = Arc::clone(&selected_image_id_ref);
        let selected_image_data_ref = Arc::clone(&selected_image_data_ref);
        let animation_data_ref = Arc::clone(&animation_data_ref);

        move || {
            let editor_state = editor_state.clone();
            let polygon_selected_ref = polygon_selected_ref.clone();
            let text_selected_ref = text_selected_ref.clone();
            let image_selected_ref = image_selected_ref.clone();
            let selected_polygon_id_ref = selected_polygon_id_ref.clone();
            let selected_polygon_data_ref = selected_polygon_data_ref.clone();
            let selected_text_id_ref = selected_text_id_ref.clone();
            let selected_text_data_ref = selected_text_data_ref.clone();
            let selected_image_id_ref = selected_image_id_ref.clone();
            let selected_image_data_ref = selected_image_data_ref.clone();
            let animation_data_ref = animation_data_ref.clone();

            Some(Box::new(move |image_id: Uuid, image_data: StImageConfig| {
                // cannot lock editor here! probably because called from Editor
                // {
                //     let mut editor = new_editor.lock().unwrap();
                //     // Update editor as needed
                // }

                set_image_selected(
                    editor_state.clone(),
                    text_selected_ref.clone(),
                    polygon_selected_ref.clone(),
                    image_selected_ref.clone(),
                    selected_text_id_ref.clone(),
                    selected_polygon_id_ref.clone(),
                    selected_image_id_ref.clone(),
                    selected_image_data_ref.clone(),
                    image_id,
                    image_data,
                );

                if let Ok(mut animation_data) = animation_data_ref.lock() {
                    let editor_state = editor_state.lock().unwrap();
                    let saved_state = editor_state
                        .saved_state
                        .as_ref()
                        .expect("Couldn't get Saved State");

                    let saved_animation_data = saved_state
                        .sequences
                        .iter()
                        .flat_map(|s| s.polygon_motion_paths.iter())
                        .find(|p| p.polygon_id == image_id.to_string());

                    if let Some(image_animation_data) = saved_animation_data {
                        animation_data.update(|c| {
                            *c = Some(image_animation_data.clone());
                        });
                    } else {
                        // image is not saved animation data
                        // image_index,time,width,height,x,y,rotation,scale,perspective_x,perspective_y,opacity
                    }

                    drop(editor_state);
                }
            }) as Box<dyn FnMut(Uuid, StImageConfig) + Send>)
        }
    });

    let handle_text_click: Arc<TextItemClickHandler> = Arc::new({
        let editor_state = editor_state.clone();
        let polygon_selected_ref = Arc::clone(&polygon_selected_ref);
        let text_selected_ref = Arc::clone(&text_selected_ref);
        let image_selected_ref = Arc::clone(&image_selected_ref);
        let selected_polygon_id_ref = Arc::clone(&selected_polygon_id_ref);
        let selected_polygon_data_ref = Arc::clone(&selected_polygon_data_ref);
        let selected_text_id_ref = Arc::clone(&selected_text_id_ref);
        let selected_text_data_ref = Arc::clone(&selected_text_data_ref);
        let selected_image_id_ref = Arc::clone(&selected_image_id_ref);
        let selected_image_data_ref = Arc::clone(&selected_image_data_ref);
        let animation_data_ref = Arc::clone(&animation_data_ref);

        move || {
            let editor_state = editor_state.clone();
            let polygon_selected_ref = polygon_selected_ref.clone();
            let text_selected_ref = text_selected_ref.clone();
            let image_selected_ref = image_selected_ref.clone();
            let selected_polygon_id_ref = selected_polygon_id_ref.clone();
            let selected_polygon_data_ref = selected_polygon_data_ref.clone();
            let selected_text_id_ref = selected_text_id_ref.clone();
            let selected_text_data_ref = selected_text_data_ref.clone();
            let selected_image_id_ref = selected_image_id_ref.clone();
            let selected_image_data_ref = selected_image_data_ref.clone();
            let animation_data_ref = animation_data_ref.clone();

            Some(
                Box::new(move |text_id: Uuid, text_data: TextRendererConfig| {
                    // cannot lock editor here! probably because called from Editor
                    // {
                    //     let mut editor = new_editor.lock().unwrap();
                    //     // Update editor as needed
                    // }

                    set_text_selected(
                        editor_state.clone(),
                        text_selected_ref.clone(),
                        polygon_selected_ref.clone(),
                        image_selected_ref.clone(),
                        selected_text_id_ref.clone(),
                        selected_polygon_id_ref.clone(),
                        selected_image_id_ref.clone(),
                        selected_text_data_ref.clone(),
                        text_id,
                        text_data,
                    );

                    if let Ok(mut animation_data) = animation_data_ref.lock() {
                        let editor_state = editor_state.lock().unwrap();
                        let saved_state = editor_state
                            .saved_state
                            .as_ref()
                            .expect("Couldn't get Saved State");

                        let saved_animation_data = saved_state
                            .sequences
                            .iter()
                            .flat_map(|s| s.polygon_motion_paths.iter())
                            .find(|p| p.polygon_id == text_id.to_string());

                        if let Some(text_animation_data) = saved_animation_data {
                            animation_data.update(|c| {
                                *c = Some(text_animation_data.clone());
                            });
                        } else {
                            // text is not saved animation data
                            // text_index,time,width,height,x,y,rotation,scale,perspective_x,perspective_y,opacity
                        }

                        drop(editor_state);
                    }
                }) as Box<dyn FnMut(Uuid, TextRendererConfig) + Send>,
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

            Some(Box::new(move |object_id: Uuid, point: Point| {
                // cannot lock editor here! probably because called from Editor
                // {
                //     let mut editor = new_editor.lock().unwrap();
                //     // Update editor as needed
                // }

                // let value = string_to_f32(&value).map_err(|_| "Couldn't convert string to f32").expect("Couldn't convert string to f32");

                let mut current_animation_data = animation_data
                    .get()
                    .expect("Couldn't get current Animation Data");
                let mut current_keyframe = selected_keyframes.get();

                let mut editor_state = editor_state.lock().unwrap();

                if let Some(current_keyframe) = current_keyframe.get_mut(0) {
                    // let mut current_keyframe = current_keyframe.get_mut(0).expect("Couldn't get Selected Keyframe");
                    let mut current_sequence = selected_sequence_data.get();
                    // let current_polygon = selected_polygon_data.read();
                    // let current_polygon = current_polygon.borrow();

                    // update keyframe
                    current_keyframe.value =
                        KeyframeValue::Position([point.x as i32, point.y as i32]);

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
                } else {
                    let last_saved_state = editor_state
                        .saved_state
                        .as_mut()
                        .expect("Couldn't get Saved State");

                    let object_type = find_object_type(&last_saved_state, &object_id);

                    if let Some(object_type) = object_type.clone() {
                        last_saved_state.sequences.iter_mut().for_each(|s| {
                            if s.id == selected_sequence_id.get() {
                                match object_type {
                                    ObjectType::Polygon => {
                                        s.active_polygons.iter_mut().for_each(|ap| {
                                            if ap.id == object_id.to_string() {
                                                ap.position = SavedPoint {
                                                    x: point.x as i32,
                                                    y: point.y as i32,
                                                }
                                            }
                                        });
                                    }
                                    ObjectType::TextItem => {
                                        s.active_text_items.iter_mut().for_each(|tr| {
                                            if tr.id == object_id.to_string() {
                                                tr.position = SavedPoint {
                                                    x: point.x as i32,
                                                    y: point.y as i32,
                                                }
                                            }
                                        });
                                    }
                                    ObjectType::ImageItem => {
                                        s.active_image_items.iter_mut().for_each(|si| {
                                            if si.id == object_id.to_string() {
                                                si.position = SavedPoint {
                                                    x: point.x as i32,
                                                    y: point.y as i32,
                                                }
                                            }
                                        });
                                    }
                                }
                            }
                        });

                        // TODO: probably perf hit with larger files, or does it get released?
                        let new_saved_state = last_saved_state.to_owned();

                        save_saved_state_raw(new_saved_state);

                        // drop(editor_state);

                        println!("Position updated!");
                    }
                }

                // let mut editor = editor_cloned7.lock().unwrap();
                // editor.update_motion_paths(&selected_sequence_data.get());

                // println!("Motion Paths updated!");

                (selected_sequence_data.get(), selected_keyframes.get())
            })
                as Box<
                    dyn FnMut(Uuid, Point) -> (Sequence, Vec<UIKeyframe>) + Send,
                >)
        }
    });

    let on_handle_mouse_up: Arc<OnHandleMouseUp> = Arc::new({
        let editor_state = editor_state.clone();
        let polygon_selected_ref = Arc::clone(&polygon_selected_ref);
        let text_selected_ref = Arc::clone(&text_selected_ref);
        let image_selected_ref = Arc::clone(&image_selected_ref);
        let selected_polygon_id_ref = Arc::clone(&selected_polygon_id_ref);
        let selected_polygon_data_ref = Arc::clone(&selected_polygon_data_ref);
        let selected_text_id_ref = Arc::clone(&selected_text_id_ref);
        let selected_text_data_ref = Arc::clone(&selected_text_data_ref);
        let selected_image_id_ref = Arc::clone(&selected_image_id_ref);
        let selected_image_data_ref = Arc::clone(&selected_image_data_ref);
        let animation_data_ref = Arc::clone(&animation_data_ref);

        move || {
            let editor_state = editor_state.clone();
            let polygon_selected_ref = polygon_selected_ref.clone();
            let text_selected_ref = text_selected_ref.clone();
            let image_selected_ref = image_selected_ref.clone();
            let selected_polygon_id_ref = selected_polygon_id_ref.clone();
            let selected_polygon_data_ref = selected_polygon_data_ref.clone();
            let selected_text_id_ref = selected_text_id_ref.clone();
            let selected_text_data_ref = selected_text_data_ref.clone();
            let selected_image_id_ref = selected_image_id_ref.clone();
            let selected_image_data_ref = selected_image_data_ref.clone();
            let animation_data_ref = animation_data_ref.clone();

            Some(
                Box::new(move |keyframe_id: Uuid, object_id: Uuid, point: Point| {
                    // cannot lock editor here! probably because called from Editor
                    // {
                    //     let mut editor = new_editor.lock().unwrap();
                    //     // Update editor as needed
                    // }

                    // let value = string_to_f32(&value).map_err(|_| "Couldn't convert string to f32").expect("Couldn't convert string to f32");

                    println!("Updating keyframe via handle...");

                    if (!sequence_selected.get()) {
                        return (selected_sequence_data.get(), selected_keyframes.get());
                    }

                    let selected_sequence = selected_sequence_data.get();

                    let is_polygon = selected_sequence
                        .active_polygons
                        .iter()
                        .find(|p| p.id == object_id.to_string());
                    let is_image = selected_sequence
                        .active_image_items
                        .iter()
                        .find(|i| i.id == object_id.to_string());
                    let is_text = selected_sequence
                        .active_text_items
                        .iter()
                        .find(|t| t.id == object_id.to_string());

                    if let Some(polygon) = is_polygon {
                        set_polygon_selected(
                            editor_state.clone(),
                            text_selected_ref.clone(),
                            polygon_selected_ref.clone(),
                            image_selected_ref.clone(),
                            selected_text_id_ref.clone(),
                            selected_polygon_id_ref.clone(),
                            selected_image_id_ref.clone(),
                            selected_polygon_data_ref.clone(),
                            object_id,
                            PolygonConfig {
                                id: Uuid::from_str(&polygon.id)
                                    .expect("Couldn't convert string to uuid"),
                                name: polygon.name.clone(),
                                // TODO: support triangles and other shapes by saving points
                                points: vec![
                                    Point { x: 0.0, y: 0.0 },
                                    Point { x: 1.0, y: 0.0 },
                                    Point { x: 1.0, y: 1.0 },
                                    Point { x: 0.0, y: 1.0 },
                                ],
                                fill: [
                                    polygon.fill[0] as f32,
                                    polygon.fill[1] as f32,
                                    polygon.fill[2] as f32,
                                    polygon.fill[3] as f32,
                                ],
                                dimensions: (
                                    polygon.dimensions.0 as f32,
                                    polygon.dimensions.1 as f32,
                                ),
                                position: Point {
                                    x: polygon.position.x as f32,
                                    y: polygon.position.y as f32,
                                },
                                border_radius: polygon.border_radius as f32,
                                stroke: Stroke {
                                    thickness: polygon.stroke.thickness as f32,
                                    fill: [
                                        polygon.stroke.fill[0] as f32,
                                        polygon.stroke.fill[1] as f32,
                                        polygon.stroke.fill[2] as f32,
                                        polygon.stroke.fill[3] as f32,
                                    ],
                                },
                            },
                        );
                    }

                    if let Some(image) = is_image {
                        set_image_selected(
                            editor_state.clone(),
                            text_selected_ref.clone(),
                            polygon_selected_ref.clone(),
                            image_selected_ref.clone(),
                            selected_text_id_ref.clone(),
                            selected_polygon_id_ref.clone(),
                            selected_image_id_ref.clone(),
                            selected_image_data_ref.clone(),
                            object_id,
                            StImageConfig {
                                id: image.id.clone(),
                                name: image.name.clone(),
                                dimensions: image.dimensions,
                                position: Point {
                                    x: image.position.x as f32,
                                    y: image.position.y as f32,
                                },
                                path: image.path.clone(),
                            },
                        );
                    }

                    if let Some(text) = is_text {
                        set_text_selected(
                            editor_state.clone(),
                            text_selected_ref.clone(),
                            polygon_selected_ref.clone(),
                            image_selected_ref.clone(),
                            selected_text_id_ref.clone(),
                            selected_polygon_id_ref.clone(),
                            selected_image_id_ref.clone(),
                            selected_text_data_ref.clone(),
                            object_id,
                            TextRendererConfig {
                                id: Uuid::from_str(&text.id)
                                    .expect("Couldn't convert string to uuid"),
                                name: text.name.clone(),
                                text: text.text.clone(),
                                font_family: text.font_family.clone(),
                                dimensions: (text.dimensions.0 as f32, text.dimensions.1 as f32),
                                position: Point {
                                    x: text.position.x as f32,
                                    y: text.position.y as f32,
                                },
                            },
                        );
                    }

                    if let Ok(mut animation_data) = animation_data_ref.lock() {
                        let editor_state = editor_state.lock().unwrap();
                        let saved_state = editor_state
                            .saved_state
                            .as_ref()
                            .expect("Couldn't get Saved State");

                        let saved_animation_data = saved_state
                            .sequences
                            .iter()
                            .flat_map(|s| s.polygon_motion_paths.iter())
                            .find(|p| p.polygon_id == object_id.to_string());

                        if let Some(object_animation_data) = saved_animation_data {
                            animation_data.update(|c| {
                                *c = Some(object_animation_data.clone());
                            });
                        } else {
                            // text is not saved animation data
                            // text_index,time,width,height,x,y,rotation,scale,perspective_x,perspective_y,opacity
                        }

                        drop(editor_state);
                    }

                    let mut current_animation_data = animation_data
                        .get()
                        .expect("Couldn't get current Animation Data");

                    let mut data = current_animation_data.clone();

                    let current_keyframe = data.properties.iter_mut().find_map(|a| {
                        a.keyframes
                            .iter_mut()
                            .find(|kf| kf.id == keyframe_id.to_string())
                    });

                    // get current_keyframe from handle

                    let mut editor_state = editor_state.lock().unwrap();

                    if let Some(current_keyframe) = current_keyframe {
                        println!("Current keyframe found...");

                        // let mut current_keyframe = current_keyframe.get_mut(0).expect("Couldn't get Selected Keyframe");
                        let mut current_sequence = selected_sequence_data.get();

                        // update keyframe
                        current_keyframe.value =
                            KeyframeValue::Position([point.x as i32, point.y as i32]);

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
                    } else {
                        println!("Current keyframe not found!");
                    }

                    (selected_sequence_data.get(), selected_keyframes.get())
                })
                    as Box<dyn FnMut(Uuid, Uuid, Point) -> (Sequence, Vec<UIKeyframe>) + Send>,
            )
        }
    });

    // Use create_effect to set the handler only once
    create_effect({
        let handle_polygon_click = Arc::clone(&handle_polygon_click);
        let handle_image_click = Arc::clone(&handle_image_click);
        let handle_text_click = Arc::clone(&handle_text_click);
        let editor_cloned3 = Arc::clone(&editor_cloned3);
        let state_cloned5 = Arc::clone(&state_cloned5);
        let viewport_cloned3 = Arc::clone(&viewport_cloned3);

        move |_| {
            let editor_state = state_cloned5.lock().unwrap();

            let saved_state = editor_state
                .saved_state
                .as_ref()
                .expect("Couldn't get saved state");
            let cloned_sequences = saved_state.sequences.clone();

            drop(editor_state);

            let mut editor = editor_cloned3.lock().unwrap();
            let viewport = viewport_cloned3.lock().unwrap();
            let camera = editor.camera.expect("Couldn't get camera");

            // attach object interaction handlers
            editor.handle_polygon_click = Some(Arc::clone(&handle_polygon_click));
            editor.handle_text_click = Some(Arc::clone(&handle_text_click));
            editor.handle_image_click = Some(Arc::clone(&handle_image_click));
            editor.on_mouse_up = Some(Arc::clone(&on_mouse_up));
            editor.on_handle_mouse_up = Some(Arc::clone(&on_handle_mouse_up));

            // restore all objects as hidden, avoids too much loading mid-usage
            editor.polygons = Vec::new();
            editor.text_items = Vec::new();
            editor.image_items = Vec::new();

            let gpu_helper = gpu_cloned3.lock().unwrap();
            let gpu_resources = gpu_helper
                .gpu_resources
                .as_ref()
                .expect("Couldn't get gpu resources");
            let device = &gpu_resources.device;
            let queue = &gpu_resources.queue;

            cloned_sequences.iter().enumerate().for_each(|(i, s)| {
                editor.restore_sequence_objects(
                    &s,
                    WindowSize {
                        width: viewport.width as u32,
                        height: viewport.height as u32,
                    },
                    &camera,
                    true,
                    device,
                    queue,
                );
            });
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
            polygon_selected,
        ),
        dyn_container(
            move || {
                sequence_selected.get()
                    && !polygon_selected.get()
                    && !text_selected.get()
                    && !image_selected.get()
                    && selected_keyframes.get().len() == 0
            },
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
            move || polygon_selected.get() || text_selected.get() || image_selected.get(),
            move |object_selected_real| {
                if object_selected_real {
                    let state_cloned3 = state_cloned3.clone();
                    let state_cloned4 = state_cloned4.clone();
                    let editor_cloned4 = editor_cloned4.clone();
                    let state_cloned6 = state_cloned6.clone();
                    let state_cloned7 = state_cloned7.clone();
                    let editor_cloned5 = editor_cloned5.clone();
                    let editor_cloned6 = editor_cloned6.clone();
                    let gpu_cloned2 = gpu_cloned2.clone();
                    let editor_cloned7 = editor_cloned7.clone();
                    let viewport_cloned2 = viewport_cloned2.clone();
                    let gpu_cloned3 = gpu_cloned2.clone();
                    let viewport_cloned3 = viewport_cloned2.clone();
                    let state_cloned8 = state_cloned8.clone();
                    let gpu_cloned4 = gpu_cloned4.clone();
                    let editor_cloned8 = editor_cloned8.clone();
                    let viewport_cloned4 = viewport_cloned4.clone();
                    let state_cloned9 = state_cloned9.clone();
                    let gpu_cloned5 = gpu_cloned5.clone();
                    let editor_cloned9 = editor_cloned9.clone();
                    let viewport_cloned5 = viewport_cloned5.clone();

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
                                    let gpu_cloned2 = gpu_cloned2.clone();
                                    let editor_cloned7 = editor_cloned7.clone();
                                    let viewport_cloned2 = viewport_cloned2.clone();

                                    container(
                                        (v_stack((
                                            label(|| "Polygon Properties"),
                                            properties_view(
                                                state_cloned5,
                                                gpu_cloned2,
                                                editor_cloned7,
                                                viewport_cloned2,
                                                polygon_selected,
                                                selected_polygon_id,
                                                selected_polygon_data,
                                                selected_sequence_id,
                                            ),
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
                            move || text_selected.get() && selected_keyframes.get().len() == 0,
                            move |text_selected_real| {
                                if text_selected_real {
                                    let state_cloned8 = state_cloned8.clone();
                                    let gpu_cloned4 = gpu_cloned4.clone();
                                    let editor_cloned8 = editor_cloned8.clone();
                                    let viewport_cloned4 = viewport_cloned4.clone();

                                    container(
                                        (v_stack((
                                            label(|| "Text Properties"),
                                            text_properties_view(
                                                state_cloned8,
                                                gpu_cloned4,
                                                editor_cloned8,
                                                viewport_cloned4,
                                                text_selected,
                                                selected_text_id,
                                                selected_text_data,
                                                selected_sequence_id,
                                            ),
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
                            move || image_selected.get() && selected_keyframes.get().len() == 0,
                            move |image_selected_real| {
                                if image_selected_real {
                                    let state_cloned9 = state_cloned9.clone();
                                    let gpu_cloned5 = gpu_cloned5.clone();
                                    let editor_cloned9 = editor_cloned9.clone();
                                    let viewport_cloned5 = viewport_cloned5.clone();

                                    container(
                                        (v_stack((
                                            label(|| "Image Properties"),
                                            image_properties_view(
                                                state_cloned9,
                                                gpu_cloned5,
                                                editor_cloned9,
                                                viewport_cloned5,
                                                image_selected,
                                                selected_image_id,
                                                selected_image_data,
                                                selected_sequence_id,
                                            ),
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
                                    let gpu_cloned3 = gpu_cloned3.clone();
                                    let viewport_cloned3 = viewport_cloned3.clone();

                                    keyframe_properties_view(
                                        state_cloned3,
                                        gpu_cloned3,
                                        editor_cloned5,
                                        viewport_cloned3,
                                        polygon_selected,
                                        selected_polygon_id,
                                        selected_polygon_data,
                                        selected_sequence_id,
                                        selected_keyframe,
                                        selected_keyframes,
                                        animation_data,
                                        sequence_selected,
                                        selected_sequence_data,
                                    )
                                    .into_any()
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

                                    editor.current_sequence_data =
                                        Some(selected_sequence_data.get());
                                    editor.is_playing = true;
                                }

                                // EventPropagation::Continue
                            }),
                            keyframe_timeline,
                        ))
                        .style(|s| s.margin_top(425.0)),
                    ))
                    .into_any()
                } else {
                    empty().into_any()
                }
            },
        ),
    ))
}

pub fn welcome_tab_interface(
    editor_state: Arc<Mutex<EditorState>>,
    editor: std::sync::Arc<Mutex<Editor>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
) -> impl View {
    let editor_state_cloned = editor_state.clone();
    let editor_cloned = editor.clone();
    let gpu_helper_cloned = gpu_helper.clone();
    let viewport_cloned = viewport.clone();
    let state_2 = Arc::clone(&editor_state);

    let tabs: im::Vector<&str> = vec!["Projects", "Settings"].into_iter().collect();
    let (tabs, _set_tabs) = create_signal(tabs);
    let (active_tab, set_active_tab) = create_signal(0);

    let list = scroll({
        virtual_stack(
            VirtualDirection::Vertical,
            VirtualItemSize::Fixed(Box::new(|| 90.0)),
            move || tabs.get(),
            move |item| *item,
            move |item| {
                let index = tabs
                    .get_untracked()
                    .iter()
                    .position(|it| *it == item)
                    .unwrap();
                let active = index == active_tab.get();
                let icon_name = match item {
                    "Projects" => "folder-plus",
                    "Settings" => "gear",
                    _ => "plus",
                };
                let destination_view = match item {
                    "Projects" => "manage_projects",
                    "Settings" => "editor_settings",
                    _ => "plus",
                };
                stack((
                    // label(move || item).style(|s| s.font_size(18.0)),
                    // svg(create_icon("plus")).style(|s| s.width(24).height(24)),
                    nav_button(
                        item,
                        icon_name,
                        Some({
                            let editor = editor.clone();

                            move || {
                                println!("Click...");
                                set_active_tab.update(|v: &mut usize| {
                                    *v = tabs
                                        .get_untracked()
                                        .iter()
                                        .position(|it| *it == item)
                                        .unwrap();
                                });

                                let mut editor = editor.lock().unwrap();

                                // no need to set current_view_signal, alhtough it could live in app_view if needed

                                // let mut renderer_state = editor_state
                                //     .renderer_state
                                //     .as_mut()
                                //     .expect("Couldn't get RendererState")
                                //     .lock()
                                //     .unwrap();
                                editor.current_view = destination_view.to_string();

                                // EventPropagation::Continue
                            }
                        }),
                        active,
                    ),
                ))
                // .on_click()
                .on_event(EventListener::KeyDown, move |e| {
                    if let Event::KeyDown(key_event) = e {
                        let active = active_tab.get();
                        if key_event.modifiers.is_empty() {
                            match key_event.key.logical_key {
                                Key::Named(NamedKey::ArrowUp) => {
                                    if active > 0 {
                                        set_active_tab.update(|v| *v -= 1)
                                    }
                                    EventPropagation::Stop
                                }
                                Key::Named(NamedKey::ArrowDown) => {
                                    if active < tabs.get().len() - 1 {
                                        set_active_tab.update(|v| *v += 1)
                                    }
                                    EventPropagation::Stop
                                }
                                _ => EventPropagation::Continue,
                            }
                        } else {
                            EventPropagation::Continue
                        }
                    } else {
                        EventPropagation::Continue
                    }
                })
                .keyboard_navigatable()
                .style(move |s| {
                    s.margin_bottom(15.0)
                        .border_radius(15)
                        .apply_if(index == active_tab.get(), |s| {
                            s.border(1.0).border_color(Color::GRAY)
                        })
                })
            },
        )
        .style(|s| {
            s.flex_col()
                .height_full()
                .width(110.0)
                .padding_vert(15.0)
                .padding_horiz(20.0)
        })
    })
    .scroll_style(|s| s.shrink_to_fit());

    container((
        list, // tab list
        tab(
            // active tab
            move || active_tab.get(),
            move || tabs.get(),
            |it| *it,
            move |it| match it {
                "Projects" => project_browser(
                    editor_state_cloned.clone(),
                    editor_cloned.clone(),
                    gpu_helper_cloned.clone(),
                    viewport_cloned.clone(),
                )
                .into_any(),
                "Settings" => editor_settings(gpu_helper.clone(), viewport.clone()).into_any(),
                _ => label(|| "Not implemented".to_owned()).into_any(),
            },
        )
        .style(|s| s.flex_col().items_start().margin_top(20.0)),
    ))
    .style(|s| s.flex_col().width_full().height_full())
}

pub fn selection_view(
    editor_state: Arc<Mutex<EditorState>>,
    editor: std::sync::Arc<Mutex<Editor>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
) -> impl IntoView {
    container((welcome_tab_interface(
        editor_state.clone(),
        editor.clone(),
        gpu_helper.clone(),
        viewport.clone(),
    ),))
}

pub fn app_view(
    editor_state: Arc<Mutex<EditorState>>,
    editor: std::sync::Arc<Mutex<Editor>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
) -> impl IntoView {
    let project_selected = create_rw_signal(Uuid::nil());

    let editor_state_cloned = Arc::clone(&editor_state);

    create_effect(move |_| {
        let mut editor_state = editor_state_cloned.lock().unwrap();
        editor_state.project_selected_signal = Some(project_selected);
    });

    dyn_container(
        move || project_selected.get(),
        move |project_selected_real| {
            if project_selected_real != Uuid::nil() {
                project_view(
                    editor_state.clone(),
                    editor.clone(),
                    gpu_helper.clone(),
                    viewport.clone(),
                )
                .into_any()
            } else {
                selection_view(
                    editor_state.clone(),
                    editor.clone(),
                    gpu_helper.clone(),
                    viewport.clone(),
                )
                .into_any()
            }
        },
    )
}
