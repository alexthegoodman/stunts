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
use std::sync::Arc;

#[derive(Clone, Debug)]
struct Sequence {
    id: String,
    track_type: TrackType,
    start_time: f32, // in seconds
    duration: f32,   // in seconds
                     // Add other sequence properties here
}

#[derive(Clone, Debug, PartialEq)]
enum TrackType {
    Audio,
    Video,
}

struct TimelineState {
    sequences: RwSignal<Vec<Sequence>>,
    dragging_sequence: RwSignal<Option<(String, f32)>>, // (id, original_start_time)
    pixels_per_second: f32,
}

impl TimelineState {
    fn new() -> Self {
        Self {
            sequences: RwSignal::new(Vec::new()),
            dragging_sequence: RwSignal::new(None),
            pixels_per_second: 100.0, // Adjust based on zoom level
        }
    }

    fn move_sequence(&self, id: &str, new_start_time: f32) {
        if let Some(seq) = self.sequences.get().iter().find(|s| s.id == id) {
            let old_start = seq.start_time;
            // seq.start_time = new_start_time; // TODO: need to set?

            // Shift other sequences in the same track if needed
            let track_type = seq.track_type.clone();
            let mut sequences: Vec<Sequence> = self.sequences.get();
            let mapped = sequences
                .iter()
                .filter(|s| s.track_type == track_type && s.id != id && s.start_time >= old_start)
                .map(|s| {
                    let mut se = s.clone();
                    se.start_time += new_start_time - old_start;
                    se
                })
                .collect();
            self.sequences.set(mapped);
        }
    }
}

fn build_timeline(state: Arc<TimelineState>) -> impl View {
    // TODO: many tracks
    v_stack((
        // Audio track
        container(stack((
            // Background
            container((empty())).style(|s| s.width_full().height(50)),
            // Sequences
            sequence_track(state.clone(), TrackType::Audio),
        ))),
        // Video track
        container(stack((
            // background
            container((empty())).style(|s| s.width_full().height(50)),
            // sequences
            sequence_track(state.clone(), TrackType::Video),
        ))),
    ))
}

fn sequence_track(state: Arc<TimelineState>, track_type: TrackType) -> impl View {
    let state_2 = state.clone();

    dyn_stack(
        move || state_2.sequences.get(),
        move |sequence| sequence.id.clone(),
        {
            let state = state.clone();

            move |seq: Sequence| {
                let seq_id = seq.id.clone();
                let left = seq.start_time * state.pixels_per_second;
                let width = seq.duration * state.pixels_per_second;

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
                                .dragging_sequence
                                .set(Some((seq_id.clone(), left / state.pixels_per_second)));
                            EventPropagation::Continue
                        }
                    })
                    .on_event(EventListener::DragEnd, {
                        let state = state.clone();

                        move |evt| {
                            if let Some((id, _)) = state.dragging_sequence.get().take() {
                                let scale_factor = 1.25; // hardcode test // TODO: fix
                                let position = Vector2::new(
                                    evt.point().expect("Couldn't get point").x as f32
                                        / scale_factor,
                                    evt.point().expect("Couldn't get point").y as f32
                                        / scale_factor,
                                );

                                let new_time = position.x / state.pixels_per_second;
                                state.move_sequence(&id, new_time);
                            }
                            EventPropagation::Continue
                        }
                    })
                    .into_view()
            }
        },
    )
}
