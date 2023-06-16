use crate::behaviours::{Behaviour, ChangeTextsizeBehaviour};
use crate::components::post::{PostAction, PostKind};
use crate::environment::model::{Account, Message, Model, Status};
use crate::environment::storage::{Data, UiTab};
use crate::environment::types::{AppEvent, FileEvent, MainMenuEvent, UiConfig};
use crate::environment::Environment;
use crate::widgets::StatusAction;
use crate::windows::image_window::{ImageWindowKind, ImageWindowState};
use crate::windows::post_window::PostWindowState;
use crate::windows::preferences_window::{PreferencesChange, PreferencesWindowState};
use crate::{loc, view_model::*};
use crate::{PublicAction, StatusMutation};
use debug_panic::debug_panic;
use navicula::publisher::RefPublisher;
use navicula::Effect;

use crate::components::sidebar::MoreSelection;
use std::cell::{Cell, RefMut};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

const NEW_TOOT_TITLE: &str = "New Toot";
const NEW_TOOT_SIZE: (f64, f64) = (420., 320.);

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct State {
    /// Are we logging in
    pub logging_in: bool,
    /// Is the main-content loading
    pub loading_content: bool,
    /// Are we loading the initial content for an account
    pub loading_account: bool,
    /// Are we loading more (e.g. older) posts for an account
    pub loading_account_history: bool,
    /// Are we loading a conversation
    pub loading_conversation: bool,
    /// Are we loading more notifications
    pub loading_notifications: bool,
    /// Is the user trying to drop something
    pub is_dropping: bool,
}

#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct ReducerState {
    pub last_notification_id: Option<String>,
    pub ui_settings: UiConfig,
    pub flags: State,

    pub user_account: Option<Account>,
    pub active_tab: UiTab,
    pub selected_account: Option<AccountViewModel>,
    pub selected_notifications: Option<AccountViewModel>,

    pub error: Option<String>,
    pub did_logout: Cell<Option<bool>>,
    pub has_new_notifications: bool,
    pub logged_in: bool,
    pub is_replying: Option<(PostKind, Vec<PathBuf>)>,
    /// The current more-selection
    pub more_selection: MoreSelection,
    pub current_user: Option<Account>,
}

// Terrible hack to allow updating the app on drag and drop events.
// The listener / handler for this has to be initialized before we have
// the first `Scope`, therefore not being able to call `update`. So we
// save the actual updater after the initialization of the handler
// and pull it out when the handler is called the first time
pub type ScopeUpdaterMutex = Mutex<Option<Arc<dyn Fn(AppEvent) + Send + Sync>>>;
lazy_static::lazy_static! {
    pub static ref SCOPE_UPDATER: ScopeUpdaterMutex = Mutex::new(None);
}

pub fn reduce<'a>(
    context: &'a impl navicula::types::MessageContext<Action, Action, Action>,
    action: Action,
    reducer_state: &'a mut ReducerState,
    environment: &'a Environment,
) -> Effect<'static, Action> {
    log::trace!("{action:?}");
    if let Ok(mut m) = SCOPE_UPDATER.try_lock() {
        if m.is_none() {
            let updater = context.updater().clone();
            *m = Some(Arc::new(move |app_event| {
                updater(Action::AppEvent(app_event))
            }));
        }
    }

    let model = environment.model.clone();

    let window = context.window();
    let is_fullscreen = crate::environment::platform::is_fullscreen(window);
    let inline_window = is_fullscreen || reducer_state.ui_settings.post_window_inline;

    match action {
        Action::Login => {
            reducer_state.ui_settings = environment.repository.config().unwrap_or_default();
            reducer_state.flags.logging_in = true;
            return Effect::future(async move { model.login().await }, Action::LoggedIn);
        }
        Action::LoggedIn(result) => {
            reducer_state.flags.logging_in = false;
            match result {
                Ok(n) => {
                    reducer_state.current_user = Some(n.clone());
                    // update the toolbar. Initially, always timeline
                    environment.platform.update_toolbar(
                        &n.avatar,
                        window,
                        &crate::environment::storage::UiTab::Timeline,
                        false,
                    );

                    environment.storage.with_mutation(|mut s| {
                        s.user_account = Some(n);
                    });
                    environment.platform.update_menu(window, |config| {
                        config.logged_in = true;
                    });
                    reducer_state.logged_in = true;
                }
                Err(e) => {
                    reducer_state.logged_in = false;
                    reducer_state.error = Some(format!("Login Error: {e:?}"))
                }
            }

            let updater = context.updater().clone();

            // The future to start the subscription
            let fut = Effect::fire_forget(async move {
                let _ = model
                    .subscribe_user_stream(std::sync::Arc::new(move |msg| {
                        // We have to wrap the updater into an updater that maps
                        updater(Action::MessageEvent(msg))
                    }))
                    .await;
            });

            Effect::merge2(
                // Start the subscription
                fut,
                // Subscribe to storage changes
                environment
                    .storage
                    .subscribe("root_reducer", context, |_| Action::DataUpdated),
            )
        }
        Action::DataUpdated => {
            environment.storage.with(|s| {
                reducer_state.user_account = s.user_account.clone();
                reducer_state.active_tab = s.active_tab;
                reducer_state.selected_account = s.selected_account.clone();
                reducer_state.selected_notifications = s.selected_notifications.clone();
            });
            Effect::NONE
        }
        Action::SelectAccount(account) => {
            environment.storage.with_mutation(|mut storage| {
                storage.selected_account = Some(account.clone());
            });
            Effect::NONE
        }
        Action::SelectNotifications(account) => {
            environment.storage.with_mutation(|mut storage| {
                storage.selected_notifications = Some(account.clone());
            });
            Effect::NONE
        }

        Action::SelectConversation(status) => {
            context.send_children(Action::SelectConversation(status));
            Effect::NONE
        }
        Action::SelectMore(more) => {
            reducer_state.more_selection = more;
            context.send_children(action);
            Effect::NONE
        }

        Action::Public(action) => {
            match action {
                PublicAction::Close => Effect::NONE,
                PublicAction::Post(p) => Effect::action(Action::Post(p)),
                PublicAction::StatusMutation(mutation, status) => {
                    return mutate_status(&environment.storage, mutation, model, status, |s| {
                        // Apply the mutation
                        match mutation {
                            StatusMutation::Bookmark(o) => {
                                s.is_bookmarked = o;
                            }
                            StatusMutation::Favourite(o) => {
                                s.is_favourited = o;
                                s.update_favorited(o);
                            }
                            StatusMutation::Boost(o) => {
                                s.has_reblogged = o;
                                s.update_reblog(o);
                            }
                        }
                    });
                }
                PublicAction::Conversation(_action) => {
                    debug_panic!();
                    Effect::NONE
                }
                PublicAction::Copy(link) => {
                    crate::environment::platform::copy_to_clipboard(link);
                    Effect::NONE
                }
                PublicAction::OpenLink(url) => {
                    environment.open_url(&url);
                    Effect::NONE
                }
                PublicAction::OpenProfile(_profile) => {
                    debug_panic!();
                    Effect::NONE
                }
                PublicAction::OpenProfileLink(_link) => {
                    debug_panic!();
                    Effect::NONE
                }
                PublicAction::OpenTag(name) => {
                    let name = name.replace('#', "");
                    Effect::future(
                        async move {
                            model
                                .tag(name.clone())
                                .await
                                .map(|e| e.url)
                                .unwrap_or_else(|_| format!("https://mastodon.social/tags/{name}"))
                        },
                        |a| Action::Public(PublicAction::OpenLink(a)),
                    )
                }
                PublicAction::OpenImage(url) => {
                    environment.open_window(
                        window,
                        ImageWindowState(url, ImageWindowKind::Image),
                        800.,
                        600.,
                        "Image",
                        Rc::new(|_: ()| {}),
                    );
                    Effect::NONE
                }
                PublicAction::OpenVideo(url) => {
                    environment.open_window(
                        window,
                        ImageWindowState(url, ImageWindowKind::Video),
                        1024.,
                        768.,
                        "Video",
                        Rc::new(|_: ()| {}),
                    );
                    Effect::NONE
                }
            }
        }

        Action::StatusMutationResult(data, _original_status, mutation) => match data {
            Ok(status) => {
                let status = StatusViewModel::new(&status);
                let id = status.id.clone();
                let user_id = status.account.id.clone();

                let action_block = |mut s: RefMut<Data>| {
                    match mutation {
                        StatusMutation::Bookmark(a) => {
                            s.changed_bookmark(&status, a);
                        }
                        StatusMutation::Favourite(a) => {
                            s.changed_favorite(&status, a);
                        }
                        StatusMutation::Boost(a) => {
                            s.changed_boost(&status, a);
                        }
                    }

                    s.mutate_post(id, user_id, |post| {
                        *post = status.clone();

                        match mutation {
                            StatusMutation::Bookmark(o) => {
                                post.is_bookmarked = o;
                            }
                            StatusMutation::Favourite(o) => {
                                post.update_favorited(o);
                            }
                            StatusMutation::Boost(o) => {
                                post.update_reblog(o);
                            }
                        }
                    })
                };

                if !environment.storage.with_mutation(action_block) {
                    log::error!(
                        "Could not find post {:?} {:?}",
                        &status.id,
                        &status.account.id
                    );
                }
                Effect::NONE
            }
            Err(e) => {
                log::error!("Error {mutation:?} {e:?}");
                Effect::NONE
            }
        },
        Action::Post(kind) => {
            // Convert a notifications reply into a private reply (mastodon web)
            // does the same thing
            let kind = match (kind, reducer_state.active_tab) {
                (PostKind::Reply(n), UiTab::Mentions) => PostKind::ReplyPrivate(n),
                (PostKind::Reply(n), _) => PostKind::Reply(n),
                (PostKind::ReplyPrivate(n), _) => PostKind::ReplyPrivate(n),
                (PostKind::Post, _) => PostKind::Post,
            };
            if inline_window {
                reducer_state.is_replying = Some((kind, vec![]));
                return Effect::NONE;
            }
            let title = match &kind {
                &PostKind::Post => NEW_TOOT_TITLE.to_string(),
                &PostKind::Reply(ref status) | &PostKind::ReplyPrivate(ref status) => {
                    format!("Reply to {}", status.account.acct)
                }
            };
            let Some(ref account) = reducer_state.user_account else {
                return Effect::NONE
            };

            let waker = context.updater().clone();

            environment.open_window(
                window,
                PostWindowState::new(
                    kind,
                    // notifier,
                    Vec::new(),
                    account.clone(),
                ),
                NEW_TOOT_SIZE.0,
                NEW_TOOT_SIZE.1,
                title,
                Rc::new(move |a| {
                    if let Some(o) = handle_new_post_window_event(a) {
                        (waker)(o)
                    }
                }),
            );
            Effect::NONE
        }
        Action::PostCancel => {
            reducer_state.is_replying = None;
            Effect::NONE
        }
        Action::PostDone(status) => environment.storage.with_mutation(|mut storage| {
            if let Some(ref reply_id) = status.in_reply_to_id {
                storage.replied_to_status(reply_id);
            }
            storage.possibly_update_conversation_with_reply(&status);
            Effect::NONE
        }),
        Action::Preferences => {
            let waker = context.updater().clone();
            let mapped_waker =
                Rc::new(move |action: PreferencesChange| waker(Action::PreferencesChanged(action)));
            environment.open_window(
                window,
                PreferencesWindowState::new(),
                500.,
                300.,
                loc!("Settings"),
                mapped_waker,
            );
            Effect::NONE
        }
        Action::PreferencesChanged(change) => {
            match change {
                PreferencesChange::Direction => {
                    context.send_children(action);
                    if let Ok(s) = environment.repository.config() {
                        reducer_state.ui_settings = s;
                    }
                }
                PreferencesChange::PostWindow => {
                    reducer_state.ui_settings = environment.repository.config().unwrap_or_default();
                }
            }
            Effect::NONE
        }
        Action::AppEvent(AppEvent::FileEvent(event)) => {
            match event {
                FileEvent::Hovering(valid) => reducer_state.flags.is_dropping = valid,
                FileEvent::Dropped(images) => {
                    reducer_state.flags.is_dropping = false;
                    if inline_window {
                        if reducer_state.is_replying.is_some() {
                            // send the action with the updated media
                            context.send_children(Action::AppEvent(AppEvent::FileEvent(
                                FileEvent::Dropped(images),
                            )));
                            return Effect::NONE;
                        }
                        // open window with media
                        reducer_state.is_replying = Some((PostKind::Post, images));
                        return Effect::NONE;
                    }
                    let Some(ref account) = reducer_state.user_account else {
                        return Effect::NONE
                    };
                    let waker = context.updater().clone();
                    environment.open_window(
                        window,
                        PostWindowState::new(PostKind::Post, images, account.clone()),
                        NEW_TOOT_SIZE.0,
                        NEW_TOOT_SIZE.1,
                        NEW_TOOT_TITLE,
                        Rc::new(move |a| {
                            if let Some(o) = handle_new_post_window_event(a) {
                                (waker)(o)
                            }
                        }),
                    );
                }
                FileEvent::Cancelled => reducer_state.flags.is_dropping = false,
            }
            Effect::NONE
        }
        Action::AppEvent(AppEvent::FocusChange(f)) => {
            // FIXME: Remove the JS, inject a class at
            if f {
                // if we get the focus, disable the post menu option
                environment.platform.update_menu(window, |options| {
                    options.enable_postwindow = false;
                });
            }
            Effect::NONE
        }
        Action::AppEvent(AppEvent::ClosingWindow) => {
            // FIXME: Maybe stop listening to events?
            Effect::NONE
        }
        Action::AppEvent(AppEvent::MenuEvent(m)) => {
            match m {
                MainMenuEvent::NewPost => Effect::action(Action::Post(PostKind::Post)),
                MainMenuEvent::Logout => {
                    environment.platform.update_menu(window, |options| {
                        options.enable_postwindow = false;
                    });
                    Effect::action(Action::Logout)
                }
                MainMenuEvent::ScrollDown | MainMenuEvent::ScrollUp => {
                    context.send_children(action);
                    Effect::NONE
                }
                MainMenuEvent::TextSizeIncrease
                | MainMenuEvent::TextSizeDecrease
                | MainMenuEvent::TextSizeReset => {
                    let a = ChangeTextsizeBehaviour::handle(
                        window,
                        m,
                        &mut reducer_state.ui_settings,
                        environment,
                    );
                    environment
                        .repository
                        .set_config(&reducer_state.ui_settings);
                    a
                }
                MainMenuEvent::EbouHelp => Effect::action(Action::Public(PublicAction::OpenLink(
                    "https://terhech.de/ebou".to_string(),
                ))),
                MainMenuEvent::Settings => Effect::action(Action::Preferences),
                MainMenuEvent::Reload => {
                    context.send_children(action);
                    Effect::NONE
                }
                _ => {
                    // Bubble Up
                    context.send_children(action);
                    Effect::NONE
                }
            }
        }
        Action::MessageEvent(message) => {
            environment.storage.with_mutation(|mut storage| {
                storage.handle_push_message(message, reducer_state.ui_settings.direction)
            });
            Effect::NONE
        }
        Action::ClearError => {
            reducer_state.error = None;
            Effect::NONE
        }
        Action::Logout => {
            reducer_state.flags.logging_in = true;
            environment.platform.loggedout_toolbar(window);
            let Some(user) = environment.repository.users().ok().and_then(|e| e.first().cloned()) else {
                return Effect::NONE
            };
            let _ = environment.repository.remove_user(user.id.clone());
            Effect::future(
                async move {
                    model
                        .logout(
                            user.app_client_id.clone(),
                            user.app_client_secret.clone(),
                            user.token_access_token.clone(),
                        )
                        .await
                },
                Action::LogoutDone,
            )
        }
        Action::LogoutDone(_) => {
            reducer_state.logged_in = false;
            reducer_state.did_logout.set(Some(true));
            reducer_state.flags.logging_in = false;
            environment.platform.update_menu(window, |config| {
                config.logged_in = true;
            });
            Effect::NONE
        }
    }
}

fn mutate_status<'a>(
    storage: &RefPublisher<Data>,
    mutation: StatusMutation,
    model: Model,
    status: StatusViewModel,
    action: impl Fn(&mut StatusViewModel),
) -> Effect<'a, Action> {
    let id = status.id.clone();
    let user_id = status.account.id.clone();

    let status_clone = status;

    if !storage.with_mutation(|mut s| {
        s.mutate_post(id.clone(), user_id.clone(), |post| {
            action(post);
        })
    }) {
        log::error!("Could not find post {id:?} {user_id:?}");
        return Effect::NONE;
    }

    let mapper = move |data| Action::StatusMutationResult(data, status_clone.clone(), mutation);

    let new_status = mutation.new_status();
    match mutation {
        StatusMutation::Bookmark(_) => Effect::future(
            async move { model.set_bookmark(id.0, new_status).await },
            mapper,
        ),
        StatusMutation::Favourite(_) => Effect::future(
            async move { model.set_favourite(id.0, new_status).await },
            mapper,
        ),
        StatusMutation::Boost(_) => Effect::future(
            async move { model.set_reblog(id.0, new_status).await },
            mapper,
        ),
    }
}

#[derive(Clone)]
pub enum Action {
    Login,
    LoggedIn(Result<Account, String>),
    DataUpdated,

    SelectAccount(AccountViewModel),
    SelectNotifications(AccountViewModel),
    SelectConversation(StatusId),
    SelectMore(MoreSelection),

    Public(PublicAction),
    StatusMutationResult(Result<Status, String>, StatusViewModel, StatusMutation),

    Post(PostKind),
    PostDone(Status),
    PostCancel,

    Preferences,
    PreferencesChanged(PreferencesChange),

    AppEvent(AppEvent),
    MessageEvent(Message),

    ClearError,
    Logout,
    LogoutDone(Result<(), String>),
}

impl std::fmt::Debug for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Login => write!(f, "Login"),
            Self::DataUpdated => write!(f, "DataUpdated"),
            Self::Logout => write!(f, "Logout"),
            Self::LogoutDone(_) => write!(f, "LogoutDone"),
            Self::ClearError => write!(f, "ClearError"),
            Self::LoggedIn(arg0) => f.debug_tuple("LoggedIn").field(arg0).finish(),

            Self::SelectAccount(arg0) => f.debug_tuple("SelectAccount").field(arg0).finish(),
            Self::SelectNotifications(arg0) => {
                f.debug_tuple("SelectNotifications").field(arg0).finish()
            }

            Self::SelectConversation(arg0) => {
                f.debug_tuple("SelectConversation").field(arg0).finish()
            }
            Self::SelectMore(arg0) => f.debug_tuple("SelectMore").field(arg0).finish(),
            Self::StatusMutationResult(arg0, arg1, arg2) => f
                .debug_tuple("StatusMutationResult")
                .field(arg0)
                .field(arg1)
                .field(arg2)
                .finish(),
            Self::Public(_arg0) => f.debug_tuple("Public").finish(),
            Self::Post(kind) => f.debug_tuple("PostNew").field(kind).finish(),
            Self::PostDone(kind) => f.debug_tuple("PostDone").field(&kind.id).finish(),
            Self::PostCancel => f.debug_tuple("PostCancel").finish(),
            Self::Preferences => write!(f, "Preferences"),
            Self::PreferencesChanged(_) => write!(f, "PreferencesChanged"),
            Self::AppEvent(kind) => f.debug_tuple("AppEvent").field(&kind).finish(),
            Self::MessageEvent(kind) => f.debug_tuple("MessageEvent").field(&kind).finish(),
        }
    }
}

fn handle_new_post_window_event(action: PostAction) -> Option<Action> {
    match action {
        PostAction::PostResult(Ok(status)) => Some(Action::PostDone(status)),
        PostAction::Close => None,
        PostAction::AppEvent(AppEvent::MenuEvent(evt)) => {
            Some(Action::AppEvent(AppEvent::MenuEvent(evt)))
        }
        _ => None,
    }
}

impl From<(StatusAction, StatusViewModel)> for PublicAction {
    fn from(value: (StatusAction, StatusViewModel)) -> Self {
        match value.0 {
            StatusAction::Clicked => PublicAction::Conversation(value.1.clone()),
            StatusAction::OpenTag(a) => PublicAction::OpenTag(a),
            StatusAction::OpenLink(a) => PublicAction::OpenLink(a),
            StatusAction::OpenAccount(a) => PublicAction::OpenProfileLink(a),
            StatusAction::Reply => PublicAction::Post(PostKind::Reply(value.1.clone())),
            StatusAction::Boost(a) => {
                PublicAction::StatusMutation(StatusMutation::Boost(a), value.1.clone())
            }
            StatusAction::Favorite(a) => {
                PublicAction::StatusMutation(StatusMutation::Favourite(a), value.1.clone())
            }
            StatusAction::Bookmark(a) => {
                PublicAction::StatusMutation(StatusMutation::Bookmark(a), value.1.clone())
            }
            StatusAction::OpenImage(a) => PublicAction::OpenImage(a),
            StatusAction::OpenVideo(a) => PublicAction::OpenVideo(a),
            StatusAction::Copy(a) => PublicAction::Copy(a),
        }
    }
}
