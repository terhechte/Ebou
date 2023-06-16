#![allow(non_snake_case)]

use crate::view_model::AttachmentMedia;
use crate::{loc, widgets::*};

use dioxus::prelude::*;

use super::{PostAction, ViewStore};

#[inline_props]
pub fn PostView<'a>(cx: Scope<'a>, store: ViewStore<'a>) -> Element<'a> {
    let text = &store.text;
    let mut is_posting_class = if store.posting { "true" } else { "false" };

    let is_dropping_file = store.dropping_file;

    if is_dropping_file {
        is_posting_class = "true";
    }

    // FIXME: Hack
    crate::environment::platform::execute_js_once(
        &cx,
        r#"
        window.setTimeout(() => {{
            let element = document.getElementById("text-area");
            element.selectionStart = element.value.length;
        }}, 150);
    "#,
    );

    cx.render(rsx!(
        div { class: "posting-window",
            { is_dropping_file.then(|| rsx!(div {
                class: "fullscreen file-drop-box"
        }))},
            VStack { class: "width-100",
                ToolbarView {
                    store: store
                }
                textarea {
                    id: "text-area",
                    disabled: is_posting_class,
                    placeholder: "Enter Text...",
                    oninput: move |evt| {
                        store.send(PostAction::UpdateText(evt.value.clone()));
                    },
                    autofocus: "true",
                    "{text}"
                }
                { store.error_message.as_ref().map(|error| rsx!(ErrorBox {
                content: error.clone(),
                onclick: move |_| {
                    store.send(PostAction::ClearError);
                }
            })) },
                {
                if !store.validity.0 {
                    rsx!(Label {
                        class: "char-count over",
                        "{store.validity.1} / {store.validity.2}"
                    })
                } else {
                    rsx!(Label {
                        class: "char-count",
                        "{store.validity.1} / {store.validity.2}"
                    })
                }
            },
                ImagesView { store: store }
            }
        }
    ))
}

#[inline_props]
fn ToolbarView<'a>(cx: Scope<'a>, store: &'a ViewStore<'a>) -> Element<'a> {
    let mut is_posting_class = if store.posting { "true" } else { "false" };

    // can't post if there're still images being uploaded
    let are_images_uploading = store.images.iter().any(|i| i.server_id.is_none());
    if are_images_uploading {
        is_posting_class = "true";
    }

    let current = store
        .visibility
        .unwrap_or(super::Visibility::Public);
    let is_direct = current == super::Visibility::Direct;

    cx.render(rsx!(
        HStack { class: "p-1 justify-content-between align-items-center posting-toolbar",
            (!store.is_window).then(|| rsx! {
                button {
                    class: "button me-2",
                    onclick: move |_| store.send(PostAction::Close),
                    loc!("Cancel")
                }
            })
            EmojiButton {}
            { store.posting.then(|| rsx!{
            span {
                class: "ms-auto"
            }
            Spinner {}
            span {
                class: "ms-auto"
            }
        })},
            span { class: "me-auto" }
            select {
                name: "visibility",
                class: "me-3",
                onchange: move |evt| {
                    store.send(PostAction::UpdateVisibility(evt.value.clone()));
                },
                option {
                    value: "public", "Public: Visible for all",
                }
                option {
                    value: "unlisted", "Unlisted"
                }
                option {
                    value: "private", "Followers only"
                }
                option {
                    selected: "{is_direct}",
                    value: "direct", "Mentioned people only",
                }
            }
            button {
                class: "button me-2 highlighted",
                disabled: is_posting_class,
                onclick: move |_| store.send(PostAction::Post),
                loc!("Toot")
            }
        }
    ))
}

#[inline_props]
fn ImagesView<'a>(cx: Scope<'a>, store: &'a ViewStore<'a>) -> Element<'a> {
    if store.images.is_empty() {
        return cx.render(rsx!(div {}));
    }
    cx.render(rsx!(
        VStack { class: "images",
            store.images.iter().enumerate().map(|(index, image)| {
            cx.render(rsx!(SingleImageView {
                store: store,
                index: index,
                image: image.clone()
            }))
        })
        }
    ))
}

#[inline_props]
fn SingleImageView<'a>(
    cx: Scope<'a>,
    store: &'a ViewStore<'a>,
    index: usize,
    image: AttachmentMedia,
) -> Element<'a> {
    let is_editing = use_state(cx, || false);
    let text = use_ref(cx, String::new);
    let is_uploaded = image.server_id.is_some();
    cx.render(rsx!(
        div {
            HStack { class: "p-2 align-items-center gap-2",
                { image.preview.as_ref().map(|preview| rsx!(img {
                    style: "object-fit: cover",
                    class: "preview-image",
                    src: "{preview}"
                }))},
                { if *is_editing.get() && is_uploaded {rsx!{
                    input {
                        class: "grow",
                        placeholder: "Enter Description…",
                        value: "{text.read()}",
                        oninput: move |evt| {
                            *text.write_silent() = evt.value.clone();
                        },
                        autofocus: "true",
                    }
                    IconButton {
                        icon: crate::icons::ICON_OK,
                        title: "Save",
                        onclick: move |_| {
                            is_editing.set(false);
                            let value = text.read().clone();
                            store.send(PostAction::UpdateImageDescription(*index, value));
                        }
                    }
                    IconButton {
                        icon: crate::icons::ICON_CANCEL,
                        title: "Cancel",
                        onclick: move |_| {
                            is_editing.set(false);
                        }
                    }
                }} else {rsx!{
                    span {
                        class: "label-secondary",
                        "{image.filename}"
                    }
                    span {
                        class: "label-tertiary me-auto overflow-y-hidden",
                        "{text.read()}"
                    }
                    { is_uploaded.then(|| rsx!(
                        IconButton {
                            icon: crate::icons::ICON_EDIT_CAPTION,
                            title: "Edit Description",
                            onclick: move |_| {
                                is_editing.set(true);
                            }
                        }
                    ))}
                    { (!is_uploaded).then(|| rsx!(
                        Spinner {
                            class: "mt-1 me-3"
                        }
                    ))}
                    IconButton {
                        icon: crate::icons::ICON_INFO,
                        title: "Show on Disk",
                        onclick: move |_| {
                            store.send(PostAction::ShowImageDisk(*index))
                        }
                    }
                    IconButton {
                        icon: crate::icons::ICON_DELETE,
                        title: "Remove",
                        onclick: move |_| {
                            store.send(PostAction::RemoveImage(*index))
                        }
                    }
                }}}
            }
        }
    ))
}
