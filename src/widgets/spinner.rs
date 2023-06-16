use dioxus::prelude::*;

#[inline_props]
pub fn Spinner(cx: Scope, class: Option<&'static str>) -> Element {
    let c = class.unwrap_or_default();
    cx.render(rsx!( div { class: "loader {c}" } ))
}
