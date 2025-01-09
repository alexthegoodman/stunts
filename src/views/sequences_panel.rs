use std::sync::{Arc, Mutex};

use floem::common::{card_styles, option_button, simple_button, small_button};
use floem::peniko::Color;
use floem::reactive::SignalGet;
use floem::reactive::{create_effect, create_rw_signal, RwSignal, SignalUpdate};
use floem::views::{
    h_stack, scroll, stack, v_stack, virtual_stack, Decorators, VirtualDirection, VirtualItemSize,
};
use floem::GpuHelper;
use floem::{views::label, IntoView};
use floem_renderer::gpu_resources;
use im::HashMap;
use rand::Rng;
use std::str::FromStr;
use stunts_engine::editor::{rgb_to_wgpu, Editor, Point, Viewport, WindowSize};
use stunts_engine::polygon::{Polygon, PolygonConfig, SavedPolygonConfig, Stroke};
use stunts_engine::timelines::{TimelineSequence, TrackType};
use uuid::Uuid;

use crate::editor_state::EditorState;
use crate::helpers::utilities::save_saved_state_raw;
use stunts_engine::animations::{
    AnimationData, AnimationProperty, EasingType, KeyframeValue, Sequence, UIKeyframe,
};

use super::sequence_timeline::{build_timeline, TimelineState};

pub fn sequences_view(
    gpu_helper: Arc<Mutex<GpuHelper>>,
    editor_state: Arc<Mutex<EditorState>>,
    editor: std::sync::Arc<Mutex<Editor>>,
    viewport: Arc<Mutex<Viewport>>,
    selected_sequence_data: RwSignal<Sequence>,
    selected_sequence_id: RwSignal<String>,
    sequence_selected: RwSignal<bool>,
) -> impl IntoView {
    let editor_cloned = Arc::clone(&editor);
    let editor_cloned2 = Arc::clone(&editor);
    let gpu_cloned = Arc::clone(&gpu_helper);
    let viewport_cloned = Arc::clone(&viewport);
    let state_cloned = Arc::clone(&editor_state);
    let state_cloned2 = Arc::clone(&editor_state);
    let state_cloned3 = Arc::clone(&editor_state);
    let state_cloned4 = Arc::clone(&editor_state);
    let state_cloned5 = Arc::clone(&editor_state);

    // let sequences: RwSignal<im::Vector<Sequence>> = create_rw_signal(im::Vector::new());
    let sequences: RwSignal<im::Vector<String>> = create_rw_signal(im::Vector::new());
    let sequence_quick_access: RwSignal<HashMap<String, i32>> = create_rw_signal(HashMap::new());

    let sequence_timeline_signal = create_rw_signal(TimelineState::new());

    create_effect(move |_| {
        let mut editor_state = editor_state.lock().unwrap();
        let saved_state = editor_state
            .saved_state
            .as_ref()
            .expect("Couldn't get Saved State");

        // let im_sequences: im::Vector<Sequence> =
        //     saved_state.sequences.clone().into_iter().collect();
        let im_sequences: im::Vector<String> = saved_state
            .sequences
            .clone()
            .into_iter()
            .map(|s| s.id)
            .collect();

        let mut x = 0;

        let qa_sequences: HashMap<String, i32> = saved_state
            .sequences
            .clone()
            .into_iter()
            .map(|s| {
                x = x + 1;
                (s.id, x)
            })
            .collect();

        sequences.set(im_sequences);
        sequence_quick_access.set(qa_sequences);

        // initialize TimelineState based on stored config if exists or saved sequences if not
        if saved_state.timeline_state.timeline_sequences.len() > 0 {
            let new_timeline_state = TimelineState::new();

            new_timeline_state
                .timeline_sequences
                .set(saved_state.timeline_state.timeline_sequences.clone());

            sequence_timeline_signal.set(new_timeline_state);
        } else {
            let new_timeline_state = TimelineState::new();
            let timeline_sequences: Vec<TimelineSequence> = saved_state
                .sequences
                .clone()
                .into_iter()
                .enumerate() // Add enumerate() to get the index
                .map(|(index, s)| TimelineSequence {
                    id: s.id,
                    track_type: TrackType::Video,
                    duration_ms: 20000,
                    start_time_ms: index as i32 * 20000, // Multiply index by 20000
                })
                .collect();

            new_timeline_state
                .timeline_sequences
                .set(timeline_sequences);

            sequence_timeline_signal.set(new_timeline_state);
        }
    });

    h_stack((
        v_stack((
            label(move || format!("Sequences")).style(|s| s.margin_bottom(10)),
            simple_button("New Sequence".to_string(), move |_| {
                println!("New Sequence...");

                let new_sequence_id = Uuid::new_v4().to_string();
                let new_sequence = Sequence {
                    id: new_sequence_id.clone(),
                    active_polygons: Vec::new(),
                    polygon_motion_paths: Vec::new(),
                    active_text_items: Vec::new(),
                    active_image_items: Vec::new(),
                };

                let mut editor_state = state_cloned.lock().unwrap();
                let mut new_state = editor_state
                    .saved_state
                    .as_mut()
                    .expect("Couldn't get Saved State")
                    .clone();
                new_state.sequences.push(new_sequence);

                editor_state.saved_state = Some(new_state.clone());

                save_saved_state_raw(new_state.clone());

                let mut x = 0;
                let qa_sequences: HashMap<String, i32> = new_state
                    .sequences
                    .clone()
                    .into_iter()
                    .map(|s| {
                        x = x + 1;
                        (s.id, x)
                    })
                    .collect();

                sequence_quick_access.set(qa_sequences);

                sequences.update(|s| s.push_back(new_sequence_id.clone()));

                // EventPropagation::Continue
            }),
            scroll({
                virtual_stack(
                    VirtualDirection::Vertical,
                    VirtualItemSize::Fixed(Box::new(|| 28.0)),
                    move || sequences.get(),
                    move |item| item.clone(),
                    move |item| {
                        let state_cloned2 = state_cloned2.clone();
                        let state_cloned3 = state_cloned3.clone();
                        let state_cloned4 = state_cloned4.clone();
                        let editor_cloned = editor_cloned.clone();
                        let viewport_cloned = viewport_cloned.clone();

                        let item_cloned = item.clone();

                        let sequence_quick_access = sequence_quick_access.get();
                        let quick_access_info = sequence_quick_access
                            .get(&item)
                            .expect("Couldn't find matching qa info");

                        let small_labels: Vec<String> = item
                            .split("-")
                            .into_iter()
                            .map(|id| id.to_string())
                            .collect();

                        let mut small_label = "".to_string();
                        let mut y = 0;
                        for label in small_labels {
                            if y == 0 {
                                small_label += &label;
                                y = y + 1;
                            }
                        }

                        h_stack((
                            simple_button(
                                "Edit ".to_string() + &small_label.to_string(),
                                move |_| {
                                    println!("Open Sequence...");

                                    let editor_state = state_cloned2.lock().unwrap();
                                    let saved_state = editor_state
                                        .saved_state
                                        .as_ref()
                                        .expect("Couldn't get Saved State");

                                    let saved_sequence = saved_state
                                        .sequences
                                        .iter()
                                        .find(|s| s.id == item.clone())
                                        .expect("Couldn't find matching sequence");

                                    selected_sequence_data.set(saved_sequence.clone());
                                    selected_sequence_id.set(item.clone());
                                    sequence_selected.set(true);

                                    let mut editor = editor_cloned.lock().unwrap();

                                    let camera = editor.camera.expect("Couldn't get camera");
                                    let viewport = viewport_cloned.lock().unwrap();

                                    let window_size = WindowSize {
                                        width: viewport.width as u32,
                                        height: viewport.height as u32,
                                    };

                                    let mut rng = rand::thread_rng();

                                    editor.polygons = Vec::new();

                                    saved_sequence.active_polygons.iter().for_each(|p| {
                                        let gpu_resources = editor
                                            .gpu_resources
                                            .as_ref()
                                            .expect("Couldn't get GPU Resources");

                                        // Generate a random number between 0 and 800
                                        // let random_number_800 = rng.gen_range(0..=800);

                                        // Generate a random number between 0 and 450
                                        // let random_number_450 = rng.gen_range(0..=450);

                                        let restored_polygon = Polygon::new(
                                            &window_size,
                                            &gpu_resources.device,
                                            &gpu_resources.queue,
                                            &editor
                                                .model_bind_group_layout
                                                .as_ref()
                                                .expect("Couldn't get model bind group layout"),
                                            &camera,
                                            // TODO: restoring triangles or non rectangles?
                                            vec![
                                                Point { x: 0.0, y: 0.0 },
                                                Point { x: 1.0, y: 0.0 },
                                                Point { x: 1.0, y: 1.0 },
                                                Point { x: 0.0, y: 1.0 },
                                            ],
                                            (p.dimensions.0 as f32, p.dimensions.1 as f32),
                                            Point {
                                                // x: random_number_800 as f32,
                                                // y: random_number_450 as f32,
                                                x: p.position.x as f32,
                                                y: p.position.y as f32,
                                            },
                                            // TODO: restore rotation?
                                            0.0,
                                            p.border_radius as f32,
                                            [
                                                p.fill[0] as f32,
                                                p.fill[1] as f32,
                                                p.fill[2] as f32,
                                                p.fill[3] as f32,
                                            ],
                                            Stroke {
                                                thickness: p.stroke.thickness as f32,
                                                fill: [
                                                    p.stroke.fill[0] as f32,
                                                    p.stroke.fill[1] as f32,
                                                    p.stroke.fill[2] as f32,
                                                    p.stroke.fill[3] as f32,
                                                ],
                                            },
                                            0.0,
                                            p.name.clone(),
                                            Uuid::from_str(&p.id)
                                                .expect("Couldn't convert string to uuid"),
                                        );

                                        // editor.add_polygon(restored_polygon);
                                        editor.polygons.push(restored_polygon);
                                    });

                                    println!("Polygons restored!");

                                    editor.update_motion_paths(&saved_sequence);

                                    println!("Motion Paths restored!");

                                    // EventPropagation::Continue
                                },
                            ),
                            simple_button("Duplicate".to_string(), move |_| {
                                println!("Duplicating sequence...");

                                let mut editor_state = state_cloned3.lock().unwrap();
                                let mut new_state = editor_state
                                    .saved_state
                                    .as_mut()
                                    .expect("Couldn't get Saved State")
                                    .clone();

                                let mut dup_sequence = new_state
                                    .sequences
                                    .iter_mut()
                                    .find(|s| s.id == item_cloned.clone())
                                    .expect("Couldn't find matching sequence")
                                    .clone();

                                let new_sequence_id = Uuid::new_v4().to_string();

                                dup_sequence.id = new_sequence_id.clone();

                                new_state.sequences.push(dup_sequence.clone());

                                editor_state.saved_state = Some(new_state.clone());

                                save_saved_state_raw(new_state.clone());

                                sequences.update(|s| s.push_back(new_sequence_id.clone()));

                                println!("Sequence duplicated!");
                            }),
                            simple_button("Add to Timeline".to_string(), move |_| {
                                println!("Adding sequence to sequence timeline...");

                                let mut editor_state = state_cloned4.lock().unwrap();

                                // let mut existing_timeline = editor_state
                                //     .sequence_timeline_state
                                //     .timeline_sequences
                                //     .get();

                                let sequence_timeline_state = sequence_timeline_signal.get();

                                let mut existing_timeline =
                                    sequence_timeline_state.timeline_sequences.get();

                                // Find the sequence that ends at the latest point in time
                                let start_time = if existing_timeline.is_empty() {
                                    0
                                } else {
                                    existing_timeline
                                        .iter()
                                        .map(|seq| seq.start_time_ms + seq.duration_ms)
                                        .max()
                                        .unwrap_or(0)
                                };

                                existing_timeline.push(TimelineSequence {
                                    id: Uuid::new_v4().to_string(),
                                    track_type: TrackType::Video,
                                    start_time_ms: start_time,
                                    duration_ms: 20000,
                                });

                                sequence_timeline_state
                                    .timeline_sequences
                                    .set(existing_timeline);

                                let new_savable = sequence_timeline_state.to_config();

                                let mut new_state = editor_state
                                    .saved_state
                                    .as_mut()
                                    .expect("Couldn't get Saved State")
                                    .clone();

                                new_state.timeline_state = new_savable;

                                editor_state.saved_state = Some(new_state.clone());

                                save_saved_state_raw(new_state.clone());

                                println!("Sequence added!");
                            }),
                        ))
                    },
                )
                .style(|s| {
                    s.flex_col()
                        .width(260.0)
                        .padding_vert(15.0)
                        .padding_horiz(20.0)
                        .background(Color::LIGHT_BLUE)
                })
            })
            .style(|s| s.height(400.0)),
        ))
        .style(|s| card_styles(s))
        .style(|s| s.width(300.0)),
        v_stack((
            simple_button("Play Video".to_string(), move |_| {
                let mut editor_state = state_cloned5.lock().unwrap();

                let saved_state = editor_state
                    .saved_state
                    .as_ref()
                    .expect("Couldn't get saved state");
                let cloned_sequences = saved_state.sequences.clone();

                drop(editor_state);

                let mut editor = editor_cloned2.lock().unwrap();

                if editor.video_is_playing {
                    println!("Pause Video...");

                    editor.video_is_playing = false;
                    editor.video_start_playing_time = None;
                    editor.video_current_sequence_timeline = None;
                    editor.video_current_sequences_data = None;
                    editor.is_playing = false;
                    editor.start_playing_time = None;
                } else {
                    println!("Play Video...");

                    let now = std::time::Instant::now();
                    editor.video_start_playing_time = Some(now.clone());

                    editor.video_current_sequence_timeline =
                        Some(sequence_timeline_signal.get().to_config());
                    editor.video_current_sequences_data = Some(cloned_sequences);

                    editor.video_is_playing = true;

                    // also set motion path playing
                    editor.start_playing_time = Some(now);
                    editor.is_playing = true;
                }

                // EventPropagation::Continue
            }),
            build_timeline(sequence_timeline_signal.get()),
        ))
        .style(|s| s.margin_top(425.0).margin_left(25.0)),
    ))
}
