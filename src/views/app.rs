use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::fs;
use std::path::Path;
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex, MutexGuard};

use bytemuck::Contiguous;
use floem::event::{Event, EventListener, EventPropagation};
use floem::keyboard::{Key, KeyCode, NamedKey};
use floem::kurbo::Size;
use floem::peniko::Color;
use floem::reactive::{create_effect, create_rw_signal, create_signal, RwSignal, SignalRead};
use floem::style::{Background, CursorStyle, Transition};
use floem::taffy::AlignItems;
use floem::text::Weight;
use floem::views::editor::view;
use floem::views::{
    container, dyn_container, empty, label, scroll, stack, tab, text_input, virtual_stack,
    VirtualDirection, VirtualItemSize,
};
use floem::window::WindowConfig;
use floem_renderer::gpu_resources::{self, GpuResources};
use floem_winit::dpi::{LogicalSize, PhysicalSize};
use floem_winit::event::{ElementState, MouseButton};
use stunts_engine::editor::{Editor, Viewport};
use uuid::Uuid;
// use views::buttons::{nav_button, option_button, small_button};
// use winit::{event_loop, window};
use wgpu::util::DeviceExt;

use floem::context::PaintState;
// use floem::floem_reactive::SignalGet;
use floem::reactive::{SignalGet, SignalUpdate};
use floem::views::text;
use floem::views::Decorators;
use floem::views::{h_stack, svg, v_stack};
use floem::{
    views::{button, dropdown},
    IntoView,
};
use floem::{Application, CustomRenderCallback};
use floem::{GpuHelper, View, WindowHandle};

use crate::editor_state::EditorState;

use super::aside::tab_interface;
use super::properties_panel::properties_view;

pub fn app_view(
    editor_state: Arc<Mutex<EditorState>>,
    editor: std::sync::Arc<Mutex<Editor>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
) -> impl IntoView {
    let editor_cloned = Arc::clone(&editor);
    let editor_cloned2 = Arc::clone(&editor);
    let editor_cloned3 = Arc::clone(&editor);
    let editor_cloned4 = Arc::clone(&editor);

    // let polygon_selected = create_rw_signal(false);
    // let selected_polygon_id = create_rw_signal(Uuid::nil());
    // let selected_polygon_data = create_rw_signal(PolygonConfig {
    //     id: Uuid::nil(),
    //     name: String::new(),
    //     points: Vec::new(),
    //     dimensions: (100.0, 100.0),
    //     position: Point { x: 0.0, y: 0.0 },
    //     border_radius: 0.0,
    //     fill: [0.0, 0.0, 0.0, 1.0],
    //     stroke: Stroke {
    //         fill: [1.0, 1.0, 1.0, 1.0],
    //         thickness: 2.0,
    //     },
    // });

    let editor_cloned2 = editor_cloned2.clone();

    container((
        tab_interface(
            gpu_helper.clone(),
            editor,
            viewport.clone(),
            // polygon_selected,
        ),
        // dyn_container(
        //     move || polygon_selected.get(),
        //     move |polygon_selected_real| {
        //         if polygon_selected_real {
        //             properties_view(
        //                 editor_state.clone(),
        //                 gpu_helper.clone(),
        //                 editor_cloned4.clone(),
        //                 viewport.clone(),
        //                 // polygon_selected,
        //                 // selected_polygon_id,
        //                 // selected_polygon_data,
        //             )
        //             .into_any()
        //         } else {
        //             empty().into_any()
        //         }
        //     },
        // ),
    ))
}
