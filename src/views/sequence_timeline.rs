use cgmath::Vector2;
use floem::event::EventListener;
use floem::event::EventPropagation;
use floem::peniko::Color;
use floem::reactive::create_rw_signal;
use floem::reactive::RwSignal;
use floem::reactive::SignalGet;
use floem::reactive::SignalUpdate;
use floem::style::CursorStyle;
use floem::taffy::Position;
use floem::views::*;
use floem::IntoView;
use floem::View;
use serde::Deserialize;
use serde::Serialize;
use std::sync::Arc;
use std::sync::Mutex;
use stunts_engine::timelines::SavedTimelineStateConfig;
use stunts_engine::timelines::TimelineSequence;
use stunts_engine::timelines::TrackType;

use crate::editor_state::EditorState;
use crate::helpers::utilities::save_saved_state_raw;

#[derive(Clone)]
pub struct TimelineState {
    pub timeline_sequences: RwSignal<Vec<TimelineSequence>>,
    pub dragging_timeline_sequence: RwSignal<Option<(String, i32)>>, // (id, original_start_time)
    pub pixels_per_s: i32,
}

impl TimelineState {
    pub fn new() -> Self {
        Self {
            timeline_sequences: RwSignal::new(Vec::new()),
            dragging_timeline_sequence: RwSignal::new(None),
            pixels_per_s: 10, // Adjust based on zoom level
        }
    }

    // pub fn move_timeline_sequence(&self, id: &str, new_start_time: i32) {
    //     if let Some(seq) = self.timeline_sequences.get().iter().find(|s| s.id == id) {
    //         let old_start = seq.start_time_ms;
    //         // seq.start_time = new_start_time; // TODO: need to set?

    //         // Shift other timeline_sequences in the same track if needed
    //         let track_type = seq.track_type.clone();
    //         let mut timeline_sequences: Vec<TimelineSequence> = self.timeline_sequences.get();
    //         let mapped = timeline_sequences
    //             .iter()
    //             .filter(|s| {
    //                 s.track_type == track_type && s.id != id && s.start_time_ms >= old_start
    //             })
    //             .map(|s| {
    //                 let mut se = s.clone();
    //                 se.start_time_ms += new_start_time - old_start;
    //                 se
    //             })
    //             .collect();
    //         self.timeline_sequences.set(mapped);
    //     }
    // }

    pub fn to_config(&self) -> SavedTimelineStateConfig {
        let existing_sequences = self.timeline_sequences.get();

        SavedTimelineStateConfig {
            timeline_sequences: existing_sequences,
        }
    }
}

pub fn build_timeline(editor_state: Arc<Mutex<EditorState>>, state: TimelineState) -> impl View {
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
            timeline_sequence_track(editor_state.clone(), state.clone(), TrackType::Audio),
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
            timeline_sequence_track(editor_state.clone(), state.clone(), TrackType::Video),
        )))
        .style(|s| s.position(Position::Relative).height(60)),
    ))
}

pub fn timeline_sequence_track(
    editor_state: Arc<Mutex<EditorState>>,
    state: TimelineState,
    track_type: TrackType,
) -> impl View {
    let state_2 = state.clone();

    let dragger_id = create_rw_signal(String::new());

    dyn_stack(
        move || state_2.timeline_sequences.get(),
        move |timeline_sequence| timeline_sequence.id.clone(),
        {
            let state = state.clone();
            let track_type = track_type.clone();

            move |seq: TimelineSequence| {
                let seq_id = seq.id.clone();
                let track_type = track_type.clone();
                let pixels_per_ms = state.pixels_per_s as f32 / 1000.0;
                let left = seq.start_time_ms as f32 * pixels_per_ms;
                let width = seq.duration_ms as f32 * pixels_per_ms;

                if (seq.track_type != track_type) {
                    return container((empty())).into_view();
                }

                let small_labels: Vec<String> = seq
                    .sequence_id
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

                container(label(move || small_label.clone()).style(|s| s.padding(5)))
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
                    // .on_event(EventListener::DragStart, {
                    //     let state = state.clone();
                    //     move |evt| {
                    //         state
                    //             .dragging_timeline_sequence
                    //             .set(Some((seq_id.clone(), left / state.pixels_per_ms)));
                    //         EventPropagation::Continue
                    //     }
                    // })
                    // .on_event(EventListener::DragEnd, {
                    //     let state = state.clone();
                    //     move |evt| {
                    //         if let Some((id, _)) = state.dragging_timeline_sequence.get().take() {
                    //             let scale_factor = 1.25; // hardcode test // TODO: fix
                    //             let position = Vector2::new(
                    //                 (evt.point().expect("Couldn't get point").x as f32
                    //                     / scale_factor) as i32,
                    //                 (evt.point().expect("Couldn't get point").y as f32
                    //                     / scale_factor) as i32,
                    //             );
                    //             let new_time = position.x / state.pixels_per_ms;
                    //             state.move_timeline_sequence(&id, new_time);
                    //         }
                    //         EventPropagation::Continue
                    //     }
                    // })
                    .style(move |s| {
                        s.width(width)
                            .height_full()
                            .selectable(false)
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
                    .on_event(floem::event::EventListener::DragStart, move |_| {
                        dragger_id.set(seq.id.clone());
                        floem::event::EventPropagation::Continue
                    })
                    .on_event(floem::event::EventListener::DragOver, {
                        let editor_state = editor_state.clone();

                        move |_| {
                            // let mut editor = editor.lock().unwrap();
                            let dragger_id = dragger_id.get_untracked();
                            let sortable_items = state.timeline_sequences;
                            if dragger_id != seq_id.clone() {
                                let dragger_pos = sortable_items
                                    .get()
                                    .iter()
                                    .position(|layer| layer.id == dragger_id)
                                    .or_else(|| Some(usize::MAX))
                                    .expect("Couldn't get dragger_pos");
                                let hover_pos = sortable_items
                                    .get()
                                    .iter()
                                    .position(|layer| layer.id == seq_id.clone())
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

                                // update the saved_state
                                let mut editor_state = editor_state.lock().unwrap();
                                let mut new_state = editor_state
                                    .record_state
                                    .saved_state
                                    .as_mut()
                                    .expect("Couldn't get Saved State")
                                    .clone();
                                new_state.timeline_state.timeline_sequences = sortable_items.get();

                                editor_state.record_state.saved_state = Some(new_state.clone());

                                save_saved_state_raw(new_state.clone());
                            }
                            floem::event::EventPropagation::Continue
                        }
                    })
                    .dragging_style(|s| {
                        s.box_shadow_blur(3)
                            .box_shadow_color(Color::rgba(100.0, 100.0, 100.0, 0.5))
                            .box_shadow_spread(2)
                    })
                    .into_view()
            }
        },
    )
    .style(|s: floem::style::Style| s.flex_row().row_gap(5).padding(10))
    .style(|s| s.absolute().margin_left(0.0))
}
