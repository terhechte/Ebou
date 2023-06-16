use std::borrow::Cow;

use crate::components::profile_preview::ProfileComponent;
use crate::components::profile_preview::ProfilePreviewReducer;
use crate::components::profile_preview::ProfileState;
use crate::environment::menu::ViewStoreContextMenu;
use crate::view_model::*;
use crate::widgets::*;
use crate::{icons, loc};
use dioxus::prelude::*;
use navicula::reducer::ChildReducer;

use super::ConversationReducer;
use super::ViewStore;
use super::{
    conversation_helpers::{Conversation, ConversationItem},
    reducer::Action,
};
use crate::environment::menu::{self};
use crate::PublicAction;

#[inline_props]
pub fn ConversationComponent<'a>(cx: Scope<'a>, store: ViewStore<'a>) -> Element<'a> {
    if store.is_loading {
        return cx.render(rsx!(
            div { class: "vstack p-3 m-3 align-items-center grow justify-items-center", Spinner {} }
        ));
    }

    let Some(conversation) = &store.conversation else {
        return cx.render(rsx!(ErrorPage {
            content: "Unknown Conversation"
        }))
    };

    let Some(root) = conversation.root() else {
        return cx.render(rsx!(ErrorPage {
            content: "Unknown Conversation Root"
        }))
    };

    let children = conversation.children(&root).unwrap_or_default();
    let cloned_status = root.cloned_status();
    let cloned_account = cloned_status.account.clone();

    let (a, b) = (
        cloned_status.created_full.clone(),
        cloned_status.created_human.clone(),
    );
    let time = rsx!(
        Label {
            class: "time",
            style: TextStyle::Tertiary,
            alignment: TextAlign::Right,
            title: "{a}",
            "{b}"
        }
    );

    let ctx_status = root.cloned_status();

    cx.render(rsx!(
        VStack { class: "content grow",
            UserConversationHeader { status: cloned_status, store: store }
            div { class: "conversation-container scroll",
                div { class: "content-cell no-selection conversation-ancestor",
                    ProfileComponent {
                        store: store.host_with(
                            cx,
                            &cloned_account,
                            |account| ProfileState::new(account, false),
                        )
                        time
                    }

                    StatusComponent {
                        status: root.cloned_status(),
                        is_in_conversation: true,
                        onclick: move |action| {
                            store.send(Action::Public((action, root.cloned_status()).into()))
                        },
                        sender: store.sender_fn(move |a: StatusAction| {
                            Action::Public((a, ctx_status.clone()).into())
                        })
                        ""
                    }
                }
                UserConversationComponentChildren { conversation: conversation, store: store, children: children }
            }
        }
    ))
}

#[inline_props]
fn UserConversationHeader<'a>(
    cx: Scope<'a>,
    status: StatusViewModel,
    store: &'a ViewStore<'a>,
) -> Element<'a> {
    cx.render(rsx!(
        div {
            HStack { class: "toolbar justify-content-between justify-items-center p-2 grow align-items-center",
                div { class: "icon-button",
                    button {
                        r#type: "button",
                        onclick: move |_| { store.send(Action::Close) },
                        dangerous_inner_html: icons::ICON_CANCEL
                    }
                }
                div { class: "me-auto p-1 no-selection",
                    Label { style: TextStyle::Primary, "Conversation" }
                }
                div { class: "icon-button",
                    button {
                        r#type: "button",
                        onclick: move |evt| {
                            use crate::PublicAction::*;
                            store.context_menu(cx, &evt, menu::ContextMenu::<Action>::new(
                                    "Conversation Options",
                                    true,
                                    vec![
                                        menu::ContextMenuItem::item(
                                            "Open in Browser",
                                            Action::Public(OpenLink(status.uri.clone()))),
                                        menu::ContextMenuItem::item(
                                            "Copy URL", 
                                            Action::Public(Copy(status.uri.clone()))
                                        )
                                    ],
                                ),
                            )
                        },
                        dangerous_inner_html: icons::ICON_MORE
                    }
                }
            }
        }
    ))
}

#[inline_props]
fn UserConversationComponentChild<'a>(
    cx: Scope<'a>,
    conversation: &'a Conversation,
    store: &'a ViewStore<'a>,
    child: Cow<'a, ConversationItem<'a>>,
) -> Element<'a> {
    use crate::components::profile_preview::{ProfileComponent, ProfileState};
    let children = conversation.children(child).unwrap_or_default();
    let cls = if children.is_empty() {
        ""
    } else {
        "has-children"
    };

    let is_selected = (store.conversation_id == child.id)
        .then_some("conversation-child-selected")
        .unwrap_or_default();

    let id = child.id.dom_id();

    let message = cx.render(rsx!(
        div { class: "conversation-child {cls} {is_selected}",
            div { id: "conv-{id}", class: "optionbox",
                IconButton {
                    icon: icons::ICON_OPTIONS,
                    title: loc!("Options"),
                    onclick: move |e: Event<MouseData>| {
                        use crate::widgets::StatusAction;
                        store.context_menu(
                            cx,
                            &e.data,
                            menu::ContextMenu::<Action>::new(
                                loc!("Post Options"),
                                true,
                                vec![
                                    menu::ContextMenuItem::item(
                                        loc!("Reply"),
                                        Action::Public((StatusAction::Reply, child.cloned_status()).into())
                                    ),
                                    child.is_reblogged(|reb, _| {
                                        let title = if reb { loc!("Unboost") } else { loc!("Boost") };
                                        menu::ContextMenuItem::item(
                                            title,
                                            Action::Public((StatusAction::Boost(!reb), child.cloned_status()).into())
                                        )
                                    }),
                                    child.is_favourited(|reb, _| {
                                        let title = if reb {
                                            loc!("Unfavourite")
                                        } else {
                                            loc!("Favourite")
                                        };
                                        menu::ContextMenuItem::item(
                                            title,
                                            Action::Public((StatusAction::Favorite(!reb), child.cloned_status()).into())
                                        )
                                    }),
                                    child.is_bookmarked(|reb, _| {
                                        let title = if reb {
                                            loc!("Unbookmark")
                                        } else {
                                            loc!("Bookmark")
                                        };
                                        menu::ContextMenuItem::item(
                                            title,
                                            Action::Public((StatusAction::Bookmark(!reb), child.cloned_status()).into())
                                        )
                                    }),
                                    menu::ContextMenuItem::separator(),
                                    menu::ContextMenuItem::item(loc!("Copy Link"),
                                        Action::Public(PublicAction::Copy(child.uri.clone()))
                                    ),
                                    menu::ContextMenuItem::item(loc!("Open in Browser"), 
                                        Action::Public(PublicAction::OpenLink(child.uri.clone()))
                                    ),
                                ]
                            )
                        )
                    }
                }
            }
            ProfileComponent {
                store: store.host_with(
                    cx,
                    &child.account,
                    |account| ProfileState::new(account, false),
                )
            }
            TextContent {
                content: Cow::from(child.content.clone()),
                onclick: move |action| match action {
                    TextContentAction::Tag(tag) => {
                        store.send(Action::Public(PublicAction::OpenTag(tag)))
                    }
                    TextContentAction::Link(link) => {
                        store.send(Action::Public(PublicAction::OpenLink(link)))
                    }
                    TextContentAction::Account(link) => {
                        store.send(Action::Public(PublicAction::OpenLink(link)))
                    }
                },
                class: ""
            }

            { child.status_images.iter().map(|(description, preview, url)| rsx!(div {
            class: "media-object",
            img {
                src: "{preview}",
                alt: "{description}",
                onclick: move |_| store.send(Action::Public(PublicAction::OpenImage(url.to_string()))),
            }
        }))},

            { child.media.iter().map(|media| if let Some(preview) = media.preview_url.as_ref() {
            rsx!(div {
                class: "media-object",
                img {
                    src: "{preview}",
                    alt: "{media.description}",
                    onclick: move |_| store.send(Action::Public(PublicAction::OpenVideo(media.video_url.clone()))),
                }
            })
        } else {
            rsx!(div {
                class: "media-object",
                span {
                    class: "empty label-secondary",
                    title: "{media.description}",
                    onclick: move |_| store.send(Action::Public(PublicAction::OpenVideo(media.video_url.clone()))),
                    "Video"
                }
            })
        })}
        }
    ));

    cx.render(rsx!(
        div {
            message,
            UserConversationComponentChildren { conversation: conversation, store: store, children: children }
        }
    ))
}

impl ChildReducer<ConversationReducer> for ProfilePreviewReducer {
    fn to_child(
        _message: <ConversationReducer as navicula::reducer::Reducer>::Message,
    ) -> Option<<Self as navicula::reducer::Reducer>::Action> {
        None
    }

    fn from_child(
        message: <Self as navicula::reducer::Reducer>::DelegateMessage,
    ) -> Option<<ConversationReducer as navicula::reducer::Reducer>::Action> {
        Some(Action::Public(message))
    }
}

#[inline_props]
fn UserConversationComponentChildren<'a>(
    cx: Scope<'a>,
    conversation: &'a Conversation,
    store: &'a ViewStore<'a>,
    children: Vec<ConversationItem<'a>>,
) -> Element<'a> {
    let hidden = use_state(cx, || false);
    let is_hidden = *hidden.get();
    let ln = children.len();
    let cls = if children.is_empty() {
        ""
    } else {
        "has-children"
    };

    let content = if is_hidden {
        rsx!(
            div { class: "hidden-content", onclick: move |_| hidden.set(!*hidden.get()),
                Label { "{ln} More" }
            }
        )
    } else {
        rsx! {
            children.iter().map(|child| {
                rsx!(UserConversationComponentChild {
                    conversation: conversation,
                    store: store,
                    child: Cow::Borrowed(child),
                })
            }),
            div { class: "sideline", onclick: move |_| hidden.set(!*hidden.get()) }
        }
    };

    cx.render(rsx!(div {
        class: "conversation-children {cls}",
        content
    }))
}
