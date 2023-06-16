use dioxus::prelude::*;

pub trait Segment: Eq + PartialEq + Clone {
    fn id(&self) -> u64;
    fn label(&self) -> String;
    fn selected(&self) -> bool;
    fn dot(&self) -> bool;
}

#[inline_props]
pub fn SegmentedControl<'a, Item: Segment>(
    cx: Scope<'a>,
    items: Vec<Item>,
    onclick: EventHandler<'a, Item>,
) -> Element<'a> {
    cx.render(rsx!(
        div { class: "tabbar",
            items.iter().map(|item| rsx!(TabButton {
            label: item.label(),
            onclick: move |_| onclick.call(item.clone()),
            selected: item.selected(),
            dot: item.dot(),
        }))
        }
    ))
}

#[inline_props]
fn TabButton<'a>(
    cx: Scope<'a>,
    label: String,
    onclick: EventHandler<'a, ()>,
    selected: bool,
    dot: bool,
) -> Element<'a> {
    let dot = dot.then(|| rsx!(span { class: "dot" }));
    let cls = selected.then(|| " selected").unwrap_or_default();
    cx.render(rsx!(
        button { class: "button {cls}", onclick: move |_| {
                onclick.call(());
            }, dot, "{label}" }
    ))
}
