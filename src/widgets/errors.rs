use super::{HStack, IconButton, Paragraph, VStack};
use dioxus::prelude::*;

/// A small box that displays an error, with an optional action
#[inline_props]
pub fn ErrorBox<'a>(cx: Scope<'a>, content: String, onclick: EventHandler<'a, ()>) -> Element<'a> {
    cx.render(rsx!(
        div { class: "error-box",
            HStack { class: "align-items-center",
                div { class: "grow",
                    Paragraph { "{content}" }
                }
                IconButton {
                    icon: crate::icons::ICON_DELETE,
                    title: "Clear",
                    onclick: move |_| onclick.call(())
                }
            }
        }
    ))
}

/// A growing page with a centered error message
#[inline_props]
pub fn ErrorPage<'a>(cx: Scope<'a>, content: &'a str) -> Element<'a> {
    cx.render(rsx!(
        div { class: "p-3",
            VStack { class: "grow label-primary",
                h4 { "An error Occurred" }
                Paragraph { "{content}" }
            }
        }
    ))
}
