use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crossbeam::queue;
use floem::common::{card_styles, option_button, simple_button, toggle_button};
use floem::peniko::Color;
use floem::reactive::{create_rw_signal, SignalUpdate};
use floem::reactive::{RwSignal, SignalGet};
use floem::taffy::{AlignItems, FlexDirection, FlexWrap};
use floem::views::{h_stack, stack, v_stack, Decorators};
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

use crate::editor_state::EditorState;
use crate::helpers::utilities::save_saved_state_raw;
use stunts_engine::animations::{
    AnimationData, AnimationProperty, EasingType, KeyframeValue, Sequence, UIKeyframe,
};

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
    let gpu_cloned = Arc::clone(&gpu_helper);
    let viewport_cloned = Arc::clone(&viewport);
    let gpu_cloned_2 = Arc::clone(&gpu_helper);
    let viewport_cloned_2 = Arc::clone(&viewport);
    let gpu_cloned_3 = Arc::clone(&gpu_helper);
    let viewport_cloned_3 = Arc::clone(&viewport);

    let selected_file = create_rw_signal(None::<PathBuf>);
    let local_mode = create_rw_signal("layout".to_string());

    v_stack((
        label(move || format!("Create Sequence")).style(|s| s.margin_bottom(10)),
        simple_button("Back to Sequence List".to_string(), move |_| {
            sequence_selected.set(false);
        }),
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
        )),
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
                                dimensions: (image_config.dimensions.0, image_config.dimensions.1),
                                position: SavedPoint {
                                    x: position.x as i32,
                                    y: position.y as i32,
                                },
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

                    let position = Point {
                        x: random_number_800 as f32 + 600.0,
                        y: random_number_450 as f32 + 50.0,
                    };

                    let text_config = TextRendererConfig {
                        id: new_id.clone(),
                        name: "New Text Item".to_string(),
                        text: new_text.clone(),
                        dimensions: (100.0, 100.0),
                        position,
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
                            dimensions: (
                                text_config.dimensions.0 as i32,
                                text_config.dimensions.1 as i32,
                            ),
                            position: SavedPoint {
                                x: position.x as i32,
                                y: position.y as i32,
                            },
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
    .style(|s| s.width(300.0))
}
