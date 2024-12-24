use floem::{
    event::EventListener,
    reactive::{create_signal, SignalGet, SignalSet},
    style::Style,
    view::View,
    views::{button, container, label},
    widget::Widget,
};
use std::path::PathBuf;

pub fn upload_field() -> impl View {
    let (selected_file, set_selected_file) = create_signal(None::<PathBuf>);

    container(move || {
        container(move || {
            let btn = button(|| "Choose File".to_string())
                .on_click(move |_| {
                    // Open native file picker
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        set_selected_file(Some(path));
                    }
                })
                .style(
                    Style::new()
                        .padding_horizontal(16.)
                        .padding_vertical(8.)
                        .border(1.)
                        .border_radius(4.)
                        .background("#f0f0f0"),
                );

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
            .style(Style::new().margin_left(8.));

            container(move || (btn, file_name).into_view())
                .style(Style::new().flex_direction_row().align_items_center())
        })
        .style(
            Style::new()
                .width_full()
                .padding(16.)
                .border(1.)
                .border_radius(4.)
                .border_color("#cccccc"),
        )
    })
}
