use floem::common::card_styles;
use floem::common::simple_button;
use floem::common::small_button;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use stunts_engine::editor::string_to_f32;
use stunts_engine::editor::Editor;
use stunts_engine::editor::Viewport;
use stunts_engine::polygon::PolygonConfig;
use uuid::Uuid;
use wgpu::util::DeviceExt;

use floem::peniko::{Brush, Color};
use floem::reactive::{create_effect, create_rw_signal, create_signal, RwSignal, SignalRead};
use floem::reactive::{SignalGet, SignalUpdate};
use floem::text::Weight;
use floem::views::Decorators;
use floem::views::{container, dyn_container, empty, label};
use floem::views::{h_stack, v_stack};
use floem::GpuHelper;
use floem::IntoView;

use crate::editor_state::{self, EditorState};
use crate::helpers::utilities::save_saved_state_raw;

use super::inputs::styled_input;

pub fn properties_view(
    editor_state: Arc<Mutex<EditorState>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    editor: std::sync::Arc<Mutex<Editor>>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
    polygon_selected: RwSignal<bool>,
    selected_polygon_id: RwSignal<Uuid>,
    selected_polygon_data: RwSignal<PolygonConfig>,
    selected_sequence_id: RwSignal<String>,
) -> impl IntoView {
    let editor_cloned = Arc::clone(&editor);
    let editor_state2 = Arc::clone(&editor_state);

    let aside_width = 260.0;
    let quarters = (aside_width / 4.0) + (5.0 * 4.0);
    let thirds = (aside_width / 3.0) + (5.0 * 3.0);
    let halfs = (aside_width / 2.0) + (5.0 * 2.0);

    let back_active = RwSignal::new(false);

    v_stack((
        label(|| "Properties"),
        simple_button("Back to Sequence".to_string(), move |_| {
            polygon_selected.set(false);
        }),
        h_stack((
            styled_input(
                "Width:".to_string(),
                &selected_polygon_data
                    .read()
                    .borrow()
                    .dimensions
                    .0
                    .to_string(),
                "Enter width",
                Box::new({
                    move |mut editor_state, value| {
                        editor_state
                            .update_width(&value)
                            .expect("Couldn't update width");
                        // TODO: probably should update selected_polygon_data
                        // need to update active_polygons in saved_data
                        // TODO: on_debounce_stop?
                        let value = string_to_f32(&value).expect("Couldn't convert string");
                        let mut saved_state = editor_state
                            .saved_state
                            .as_mut()
                            .expect("Couldn't get Saved State");

                        saved_state.sequences.iter_mut().for_each(|s| {
                            if s.id == selected_sequence_id.get() {
                                s.active_polygons.iter_mut().for_each(|p| {
                                    if p.id == selected_polygon_id.get().to_string() {
                                        p.dimensions = (value as i32, p.dimensions.1);
                                    }
                                });
                            }
                        });

                        save_saved_state_raw(saved_state.clone());
                    }
                }),
                editor_state,
                "width".to_string(),
            )
            .style(move |s| s.width(halfs).margin_right(5.0)),
            styled_input(
                "Height:".to_string(),
                &selected_polygon_data
                    .read()
                    .borrow()
                    .dimensions
                    .1
                    .to_string(),
                "Enter height",
                Box::new({
                    move |mut editor_state, value| {
                        editor_state
                            .update_height(&value)
                            .expect("Couldn't update height");
                        // TODO: probably should update selected_polygon_data
                        // need to update active_polygons in saved_data
                        // TODO: on_debounce_stop?
                        let value = string_to_f32(&value).expect("Couldn't convert string");
                        let mut saved_state = editor_state
                            .saved_state
                            .as_mut()
                            .expect("Couldn't get Saved State");

                        saved_state.sequences.iter_mut().for_each(|s| {
                            if s.id == selected_sequence_id.get() {
                                s.active_polygons.iter_mut().for_each(|p| {
                                    if p.id == selected_polygon_id.get().to_string() {
                                        p.dimensions = (p.dimensions.0, value as i32);
                                    }
                                });
                            }
                        });

                        save_saved_state_raw(saved_state.clone());
                    }
                }),
                editor_state2,
                "height".to_string(),
            )
            .style(move |s| s.width(halfs)),
        ))
        .style(move |s| s.width(aside_width)),
    ))
    .style(|s| card_styles(s))
    .style(|s| {
        s.width(300)
            // .absolute()
            .height(800.0)
            .margin_left(0.0)
            .margin_top(20)
            .z_index(10)
    })
}
