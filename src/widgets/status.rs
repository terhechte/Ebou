use std::borrow::Cow;
use std::sync::Arc;

use navicula::types::AppWindow;

use crate::environment::menu::context_menu;
use crate::icons;
use crate::widgets::*;

use crate::environment::menu::{self};
use crate::loc;
use crate::view_model::StatusViewModel;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum StatusAction {
    Clicked, // e.g. open conversation
    OpenTag(String),
    OpenLink(String),
    OpenAccount(String),
    Reply,
    Boost(bool),
    Favorite(bool),
    Bookmark(bool),
    OpenImage(String),
    OpenVideo(String),
    Copy(String),
}

#[inline_props]
pub fn StatusComponent<'a>(
    cx: Scope<'a>,
    status: StatusViewModel,
    is_in_conversation: Option<bool>,
    onclick: EventHandler<'a, StatusAction>,
    sender: Arc<dyn Fn(StatusAction) + Send + Sync>,
    children: Element<'a>,
) -> Element<'a> {
    let window = AppWindow::retrieve(cx);
    cx.render(rsx! {
        div {
            class: "enable-pointer-events",
            onclick: move |_| onclick.call(StatusAction::Clicked),
            children,
            TextContent {
                content: Cow::from(&status.content),
                onclick: move |action| match action {
                    TextContentAction::Tag(tag) => onclick.call(StatusAction::OpenTag(tag)),
                    TextContentAction::Link(link) => onclick.call(StatusAction::OpenLink(link)),
                    TextContentAction::Account(link) => onclick.call(StatusAction::OpenAccount(link)),
                },
                class: ""
            }
        }

        ContentCellMedia { 
            status: Cow::Borrowed(status),
            onclick: move |evt| onclick.call(evt),
            sender: sender.clone()
        }

        HStack { class: "justify-content-between m-2 gap-3 wrap enable-pointer-events",
            IconButton {
                icon: icons::ICON_REPLY,
                title: &status.replies_title,
                onclick: move |_| {
                    onclick.call(StatusAction::Reply);
                }
            }
            status.is_reblogged(|reb, icon| cx.render(rsx!(IconTextButton {
                icon: icon,
                title: &status.reblog_title,
                text: &status.reblog,
                onclick: move |_| {
                    onclick.call(StatusAction::Boost(!reb));
                }
            }))),
            status.is_favourited(|fav, icon| cx.render(rsx!(IconTextButton {
                icon: icon,
                title: &status.favourited_title,
                text: &status.favourited,
                onclick: move |_| {
                    onclick.call(StatusAction::Favorite(!fav));
                }
            }))),
            status.is_bookmarked(|bk, icon| cx.render(rsx!(IconButton {
                icon: icon,
                title: &status.bookmarked_title,
                onclick: move |_| {
                    onclick.call(StatusAction::Bookmark(!bk));
                }
            }))),
            IconButton {
                icon: icons::ICON_OPTIONS,
                title: loc!("Options"),
                onclick: move |e: Event<MouseData>| {
                    let mut items = vec![
                        menu::ContextMenuItem::item(
                            loc!("Copy Link"), 
                            StatusAction::Copy(status.uri .clone())
                        ),
                        menu::ContextMenuItem::item(
                            loc!("Open in Browser"),
                            StatusAction::OpenLink(status.uri.clone())
                        ),
                        menu::ContextMenuItem::item(
                            loc!("Copy Text"), 
                            StatusAction::Copy(status.text .clone())
                        ),
                    ];
                    if !is_in_conversation.unwrap_or_default() {
                        items.push(menu::ContextMenuItem::separator());
                        items
                            .push(
                                menu::ContextMenuItem::item(
                                    loc!("Open Conversation"),
                                    StatusAction::Clicked
                                ),
                            );
                    }
                    context_menu(cx, sender.clone(), window, &e.data, menu::ContextMenu::new(loc!("Post Options"), true, items))
                }
            }
        }
    })
}

#[inline_props]
fn ContentCellMedia<'a>(
    cx: Scope<'a>,
    status: Cow<'a, StatusViewModel>,
    onclick: EventHandler<'a, StatusAction>,
    sender: Arc<dyn Fn(StatusAction) + Send + Sync>,
) -> Element<'a> {
    let window = AppWindow::retrieve(cx);
    cx.render(rsx!(

        { status.status_images.iter().map(|(description, preview, url)| rsx!(div {
            class: "media-object",
            img {
                src: "{preview}",
                alt: "{description}",
                onclick: move |_| onclick.call(StatusAction::OpenImage(url.to_string())),
                prevent_default: "oncontextmenu",
                oncontextmenu: move |e| {
                    context_menu(cx, sender.clone(), window, &e.data, menu::ContextMenu::new(loc!("Image"), true, vec![
                        menu::ContextMenuItem::item(
                            loc!("Open"),
                            StatusAction::OpenImage(url.clone())
                        ),
                        menu::ContextMenuItem::item(
                            loc!("Open in Browser"),
                            StatusAction::OpenLink(url.clone())
                        ),
                        menu::ContextMenuItem::item(
                            loc!("Copy URL"),
                            StatusAction::Copy(url.clone())
                        ),
                    ]))
                },
            }
        }))},

        { status.media.iter().map(|video| {
            let preview = video.preview_url.as_ref().cloned().unwrap_or_default();
            rsx!(div {
                class: "enable-pointer-events",
                video {
                    // FIXME: MOVE TO CSS
                    style: "width: 448px;",
                    controls: "true",
                    poster: "{preview}",
                    source {
                        src: "{video.video_url}"
                    }
                }
                div {
                    class: "hstack justify-content-center mt-2",
                    IconTextButton {
                        icon: crate::icons::ICON_OPEN_WINDOW,
                        text: "Open in Window",
                        title: "Open in Window",
                        class: "mb-3",
                        disabled: false,
                        onclick: move |_| onclick.call(StatusAction::OpenVideo(video.video_url.to_string())),
                    },
                }
                Paragraph {
                    style: TextStyle::Tertiary,
                    class: "p-3",
                    "{video.description}"
                }
            })
        })},

        { status.card.as_ref().map(|card| {
            let mut desc = card.description.clone();
            if desc.len() > 300 {
                desc = desc.chars().take(300).collect();
                desc.push('…');
            }
            rsx!(div {
            class: "link-object",
            onclick: move |_| onclick.call(StatusAction::OpenLink(card.url.clone())),
            { card.image.as_ref().map(|image_url| rsx!(img {
                src: "{image_url}",
            }))}
            VStack {
                class: "me-auto gap-1",
                Label {
                    style: TextStyle::Primary,
                    onclick: move |_| onclick.call(StatusAction::OpenLink(card.url.clone())),
                    pointer_style: PointerStyle::Pointer,
                    "{card.title}"
                }
                Paragraph {
                    style: TextStyle::Secondary,
                    pointer_style: PointerStyle::Pointer,
                    "{desc}"
                }
            }
        })})}
    ))
}
