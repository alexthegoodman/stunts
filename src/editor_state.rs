use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

// use common_vector::basic::wgpu_to_human;
// use common_vector::editor::{InputValue, ObjectProperty};
// use common_vector::{basic::string_to_f32, editor::Editor};
use floem::keyboard::ModifiersState;
use floem::reactive::{RwSignal, SignalUpdate};
use stunts_engine::animations::{
    AnimationData, AnimationProperty, EasingType, KeyframeValue, ObjectType, UIKeyframe,
};
use stunts_engine::editor::{
    color_to_wgpu, string_to_f32, wgpu_to_human, Editor, InputValue, ObjectProperty,
};
use stunts_engine::polygon::SavedPolygonConfig;
use stunts_engine::st_image::SavedStImageConfig;
use stunts_engine::text_due::SavedTextRendererConfig;
use undo::Edit;
use undo::Record;
use uuid::Uuid;

use crate::helpers::saved_state::SavedState;
use crate::helpers::utilities::save_saved_state_raw;

#[derive(Debug)]
pub struct ObjectEdit {
    pub object_id: Uuid,
    pub object_type: ObjectType,
    pub field_name: String,
    pub old_value: ObjectProperty,
    pub new_value: ObjectProperty,
    pub signal: Option<RwSignal<String>>,
}

impl Edit for ObjectEdit {
    type Target = RecordState;
    type Output = ();

    fn edit(&mut self, record_state: &mut RecordState) {
        let mut editor = record_state.editor.lock().unwrap();
        let mut saved_state = record_state
            .saved_state
            .as_mut()
            .expect("Couldn't get saved state");

        match &self.new_value {
            ObjectProperty::Width(w) => {
                editor.update_polygon(self.object_id, "width", InputValue::Number(*w));

                let mut width = w.to_string();
                self.signal.expect("signal error").set(width);

                saved_state.sequences.iter_mut().for_each(|s| {
                    // if s.id == selected_sequence_id.get() { // would be more efficient for many sequences
                    s.active_polygons.iter_mut().for_each(|p| {
                        if p.id == self.object_id.to_string() {
                            p.dimensions = (*w as i32, p.dimensions.1);
                        }
                    });
                    // }
                });

                save_saved_state_raw(saved_state.clone());
            }
            ObjectProperty::Height(h) => {
                editor.update_polygon(self.object_id, "height", InputValue::Number(*h));

                let mut height = h.to_string();
                self.signal.expect("signal error").set(height);

                saved_state.sequences.iter_mut().for_each(|s| {
                    // if s.id == selected_sequence_id.get() {
                    s.active_polygons.iter_mut().for_each(|p| {
                        if p.id == self.object_id.to_string() {
                            p.dimensions = (p.dimensions.0, *h as i32);
                        }
                    });
                    // }
                });

                save_saved_state_raw(saved_state.clone());
            }
            ObjectProperty::Red(h) => {
                editor.update_polygon(self.object_id, "red", InputValue::Number(*h));

                let mut red = h.to_string();
                self.signal.expect("signal error").set(red);

                saved_state.sequences.iter_mut().for_each(|s| {
                    // if s.id == selected_sequence_id.get() {
                    s.active_polygons.iter_mut().for_each(|p| {
                        if p.id == self.object_id.to_string() {
                            p.fill[0] = color_to_wgpu(*h) as i32;
                        }
                    });
                    // }
                });

                save_saved_state_raw(saved_state.clone());
            }
            ObjectProperty::Green(h) => {
                editor.update_polygon(self.object_id, "green", InputValue::Number(*h));

                let mut green = h.to_string();
                self.signal.expect("signal error").set(green);

                saved_state.sequences.iter_mut().for_each(|s| {
                    // if s.id == selected_sequence_id.get() {
                    s.active_polygons.iter_mut().for_each(|p| {
                        if p.id == self.object_id.to_string() {
                            p.fill[1] = color_to_wgpu(*h) as i32;
                        }
                    });
                    // }
                });

                save_saved_state_raw(saved_state.clone());
            }
            ObjectProperty::Blue(h) => {
                editor.update_polygon(self.object_id, "blue", InputValue::Number(*h));

                let mut blue = h.to_string();
                self.signal.expect("signal error").set(blue);

                saved_state.sequences.iter_mut().for_each(|s| {
                    // if s.id == selected_sequence_id.get() {
                    s.active_polygons.iter_mut().for_each(|p| {
                        if p.id == self.object_id.to_string() {
                            p.fill[2] = color_to_wgpu(*h) as i32;
                        }
                    });
                    // }
                });

                save_saved_state_raw(saved_state.clone());
            }
            ObjectProperty::BorderRadius(h) => {
                editor.update_polygon(self.object_id, "border_radius", InputValue::Number(*h));

                let mut border_radius = h.to_string();
                self.signal.expect("signal error").set(border_radius);

                saved_state.sequences.iter_mut().for_each(|s| {
                    // if s.id == selected_sequence_id.get() {
                    s.active_polygons.iter_mut().for_each(|p| {
                        if p.id == self.object_id.to_string() {
                            p.border_radius = *h as i32;
                        }
                    });
                    // }
                });

                save_saved_state_raw(saved_state.clone());
            }
            ObjectProperty::StrokeThickness(h) => {
                editor.update_polygon(self.object_id, "stroke_thickness", InputValue::Number(*h));

                let mut stroke_thickness = h.to_string();
                self.signal.expect("signal error").set(stroke_thickness);

                saved_state.sequences.iter_mut().for_each(|s| {
                    // if s.id == selected_sequence_id.get() {
                    s.active_polygons.iter_mut().for_each(|p| {
                        if p.id == self.object_id.to_string() {
                            p.stroke.thickness = *h as i32;
                        }
                    });
                    // }
                });

                save_saved_state_raw(saved_state.clone());
            }
            ObjectProperty::StrokeRed(h) => {
                editor.update_polygon(self.object_id, "stroke_red", InputValue::Number(*h));

                let mut stroke_red = h.to_string();
                self.signal.expect("signal error").set(stroke_red);

                saved_state.sequences.iter_mut().for_each(|s| {
                    // if s.id == selected_sequence_id.get() {
                    s.active_polygons.iter_mut().for_each(|p| {
                        if p.id == self.object_id.to_string() {
                            p.stroke.fill[0] = color_to_wgpu(*h) as i32;
                        }
                    });
                    // }
                });

                save_saved_state_raw(saved_state.clone());
            }
            ObjectProperty::StrokeGreen(h) => {
                editor.update_polygon(self.object_id, "stroke_green", InputValue::Number(*h));

                let mut stroke_green = h.to_string();
                self.signal.expect("signal error").set(stroke_green);

                saved_state.sequences.iter_mut().for_each(|s| {
                    // if s.id == selected_sequence_id.get() {
                    s.active_polygons.iter_mut().for_each(|p| {
                        if p.id == self.object_id.to_string() {
                            p.stroke.fill[1] = color_to_wgpu(*h) as i32;
                        }
                    });
                    // }
                });

                save_saved_state_raw(saved_state.clone());
            }
            ObjectProperty::StrokeBlue(h) => {
                editor.update_polygon(self.object_id, "stroke_blue", InputValue::Number(*h));

                let mut stroke_blue = h.to_string();
                self.signal.expect("signal error").set(stroke_blue);

                saved_state.sequences.iter_mut().for_each(|s| {
                    // if s.id == selected_sequence_id.get() {
                    s.active_polygons.iter_mut().for_each(|p| {
                        if p.id == self.object_id.to_string() {
                            p.stroke.fill[2] = color_to_wgpu(*h) as i32;
                        }
                    });
                    // }
                });

                save_saved_state_raw(saved_state.clone());
            } // ObjectProperty::Points(w) => {
              //     editor.update_polygon(self.object_id, "points", InputValue::Points(w.clone()));
              // }
        }
    }

    fn undo(&mut self, record_state: &mut RecordState) {
        let mut editor = record_state.editor.lock().unwrap();
        let mut saved_state = record_state
            .saved_state
            .as_mut()
            .expect("Couldn't get saved state");

        match &self.old_value {
            ObjectProperty::Width(w) => {
                editor.update_polygon(self.object_id, "width", InputValue::Number(*w));

                let mut width = w.to_string();
                self.signal.expect("signal error").set(width);

                saved_state.sequences.iter_mut().for_each(|s| {
                    // if s.id == selected_sequence_id.get() { // would be more efficient for many sequences
                    s.active_polygons.iter_mut().for_each(|p| {
                        if p.id == self.object_id.to_string() {
                            p.dimensions = (*w as i32, p.dimensions.1);
                        }
                    });
                    // }
                });

                save_saved_state_raw(saved_state.clone());
            }
            ObjectProperty::Height(h) => {
                editor.update_polygon(self.object_id, "height", InputValue::Number(*h));

                let mut height = h.to_string();
                self.signal.expect("signal error").set(height);

                saved_state.sequences.iter_mut().for_each(|s| {
                    // if s.id == selected_sequence_id.get() {
                    s.active_polygons.iter_mut().for_each(|p| {
                        if p.id == self.object_id.to_string() {
                            p.dimensions = (p.dimensions.0, *h as i32);
                        }
                    });
                    // }
                });

                save_saved_state_raw(saved_state.clone());
            }
            ObjectProperty::Red(h) => {
                // let mut stroke_green = h.to_string();
                let red_human = wgpu_to_human(*h);

                editor.update_polygon(self.object_id, "red", InputValue::Number(red_human));

                self.signal
                    .expect("signal error")
                    .set(red_human.to_string());

                saved_state.sequences.iter_mut().for_each(|s| {
                    // if s.id == selected_sequence_id.get() {
                    s.active_polygons.iter_mut().for_each(|p| {
                        if p.id == self.object_id.to_string() {
                            p.fill[0] = color_to_wgpu(*h) as i32;
                        }
                    });
                    // }
                });

                save_saved_state_raw(saved_state.clone());
            }
            ObjectProperty::Green(h) => {
                // let mut stroke_green = h.to_string();
                let green_human = wgpu_to_human(*h);

                editor.update_polygon(self.object_id, "green", InputValue::Number(green_human));

                self.signal
                    .expect("signal error")
                    .set(green_human.to_string());

                saved_state.sequences.iter_mut().for_each(|s| {
                    // if s.id == selected_sequence_id.get() {
                    s.active_polygons.iter_mut().for_each(|p| {
                        if p.id == self.object_id.to_string() {
                            p.fill[1] = color_to_wgpu(*h) as i32;
                        }
                    });
                    // }
                });

                save_saved_state_raw(saved_state.clone());
            }
            ObjectProperty::Blue(h) => {
                // let mut stroke_green = h.to_string();
                let blue_human = wgpu_to_human(*h);

                editor.update_polygon(self.object_id, "blue", InputValue::Number(blue_human));

                self.signal
                    .expect("signal error")
                    .set(blue_human.to_string());

                saved_state.sequences.iter_mut().for_each(|s| {
                    // if s.id == selected_sequence_id.get() {
                    s.active_polygons.iter_mut().for_each(|p| {
                        if p.id == self.object_id.to_string() {
                            p.fill[2] = color_to_wgpu(*h) as i32;
                        }
                    });
                    // }
                });

                save_saved_state_raw(saved_state.clone());
            }
            ObjectProperty::BorderRadius(h) => {
                editor.update_polygon(self.object_id, "border_radius", InputValue::Number(*h));

                let mut border_radius = h.to_string();
                self.signal.expect("signal error").set(border_radius);

                saved_state.sequences.iter_mut().for_each(|s| {
                    // if s.id == selected_sequence_id.get() {
                    s.active_polygons.iter_mut().for_each(|p| {
                        if p.id == self.object_id.to_string() {
                            p.border_radius = *h as i32;
                        }
                    });
                    // }
                });

                save_saved_state_raw(saved_state.clone());
            }
            ObjectProperty::StrokeThickness(h) => {
                editor.update_polygon(self.object_id, "stroke_thickness", InputValue::Number(*h));

                let mut stroke_thickness = h.to_string();
                self.signal.expect("signal error").set(stroke_thickness);

                saved_state.sequences.iter_mut().for_each(|s| {
                    // if s.id == selected_sequence_id.get() {
                    s.active_polygons.iter_mut().for_each(|p| {
                        if p.id == self.object_id.to_string() {
                            p.stroke.thickness = *h as i32;
                        }
                    });
                    // }
                });

                save_saved_state_raw(saved_state.clone());
            }
            ObjectProperty::StrokeRed(h) => {
                // let mut stroke_red = h.to_string();
                let red_human = wgpu_to_human(*h);

                editor.update_polygon(self.object_id, "stroke_red", InputValue::Number(red_human));

                self.signal
                    .expect("signal error")
                    .set(red_human.to_string());

                saved_state.sequences.iter_mut().for_each(|s| {
                    // if s.id == selected_sequence_id.get() {
                    s.active_polygons.iter_mut().for_each(|p| {
                        if p.id == self.object_id.to_string() {
                            p.stroke.fill[0] = color_to_wgpu(*h) as i32;
                        }
                    });
                    // }
                });

                save_saved_state_raw(saved_state.clone());
            }
            ObjectProperty::StrokeGreen(h) => {
                // let mut stroke_green = h.to_string();
                let green_human = wgpu_to_human(*h);

                editor.update_polygon(
                    self.object_id,
                    "stroke_green",
                    InputValue::Number(green_human),
                );

                self.signal
                    .expect("signal error")
                    .set(green_human.to_string());

                saved_state.sequences.iter_mut().for_each(|s| {
                    // if s.id == selected_sequence_id.get() {
                    s.active_polygons.iter_mut().for_each(|p| {
                        if p.id == self.object_id.to_string() {
                            p.stroke.fill[1] = color_to_wgpu(*h) as i32;
                        }
                    });
                    // }
                });

                save_saved_state_raw(saved_state.clone());
            }
            ObjectProperty::StrokeBlue(h) => {
                // let mut stroke_blue = h.to_string();
                let blue_human = wgpu_to_human(*h);

                editor.update_polygon(
                    self.object_id,
                    "stroke_blue",
                    InputValue::Number(blue_human),
                );

                self.signal
                    .expect("signal error")
                    .set(blue_human.to_string());

                saved_state.sequences.iter_mut().for_each(|s| {
                    // if s.id == selected_sequence_id.get() {
                    s.active_polygons.iter_mut().for_each(|p| {
                        if p.id == self.object_id.to_string() {
                            p.stroke.fill[2] = color_to_wgpu(*h) as i32;
                        }
                    });
                    // }
                });

                save_saved_state_raw(saved_state.clone());
            } // ObjectProperty::Points(w) => {
              //     editor.update_polygon(self.object_id, "points", InputValue::Points(w.clone()));
              // }
        }
    }
}

pub struct EditorState {
    pub editor: Arc<Mutex<Editor>>,
    pub record: Arc<Mutex<Record<ObjectEdit>>>,
    pub record_state: RecordState,
    pub polygon_selected: bool,
    pub selected_polygon_id: Uuid,
    pub text_selected: bool,
    pub selected_text_id: Uuid,
    pub image_selected: bool,
    pub selected_image_id: Uuid,
    pub value_signals: Arc<Mutex<HashMap<String, RwSignal<String>>>>,
    pub current_modifiers: ModifiersState,
    // pub saved_state: Option<SavedState>,
    pub project_selected_signal: Option<RwSignal<Uuid>>,
    pub active_sequence_mode: RwSignal<String>,
    // pub sequence_timeline_state: TimelineState,
}

pub struct RecordState {
    pub editor: Arc<Mutex<Editor>>,
    // pub record: Arc<Mutex<Record<ObjectEdit>>>,
    // pub editor_state: EditorState,
    pub saved_state: Option<SavedState>,
}

impl EditorState {
    pub fn new(editor: Arc<Mutex<Editor>>, record: Arc<Mutex<Record<ObjectEdit>>>) -> Self {
        // let sequence_timeline_state = TimelineState::new();

        Self {
            editor: Arc::clone(&editor),
            record: Arc::clone(&record),
            record_state: RecordState {
                editor: Arc::clone(&editor),
                saved_state: None,
                // record: Arc::clone(&record),
            },
            polygon_selected: false,
            selected_polygon_id: Uuid::nil(),
            text_selected: false,
            selected_text_id: Uuid::nil(),
            image_selected: false,
            selected_image_id: Uuid::nil(),
            value_signals: Arc::new(Mutex::new(HashMap::new())),
            current_modifiers: ModifiersState::empty(),
            // saved_state: None,
            project_selected_signal: None,
            // sequence_timeline_state,
            active_sequence_mode: RwSignal::new("layout".to_string()),
        }
    }

    pub fn save_default_keyframes(
        &mut self,
        savable_item_id: String,
        object_type: ObjectType,
    ) -> AnimationData {
        let mut properties = Vec::new();

        let mut position_keyframes = Vec::new();

        position_keyframes.push(UIKeyframe {
            id: Uuid::new_v4().to_string(),
            time: Duration::from_secs(0),
            value: KeyframeValue::Position([0, 0]),
            easing: EasingType::EaseInOut,
        });
        position_keyframes.push(UIKeyframe {
            id: Uuid::new_v4().to_string(),
            time: Duration::from_millis(2500),
            value: KeyframeValue::Position([10, 10]),
            easing: EasingType::EaseInOut,
        });
        position_keyframes.push(UIKeyframe {
            id: Uuid::new_v4().to_string(),
            time: Duration::from_secs(5),
            value: KeyframeValue::Position([20, 20]),
            easing: EasingType::EaseInOut,
        });
        position_keyframes.push(UIKeyframe {
            id: Uuid::new_v4().to_string(),
            time: Duration::from_secs(15),
            value: KeyframeValue::Position([20, 20]),
            easing: EasingType::EaseInOut,
        });
        position_keyframes.push(UIKeyframe {
            id: Uuid::new_v4().to_string(),
            time: Duration::from_millis(17500),
            value: KeyframeValue::Position([30, 30]),
            easing: EasingType::EaseInOut,
        });
        position_keyframes.push(UIKeyframe {
            id: Uuid::new_v4().to_string(),
            time: Duration::from_secs(20),
            value: KeyframeValue::Position([40, 40]),
            easing: EasingType::EaseInOut,
        });

        let mut position_prop = AnimationProperty {
            name: "Position".to_string(),
            property_path: "position".to_string(),
            children: Vec::new(),
            keyframes: position_keyframes,
            depth: 0,
        };

        let mut rotation_keyframes = Vec::new();

        rotation_keyframes.push(UIKeyframe {
            id: Uuid::new_v4().to_string(),
            time: Duration::from_secs(0),
            value: KeyframeValue::Rotation(0),
            easing: EasingType::EaseInOut,
        });
        rotation_keyframes.push(UIKeyframe {
            id: Uuid::new_v4().to_string(),
            time: Duration::from_millis(2500),
            value: KeyframeValue::Rotation(0),
            easing: EasingType::EaseInOut,
        });
        rotation_keyframes.push(UIKeyframe {
            id: Uuid::new_v4().to_string(),
            time: Duration::from_secs(5),
            value: KeyframeValue::Rotation(0),
            easing: EasingType::EaseInOut,
        });
        rotation_keyframes.push(UIKeyframe {
            id: Uuid::new_v4().to_string(),
            time: Duration::from_secs(15),
            value: KeyframeValue::Rotation(0),
            easing: EasingType::EaseInOut,
        });
        rotation_keyframes.push(UIKeyframe {
            id: Uuid::new_v4().to_string(),
            time: Duration::from_millis(17500),
            value: KeyframeValue::Rotation(0),
            easing: EasingType::EaseInOut,
        });
        rotation_keyframes.push(UIKeyframe {
            id: Uuid::new_v4().to_string(),
            time: Duration::from_secs(20),
            value: KeyframeValue::Rotation(0),
            easing: EasingType::EaseInOut,
        });

        let mut rotation_prop = AnimationProperty {
            name: "Rotation".to_string(),
            property_path: "rotation".to_string(),
            children: Vec::new(),
            keyframes: rotation_keyframes,
            depth: 0,
        };

        let mut scale_keyframes = Vec::new();

        scale_keyframes.push(UIKeyframe {
            id: Uuid::new_v4().to_string(),
            time: Duration::from_secs(0),
            value: KeyframeValue::Scale(100),
            easing: EasingType::EaseInOut,
        });
        scale_keyframes.push(UIKeyframe {
            id: Uuid::new_v4().to_string(),
            time: Duration::from_millis(2500),
            value: KeyframeValue::Scale(100),
            easing: EasingType::EaseInOut,
        });
        scale_keyframes.push(UIKeyframe {
            id: Uuid::new_v4().to_string(),
            time: Duration::from_secs(5),
            value: KeyframeValue::Scale(100),
            easing: EasingType::EaseInOut,
        });
        scale_keyframes.push(UIKeyframe {
            id: Uuid::new_v4().to_string(),
            time: Duration::from_secs(15),
            value: KeyframeValue::Scale(100),
            easing: EasingType::EaseInOut,
        });
        scale_keyframes.push(UIKeyframe {
            id: Uuid::new_v4().to_string(),
            time: Duration::from_millis(17500),
            value: KeyframeValue::Scale(100),
            easing: EasingType::EaseInOut,
        });
        scale_keyframes.push(UIKeyframe {
            id: Uuid::new_v4().to_string(),
            time: Duration::from_secs(20),
            value: KeyframeValue::Scale(100),
            easing: EasingType::EaseInOut,
        });

        let mut scale_prop = AnimationProperty {
            name: "Scale".to_string(),
            property_path: "scale".to_string(),
            children: Vec::new(),
            keyframes: scale_keyframes,
            depth: 0,
        };

        let mut opacity_keyframes = Vec::new();

        opacity_keyframes.push(UIKeyframe {
            id: Uuid::new_v4().to_string(),
            time: Duration::from_secs(0),
            value: KeyframeValue::Opacity(100),
            easing: EasingType::EaseInOut,
        });
        opacity_keyframes.push(UIKeyframe {
            id: Uuid::new_v4().to_string(),
            time: Duration::from_millis(2500),
            value: KeyframeValue::Opacity(100),
            easing: EasingType::EaseInOut,
        });
        opacity_keyframes.push(UIKeyframe {
            id: Uuid::new_v4().to_string(),
            time: Duration::from_secs(5),
            value: KeyframeValue::Opacity(100),
            easing: EasingType::EaseInOut,
        });
        opacity_keyframes.push(UIKeyframe {
            id: Uuid::new_v4().to_string(),
            time: Duration::from_secs(15),
            value: KeyframeValue::Opacity(100),
            easing: EasingType::EaseInOut,
        });
        opacity_keyframes.push(UIKeyframe {
            id: Uuid::new_v4().to_string(),
            time: Duration::from_millis(17500),
            value: KeyframeValue::Opacity(100),
            easing: EasingType::EaseInOut,
        });
        opacity_keyframes.push(UIKeyframe {
            id: Uuid::new_v4().to_string(),
            time: Duration::from_secs(20),
            value: KeyframeValue::Opacity(100),
            easing: EasingType::EaseInOut,
        });

        let mut opacity_prop = AnimationProperty {
            name: "Opacity".to_string(),
            property_path: "opacity".to_string(),
            children: Vec::new(),
            keyframes: opacity_keyframes,
            depth: 0,
        };

        properties.push(position_prop);
        properties.push(rotation_prop);
        properties.push(scale_prop);
        // properties.push(perspective_x_prop);
        // properties.push(perspective_y_prop);
        properties.push(opacity_prop);

        let new_motion_path = AnimationData {
            id: Uuid::new_v4().to_string(),
            object_type,
            polygon_id: savable_item_id.clone(),
            duration: Duration::from_secs(20),
            properties: properties,
            start_time_ms: 0,
        };

        new_motion_path
    }

    /// squish keyframes into target_duration, keeping proportional time between them
    pub fn scale_keyframes(
        &mut self,
        selected_sequence_id: String,
        target_duration_s: f32,
    ) -> Vec<AnimationData> {
        let target_duration = Duration::from_secs_f32(target_duration_s);
        let mut animations = Vec::new();

        // Find the animation with matching ID
        let saved_state = self
            .record_state
            .saved_state
            .as_ref()
            .expect("Couldn't get saved state");
        if let Some(sequence) = saved_state
            .sequences
            .iter()
            .find(|a| a.id == selected_sequence_id)
        {
            for animation in sequence.polygon_motion_paths.clone() {
                let original_duration = animation.duration;
                let scale_factor = target_duration.as_secs_f32() / original_duration.as_secs_f32();

                // Create a new animation with scaled keyframes
                let mut new_animation = animation.clone();
                new_animation.duration = target_duration;

                // Scale each property's keyframes
                for property in &mut new_animation.properties {
                    for keyframe in &mut property.keyframes {
                        let original_time_secs = keyframe.time.as_secs_f32();
                        let new_time_secs = original_time_secs * scale_factor;
                        keyframe.time = Duration::from_secs_f32(new_time_secs);
                    }
                }

                animations.push(new_animation);
            }
        }

        animations
    }

    pub fn add_saved_polygon(
        &mut self,
        selected_sequence_id: String,
        savable_polygon: SavedPolygonConfig,
    ) {
        let new_motion_path =
            self.save_default_keyframes(savable_polygon.id.clone(), ObjectType::Polygon);

        let mut saved_state = self
            .record_state
            .saved_state
            .as_mut()
            .expect("Couldn't get Saved State");

        saved_state.sequences.iter_mut().for_each(|s| {
            if s.id == selected_sequence_id {
                s.active_polygons.push(savable_polygon.clone());
                s.polygon_motion_paths.push(new_motion_path.clone());
            }
        });

        save_saved_state_raw(saved_state.clone());

        self.record_state.saved_state = Some(saved_state.clone());
    }

    pub fn add_saved_text_item(
        &mut self,
        selected_sequence_id: String,
        savable_text_item: SavedTextRendererConfig,
    ) {
        let new_motion_path =
            self.save_default_keyframes(savable_text_item.id.clone(), ObjectType::TextItem);

        let mut saved_state = self
            .record_state
            .saved_state
            .as_mut()
            .expect("Couldn't get Saved State");

        saved_state.sequences.iter_mut().for_each(|s| {
            if s.id == selected_sequence_id {
                s.active_text_items.push(savable_text_item.clone());
                s.polygon_motion_paths.push(new_motion_path.clone()); // storing alongside polygon motion paths for now
            }
        });

        save_saved_state_raw(saved_state.clone());

        self.record_state.saved_state = Some(saved_state.clone());
    }

    pub fn add_saved_image_item(
        &mut self,
        selected_sequence_id: String,
        savable_image_item: SavedStImageConfig,
    ) {
        let new_motion_path =
            self.save_default_keyframes(savable_image_item.id.clone(), ObjectType::ImageItem);

        let mut saved_state = self
            .record_state
            .saved_state
            .as_mut()
            .expect("Couldn't get Saved State");

        saved_state.sequences.iter_mut().for_each(|s| {
            if s.id == selected_sequence_id {
                s.active_image_items.push(savable_image_item.clone());
                s.polygon_motion_paths.push(new_motion_path.clone()); // storing alongside polygon motion paths for now
            }
        });

        save_saved_state_raw(saved_state.clone());

        self.record_state.saved_state = Some(saved_state.clone());
    }

    // Helper method to register a new signal
    pub fn register_signal(&mut self, name: String, signal: RwSignal<String>) {
        let mut signals = self.value_signals.lock().unwrap();
        signals.insert(name + &self.selected_polygon_id.to_string(), signal);
    }

    pub fn update_width(&mut self, new_width_str: &str) -> Result<(), String> {
        let new_width =
            string_to_f32(new_width_str).map_err(|_| "Couldn't convert string to f32")?;

        let old_width = {
            let editor = self.record_state.editor.lock().unwrap();
            editor.get_polygon_width(self.selected_polygon_id)
        };

        let edit = ObjectEdit {
            object_id: self.selected_polygon_id,
            object_type: ObjectType::Polygon,
            old_value: ObjectProperty::Width(old_width),
            new_value: ObjectProperty::Width(new_width),
            field_name: "width".to_string(),
            signal: Some(
                self.value_signals
                    .lock()
                    .unwrap()
                    .get(&format!("width{}", self.selected_polygon_id))
                    .cloned()
                    .expect("Couldn't get width value signal"),
            ),
        };

        let mut record = self.record.lock().unwrap();
        record.edit(&mut self.record_state, edit);

        Ok(())
    }

    pub fn update_height(&mut self, new_height_str: &str) -> Result<(), String> {
        let new_height =
            string_to_f32(new_height_str).map_err(|_| "Couldn't convert string to f32")?;

        let old_height = {
            let editor = self.editor.lock().unwrap();
            editor.get_polygon_height(self.selected_polygon_id)
        };

        let edit = ObjectEdit {
            object_id: self.selected_polygon_id,
            object_type: ObjectType::Polygon,
            old_value: ObjectProperty::Height(old_height),
            new_value: ObjectProperty::Height(new_height),
            field_name: "height".to_string(),
            signal: Some(
                self.value_signals
                    .lock()
                    .unwrap()
                    .get(&format!("height{}", self.selected_polygon_id))
                    .cloned()
                    .expect("Couldn't get width value signal"),
            ),
        };

        let mut record = self.record.lock().unwrap();
        record.edit(&mut self.record_state, edit);

        Ok(())
    }

    pub fn update_red(&mut self, new_red_str: &str) -> Result<(), String> {
        let new_red = string_to_f32(new_red_str).map_err(|_| "Couldn't convert string to f32")?;

        let old_red = {
            let editor = self.editor.lock().unwrap();
            editor.get_polygon_red(self.selected_polygon_id)
        };

        let edit = ObjectEdit {
            object_id: self.selected_polygon_id,
            object_type: ObjectType::Polygon,
            old_value: ObjectProperty::Red(old_red),
            new_value: ObjectProperty::Red(new_red),
            field_name: "red".to_string(),
            signal: Some(
                self.value_signals
                    .lock()
                    .unwrap()
                    .get(&format!("red{}", self.selected_polygon_id))
                    .cloned()
                    .expect("Couldn't get width value signal"),
            ),
        };

        let mut record = self.record.lock().unwrap();
        record.edit(&mut self.record_state, edit);

        Ok(())
    }

    pub fn update_green(&mut self, new_green_str: &str) -> Result<(), String> {
        let new_green =
            string_to_f32(new_green_str).map_err(|_| "Couldn't convert string to f32")?;

        let old_green = {
            let editor = self.editor.lock().unwrap();
            editor.get_polygon_green(self.selected_polygon_id)
        };

        let edit = ObjectEdit {
            object_id: self.selected_polygon_id,
            object_type: ObjectType::Polygon,
            old_value: ObjectProperty::Green(old_green),
            new_value: ObjectProperty::Green(new_green),
            field_name: "green".to_string(),
            signal: Some(
                self.value_signals
                    .lock()
                    .unwrap()
                    .get(&format!("green{}", self.selected_polygon_id))
                    .cloned()
                    .expect("Couldn't get green value signal"),
            ),
        };

        let mut record = self.record.lock().unwrap();
        record.edit(&mut self.record_state, edit);

        Ok(())
    }

    pub fn update_blue(&mut self, new_blue_str: &str) -> Result<(), String> {
        let new_blue = string_to_f32(new_blue_str).map_err(|_| "Couldn't convert string to f32")?;

        let old_blue = {
            let editor = self.editor.lock().unwrap();
            editor.get_polygon_blue(self.selected_polygon_id)
        };

        let edit = ObjectEdit {
            object_id: self.selected_polygon_id,
            object_type: ObjectType::Polygon,
            old_value: ObjectProperty::Blue(old_blue),
            new_value: ObjectProperty::Blue(new_blue),
            field_name: "blue".to_string(),
            signal: Some(
                self.value_signals
                    .lock()
                    .unwrap()
                    .get(&format!("blue{}", self.selected_polygon_id))
                    .cloned()
                    .expect("Couldn't get blue value signal"),
            ),
        };

        let mut record = self.record.lock().unwrap();
        record.edit(&mut self.record_state, edit);

        Ok(())
    }

    pub fn update_border_radius(&mut self, new_border_radius_str: &str) -> Result<(), String> {
        let new_border_radius = string_to_f32(new_border_radius_str)
            .map_err(|_| "Couldn't convert string to height")?;

        let old_border_radius = {
            let editor = self.editor.lock().unwrap();
            editor.get_polygon_border_radius(self.selected_polygon_id)
        };

        let edit = ObjectEdit {
            object_id: self.selected_polygon_id,
            object_type: ObjectType::Polygon,
            old_value: ObjectProperty::BorderRadius(old_border_radius),
            new_value: ObjectProperty::BorderRadius(new_border_radius),
            field_name: "border_radius".to_string(),
            signal: Some(
                self.value_signals
                    .lock()
                    .unwrap()
                    .get(&format!("border_radius{}", self.selected_polygon_id))
                    .cloned()
                    .expect("Couldn't get border_radius value signal"),
            ),
        };

        let mut record = self.record.lock().unwrap();
        record.edit(&mut self.record_state, edit);

        Ok(())
    }

    pub fn update_stroke_thickness(
        &mut self,
        new_stroke_thickness_str: &str,
    ) -> Result<(), String> {
        let new_stroke_thickness = string_to_f32(new_stroke_thickness_str)
            .map_err(|_| "Couldn't convert string to height")?;

        let old_stroke_thickness = {
            let editor = self.editor.lock().unwrap();
            editor.get_polygon_stroke_thickness(self.selected_polygon_id)
        };

        let edit = ObjectEdit {
            object_id: self.selected_polygon_id,
            object_type: ObjectType::Polygon,
            old_value: ObjectProperty::StrokeThickness(old_stroke_thickness),
            new_value: ObjectProperty::StrokeThickness(new_stroke_thickness),
            field_name: "stroke_thickness".to_string(),
            signal: Some(
                self.value_signals
                    .lock()
                    .unwrap()
                    .get(&format!("stroke_thickness{}", self.selected_polygon_id))
                    .cloned()
                    .expect("Couldn't get stroke_thickness value signal"),
            ),
        };

        let mut record = self.record.lock().unwrap();
        record.edit(&mut self.record_state, edit);

        Ok(())
    }

    pub fn update_stroke_red(&mut self, new_stroke_red_str: &str) -> Result<(), String> {
        let new_stroke_red =
            string_to_f32(new_stroke_red_str).map_err(|_| "Couldn't convert string to height")?;

        let old_stroke_red = {
            let editor = self.editor.lock().unwrap();
            editor.get_polygon_stroke_red(self.selected_polygon_id)
        };

        let edit = ObjectEdit {
            object_id: self.selected_polygon_id,
            object_type: ObjectType::Polygon,
            old_value: ObjectProperty::StrokeRed(old_stroke_red),
            new_value: ObjectProperty::StrokeRed(new_stroke_red),
            field_name: "stroke_red".to_string(),
            signal: Some(
                self.value_signals
                    .lock()
                    .unwrap()
                    .get(&format!("stroke_red{}", self.selected_polygon_id))
                    .cloned()
                    .expect("Couldn't get stroke_red value signal"),
            ),
        };

        let mut record = self.record.lock().unwrap();
        record.edit(&mut self.record_state, edit);

        Ok(())
    }

    pub fn update_stroke_green(&mut self, new_stroke_green_str: &str) -> Result<(), String> {
        let new_stroke_green =
            string_to_f32(new_stroke_green_str).map_err(|_| "Couldn't convert string to height")?;

        let old_stroke_green = {
            let editor = self.editor.lock().unwrap();
            editor.get_polygon_stroke_green(self.selected_polygon_id)
        };

        let edit = ObjectEdit {
            object_id: self.selected_polygon_id,
            object_type: ObjectType::Polygon,
            old_value: ObjectProperty::StrokeGreen(old_stroke_green),
            new_value: ObjectProperty::StrokeGreen(new_stroke_green),
            field_name: "stroke_green".to_string(),
            signal: Some(
                self.value_signals
                    .lock()
                    .unwrap()
                    .get(&format!("stroke_green{}", self.selected_polygon_id))
                    .cloned()
                    .expect("Couldn't get stroke_green value signal"),
            ),
        };

        let mut record = self.record.lock().unwrap();
        record.edit(&mut self.record_state, edit);

        Ok(())
    }

    pub fn update_stroke_blue(&mut self, new_stroke_blue_str: &str) -> Result<(), String> {
        let new_stroke_blue =
            string_to_f32(new_stroke_blue_str).map_err(|_| "Couldn't convert string to height")?;

        let old_stroke_blue = {
            let editor = self.editor.lock().unwrap();
            editor.get_polygon_stroke_blue(self.selected_polygon_id)
        };

        let edit = ObjectEdit {
            object_id: self.selected_polygon_id,
            object_type: ObjectType::Polygon,
            old_value: ObjectProperty::StrokeBlue(old_stroke_blue),
            new_value: ObjectProperty::StrokeBlue(new_stroke_blue),
            field_name: "stroke_blue".to_string(),
            signal: Some(
                self.value_signals
                    .lock()
                    .unwrap()
                    .get(&format!("stroke_blue{}", self.selected_polygon_id))
                    .cloned()
                    .expect("Couldn't get stroke_blue value signal"),
            ),
        };

        let mut record = self.record.lock().unwrap();
        record.edit(&mut self.record_state, edit);

        Ok(())
    }

    pub fn undo(&mut self) {
        let mut record = self.record.lock().unwrap();

        if record.undo(&mut self.record_state).is_some() {
            println!("Undo successful");
            // println!("record cannB... {:?}", self.record.head());
        }
    }

    pub fn redo(&mut self) {
        let mut record = self.record.lock().unwrap();

        if record.redo(&mut self.record_state).is_some() {
            println!("Redo successful");
        }
    }
}
