use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

// use common_vector::basic::wgpu_to_human;
// use common_vector::editor::{InputValue, PolygonProperty};
// use common_vector::{basic::string_to_f32, editor::Editor};
use floem::keyboard::ModifiersState;
use floem::reactive::{RwSignal, SignalUpdate};
use stunts_engine::animations::{
    AnimationData, AnimationProperty, EasingType, KeyframeValue, UIKeyframe,
};
use stunts_engine::editor::{string_to_f32, Editor, InputValue, PolygonProperty};
use stunts_engine::polygon::SavedPolygonConfig;
use undo::Edit;
use undo::Record;
use uuid::Uuid;

use crate::helpers::saved_state::SavedState;
use crate::helpers::utilities::save_saved_state_raw;

#[derive(Debug)]
pub struct PolygonEdit {
    pub polygon_id: Uuid,
    pub field_name: String,
    pub old_value: PolygonProperty,
    pub new_value: PolygonProperty,
    pub signal: Option<RwSignal<String>>,
}

impl Edit for PolygonEdit {
    type Target = RecordState;
    type Output = ();

    fn edit(&mut self, record_state: &mut RecordState) {
        let mut editor = record_state.editor.lock().unwrap();

        match &self.new_value {
            PolygonProperty::Width(w) => {
                editor.update_polygon(self.polygon_id, "width", InputValue::Number(*w));

                let mut width = w.to_string();
                self.signal.expect("signal error").set(width);
            }
            PolygonProperty::Height(h) => {
                editor.update_polygon(self.polygon_id, "height", InputValue::Number(*h));

                let mut height = h.to_string();
                self.signal.expect("signal error").set(height);
            }
        }
    }

    fn undo(&mut self, record_state: &mut RecordState) {
        let mut editor = record_state.editor.lock().unwrap();

        match &self.old_value {
            PolygonProperty::Width(w) => {
                editor.update_polygon(self.polygon_id, "width", InputValue::Number(*w));

                let mut width = w.to_string();
                self.signal.expect("signal error").set(width);
            }
            PolygonProperty::Height(h) => {
                editor.update_polygon(self.polygon_id, "height", InputValue::Number(*h));

                let mut height = h.to_string();
                self.signal.expect("signal error").set(height);
            }
        }
    }
}

pub struct EditorState {
    pub editor: Arc<Mutex<Editor>>,
    pub record: Arc<Mutex<Record<PolygonEdit>>>,
    pub record_state: RecordState,
    pub polygon_selected: bool,
    pub selected_polygon_id: Uuid,
    pub value_signals: Arc<Mutex<HashMap<String, RwSignal<String>>>>,
    pub current_modifiers: ModifiersState,
    pub saved_state: Option<SavedState>,
    pub project_selected_signal: Option<RwSignal<Uuid>>,
}

pub struct RecordState {
    pub editor: Arc<Mutex<Editor>>,
    // pub record: Arc<Mutex<Record<PolygonEdit>>>,
}

impl EditorState {
    pub fn new(editor: Arc<Mutex<Editor>>, record: Arc<Mutex<Record<PolygonEdit>>>) -> Self {
        Self {
            editor: Arc::clone(&editor),
            record: Arc::clone(&record),
            record_state: RecordState {
                editor: Arc::clone(&editor),
                // record: Arc::clone(&record),
            },
            polygon_selected: false,
            selected_polygon_id: Uuid::nil(),
            value_signals: Arc::new(Mutex::new(HashMap::new())),
            current_modifiers: ModifiersState::empty(),
            saved_state: None,
            project_selected_signal: None,
        }
    }

    pub fn add_saved_polygon(
        &mut self,
        selected_sequence_id: String,
        savable_polygon: SavedPolygonConfig,
    ) {
        let mut saved_state = self.saved_state.as_mut().expect("Couldn't get Saved State");

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

        properties.push(position_prop); // usually easiest with a top-left anchor point...
        properties.push(rotation_prop); // TODO: best with centered anchor point?
        properties.push(scale_prop); // TODO: also better with centered anchor point?
                                     // properties.push(perspective_x_prop);
                                     // properties.push(perspective_y_prop);
        properties.push(opacity_prop);

        let new_motion_path = AnimationData {
            id: Uuid::new_v4().to_string(),
            polygon_id: savable_polygon.id.clone(),
            duration: Duration::from_secs(20),
            properties: properties,
        };

        saved_state.sequences.iter_mut().for_each(|s| {
            if s.id == selected_sequence_id {
                s.active_polygons.push(savable_polygon.clone());
                s.polygon_motion_paths.push(new_motion_path.clone());
            }
        });

        save_saved_state_raw(saved_state.clone());

        self.saved_state = Some(saved_state.clone());
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

        let edit = PolygonEdit {
            polygon_id: self.selected_polygon_id,
            old_value: PolygonProperty::Width(old_width),
            new_value: PolygonProperty::Width(new_width),
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

        let edit = PolygonEdit {
            polygon_id: self.selected_polygon_id,
            old_value: PolygonProperty::Height(old_height),
            new_value: PolygonProperty::Height(new_height),
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
