use std::cmp::Ordering;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use crossbeam::queue;
use floem::common::{card_styles, create_icon, option_button, simple_button, toggle_button};
use floem::peniko::Color;
use floem::reactive::{create_effect, create_rw_signal, SignalUpdate};
use floem::reactive::{RwSignal, SignalGet};
use floem::style::CursorStyle;
use floem::taffy::{AlignItems, FlexDirection, FlexWrap};
use floem::views::{dyn_stack, h_stack, scroll, stack, svg, v_stack, Decorators};
use floem::GpuHelper;
use floem::{views::label, IntoView};
use floem_renderer::gpu_resources;
use rand::Rng;
use stunts_engine::editor::{Editor, Point, Viewport, WindowSize};
use stunts_engine::polygon::{
    Polygon, PolygonConfig, SavedPoint, SavedPolygonConfig, SavedStroke, Stroke,
};
use stunts_engine::st_image::{SavedStImageConfig, StImageConfig};
use stunts_engine::text_due::{SavedTextRendererConfig, TextRendererConfig};
use uuid::Uuid;

use crate::editor_state::{self, EditorState};
use crate::helpers::utilities::save_saved_state_raw;
use stunts_engine::animations::{
    AnimationData, AnimationProperty, EasingType, KeyframeValue, Sequence, UIKeyframe,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LayerKind {
    Polygon,
    // Path,
    Image,
    Text,
    // Group,
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
}

pub fn sortable_item<F>(
    editor: std::sync::Arc<Mutex<Editor>>,
    sortable_items: RwSignal<Vec<Layer>>,
    dragger_id: RwSignal<Uuid>,
    item_id: Uuid,
    layer_name: String,
    icon_name: &'static str,
    on_items_updated: F,
) -> impl IntoView
where
    F: Fn() + Clone + 'static,
{
    h_stack((
        svg(create_icon(icon_name))
            .style(|s| s.width(24).height(24).color(Color::BLACK))
            .style(|s| s.margin_right(7.0))
            .on_event_stop(
                floem::event::EventListener::PointerDown,
                |_| { /* Disable dragging for this view */ },
            ),
        label(move || layer_name.to_string())
            .style(|s| s.selectable(false).cursor(CursorStyle::RowResize)),
    ))
    .style(|s| s.selectable(false).cursor(CursorStyle::RowResize))
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
        s.width(220.0)
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

pub fn sequence_panel(
    editor_state: Arc<Mutex<EditorState>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    editor: std::sync::Arc<Mutex<Editor>>,
    viewport: Arc<Mutex<Viewport>>,
    sequence_selected: RwSignal<bool>,
    selected_sequence_id: RwSignal<String>,
    selected_sequence_data: RwSignal<Sequence>,
) -> impl IntoView {
    let state_cloned = Arc::clone(&editor_state);
    let state_cloned_2 = Arc::clone(&editor_state);
    let state_cloned_3 = Arc::clone(&editor_state);
    let state_cloned_4 = Arc::clone(&editor_state);
    let state_cloned_5 = Arc::clone(&editor_state);
    let editor_cloned = Arc::clone(&editor);
    let editor_cloned_2 = Arc::clone(&editor);
    let editor_cloned_3 = Arc::clone(&editor);
    let editor_cloned_4 = Arc::clone(&editor);
    let editor_cloned_5 = Arc::clone(&editor);
    let editor_cloned_6 = Arc::clone(&editor);
    let editor_cloned_7 = Arc::clone(&editor);
    let editor_cloned_8 = Arc::clone(&editor);
    let gpu_cloned = Arc::clone(&gpu_helper);
    let viewport_cloned = Arc::clone(&viewport);
    let gpu_cloned_2 = Arc::clone(&gpu_helper);
    let viewport_cloned_2 = Arc::clone(&viewport);
    let gpu_cloned_3 = Arc::clone(&gpu_helper);
    let viewport_cloned_3 = Arc::clone(&viewport);
    let viewport_cloned_4 = Arc::clone(&viewport);

    let selected_file = create_rw_signal(None::<PathBuf>);
    let local_mode = create_rw_signal("layout".to_string());

    let layers: RwSignal<Vec<Layer>> = create_rw_signal(Vec::new());
    let layers_ref = Arc::new(Mutex::new(layers));
    let window_height = create_rw_signal(0.0);
    let dragger_id = create_rw_signal(Uuid::nil());

    create_effect({
        let editor_cloned_6 = Arc::clone(&editor_cloned_6);

        move |_| {
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

            // sort layers by layer_index property, lower values should come first in the list
            // but reverse the order because the UI outputs the first one first, thus it displays last
            new_layers.sort_by(|a, b| b.initial_layer_index.cmp(&a.initial_layer_index));

            layers.set(new_layers);
        }
    });

    create_effect({
        let viewport_cloned_4 = Arc::clone(&viewport_cloned_4);
        move |_| {
            let viewport = viewport_cloned_4.lock().unwrap();

            window_height.set(viewport.height);
        }
    });

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
            });

        drop(editor);

        let mut editor_state = state_cloned_5.lock().unwrap();

        let mut saved_state = editor_state
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
                    });
            }
        });

        save_saved_state_raw(saved_state.clone());

        editor_state.saved_state = Some(saved_state.clone());

        drop(editor_state);
    };

    v_stack((
        v_stack((
            label(move || format!("Update Sequence")).style(|s| s.margin_bottom(10)),
            simple_button("Back to Sequence List".to_string(), move |_| {
                sequence_selected.set(false);

                let mut editor = editor_cloned_5.lock().unwrap();

                editor.hide_all_objects();

                drop(editor);
            })
            .style(|s| s.margin_bottom(5.0)),
            v_stack((
                simple_button("Generate Animation".to_string(), move |_| {
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
                        .saved_state
                        .as_mut()
                        .expect("Couldn't get Saved State");

                    saved_state.sequences.iter_mut().for_each(|s| {
                        if s.id == selected_sequence_id.get() {
                            s.polygon_motion_paths = predicted_keyframes.clone();
                        }
                    });

                    save_saved_state_raw(saved_state.clone());

                    editor_state.saved_state = Some(saved_state.clone());

                    drop(editor_state);
                })
                .style(|s| s.background(Color::rgb8(255, 25, 25)).color(Color::WHITE)),
                // maybe not needed after all
                // h_stack((
                //     toggle_button(
                //         "Layout",
                //         "translate",
                //         "layout".to_string(),
                //         {
                //             let state_cloned_4 = state_cloned_4.clone();

                //             move |_| {
                //                 let mut state_helper = state_cloned_4.lock().unwrap();

                //                 local_mode.set("layout".to_string());
                //                 state_helper.active_sequence_mode.set("layout".to_string());
                //             }
                //         },
                //         local_mode,
                //     )
                //     .style(|s| s.margin_right(4.0)),
                //     toggle_button(
                //         "Keyframes",
                //         "scale",
                //         "keyframes".to_string(),
                //         {
                //             let state_cloned_5 = state_cloned_5.clone();

                //             move |_| {
                //                 let mut state_helper = state_cloned_5.lock().unwrap();

                //                 local_mode.set("keyframes".to_string());
                //                 state_helper
                //                     .active_sequence_mode
                //                     .set("keyframes".to_string());
                //             }
                //         },
                //         local_mode,
                //     ),
                // )),
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
                    }),
                    false,
                ),
                option_button(
                    "Add Image",
                    "image",
                    Some(move || {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("images", &["png", "jpg", "jpeg"])
                            .pick_file()
                        {
                            // selected_file.set(Some(path));

                            // add a rendererstate polygon + image pair? + add as image to saved state?

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
                                path: path.to_str().expect("Couldn't get path string").to_string(),
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
                                &path,
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
                                    path: path
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
                        let new_text = "Hello world!".to_string();
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
                        };

                        let gpu_helper = gpu_cloned_2.lock().unwrap();
                        let device = &gpu_helper
                            .gpu_resources
                            .as_ref()
                            .expect("Couldn't get gpu resources")
                            .device;
                        let viewport = viewport_cloned_2.lock().unwrap();
                        let window_size = WindowSize {
                            width: viewport.width as u32,
                            height: viewport.height as u32,
                        };

                        editor.add_text_item(
                            &window_size,
                            &device,
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
                            },
                        );
                    }),
                    false,
                ),
                // option_button(
                //     "Add Video",
                //     "video",
                //     Some(move || {
                //         // import with decoder
                //     }),
                //     false,
                // ),
            ))
            .style(|s| {
                s.flex()
                    .width(260.0)
                    .flex_direction(FlexDirection::Row)
                    .justify_start()
                    .align_items(AlignItems::Start)
                    .flex_wrap(FlexWrap::Wrap)
                    .gap(5.0)
            }),
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
            label(|| "Scene").style(|s| s.font_size(14.0).margin_bottom(15.0)),
            scroll(
                dyn_stack(
                    move || layers.get(),
                    |layer: &Layer| layer.instance_id,
                    move |layer| {
                        let editor = editor_cloned_7.clone();
                        let on_items_updated = on_items_updated.clone();
                        let icon_name = match layer.instance_kind {
                            LayerKind::Polygon => "triangle",
                            LayerKind::Text => "sphere",
                            LayerKind::Image => "triangle",
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
                            layer.instance_name.clone(),
                            icon_name,
                            on_items_updated,
                        )
                    },
                )
                .style(|s: floem::style::Style| s.flex_col().column_gap(5).padding(10))
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
