use std::sync::{Arc, Mutex};

use floem::common::{card_styles, option_button, small_button};
use floem::peniko::Color;
use floem::views::{
    scroll, stack, v_stack, virtual_stack, Decorators, VirtualDirection, VirtualItemSize,
};
use floem::GpuHelper;
use floem::{views::label, IntoView};
use stunts_engine::editor::{Editor, Point, Viewport, WindowSize};
use stunts_engine::polygon::{Polygon, PolygonConfig, Stroke};
use uuid::Uuid;

pub fn assets_view(
    gpu_helper: Arc<Mutex<GpuHelper>>,
    editor: std::sync::Arc<Mutex<Editor>>,
    viewport: Arc<Mutex<Viewport>>,
) -> impl IntoView {
    let editor_cloned = Arc::clone(&editor);
    let gpu_cloned = Arc::clone(&gpu_helper);
    let viewport_cloned = Arc::clone(&viewport);

    v_stack((
        label(move || format!("Assets / Motion")).style(|s| s.margin_bottom(10)),
        scroll({
            virtual_stack(
                VirtualDirection::Vertical,
                VirtualItemSize::Fixed(Box::new(|| 90.0)),
                move || tabs.get(),
                move |item| *item,
                move |item| {
                    let index = tabs
                        .get_untracked()
                        .iter()
                        .position(|it| *it == item)
                        .unwrap();
                    let active = index == active_tab.get();
                    let icon_name = match item {
                        "Motion" => "brush",
                        "Settings" => "gear",
                        _ => "plus",
                    };
                    stack((small_button(
                        item,
                        icon_name,
                        move |_| {
                            println!("Click...");
                            set_active_tab.update(|v: &mut usize| {
                                *v = tabs
                                    .get_untracked()
                                    .iter()
                                    .position(|it| *it == item)
                                    .unwrap();
                            });
                            // EventPropagation::Continue
                        },
                        active,
                    ),))
                    .style(move |s| {
                        s.margin_bottom(15.0)
                            .border_radius(15)
                            .apply_if(active, |s| s.border(1.0).border_color(Color::GRAY))
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
