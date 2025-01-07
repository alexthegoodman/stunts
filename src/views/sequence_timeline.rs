use cgmath::Vector2;
use floem::event::EventListener;
use floem::event::EventPropagation;
use floem::reactive::RwSignal;
use floem::reactive::SignalGet;
use floem::reactive::SignalUpdate;
use floem::style::CursorStyle;
use floem::views::*;
use floem::IntoView;
use floem::View;
use serde::Deserialize;
use serde::Serialize;
use std::sync::Arc;

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct TimelineSequence {
    pub id: String,
    pub track_type: TrackType,
    pub start_time_ms: i32, // in milliseconds
    pub duration_ms: i32,   // in milliseconds
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub enum TrackType {
    Audio,
    Video,
}

#[derive(Clone)]
pub struct TimelineState {
    pub timeline_sequences: RwSignal<Vec<TimelineSequence>>,
    pub dragging_timeline_sequence: RwSignal<Option<(String, i32)>>, // (id, original_start_time)
    pub pixels_per_ms: i32,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct SavedTimelineStateConfig {
    pub timeline_sequences: Vec<TimelineSequence>,
    // pub pixels_per_second: f32,
}

impl TimelineState {
    pub fn new() -> Self {
        Self {
            timeline_sequences: RwSignal::new(Vec::new()),
            dragging_timeline_sequence: RwSignal::new(None),
            pixels_per_ms: 1, // Adjust based on zoom level
        }
    }

    pub fn move_timeline_sequence(&self, id: &str, new_start_time: i32) {
        if let Some(seq) = self.timeline_sequences.get().iter().find(|s| s.id == id) {
            let old_start = seq.start_time_ms;
            // seq.start_time = new_start_time; // TODO: need to set?

            // Shift other timeline_sequences in the same track if needed
            let track_type = seq.track_type.clone();
            let mut timeline_sequences: Vec<TimelineSequence> = self.timeline_sequences.get();
            let mapped = timeline_sequences
                .iter()
                .filter(|s| {
                    s.track_type == track_type && s.id != id && s.start_time_ms >= old_start
                })
                .map(|s| {
                    let mut se = s.clone();
                    se.start_time_ms += new_start_time - old_start;
                    se
                })
                .collect();
            self.timeline_sequences.set(mapped);
        }
    }

    pub fn to_config(&self) -> SavedTimelineStateConfig {
        let existing_sequences = self.timeline_sequences.get();

        SavedTimelineStateConfig {
            timeline_sequences: existing_sequences,
        }
    }
}

pub fn build_timeline(state: TimelineState) -> impl View {
    // TODO: many tracks
    v_stack((
        // Audio track
        container(stack((
            // Background
            container((empty())).style(|s| s.width_full().height(50)),
            // TimelineSequences
            timeline_sequence_track(state.clone(), TrackType::Audio),
        ))),
        // Video track
        container(stack((
            // background
            container((empty())).style(|s| s.width_full().height(50)),
            // timeline_sequences
            timeline_sequence_track(state.clone(), TrackType::Video),
        ))),
    ))
}

pub fn timeline_sequence_track(state: TimelineState, track_type: TrackType) -> impl View {
    let state_2 = state.clone();

    dyn_stack(
        move || state_2.timeline_sequences.get(),
        move |timeline_sequence| timeline_sequence.id.clone(),
        {
            let state = state.clone();

            move |seq: TimelineSequence| {
                let seq_id = seq.id.clone();
                let left = seq.start_time_ms * state.pixels_per_ms;
                let width = seq.duration_ms * state.pixels_per_ms;

                if (seq.track_type != track_type) {
                    return container((empty())).into_view();
                }

                container(label(move || seq.id.clone()).style(|s| s.padding(5)))
                    .style(move |s| {
                        s.absolute()
                            .margin_left(left)
                            .width(width)
                            .height_full()
                            // .background(if track_type == TrackType::Audio {
                            //     rgb(0x4A90E2)
                            // } else {
                            //     rgb(0xE24A90)
                            // })
                            .cursor(CursorStyle::Pointer)
                    })
                    .on_event(EventListener::DragStart, {
                        let state = state.clone();

                        move |evt| {
                            state
                                .dragging_timeline_sequence
                                .set(Some((seq_id.clone(), left / state.pixels_per_ms)));
                            EventPropagation::Continue
                        }
                    })
                    .on_event(EventListener::DragEnd, {
                        let state = state.clone();

                        move |evt| {
                            if let Some((id, _)) = state.dragging_timeline_sequence.get().take() {
                                let scale_factor = 1.25; // hardcode test // TODO: fix
                                let position = Vector2::new(
                                    (evt.point().expect("Couldn't get point").x as f32
                                        / scale_factor) as i32,
                                    (evt.point().expect("Couldn't get point").y as f32
                                        / scale_factor) as i32,
                                );

                                let new_time = position.x / state.pixels_per_ms;
                                state.move_timeline_sequence(&id, new_time);
                            }
                            EventPropagation::Continue
                        }
                    })
                    .into_view()
            }
        },
    )
}
