use floem::common::card_styles;
use floem::common::small_button;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use stunts_engine::editor::Editor;
use stunts_engine::editor::Viewport;
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

use super::inputs::styled_input;

pub fn properties_view(
    editor_state: Arc<Mutex<EditorState>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    editor: std::sync::Arc<Mutex<Editor>>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
    // polygon_selected: RwSignal<bool>,
    // selected_polygon_id: RwSignal<Uuid>,
    // selected_polygon_data: RwSignal<PolygonConfig>,
) -> impl IntoView {
    let editor_cloned = Arc::clone(&editor);

    let editor_state2 = Arc::clone(&editor_state);

    let aside_width = 260.0;
    let quarters = (aside_width / 4.0) + (5.0 * 4.0);
    let thirds = (aside_width / 3.0) + (5.0 * 3.0);
    let halfs = (aside_width / 2.0) + (5.0 * 2.0);

    let back_active = RwSignal::new(false);

    v_stack((label(|| "Properties"),))
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
