use crate::environment::Environment;
use dioxus::prelude::*;

use super::reducer::{LoginAction, LoginState, Selection, ViewStore};
use crate::loc;
use crate::widgets::*;

#[inline_props]
pub fn LoginApp<'a>(
    cx: Scope<'a>,
    environment: &'a UseState<Environment>,
    should_show_login: &'a UseState<bool>,
) -> Element<'a> {
    log::trace!("rerender login app");

    let view_store: ViewStore = navicula::root(cx, &[], environment.get(), LoginState::default);

    let did_close = use_state(cx, || false);

    // We're done, return the model
    if view_store.done && !environment.get().model.has_token {
        if let Some(model) = view_store.send_model.take().take().map(|o| o.cloned()) {
            let mut mutable_environment = environment.get().clone();
            mutable_environment.update_model(model);
            environment.set(mutable_environment);
        }
    }

    if !(*did_close.get()) && view_store.close && view_store.done {
        did_close.set(true);
        should_show_login.set(false);
    }

    cx.render(rsx!(
        div { class: "login-container", MainView { view_store: view_store } }
    ))
}

#[inline_props]
fn MainView<'a>(cx: Scope<'a>, view_store: ViewStore<'a>) -> Element<'a> {
    cx.render(rsx!(
        div { class: "vstack p-2 m-2 grow align-items-center justify-content-center justify-items-center",
            div { class: "justify-self-center", Welcome { view_store: view_store } }
        }
    ))
}

#[inline_props]
fn Welcome<'a>(cx: Scope<'a>, view_store: &'a ViewStore<'a>) -> Element<'a> {
    use LoginAction::*;
    use PageVisibility::*;

    let entered_code = use_state(cx, String::new);

    let (a, b, c, action) = match (view_store.app_data.is_some(), view_store.done) {
        (false, false) => (Visible, Pre, Pre, ChosenInstance),
        (true, false) => (Post, Visible, Pre, EnteredCode(entered_code.get().clone())),
        (true, true) => (Post, Post, Visible, CloseLogin),
        (false, true) => return cx.render(rsx!( div { "Error" } )),
    };

    let (enabled, visible_l, visible_r, t1, mut t2) = match (
        b,
        view_store.selected_instance.is_some() || view_store.selected_instance_url.is_some(),
        !(entered_code.get().is_empty()),
    ) {
        (Pre, false, _) => (false, true, true, loc!("Register"), loc!("Continue")),
        (Pre, true, _) => (true, true, true, loc!("Register"), loc!("Continue")),
        (Visible, _, false) => (true, true, true, loc!("Back"), loc!("Confirm")),
        (Visible, _, true) => (true, false, true, "", loc!("Confirm")),
        _ => (true, false, true, "", loc!("Done")),
    };

    if view_store.selected_instance_url.is_some() && matches!(action, LoginAction::ChosenInstance) {
        t2 = loc!("Use Custom");
    }

    let has_entered_code = matches!(action, LoginAction::EnteredCode(_));

    cx.render(rsx!(
        div { class: "login-form no-selection",
            VStack { class: "gap-3",
                HStack { class: "gap-3 align-items-center",
                    h3 { loc!("Welcome to Ebou") }
                    span { "𝛼" }
                }
                div { class: "page-container",
                    Page1 { visibility: a, view_store: view_store }
                    Page2 { visibility: b, view_store: view_store, code: entered_code }
                    Page3 { visibility: c, view_store: view_store }
                }
                PageButtons {
                    loading: view_store.is_loading,
                    left: ButtonConfig {
                        visible: visible_l,
                        enabled,
                        title: t1,
                        onclick: Box::new(move || {
                            if has_entered_code {
                                view_store.send(LoginAction::ChosenInstance)
                            } else {
                                view_store.send(LoginAction::ActionRegister)
                            }
                        }),
                    },
                    right: ButtonConfig {
                        visible: visible_r,
                        enabled,
                        title: t2,
                        onclick: Box::new(move || view_store.send(action.clone())),
                    }
                }
                view_store.error_message.as_ref().map(|error| rsx!(ErrorBox {
                content: error.clone(),
                onclick: move |_| {}
            }))
            }
        }
    ))
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum PageVisibility {
    Pre,
    Visible,
    Post,
}

impl PageVisibility {
    fn class(&self) -> &'static str {
        match self {
            PageVisibility::Pre => "pre-appear",
            PageVisibility::Visible => "appear",
            PageVisibility::Post => "post-appear",
        }
    }
}

#[inline_props]
fn Page1<'a>(
    cx: Scope<'a>,
    visibility: PageVisibility,
    view_store: &'a ViewStore<'a>,
) -> Element<'a> {
    let class = visibility.class();
    cx.render(rsx!(
        VStack { class: "page1 page gap-3 {class}",
            input {
                r#type: "text",
                placeholder: "Select or Enter a Mastodon or Pleroma Server (including https://)",
                autocomplete: "off",
                spellcheck: "false",
                oninput: move |evt| {
                    let value = evt.value.clone();
                    view_store.send(LoginAction::SelectInstance(Selection::Host(value)))
                }
            }
            InstanceList { view_store: view_store }
        }
    ))
}

struct ButtonConfig<'a> {
    visible: bool,
    enabled: bool,
    title: &'a str,
    onclick: Box<dyn Fn() + 'a>,
}

impl<'a> ButtonConfig<'a> {
    fn classes(&self) -> String {
        let mut base = "button ".to_string();
        if self.visible {
            // base.push_str("visible ");
        } else {
            base.push_str("hidden-button ");
        }
        base
    }

    fn disabled(&self) -> String {
        if self.enabled { "false" } else { "true" }.to_string()
    }
}

#[inline_props]
fn PageButtons<'a>(
    cx: Scope<'a>,
    loading: bool,
    left: ButtonConfig<'a>,
    right: ButtonConfig<'a>,
) -> Element<'a> {
    let lc = left.classes();
    let rc = right.classes();
    cx.render(rsx!(
        HStack { class: "justify-content-between align-items-center",
            button { class: "{lc}", onclick: move |_| (*left.onclick)(), disabled: "{left.disabled()}", "{left.title}" }
            loading.then(|| rsx!(Spinner {})),
            button {
                class: "{rc} highlighted",
                onclick: move |_| (*right.onclick)(),
                disabled: "{right.disabled()}",
                "{right.title}"
            }
        }
    ))
}

#[inline_props]
fn InstanceList<'a>(cx: Scope<'a>, view_store: &'a ViewStore<'a>) -> Element<'a> {
    cx.render(rsx!(
        div { class: "login-instance-list scroll",
            { view_store.instances.iter().map(|x| rsx!(
            InstanceView {
                image: x.thumbnail.as_deref(),
                name: &x.name,
                users: &x.users,
                selected: Some(x) == view_store.selected_instance.as_ref(),
                onclick: move |_| {
                    view_store.send(LoginAction::SelectInstance(Selection::Instance(x.clone())))
                }
            }
        )) }
        }
    ))
}

#[inline_props]
fn InstanceView<'a>(
    cx: Scope<'a>,
    image: Option<Option<&'a str>>,
    name: &'a str,
    users: &'a str,
    selected: bool,
    onclick: EventHandler<'a, ()>,
) -> Element<'a> {
    // FIXME: Take locale into account
    use numfmt::*;
    let mut f = Formatter::new() // start with blank representation
        .separator(',')
        .unwrap()
        .precision(Precision::Decimals(0));
    let value: f64 = users.parse().unwrap_or_default();
    let rendered = f.fmt(value);
    let class = selected.then(|| "selected").unwrap_or_default();
    cx.render(rsx!(
        div { onclick: move |_| onclick.call(()),
            HStack { class: "gap-3 p-3 login-instance {class} align-items-center",
                { image.flatten().map(|img| rsx!(img {
                width: 32,
                height: 32,
                src: "{img}"
            })).unwrap_or_else(|| rsx!(div {
                class: "no-img"
            }))},
                Label { onclick: move |_| onclick.call(()), clickable: true, pointer_style: PointerStyle::Pointer, class: "me-auto", "https://{name}" }
                Label { clickable: true, onclick: move |_| onclick.call(()), pointer_style: PointerStyle::Pointer, style: TextStyle::Secondary, "{rendered} Users" }
            }
        }
    ))
}

#[inline_props]
fn Page2<'a>(
    cx: Scope<'a>,
    visibility: PageVisibility,
    view_store: &'a ViewStore<'a>,
    code: &'a UseState<String>,
) -> Element<'a> {
    use copypasta::ClipboardProvider;
    let class = visibility.class();
    cx.render(rsx!(
        VStack { class: "page2 page gap-3 {class}",
            div { class: "p-2",
                Paragraph { loc!("A website should just have opened in your browser.") }
                Paragraph { style: TextStyle::Secondary, loc!("Please authorize Ebou and then copy & paste the code into the box below.") }
            }
            HStack { class: "gap-2 align-items-center",
                input {
                    r#type: "text",
                    class: "grow",
                    placeholder: "Code",
                    autocomplete: "off",
                    spellcheck: "false",
                    value: "{code}",
                    oninput: move |evt| {
                        let value = evt.value.clone();
                        code.set(value);
                    }
                }
                button {
                    class: "paste-button",
                    onclick: move |_| {
                        if let Ok(mut ctx) = copypasta::ClipboardContext::new() {
                            if let Ok(s) = ctx.get_contents() {
                                code.set(s);
                            }
                        }
                    },
                    loc!("Paste")
                }
            }
            small { class: "p-2 label-tertiary",
                a {
                    style: "text-decoration: underline; cursor: pointer;",
                    onclick: move |_| {
                        if let Some(url) = view_store.app_data.as_ref().and_then(|e| e.url.clone()) {
                             _ = match copypasta::ClipboardContext::new() {
                                Ok(mut ctx) => ctx.set_contents(url).map_or_else(
                                    |e| Some(format!("could not copy url: {e:?}")),
                                    |_| None),
                                Err(e) => Some(format!("could not access clipboard: {e:?}"))
                            };
                        }
                    },
                    loc!("Copy the browser URL to the clipboard")
                }
            }
        }
    ))
}

#[inline_props]
fn Page3<'a>(
    cx: Scope<'a>,
    visibility: PageVisibility,
    view_store: &'a ViewStore<'a>,
) -> Element<'a> {
    let name = view_store
        .account
        .as_ref()
        .map(|a| a.display_name.clone())
        .unwrap_or_default();
    let class = visibility.class();
    cx.render(rsx!(
        VStack { class: "page3 page gap-3 {class}",
            div { class: "p-2",
                h5 { "👋 {name}" }
                Paragraph { loc!("Ebou is still a very early alpha. Expect bugs and missing features.") }
                Paragraph { style: TextStyle::Secondary, loc!("You can report feedback by sending me a private message.") }
                (!view_store.did_follow).then(|| rsx!(
                button {
                    onclick: move |_| view_store.send(LoginAction::ActionFollow),
                    "Follow me (@terhechte@mastodon.social)",
                }
            )),

                Paragraph { style: TextStyle::Secondary, loc!("One Tip: Tap a selection in the left column twice, to scroll to the timeline bottom") }
            }
        }
    ))
}
