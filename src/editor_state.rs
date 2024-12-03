use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// use common_vector::basic::wgpu_to_human;
// use common_vector::editor::{InputValue, PolygonProperty};
// use common_vector::{basic::string_to_f32, editor::Editor};
use floem::keyboard::ModifiersState;
use floem::reactive::{RwSignal, SignalUpdate};
use stunts_engine::editor::{string_to_f32, Editor, InputValue, PolygonProperty};
use undo::Edit;
use undo::Record;
use uuid::Uuid;

use crate::helpers::saved_state::SavedState;

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
        }
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
