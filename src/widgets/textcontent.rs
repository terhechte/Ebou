use std::borrow::Cow;

use dioxus::prelude::*;

use crate::helper::HtmlItem;

pub enum TextContentAction {
    Tag(String),
    Link(String),
    Account(String),
}

#[inline_props]
pub fn TextContent<'a>(
    cx: Scope<'a>,
    content: Cow<'a, [HtmlItem]>,
    onclick: EventHandler<'a, TextContentAction>,
    class: &'a str,
) -> Element<'a> {
    use crate::helper::HtmlItem::*;
    cx.render(rsx!(
        div { class: "attributed-text {class}",
            p {
                {content.iter().map(|item| match item {
                Text { content } => rsx!(span {
                    "{content} "
                }),
                Mention { url, name} => rsx!(span {
                    class: "mention",
                    onclick: move |_| onclick.call(TextContentAction::Account(url.clone())),
                    "{name}"
                }),
                Link { name, url } => rsx!(span {
                    class: "link",
                    onclick: move |_| onclick.call(TextContentAction::Link(url.clone())),
                    "{name} "
                }),
                Hashtag { name } => rsx!(span {
                    class: "tag",
                    onclick: move |_| onclick.call(TextContentAction::Tag(name.clone())),
                    "{name} "
                }),
                Image { url } => rsx!(img {
                    src: "{url}",
                    class: "emoji-entry"
                }),
                Break => rsx!(br {})
            })}
            }
        }
    ))
}
