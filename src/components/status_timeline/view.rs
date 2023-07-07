use super::{reducer::*, TimelineReducer};
use crate::components::profile_preview::{
     FollowProfileComponent, ProfileComponent, ProfileState, ProfilePreviewReducer,
};
use crate::environment::types::TimelineDirection;
use crate::icons;
use crate::environment::menu::{ self, ViewStoreContextMenu};
use crate::widgets::*;
use crate::{loc, view_model::*};
use dioxus::prelude::*;
use enumset::EnumSet;
use navicula::reducer::ChildReducer;

use super::reducer::ViewStore;

#[inline_props]
pub fn TimelineComponent<'a>(cx: Scope<'a>, store: ViewStore<'a>) -> Element<'a> {
    let account = &store.account;

    let account_settings = account.as_ref().map(|account| {
        store
            .ui_settings
            .visibility
            .get(&account.id.0)
            .cloned()
            .map(|e| (e, account))
            .unwrap_or_else(|| (EnumSet::all(), account))
    });

    let vstack_class = if store
        .is_loading { "toolbar hidden" } else { "toolbar" };

    cx.render(rsx!(
        div { class: "timeline-component",
            VStack {
                account_settings.map(|(view_settings, account)| rsx!(AccountProfileHeader {
                    store: store,
                    vstack_class: vstack_class,
                    account: account,
                    view_settings: Some(view_settings)
                }))


                div {
                    id: "{store.identifier}",
                    class: "scroll content-cell-container",
                    TimelineContents {
                        store: store.clone(),
                        account_settings: account_settings,
                        show_profile: true,
                    }
                }
            }
        }
    ))
}

#[derive(Props)]
pub struct TimelineContentsProps<'a> {
    store: ViewStore<'a>,
    #[props(!optional)]
    account_settings: Option<(EnumSet<AccountVisibility>, &'a AccountViewModel)>,
    show_profile: Option<bool>,
}

pub fn TimelineContents<'a>(
    cx: Scope<'a, TimelineContentsProps<'a>>,
) -> Element<'a> {
    let show_profile = cx.props.show_profile.unwrap_or(true);

    let store = &cx.props.store;

    let posts = &store.posts;
    let last = posts.last().map(|e| e.id.clone());
    let last2 = last.clone();

    render! {
        {
            (store.direction() == TimelineDirection::NewestBottom).then(|| {
                {(store.is_loading || store.is_loading_more).then(|| rsx!(div {
                    class: "hstack p-2 m-2 grow align-self-center",
                    Spinner {}
                }))}
            })
        },

        // If we're loading the initial account data (not history) we display
        // nothing until it is loaded
        {(!store.is_loading).then(|| rsx! {
            {
                (store.direction() == TimelineDirection::NewestBottom).then(move || {
                    rsx!(MoreToots {
                        key: "more_toots_top",
                        icon: crate::icons::ICON_LOAD_OLDER,
                        is_loading: store.is_loading || store.is_loading_more,
                        can_load_more: store.can_load_more,
                        onclick: move |_| store.send(Action::LoadMoreData(last.clone()))
                    })
                })
            }

            { posts.iter()
                .filter(|status| {
                    let Some((view_settings, _)) = cx.props.account_settings else {
                        return true
                    };
                    if status.is_reply && !view_settings.contains(AccountVisibility::Replies) {
                        return false;
                    }
                    if status.is_reblog && !view_settings.contains(AccountVisibility::Boosts) {
                        return false;
                    }
                    if !status.is_reply && status.is_reblog &&
                    !view_settings.contains(AccountVisibility::Toots) {
                        return false;
                    }
                    true
                })
                .map(|status| rsx!(ContentCellComponent {
                    // This has to be here so that new entries in the timeline don't crash
                    // because the type might have changed and thus the use_hook allocation
                    // order changes
                    key: "{status.id.0}",
                    status: status.clone(),
                    store: store,
                    show_profile: show_profile
            })) }

            {
                (store.direction() == TimelineDirection::NewestTop).then(move || {
                    rsx!(MoreToots {
                        key: "more_toots_bottom",
                        icon: crate::icons::ICON_LOAD_OLDER_TIMELINE,
                        is_loading: store.is_loading || store.is_loading_more,
                        can_load_more: store.can_load_more,
                        onclick: move |_| store.send(Action::LoadMoreData(last2.clone()))
                    })
                })
            }
        })},

        {
            (store.direction() == TimelineDirection::NewestTop).then(|| {
                {(store.is_loading_more || store.is_loading).then(|| rsx!(div {
                    class: "hstack p-2 m-2 grow align-self-center",
                    Spinner {}
                }))}
            })
        }
    }
}

#[inline_props]
fn AccountProfileHeader<'a>(
    cx: Scope<'a>,
    store: &'a ViewStore<'a>,
    vstack_class: &'a str,
    account: &'a AccountViewModel,
    view_settings: Option<Option<EnumSet<AccountVisibility>>>,
) -> Element<'a> {
    let key = account.id.0.clone();
    render! {
        FollowProfileComponent {
            key: "{key}",
            class: vstack_class,
            is_expanded: false,
            store: store.host_with(
                cx,
                *account,
                |account| ProfileState::new(account, true),
            ),
            IconButton {
                icon: crate::icons::ICON_MORE,
                title: loc!("More Account Actions"),
                onclick: move |evt: Event<MouseData>| {
                    use crate::PublicAction;
                    let mut items = vec![
                        menu::ContextMenuItem::item(loc!("Open"), Action::Public(PublicAction::OpenProfile((*account).clone()))),
                        menu::ContextMenuItem::separator(),
                        menu::ContextMenuItem::item(loc!("Open Profile in Browser"), Action::Public(PublicAction::OpenLink(account.url.clone()))),
                        menu::ContextMenuItem::item(loc!("Copy Profile URL"), Action::Public(PublicAction::Copy(account.url.clone()))),
                    ];
                    if let Some(Some(view_settings)) = view_settings {
                        items.push(menu::ContextMenuItem::separator());
                        items.push(menu::ContextMenuItem::submenu(loc!("Show"), vec![
                            menu::ContextMenuItem::checkbox(loc!("Toots"),
                                view_settings.contains(AccountVisibility::Toots),
                                Action::AccountVisibility(account.id.clone(), AccountVisibility::Toots)
                            ),
                            menu::ContextMenuItem::checkbox(loc!("Replies"),
                                view_settings.contains(AccountVisibility::Replies),
                                Action::AccountVisibility(account.id.clone(), AccountVisibility::Replies)
                            ),
                            menu::ContextMenuItem::checkbox(loc!("Boosts"),
                                view_settings.contains(AccountVisibility::Boosts),
                                Action::AccountVisibility(account.id.clone(), AccountVisibility::Boosts)
                            ),
                        ]));
                    }

                    store.context_menu(
                        cx,
                        &evt.data, menu::ContextMenu::<Action>::new(loc!("Profile Options"), true, items))
                }
            }
        }
    }
}

#[inline_props]
fn MoreToots<'a>(
    cx: Scope<'a>,
    icon: &'a str,
    is_loading: bool,
    can_load_more: bool,
    onclick: EventHandler<'a, ()>,
) -> Element<'a> {
    render! {(!is_loading).then(|| rsx!(div {
        class: "hstack justify-content-center",
        can_load_more.then(|| rsx!(
            IconTextButton {
                icon: icon,
                text: loc!("Older Toots"),
                title: loc!("Load older toots"),
                class: "mb-3",
                onclick: move |_| {
                    onclick.call(())
                },
            }
        ))
    }))}
}

#[inline_props]
fn ContentCellComponent<'a>(
    cx: Scope<'a>,
    status: StatusViewModel,
    store: &'a ViewStore<'a>,
    show_profile: bool,
) -> Element<'a> {
    if let Some(ref boosted) = status.reblog_status {
        let is_selected = store.known_conversations.contains(&boosted.id);
        let sel_class = if is_selected { "cell-selected" } else { "content-cell-selectable" };
        let sax = *boosted.clone();
        cx.render(rsx!(
            div {
                class: "boost-container no-selection content-cell-bottom-margin ",
                id: "{status.id.dom_id()}",
                div { class: "p-1 boost-header",
                    span { class: "icon", title: loc!("Boosted"), dangerous_inner_html: icons::ICON_BOOST2 }
                    Label { 
                        class: "ms-2 me-auto",
                        title: "{status.created_human} - {status.created_full}",
                        "{status.account.username} boosted" 
                    }
                }
                div { class: "vstack content-cell no-selection {sel_class}",
                    StatusComponent {
                        status: boosted.as_ref().clone(),
                        onclick: move |action| { store.send((action, boosted.as_ref()).into()) },
                        sender: store.sender_fn(move |a: StatusAction| {
                            (a, &sax).into()
                        }),
                        ProfileComponent {
                            store: store.host_with(
                                cx,
                                &boosted.account,
                                |account| ProfileState::new(account, false),
                            ),
                            Label {
                                style: TextStyle::Tertiary,
                                title: "{status.created_full}",
                                "{status.created_human}"
                            }
                        }
                    }
                }
            }
        ))
    } else {
        let is_selected = store.known_conversations.contains(&status.id);
        let sel_class = if is_selected { "cell-selected" } else { "content-cell-selectable" };
        let sax = status.clone();
        cx.render(rsx!(
            div {
                class: "content-cell no-selection {sel_class} content-cell-bottom-margin",
                id: "{status.id.dom_id()}",
                if store.account.is_none() && *show_profile {
                    rsx!(ProfileComponent {
                        store: store.host_with(
                            cx,
                            &status.account,
                            |account| ProfileState::new(account, false),
                            // || ProfileState::new(status.account.clone()),
                        ),
                        FormattedTime {
                            human_time: &status.created_human,
                            full_time: &status.created_full,
                            align: TextAlign::Right
                        }
                    })
                }
                StatusComponent {
                    status: status.clone(),
                    onclick: move |action| { store.send((action, status).into()) },
                    sender: store.sender_fn(move |a| {
                        (a, &sax).into()
                    }),
                    div { class: "p-2",
                        FormattedTime {
                            human_time: &status.created_human,
                            full_time: &status.created_full,
                            align: TextAlign::Left
                        }
                    }
                }
            }
        ))
    }
}

impl From<(StatusAction, &StatusViewModel)> for Action {
    fn from(value: (StatusAction, &StatusViewModel)) -> Self {
        let (action, status) = value;
        Action::Public((action, status.clone()).into())
    }
}

impl ChildReducer<TimelineReducer> for ProfilePreviewReducer {
    fn to_child(_message: <TimelineReducer as navicula::Reducer>::Message) -> Option<<Self as navicula::Reducer>::Action> {
        None
    }

    fn from_child(message: <Self as navicula::Reducer>::DelegateMessage) -> Option<<TimelineReducer as navicula::Reducer>::Action> {
        log::debug!("ChildReducer<TimelineReducer>:ProfilePreviewReducer: {message:?}");
        Some(Action::Public(message))
    }
}