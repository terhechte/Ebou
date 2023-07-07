use super::*;

#[inline_props]
pub fn SidebarTextHeadline<'a>(cx: Scope<'a>, text: &'a str) -> Element<'a> {
    render! {
        Label {
            style: TextStyle::Quartery,
            class: "mb-1 mt-2",
            "{text}"
        }
    }
}

#[inline_props]
pub fn SidebarTextEntry<'a>(
    cx: Scope<'a>,
    icon: &'a str,
    text: &'a str,
    selected: bool,
    onclick: EventHandler<'a, ()>,
) -> Element<'a> {
    let class = selected.then(|| "selected").unwrap_or_default();
    render! {
        div {
            class: "sidebar-text-entry {class} no-selection force-pointer",
            onclick: move |_| onclick.call(()),
            span {
                class: "icon no-selection force-pointer",
                "{icon}"
            }
            Label {
                onclick: move |_| onclick.call(()),
                style: TextStyle::Secondary,
                clickable: false,
                pointer_style: PointerStyle::Pointer,
                "{text}"
            }
        }
    }
}
