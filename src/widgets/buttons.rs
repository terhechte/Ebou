use super::{Label, PointerStyle};
use dioxus::prelude::*;

pub fn EmojiButton(cx: Scope<'_>) -> Element<'_> {
    // On windows, this isn't possible, it seems
    #[cfg(target_os = "windows")]
    return cx.render(rsx!({}));

    #[cfg(not(target_os = "windows"))]
    {
        use navicula::types::AppWindow;
        let window = AppWindow::retrieve(cx);
        cx.render(rsx!(
            div { class: "icon-button", title: "Emoji & Symbols",
                button {
                    prevent_default: "onmousedown",
                    onmousedown: move |_| {
                        #[cfg(not(target_os = "linux"))]
                        crate::environment::platform::show_emoji_popup(&window);
                    },
                    dangerous_inner_html: crate::icons::ICON_EMOJI
                }
            }
        ))
    }
}

#[derive(Props)]
pub struct IconButtonProps<'a, S: AsRef<str>> {
    pub icon: &'a str,
    pub title: S,
    #[props(optional)]
    pub class: Option<&'a str>,
    pub onclick: EventHandler<'a, MouseEvent>,
}

pub fn IconButton<'a, S: AsRef<str>>(cx: Scope<'a, IconButtonProps<'a, S>>) -> Element {
    let class = cx.props.class.unwrap_or("");
    cx.render(rsx!(
        div { class: "icon-button {class}", title: "{cx.props.title.as_ref()}", button { r#type: "button", onclick: move |e| cx.props.onclick.call(e), dangerous_inner_html: cx.props.icon } }
    ))
}

#[derive(Props)]
pub struct IconProps<'a, S: AsRef<str>> {
    pub icon: &'a str,
    pub title: S,
    #[props(optional)]
    pub class: Option<&'a str>,
}

pub fn Icon<'a, S: AsRef<str>>(cx: Scope<'a, IconProps<'a, S>>) -> Element<'a> {
    let class = cx.props.class.unwrap_or("");
    cx.render(rsx!(
        div { class: "icon {class}", title: "{cx.props.title.as_ref()}", span { dangerous_inner_html: "{cx.props.icon}" } }
    ))
}

#[derive(Props)]
pub struct IconTextButtonProps<'a, S: AsRef<str>, ST: AsRef<str>> {
    pub icon: &'a str,
    pub text: S,
    pub title: ST,
    #[props(optional)]
    pub class: Option<&'a str>,
    #[props(optional)]
    pub disabled: Option<bool>,
    pub onclick: EventHandler<'a, MouseEvent>,
}

pub fn IconTextButton<'a, S: AsRef<str>, ST: AsRef<str>>(
    cx: Scope<'a, IconTextButtonProps<'a, S, ST>>,
) -> Element {
    let disabled = cx
        .props
        .disabled
        .and_then(|e| e.then_some("disabled"))
        .unwrap_or("");
    let pointer_style = cx
        .props
        .disabled
        .and_then(|e| e.then_some(PointerStyle::Default))
        .unwrap_or(PointerStyle::Pointer);
    let rule = pointer_style.rule();
    let class = cx.props.class.unwrap_or("");
    cx.render(rsx!(
        div {
            style: "{rule}",
            class: "icon-button text {disabled} {class}",
            title: "{cx.props.title.as_ref()}",
            onclick: move |e| cx.props.onclick.call(e),
            button { r#type: "button", style: "{rule}", dangerous_inner_html: cx.props.icon }
            Label { onclick: move |e| cx.props.onclick.call(e), pointer_style: pointer_style, "{cx.props.text.as_ref()}" }
        }
    ))
}

#[derive(Props)]
pub struct TextButtonProps<'a, S: AsRef<str>, ST: AsRef<str>> {
    pub text: S,
    pub title: ST,
    #[props(optional)]
    pub class: Option<&'a str>,
    #[props(optional)]
    pub disabled: Option<bool>,
    pub onclick: EventHandler<'a, MouseEvent>,
}

#[allow(unused)]
pub fn TextButton<'a, S: AsRef<str>, ST: AsRef<str>>(
    cx: Scope<'a, TextButtonProps<'a, S, ST>>,
) -> Element {
    let disabled = cx
        .props
        .disabled
        .and_then(|e| e.then_some("disabled"))
        .unwrap_or("");
    let pointer_style = cx
        .props
        .disabled
        .and_then(|e| e.then_some(PointerStyle::Default))
        .unwrap_or(PointerStyle::Pointer);
    let rule = pointer_style.rule();
    let class = cx.props.class.unwrap_or("");
    cx.render(rsx!(
        div {
            style: "{rule}",
            class: "text-button {disabled} {class}",
            title: "{cx.props.title.as_ref()}",
            onclick: move |e| cx.props.onclick.call(e),
            Label { onclick: move |e| cx.props.onclick.call(e), pointer_style: pointer_style, "{cx.props.text.as_ref()}" }
        }
    ))
}
