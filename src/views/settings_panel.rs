use floem::views::Decorators;
use floem::{views::label, IntoView};

pub fn settings_view() -> impl IntoView {
    (label(move || format!("Settings")).style(|s| s.margin_bottom(10)),)
}
