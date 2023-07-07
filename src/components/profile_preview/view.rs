use std::borrow::Cow;

use super::reducer::{ProfileAction, ViewStore};
use crate::PublicAction;
use crate::{
    environment::menu::{self, ViewStoreContextMenu},
    loc,
    view_model::AccountViewModel,
    widgets::*,
};
use dioxus::prelude::*;

/// A profile in the follows / followers list
#[inline_props]
pub fn ListProfileComponent<'a>(
    cx: Scope<'a>,
    store: ViewStore<'a>,
    onclick: EventHandler<'a, ()>,
    children: Element<'a>,
) -> Element<'a> {
    let account = &store.account;
    let (image_class, container_class) = ("image-author-big", "show");

    render!(
        VStack { class: "profile-preview profile-preview-style p-2 {container_class} grow enable-pointer-events",
            div {
                onclick: |_| {
                    onclick.call(());
                },
                class: "align-items-center hstack",
                img {
                    class: "{image_class} me-2 force-pointer",
                    src: "{account.image}",
                    alt: "{account.display_name} {account.username}",
                }
                Label {
                    class: "me-2 force-pointer",
                    vertical_alignment: VerticalTextAlign::Middle,
                    "{account.username}"
                }
                Label {
                    class: "me-auto force-pointer",
                    vertical_alignment: VerticalTextAlign::Middle,
                    style: TextStyle::Tertiary,
                    dangerous_content: "{account.display_name_html}"
                }
                children
            }
            HiddenProfileContent { store: store, account: account, show_links_small: true }
        }
    )
}

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq)]
#[repr(u64)]
pub enum SelectedProfileTab {
    #[default]
    Posts,
    Following,
    Followers,
}

impl std::fmt::Display for SelectedProfileTab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SelectedProfileTab::Posts => write!(f, loc!("Posts")),
            SelectedProfileTab::Following => write!(f, loc!("Following")),
            SelectedProfileTab::Followers => write!(f, loc!("Followers")),
        }
    }
}

#[derive(Copy, Clone, Default, Eq, PartialEq)]
struct ProfileTab {
    kind: SelectedProfileTab,
    content: u32,
    selected: bool,
    dot: bool,
}

impl ProfileTab {
    fn new(kind: SelectedProfileTab, content: u32, selected: &SelectedProfileTab) -> Self {
        ProfileTab {
            kind,
            content,
            selected: &kind == selected,
            dot: false,
        }
    }
}

impl Segment for ProfileTab {
    fn id(&self) -> u64 {
        self.kind as u64
    }

    fn label(&self) -> String {
        format!("{} {}", self.content, self.kind)
    }

    fn selected(&self) -> bool {
        self.selected
    }

    fn dot(&self) -> bool {
        self.dot
    }
}

/// The header profile when opening a users profile
#[inline_props]
pub fn ProfilePageHeader<'a>(
    cx: Scope<'a>,
    store: ViewStore<'a>,
    selected: SelectedProfileTab,
    onclick: EventHandler<'a, SelectedProfileTab>,
) -> Element<'a> {
    let account = &store.account;
    let (image_class, container_class) = ("image-author-big", "show");

    let full_bio = use_state(cx, || true);
    let items = vec![
        ProfileTab::new(SelectedProfileTab::Posts, account.statuses, selected),
        ProfileTab::new(SelectedProfileTab::Following, account.following, selected),
        ProfileTab::new(SelectedProfileTab::Followers, account.followers, selected),
    ];

    render!(
        VStack { class: "profile-preview profile-preview-style p-2 {container_class} grow enable-pointer-events",
            div {
                class: "align-items-center hstack",
                img {
                    class: "{image_class} me-2 force-pointer",
                    src: "{account.image}",
                    alt: "{account.display_name} {account.username}",
                }
                Label {
                    class: "me-2 force-pointer",
                    vertical_alignment: VerticalTextAlign::Middle,
                    "{account.username}"
                }
                Label {
                    class: "me-auto force-pointer",
                    vertical_alignment: VerticalTextAlign::Middle,
                    style: TextStyle::Tertiary,
                    dangerous_content: "{account.display_name_html}"
                }
                FollowInformation {
                    store: store
                }
                ProfileActions {
                    store: store,
                    account: account,
                    show_open: false
                }
            }
            if *full_bio.get() {
                rsx!(
                    div {
                        onclick: move |_| full_bio.set(false),
                        ProfileBio {
                            store: store
                        }
                    }
                )
            } else {
                rsx!(
                    div {
                        style: "height: 30px; overflow: clip;",
                        onclick: move |_| full_bio.set(true),
                        ProfileBio {
                            store: store
                        }
                    }
                )
            }
            SegmentedControl {
                items: items,
                onclick: move |i: ProfileTab| onclick.call(i.kind)
            }
        }
    )
}

#[inline_props]
pub fn ProfileComponent<'a>(
    cx: Scope<'a>,
    store: ViewStore<'a>,
    children: Element<'a>,
) -> Element<'a> {
    let account = &store.account;
    let expanded = use_state(cx, || false);
    let current = *expanded.get();
    let (image_class, container_class) = if current {
        ("image-author-small", "show")
    } else {
        ("image-author-small", "force-single-line")
    };
    render!(
        VStack { class: "profile-preview profile-preview-style p-2 {container_class} grow enable-pointer-events",
            HStack { class: "align-items-center",
                img {
                    class: "{image_class} me-2 force-pointer",
                    src: "{account.image}",
                    alt: "{account.display_name} {account.username}",
                    onclick: move |_| {
                        if !current && !store.is_loading {
                            store.send(ProfileAction::LoadRelationship);
                        }
                        expanded.set(!current);
                    }
                }
                Label {
                    class: "me-2 force-pointer",
                    onclick: move |_| {
                        store.send(ProfileAction::LoadRelationship);
                        expanded.set(!current);
                    },
                    vertical_alignment: VerticalTextAlign::Middle,
                    force_singleline: true,
                    clickable: true,
                    "{account.username}"
                }
                Label {
                    class: "me-auto force-pointer",
                    onclick: move |_| {
                        store.send(ProfileAction::LoadRelationship);
                        expanded.set(!current);
                    },
                    vertical_alignment: VerticalTextAlign::Middle,
                    style: TextStyle::Tertiary,
                    force_singleline: true,
                    clickable: true,
                    dangerous_content: "{account.display_name_html}"
                }
                children
            }
            HiddenProfileContent { store: store, account: account, show_links_small: true }
        }
    )
}

#[inline_props]
/// A profile component for the account timeline
pub fn FollowProfileComponent<'a>(
    cx: Scope<'a>,
    store: ViewStore<'a>,
    class: &'a str,
    is_expanded: bool,
    children: Element<'a>,
) -> Element<'a> {
    let account = &store.account;
    let expanded = use_state(cx, || *is_expanded);
    let current = *expanded.get();
    let container_class = if current { "show" } else { "" };
    let favorited_class = if store.is_favorite {
        "favorite-active"
    } else {
        "favorite-inactive"
    };

    render! {
        div { class: "vstack profile-preview {class} {container_class} p2 enable-pointer-events",
            HStack { class: "align-items-center",
                Label {
                    class: "me-auto force-pointer",
                    clickable: true,
                    title: "{account.acct}",
                    onclick: move |_| {
                        expanded.set(!current);
                    },
                    dangerous_content: "{account.display_name_html}"
                }
                IconButton {
                    icon: crate::icons::ICON_LIKE_ACCOUNT,
                    class: favorited_class,
                    title: loc!("Mark this account as one you'd like to see at the top"),
                    onclick: move |_| { store.send(ProfileAction::ToggleFavourite) }
                }
                IconButton {
                    icon: crate::icons::ICON_PROFILE,
                    class: "m-1",
                    title: loc!("Open Profile"),
                    onclick: move |_| {
                        store.send(ProfileAction::Public(PublicAction::OpenProfile(store.account.clone())))
                    },
                }
                children
            }
            HiddenProfileContent { store: store, account: account, show_links_small: false }
        }
    }
}

fn send_public(store: &ViewStore, action: crate::PublicAction) {
    store.send(ProfileAction::Public(action));
}

#[inline_props]
fn HiddenProfileContent<'a>(
    cx: Scope<'a>,
    store: &'a ViewStore<'a>,
    account: &'a AccountViewModel,
    show_links_small: bool,
) -> Element<'a> {
    let has_error = store.error.is_some();
    render! {
        ProfileBio {
            store: store
        }

        (!show_links_small && !account.fields.is_empty()).then(|| rsx!(VStack {
            class: "profile-fields profile-preview-addition-flex",
            account.fields.iter().map(|field| rsx! {
                HStack {
                    class: "align-items-center",
                    field.verified_at.map(|v| rsx!(Icon {
                        class: "verified-checkmark me-2",
                        icon: crate::icons::ICON_CHECKMARK,
                        title: format!("Verified on {v}"),
                    }))
                    Label {
                        style: TextStyle::Tertiary,
                        class: "me-auto",
                        "{field.name}"
                    }
                    field.link.as_ref().map(|link| rsx!(a {
                        href: "{link.to_string()}",
                        title: "field.value_parsed",
                        "{field.value_parsed}"
                    })).unwrap_or_else(|| rsx!( Label {
                        "{field.value_parsed}"
                    }))
                }
            })
        })),

        {store.error.as_ref().map(|e| rsx!(
            ErrorBox {
                content: format!("Could not load relationship:\n{e}"),
                onclick: move |_| store.send(ProfileAction::LoadRelationship)
            }
        ))},

        {(!has_error).then(|| rsx!(
            VStack {
                class: "profile-preview-addition-flex",
                HStack {
                    class: "mb-2 align-items-center wrap profile-actions",
                    HStack {
                        class: "me-auto no-selection profile-action-all",
                        Label {
                            class: "me-1",
                            style: TextStyle::Primary,
                            "{account.statuses_str}"
                        }
                        Label {
                            class: "me-3 profile-action-posts",
                            style: TextStyle::Tertiary,
                            loc!("Posts")
                        }
                        Label {
                            class: "me-1",
                            style: TextStyle::Primary,
                            "{account.following_str}"
                        }
                        Label {
                            class: "me-3 profile-action-following",
                            style: TextStyle::Tertiary,
                            loc!("Following")
                        }
                        Label {
                            class: "me-1",
                            style: TextStyle::Primary,
                            "{account.followers_str}"
                        }
                        Label {
                            class: "me-auto profile-action-followers",
                            style: TextStyle::Tertiary,
                            loc!("Followers")
                        }
                    }
                    FollowInformation {
                        store: store
                    }
                    show_links_small.then(|| rsx!(ProfileActions {
                        store: store,
                        account: account,
                        show_open: true
                    }))
                }
            }
        ))}
    }
}

#[inline_props]
fn ProfileBio<'a>(cx: Scope<'a>, store: &'a ViewStore<'a>) -> Element<'a> {
    render! {
        Paragraph { class: "profile-bio profile-preview-addition", style: TextStyle::Tertiary,
            TextContent {
                content: Cow::from(&store.account.note_html),
                onclick: move |action| match action {
                    TextContentAction::Tag(tag) => send_public(store, PublicAction::OpenTag(tag)),
                    TextContentAction::Link(link) => {
                        send_public(store, PublicAction::OpenLink(link))
                    }
                    TextContentAction::Account(link) => {
                        send_public(store, PublicAction::OpenLink(link))
                    }
                },
                class: ""
            }
        }
    }
}

#[inline_props]
fn FollowInformation<'a>(cx: Scope<'a>, store: &'a ViewStore<'a>) -> Element<'a> {
    let loading = store.is_loading;
    let follow_icon = if store.following {
        crate::icons::ICON_UNFOLLOW
    } else {
        crate::icons::ICON_FOLLOW
    };
    render! {
        (!loading).then(|| rsx!{
            store.followed_by.then(|| rsx!(div {
                class: "profile-follows-back ms-2",
                title: loc!("This account is following you"),
                dangerous_inner_html: crate::icons::ICON_FOLLOWBACK
            }))

            IconButton {
                icon: follow_icon,
                class: "m-1",
                title: if store.following { loc!("You're following this account") } else {
                    loc!("You're not following this account")
                } ,
                onclick: move |_| {
                    store.send(ProfileAction::ToggleFollow)
                },
            }
        }).unwrap_or_else(|| rsx!(Spinner {}))
    }
}

#[inline_props]
fn ProfileActions<'a>(
    cx: Scope<'a>,
    store: &'a ViewStore<'a>,
    account: &'a AccountViewModel,
    show_open: bool,
) -> Element<'a> {
    render! {
        if *show_open {
            rsx!(IconButton {
                icon: crate::icons::ICON_PROFILE,
                class: "m-1",
                title: loc!("Open Profile"),
                onclick: move |_| {
                    store.send(ProfileAction::Public(PublicAction::OpenProfile(store.account.clone())))
                },
            })
        }
        IconButton {
            icon: crate::icons::ICON_MORE,
            class: "ms-2",
            title: "Profile Actions",
            onclick: move |e: Event<MouseData>| {
                use ProfileAction::Public;
                use PublicAction::{OpenLink, Copy, OpenProfile};
                let cloned = (*account).clone();

                // build up the menu
                let items: Vec<menu::ContextMenuItem> = account.fields.iter().filter_map(|item| {
                    item.link.as_ref().map(|url| menu::ContextMenuItem::item(&item.name, Public(OpenLink(url.to_string()))))
                }).collect();

                let mut menu = vec![
                    menu::ContextMenuItem::item("Open in Browser", Public(OpenLink(account.url.clone()))),
                    menu::ContextMenuItem::item("Copy", Public(Copy(account.url.clone()))),
                ];
                if *show_open {
                    menu.insert(0, menu::ContextMenuItem::separator());
                    menu.insert(0, menu::ContextMenuItem::item("Open", Public(OpenProfile(cloned))));
                }
                if !items.is_empty() {
                    menu.push(menu::ContextMenuItem::submenu("Links", items));
                }
                store.context_menu(
                    cx,
                    &e.data, menu::ContextMenu::<ProfileAction>::new("Actions", true, menu))
            }
        }
    }
}
