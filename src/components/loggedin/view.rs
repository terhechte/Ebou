use dioxus::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;

use crate::components::more::MoreReducer;
use crate::components::post::PostKind;
use crate::environment::types::{AppEvent, FileEvent, MainMenuEvent};
use crate::view_model::*;

use navicula::root;

use super::reducer::{Action, ReducerState};
use crate::environment::{model::Model, Environment};

use super::RootReducer;
use super::ViewStore;
use crate::components::sidebar::{SidebarAction, SidebarComponent, SidebarReducer, SidebarState};
use crate::widgets::*;
use crate::{PublicAction, StatusMutation};
use navicula::reducer::{ChildReducer, Reducer};

#[inline_props]
pub fn LoggedInApp<'a>(
    cx: Scope<'a>,
    environment: &'a UseState<Environment>,
    should_show_login: &'a UseState<bool>,
) -> Element<'a> {
    log::trace!("rerender loggedin-app 1");

    let updater = cx.schedule_update();
    let (sender, receiver) = cx.use_hook(flume::unbounded);
    let moved_sender = sender.clone();
    let moved_updater = updater.clone();
    environment.platform.handle_menu_events(
        cx,
        Arc::new(move |a| {
            let _ = moved_sender.send(Action::AppEvent(a));
            moved_updater();
        }),
    );

    let cloned_sender = sender.clone();
    let cloned_updater = updater.clone();
    let toolbar_sender = Arc::new(move |action| {
        if let Err(e) = cloned_sender.send(Action::AppEvent(action)) {
            log::error!("Could not send msg: {e:?}");
        }
        cloned_updater();
    });

    cx.use_hook(|| {
        environment
            .platform
            .set_toolbar_handler(toolbar_sender.clone());
    });

    let view_store: ViewStore = root(cx, &[receiver.clone()], environment.get(), || {
        ReducerState::default()
    });

    // FIXME: At some point, move the side effects so that it also works
    // with one absolutely-root reducer
    if view_store.did_logout.get().is_some() {
        let mut mutable_environment = environment.get().clone();
        mutable_environment.update_model(Model::default());
        environment.set(mutable_environment);
        should_show_login.set(true);
        view_store.did_logout.set(None);
    }

    let is_dropping = view_store.flags.is_dropping;
    let error = view_store.error.clone();

    cx.render(rsx!(
        div {
            MainComponent { store: view_store.clone() }
            {
                error.map(|error|
                    rsx!(div {
                        class: "error-box-bottom",
                        ErrorBox {
                            content: error.clone(),
                            onclick: move |_| {
                                view_store.send(Action::ClearError)
                            }
                        }
                }))},
            {
                is_dropping.then(|| rsx!(div {
                    class: "fullscreen file-drop-box"
                }))
            }
        }
    ))
}

#[inline_props]
fn ReplyComponent<'a>(
    cx: Scope<'a>,
    store: &'a ViewStore<'a>,
    kind: &'a PostKind,
    images: &'a [PathBuf],
) -> Element<'a> {
    let Some(account) = store.user_account.as_ref().cloned() else {
        return cx.render(rsx!(div {}))
    };
    // we don't need the sender.. we have a build-in one. check how to get rid of it
    use crate::components::post::{PostView, State};
    let state = State::new(account, (*kind).clone(), false, images.to_vec());
    render! {
        div {
            class: "reply-window-container",
            div {
                class: "reply-window-child",
                PostView {
                    store: store.host(
                        cx,
                        || state
                    )
                }
            }
        }
    }
}

impl ChildReducer<RootReducer> for crate::components::post::PostReducer {
    fn to_child(message: <RootReducer as Reducer>::Message) -> Option<<Self as Reducer>::Action> {
        match message {
            Action::AppEvent(AppEvent::FileEvent(FileEvent::Dropped(images))) => {
                Some(crate::components::post::PostAction::DroppedPaths(images))
            }
            _ => None,
        }
    }

    fn from_child(
        _message: <Self as Reducer>::DelegateMessage,
    ) -> Option<<RootReducer as Reducer>::Action> {
        Some(Action::PostCancel)
    }
}

#[inline_props]
fn MainComponent<'a>(cx: Scope<'a>, store: ViewStore<'a>) -> Element<'a> {
    log::trace!("Rerender MainComponent");
    if store.flags.logging_in {
        cx.render(rsx!(
            div { class: "vstack p-2 m-2 grow align-items-center justify-items-center full-width full-height",
                p { class: "label-secondary mt-p20" }
                div { class: "hstack p-2 m-2 grow align-self-center full-width full-height",
                    Spinner {}
                }
            }
        ))
    } else if store.logged_in {
        cx.render(rsx! {
            SplitViewComponent {
                sidebar: cx.render(
                    rsx!(
                        SidebarComponent {
                            store: store.host(cx, SidebarState::default)
                        }
                    ),
                ),
                content: cx.render(rsx!(ContentComponent { store: store }))
            }
            store.is_replying.as_ref().map(|(kind, images)| rsx!(ReplyComponent {
                store: store,
                kind: kind
                images: images
            }))
        })
    } else {
        cx.render(rsx!(div {}))
    }
}

impl ChildReducer<RootReducer> for SidebarReducer {
    fn to_child(message: <RootReducer as Reducer>::Message) -> Option<<Self as Reducer>::Action> {
        match message {
            Action::PreferencesChanged(_) => Some(SidebarAction::Reload(true)),
            Action::AppEvent(AppEvent::MenuEvent(MainMenuEvent::Reload)) => {
                Some(SidebarAction::Reload(true))
            }
            Action::AppEvent(AppEvent::MenuEvent(
                MainMenuEvent::ScrollUp | MainMenuEvent::ScrollDown,
            )) => None,
            Action::AppEvent(m @ AppEvent::MenuEvent(_)) => Some(SidebarAction::AppEvent(m)),
            _ => None,
        }
    }

    fn from_child(
        message: <Self as Reducer>::DelegateMessage,
    ) -> Option<<RootReducer as Reducer>::Action> {
        use crate::components::sidebar::SidebarDelegateAction;
        Some(match message {
            SidebarDelegateAction::SelectAccount(a) => Action::SelectAccount(a),
            SidebarDelegateAction::SelectedNotifications(a) => Action::SelectNotifications(a),
            SidebarDelegateAction::Root(a) => a,
            SidebarDelegateAction::AppEvent(a) => Action::AppEvent(a),
            SidebarDelegateAction::SelectMore(a) => Action::SelectMore(a),
        })
    }
}

#[inline_props]
fn ContentComponent<'a>(cx: Scope<'a>, store: &'a ViewStore<'a>) -> Element<'a> {
    let tab = store.active_tab;
    render! {
        div { class: "content-component",
            AccountContentComponent {
                store: store,
                hidden: !tab.is_timeline()
            }
            NotificationContentComponent {
                store: store,
                hidden: !tab.is_mentions()
            }
            MoreComponent {
                store: store
                hidden: !tab.is_more()
            }
        }
    }
}

#[inline_props]
fn AccountContentComponent<'a>(
    cx: Scope<'a>,
    store: &'a ViewStore<'a>,
    hidden: bool,
) -> Element<'a> {
    let Some(account) = store.selected_account.as_ref() else {
        return render! {
            div {}
        }
    };

    use crate::components::component_stack::{Stack, State};

    render!(HideableView {
        hidden: *hidden,
        Stack {
            store: store.host_with(
                cx,
                account,
                |a| State::new(RootTimelineKind::GroupedAccount(a))
            )
        }
    })
}

use crate::components::component_stack::{Action as StackAction, RootTimelineKind, StackReducer};
impl ChildReducer<RootReducer> for StackReducer {
    fn to_child(message: <RootReducer as Reducer>::Message) -> Option<<Self as Reducer>::Action> {
        match message {
            Action::AppEvent(a) => Some(StackAction::AppEvent(a)),
            _ => None,
        }
    }

    fn from_child(
        message: <Self as Reducer>::DelegateMessage,
    ) -> Option<<RootReducer as Reducer>::Action> {
        use crate::components::component_stack::DelegateMessage;
        match message {
            // DelegateMessage::AppEvent(a) => Action::AppEvent(a),
            DelegateMessage::PublicAction(a) => Some(Action::Public(a)),
            DelegateMessage::ConversationAction(a) => Some(Action::Public(a)),
        }
    }
}

#[inline_props]
fn NotificationContentComponent<'a>(
    cx: Scope<'a>,
    store: &'a ViewStore<'a>,
    hidden: bool,
) -> Element<'a> {
    let Some(account) = store.selected_notifications.as_ref() else {
        return render! {
            div {}
        }
    };
    use crate::components::component_stack::{Stack, State};

    render!(HideableView {
        hidden: *hidden,

        Stack {
            store: store.host_with(
                cx,
                account,
                |a| State::new(RootTimelineKind::Notifications(a))
            )
        }
    })
}

#[inline_props]
fn MoreComponent<'a>(cx: Scope<'a>, store: &'a ViewStore<'a>, hidden: bool) -> Element<'a> {
    log::trace!("render MoreComponent {}", cx.scope_id().0);
    let selection = store.more_selection;
    let Some(account) = store.current_user.as_ref() else {
        return render!(div {})
    };

    use crate::components::more::{MoreViewComponent, State};
    render!(HideableView {
        hidden: *hidden,
         MoreViewComponent {
            store: store.host_with(cx, &selection, |s| State::new(s, account.clone()))
        }
    })
}

impl ChildReducer<RootReducer> for MoreReducer {
    fn to_child(message: <RootReducer as Reducer>::Message) -> Option<<Self as Reducer>::Action> {
        use crate::components::more::Action as MoreAction;
        match message {
            Action::AppEvent(a) => Some(MoreAction::AppEvent(a)),
            Action::SelectMore(s) => Some(MoreAction::Selection(s)),
            _ => None,
        }
    }

    fn from_child(
        message: <Self as Reducer>::DelegateMessage,
    ) -> Option<<RootReducer as Reducer>::Action> {
        use crate::components::more::PublicAction;
        Some(match message {
            PublicAction::Timeline(action) => Action::Public(action),
        })
    }
}

impl From<(StatusAction, &StatusViewModel)> for Action {
    fn from(value: (StatusAction, &StatusViewModel)) -> Self {
        let (action, status) = value;
        match action {
            StatusAction::Clicked => Action::SelectConversation(StatusId(status.id.0.clone())),
            StatusAction::Boost(s) => Action::Public(PublicAction::StatusMutation(
                StatusMutation::Boost(s),
                status.clone(),
            )),
            StatusAction::Reply => Action::Post(PostKind::Reply(status.clone())),
            StatusAction::Favorite(s) => Action::Public(PublicAction::StatusMutation(
                StatusMutation::Favourite(s),
                status.clone(),
            )),
            StatusAction::Bookmark(s) => Action::Public(PublicAction::StatusMutation(
                StatusMutation::Bookmark(s),
                status.clone(),
            )),
            StatusAction::OpenAccount(a) => Action::Public(PublicAction::OpenLink(a)),
            StatusAction::OpenLink(a) => Action::Public(PublicAction::OpenLink(a)),
            StatusAction::OpenTag(a) => Action::Public(PublicAction::OpenTag(a)),
            StatusAction::OpenImage(a) => Action::Public(PublicAction::OpenImage(a)),
            StatusAction::OpenVideo(a) => Action::Public(PublicAction::OpenVideo(a)),
            StatusAction::Copy(a) => Action::Public(PublicAction::Copy(a)),
        }
    }
}
