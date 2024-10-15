use floem::peniko::Color;
use floem::views::{h_stack, svg, v_stack};
use floem::{
    reactive::create_signal,
    unit::UnitExt, // Import UnitExt for .px() method
    views::{button, label, Decorators},
    IntoView,
};

fn app_view() -> impl IntoView {
    let (counter, mut set_counter) = create_signal(0);

    (
        label(move || format!("Value: {counter}")).style(|s| s.margin_bottom(10)),
        (
            styled_button("Increment", "plus", move || set_counter += 1),
            styled_button("Decrement", "minus", move || set_counter -= 1),
        )
            .style(|s| s.flex_col().gap(10).margin_top(10)),
    )
        .style(|s| s.flex_col().items_center())
}

fn create_icon(name: &str) -> String {
    match name {
        "plus" => r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24"><path fill="none" d="M0 0h24v24H0z"/><path d="M11 11V5h2v6h6v2h-6v6h-2v-6H5v-2z"/></svg>"#.to_string(),
        "minus" => r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24"><path fill="none" d="M0 0h24v24H0z"/><path d="M5 11h14v2H5z"/></svg>"#.to_string(),
        _ => "".to_string(),
    }
}

fn styled_button(
    text: &'static str,
    icon_name: &'static str,
    action: impl FnMut() + 'static,
) -> impl IntoView {
    // button(text)
    button(v_stack((
        svg(create_icon(icon_name)).style(|s| s.width(24).height(24)),
        label(move || text),
    )))
    .action(action)
    .style(|s| {
        s.width(70)
            .height(70)
            .border_radius(15)
            .box_shadow_blur(15)
            .box_shadow_spread(4)
            .box_shadow_color(Color::rgba(0.0, 0.0, 0.0, 0.16))
            // .transition("all 0.2s")
            .hover(|s| s.box_shadow_color(Color::rgba(0.0, 0.0, 0.0, 0.32)))
    })
}

fn main() {
    floem::launch(app_view);
}
