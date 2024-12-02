use floem::views::Decorators;
use floem::{views::label, IntoView};

pub fn assets_view() -> impl IntoView {
    (label(move || format!("Assets")).style(|s| s.margin_bottom(10)),)
}
