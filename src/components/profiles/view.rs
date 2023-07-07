use dioxus::prelude::*;
use navicula::reducer::ChildReducer;

use super::reducer::ViewStore;
use super::{reducer::ProfilesAction, ProfilesReducer};
use crate::components::profile_preview::{
    ListProfileComponent, ProfilePreviewReducer, ProfileState,
};
use crate::loc;
use crate::view_model::AccountViewModel;
use crate::widgets::*;

#[inline_props]
pub fn ProfilesView<'a>(cx: Scope<'a>, store: ViewStore<'a>) -> Element<'a> {
    let nested_paddings = if store.is_nested {
        ""
    } else {
        "content-cell-container"
    };

    render! {
        div {
            class: "profiles-list-component",
            VStack {
                div {
                    class: "scroll {nested_paddings}",
                    { store.is_loading.then(|| rsx!(div {
                        class: "hstack p-2 m-2 grow align-self-center",
                        Spinner {}
                    }))}

                    for profile in &store.profiles {
                        ProfileView {
                            key: "a{profile.id.0}",
                            store: store,
                            profile: profile,
                        }
                    }

                    { store.is_loading_more.then(|| rsx!(div {
                        class: "hstack p-2 m-2 grow align-self-center",
                        Spinner {}
                    }))}

                    store.can_load_more.then(|| {
                        rsx!(MoreFollowers {
                            key: "more_toots_top",
                            icon: crate::icons::ICON_LOAD_OLDER_TIMELINE,
                            is_loading: store.is_loading || store.is_loading_more,
                            can_load_more: store.can_load_more,
                            onclick: move |_| store.send(ProfilesAction::LoadMoreData)
                        })
                    })
                }
            }
        }
    }
}

#[inline_props]
pub fn ProfileView<'a>(
    cx: Scope<'a>,
    store: &'a ViewStore<'a>,
    profile: &'a AccountViewModel,
) -> Element<'a> {
    render! {
        ListProfileComponent {
            key: "b{profile.id.0}",
            onclick: |_| {
                store.send(ProfilesAction::Select(Some((*profile).clone())));
            },
            store: store.host_with(
                cx,
                *profile,
                |account| ProfileState::new(account, true),
            ),
            button {
                class:  "button highlighted ms-auto",
                title: loc!("Open Profile"),
                onclick: move |_| {
                    store.send(ProfilesAction::Select(Some((*profile).clone())));
                },
                loc!("Open")
            }
        }
    }
}

impl ChildReducer<ProfilesReducer> for ProfilePreviewReducer {
    fn to_child(
        _message: <ProfilesReducer as navicula::Reducer>::Message,
    ) -> Option<<Self as navicula::Reducer>::Action> {
        None
    }

    fn from_child(
        message: <Self as navicula::Reducer>::DelegateMessage,
    ) -> Option<<ProfilesReducer as navicula::Reducer>::Action> {
        Some(ProfilesAction::TimelineAction(message))
    }
}

#[inline_props]
fn MoreFollowers<'a>(
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
                text: loc!("More Followers"),
                title: loc!("Load more followers"),
                class: "mb-3",
                onclick: move |_| {
                    onclick.call(())
                },
            }
        ))
    }))}
}
