use floem::peniko::Color;
use floem::reactive::{SignalGet, SignalUpdate};
use floem::views::text;
use floem::views::{h_stack, svg, v_stack};
use floem::{
    reactive::create_signal,
    unit::UnitExt, // Import UnitExt for .px() method
    views::{button, dropdown, label, Decorators},
    IntoView,
};

// Define an enum for our dropdown options
#[derive(Clone, PartialEq, Debug)]
enum DropdownOption {
    Option1,
    Option2,
    Option3,
}

impl std::fmt::Display for DropdownOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DropdownOption::Option1 => write!(f, "Option 1"),
            DropdownOption::Option2 => write!(f, "Option 2"),
            DropdownOption::Option3 => write!(f, "Option 3"),
        }
    }
}

fn app_view() -> impl IntoView {
    let (counter, mut set_counter) = create_signal(0);
    let (selected_option, set_selected_option) = create_signal(DropdownOption::Option1);

    println!("selected_option {:?}", selected_option.get());

    (
        label(move || format!("Value: {counter}")).style(|s| s.margin_bottom(10)),
        (
            styled_button("Increment", "plus", move || set_counter += 1),
            styled_button("Decrement", "minus", move || set_counter -= 1),
        )
            .style(|s| s.flex_col().gap(10).margin_top(10)),
        dropdown::dropdown(
            // Active item (currently selected option)
            move || {
                let see = selected_option.get();
                println!("see {:?}", see);
                see
            },
            // Main view (what's always visible)
            |option: DropdownOption| Box::new(label(move || format!("Selected: {}", option))),
            // Iterator of all options
            vec![
                DropdownOption::Option1,
                DropdownOption::Option2,
                DropdownOption::Option3,
            ],
            // List item view (how each option in the dropdown is displayed)
            // move |option: DropdownOption| {
            //     let option_clone = option.clone();
            //     Box::new(button(option.to_string()).action(move || {
            //         println!("DropdownOption {:?}", option_clone.clone());
            //         set_selected_option.set(option_clone.clone());
            //     }))
            // },
            move |m| text(m.to_string()).into_any(),
        )
        .on_accept(move |new| set_selected_option.set(new)),
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
