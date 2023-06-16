use dioxus::prelude::*;

#[derive(Clone, Copy, Eq, PartialEq, Debug, Default)]
pub enum TextStyle {
    #[default]
    Primary,
    Secondary,
    Tertiary,
    Quartery,
}

impl TextStyle {
    fn as_css(&self) -> &'static str {
        match self {
            TextStyle::Primary => "label-primary",
            TextStyle::Secondary => "label-secondary",
            TextStyle::Tertiary => "label-tertiary",
            TextStyle::Quartery => "label-quartiary",
        }
    }
}

#[allow(unused)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

impl TextAlign {
    fn rule(&self) -> &'static str {
        match self {
            TextAlign::Left => "text-align: left;",
            TextAlign::Center => "text-align: center;",
            TextAlign::Right => "text-align: right;",
        }
    }
}

#[allow(unused)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum VerticalTextAlign {
    #[default]
    Baseline,
    Top,
    Middle,
    Bottom,
}

impl VerticalTextAlign {
    fn rule(&self) -> &'static str {
        match self {
            VerticalTextAlign::Baseline => "vertical-align: baseline;",
            VerticalTextAlign::Top => "vertical-align: top;",
            VerticalTextAlign::Middle => "vertical-align: middle;",
            VerticalTextAlign::Bottom => "vertical-align: bottom;",
        }
    }
}

#[allow(unused)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum PointerStyle {
    #[default]
    Default,
    Pointer,
}

impl PointerStyle {
    // this `pub` is needed for IconTextButton. Once we have a proper button abstraction, remove
    pub fn rule(&self) -> &'static str {
        match self {
            PointerStyle::Default => "cursor: default;",
            PointerStyle::Pointer => "cursor: pointer;",
        }
    }
}

#[inline_props]
pub fn Paragraph<'a>(
    cx: Scope<'a>,
    style: Option<TextStyle>,
    class: Option<&'static str>,
    pointer_style: Option<PointerStyle>,
    children: Element<'a>,
) -> Element<'a> {
    let style_class = style.unwrap_or_default().as_css();
    let class = class.unwrap_or_default();
    let pointer_style = pointer_style.unwrap_or_default().rule();

    cx.render(rsx!(p {
        class: "{style_class} {class} no-selection",
        style: "{pointer_style}",
        children
    }))
}

#[derive(Props)]
pub struct LabelProps<'a> {
    #[props(optional)]
    pub style: Option<TextStyle>,
    #[props(optional)]
    pub class: Option<&'static str>,
    #[props(optional)]
    pub force_singleline: Option<bool>,
    #[props(optional)]
    pub selectable: Option<bool>,
    #[props(optional)]
    pub clickable: Option<bool>,
    #[props(optional)]
    pub title: Option<&'a str>,
    #[props(optional)]
    pub onclick: Option<EventHandler<'a, Event<MouseData>>>,
    #[props(optional)]
    pub alignment: Option<TextAlign>,
    #[props(optional)]
    pub vertical_alignment: Option<VerticalTextAlign>,
    #[props(optional)]
    pub pointer_style: Option<PointerStyle>,
    #[props(optional)]
    pub dangerous_content: Option<&'a str>,
    pub children: Element<'a>,
}

pub fn Label<'a>(cx: Scope<'a, LabelProps<'a>>) -> Element<'a> {
    let style_class = cx.props.style.unwrap_or_default().as_css();
    let class = cx.props.class.unwrap_or_default();
    let alignment = cx.props.alignment.map(|e| e.rule()).unwrap_or_default();
    let valign = cx
        .props
        .vertical_alignment
        .map(|e| e.rule())
        .unwrap_or_default();
    let singleline = wrp(&cx.props.force_singleline, "overflow-y-hidden no-wrap");
    let selection = if cx.props.selectable.unwrap_or_default() {
        ""
    } else {
        "no-selection"
    };
    let clickable = wrp(&cx.props.clickable, "label-clickable");
    let pointer_style = cx.props.pointer_style.unwrap_or_default().rule();

    let handler = |ev: Event<MouseData>| {
        if let Some(ref o) = cx.props.onclick {
            o.call(ev)
        }
    };

    // lots of overhead just so we can support custom emoji via dangerous content
    if let Some(dangerdanger) = cx.props.dangerous_content {
        cx.render(rsx!(
            span {
                class: "{style_class} {selection} {singleline} {clickable} {class}",
                style: "{alignment} {valign} {pointer_style}",
                title: cx.props.title,
                onclick: handler,
                dangerous_inner_html: "{dangerdanger}",
                &cx.props.children
            }
        ))
    } else {
        cx.render(rsx!(
            span {
                class: "{style_class} {selection} {singleline} {clickable} {class}",
                style: "{alignment} {valign} {pointer_style}",
                title: cx.props.title,
                onclick: handler,
                &cx.props.children
            }
        ))
    }
}

fn wrp<'a, 'b>(input: &'a Option<bool>, k: &'a str) -> &'b str
where
    'a: 'b,
{
    let c = input.unwrap_or_default();
    if c {
        k
    } else {
        ""
    }
}
