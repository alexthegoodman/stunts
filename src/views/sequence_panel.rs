use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crossbeam::queue;
use floem::action::debounce_action;
use floem::common::{
    card_styles, create_icon, icon_button, option_button, simple_button, small_button,
    toggle_button,
};
use floem::peniko::Color;
use floem::reactive::SignalRead;
use floem::reactive::{create_effect, create_rw_signal, SignalUpdate};
use floem::reactive::{RwSignal, SignalGet};
use floem::style::CursorStyle;
use floem::taffy::{AlignItems, FlexDirection, FlexWrap};
use floem::views::{
    dyn_container, dyn_stack, empty, h_stack, list, scroll, stack, svg, v_stack, Checkbox,
    Decorators,
};
use floem::GpuHelper;
use floem::{views::label, IntoView};
use floem_renderer::gpu_resources;
use palette::rgb::Rgb;
use palette::Srgb;
use rand::Rng;
use stunts_engine::capture::{
    get_sources, MousePosition, RectInfo, SourceData, StCapture, WindowInfo,
};
use stunts_engine::editor::{
    rgb_to_wgpu, string_to_f32, string_to_u32, wgpu_to_human, ControlMode, Editor, InputValue,
    Point, Viewport, WindowSize,
};
use stunts_engine::polygon::{
    Polygon, PolygonConfig, SavedPoint, SavedPolygonConfig, SavedStroke, Stroke,
};
use stunts_engine::st_image::{SavedStImageConfig, StImage, StImageConfig};
use stunts_engine::st_video::{SavedStVideoConfig, StVideoConfig};
use stunts_engine::text;
use stunts_engine::text_due::{SavedTextRendererConfig, TextRenderer, TextRendererConfig};
use uuid::Uuid;

use crate::editor_state::{self, EditorState};
use crate::helpers::saved_state;
use crate::helpers::utilities::{
    get_captures_dir, get_ground_truth_dir, get_images_dir, get_videos_dir, save_saved_state_raw,
};
use stunts_engine::animations::{
    AnimationData, AnimationProperty, BackgroundFill, EasingType, KeyframeValue, ObjectType,
    Sequence, UIKeyframe,
};

use super::inputs::{debounce_input, inline_dropdown, DropdownOption};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LayerKind {
    Polygon,
    // Path,
    Image,
    Text,
    // Group,
    Video,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Layer {
    pub instance_id: Uuid,
    pub instance_name: String,
    pub instance_kind: LayerKind,
    pub initial_layer_index: i32,
}

impl Layer {
    pub fn from_polygon_config(config: &PolygonConfig) -> Self {
        Layer {
            instance_id: config.id,
            instance_name: config.name.clone(),
            instance_kind: LayerKind::Polygon,
            initial_layer_index: config.layer,
        }
    }
    pub fn from_image_config(config: &StImageConfig) -> Self {
        Layer {
            instance_id: Uuid::from_str(&config.id).expect("Couldn't convert uuid to string"),
            instance_name: config.name.clone(),
            instance_kind: LayerKind::Image,
            initial_layer_index: config.layer,
        }
    }
    pub fn from_text_config(config: &TextRendererConfig) -> Self {
        Layer {
            instance_id: config.id,
            instance_name: config.name.clone(),
            instance_kind: LayerKind::Text,
            initial_layer_index: config.layer,
        }
    }
    pub fn from_video_config(config: &StVideoConfig) -> Self {
        Layer {
            instance_id: Uuid::from_str(&config.id).expect("Couldn't convert uuid to string"),
            instance_name: config.name.clone(),
            instance_kind: LayerKind::Video,
            initial_layer_index: config.layer,
        }
    }
}

pub fn sortable_item<F, FB, FC>(
    editor: std::sync::Arc<Mutex<Editor>>,
    sortable_items: RwSignal<Vec<Layer>>,
    dragger_id: RwSignal<Uuid>,
    item_id: Uuid,
    kind: LayerKind,
    layer_name: String,
    icon_name: &'static str,
    on_items_updated: F,
    on_item_duplicated: FB,
    on_item_deleted: FC,
) -> impl IntoView
where
    F: Fn() + Clone + 'static,
    FB: Fn(Uuid, LayerKind) + Clone + 'static,
    FC: Fn(Uuid, LayerKind) + Clone + 'static,
{
    h_stack((
        h_stack((
            svg(create_icon(icon_name))
                .style(|s| s.width(24).height(24).color(Color::BLACK))
                .style(|s| s.margin_right(7.0).selectable(false)),
            // .on_event_stop(
            //     floem::event::EventListener::PointerDown,
            //     |_| { /* Disable dragging for this view */ },
            // ),
            label(move || layer_name.to_string())
                .style(|s| s.selectable(false).cursor(CursorStyle::RowResize)),
        ))
        .style(|s| s.align_items(AlignItems::Center)),
        h_stack((
            icon_button("copy", "Duplicate".to_string(), move |_| {
                on_item_duplicated(item_id, kind);
            }),
            icon_button("trash", "Delete".to_string(), move |_| {
                on_item_deleted(item_id, kind);
            }),
        )),
    ))
    .style(|s| s.justify_between().cursor(CursorStyle::RowResize))
    .draggable()
    .on_event(floem::event::EventListener::DragStart, move |_| {
        dragger_id.set(item_id);
        floem::event::EventPropagation::Continue
    })
    .on_event(floem::event::EventListener::DragOver, move |_| {
        // let mut editor = editor.lock().unwrap();
        let dragger_id = dragger_id.get_untracked();
        if dragger_id != item_id {
            let dragger_pos = sortable_items
                .get()
                .iter()
                .position(|layer| layer.instance_id == dragger_id)
                .or_else(|| Some(usize::MAX))
                .expect("Couldn't get dragger_pos");
            let hover_pos = sortable_items
                .get()
                .iter()
                .position(|layer| layer.instance_id == item_id)
                .or_else(|| Some(usize::MAX))
                .expect("Couldn't get hover_pos");

            sortable_items.update(|items| {
                if (dragger_pos <= items.len() && hover_pos <= items.len()) {
                    let item = items.get(dragger_pos).cloned();
                    items.remove(dragger_pos);
                    // editor.layer_list.remove(dragger_pos);

                    if let Some(selected_item) = item {
                        items.insert(hover_pos, selected_item.clone());
                        // editor
                        //     .layer_list
                        //     .insert(hover_pos, selected_item.instance_id);
                    }
                }
            });
        }
        floem::event::EventPropagation::Continue
    })
    .on_event(floem::event::EventListener::DragEnd, move |_| {
        on_items_updated();

        floem::event::EventPropagation::Continue
    })
    .dragging_style(|s| {
        s.box_shadow_blur(3)
            .box_shadow_color(Color::rgba(100.0, 100.0, 100.0, 0.5))
            .box_shadow_spread(2)
    })
    .style(|s| {
        s.width(260.0)
            .border_radius(15.0)
            .align_items(AlignItems::Center)
            .padding_vert(8)
            .background(Color::rgb(255.0, 239.0, 194.0))
            .border_bottom(1)
            .border_color(Color::rgb(200.0, 200.0, 200.0))
            .hover(|s| s.background(Color::rgb(222.0, 206.0, 160.0)))
            .active(|s| s.background(Color::rgb(237.0, 218.0, 164.0)))
    })
    // .on_click(|_| {
    //     println!("Layer selected");
    //     EventPropagation::Stop
    // })
}

pub fn import_video_to_scene(
    editor_cloned: std::sync::Arc<Mutex<Editor>>,
    editor_state_cloned: Arc<Mutex<EditorState>>,
    gpu_helper_cloned: Arc<Mutex<GpuHelper>>,
    viewport_cloned: Arc<Mutex<Viewport>>,
    selected_sequence_id: RwSignal<String>,
    selected_sequence_data: RwSignal<Sequence>,
    output_path: PathBuf,
    mouse_positions_path: Option<PathBuf>,
    source_data_path: Option<PathBuf>,
) {
    let mut saved_mouse_path = None;
    let mut stored_mouse_positions = None;
    if let Some(mouse_path) = &mouse_positions_path {
        if let Ok(positions) = fs::read_to_string(mouse_path) {
            if let Ok(mouse_positions) = serde_json::from_str::<Vec<MousePosition>>(&positions) {
                let the_path = mouse_path.to_str().expect("Couldn't make string from path");
                saved_mouse_path = Some(the_path.to_string());
                stored_mouse_positions = Some(mouse_positions);
            }
        }
    }

    let mut stored_source_data = None;
    if let Some(source_path) = &source_data_path {
        if let Ok(source_data) = fs::read_to_string(source_path) {
            if let Ok(data) = serde_json::from_str::<SourceData>(&source_data) {
                stored_source_data = Some(data);
            }
        }
    }

    let mut editor = editor_cloned.lock().unwrap();

    let mut rng = rand::thread_rng();
    let random_number_800 = rng.gen_range(0..=800);
    let random_number_450 = rng.gen_range(0..=450);

    let new_id = Uuid::new_v4();

    let position = Point {
        x: random_number_800 as f32 + 600.0,
        y: random_number_450 as f32 + 50.0,
    };

    let video_config = StVideoConfig {
        id: new_id.clone().to_string(),
        name: "New Video Item".to_string(),
        dimensions: (400, 225), // 16:9
        position,
        path: output_path
            .to_str()
            .expect("Couldn't get path string")
            .to_string(),
        layer: -1,
        mouse_path: saved_mouse_path.clone(),
    };

    let gpu_helper = gpu_helper_cloned.lock().unwrap();
    let gpu_resources = gpu_helper
        .gpu_resources
        .as_ref()
        .expect("Couldn't get gpu resources");
    let device = &gpu_resources.device;
    let queue = &gpu_resources.queue;
    let viewport = viewport_cloned.lock().unwrap();
    let window_size = WindowSize {
        width: viewport.width as u32,
        height: viewport.height as u32,
    };

    editor.add_video_item(
        &window_size,
        &device,
        &queue,
        video_config.clone(),
        &output_path,
        new_id,
        selected_sequence_id.get(),
        stored_mouse_positions,
        stored_source_data,
    );

    let source_duration_ms = editor
        .video_items
        .last()
        .expect("Couldn't get latest video")
        .source_duration_ms
        .clone();

    drop(viewport);
    drop(gpu_helper);
    drop(editor);

    let mut editor_state = editor_state_cloned.lock().unwrap();
    editor_state.add_saved_video_item(
        selected_sequence_id.get(),
        SavedStVideoConfig {
            id: video_config.id.to_string().clone(),
            name: video_config.name.clone(),
            path: output_path
                .to_str()
                .expect("Couldn't get path as string")
                .to_string(),
            dimensions: (video_config.dimensions.0, video_config.dimensions.1),
            position: SavedPoint {
                x: position.x as i32 - 600,
                y: position.y as i32 - 50,
            },
            layer: video_config.layer.clone(),
            mouse_path: saved_mouse_path.clone(),
        },
        source_duration_ms,
    );

    let saved_state = editor_state
        .record_state
        .saved_state
        .as_ref()
        .expect("Couldn't get saved state");
    let mut updated_sequence = saved_state
        .sequences
        .iter()
        .find(|s| s.id == selected_sequence_id.get())
        .expect("Couldn't get updated sequence")
        .clone();

    selected_sequence_data.set(updated_sequence.clone());

    let sequence_cloned = updated_sequence.clone();

    drop(editor_state);

    let mut editor = editor_cloned.lock().unwrap();

    editor.current_sequence_data = Some(sequence_cloned.clone());
    editor.update_motion_paths(&sequence_cloned);

    drop(editor);
}

pub fn sequence_panel(
    editor_state: Arc<Mutex<EditorState>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    editor: std::sync::Arc<Mutex<Editor>>,
    viewport: Arc<Mutex<Viewport>>,
    sequence_selected: RwSignal<bool>,
    selected_sequence_id: RwSignal<String>,
    selected_sequence_data: RwSignal<Sequence>,
    polygon_selected: RwSignal<bool>,
    selected_polygon_id: RwSignal<Uuid>,
) -> impl IntoView {
    let state_cloned = Arc::clone(&editor_state);
    let state_cloned_2 = Arc::clone(&editor_state);
    let state_cloned_3 = Arc::clone(&editor_state);
    let state_cloned_4 = Arc::clone(&editor_state);
    let state_cloned_5 = Arc::clone(&editor_state);
    let state_cloned_6 = Arc::clone(&editor_state);
    let state_cloned_7 = Arc::clone(&editor_state);
    let state_cloned_8 = Arc::clone(&editor_state);
    let state_cloned_9 = Arc::clone(&editor_state);
    let state_cloned_10 = Arc::clone(&editor_state);
    let state_cloned_11 = Arc::clone(&editor_state);
    let state_cloned_12 = Arc::clone(&editor_state);
    let state_cloned_13 = Arc::clone(&editor_state);
    let state_cloned_14 = Arc::clone(&editor_state);
    let state_cloned_15 = Arc::clone(&editor_state);
    let state_cloned_16 = Arc::clone(&editor_state);
    let state_cloned_17 = Arc::clone(&editor_state);
    let state_cloned_18 = Arc::clone(&editor_state);
    let state_cloned_19 = Arc::clone(&editor_state);
    let state_cloned_20 = Arc::clone(&editor_state);
    let state_cloned_21 = Arc::clone(&editor_state);
    let editor_cloned = Arc::clone(&editor);
    let editor_cloned_2 = Arc::clone(&editor);
    let editor_cloned_3 = Arc::clone(&editor);
    let editor_cloned_4 = Arc::clone(&editor);
    let editor_cloned_5 = Arc::clone(&editor);
    let editor_cloned_6 = Arc::clone(&editor);
    let editor_cloned_7 = Arc::clone(&editor);
    let editor_cloned_8 = Arc::clone(&editor);
    let editor_cloned_9 = Arc::clone(&editor);
    let editor_cloned_10 = Arc::clone(&editor);
    let editor_cloned_11 = Arc::clone(&editor);
    let editor_cloned_12 = Arc::clone(&editor);
    let editor_cloned_13 = Arc::clone(&editor);
    let editor_cloned_14 = Arc::clone(&editor);
    let editor_cloned_15 = Arc::clone(&editor);
    let gpu_cloned = Arc::clone(&gpu_helper);
    let viewport_cloned = Arc::clone(&viewport);
    let gpu_cloned_2 = Arc::clone(&gpu_helper);
    let viewport_cloned_2 = Arc::clone(&viewport);
    let gpu_cloned_3 = Arc::clone(&gpu_helper);
    let gpu_cloned_4 = Arc::clone(&gpu_helper);
    let gpu_cloned_5 = Arc::clone(&gpu_helper);
    let gpu_cloned_6 = Arc::clone(&gpu_helper);
    let viewport_cloned_3 = Arc::clone(&viewport);
    let viewport_cloned_4 = Arc::clone(&viewport);
    let viewport_cloned_5 = Arc::clone(&viewport);
    let viewport_cloned_6 = Arc::clone(&viewport);
    let viewport_cloned_7 = Arc::clone(&viewport);
    let viewport_cloned_8 = Arc::clone(&viewport);

    let selected_file = create_rw_signal(None::<PathBuf>);
    let local_mode = create_rw_signal("layout".to_string());

    let layers: RwSignal<Vec<Layer>> = create_rw_signal(Vec::new());
    let layers_ref = Arc::new(Mutex::new(layers));
    let window_height = create_rw_signal(0.0);
    let dragger_id = create_rw_signal(Uuid::nil());

    let select_active = create_rw_signal(true);
    let pan_active = create_rw_signal(false);

    let st_capture = create_rw_signal(StCapture::new(get_captures_dir()));
    let capture_selected = create_rw_signal(false);
    let capture_sources = create_rw_signal(Vec::new());
    let selected_source = create_rw_signal(WindowInfo {
        hwnd: 0,
        title: String::new(),
        rect: RectInfo {
            left: 0,
            right: 0,
            top: 0,
            bottom: 0,
            width: 100,
            height: 100,
        },
    });
    let source_selected = create_rw_signal(false);
    let is_recording = create_rw_signal(false);

    let capture_paths = RwSignal::new((String::new(), String::new(), String::new()));
    debounce_action(capture_paths, Duration::from_millis(1000), {
        let editor_cloned_13 = editor_cloned_13.clone();
        let state_cloned_11 = state_cloned_11.clone();
        let gpu_cloned_5 = gpu_cloned_5.clone();
        let viewport_cloned_7 = viewport_cloned_7.clone();

        move || {
            println!("Importing video to scene...");
            // r.set(local_r.get_untracked());
            // Now, import the video!
            let (capture_path, mouse_positions_path, source_data_path) = capture_paths.get();

            import_video_to_scene(
                editor_cloned_13.clone(),
                state_cloned_11.clone(),
                gpu_cloned_5.clone(),
                viewport_cloned_7.clone(),
                selected_sequence_id,
                selected_sequence_data,
                Path::new(&capture_path).to_path_buf(),
                Some(Path::new(&mouse_positions_path).to_path_buf()),
                Some(Path::new(&source_data_path).to_path_buf()),
            );
        }
    });

    let sequence_duration_input = create_rw_signal(String::new());
    let target_duration_signal = create_rw_signal(String::new());

    let count_dropdown_options: RwSignal<Vec<DropdownOption>> = create_rw_signal(Vec::new());
    let selected_count = create_rw_signal("4".to_string());

    let curve_is_checked = create_rw_signal(false);
    let choreographed_is_checked = create_rw_signal(true);
    let fade_is_checked = create_rw_signal(true);

    let background_polygon_data: RwSignal<PolygonConfig> = create_rw_signal(PolygonConfig {
        id: Uuid::nil(),
        name: String::new(),
        points: vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 1.0, y: 0.0 },
            Point { x: 1.0, y: 1.0 },
            Point { x: 0.0, y: 1.0 },
        ],
        dimensions: (800.0 as f32, 450.0 as f32),
        position: Point { x: 400.0, y: 225.0 },
        border_radius: 0.0,
        fill: [0.8, 0.8, 0.8, 1.0],
        stroke: Stroke {
            thickness: 0.0,
            fill: rgb_to_wgpu(0, 0, 0, 255.0),
        },
        layer: -89,
    });

    create_effect(move |_| {
        let mut current_background_data = background_polygon_data.get_untracked();
        let selected_sequence = selected_sequence_data.get_untracked();

        if let Some(background_fill) = selected_sequence.background_fill {
            current_background_data.id =
                Uuid::from_str(&selected_sequence.id).expect("Couldn't convert string to uuid");

            match background_fill {
                BackgroundFill::Color(fill) => {
                    current_background_data.fill =
                        rgb_to_wgpu(fill[0] as u8, fill[1] as u8, fill[2] as u8, fill[3] as f32);
                }
                BackgroundFill::Gradient() => {
                    println!("Gradient not supported yet...");
                }
            }

            background_polygon_data.set(current_background_data);
        }
    });

    create_effect(move |_| {
        let mut options: Vec<DropdownOption> = Vec::new();
        options.push(DropdownOption {
            id: "4".to_string(),
            label: "4".to_string(),
        });
        options.push(DropdownOption {
            id: "6".to_string(),
            label: "6".to_string(),
        });

        count_dropdown_options.set(options);
    });

    create_effect({
        let editor_cloned_6 = Arc::clone(&editor_cloned_6);

        move |_| {
            let is_curved = curve_is_checked.get();
            let is_choreographed = choreographed_is_checked.get();
            let is_fade = fade_is_checked.get();

            let mut editor = editor_cloned_6.lock().unwrap();

            editor.generation_curved = is_curved;
            editor.generation_choreographed = is_choreographed;
            editor.generation_fade = is_fade;

            drop(editor);
        }
    });

    create_effect({
        let editor_cloned_6 = Arc::clone(&editor_cloned_6);

        move |_| {
            println!("Panel lock...");
            let mut editor = editor_cloned_6.lock().unwrap();

            let mut new_layers = Vec::new();
            editor.polygons.iter().for_each(|polygon| {
                if !polygon.hidden {
                    let polygon_config: PolygonConfig = polygon.to_config();
                    let new_layer: Layer = Layer::from_polygon_config(&polygon_config);
                    new_layers.push(new_layer);
                }
            });
            editor.text_items.iter().for_each(|text| {
                if !text.hidden {
                    let text_config: TextRendererConfig = text.to_config();
                    let new_layer: Layer = Layer::from_text_config(&text_config);
                    new_layers.push(new_layer);
                }
            });
            editor.image_items.iter().for_each(|image| {
                if !image.hidden {
                    let image_config: StImageConfig = image.to_config();
                    let new_layer: Layer = Layer::from_image_config(&image_config);
                    new_layers.push(new_layer);
                }
            });
            editor.video_items.iter().for_each(|video| {
                if !video.hidden {
                    let video_config: StVideoConfig = video.to_config();
                    let new_layer: Layer = Layer::from_video_config(&video_config);
                    new_layers.push(new_layer);
                }
            });

            // sort layers by layer_index property, lower values should come first in the list
            // but reverse the order because the UI outputs the first one first, thus it displays last
            new_layers.sort_by(|a, b| b.initial_layer_index.cmp(&a.initial_layer_index));

            layers.set(new_layers);

            drop(editor);

            // let editor_state = state_cloned_7.lock().unwrap();
            // let saved_state = editor_state
            //     .record_state
            //     .saved_state
            //     .as_ref()
            //     .expect("Couldn't get saved state");
            // let timeline_sequence = saved_state
            //     .timeline_state
            //     .timeline_sequences
            //     .iter()
            //     .find(|ts| ts.sequence_id == selected_sequence_id.get_untracked());

            // if let Some(sequence) = timeline_sequence {
            //     let initial_duration = sequence.duration_ms / 1000;
            //     sequence_duration_input.set(initial_duration.to_string());
            // }

            // drop(editor_state);
        }
    });

    create_effect({
        let viewport_cloned_4 = Arc::clone(&viewport_cloned_4);
        move |_| {
            println!("Viewport lock...");
            let viewport = viewport_cloned_4.lock().unwrap();

            window_height.set(viewport.height);
        }
    });

    // this function can be reused for resetting layers to correctness and save it out
    let on_items_updated = move || {
        // update layers for objects in sequence in saved state and editor
        let updated_layers = layers.get();

        let mut editor = editor_cloned_8.lock().unwrap();

        updated_layers
            .iter()
            .enumerate()
            .for_each(|(index, l)| match l.instance_kind {
                LayerKind::Polygon => {
                    editor.polygons.iter_mut().for_each(|p| {
                        if p.id == l.instance_id {
                            p.update_layer(-(index as i32));
                        }
                    });
                }
                LayerKind::Text => {
                    editor.text_items.iter_mut().for_each(|t| {
                        if t.id == l.instance_id {
                            t.update_layer(-(index as i32));
                        }
                    });
                }
                LayerKind::Image => {
                    editor.image_items.iter_mut().for_each(|i| {
                        if i.id == l.instance_id.to_string() {
                            i.update_layer(-(index as i32));
                        }
                    });
                }
                LayerKind::Video => {
                    editor.video_items.iter_mut().for_each(|v| {
                        if v.id == l.instance_id.to_string() {
                            v.update_layer(-(index as i32));
                        }
                    });
                }
            });

        drop(editor);

        let mut editor_state = state_cloned_5.lock().unwrap();

        let mut saved_state = editor_state
            .record_state
            .saved_state
            .as_mut()
            .expect("Couldn't get Saved State");

        saved_state.sequences.iter_mut().for_each(|s| {
            if s.id == selected_sequence_id.get() {
                updated_layers
                    .iter()
                    .enumerate()
                    .for_each(|(index, l)| match l.instance_kind {
                        LayerKind::Polygon => {
                            s.active_polygons.iter_mut().for_each(|p| {
                                if p.id == l.instance_id.to_string() {
                                    p.layer = -(index as i32);
                                }
                            });
                        }
                        LayerKind::Text => {
                            s.active_text_items.iter_mut().for_each(|t| {
                                if t.id == l.instance_id.to_string() {
                                    t.layer = -(index as i32);
                                }
                            });
                        }
                        LayerKind::Image => {
                            s.active_image_items.iter_mut().for_each(|i| {
                                if i.id == l.instance_id.to_string() {
                                    i.layer = -(index as i32);
                                }
                            });
                        }
                        LayerKind::Video => {
                            s.active_video_items.iter_mut().for_each(|v| {
                                if v.id == l.instance_id.to_string() {
                                    v.layer = -(index as i32);
                                }
                            });
                        }
                    });
            }
        });

        save_saved_state_raw(saved_state.clone());

        editor_state.record_state.saved_state = Some(saved_state.clone());

        drop(editor_state);
    };

    let on_item_duplicated = {
        let on_items_updated = on_items_updated.clone();

        move |object_id, kind| {
            let mut editor = editor_cloned_11.lock().unwrap();
            let viewport = viewport_cloned_6.lock().unwrap();
            let camera = editor.camera.expect("Couldn't get camera");
            let gpu_helper = gpu_cloned_4.lock().unwrap();
            let gpu_resources = gpu_helper
                .gpu_resources
                .as_ref()
                .expect("Couldn't get gpu resources");

            let window_size = WindowSize {
                width: viewport.width as u32,
                height: viewport.height as u32,
            };

            let new_id = Uuid::new_v4();
            let new_offset = 50.0;
            let mut new_polygon_config = None;
            let mut new_image_config = None;
            let mut new_text_config = None;

            // duplicate relevant object and layer
            match kind {
                LayerKind::Polygon => {
                    let mut existing_polygon = editor
                        .polygons
                        .iter()
                        .find(|p| p.id == object_id)
                        .expect("Couldn't find matching polygon");

                    let mut polygon_config: PolygonConfig = existing_polygon.to_config();

                    polygon_config.id = new_id;
                    polygon_config.position = Point {
                        x: polygon_config.position.x + new_offset,
                        y: polygon_config.position.y + new_offset,
                    };

                    let mut duplicated_polygon = Polygon::from_config(
                        &polygon_config,
                        &window_size,
                        &gpu_resources.device,
                        &gpu_resources.queue,
                        &editor
                            .model_bind_group_layout
                            .as_ref()
                            .expect("Couldn't get model bind group layout"),
                        &editor
                            .group_bind_group_layout
                            .as_ref()
                            .expect("Couldn't get group bind group layout"),
                        &camera,
                        selected_sequence_id.get(),
                    );

                    duplicated_polygon.hidden = false;

                    editor.polygons.push(duplicated_polygon);

                    let duplicated_layer: Layer = Layer::from_polygon_config(&polygon_config);

                    layers.update(|l| {
                        l.push(duplicated_layer);
                    });

                    new_polygon_config = Some(polygon_config);
                }
                LayerKind::Text => {
                    let mut existing_text = editor
                        .text_items
                        .iter()
                        .find(|p| p.id == object_id)
                        .expect("Couldn't find matching text");

                    let mut text_config: TextRendererConfig = existing_text.to_config();

                    text_config.id = new_id;
                    text_config.position = Point {
                        x: text_config.position.x + new_offset,
                        y: text_config.position.y + new_offset,
                    };

                    let font_data = editor
                        .font_manager
                        .get_font_by_name(&text_config.font_family)
                        .expect("Couldn't get font family");

                    let mut duplicated_text = TextRenderer::from_config(
                        &text_config,
                        &window_size,
                        &gpu_resources.device,
                        &gpu_resources.queue,
                        &editor
                            .model_bind_group_layout
                            .as_ref()
                            .expect("Couldn't get model bind group layout"),
                        &editor
                            .group_bind_group_layout
                            .as_ref()
                            .expect("Couldn't get group bind group layout"),
                        &camera,
                        selected_sequence_id.get(),
                        font_data,
                    );

                    duplicated_text.hidden = false;

                    editor.text_items.push(duplicated_text);

                    let duplicated_layer: Layer = Layer::from_text_config(&text_config);

                    layers.update(|l| {
                        l.push(duplicated_layer);
                    });

                    new_text_config = Some(text_config);
                }
                LayerKind::Image => {
                    let mut existing_image = editor
                        .image_items
                        .iter()
                        .find(|p| p.id == object_id.to_string())
                        .expect("Couldn't find matching image");

                    let mut image_config: StImageConfig = existing_image.to_config();

                    image_config.id = new_id.to_string();
                    image_config.position = Point {
                        x: image_config.position.x + new_offset,
                        y: image_config.position.y + new_offset,
                    };

                    let mut duplicated_image = StImage::from_config(
                        &image_config,
                        &window_size,
                        &gpu_resources.device,
                        &gpu_resources.queue,
                        &editor
                            .model_bind_group_layout
                            .as_ref()
                            .expect("Couldn't get model bind group layout"),
                        &editor
                            .group_bind_group_layout
                            .as_ref()
                            .expect("Couldn't get group bind group layout"),
                        &camera,
                        selected_sequence_id.get(),
                    );

                    duplicated_image.hidden = false;

                    editor.image_items.push(duplicated_image);

                    let duplicated_layer: Layer = Layer::from_image_config(&image_config);

                    layers.update(|l| {
                        l.push(duplicated_layer);
                    });

                    new_image_config = Some(image_config);
                }
                LayerKind::Video => {
                    println!("Duplicate not implemented for video");
                }
            };

            drop(viewport);
            drop(gpu_helper);
            drop(editor);

            // update duplicated object motion path with offset akin to object itself
            let mut editor_state = state_cloned_8.lock().unwrap();
            let saved_state = editor_state
                .record_state
                .saved_state
                .as_mut()
                .expect("Couldn't get saved state");
            let mut sequence = selected_sequence_data.get();

            let animation = sequence
                .polygon_motion_paths
                .iter_mut()
                .find(|pm| pm.polygon_id == object_id.to_string())
                .expect("Couldn't get matching path");

            animation.id = Uuid::new_v4().to_string();
            animation.polygon_id = new_id.to_string();

            animation
                .properties
                .iter_mut()
                .filter(|p| p.name == "Position".to_string())
                .for_each(|p| {
                    p.keyframes.iter_mut().for_each(|k| match k.value {
                        KeyframeValue::Position(pos) => {
                            k.id = Uuid::new_v4().to_string();
                            k.value = KeyframeValue::Position([
                                pos[0] + new_offset as i32,
                                pos[1] + new_offset as i32,
                            ])
                        }
                        _ => {}
                    });
                });

            // duplicate relevant motion paths
            let saved_sequence = saved_state
                .sequences
                .iter_mut()
                .find(|s| s.id == selected_sequence_id.get())
                .expect("Couldn't find selected sequence");

            saved_sequence.polygon_motion_paths.push(animation.clone());

            match kind {
                LayerKind::Polygon => {
                    let polygon_config = new_polygon_config.expect("Couldn't get new config");

                    saved_sequence.active_polygons.push(SavedPolygonConfig {
                        id: polygon_config.id.to_string().clone(),
                        name: polygon_config.name.clone(),
                        dimensions: (
                            polygon_config.dimensions.0 as i32,
                            polygon_config.dimensions.1 as i32,
                        ),
                        fill: [
                            polygon_config.fill[0] as i32,
                            polygon_config.fill[1] as i32,
                            polygon_config.fill[2] as i32,
                            polygon_config.fill[3] as i32,
                        ],
                        border_radius: polygon_config.border_radius as i32, // multiply by 100?
                        position: SavedPoint {
                            x: polygon_config.position.x as i32,
                            y: polygon_config.position.y as i32,
                        },
                        stroke: SavedStroke {
                            thickness: polygon_config.stroke.thickness as i32,
                            fill: [
                                polygon_config.stroke.fill[0] as i32,
                                polygon_config.stroke.fill[1] as i32,
                                polygon_config.stroke.fill[2] as i32,
                                polygon_config.stroke.fill[3] as i32,
                            ],
                        },
                        layer: polygon_config.layer.clone(),
                    });
                }
                LayerKind::Text => {
                    let text_config = new_text_config.expect("Couldn't get new config");

                    saved_sequence
                        .active_text_items
                        .push(SavedTextRendererConfig {
                            id: text_config.id.to_string().clone(),
                            name: text_config.name.clone(),
                            text: text_config.text.clone(),
                            font_family: text_config.font_family.clone(),
                            dimensions: (
                                text_config.dimensions.0 as i32,
                                text_config.dimensions.1 as i32,
                            ),
                            position: SavedPoint {
                                x: text_config.position.x as i32,
                                y: text_config.position.y as i32,
                            },
                            layer: text_config.layer.clone(),
                            color: text_config.color.clone(),
                            font_size: text_config.font_size.clone(),
                            background_fill: Some(text_config.background_fill),
                        });
                }
                LayerKind::Image => {
                    let image_config = new_image_config.expect("Couldn't get new config");

                    saved_sequence.active_image_items.push(SavedStImageConfig {
                        id: image_config.id.to_string().clone(),
                        name: image_config.name.clone(),
                        path: image_config.path,
                        dimensions: (image_config.dimensions.0, image_config.dimensions.1),
                        position: SavedPoint {
                            x: image_config.position.x as i32,
                            y: image_config.position.y as i32,
                        },
                        layer: image_config.layer.clone(),
                    });
                }
                LayerKind::Video => {
                    println!("Duplicate not implemented for video");
                }
            }

            // no need to update the unused sequence, use this one
            let sequence = saved_sequence.clone();

            drop(editor_state);

            // rerender motion paths
            let mut editor = editor_cloned_11.lock().unwrap();

            editor.update_motion_paths(&sequence);

            drop(editor);

            // also set selected_sequence_data
            selected_sequence_data.set(sequence);

            // update layer ordering and save saved state
            on_items_updated();
        }
    };

    // TODO: handle case where object is currently selected
    let on_item_deleted = {
        let on_items_updated = on_items_updated.clone();

        move |object_id, kind| {
            let mut editor = editor_cloned_12.lock().unwrap();

            // update editor / renderer (remove relevant object)
            match kind {
                LayerKind::Polygon => {
                    let index = editor
                        .polygons
                        .iter()
                        .position(|p| p.id == object_id)
                        .expect("Couldn't match object");

                    editor.polygons.swap_remove(index);
                }
                LayerKind::Text => {
                    let index = editor
                        .text_items
                        .iter()
                        .position(|p| p.id == object_id)
                        .expect("Couldn't match object");

                    editor.text_items.swap_remove(index);
                }
                LayerKind::Image => {
                    let index = editor
                        .image_items
                        .iter()
                        .position(|p| p.id == object_id.to_string())
                        .expect("Couldn't match object");

                    editor.image_items.swap_remove(index);
                }
                LayerKind::Video => {
                    let index = editor
                        .video_items
                        .iter()
                        .position(|p| p.id == object_id.to_string())
                        .expect("Couldn't match object");

                    editor.video_items.swap_remove(index);
                }
            }

            drop(editor);

            // update saved state (remove object and related animation path)
            let mut editor_state = state_cloned_10.lock().unwrap();
            let mut saved_state = editor_state
                .record_state
                .saved_state
                .as_mut()
                .expect("Couldn't get saved state");
            let mut sequence = selected_sequence_data.get();

            match kind {
                LayerKind::Polygon => {
                    let object_index = sequence
                        .active_polygons
                        .iter()
                        .position(|p| p.id == object_id.to_string())
                        .expect("Couldn't find object match");

                    let path_index = sequence
                        .polygon_motion_paths
                        .iter()
                        .position(|p| p.polygon_id == object_id.to_string())
                        .expect("Couldn't find object match");

                    sequence.active_polygons.remove(object_index);
                    sequence.polygon_motion_paths.remove(path_index);
                }
                LayerKind::Text => {
                    let object_index = sequence
                        .active_text_items
                        .iter()
                        .position(|p| p.id == object_id.to_string())
                        .expect("Couldn't find object match");

                    let path_index = sequence
                        .polygon_motion_paths
                        .iter()
                        .position(|p| p.polygon_id == object_id.to_string())
                        .expect("Couldn't find object match");

                    sequence.active_text_items.remove(object_index);
                    sequence.polygon_motion_paths.remove(path_index);
                }
                LayerKind::Image => {
                    let object_index = sequence
                        .active_image_items
                        .iter()
                        .position(|p| p.id == object_id.to_string())
                        .expect("Couldn't find object match");

                    let path_index = sequence
                        .polygon_motion_paths
                        .iter()
                        .position(|p| p.polygon_id == object_id.to_string())
                        .expect("Couldn't find object match");

                    sequence.active_image_items.remove(object_index);
                    sequence.polygon_motion_paths.remove(path_index);
                }
                LayerKind::Video => {
                    let object_index = sequence
                        .active_video_items
                        .iter()
                        .position(|p| p.id == object_id.to_string())
                        .expect("Couldn't find object match");

                    let path_index = sequence
                        .polygon_motion_paths
                        .iter()
                        .position(|p| p.polygon_id == object_id.to_string())
                        .expect("Couldn't find object match");

                    sequence.active_video_items.remove(object_index);
                    sequence.polygon_motion_paths.remove(path_index);
                }
            }

            saved_state.sequences.iter_mut().for_each(|s| {
                if s.id == selected_sequence_id.get() {
                    *s = sequence.clone();
                }
            });

            drop(editor_state);

            // rerender motion paths
            let mut editor = editor_cloned_12.lock().unwrap();

            editor.update_motion_paths(&sequence);

            drop(editor);

            // update selected_sequence_data
            selected_sequence_data.set(sequence);

            // update layers
            let mut current_layers = layers.get();
            let layer_index = current_layers
                .iter()
                .position(|l| l.instance_id == object_id)
                .expect("Couldn't find matching layer");
            current_layers.remove(layer_index);

            layers.set(current_layers);

            // update layer ordering and save saved state
            on_items_updated();
        }
    };

    let on_count_selection = move |count: String| {
        // TODO: wrap up in editor_state for undo/redo

        println!("on_count_selection {:?}", count);

        // update editor's text_item, recall render text
        let mut editor = editor.lock().unwrap();

        editor.generation_count = string_to_u32(&count).expect("Couldn't convert string to u32");

        drop(editor);
    };

    let aside_width = 260.0;
    let quarters = (aside_width / 4.0) + (5.0 * 4.0);
    let thirds = (aside_width / 3.0) + (5.0 * 3.0);
    let halfs = (aside_width / 2.0) + (5.0 * 2.0);

    let colors = [
        ["#FFE4E1", "#FF6B6B", "#FF0000", "#B22222", "#8B0000"], // red
        ["#FFECD9", "#FFB347", "#FF8C00", "#D95E00", "#993D00"], // orange
        ["#FFFACD", "#FFE66D", "#FFD700", "#DAA520", "#B8860B"], // yellow
        ["#E8F5E9", "#7CB342", "#2E7D32", "#1B5E20", "#0A3D0A"], // green
        ["#E3F2FD", "#64B5F6", "#1E88E5", "#1565C0", "#0D47A1"], // blue
        ["#F3E5F5", "#AB47BC", "#8E24AA", "#6A1B9A", "#4A148C"], // purple
        ["#FCE4EC", "#F06292", "#E91E63", "#C2185B", "#880E4F"], // pink
        ["#E0F2F1", "#4DB6AC", "#00897B", "#00695C", "#004D40"], // teal
        ["#EFEBE9", "#A1887F", "#795548", "#5D4037", "#3E2723"], // brown
        ["#F5F5F5", "#BDBDBD", "#757575", "#424242", "#212121"], // gray
    ];

    // 50 color / text combinations (style portion of format)
    // background_color_index, text_length, font_family_index, font_size, font_color_index
    let themes = [
        [0.0, 120.0, 12.0, 24.0, 0.4],
        [1.2, 80.0, 25.0, 32.0, 1.0],
        [2.1, 150.0, 37.0, 18.0, 2.3],
        [3.3, 200.0, 45.0, 20.0, 3.1],
        [4.4, 100.0, 50.0, 28.0, 4.0],
        [5.2, 90.0, 55.0, 22.0, 5.1],
        [6.0, 130.0, 10.0, 26.0, 6.3],
        [7.2, 110.0, 30.0, 16.0, 7.4],
        [8.1, 140.0, 40.0, 20.0, 8.3],
        [9.3, 180.0, 5.0, 18.0, 9.1],
        [0.1, 95.0, 18.0, 30.0, 0.3],
        [1.3, 110.0, 22.0, 20.0, 1.2],
        [2.2, 130.0, 35.0, 22.0, 2.4],
        [3.0, 160.0, 48.0, 18.0, 3.2],
        [4.1, 75.0, 7.0, 28.0, 4.3],
        [5.4, 140.0, 53.0, 24.0, 5.0],
        [6.2, 100.0, 14.0, 26.0, 6.1],
        [7.1, 120.0, 29.0, 20.0, 7.3],
        [8.2, 150.0, 42.0, 18.0, 8.4],
        [9.0, 200.0, 3.0, 16.0, 9.2],
        [0.3, 85.0, 20.0, 32.0, 0.2],
        [1.4, 105.0, 26.0, 24.0, 1.1],
        [2.0, 115.0, 38.0, 20.0, 2.3],
        [3.2, 170.0, 47.0, 18.0, 3.4],
        [4.2, 90.0, 9.0, 30.0, 4.1],
        [5.1, 125.0, 54.0, 22.0, 5.3],
        [6.3, 135.0, 16.0, 24.0, 6.2],
        [7.0, 145.0, 31.0, 18.0, 7.4],
        [8.3, 155.0, 43.0, 20.0, 8.1],
        [9.4, 180.0, 6.0, 16.0, 9.0],
        [0.4, 100.0, 23.0, 28.0, 0.1],
        [1.0, 115.0, 27.0, 22.0, 1.3],
        [2.3, 140.0, 39.0, 20.0, 2.2],
        [3.1, 160.0, 46.0, 18.0, 3.0],
        [4.3, 80.0, 8.0, 32.0, 4.2],
        [5.0, 130.0, 55.0, 24.0, 5.4],
        [6.1, 95.0, 15.0, 26.0, 6.4],
        [7.3, 110.0, 32.0, 20.0, 7.2],
        [8.4, 165.0, 44.0, 18.0, 8.0],
        [9.2, 190.0, 4.0, 16.0, 9.3],
    ];

    v_stack((
        v_stack((
            label(move || format!("Update Sequence"))
                .style(|s| s.font_size(14.0).margin_bottom(10)),
            scroll(
                (v_stack((
                    simple_button("Back to Sequence List".to_string(), move |_| {
                        // polygon_selected.set(false);
                        // selected_polygon_id.set(Uuid::nil());
                        sequence_selected.set(false);

                        let mut editor = editor_cloned_5.lock().unwrap();
                        let viewport = viewport_cloned_5.lock().unwrap();
                        let window_size = WindowSize {
                            width: viewport.width as u32,
                            height: viewport.height as u32,
                        };

                        editor.reset_bounds(&window_size);
                        editor.hide_all_objects();

                        drop(editor);
                    })
                    .style(|s| s.margin_bottom(5.0)),
                    h_stack((
                        inline_dropdown(
                            "Choose keyframe count".to_string(),
                            selected_count,
                            count_dropdown_options,
                            on_count_selection,
                        ),
                        Checkbox::new_labeled_rw(curve_is_checked, || "Curved Paths"),
                    ))
                    .style(|s| s.margin_bottom(5.0)),
                    h_stack((
                        Checkbox::new_labeled_rw(choreographed_is_checked, || "Auto-Choreograph"),
                        Checkbox::new_labeled_rw(fade_is_checked, || "Auto-Fade"),
                    ))
                    .style(|s| s.margin_bottom(5.0)),
                    v_stack((simple_button("Generate Animation".to_string(), move |_| {
                        // hook into CommonMotion2D run_motion_inference
                        let mut editor = editor_cloned_4.lock().unwrap();

                        let predicted_keyframes = editor.run_motion_inference();

                        let mut new_sequence = selected_sequence_data.get();
                        new_sequence.polygon_motion_paths = predicted_keyframes.clone();

                        selected_sequence_data.set(new_sequence);

                        editor.update_motion_paths(&selected_sequence_data.get());
                        println!("Motion Paths updated!");

                        drop(editor);

                        let mut editor_state = state_cloned_4.lock().unwrap();

                        let mut saved_state = editor_state
                            .record_state
                            .saved_state
                            .as_mut()
                            .expect("Couldn't get Saved State");

                        saved_state.sequences.iter_mut().for_each(|s| {
                            if s.id == selected_sequence_id.get() {
                                s.polygon_motion_paths = predicted_keyframes.clone();
                            }
                        });

                        save_saved_state_raw(saved_state.clone());

                        editor_state.record_state.saved_state = Some(saved_state.clone());

                        drop(editor_state);
                    })
                    .style(|s| s.background(Color::rgb8(255, 25, 25)).color(Color::WHITE)),))
                    .style(|s| s.margin_bottom(5.0)),
                    h_stack((
                        small_button(
                            "Select",
                            "motion-arrow", // TODO: "cursor"
                            move |_| {
                                pan_active.set(false);
                                select_active.set(true);

                                let mut editor = editor_cloned_9.lock().unwrap();

                                editor.control_mode = ControlMode::Select;
                            },
                            select_active,
                        ),
                        small_button(
                            "Pan",
                            "translate",
                            move |_| {
                                select_active.set(false);
                                pan_active.set(true);

                                let mut editor = editor_cloned_10.lock().unwrap();

                                editor.control_mode = ControlMode::Pan;
                            },
                            pan_active,
                        ),
                    ))
                    .style(|s| s.margin_bottom(5.0)),
                    stack((
                        option_button(
                            "Add Square",
                            "square",
                            Some(move || {
                                let mut editor = editor_cloned.lock().unwrap();
                                // let mut square_handler = square_handler.lock().unwrap();
                                println!("Handle square...");

                                // square_handler.handle_button_click(editor_cloned);

                                let mut rng = rand::thread_rng();

                                // Generate a random number between 0 and 800
                                let random_number_800 = rng.gen_range(0..=800);

                                // Generate a random number between 0 and 450
                                let random_number_450 = rng.gen_range(0..=450);

                                let new_id = Uuid::new_v4();

                                let polygon_config = PolygonConfig {
                                    id: new_id.clone(),
                                    name: "Square".to_string(),
                                    points: vec![
                                        Point { x: 0.0, y: 0.0 },
                                        Point { x: 1.0, y: 0.0 },
                                        Point { x: 1.0, y: 1.0 },
                                        Point { x: 0.0, y: 1.0 },
                                    ],
                                    dimensions: (100.0, 100.0),
                                    position: Point {
                                        x: random_number_800 as f32,
                                        y: random_number_450 as f32,
                                    },
                                    border_radius: 0.0,
                                    fill: [1.0, 1.0, 1.0, 1.0],
                                    stroke: Stroke {
                                        fill: [1.0, 1.0, 1.0, 1.0],
                                        thickness: 2.0,
                                    },
                                    layer: -2,
                                };
                                let gpu_helper = gpu_cloned.lock().unwrap();
                                let gpu_resources = gpu_helper
                                    .gpu_resources
                                    .as_ref()
                                    .expect("Couldn't get gpu resources");
                                let device = &gpu_resources.device;
                                let queue = &gpu_resources.queue;
                                let viewport = viewport_cloned.lock().unwrap();
                                let window_size = WindowSize {
                                    width: viewport.width as u32,
                                    height: viewport.height as u32,
                                };
                                let camera = editor.camera.expect("Couldn't get camera");

                                editor.add_polygon(
                                    &window_size,
                                    &device,
                                    &queue,
                                    &camera,
                                    polygon_config.clone(),
                                    "Polygon".to_string(),
                                    new_id,
                                    selected_sequence_id.get(),
                                );

                                drop(viewport);
                                drop(gpu_helper);
                                drop(editor);

                                let mut editor_state = state_cloned.lock().unwrap();
                                editor_state.add_saved_polygon(
                                    selected_sequence_id.get(),
                                    SavedPolygonConfig {
                                        id: polygon_config.id.to_string().clone(),
                                        name: polygon_config.name.clone(),
                                        dimensions: (
                                            polygon_config.dimensions.0 as i32,
                                            polygon_config.dimensions.1 as i32,
                                        ),
                                        fill: [
                                            polygon_config.fill[0] as i32,
                                            polygon_config.fill[1] as i32,
                                            polygon_config.fill[2] as i32,
                                            polygon_config.fill[3] as i32,
                                        ],
                                        border_radius: polygon_config.border_radius as i32, // multiply by 100?
                                        position: SavedPoint {
                                            x: polygon_config.position.x as i32,
                                            y: polygon_config.position.y as i32,
                                        },
                                        stroke: SavedStroke {
                                            thickness: polygon_config.stroke.thickness as i32,
                                            fill: [
                                                polygon_config.stroke.fill[0] as i32,
                                                polygon_config.stroke.fill[1] as i32,
                                                polygon_config.stroke.fill[2] as i32,
                                                polygon_config.stroke.fill[3] as i32,
                                            ],
                                        },
                                        layer: polygon_config.layer.clone(),
                                    },
                                );

                                let saved_state = editor_state
                                    .record_state
                                    .saved_state
                                    .as_ref()
                                    .expect("Couldn't get saved state");
                                let updated_sequence = saved_state
                                    .sequences
                                    .iter()
                                    .find(|s| s.id == selected_sequence_id.get())
                                    .expect("Couldn't get updated sequence");

                                selected_sequence_data.set(updated_sequence.clone());

                                let sequence_cloned = updated_sequence.clone();

                                drop(editor_state);

                                let mut editor = editor_cloned.lock().unwrap();

                                editor.current_sequence_data = Some(sequence_cloned.clone());
                                editor.update_motion_paths(&sequence_cloned);

                                drop(editor);
                            }),
                            false,
                        ),
                        option_button(
                            "Add Image",
                            "image",
                            Some(move || {
                                if let Some(original_path) = rfd::FileDialog::new()
                                    .add_filter("images", &["png", "jpg", "jpeg"])
                                    .pick_file()
                                {
                                    // Get the storage directory
                                    let storage_dir = get_images_dir();

                                    // Create a new file name to avoid conflicts
                                    let file_name =
                                        original_path.file_name().expect("Couldn't get file name");
                                    let new_path = storage_dir.join(file_name);

                                    // Copy the image to the storage directory
                                    fs::copy(&original_path, &new_path)
                                        .expect("Couldn't copy image to storage directory");

                                    // Add to scene
                                    let mut editor = editor_cloned_3.lock().unwrap();

                                    let mut rng = rand::thread_rng();
                                    let random_number_800 = rng.gen_range(0..=800);
                                    let random_number_450 = rng.gen_range(0..=450);

                                    let new_id = Uuid::new_v4();

                                    let position = Point {
                                        x: random_number_800 as f32 + 600.0,
                                        y: random_number_450 as f32 + 50.0,
                                    };

                                    let image_config = StImageConfig {
                                        id: new_id.clone().to_string(),
                                        name: "New Image Item".to_string(),
                                        dimensions: (100, 100),
                                        position,
                                        path: new_path
                                            .to_str()
                                            .expect("Couldn't get path string")
                                            .to_string(),
                                        layer: -1,
                                    };

                                    let gpu_helper = gpu_cloned_3.lock().unwrap();
                                    let gpu_resources = gpu_helper
                                        .gpu_resources
                                        .as_ref()
                                        .expect("Couldn't get gpu resources");
                                    let device = &gpu_resources.device;
                                    let queue = &gpu_resources.queue;
                                    let viewport = viewport_cloned_3.lock().unwrap();
                                    let window_size = WindowSize {
                                        width: viewport.width as u32,
                                        height: viewport.height as u32,
                                    };

                                    editor.add_image_item(
                                        &window_size,
                                        &device,
                                        &queue,
                                        image_config.clone(),
                                        &new_path,
                                        new_id,
                                        selected_sequence_id.get(),
                                    );

                                    drop(viewport);
                                    drop(gpu_helper);
                                    drop(editor);

                                    let mut editor_state = state_cloned_3.lock().unwrap();
                                    editor_state.add_saved_image_item(
                                        selected_sequence_id.get(),
                                        SavedStImageConfig {
                                            id: image_config.id.to_string().clone(),
                                            name: image_config.name.clone(),
                                            path: new_path
                                                .to_str()
                                                .expect("Couldn't get path as string")
                                                .to_string(),
                                            dimensions: (
                                                image_config.dimensions.0,
                                                image_config.dimensions.1,
                                            ),
                                            position: SavedPoint {
                                                x: position.x as i32,
                                                y: position.y as i32,
                                            },
                                            layer: image_config.layer.clone(),
                                        },
                                    );

                                    let saved_state = editor_state
                                        .record_state
                                        .saved_state
                                        .as_ref()
                                        .expect("Couldn't get saved state");
                                    let updated_sequence = saved_state
                                        .sequences
                                        .iter()
                                        .find(|s| s.id == selected_sequence_id.get())
                                        .expect("Couldn't get updated sequence");

                                    selected_sequence_data.set(updated_sequence.clone());

                                    let sequence_cloned = updated_sequence.clone();

                                    drop(editor_state);

                                    let mut editor = editor_cloned_3.lock().unwrap();

                                    editor.current_sequence_data = Some(sequence_cloned.clone());
                                    editor.update_motion_paths(&sequence_cloned);

                                    drop(editor);
                                }
                            }),
                            false,
                        ),
                        option_button(
                            "Add Text",
                            "text",
                            Some(move || {
                                // use text_due.rs to add text to wgpu scene
                                let mut editor = editor_cloned_2.lock().unwrap();

                                let mut rng = rand::thread_rng();
                                let random_number_800 = rng.gen_range(0..=800);
                                let random_number_450 = rng.gen_range(0..=450);

                                let new_id = Uuid::new_v4();
                                let new_text = "New text".to_string();
                                let font_family = "Aleo".to_string();

                                let position = Point {
                                    x: random_number_800 as f32 + 600.0,
                                    y: random_number_450 as f32 + 50.0,
                                };

                                let text_config = TextRendererConfig {
                                    id: new_id.clone(),
                                    name: "New Text Item".to_string(),
                                    text: new_text.clone(),
                                    font_family: font_family.clone(),
                                    dimensions: (100.0, 100.0),
                                    position,
                                    layer: -2,
                                    color: [20, 20, 200, 255],
                                    font_size: 28,
                                    background_fill: [200, 200, 200, 255],
                                };

                                let gpu_helper = gpu_cloned_2.lock().unwrap();
                                let gpu_resources = gpu_helper
                                    .gpu_resources
                                    .as_ref()
                                    .expect("Couldn't get gpu resources");
                                let device = &gpu_resources.device;
                                let queue = &gpu_resources.queue;
                                let viewport = viewport_cloned_2.lock().unwrap();
                                let window_size = WindowSize {
                                    width: viewport.width as u32,
                                    height: viewport.height as u32,
                                };

                                editor.add_text_item(
                                    &window_size,
                                    &device,
                                    &queue,
                                    text_config.clone(),
                                    new_text.clone(),
                                    new_id,
                                    selected_sequence_id.get(),
                                );

                                drop(viewport);
                                drop(gpu_helper);
                                drop(editor);

                                let mut editor_state = state_cloned_2.lock().unwrap();
                                editor_state.add_saved_text_item(
                                    selected_sequence_id.get(),
                                    SavedTextRendererConfig {
                                        id: text_config.id.to_string().clone(),
                                        name: text_config.name.clone(),
                                        text: new_text,
                                        font_family,
                                        dimensions: (
                                            text_config.dimensions.0 as i32,
                                            text_config.dimensions.1 as i32,
                                        ),
                                        position: SavedPoint {
                                            x: position.x as i32,
                                            y: position.y as i32,
                                        },
                                        layer: text_config.layer.clone(),
                                        color: text_config.color.clone(),
                                        font_size: text_config.font_size.clone(),
                                        background_fill: Some(text_config.background_fill),
                                    },
                                );

                                let saved_state = editor_state
                                    .record_state
                                    .saved_state
                                    .as_ref()
                                    .expect("Couldn't get saved state");
                                let updated_sequence = saved_state
                                    .sequences
                                    .iter()
                                    .find(|s| s.id == selected_sequence_id.get())
                                    .expect("Couldn't get updated sequence");

                                selected_sequence_data.set(updated_sequence.clone());

                                let sequence_cloned = updated_sequence.clone();

                                drop(editor_state);

                                let mut editor = editor_cloned_2.lock().unwrap();

                                editor.current_sequence_data = Some(sequence_cloned.clone());
                                editor.update_motion_paths(&sequence_cloned);

                                drop(editor);
                            }),
                            false,
                        ),
                        option_button(
                            "Add Video",
                            "video",
                            Some(move || {
                                if let Some(original_path) = rfd::FileDialog::new()
                                    .add_filter("videos", &["mp4"])
                                    .pick_file()
                                {
                                    // add a rendererstate polygon + video pair?

                                    // Get the storage directory
                                    let storage_dir = get_videos_dir();

                                    // Create a new file name to avoid conflicts
                                    let file_name =
                                        original_path.file_name().expect("Couldn't get file name");
                                    let new_path = storage_dir.join(file_name);

                                    // Copy the image to the storage directory
                                    fs::copy(&original_path, &new_path)
                                        .expect("Couldn't copy image to storage directory");

                                    import_video_to_scene(
                                        editor_cloned_14.clone(),
                                        state_cloned_14.clone(),
                                        gpu_cloned_6.clone(),
                                        viewport_cloned_8.clone(),
                                        selected_sequence_id,
                                        selected_sequence_data,
                                        new_path,
                                        None,
                                        None,
                                    );
                                }
                            }),
                            false,
                        ),
                        option_button(
                            "Capture Screen",
                            "video",
                            Some(move || {
                                let sources = get_sources().expect("Couldn't get capture sources");

                                let sources_with_titles = sources
                                    .iter()
                                    .cloned()
                                    .filter(|s| s.title.len() > 1)
                                    .collect();

                                capture_sources.set(sources_with_titles);
                                capture_selected.set(true);
                            }),
                            false,
                        ),
                        dyn_container(
                            move || capture_selected.get(),
                            move |capture_selected_real| {
                                let st_capture = st_capture.clone();
                                let capture_sources = capture_sources.clone();
                                let state_cloned_12 = state_cloned_12.clone();
                                let state_cloned_13 = state_cloned_13.clone();
                                let editor_cloned_13 = editor_cloned_13.clone();
                                let state_cloned_11 = state_cloned_11.clone();
                                let gpu_cloned_5 = gpu_cloned_5.clone();
                                let viewport_cloned_7 = viewport_cloned_7.clone();

                                if capture_selected_real {
                                    v_stack((
                                        scroll({
                                            dyn_stack(
                                                move || capture_sources.get(),
                                                move |source| source.hwnd.clone(),
                                                move |source| {
                                                    simple_button(source.title.clone(), move |_| {
                                                        selected_source.set(source.clone());
                                                        source_selected.set(true);
                                                    })
                                                    .style(|s| s.width(260.0))
                                                },
                                            )
                                            .style(
                                                |s| s.flex().flex_direction(FlexDirection::Column),
                                            )
                                        })
                                        .style(|s| s.height(200.0).width(260.0)),
                                        dyn_container(
                                            move || is_recording.get(),
                                            move |is_recording_real| {
                                                let st_capture = st_capture.clone();
                                                let state_cloned_12 = state_cloned_12.clone();
                                                let state_cloned_13 = state_cloned_13.clone();
                                                let editor_cloned_13 = editor_cloned_13.clone();
                                                let state_cloned_11 = state_cloned_11.clone();
                                                let gpu_cloned_5 = gpu_cloned_5.clone();
                                                let viewport_cloned_7 = viewport_cloned_7.clone();

                                                // let capture_dir = get_captures_dir();
                                                // let mut st_capture = StCapture::new(capture_dir);
                                                let mut st_capture = st_capture.get();

                                                if is_recording_real {
                                                    simple_button(
                                                        "Stop Capture".to_string(),
                                                        move |_| {
                                                            let editor_state =
                                                                state_cloned_12.lock().unwrap();

                                                            let project_selected =
                                                    editor_state.project_selected_signal.expect(
                                                        "Couldn't get project selection signal",
                                                    );

                                                            let project_id = project_selected.get();

                                                            let (mouse_positions_path) = st_capture
                                                                .stop_mouse_tracking(
                                                                    project_id.to_string(),
                                                                )
                                                                .expect(
                                                                    "Couldn't stop mouse tracking",
                                                                );
                                                            let (output_path, source_data_path) =
                                                            st_capture
                                                                .stop_video_capture(
                                                                    project_id.to_string(),
                                                                )
                                                                .expect(
                                                                    "Couldn't stop video capture",
                                                                );

                                                            capture_paths.set((
                                                                output_path,
                                                                mouse_positions_path,
                                                                source_data_path,
                                                            ));

                                                            source_selected.set(false);
                                                            is_recording.set(false);
                                                        },
                                                    )
                                                    .style(|s| {
                                                        s.width(260.0).background(Color::RED)
                                                    })
                                                    .into_any()
                                                } else {
                                                    simple_button(
                                                        "Start Capture".to_string(),
                                                        move |_| {
                                                            if !source_selected.get() {
                                                                println!(
                                                                    "Source must be selected!"
                                                                );
                                                                return;
                                                            }

                                                            let editor_state =
                                                                state_cloned_13.lock().unwrap();

                                                            let project_selected =
                                                    editor_state.project_selected_signal.expect(
                                                        "Couldn't get project selection signal",
                                                    );

                                                            let project_id = project_selected.get();

                                                            let source = selected_source.get();

                                                            st_capture
                                                                .start_mouse_tracking()
                                                                .expect(
                                                                    "Couldn't start mouse tracking",
                                                                );
                                                            st_capture
                                                                .start_video_capture(
                                                                    source.hwnd,
                                                                    source.rect.width as u32,
                                                                    source.rect.height as u32,
                                                                    project_id.to_string(),
                                                                )
                                                                .expect(
                                                                    "Couldn't start video capture",
                                                                );

                                                            is_recording.set(true);
                                                        },
                                                    )
                                                    .style(|s| {
                                                        s.width(260.0).background(Color::PALE_GREEN)
                                                    })
                                                    .into_any()
                                                }
                                            },
                                        ),
                                    ))
                                    .into_any()
                                } else {
                                    empty().into_any()
                                }
                            },
                        ),
                    ))
                    .style(|s| {
                        s.flex()
                            .width(260.0)
                            .flex_direction(FlexDirection::Row)
                            .justify_start()
                            .align_items(AlignItems::Start)
                            .flex_wrap(FlexWrap::Wrap)
                            .gap(5.0)
                            .margin_bottom(5.0)
                    }),
                    stack((list(themes.iter().map(|theme: &[f64; 5]| {
                        let editor_cloned_15 = editor_cloned_15.clone();
                        let state_cloned_15 = state_cloned_15.clone();

                        let background_color_row = theme[0].trunc() as usize;
                        let background_color_column = (theme[0].fract() * 10.0) as usize;
                        let background_color =
                            colors[background_color_row][background_color_column];

                        // println!(
                        //     "Background color: {:?} {:?} {:?}",
                        //     background_color_row, background_color_column, background_color
                        // );

                        let background_color: Rgb<Rgb, u8> = Rgb::from_str(&background_color)
                            .expect("Couldn't get background color");

                        let text_color_row = theme[4].trunc() as usize;
                        let text_color_column = (theme[4].fract() * 10.0) as usize;
                        let text_color = colors[text_color_row][text_color_column];

                        // println!(
                        //     "Text color: {:?} {:?} {:?}",
                        //     text_color_row, text_color_column, text_color
                        // );

                        let text_color: Rgb<Rgb, u8> =
                            Rgb::from_str(&text_color).expect("Couldn't get text color");

                        let font_index = theme[2];

                        option_button(
                            "Apply Theme",
                            "brush",
                            Some(move || {
                                // apply theme to background canvas and text objects
                                println!(
                                    "Updating text color... {} {} {}",
                                    text_color.red, text_color.green, text_color.blue
                                );

                                let text_color_wgpu = rgb_to_wgpu(
                                    text_color.red,
                                    text_color.green,
                                    text_color.blue,
                                    255.0,
                                );

                                let text_color = [
                                    text_color.red as i32,
                                    text_color.green as i32,
                                    text_color.blue as i32,
                                    255,
                                ];

                                let background_color_wgpu = rgb_to_wgpu(
                                    background_color.red,
                                    background_color.green,
                                    background_color.blue,
                                    255.0,
                                );

                                // using for text and canvas, so text_color can provide contrast
                                let background_color = [
                                    background_color.red as i32,
                                    background_color.green as i32,
                                    background_color.blue as i32,
                                    255,
                                ];

                                let mut editor = editor_cloned_15.lock().unwrap();

                                let ids_to_update: Vec<_> = editor
                                    .text_items
                                    .iter()
                                    .filter(|text| {
                                        text.current_sequence_id.to_string()
                                            == selected_sequence_id.get()
                                    })
                                    .map(|text| text.id)
                                    .collect();

                                let font_id = editor
                                    .font_manager
                                    .font_data
                                    .get(font_index as usize)
                                    .expect("Couldn't get font by index")
                                    .0
                                    .clone();

                                for id in ids_to_update.clone() {
                                    editor.update_text_color(id, background_color);
                                    editor.update_text_font_family(font_id.clone(), id);
                                }

                                drop(editor);

                                // update selected_text_data
                                // let mut new_text_data = selected_text_data.get();
                                // new_text_data.color = color;
                                // selected_text_data.set(new_text_data);

                                // save to saved_state
                                let mut editor_state = state_cloned_15.lock().unwrap();
                                let mut saved_state = editor_state
                                    .record_state
                                    .saved_state
                                    .as_mut()
                                    .expect("Couldn't get Saved State");

                                saved_state.sequences.iter_mut().for_each(|s| {
                                    if s.id == selected_sequence_id.get() {
                                        s.active_text_items.iter_mut().for_each(|t| {
                                            // if t.id == selected_text_id.get().to_string() {
                                            t.color = background_color;
                                            // }
                                        });
                                    }
                                });

                                drop(editor_state);

                                // save_saved_state_raw(saved_state.clone());

                                // editor_state.record_state.saved_state = Some(saved_state.clone());

                                println!("Updating text background...");

                                // let red_human = wgpu_to_human(*h);

                                let mut editor = editor_cloned_15.lock().unwrap();

                                for id in ids_to_update.clone() {
                                    editor.update_text(
                                        id,
                                        "red_fill",
                                        InputValue::Number(text_color_wgpu[0] as f32),
                                    );
                                    editor.update_text(
                                        id,
                                        "green_fill",
                                        InputValue::Number(text_color_wgpu[1] as f32),
                                    );
                                    editor.update_text(
                                        id,
                                        "blue_fill",
                                        InputValue::Number(text_color_wgpu[2] as f32),
                                    );
                                }

                                drop(editor);

                                let mut editor_state = state_cloned_15.lock().unwrap();
                                let mut saved_state = editor_state
                                    .record_state
                                    .saved_state
                                    .as_mut()
                                    .expect("Couldn't get Saved State");

                                saved_state.sequences.iter_mut().for_each(|s| {
                                    // if s.id == selected_sequence_id.get() {
                                    s.active_text_items.iter_mut().for_each(|p| {
                                        // if p.id == self.object_id.to_string() {
                                        let background_fill = p
                                            .background_fill
                                            .as_mut()
                                            .expect("Couldn't get bg fill");
                                        *background_fill = text_color;
                                        // }
                                    });
                                    // }
                                });

                                drop(editor_state);

                                println!("Updating canvas background...");

                                let mut editor = editor_cloned_15.lock().unwrap();

                                let background_uuid = Uuid::from_str(&selected_sequence_id.get())
                                    .expect("Couldn't convert string to uuid");

                                editor.update_background(
                                    background_uuid,
                                    "red",
                                    InputValue::Number(background_color[0] as f32),
                                );
                                editor.update_background(
                                    background_uuid,
                                    "green",
                                    InputValue::Number(background_color[1] as f32),
                                );
                                editor.update_background(
                                    background_uuid,
                                    "blue",
                                    InputValue::Number(background_color[2] as f32),
                                );

                                drop(editor);

                                let mut editor_state = state_cloned_15.lock().unwrap();
                                let mut saved_state = editor_state
                                    .record_state
                                    .saved_state
                                    .as_mut()
                                    .expect("Couldn't get Saved State");
                                // let mut red = h.to_string();
                                // self.signal.expect("signal error").set(red);

                                saved_state.sequences.iter_mut().for_each(|s| {
                                    if s.id == selected_sequence_id.get() {
                                        if s.background_fill.is_none() {
                                            s.background_fill = Some(BackgroundFill::Color([
                                                wgpu_to_human(0.8) as i32,
                                                wgpu_to_human(0.8) as i32,
                                                wgpu_to_human(0.8) as i32,
                                                255,
                                            ]));
                                        }

                                        let background_fill = s
                                            .background_fill
                                            .as_mut()
                                            .expect("Couldn't get background fill");

                                        match background_fill {
                                            BackgroundFill::Color(fill) => {
                                                *fill = background_color;
                                            }
                                            _ => {
                                                println!("Not supported");
                                            }
                                        }
                                    }
                                });

                                save_saved_state_raw(saved_state.clone());
                                // editor_state.record_state.saved_state = Some(saved_state.clone()); // all are iter_mut

                                drop(editor_state);
                            }),
                            false,
                        )
                        .style(move |s| {
                            s.width(260.0)
                                .background(Color::rgb8(
                                    background_color.red,
                                    background_color.green,
                                    background_color.blue,
                                ))
                                .color(Color::rgb8(
                                    text_color.red,
                                    text_color.green,
                                    text_color.blue,
                                ))
                        })
                    })),))
                    .style(move |s| {
                        s.flex()
                            .width(260.0)
                            .flex_direction(FlexDirection::Row)
                            .justify_start()
                            .align_items(AlignItems::Start)
                            .flex_wrap(FlexWrap::Wrap)
                            .gap(5.0)
                            .margin_bottom(5.0)
                    }),
                    v_stack((
                        label(|| "Background Color").style(|s| s.margin_bottom(5.0)),
                        h_stack((
                            debounce_input(
                                "Red:".to_string(),
                                &wgpu_to_human(background_polygon_data.read().borrow().fill[0])
                                    .to_string(),
                                "0-255",
                                move |value| {
                                    let mut editor_state = state_cloned_16.lock().unwrap();

                                    editor_state
                                        .update_background_red(&value, selected_sequence_id.get())
                                        .expect("Couldn't update red");

                                    drop(editor_state);
                                },
                                state_cloned_17,
                                "red".to_string(),
                                ObjectType::Polygon,
                            )
                            .style(move |s| s.width(thirds).margin_right(5.0)),
                            debounce_input(
                                "Green:".to_string(),
                                &wgpu_to_human(background_polygon_data.read().borrow().fill[1])
                                    .to_string(),
                                "0-255",
                                move |value| {
                                    let mut editor_state = state_cloned_18.lock().unwrap();

                                    editor_state
                                        .update_background_green(&value, selected_sequence_id.get())
                                        .expect("Couldn't update green");

                                    drop(editor_state);
                                },
                                state_cloned_19,
                                "green".to_string(),
                                ObjectType::Polygon,
                            )
                            .style(move |s| s.width(thirds).margin_right(5.0)),
                            debounce_input(
                                "Blue:".to_string(),
                                &wgpu_to_human(background_polygon_data.read().borrow().fill[2])
                                    .to_string(),
                                "0-255",
                                move |value| {
                                    let mut editor_state = state_cloned_20.lock().unwrap();

                                    editor_state
                                        .update_background_blue(&value, selected_sequence_id.get())
                                        .expect("Couldn't update blue");

                                    drop(editor_state);
                                },
                                state_cloned_21,
                                "blue".to_string(),
                                ObjectType::Polygon,
                            )
                            .style(move |s| s.width(thirds)),
                        ))
                        .style(move |s| s.width(aside_width)),
                    )),
                ))),
            )
            // .style(|s: floem::style::Style| s.flex_col().column_gap(2))
            .style(move |s| s.height(window_height.get() / 2.0 - 200.0)),
        ))
        .style(|s| card_styles(s))
        .style(move |s| {
            s.width(300)
                // .absolute()
                .height(window_height.get() / 2.0 - 120.0)
                .margin_left(0.0)
                .margin_top(20)
                .z_index(1)
        }),
        v_stack((
            label(|| "Scene").style(|s| s.font_size(14.0).margin_bottom(10.0)),
            scroll(
                dyn_stack(
                    move || layers.get(),
                    |layer: &Layer| layer.instance_id,
                    move |layer| {
                        let editor = editor_cloned_7.clone();
                        let on_items_updated = on_items_updated.clone();
                        let on_item_duplicated = on_item_duplicated.clone();
                        let on_item_deleted = on_item_deleted.clone();

                        let icon_name = match layer.instance_kind {
                            LayerKind::Polygon => "square",
                            LayerKind::Text => "text",
                            LayerKind::Image => "image",
                            LayerKind::Video => "video",
                            // LayerKind::Path =>
                            //         // LayerKind::Imag(data) =>
                            //         // LayerKind::Text =>
                            //         // LayerKind::Group =>
                        };
                        sortable_item(
                            editor,
                            layers,
                            dragger_id,
                            layer.instance_id,
                            layer.instance_kind,
                            layer.instance_name.clone(),
                            icon_name,
                            on_items_updated,
                            on_item_duplicated,
                            on_item_deleted,
                        )
                    },
                )
                .style(|s: floem::style::Style| s.flex_col().column_gap(2))
                .into_view(),
            )
            .style(move |s| s.height(window_height.get() / 2.0 - 190.0)),
        ))
        .style(|s| card_styles(s))
        .style(move |s| {
            s.width(300)
                // .absolute()
                .height(window_height.get() / 2.0 - 120.0)
                .margin_left(0.0)
                .margin_top(20)
                .z_index(1)
        }),
    ))
}
