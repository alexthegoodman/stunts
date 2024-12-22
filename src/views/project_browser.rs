use std::str::FromStr;
use std::sync::{Arc, Mutex, MutexGuard};

use floem::common::{alert, card_styles, create_icon, nav_button};
use floem::event::{Event, EventListener, EventPropagation};
use floem::keyboard::{Key, KeyCode, NamedKey};
use floem::peniko::Color;
use floem::reactive::{create_effect, create_rw_signal, create_signal, RwSignal, SignalRead};
use floem::style::CursorStyle;
use floem::taffy::AlignItems;
use floem::text::Weight;
use floem::views::{
    container, dyn_container, dyn_stack, empty, h_stack, img, label, scroll, stack, svg, tab,
    text_input, v_stack, virtual_list, virtual_stack, VirtualDirection, VirtualItemSize,
};
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
use floem::IntoView;
use floem::{GpuHelper, View, WindowHandle};

use crate::editor_state::EditorState;
use crate::helpers::projects::{get_projects, ProjectInfo};
use crate::helpers::utilities::load_project_state;
// use crate::helpers::projects::{get_projects, ProjectInfo};
// use crate::helpers::websocket::WebSocketManager;

pub fn project_item(
    project_info: ProjectInfo,
    sortable_items: RwSignal<Vec<ProjectInfo>>,
    project_label: String,
    icon_name: &'static str,
) -> impl IntoView {
    h_stack((
        svg(create_icon(icon_name))
            .style(|s| s.width(24).height(24).color(Color::BLACK))
            .style(|s| s.margin_right(7.0)),
        // .on_event_stop(
        //     floem::event::EventListener::PointerDown,
        //     |_| { /* Disable dragging for this view */ },
        // ),
        label(move || project_label.to_string()),
    ))
    .style(|s| {
        s.width(260.0)
            .border_radius(15.0)
            .align_items(AlignItems::Center)
            .justify_start()
            .padding_vert(8)
            .background(Color::rgb(255.0, 255.0, 255.0))
            .border_bottom(1)
            .border_color(Color::rgb(200.0, 200.0, 200.0))
            .hover(|s| {
                s.background(Color::rgb(100.0, 100.0, 100.0))
                    .cursor(CursorStyle::Pointer)
            })
            .active(|s| s.background(Color::rgb(237.0, 218.0, 164.0)))
    })
    // .on_click(|_| {
    //     println!("Layer selected");
    //     EventPropagation::Stop
    // })
}

pub fn project_browser(
    editor_state: Arc<Mutex<EditorState>>,
    editor: std::sync::Arc<Mutex<Editor>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
) -> impl View {
    // TODO: Alert for Start CommonOS File Manager to use Midpoint
    let projects = get_projects().expect("Couldn't get projects");

    let gpu_2 = Arc::clone(&gpu_helper);

    let project_list = create_rw_signal(projects);
    let loading_project = create_rw_signal(false);

    v_stack((
        dyn_container(
            move || loading_project.get(),
            move |loading_project_real| {
                if (loading_project_real) {
                    alert(
                        floem::common::AlertVariant::Info,
                        "Loading your project...".to_string(),
                    )
                    .into_any()
                } else {
                    empty().into_any()
                }
            },
        )
        .into_view(),
        alert(
            floem::common::AlertVariant::Info,
            "Make sure CommonOS Files is running and you are signed in to assure you can generate concepts, models, and animations.".to_string(),
        ).style(|s| s.margin_bottom(10.0)),
        (label(|| "Select a Project").style(|s| s.margin_bottom(4.0))),
        scroll(
            dyn_stack(
                move || project_list.get(),
                move |project| project.name.clone(),
                move |project| {
                    project_item(
                        project.clone(),
                        project_list,
                        "Project".to_string(),
                        "sphere",
                    )
                    .on_click({
                        let editor = editor.clone();
                        let editor_state = editor_state.clone();
                        // let manager = manager.clone();
                        let gpu_2 = gpu_2.clone();

                        move |_| {
                            if (loading_project.get()) {
                                return EventPropagation::Continue;
                            }

                            loading_project.set(true);

                            // join the WebSocket group for this project
                            // manager.join_group(); // locks and drops the state_helper

                            let mut editor_state = editor_state.lock().unwrap();

                            let uuid = Uuid::from_str(&project.name.clone())
                                .expect("Couldn't convert project name to id");

                            let destination_view = "scene".to_string();
                            // no need to set here, the default is scene
                            // let current_view_signal = state_helper
                            //     .current_view_signal
                            //     .expect("Couldn't get current view signal");
                            // current_view_signal.set(destination_view.clone());

                            // retrieve saved state of project and set on helper
                            // restore the saved state to the rendererstate
                            println!("Loading saved state...");
                            let saved_state = load_project_state(uuid.clone().to_string()).expect("Couldn't get Saved State");
                            editor_state.saved_state = Some(saved_state.clone());

                            // update the UI signal
                            let project_selected = editor_state
                                .project_selected_signal
                                .expect("Couldn't get project selection signal");
                            
                            project_selected.set(uuid.clone());

                            drop(editor_state);

                            // update renderer_state with project_selected (and current_view if necessary)
                            // let mut renderer_state = editor
                            //     .renderer_state
                            //     .as_mut()
                            //     .expect("Couldn't find RendererState")
                            //     .lock()
                            //     .unwrap();
                            let mut editor = editor.lock().unwrap();

                            editor.project_selected = Some(uuid.clone());
                            editor.current_view = destination_view.clone();

                            drop(editor);

                            println!("Project selected {:?}", project.name.clone());

                            EventPropagation::Stop
                        }
                    })
                },
            )
            // .style(|s| s.flex_col().column_gap(5).padding(10))
            .into_view(),
        ),
    ))
    .style(|s| card_styles(s))
    .style(|s| s.width(300.0))
}
