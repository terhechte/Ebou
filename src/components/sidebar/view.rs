use std::cmp::Ordering;

use super::reducer::{MoreSelection, SidebarAction, ViewStore};
use crate::{
    components::post::PostKind,
    environment::menu::{self, ViewStoreContextMenu},
    environment::{
        model::Account,
        storage::UiTab,
        types::{AppEvent, MainMenuEvent},
    },
    loc,
    view_model::{AccountUpdateViewModel, AccountViewModel},
    widgets::*,
};
use dioxus::{html::br, prelude::*};
use itertools::Itertools;

use crate::components::loggedin::Action;
use crate::PublicAction;

/// Custom sidebar navigation for iPadOS
#[inline_props]
pub fn SidebarNavigationComponent<'a>(cx: Scope<'a>, store: ViewStore<'a>) -> Element<'a> {
    log::trace!("Rerender SidebarComponent");
    let has_notifications = store.has_new_notifications;
    let has_messages = false; // FIXME
    let tab = store.active_tab;
    let tabs = vec![
        TabBarItem::new(
            UiTab::Timeline,
            loc!("Timelines").to_string(),
            tab.is_timeline(),
            false,
        ),
        TabBarItem::new(
            UiTab::Mentions,
            loc!("Mentions").to_string(),
            tab.is_mentions(),
            has_notifications,
        ),
        TabBarItem::new(
            UiTab::Messages,
            loc!("Messages").to_string(),
            tab.is_messages(),
            has_messages,
        ),
        TabBarItem::new(UiTab::More, loc!("More").to_string(), tab.is_more(), false),
    ];

    let icon = crate::icons::ICON_WRITE;

    render! {
        div {
            class: "side-navigation-bar",
            // first the user icon
            AccountImageComponent { store: store }
            // then the tabs
            // and at the bottom a pref button for the preferences
            div {
                br {}
            }
            tabs.into_iter().map(|btn| rsx!(SidebarNavButton { store: store, item: btn }))

            div {
                class: "mb-auto"
            }

            button {
                class: "sidebar-nav-button",
                margin_bottom: "80px",
                onclick: move |_| {
                    store.send(SidebarAction::Root(Action::AppEvent(AppEvent::MenuEvent(MainMenuEvent::NewPost))));
                },
                div {
                    class: "icon-button",
                    dangerous_inner_html: "{icon}"
                }
            }
        }
    }
}

#[inline_props]
fn SidebarNavButton<'a>(cx: Scope<'a>, store: &'a ViewStore<'a>, item: TabBarItem) -> Element<'a> {
    let icon = match item.id {
        UiTab::Timeline => crate::icons::ICON_PROFILE,
        UiTab::Mentions => crate::icons::ICON_RELOAD,
        UiTab::Messages => crate::icons::ICON_OPEN_WINDOW,
        UiTab::More => crate::icons::ICON_MORE,
    };
    let class = item.selected.then_some("selected").unwrap_or_default();
    let dot = item.dot.then(|| rsx!(span { class: "dot" }));
    cx.render(rsx!(
        button {
            class: "sidebar-nav-button {class}",
            onclick: move |_| {
                store.send(SidebarAction::ChangeTab(item.id));
            },
            div {
                class: "icon-button",
                dangerous_inner_html: "{icon}"
            }
            dot
        }
    ))
}

#[inline_props]
pub fn SidebarComponent<'a>(cx: Scope<'a>, store: ViewStore<'a>) -> Element<'a> {
    log::trace!("Rerender SidebarComponent");
    let has_notifications = store.has_new_notifications;
    let has_messages = false; // FIXME
    let tab = store.active_tab;
    let tabs = vec![
        TabBarItem::new(
            UiTab::Timeline,
            loc!("Timelines").to_string(),
            tab.is_timeline(),
            false,
        ),
        TabBarItem::new(
            UiTab::Mentions,
            loc!("Mentions").to_string(),
            tab.is_mentions(),
            has_notifications,
        ),
        TabBarItem::new(
            UiTab::Messages,
            loc!("Messages").to_string(),
            tab.is_messages(),
            has_messages,
        ),
        TabBarItem::new(UiTab::More, loc!("More").to_string(), tab.is_more(), false),
    ];

    #[cfg(target_os = "macos")]
    let is_not_macos = false;

    #[cfg(not(target_os = "macos"))]
    let is_not_macos = true;

    #[cfg(target_os = "ios")]
    let is_not_macos = false;

    let col_class = if store.current_section_selection() {
        "collapsed"
    } else {
        ""
    };

    let cloned_tabs = tabs.clone();
    cx.render(rsx! {
        VStack { class: "sidebar {col_class}",
            is_not_macos.then(|| rsx! {
                MenuComponent { store: store }

                TabBar {
                    items: cloned_tabs.clone(),
                    onclick: move |item: TabBarItem| { store.send(SidebarAction::ChangeTab(item.id)) }
                }
            })

            {
                if tab == tabs[0].id {
                    rsx!(SidebarAccountsComponent {
                        store: store,
                        // search_state: search_state
                    })
                } else if tab == tabs[1].id {
                    rsx!(SidebarNotificationsComponent {
                        store: store,
                        // search_state: search_state
                    })
                } else if tab == tabs[2].id {
                    rsx!(div {
                        class: "hstack p-3 m-3 grow align-self-center label-secondary",
                        "Not implemented yet"
                    })
                } else if tab == tabs[3].id {
                    rsx!(SidebarMoreComponent {
                        store: store
                    })
                } else {
                    rsx!({})
                }
            }
        }
    })
}

#[inline_props]
fn SidebarAccountsComponent<'a>(cx: Scope<'a>, store: &'a ViewStore<'a>) -> Element<'a> {
    log::trace!("SidebarAccountsComponent: {}", store.accounts.len());
    let selection = &store.selected_account;
    // let search_term = search_state.get();

    let is_loading = store.loading_content;

    let search_term = &store.search_term;

    // if we have search results, filter them against the known
    // accounts. we have to do this here (instead of the reducer),
    // because the
    let favorites = &store.favorites;

    // Can we load more for the selected list / timeline?
    let can_load_more = store.has_more();

    cx.render(rsx! {
        ListSelector { store: store }

        SearchComponent { placeholder: "Search", store: store }

        div { class: "scroll",
            div { class: "scroll-margin-fix",
                {
                store.accounts.iter()
                    .filter(|model| if search_term.is_empty() {
                        true
                    } else {
                        model.account.display_name.contains(search_term) ||
                        model.account.username.contains(search_term)
                    })
                    .sorted_by(|a, b| {
                        match (favorites.contains(&a.id.0), favorites.contains(&b.id.0)) {
                            (true, false) => Ordering::Less,
                            (false, true) => Ordering::Greater,
                            _ => Ordering::Equal
                        }
                    })
                    .map(move |model| rsx!(CellComponent {
                        model: model.clone(),
                        selected: selection.as_ref().map(|e| &e.id) == Some(&model.id),
                        store: store,
                        onclick: move |_| store.send(SidebarAction::SelectAccount(model.account.clone())),
                        favorited: favorites.contains(&model.id.0)
                    }))
                },

                {
                    (!store.search_results.is_empty()).then(|| rsx!(div {
                        class: "m-3",
                        Label {
                            style: TextStyle::Secondary,
                            loc!("More")
                        }
                    }))
                },

                store.search_results.iter().map(|account| {
                    rsx!(AccountCellComponent {
                        account: account.clone(),
                        selected: store.selected_account.as_ref().map(|e| e.id.0.as_str()) == Some(account.id.as_str()),
                        store: store,
                        onclick: move |_| store.send(SidebarAction::SelectAccount(AccountViewModel::new(account)))
                    })
                }),

                {
                    (search_term.is_empty() && !is_loading && !store.posts_empty && can_load_more)
                    .then(|| rsx!(div {
                        class: "hstack justify-content-center mt-2",
                        IconTextButton {
                            icon: crate::icons::ICON_LOAD_OLDER_TIMELINE,
                            text: "More",
                            title: "Load more timeline data",
                            class: "mb-3",
                            onclick: move |_| {
                                store.send(SidebarAction::LoadMoreTimeline);
                            },
                        },
                    }))
                }
            }
        }

        { (is_loading || store.is_searching).then(|| rsx!(div {
            class: "hstack p-2 m-2 grow align-self-center",
            Spinner {}
        }))}
    })
}

enum SidebarMoreEntry<'a> {
    More(MoreSelection),
    Title(&'a str),
}

#[inline_props]
fn SidebarMoreComponent<'a>(cx: Scope<'a>, store: &'a ViewStore<'a>) -> Element<'a> {
    let selection = store.more_selection;
    let structure = {
        use MoreSelection::*;
        use SidebarMoreEntry::*;
        &[
            Title(loc!("Timelines")),
            More(Classic),
            More(Local),
            More(Federated),
            //Title(loc!("Explore")),
            //More(Posts),
            //More(Hashtags),
            Title(loc!("Account")),
            More(Yours),
            More(Followers),
            More(Following),
            More(Bookmarks),
            More(Favorites),
        ]
    };

    cx.render(rsx! {
        div { class: "scroll",
            div {
                class: "scroll-margin-fix",
                VStack {
                    class: "p-3 gap-2",

                    for entry in structure {
                        match entry {
                            SidebarMoreEntry::Title(t) => rsx!(SidebarTextHeadline {
                                text: t,
                            }),
                            SidebarMoreEntry::More(m) => rsx!(SidebarTextEntry {
                                icon: m.content(),
                                text: m.title(),
                                selected: selection == *m,
                                onclick: move |_| store.send(SidebarAction::MoreSelection(*m))
                            })
                        }
                    }
                }
            }
        }
    })
}

#[inline_props]
fn SidebarNotificationsComponent<'a>(cx: Scope<'a>, store: &'a ViewStore<'a>) -> Element<'a> {
    let selection = &store.selected_notifications;
    let is_loading = store.loading_notifications;

    cx.render(rsx! {
        div { class: "scroll",
            div { class: "scroll-margin-fix",
                store.notification_accounts.iter()
                    // .filter(|account| if search_term.is_empty() {
                    //     true
                    // } else {
                    //     account.display_name.contains(search_term) ||
                    //     account.username.contains(search_term)
                    // })
                    .map(move |model| rsx!(CellComponent {
                        model: model.clone(),
                        store: store,
                        selected: selection.as_ref() == Some(&model.account),
                        onclick: move |_| store.send(SidebarAction::SelectedNotifications(model.account.clone())),
                        favorited: false
                    })),

                {
                    (/*search_term.is_empty() &&*/ !is_loading && !store.notification_posts_empty)
                    .then(|| rsx!(div {
                        class: "hstack justify-content-center mt-2",
                        IconTextButton {
                            icon: crate::icons::ICON_LOAD_OLDER_TIMELINE,
                            text: "More",
                            title: "Load more notification data",
                            class: "mb-3",
                            onclick: move |_| {
                                store.send(SidebarAction::LoadNotifications);
                            },
                        },
                    }))
                }
            }
        }

        { is_loading.then(|| rsx!(div {
            class: "hstack p-2 m-2 grow align-self-center",
            Spinner {}
        }))}
    })
}

#[inline_props]
fn AccountImageComponent<'a>(cx: Scope<'a>, store: &'a ViewStore<'a>) -> Element<'a> {
    let img = store
        .user_account
        .as_ref()
        .map(|i| {
            rsx!(img {
                onclick: move |evt| {
                    store.context_menu(
                        cx,
                        &evt,
                        menu::ContextMenu::<SidebarAction>::new(
                            "Account Options",
                            true,
                            vec![
                                menu::ContextMenuItem::item(
                                    "Open in Browser",
                                    SidebarAction::Root(Action::Public(PublicAction::OpenLink(
                                        i.url.clone(),
                                    ))),
                                ),
                                menu::ContextMenuItem::item(
                                    "Copy URL",
                                    SidebarAction::Root(Action::Public(PublicAction::Copy(
                                        i.url.clone(),
                                    ))),
                                ),
                                menu::ContextMenuItem::separator(),
                                menu::ContextMenuItem::item(
                                    "Logout",
                                    SidebarAction::Root(Action::Logout),
                                ),
                            ],
                        ),
                    )
                },
                class: "image-author-small",
                src: "{i.avatar_static}"
            })
        })
        .unwrap_or_else(|| {
            rsx!(span {
                style: "display: inline-block",
                class: "image-author-small"
            })
        });
    render! {
        img
    }
}

#[inline_props]
fn MenuComponent<'a>(cx: Scope<'a>, store: &'a ViewStore<'a>) -> Element<'a> {
    let img = store
        .user_account
        .as_ref()
        .map(|i| {
            rsx!(img {
                onclick: move |evt| {
                    store.context_menu(
                        cx,
                        &evt,
                        menu::ContextMenu::<SidebarAction>::new(
                            "Account Options",
                            true,
                            vec![
                                menu::ContextMenuItem::item(
                                    "Open in Browser",
                                    SidebarAction::Root(Action::Public(PublicAction::OpenLink(
                                        i.url.clone(),
                                    ))),
                                ),
                                menu::ContextMenuItem::item(
                                    "Copy URL",
                                    SidebarAction::Root(Action::Public(PublicAction::Copy(
                                        i.url.clone(),
                                    ))),
                                ),
                                menu::ContextMenuItem::separator(),
                                menu::ContextMenuItem::item(
                                    "Logout",
                                    SidebarAction::Root(Action::Logout),
                                ),
                            ],
                        ),
                    )
                },
                class: "image-author-small",
                src: "{i.avatar_static}"
            })
        })
        .unwrap_or_else(|| {
            rsx!(span {
                style: "display: inline-block",
                class: "image-author-small"
            })
        });
    let username = store
        .user_account
        .as_ref()
        .map(|e| e.username.clone())
        .unwrap_or_default();
    cx.render(rsx!(
        HStack { class: "justify-content-between justify-items-center p-2 ms-2 no-selection",
            div { title: "{username}", img }
            div { class: "icon-button ms-auto me-2", button {
                r#type: "button",
                onclick: move |_| { store.send(SidebarAction::Root(Action::AppEvent(AppEvent::MenuEvent(MainMenuEvent::Reload)))) },
                dangerous_inner_html: crate::icons::ICON_RELOAD
            } }
            div { class: "icon-button",
                button {
                    r#type: "button",
                    onclick: move |_| { store.send(SidebarAction::Root(Action::Post(PostKind::Post))) },
                    dangerous_inner_html: crate::icons::ICON_WRITE
                }
            }
        }
    ))
}

#[inline_props]
fn SearchComponent<'a>(
    cx: Scope<'a>,
    placeholder: &'static str,
    store: &'a ViewStore<'a>,
) -> Element<'a> {
    cx.render(rsx!(
        div { class: "p-2 pe-3",
            input {
                class: "width-100",
                r#type: "text",
                value: "{store.search_term}",
                placeholder: "{placeholder}",
                autocomplete: "off",
                spellcheck: "false",
                oninput: move |evt| {
                    store.send(SidebarAction::Search(evt.value.clone()));
                }
            }
        }
    ))
}

#[inline_props]
fn CellComponent<'a>(
    cx: Scope<'a>,
    model: AccountUpdateViewModel,
    selected: bool,
    store: &'a ViewStore<'a>,
    onclick: EventHandler<'a, ()>,
    favorited: bool,
) -> Element {
    // let window = AppWindow::retrieve(cx);
    let class = selected.then(|| "cell selected").unwrap_or("cell");
    cx.render(rsx!(
        div {
            class: "{class} grow",
            onclick: move |_| onclick.call(()),
            prevent_default: "oncontextmenu",
            oncontextmenu: move |e| {
                store.context_menu(
                    cx,
                    &e.data,
                    menu::ContextMenu::<SidebarAction>::new(
                        "Account",
                        true,
                        vec![
                            menu::ContextMenuItem::item(
                                "Open in Browser", 
                                Action::Public(PublicAction::OpenLink(model.account.url.clone()))
                            ),
                            menu::ContextMenuItem::item(
                                "Copy URL",
                                Action::Public(PublicAction::Copy(model.account.url.clone()))
                            ),
                            menu::ContextMenuItem::item(
                                "Copy Account Name", 
                                Action::Public(PublicAction::Copy(model.account.acct.clone()))
                            ),
                        ],
                    ),
                )
            },
            HStack { class: "gap-2 grow",
                VStack { class: "align-items-center no-shrink noclip",
                    img {
                        class: "image-author",
                        src: "{model.account.image}",
                        alt: "{model.account.display_name}",
                        width: 42,
                        height: 42
                    }
                    favorited.then(|| rsx!(div {
                    class: "favorite-icon",
                    Icon {
                        icon: crate::icons::ICON_LIKE_ACCOUNT,
                        title: loc!("Favorite Account. Always at the top"),
                }}))
                }
                VStack { class: "gap-1 grow account-preview-fields",
                    HStack { class: "justify-content-between force-single-line",
                        Label {
                            onclick: move |_| onclick.call(()),
                            style: TextStyle::Primary,
                            class: "me-auto",
                            pointer_style: PointerStyle::Pointer,
                            "{model.account.username}"
                        }
                        Label { style: TextStyle::Secondary, pointer_style: PointerStyle::Pointer, "{model.last_updated_human}" }
                    }
                    Paragraph {
                        class: "status-content",
                        style: TextStyle::Tertiary,
                        pointer_style: PointerStyle::Pointer,
                        "{model.content}"
                    }
                }
            }
        }
    ))
}

#[inline_props]
fn AccountCellComponent<'a>(
    cx: Scope<'a>,
    store: &'a ViewStore<'a>,
    account: Account,
    selected: bool,
    onclick: EventHandler<'a, ()>,
) -> Element {
    let class = selected.then(|| "cell selected").unwrap_or("cell");
    cx.render(rsx!(
        div {
            class: "{class} grow",
            onclick: move |_| onclick.call(()),
            prevent_default: "oncontextmenu",
            oncontextmenu: move |e| {
                store.context_menu(
                    cx,
                    &e.data,
                    menu::ContextMenu::<SidebarAction>::new(
                        "Account",
                        true,
                        vec![
                            menu::ContextMenuItem::item(
                                "Open in Browser",
                                Action::Public(PublicAction::OpenLink(account .url.clone()))
                            ),
                            menu::ContextMenuItem::item(
                                "Copy URL",
                                Action::Public(PublicAction::Copy(account.url.clone()))
                            ),
                            menu::ContextMenuItem::item(
                                "Copy Account Name",
                                Action::Public(PublicAction::Copy(account.acct.clone()))
                            ),
                        ],
                    ),
                )
            },
            HStack { class: "gap-2 grow",
                img {
                    class: "image-author",
                    src: "{account.avatar_static}",
                    alt: "{account.display_name}",
                    width: 32,
                    height: 32
                }
                VStack { class: "gap-1 grow account-preview-fields",
                    HStack { class: "justify-content-between force-single-line",
                        Label {
                            onclick: move |_| onclick.call(()),
                            style: TextStyle::Primary,
                            class: "me-auto",
                            pointer_style: PointerStyle::Pointer,
                            "{account.username}"
                        }
                    }
                    Paragraph {
                        class: "status-content",
                        style: TextStyle::Tertiary,
                        pointer_style: PointerStyle::Pointer,
                        "{account.display_name}"
                    }
                }
            }
        }
    ))
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TabBarItem {
    id: UiTab,
    label: String,
    selected: bool,
    dot: bool,
}

impl TabBarItem {
    pub fn new(id: UiTab, label: String, selected: bool, dot: bool) -> Self {
        Self {
            id,
            label,
            selected,
            dot,
        }
    }
}

#[inline_props]
pub fn TabBar<'a>(
    cx: Scope<'a>,
    items: Vec<TabBarItem>,
    onclick: EventHandler<'a, TabBarItem>,
) -> Element<'a> {
    cx.render(rsx!(
        div { class: "tabbar",
            items.iter().map(|item| rsx!(TabButton {
            label: item.label.as_str(),
            onclick: move |_| onclick.call(item.clone()),
            selected: item.selected,
            dot: item.dot,
        }))
        }
    ))
}

#[inline_props]
pub fn TabButton<'a>(
    cx: Scope<'a>,
    label: &'a str,
    onclick: EventHandler<'a, ()>,
    selected: bool,
    dot: bool,
) -> Element<'a> {
    let dot = dot.then(|| rsx!(span { class: "dot" }));
    let cls = selected.then(|| " selected").unwrap_or_default();
    cx.render(rsx!(
        button { class: "button {cls}", onclick: move |_| {
                onclick.call(());
            }, dot, "{label}" }
    ))
}

#[inline_props]
fn ListSelector<'a>(cx: Scope<'a>, store: &'a ViewStore<'a>) -> Element<'a> {
    let has_timelines = if store.list_names.len() > 1 {
        "false"
    } else {
        "true"
    };
    render! {
        select {
            name: "list",
            class: "m-2",
            disabled: "{has_timelines}",
            onchange: move |evt| {
                store.send(SidebarAction::SelectList(evt.value.clone()));
            },
            option { value: "", loc!("Timeline") }
            for (id , name) in store.list_names.iter() {
                option { value: "{id}", "{name}" }
            }
        }
    }
}

impl MoreSelection {
    fn content(&self) -> &str {
        match self {
            Self::Classic => "􀭞",   // square.fill.text.grid.1x2
            Self::Yours => "􀈎",     // square.and.pencil
            Self::Local => "􀝋",     // person.3.fill
            Self::Federated => "􀆪", // globe
            Self::Posts => "􀌪",     // bubble.left
            Self::Hashtags => "􀋡",  // tag
            Self::Followers => "􀉬", // person.2.fill
            Self::Following => "􀉫", // person.2
            Self::Bookmarks => "􀼺", // bookmark.square.fill
            Self::Favorites => "􀠨", // star.square.fill
        }
    }

    fn title(&self) -> &str {
        match self {
            MoreSelection::Classic => loc!("Classic Timeline"),
            MoreSelection::Yours => loc!("Your Posts"),
            MoreSelection::Local => loc!("Local"),
            MoreSelection::Federated => loc!("Federated"),
            MoreSelection::Posts => loc!("Posts"),
            MoreSelection::Hashtags => loc!("Hashtags"),
            MoreSelection::Followers => loc!("Followers"),
            MoreSelection::Following => loc!("Following"),
            MoreSelection::Bookmarks => loc!("Bookmarks"),
            MoreSelection::Favorites => loc!("Favorites"),
        }
    }
}
