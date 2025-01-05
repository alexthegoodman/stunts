use std::sync::{Arc, Mutex};

use floem::common::{card_styles, option_button, simple_button, small_button};
use floem::peniko::Color;
use floem::reactive::SignalGet;
use floem::reactive::{create_effect, create_rw_signal, RwSignal, SignalUpdate};
use floem::views::{
    h_stack, scroll, stack, v_stack, virtual_stack, Decorators, VirtualDirection, VirtualItemSize,
};
use floem::GpuHelper;
use floem::{views::label, IntoView};
use floem_renderer::gpu_resources;
use im::HashMap;
use rand::Rng;
use std::str::FromStr;
use stunts_engine::editor::{rgb_to_wgpu, Editor, Point, Viewport, WindowSize};
use stunts_engine::polygon::{Polygon, PolygonConfig, SavedPolygonConfig, Stroke};
use uuid::Uuid;

use crate::editor_state::EditorState;
use crate::helpers::utilities::save_saved_state_raw;
use stunts_engine::animations::{
    AnimationData, AnimationProperty, EasingType, KeyframeValue, Sequence, UIKeyframe,
};

use super::upload_field::upload_field;

pub fn images_view(
    gpu_helper: Arc<Mutex<GpuHelper>>,
    editor_state: Arc<Mutex<EditorState>>,
    editor: std::sync::Arc<Mutex<Editor>>,
    viewport: Arc<Mutex<Viewport>>,
) -> impl IntoView {
    let editor_cloned = Arc::clone(&editor);
    let gpu_cloned = Arc::clone(&gpu_helper);
    let viewport_cloned = Arc::clone(&viewport);
    let state_cloned = Arc::clone(&editor_state);
    let state_cloned2 = Arc::clone(&editor_state);
    let state_cloned3 = Arc::clone(&editor_state);
    let state_cloned4 = Arc::clone(&editor_state);

    // let sequences: RwSignal<im::Vector<Sequence>> = create_rw_signal(im::Vector::new());
    let images: RwSignal<im::Vector<String>> = create_rw_signal(im::Vector::new());
    // let sequence_quick_access: RwSignal<HashMap<String, i32>> = create_rw_signal(HashMap::new());

    create_effect(move |_| {
        // let editor_state = editor_state.lock().unwrap();
        // let saved_state = editor_state
        //     .saved_state
        //     .as_ref()
        //     .expect("Couldn't get Saved State");

        // let im_sequences: im::Vector<String> = saved_state
        //     .sequences
        //     .clone()
        //     .into_iter()
        //     .map(|s| s.id)
        //     .collect();

        // let mut x = 0;

        // let qa_sequences: HashMap<String, i32> = saved_state
        //     .sequences
        //     .clone()
        //     .into_iter()
        //     .map(|s| {
        //         x = x + 1;
        //         (s.id, x)
        //     })
        //     .collect();

        // sequences.set(im_sequences);
        // sequence_quick_access.set(qa_sequences);
    });

    v_stack((
        label(move || format!("Images")).style(|s| s.margin_bottom(10)),
        upload_field(),
        scroll({
            virtual_stack(
                VirtualDirection::Vertical,
                VirtualItemSize::Fixed(Box::new(|| 28.0)),
                move || images.get(),
                move |item| item.clone(),
                move |item| {
                    let state_cloned2 = state_cloned2.clone();
                    let state_cloned3 = state_cloned3.clone();
                    let editor_cloned = editor_cloned.clone();
                    let viewport_cloned = viewport_cloned.clone();

                    let item_cloned = item.clone();

                    // let sequence_quick_access = sequence_quick_access.get();
                    // let quick_access_info = sequence_quick_access
                    //     .get(&item)
                    //     .expect("Couldn't find matching qa info");

                    h_stack((simple_button("Add Image to Scene".to_string(), move |_| {
                        println!("Adding image...");

                        println!("Image added!");

                        // EventPropagation::Continue
                    }),))
                },
            )
            .style(|s| {
                s.flex_col()
                    .width(260.0)
                    .padding_vert(15.0)
                    .padding_horiz(20.0)
                    .background(Color::LIGHT_BLUE)
            })
        })
        .style(|s| s.height(400.0)),
    ))
    .style(|s| card_styles(s))
    .style(|s| s.width(300.0))
}
