#![allow(non_snake_case)]
use dioxus::prelude::*;

#[inline_props]
pub fn SplitViewComponent<'a>(
    cx: Scope<'a>,
    navbar: Option<Element<'a>>,
    sidebar: Element<'a>,
    content: Element<'a>,
) -> Element<'a> {
    let size: &UseState<Option<f64>> = use_state(cx, || {
        if cfg!(target_os = "ios") {
            None
        } else {
            Some(290.)
        }
    });
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
            navbar.as_ref().map(|navbar| rsx!(NavBarComponent { navbar }))
            SidebarComponent { size: size.clone(), is_resizing: is_resizing.clone(), sidebar }
            ResizeComponent { size: size, is_resizing: is_resizing }
            ContentComponent { is_resizing: is_resizing.clone(), content }
        }
    ))
}

#[inline_props]
fn NavBarComponent<'a>(cx: Scope<'a>, children: Element<'a>) -> Element<'a> {
    render! {
        div {
            class: "sb-navbar",
            children
        }
    }
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
    // on iPad OS do nothing
    if cfg!(target_os = "ios") {
        return cx.render(rsx!(div {}));
    }
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
