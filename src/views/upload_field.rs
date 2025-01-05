use floem::common::simple_button;
use floem::event::EventPropagation;
use floem::reactive::create_rw_signal;
use floem::reactive::SignalUpdate;
use floem::taffy::AlignItems;
use floem::views::Decorators;
use floem::{
    event::EventListener,
    reactive::{create_signal, SignalGet},
    style::Style,
    views::{button, container, label},
    View,
};
use std::path::PathBuf;

pub fn upload_field() -> impl View {
    // let (selected_file, set_selected_file) = create_signal(None::<PathBuf>);
    let selected_file = create_rw_signal(None::<PathBuf>);

    let btn = simple_button("Choose File".to_string(), move |_| {
        // Open native file picker
        if let Some(path) = rfd::FileDialog::new().pick_file() {
            selected_file.set(Some(path));
        }

        // EventPropagation::Stop
    })
    .style(|s| {
        s.padding_horiz(16.)
            .padding_vert(8.)
            .border(1.)
            .border_radius(4.)
    });

    let file_name = label(move || {
        selected_file
            .get()
            .map(|p| {
                p.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            })
            .unwrap_or_else(|| "No file selected".to_string())
    })
    .style(|s| s.margin_left(8.));

    (container(
        (container((btn, file_name)).style(|s| s.flex_row().align_items(AlignItems::Center))),
    )
    .style(|s| s.width_full().padding(16.).border(1.).border_radius(4.)))
}
