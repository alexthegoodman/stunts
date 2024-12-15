use std::sync::{Arc, Mutex};

use floem::common::{card_styles, option_button, simple_button};
use floem::reactive::SignalUpdate;
use floem::reactive::{RwSignal, SignalGet};
use floem::views::{v_stack, Decorators};
use floem::GpuHelper;
use floem::{views::label, IntoView};
use stunts_engine::editor::{Editor, Point, Viewport, WindowSize};
use stunts_engine::polygon::{Polygon, PolygonConfig, SavedPolygonConfig, Stroke};
use uuid::Uuid;

use crate::editor_state::EditorState;
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
    let editor_cloned = Arc::clone(&editor);
    let gpu_cloned = Arc::clone(&gpu_helper);
    let viewport_cloned = Arc::clone(&viewport);

    v_stack((
        label(move || format!("Assets / Motion")).style(|s| s.margin_bottom(10)),
        simple_button("Back to Sequence List".to_string(), move |_| {
            sequence_selected.set(false);
        }),
        option_button(
            "Add Square",
            "square",
            Some(move || {
                let mut editor = editor_cloned.lock().unwrap();
                // let mut square_handler = square_handler.lock().unwrap();
                println!("Handle square...");

                // square_handler.handle_button_click(editor_cloned);

                let polygon_config = PolygonConfig {
                    id: Uuid::new_v4(),
                    name: "Square".to_string(),
                    points: vec![
                        Point { x: 0.0, y: 0.0 },
                        Point { x: 1.0, y: 0.0 },
                        Point { x: 1.0, y: 1.0 },
                        Point { x: 0.0, y: 1.0 },
                    ],
                    dimensions: (100.0, 100.0),
                    position: Point { x: 600.0, y: 100.0 },
                    border_radius: 0.0,
                    fill: [1.0, 1.0, 1.0, 1.0],
                    stroke: Stroke {
                        fill: [1.0, 1.0, 1.0, 1.0],
                        thickness: 2.0,
                    },
                };
                let gpu_helper = gpu_cloned.lock().unwrap();
                let device = &gpu_helper
                    .gpu_resources
                    .as_ref()
                    .expect("Couldn't get gpu resources")
                    .device;
                let viewport = viewport_cloned.lock().unwrap();
                let window_size = WindowSize {
                    width: viewport.width as u32,
                    height: viewport.height as u32,
                };
                let camera = editor.camera.expect("Couldn't get camera");
                let new_id = Uuid::new_v4();
                editor.add_polygon(
                    &window_size,
                    &device,
                    &camera,
                    polygon_config.clone(),
                    "Polygon".to_string(),
                    new_id,
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
                    },
                );
            }),
            false,
        ),
    ))
    .style(|s| card_styles(s))
    .style(|s| s.width(300.0))
}
