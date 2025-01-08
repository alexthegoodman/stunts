use floem::common::{card_styles, tab_button};
use floem::event::{Event, EventListener, EventPropagation};
use floem::keyboard::{Key, NamedKey};
use floem::reactive::create_signal;
use floem::reactive::SignalGet;
use floem::reactive::SignalUpdate;
use floem::views::{container, dyn_stack, h_stack, scroll, stack, tab, text, v_stack, Decorators};
use floem::View;
use floem::{views::label, IntoView};

const BSD_DISCLAIMER: &str = include_str!("../../licenses/bsd.txt");
const OFL_LICENSE: &str = include_str!("../../licenses/ofl.txt");
const FONT_COPYRIGHTS: &str = include_str!("../../licenses/copyrights.txt");

pub fn bsd_disclaimer_view() -> impl View {
    container((text(BSD_DISCLAIMER)))
}

pub fn ofl_license_view() -> impl View {
    v_stack((text(FONT_COPYRIGHTS), text(OFL_LICENSE)))
}

pub fn settings_view() -> impl View {
    let tabs: im::Vector<&str> = vec!["BSD Disclaimer", "OFL License"].into_iter().collect();
    let (tabs, _set_tabs) = create_signal(tabs);
    let (active_tab, set_active_tab) = create_signal(0);

    let list = scroll({
        dyn_stack(
            // VirtualDirection::Horizontal,
            // VirtualItemSize::Fixed(Box::new(|| 90.0)),
            move || tabs.get(),
            move |item| *item,
            move |item| {
                let index = tabs
                    .get_untracked()
                    .iter()
                    .position(|it| *it == item)
                    .unwrap();
                let active = index == active_tab.get();
                // let icon_name = match item {
                //     "Projects" => "folder-plus",
                //     "Settings" => "gear",
                //     _ => "plus",
                // };
                // let destination_view = match item {
                //     "Projects" => "projects",
                //     "Settings" => "editor_settings",
                //     _ => "plus",
                // };
                stack((
                    // label(move || item).style(|s| s.font_size(18.0)),
                    // svg(create_icon("plus")).style(|s| s.width(24).height(24)),
                    tab_button(
                        item,
                        // icon_name,
                        Some({
                            // let state_helper = state_helper.clone();

                            move || {
                                println!("Click...");
                                set_active_tab.update(|v: &mut usize| {
                                    *v = tabs
                                        .get_untracked()
                                        .iter()
                                        .position(|it| *it == item)
                                        .unwrap();
                                });

                                // EventPropagation::Continue
                            }
                        }),
                        index,
                        active_tab,
                    ),
                ))
                // .on_click()
                .on_event(EventListener::KeyDown, move |e| {
                    if let Event::KeyDown(key_event) = e {
                        let active = active_tab.get();
                        if key_event.modifiers.is_empty() {
                            match key_event.key.logical_key {
                                Key::Named(NamedKey::ArrowUp) => {
                                    if active > 0 {
                                        set_active_tab.update(|v| *v -= 1)
                                    }
                                    EventPropagation::Stop
                                }
                                Key::Named(NamedKey::ArrowDown) => {
                                    if active < tabs.get().len() - 1 {
                                        set_active_tab.update(|v| *v += 1)
                                    }
                                    EventPropagation::Stop
                                }
                                _ => EventPropagation::Continue,
                            }
                        } else {
                            EventPropagation::Continue
                        }
                    } else {
                        EventPropagation::Continue
                    }
                })
                .keyboard_navigatable()
            },
        )
        .style(|s| s.flex_row().padding_vert(7.0).height(55.0))
    })
    // .scroll_style(|s| s.shrink_to_fit())
    .style(|s| s.height(55.0).width(260.0));

    h_stack((
        (v_stack(((label(|| "Settings"), list)))
            .style(|s| card_styles(s))
            .style(|s| s.width(300.0))),
        tab(
            // active tab
            move || active_tab.get(),
            move || tabs.get(),
            |it| *it,
            move |it| match it {
                "BSD Disclaimer" => bsd_disclaimer_view().into_any(),
                "OFL License" => ofl_license_view().into_any(),
                _ => label(|| "Not implemented".to_owned()).into_any(),
            },
        )
        .style(|s| s.flex_col().items_start().margin_top(20.0)),
    ))
}
