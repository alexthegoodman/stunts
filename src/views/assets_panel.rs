use std::sync::{Arc, Mutex};

use floem::common::{card_styles, option_button, simple_button, small_button};
use floem::peniko::Color;
use floem::reactive::SignalGet;
use floem::reactive::{create_effect, create_rw_signal, RwSignal, SignalUpdate};
use floem::views::{
    scroll, stack, v_stack, virtual_stack, Decorators, VirtualDirection, VirtualItemSize,
};
use floem::GpuHelper;
use floem::{views::label, IntoView};
use stunts_engine::editor::{Editor, Point, Viewport, WindowSize};
use stunts_engine::polygon::{Polygon, PolygonConfig, Stroke};
use uuid::Uuid;

use crate::editor_state::EditorState;
use crate::helpers::saved_state::Sequence;

pub fn assets_view(
    gpu_helper: Arc<Mutex<GpuHelper>>,
    editor_state: Arc<Mutex<EditorState>>,
    editor: std::sync::Arc<Mutex<Editor>>,
    viewport: Arc<Mutex<Viewport>>,
) -> impl IntoView {
    let editor_cloned = Arc::clone(&editor);
    let gpu_cloned = Arc::clone(&gpu_helper);
    let viewport_cloned = Arc::clone(&viewport);

    let sequences: RwSignal<im::Vector<Sequence>> = create_rw_signal(im::Vector::new());

    create_effect(move |_| {
        let editor_state = editor_state.lock().unwrap();
        let saved_state = editor_state
            .saved_state
            .as_ref()
            .expect("Couldn't get Saved State");

        let im_sequences: im::Vector<Sequence> =
            saved_state.sequences.clone().into_iter().collect();

        sequences.set(im_sequences);
    });

    v_stack((
        label(move || format!("Sequences")).style(|s| s.margin_bottom(10)),
        scroll({
            virtual_stack(
                VirtualDirection::Vertical,
                VirtualItemSize::Fixed(Box::new(|| 90.0)),
                move || sequences.get(),
                move |item| item.id.clone(),
                move |item| {
                    simple_button(item.id.clone(), move |_| {
                        println!("Open Sequence...");
                        // set_active_tab.update(|v: &mut usize| {
                        //     *v = tabs
                        //         .get_untracked()
                        //         .iter()
                        //         .position(|it| *it == item)
                        //         .unwrap();
                        // });
                        // EventPropagation::Continue
                    })
                },
            )
            .style(|s| {
                s.flex_col()
                    .height_full()
                    .width(110.0)
                    .padding_vert(15.0)
                    .padding_horiz(20.0)
            })
        }),
    ))
    .style(|s| card_styles(s))
}
