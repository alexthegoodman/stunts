use cgmath::Vector2;
use floem::event::EventListener;
use floem::event::EventPropagation;
use floem::peniko::Color;
use floem::reactive::create_rw_signal;
use floem::reactive::RwSignal;
use floem::reactive::SignalGet;
use floem::reactive::SignalTrack;
use floem::reactive::SignalUpdate;
use floem::style::CursorStyle;
use floem::taffy::Display;
use floem::taffy::Position;
use floem::views::*;
use floem::IntoView;
use floem::View;
use im::HashMap;
use serde::Deserialize;
use serde::Serialize;
use std::sync::Arc;
use std::sync::Mutex;
use stunts_engine::editor::Editor;
use stunts_engine::editor::Point;
use stunts_engine::timelines::SavedTimelineStateConfig;
use stunts_engine::timelines::TimelineSequence;
use stunts_engine::timelines::TrackType;

use crate::editor_state::EditorState;
use crate::helpers::utilities::save_saved_state_raw;

// #[derive(Clone)]
// pub struct TimelineState {
//     pub timeline_sequences: RwSignal<Vec<TimelineSequence>>,
//     pub dragging_timeline_sequence: RwSignal<Option<(String, i32)>>, // (id, original_start_time)
//     pub pixels_per_s: i32,
// }

// impl TimelineState {
//     pub fn new() -> Self {
//         Self {
//             timeline_sequences: RwSignal::new(Vec::new()),
//             dragging_timeline_sequence: RwSignal::new(None),
//             pixels_per_s: 10, // Adjust based on zoom level
//         }
//     }

//     // pub fn move_timeline_sequence(&self, id: &str, new_start_time: i32) {
//     //     if let Some(seq) = self.timeline_sequences.get().iter().find(|s| s.id == id) {
//     //         let old_start = seq.start_time_ms;

//     //         let mut seq = seq.clone();
//     //         seq.start_time_ms = new_start_time; // need to set?

//     //         // Shift other timeline_sequences in the same track if needed
//     //         let track_type = seq.track_type.clone();
//     //         let mut timeline_sequences: Vec<TimelineSequence> = self.timeline_sequences.get();
//     //         let mut mapped: Vec<TimelineSequence> = timeline_sequences
//     //             .iter()
//     //             .filter(|s| s.track_type == track_type && s.id != id)
//     //             .map(|s| {
//     //                 let mut se = s.clone();
//     //                 if se.start_time_ms >= old_start {
//     //                     se.start_time_ms += new_start_time - old_start;
//     //                 }
//     //                 se
//     //             })
//     //             .collect();

//     //         mapped.push(seq); // add current since skipped in fliter

//     //         self.timeline_sequences.set(mapped);
//     //     }
//     // }

//     pub fn move_timeline_sequence(&self, id: &str, new_start_time: i32) {
//         // Does not shift other sequences
//         let mut timeline_sequences: Vec<TimelineSequence> = self.timeline_sequences.get();

//         timeline_sequences.iter_mut().for_each(|ts| {
//             if ts.id == id {
//                 ts.start_time_ms = new_start_time;
//             }
//         });

//         self.timeline_sequences.set(timeline_sequences);
//     }

//     pub fn to_config(&self) -> SavedTimelineStateConfig {
//         let existing_sequences = self.timeline_sequences.get();

//         SavedTimelineStateConfig {
//             timeline_sequences: existing_sequences,
//         }
//     }
// }

pub fn build_timeline(
    editor: Arc<Mutex<Editor>>,
    editor_state: Arc<Mutex<EditorState>>,
    // state: RwSignal<TimelineState>,
    timeline_sequences: RwSignal<Vec<TimelineSequence>>,
    dragging_timeline_sequence: RwSignal<Option<(String, i32)>>,
    export_play_timeline_config: RwSignal<Option<SavedTimelineStateConfig>>,
    pixels_per_s: i32,
    sequence_quick_access: RwSignal<HashMap<String, String>>,
) -> impl View {
    // TODO: many tracks
    v_stack((
        // Audio track
        container(stack((
            // Background
            container((empty()))
                .style(|s| {
                    s.width(700.0)
                        .height(60)
                        .background(Color::rgb8(100, 150, 200))
                        .z_index(1)
                })
                .style(|s| s.absolute().margin_left(0.0)),
            // TimelineSequences
            timeline_sequence_track(
                editor.clone(),
                editor_state.clone(),
                // state,
                timeline_sequences,
                dragging_timeline_sequence,
                export_play_timeline_config,
                TrackType::Audio,
                pixels_per_s,
                sequence_quick_access,
            ),
        )))
        .style(|s| s.position(Position::Relative).height(60)),
        // Video track
        container(stack((
            // background
            container((empty()))
                .style(|s| {
                    s.width(700.0)
                        .height(60)
                        .background(Color::rgb8(200, 150, 100))
                        .z_index(1)
                })
                .style(|s| s.absolute().margin_left(0.0)),
            // timeline_sequences
            timeline_sequence_track(
                editor.clone(),
                editor_state.clone(),
                // state,
                timeline_sequences,
                dragging_timeline_sequence,
                export_play_timeline_config,
                TrackType::Video,
                pixels_per_s,
                sequence_quick_access,
            ),
        )))
        .style(|s| s.position(Position::Relative).height(60)),
    ))
}

pub fn timeline_sequence_track(
    editor: Arc<Mutex<Editor>>,
    editor_state: Arc<Mutex<EditorState>>,
    timeline_sequences: RwSignal<Vec<TimelineSequence>>,
    dragging_timeline_sequence: RwSignal<Option<(String, i32)>>,
    export_play_timeline_config: RwSignal<Option<SavedTimelineStateConfig>>,
    // state: RwSignal<TimelineState>,
    track_type: TrackType,
    pixels_per_s: i32,
    sequence_quick_access: RwSignal<HashMap<String, String>>,
) -> impl View {
    // let state_2 = state.clone();

    let dragger_id = create_rw_signal(String::new());

    dyn_stack(
        move || timeline_sequences.get(),
        move |timeline_sequence| timeline_sequence.id.clone(),
        {
            // let state = state.clone();
            let track_type = track_type.clone();

            move |seq: TimelineSequence| {
                let seq_id = seq.id.clone();
                let track_type = track_type.clone();
                let pixels_per_ms = pixels_per_s as f32 / 1000.0;
                let left = seq.start_time_ms as f32 * pixels_per_ms;
                let left_signal = create_rw_signal(left);
                // println!("seq {:?} {:?}", seq_id, left);
                let width = seq.duration_ms as f32 * pixels_per_ms;

                if (seq.track_type != track_type) {
                    return container((empty())).into_view();
                }

                let sequence_quick_access = sequence_quick_access.get();
                let quick_access_info = sequence_quick_access
                    // .clone()
                    .get(&seq.sequence_id)
                    .expect("Couldn't find matching qa info")
                    .clone();

                // drop(sequence_quick_access);

                container(
                    label(move || quick_access_info.clone())
                        .style(|s| s.padding(5).selectable(false)),
                )
                // .style(move |s| {
                //     s.absolute()
                //         .margin_left(left)
                //         .width(width)
                //         .height_full()
                //         // .background(if track_type == TrackType::Audio {
                //         //     rgb(0x4A90E2)
                //         // } else {
                //         //     rgb(0xE24A90)
                //         // })
                //         .cursor(CursorStyle::Pointer)
                //         .z_index(5)
                // })
                .on_event(EventListener::DragStart, {
                    // let state = state.clone();

                    move |evt| {
                        dragging_timeline_sequence
                            .set(Some((seq_id.clone(), (left / pixels_per_ms) as i32)));

                        EventPropagation::Continue
                    }
                })
                .on_event(EventListener::DragEnd, {
                    // let state = state.clone();
                    let editor = editor.clone();
                    let editor_state = editor_state.clone();

                    move |evt| {
                        if let Some((id, _)) = dragging_timeline_sequence.get().take() {
                            // relative to what?
                            // let scale_factor = 1.25; // hardcode test // TODO: fix
                            // let evt_point = evt.point().expect("Couldn't get point");
                            // let position = Vector2::new(
                            //     (evt_point.x as f32 / scale_factor) as i32,
                            //     (evt_point.y as f32 / scale_factor) as i32,
                            // );

                            // more definitive
                            let editor = editor.lock().unwrap();
                            // let camera = editor.camera.expect("Couldn't get camera");

                            let position = Point {
                                x: editor.last_screen.x - 600.0, // 600.0 for sidebar
                                y: editor.last_screen.y - 400.0, // 400.0 for size of canvas
                            };

                            println!("drag_end {:?}", position);

                            let mut new_time_ms = 0;
                            if position.x != 0.0 {
                                new_time_ms = (position.x / pixels_per_ms) as i32;
                            }

                            drop(editor);

                            // state.get().move_timeline_sequence(&id, new_time_ms);

                            let mut seqs: Vec<TimelineSequence> = timeline_sequences.get();

                            seqs.iter_mut().for_each(|ts| {
                                if ts.id == id {
                                    ts.start_time_ms = new_time_ms;
                                }
                            });

                            timeline_sequences.set(seqs);

                            // timeline_sequences.update(|seqs| {
                            //     seqs.iter_mut().for_each(|ts| {
                            //         if ts.id == id {
                            //             ts.start_time_ms = new_time_ms;
                            //         }
                            //     })
                            // });
                            export_play_timeline_config.set(Some(SavedTimelineStateConfig {
                                timeline_sequences: timeline_sequences.get(),
                            }));

                            left_signal.set(new_time_ms as f32 * pixels_per_ms);

                            // update the saved_state
                            let mut editor_state = editor_state.lock().unwrap();
                            let mut new_state = editor_state
                                .record_state
                                .saved_state
                                .as_mut()
                                .expect("Couldn't get Saved State")
                                .clone();

                            // need to update start_times for sequence items
                            // timeline_sequences (sortable_items) gets set in move_timeline_sequence
                            new_state.timeline_state.timeline_sequences = timeline_sequences.get();

                            editor_state.record_state.saved_state = Some(new_state.clone());

                            save_saved_state_raw(new_state.clone());
                        }
                        EventPropagation::Continue
                    }
                })
                .style(move |s| {
                    s.absolute()
                        .inset_left(left_signal.get())
                        .width(width)
                        // .height_full()
                        .height(40.0)
                        .selectable(false)
                        .border_radius(5.0)
                        // .cursor(CursorStyle::ColResize)
                        .background(if track_type == TrackType::Audio {
                            Color::rgb8(200, 100, 100)
                        } else {
                            Color::rgb8(100, 200, 100)
                        })
                        .cursor(CursorStyle::Pointer)
                        .z_index(5)
                })
                .draggable()
                // .on_event(floem::event::EventListener::DragStart, move |_| {
                //     dragger_id.set(seq.id.clone());
                //     floem::event::EventPropagation::Continue
                // })
                // .on_event(floem::event::EventListener::DragOver, {
                //     let editor_state = editor_state.clone();
                //     move |_| {
                //         // let mut editor = editor.lock().unwrap();
                //         let dragger_id = dragger_id.get_untracked();
                //         let sortable_items = state.timeline_sequences;
                //         if dragger_id != seq_id.clone() {
                //             let dragger_pos = sortable_items
                //                 .get()
                //                 .iter()
                //                 .position(|layer| layer.id == dragger_id)
                //                 .or_else(|| Some(usize::MAX))
                //                 .expect("Couldn't get dragger_pos");
                //             let hover_pos = sortable_items
                //                 .get()
                //                 .iter()
                //                 .position(|layer| layer.id == seq_id.clone())
                //                 .or_else(|| Some(usize::MAX))
                //                 .expect("Couldn't get hover_pos");
                //             sortable_items.update(|items| {
                //                 if (dragger_pos <= items.len() && hover_pos <= items.len()) {
                //                     let item = items.get(dragger_pos).cloned();
                //                     items.remove(dragger_pos);
                //                     // editor.layer_list.remove(dragger_pos);
                //                     if let Some(selected_item) = item {
                //                         items.insert(hover_pos, selected_item.clone());
                //                         // editor
                //                         //     .layer_list
                //                         //     .insert(hover_pos, selected_item.instance_id);
                //                     }
                //                 }
                //             });
                //             // update the saved_state
                //             let mut editor_state = editor_state.lock().unwrap();
                //             let mut new_state = editor_state
                //                 .record_state
                //                 .saved_state
                //                 .as_mut()
                //                 .expect("Couldn't get Saved State")
                //                 .clone();
                //             // TODO: need to update start_times for sequence items
                //             // set sortable_items too
                //             editor_state.record_state.saved_state = Some(new_state.clone());
                //             save_saved_state_raw(new_state.clone());
                //         }
                //         floem::event::EventPropagation::Continue
                //     }
                // })
                .dragging_style(|s| {
                    s.box_shadow_blur(3)
                        .box_shadow_color(Color::rgba(100.0, 100.0, 100.0, 0.5))
                        .box_shadow_spread(2)
                        .position(Position::Relative)
                })
                .into_view()
            }
        },
    )
    .style(|s: floem::style::Style| s.display(Display::Block).padding(10))
    .style(|s| s.absolute().margin_left(0.0).height(60))
}
