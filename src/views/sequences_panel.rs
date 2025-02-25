use std::path::Path;
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
use stunts_engine::editor::{rgb_to_wgpu, wgpu_to_human, Editor, Point, Viewport, WindowSize};
use stunts_engine::polygon::{Polygon, PolygonConfig, SavedPolygonConfig, Stroke};
use stunts_engine::st_image::{StImage, StImageConfig};
use stunts_engine::text_due::{TextRenderer, TextRendererConfig};
use stunts_engine::timelines::{SavedTimelineStateConfig, TimelineSequence, TrackType};
use uuid::Uuid;

use crate::editor_state::EditorState;
use crate::helpers::utilities::{parse_animation_data, save_saved_state_raw};
use stunts_engine::animations::{
    AnimationData, AnimationProperty, BackgroundFill, EasingType, KeyframeValue, Sequence,
    UIKeyframe,
};

use super::export::export_widget;
use super::keyframe_timeline::TimelineState;
use super::sequence_timeline::build_timeline;

fn get_last_n_items<T>(vec: &[T], n: usize) -> &[T] {
    if n > vec.len() {
        &vec[..]
    } else {
        &vec[vec.len() - n..]
    }
}

pub fn sequences_view(
    gpu_helper: Arc<Mutex<GpuHelper>>,
    editor_state: Arc<Mutex<EditorState>>,
    editor: std::sync::Arc<Mutex<Editor>>,
    viewport: Arc<Mutex<Viewport>>,
    selected_sequence_data: RwSignal<Sequence>,
    selected_sequence_id: RwSignal<String>,
    sequence_selected: RwSignal<bool>,
    polygon_selected: RwSignal<bool>,
) -> impl IntoView {
    let editor_cloned = Arc::clone(&editor);
    let editor_cloned2 = Arc::clone(&editor);
    let editor_cloned3 = Arc::clone(&editor);
    let gpu_cloned = Arc::clone(&gpu_helper);
    let viewport_cloned = Arc::clone(&viewport);
    let viewport_cloned2 = Arc::clone(&viewport);
    let state_cloned = Arc::clone(&editor_state);
    let state_cloned2 = Arc::clone(&editor_state);
    let state_cloned3 = Arc::clone(&editor_state);
    let state_cloned4 = Arc::clone(&editor_state);
    let state_cloned5 = Arc::clone(&editor_state);
    let state_cloned6 = Arc::clone(&editor_state);
    let state_cloned7 = Arc::clone(&editor_state);
    let state_cloned8 = Arc::clone(&editor_state);
    let viewport_cloned3 = Arc::clone(&viewport);

    // let sequences: RwSignal<im::Vector<Sequence>> = create_rw_signal(im::Vector::new());
    let sequences: RwSignal<im::Vector<String>> = create_rw_signal(im::Vector::new());
    let sequence_quick_access: RwSignal<HashMap<String, String>> = create_rw_signal(HashMap::new());
    let sequence_durations: RwSignal<HashMap<String, i32>> = create_rw_signal(HashMap::new());

    // let sequence_timeline_signal = create_rw_signal(TimelineState::new());
    let timeline_sequences: RwSignal<Vec<TimelineSequence>> = create_rw_signal(Vec::new());
    let dragging_timeline_sequence: RwSignal<Option<(String, i32)>> = create_rw_signal(None);
    let export_play_timeline_config: RwSignal<Option<SavedTimelineStateConfig>> =
        create_rw_signal(None);

    create_effect(move |_| {
        let mut editor_state = editor_state.lock().unwrap();
        let saved_state = editor_state
            .record_state
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

        let qa_sequences: HashMap<String, String> = saved_state
            .sequences
            .clone()
            .into_iter()
            .map(|s| {
                x = x + 1;
                (s.id, s.name)
            })
            .collect();

        let sequence_durs = saved_state
            .sequences
            .clone()
            .into_iter()
            .map(|s| (s.id, s.duration_ms))
            .collect();

        sequences.set(im_sequences);
        sequence_quick_access.set(qa_sequences);
        sequence_durations.set(sequence_durs);

        // initialize TimelineState based on stored config if exists or saved sequences if not
        if saved_state.timeline_state.timeline_sequences.len() > 0 {
            timeline_sequences.set(saved_state.timeline_state.timeline_sequences.clone());

            export_play_timeline_config.set(Some(SavedTimelineStateConfig {
                timeline_sequences: timeline_sequences.get(),
            }));

            // sequence_timeline_signal.set(new_timeline_state);
        } else {
            // let new_timeline_state = TimelineState::new();
            // let timeline_sequences: Vec<TimelineSequence> = saved_state
            //     .sequences
            //     .clone()
            //     .into_iter()
            //     .enumerate() // Add enumerate() to get the index
            //     .map(|(index, s)| TimelineSequence {
            //         id: s.id,
            //         track_type: TrackType::Video,
            //         duration_ms: 20000,
            //         start_time_ms: index as i32 * 20000, // Multiply index by 20000
            //     })
            //     .collect();

            // new_timeline_state
            //     .timeline_sequences
            //     .set(timeline_sequences);

            // sequence_timeline_signal.set(new_timeline_state);
        }
    });

    h_stack((
        v_stack((
            label(move || format!("Sequences")).style(|s| s.margin_bottom(10)),
            // double check this exports with the latest timeline
            export_widget(state_cloned6, viewport_cloned3, export_play_timeline_config),
            simple_button("New Sequence".to_string(), move |_| {
                println!("New Sequence...");

                let new_sequence_id = Uuid::new_v4().to_string();
                let new_sequence = Sequence {
                    id: new_sequence_id.clone(),
                    name: "New Sequence".to_string(),
                    background_fill: Some(BackgroundFill::Color([
                        wgpu_to_human(0.8) as i32,
                        wgpu_to_human(0.8) as i32,
                        wgpu_to_human(0.8) as i32,
                        1,
                    ])),
                    duration_ms: 20000,
                    active_polygons: Vec::new(),
                    polygon_motion_paths: Vec::new(),
                    active_text_items: Vec::new(),
                    active_image_items: Vec::new(),
                    active_video_items: Vec::new(),
                };

                let mut editor_state = state_cloned.lock().unwrap();
                let mut new_state = editor_state
                    .record_state
                    .saved_state
                    .as_mut()
                    .expect("Couldn't get Saved State")
                    .clone();
                new_state.sequences.push(new_sequence);

                editor_state.record_state.saved_state = Some(new_state.clone());

                save_saved_state_raw(new_state.clone());

                let mut x = 0;
                let qa_sequences: HashMap<String, String> = new_state
                    .sequences
                    .clone()
                    .into_iter()
                    .map(|s| {
                        x = x + 1;
                        (s.id, s.name)
                    })
                    .collect();

                sequence_quick_access.set(qa_sequences);

                sequences.update(|s| s.push_back(new_sequence_id.clone()));

                // EventPropagation::Continue
            })
            .style(|s| s.margin_bottom(5.0)),
            // simple_button("TMP: Import Sequences".to_string(), move |_| {
            //     println!("Import Sequences...");

            //     let imported_data =
            //         include_str!("D:/projects/common/common-motion-2d-reg/backup/augmented.txt");

            //     let animated_data =
            //         parse_animation_data(&imported_data).expect("Couldn't parse imported data");

            //     let animated_data = get_last_n_items(&animated_data, 300);

            //     let new_ids: Vec<String> = animated_data.iter().map(|s| s.id.clone()).collect();

            //     let mut editor_state = state_cloned8.lock().unwrap();
            //     let mut new_state = editor_state
            //         .saved_state
            //         .as_mut()
            //         .expect("Couldn't get Saved State")
            //         .clone();

            //     new_state.sequences = animated_data.to_vec();

            //     editor_state.saved_state = Some(new_state.clone());

            //     save_saved_state_raw(new_state.clone());

            //     let mut x = 0;
            //     let qa_sequences: HashMap<String, i32> = new_state
            //         .sequences
            //         .clone()
            //         .into_iter()
            //         .map(|s| {
            //             x = x + 1;
            //             (s.id, x)
            //         })
            //         .collect();

            //     sequence_quick_access.set(qa_sequences);

            //     // new_ids.iter().for_each(|id| {
            //     //     sequences.update(|s| s.push_back(id.clone()));
            //     // });

            //     sequences.set(im::Vector::from_iter(new_ids));

            //     // EventPropagation::Continue
            // }),
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
                        let item_cloned2 = item.clone();

                        let sequence_quick_access = sequence_quick_access.get();
                        let quick_access_info = sequence_quick_access
                            .get(&item)
                            .expect("Couldn't find matching qa info");

                        h_stack((
                            simple_button("Edit ".to_string() + &quick_access_info, move |_| {
                                println!("Open Sequence...");

                                let mut editor_state = state_cloned2.lock().unwrap();
                                let saved_state = editor_state
                                    .record_state
                                    .saved_state
                                    .as_ref()
                                    .expect("Couldn't get Saved State");

                                let saved_sequence = saved_state
                                    .sequences
                                    .iter()
                                    .find(|s| s.id == item.clone())
                                    .expect("Couldn't find matching sequence")
                                    .clone();

                                let mut background_fill = Some(BackgroundFill::Color([
                                    wgpu_to_human(0.8) as i32,
                                    wgpu_to_human(0.8) as i32,
                                    wgpu_to_human(0.8) as i32,
                                    255,
                                ]));

                                if saved_sequence.background_fill.is_some() {
                                    background_fill = saved_sequence.background_fill.clone();
                                }

                                // for the background polygon and its signal
                                editor_state.selected_polygon_id =
                                    Uuid::from_str(&saved_sequence.id)
                                        .expect("Couldn't convert string to uuid");

                                drop(editor_state);

                                println!("Opening Sequence...");

                                let mut editor = editor_cloned.lock().unwrap();

                                let camera = editor.camera.expect("Couldn't get camera");
                                let viewport = viewport_cloned.lock().unwrap();

                                let window_size = WindowSize {
                                    width: viewport.width as u32,
                                    height: viewport.height as u32,
                                };

                                let mut rng = rand::thread_rng();

                                // editor.polygons = Vec::new();
                                // editor.text_items = Vec::new();
                                // editor.image_items = Vec::new();

                                // editor.restore_sequence_objects(
                                //     &saved_sequence,
                                //     window_size,
                                //     &camera,
                                //     false,
                                // );

                                // set hidden to false based on sequence
                                // also reset all objects to hidden=true beforehand
                                editor.polygons.iter_mut().for_each(|p| {
                                    p.hidden = true;
                                });
                                editor.image_items.iter_mut().for_each(|i| {
                                    i.hidden = true;
                                });
                                editor.text_items.iter_mut().for_each(|t| {
                                    t.hidden = true;
                                });
                                editor.video_items.iter_mut().for_each(|t| {
                                    t.hidden = true;
                                });

                                saved_sequence.active_polygons.iter().for_each(|ap| {
                                    let polygon = editor
                                        .polygons
                                        .iter_mut()
                                        .find(|p| p.id.to_string() == ap.id)
                                        .expect("Couldn't find polygon");
                                    polygon.hidden = false;
                                });
                                saved_sequence.active_image_items.iter().for_each(|si| {
                                    let image = editor
                                        .image_items
                                        .iter_mut()
                                        .find(|i| i.id.to_string() == si.id)
                                        .expect("Couldn't find image");
                                    image.hidden = false;
                                });
                                saved_sequence.active_text_items.iter().for_each(|tr| {
                                    let text = editor
                                        .text_items
                                        .iter_mut()
                                        .find(|t| t.id.to_string() == tr.id)
                                        .expect("Couldn't find image");
                                    text.hidden = false;
                                });
                                saved_sequence.active_video_items.iter().for_each(|tr| {
                                    let video = editor
                                        .video_items
                                        .iter_mut()
                                        .find(|t| t.id.to_string() == tr.id)
                                        .expect("Couldn't find image");
                                    video.hidden = false;
                                });

                                match background_fill.expect("Couldn't get default background fill")
                                {
                                    BackgroundFill::Color(fill) => {
                                        editor.replace_background(
                                            Uuid::from_str(&saved_sequence.id)
                                                .expect("Couldn't convert string to uuid"),
                                            rgb_to_wgpu(
                                                fill[0] as u8,
                                                fill[1] as u8,
                                                fill[2] as u8,
                                                fill[3] as f32,
                                            ),
                                        );
                                    }
                                    _ => {
                                        println!("Not supported yet...");
                                    }
                                }

                                println!("Objects restored!");

                                editor.update_motion_paths(&saved_sequence);

                                println!("Motion Paths restored!");

                                drop(editor);

                                selected_sequence_data.set(saved_sequence.clone());
                                selected_sequence_id.set(item.clone());
                                sequence_selected.set(true);

                                // EventPropagation::Continue
                            })
                            .style(|s| s.margin_right(2.0)),
                            simple_button("Duplicate".to_string(), move |_| {
                                println!("Duplicating sequence...");

                                let mut editor_state = state_cloned3.lock().unwrap();
                                let mut new_state = editor_state
                                    .record_state
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

                                editor_state.record_state.saved_state = Some(new_state.clone());

                                save_saved_state_raw(new_state.clone());

                                sequences.update(|s| s.push_back(new_sequence_id.clone()));

                                println!("Sequence duplicated!");
                            })
                            .style(|s| s.margin_right(2.0)),
                            simple_button("Add to Timeline".to_string(), {
                                let item_cloned = item_cloned2.clone();

                                move |_| {
                                    println!("Adding sequence to sequence timeline...");

                                    let mut editor_state = state_cloned4.lock().unwrap();

                                    // let mut existing_timeline = editor_state
                                    //     .sequence_timeline_state
                                    //     .timeline_sequences
                                    //     .get();

                                    // let sequence_timeline_state = sequence_timeline_state;

                                    let mut existing_timeline = timeline_sequences.get();
                                    let sequence_durations = sequence_durations.get();

                                    // Find the sequence that ends at the latest point in time
                                    let start_time = if existing_timeline.is_empty() {
                                        0
                                    } else {
                                        existing_timeline
                                            .iter()
                                            .map(|seq| {
                                                let duration_ms = sequence_durations
                                                    .get(&seq.sequence_id)
                                                    .expect("Couldn't get duration");

                                                seq.start_time_ms + duration_ms
                                            })
                                            .max()
                                            .unwrap_or(0)
                                    };

                                    existing_timeline.push(TimelineSequence {
                                        id: Uuid::new_v4().to_string(),
                                        sequence_id: item_cloned.clone(),
                                        track_type: TrackType::Video,
                                        start_time_ms: start_time,
                                        // duration_ms: 20000,
                                    });

                                    timeline_sequences.set(existing_timeline);
                                    export_play_timeline_config.set(Some(
                                        SavedTimelineStateConfig {
                                            timeline_sequences: timeline_sequences.get(),
                                        },
                                    ));

                                    let new_savable = export_play_timeline_config
                                        .get()
                                        .expect("Couldn't get timeline config");

                                    let mut new_state = editor_state
                                        .record_state
                                        .saved_state
                                        .as_mut()
                                        .expect("Couldn't get Saved State")
                                        .clone();

                                    new_state.timeline_state = new_savable;

                                    editor_state.record_state.saved_state = Some(new_state.clone());

                                    save_saved_state_raw(new_state.clone());

                                    println!("Sequence added!");
                                }
                            }),
                        ))
                        .style(|s| s.margin_bottom(2.0))
                    },
                )
                .style(|s| {
                    s.flex_col().width(260.0)
                    // .padding_vert(15.0)
                    // .padding_horiz(20.0)
                    // .background(Color::LIGHT_BLUE)
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
                    .record_state
                    .saved_state
                    .as_ref()
                    .expect("Couldn't get saved state");
                let cloned_sequences = saved_state.sequences.clone();

                drop(editor_state);

                let mut editor = editor_cloned2.lock().unwrap();
                let viewport = viewport_cloned2.lock().unwrap();
                let camera = editor.camera.expect("Couldn't get camera");

                if editor.video_is_playing {
                    println!("Pause Video...");

                    editor.video_is_playing = false;
                    editor.video_start_playing_time = None;
                    editor.video_current_sequence_timeline = None;
                    editor.video_current_sequences_data = None;
                    editor.is_playing = false;
                    editor.start_playing_time = None;

                    // TODO: reset_sequence_objects?
                    editor.video_items.iter_mut().for_each(|v| {
                        v.reset_playback().expect("Couldn't reset video playback");
                    });
                } else {
                    println!("Play Video...");

                    // editor.polygons = Vec::new();
                    // editor.text_items = Vec::new();
                    // editor.image_items = Vec::new();

                    // cloned_sequences.iter().enumerate().for_each(|(i, s)| {
                    //     editor.restore_sequence_objects(
                    //         &s,
                    //         WindowSize {
                    //             width: viewport.width as u32,
                    //             height: viewport.height as u32,
                    //         },
                    //         &camera,
                    //         if i == 0 { false } else { true },
                    //     );
                    // });

                    // set hidden to false for first sequence
                    let mut first_sequence: Option<Sequence> = None;
                    cloned_sequences.iter().enumerate().for_each(|(i, s)| {
                        if i == 0 {
                            first_sequence = Some(s.clone());
                            s.active_polygons.iter().for_each(|ap| {
                                let polygon = editor
                                    .polygons
                                    .iter_mut()
                                    .find(|p| p.id.to_string() == ap.id)
                                    .expect("Couldn't find polygon");
                                polygon.hidden = false;
                            });
                            s.active_image_items.iter().for_each(|si| {
                                let image = editor
                                    .image_items
                                    .iter_mut()
                                    .find(|i| i.id.to_string() == si.id)
                                    .expect("Couldn't find image");
                                image.hidden = false;
                            });
                            s.active_text_items.iter().for_each(|tr| {
                                let text = editor
                                    .text_items
                                    .iter_mut()
                                    .find(|t| t.id.to_string() == tr.id)
                                    .expect("Couldn't find image");
                                text.hidden = false;
                            });
                            s.active_video_items.iter().for_each(|tr| {
                                let video = editor
                                    .video_items
                                    .iter_mut()
                                    .find(|t| t.id.to_string() == tr.id)
                                    .expect("Couldn't find image");
                                video.hidden = false;
                            });
                        } else {
                            s.active_polygons.iter().for_each(|ap| {
                                let polygon = editor
                                    .polygons
                                    .iter_mut()
                                    .find(|p| p.id.to_string() == ap.id)
                                    .expect("Couldn't find polygon");
                                polygon.hidden = true;
                            });
                            s.active_image_items.iter().for_each(|si| {
                                let image = editor
                                    .image_items
                                    .iter_mut()
                                    .find(|i| i.id.to_string() == si.id)
                                    .expect("Couldn't find image");
                                image.hidden = true;
                            });
                            s.active_text_items.iter().for_each(|tr| {
                                let text = editor
                                    .text_items
                                    .iter_mut()
                                    .find(|t| t.id.to_string() == tr.id)
                                    .expect("Couldn't find image");
                                text.hidden = true;
                            });
                            s.active_video_items.iter().for_each(|tr| {
                                let video = editor
                                    .video_items
                                    .iter_mut()
                                    .find(|t| t.id.to_string() == tr.id)
                                    .expect("Couldn't find image");
                                video.hidden = true;
                            });
                        }
                    });

                    println!("All sequence objects restored...");

                    editor.current_sequence_data =
                        Some(first_sequence.expect("Couldn't get first sequence"));

                    let now = std::time::Instant::now();
                    editor.video_start_playing_time = Some(now.clone());

                    editor.video_current_sequence_timeline = Some(
                        export_play_timeline_config
                            .get()
                            .expect("Couldn't get a timeline"),
                    );
                    editor.video_current_sequences_data = Some(cloned_sequences);

                    editor.video_is_playing = true;

                    // also set motion path playing
                    editor.start_playing_time = Some(now);
                    editor.is_playing = true;

                    println!("Video playing!");
                }

                // EventPropagation::Continue
            }),
            build_timeline(
                editor_cloned3,
                state_cloned7,
                timeline_sequences,
                dragging_timeline_sequence,
                export_play_timeline_config,
                10,
                sequence_quick_access,
                sequence_durations,
            ),
        ))
        .style(|s| s.margin_top(425.0).margin_left(25.0)),
    ))
}
