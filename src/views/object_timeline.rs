use cgmath::Vector2;
use floem::event::EventListener;
use floem::event::EventPropagation;
use floem::kurbo::Point as KurboPoint;
use floem::peniko::Color;
use floem::peniko::ColorStop;
use floem::peniko::ColorStops;
use floem::peniko::Extend;
use floem::peniko::Gradient;
use floem::peniko::GradientKind;
use floem::reactive::create_effect;
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
use serde::Deserialize;
use serde::Serialize;
use std::sync::Arc;
use std::sync::Mutex;
use stunts_engine::animations::AnimationData;
use stunts_engine::animations::ObjectType;
use stunts_engine::animations::Sequence;
use stunts_engine::editor::Editor;
use stunts_engine::editor::Point;
use stunts_engine::timelines::SavedTimelineStateConfig;
use stunts_engine::timelines::TimelineSequence;
use stunts_engine::timelines::TrackType;

use crate::editor_state::EditorState;
use crate::helpers::utilities::save_saved_state_raw;

pub fn build_object_timeline(
    editor: Arc<Mutex<Editor>>,
    editor_state: Arc<Mutex<EditorState>>,
    // timeline_animations: RwSignal<Vec<AnimationData>>,
    selected_sequence_data: RwSignal<Sequence>,
    deafult_pixels_per_s: i32,
) -> impl View {
    let editor_state2 = Arc::clone(&editor_state);
    let pixels_per_s = create_rw_signal(deafult_pixels_per_s);
    let timeline_width = create_rw_signal(700);

    create_effect(move |_| {
        // determine the pixels per s
        let sequence_data = selected_sequence_data.get();

        let total_s = sequence_data.duration_ms / 1000;
        let new_per_s = timeline_width.get() / total_s;

        pixels_per_s.set(new_per_s);
    });

    v_stack((
        // Tick marks for the timeline
        container(dyn_stack(
            move || {
                let total_s = selected_sequence_data.get().duration_ms / 1000;
                (0..=total_s).collect::<Vec<_>>() // Create a tick for each second
            },
            move |time| time.to_string(), // Unique identifier
            move |time| {
                let pixels_per_second = pixels_per_s.get();
                container((label(move || time.to_string())))
                    .style(move |s| {
                        s.width(1.0)
                            .height(20.0)
                            .background(Color::rgb8(50, 50, 50)) // Tick mark color
                            .position(Position::Absolute)
                            .margin_left((time * pixels_per_second) as f32)
                    })
                    .style(|s| s.margin_top(0.0))
            },
        ))
        .style(|s| s.height(20).position(Position::Relative)),
        // tracks
        dyn_container(
            move || selected_sequence_data.get(),
            move |data| {
                let editor = editor.clone();
                let editor_state = editor_state.clone();
                let timeline_width = timeline_width.clone();

                if data.id.len() > 0 && data.polygon_motion_paths.len() > 0 {
                    dyn_stack(
                        move || data.polygon_motion_paths.clone(),
                        move |timeline_animation| timeline_animation.id.clone(),
                        {
                            move |animation| {
                                container(stack((
                                    // background
                                    container((empty()))
                                        .style(move |s| {
                                            s.width(timeline_width.get())
                                                .height(50)
                                                .background(Color::rgb8(200, 150, 100))
                                                .z_index(1)
                                        })
                                        .style(|s| s.absolute().margin_left(0.0)),
                                    // timeline_sequences
                                    timeline_object_track(
                                        editor.clone(),
                                        editor_state.clone(),
                                        selected_sequence_data,
                                        pixels_per_s,
                                        animation,
                                    ),
                                )))
                                .style(|s| s.position(Position::Relative).height(50))
                            }
                        },
                    )
                    .style(|s| s.flex_col().gap(1.0))
                    .into_any()
                } else {
                    container((empty())).into_any()
                }
            },
        ),
    ))
}

pub fn timeline_object_track(
    editor: Arc<Mutex<Editor>>,
    editor_state: Arc<Mutex<EditorState>>,
    // timeline_animations: RwSignal<Vec<AnimationData>>,
    selected_sequence_data: RwSignal<Sequence>,
    pixels_per_s: RwSignal<i32>,
    animation: AnimationData,
) -> impl View {
    // let state_2 = state.clone();

    let dragger_id = create_rw_signal(String::new());

    let animation_id = animation.id.clone();
    let pixels_per_ms = pixels_per_s.get() as f32 / 1000.0;
    let left = animation.start_time_ms as f32 * pixels_per_ms;
    let left_signal = create_rw_signal(left);
    let width = animation.duration.as_millis() as f32 * pixels_per_ms;

    let sequence_data = selected_sequence_data.get();
    let related_object_id = animation.polygon_id;

    let small_label = match animation.object_type {
        ObjectType::Polygon => sequence_data
            .active_polygons
            .iter()
            .find(|p| p.id == related_object_id)
            .expect("Couldn't find polygon")
            .name
            .clone(),
        ObjectType::ImageItem => sequence_data
            .active_image_items
            .iter()
            .find(|p| p.id == related_object_id)
            .expect("Couldn't find image")
            .name
            .clone(),
        ObjectType::TextItem => sequence_data
            .active_text_items
            .iter()
            .find(|p| p.id == related_object_id)
            .expect("Couldn't find text item")
            .name
            .clone(),
    };

    // Linear gradient from left to right
    let gradient = Gradient {
        kind: GradientKind::Linear {
            start: KurboPoint::new(50.0, 0.0), // Start further left
            end: KurboPoint::new(200.0, 50.0), // End further right to allow more space
        },
        extend: Extend::Repeat,
        stops: ColorStops::from_vec(vec![
            ColorStop {
                offset: 0.5,
                color: Color::rgb8(153, 199, 162),
            },
            ColorStop {
                offset: 1.0,
                color: Color::rgb8(200, 204, 124),
            },
        ]),
    };

    container(
        container(label(move || small_label.clone()).style(|s| s.padding(5).selectable(false)))
            .on_event(EventListener::DragStart, {
                // let state = state.clone();

                move |evt| {
                    dragger_id.set(animation_id.clone());

                    EventPropagation::Continue
                }
            })
            .on_event(EventListener::DragEnd, {
                // let state = state.clone();
                let editor = editor.clone();
                let editor_state = editor_state.clone();

                move |evt| {
                    if let (id) = dragger_id.get() {
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

                        let mut anims: Vec<AnimationData> =
                            selected_sequence_data.get().polygon_motion_paths;

                        anims.iter_mut().for_each(|ad| {
                            if ad.id == id {
                                ad.start_time_ms = new_time_ms;
                            }
                        });

                        selected_sequence_data.update(|s| {
                            s.polygon_motion_paths = anims.clone();
                        });

                        // export_play_timeline_config.set(Some(SavedTimelineStateConfig {
                        //     timeline_sequences: timeline_sequences.get(),
                        // }));

                        left_signal.set(new_time_ms as f32 * pixels_per_ms);

                        // update the saved_state
                        let mut editor_state = editor_state.lock().unwrap();
                        let mut new_state = editor_state
                            .record_state
                            .saved_state
                            .as_mut()
                            .expect("Couldn't get Saved State")
                            .clone();

                        new_state.sequences.iter_mut().for_each(move |s| {
                            if s.id == selected_sequence_data.get().id {
                                s.polygon_motion_paths = anims.clone();
                            }
                        });

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
                    .height(40.0)
                    .selectable(false)
                    .border_radius(5.0)
                    // .cursor(CursorStyle::ColResize)
                    // .background(Color::rgb8(100, 200, 100))
                    .background(gradient.clone())
                    .cursor(CursorStyle::Pointer)
                    .z_index(5)
            })
            .draggable()
            .dragging_style(|s| {
                s.box_shadow_blur(3)
                    .box_shadow_color(Color::rgba(100.0, 100.0, 100.0, 0.5))
                    .box_shadow_spread(2)
                    .position(Position::Relative)
            })
            .into_view(),
    )
    .style(|s: floem::style::Style| s.display(Display::Block).padding(5))
    .style(|s| s.absolute().margin_left(0.0).height(50))
}
