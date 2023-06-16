#![allow(non_snake_case)]
use dioxus::prelude::*;

const STYLE: &str = r#"
    .sb-main {
        width: 100%;
        height: 100vh;
        display: flex;
        user-select: none;
    }
    .sb-sidebar {
        box-sizing: border-box;
        flex-shrink: 0;
        height: 100vh;
        width: 290px;
        min-width: 190px;
        max-width: 450px;
    }
    .sb-resize {
        box-sizing: border-box;
        width: 5px;
        flex: 0 0 auto;
        cursor: ew-resize;
        padding: 0;
        user-select: none;
        -webkit-user-select: none;
        /*background-color: var(--g-backgroundWindow);*/
        background-color: transparent;
        border-right: 1px solid #000;
    }
    .sb-content {
        flex-grow: 1;
        box-sizing: border-box;
        height: 100%;
    }
    .sb-is-resizing * {
        user-select: none;
        -webkit-user-select: none;
        pointer-events: none;
    }
    .sb-resize.sb-is-resizing {
        border-right: 1px solid var(--g-selectedContentBackgroundColor);
    }
    .sb-resize:hover {
        border-right: 2px solid #333;
    }
    /* .sb-resize.sb-is-resizing {
        background-color: var(--g-selectedContentBackgroundColor);
    }*/
    "#;

#[inline_props]
pub fn SplitViewComponent<'a>(
    cx: Scope<'a>,
    sidebar: Element<'a>,
    content: Element<'a>,
) -> Element<'a> {
    let style_html = format!("<style>{STYLE}</style>");

    let size: &UseState<Option<f64>> = use_state(cx, || Some(290.));
    let is_resizing = use_state(cx, || false);
    cx.render(rsx!(
        div {
            class: "sb-main",
            onmouseup: move |_| {
                if *is_resizing.current() {
                    is_resizing.set(false);
                }
            },
            onmousemove: move |event| {
                if *is_resizing.current() {
                    let value = event.data.page_coordinates().x - 8.;
                    size.set(Some(value))
                }
            },
            div { dangerous_inner_html: "{style_html}" }
            SidebarComponent { size: size.clone(), is_resizing: is_resizing.clone(), sidebar }
            ResizeComponent { size: size, is_resizing: is_resizing }
            ContentComponent { is_resizing: is_resizing.clone(), content }
        }
    ))
}

#[inline_props]
fn SidebarComponent<'a>(
    cx: Scope<'a>,
    size: UseState<Option<f64>>,
    is_resizing: UseState<bool>,
    children: Element<'a>,
) -> Element<'a> {
    let style = if let Some(s) = *size.current() {
        format!("width: {s}px;")
    } else {
        "".to_string()
    };
    let class = is_resizing.then(|| "sb-is-resizing").unwrap_or_default();
    cx.render(rsx! {
        div { class: "sb-sidebar {class}", style: "{style}", children }
    })
}

#[inline_props]
fn ResizeComponent<'a>(
    cx: Scope<'a>,
    size: &'a UseState<Option<f64>>,
    is_resizing: &'a UseState<bool>,
) -> Element<'a> {
    let class = is_resizing.then(|| "sb-is-resizing").unwrap_or_default();
    cx.render(rsx!(div {
        class: "sb-resize {class}",
        onmousedown: move |_event| {
            is_resizing.set(true);
        },
        onmouseup: move |_event| { is_resizing.set(false) },
        onmouseout: move |event| {
            if *is_resizing.current() {
                let value = event.data.page_coordinates().x - 8.;
                size.set(Some(value))
            }
        }
    }))
}

#[inline_props]
fn ContentComponent<'a>(
    cx: Scope<'a>,
    is_resizing: UseState<bool>,
    children: Element<'a>,
) -> Element<'a> {
    let class = is_resizing.then(|| "sb-is-resizing").unwrap_or_default();
    cx.render(rsx! {
        div { class: "sb-content {class}", children }
    })
}
