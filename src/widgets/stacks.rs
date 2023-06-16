use dioxus::prelude::*;

#[derive(Props)]
pub struct VStackProps<'a> {
    #[props(optional)]
    pub class: Option<&'a str>,
    pub children: Element<'a>,
}

pub fn VStack<'a>(cx: Scope<'a, VStackProps<'a>>) -> Element {
    let custom_cls = cx.props.class.unwrap_or_default();
    cx.render(rsx!(
        div { class: "vstack {custom_cls}", &cx.props.children }
    ))
}

#[derive(Props)]
pub struct HStackProps<'a> {
    #[props(optional)]
    pub class: Option<&'a str>,
    pub children: Element<'a>,
}

pub fn HStack<'a>(cx: Scope<'a, HStackProps<'a>>) -> Element {
    let custom_cls = cx.props.class.unwrap_or_default();
    cx.render(rsx!(
        div { class: "hstack {custom_cls}", &cx.props.children }
    ))
}
