use dioxus::prelude::*;

#[derive(Props)]
pub struct HideableViewProps<'a> {
    #[props(optional)]
    pub class: Option<&'a str>,
    pub hidden: bool,
    pub children: Element<'a>,
}

pub fn HideableView<'a>(cx: Scope<'a, HideableViewProps<'a>>) -> Element {
    let custom_cls = cx.props.class.unwrap_or_default();
    let h = cx.props.hidden;
    let s = if h {
        "display: none;"
    } else {
        "display: flex; flex-grow: 1;"
    };
    render! {
        div {
            style: "{s}",
            class: "{custom_cls}",
            &cx.props.children
        }
    }
}
