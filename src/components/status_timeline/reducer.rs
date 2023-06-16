use std::time::Duration;

use enumset::EnumSet;

use super::providers::AnyTimelineProvider;
use crate::environment::model::Status;
use crate::environment::types::{AppEvent, MainMenuEvent, TimelineDirection, UiConfig};
use crate::environment::Environment;
use crate::view_model::{
    AccountId, AccountViewModel, AccountVisibility, StatusId, StatusViewModel,
};
use crate::PublicAction;
use navicula::Effect;

#[derive(Clone, Debug)]
pub struct State {
    pub is_loading: bool,
    pub is_loading_more: bool,
    pub can_load_more: bool,
    pub provider: AnyTimelineProvider,
    pub error: Option<String>,
    /// If there is a related account to these posts, it is set here
    pub account: Option<AccountViewModel>,
    /// This is needed because every provider will present different data
    pub posts: Vec<StatusViewModel>,
    pub ui_settings: UiConfig,
    pub should_scroll_to_newest: bool,
    pub forced_direction: Option<TimelineDirection>,
    pub identifier: String,
    pub known_conversations: Vec<StatusId>,
}

pub type ViewStore<'a> = navicula::ViewStore<'a, super::TimelineReducer>;

#[derive(Clone)]
pub enum Action {
    Initial,
    LoadData,
    LoadedData(Result<Vec<Status>, String>, bool),
    LoadMoreData(Option<StatusId>),
    LoadedMoreData(Result<Vec<Status>, String>),
    ShouldReloadSoft,
    ReloadSoft(bool),
    DataChanged,
    AccountVisibility(AccountId, AccountVisibility),
    Public(PublicAction),
    AppEvent(AppEvent),
}

impl std::fmt::Debug for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Initial => write!(f, "Initial"),
            Self::LoadData => write!(f, "LoadData"),
            Self::LoadedData(_arg0, _arg1) => f.debug_tuple("LoadedData").finish(),
            Self::LoadMoreData(_arg0) => f.debug_tuple("LoadMoreData").finish(),
            Self::LoadedMoreData(_arg0) => f.debug_tuple("LoadedMoreData").finish(),
            Self::ShouldReloadSoft => write!(f, "ShouldReloadSoft"),
            Self::ReloadSoft(arg0) => f.debug_tuple("ReloadSoft").field(arg0).finish(),
            Self::DataChanged => write!(f, "DataChanged"),
            Self::AccountVisibility(arg0, arg1) => f
                .debug_tuple("AccountVisibility")
                .field(arg0)
                .field(arg1)
                .finish(),
            Self::Public(arg0) => f.debug_tuple("Public").field(arg0).finish(),
            Self::AppEvent(arg0) => f.debug_tuple("AppEvent").field(arg0).finish(),
        }
    }
}

impl State {
    pub fn new(
        provider: AnyTimelineProvider,
        ui_settings: UiConfig,
        account: Option<AccountViewModel>,
    ) -> Self {
        let forced_direction = provider.forced_direction();
        Self {
            provider,
            is_loading: Default::default(),
            is_loading_more: Default::default(),
            can_load_more: Default::default(),
            error: Default::default(),
            account,
            posts: Default::default(),
            ui_settings,
            should_scroll_to_newest: false,
            forced_direction,
            identifier: Default::default(),
            known_conversations: Vec::new(),
        }
    }

    pub fn direction(&self) -> TimelineDirection {
        self.forced_direction.unwrap_or(self.ui_settings.direction)
    }
}

pub fn reduce<'a>(
    context: &'a impl navicula::types::MessageContext<Action, PublicAction, ()>,
    action: Action,
    state: &'a mut State,
    environment: &'a Environment,
) -> Effect<'static, Action> {
    log::trace!("{action:?}");
    let window = context.window();
    state.ui_settings = environment.repository.config().unwrap_or_default();

    match action {
        Action::Initial => {
            let identifier = format!("status_timeline_store_data_{}", state.provider.identifier())
                .replace(' ', "-");
            state.identifier = identifier;

            let reload_action = if state.provider.should_auto_reload() {
                Effect::timer(
                    Duration::from_secs(45),
                    Action::ShouldReloadSoft,
                    &state.identifier,
                )
            } else {
                Effect::NONE
            };

            Effect::merge3(
                environment
                    .storage
                    .subscribe(&state.identifier, context, |_| Action::DataChanged),
                Effect::action(Action::LoadData),
                reload_action,
            )
        }
        Action::ShouldReloadSoft => {
            let id = &state.identifier;
            let js = format!(
                r#"
                return document.getElementById('{id}').scrollTop
                "#
            );
            return Effect::ui_future(js, |v| {
                let Some(position) = v.as_f64() else {
                       return None;
                    };
                let reload = position <= 50.0;
                Some(Action::ReloadSoft(reload))
            });
        }
        Action::ReloadSoft(reload) => {
            if !reload {
                return Effect::NONE;
            }
            state.is_loading_more = true;
            let ft = state.provider.request_data(None);
            Effect::future(ft, |d| Action::LoadedData(d, true))
        }
        Action::LoadData => {
            state.is_loading = true;
            let ft = state.provider.request_data(None);
            Effect::future(ft, |d| Action::LoadedData(d, false))
        }
        Action::LoadMoreData(id) => {
            let Some(id) = id else {
                state.can_load_more = false;
                return Effect::NONE;
            };
            state.is_loading_more = true;
            let ft = state.provider.request_data(Some(id));
            Effect::future(ft, Action::LoadedMoreData)
        }
        Action::LoadedData(data, was_reload) => {
            state.is_loading = false;
            state.is_loading_more = false; // enabled via was_reload
            let Ok(updates) = data else {
                state.can_load_more = false;
                return Effect::NONE
            };

            let length = updates.len();

            let m = 3;
            if length > m {
                if let Some(m) = updates.get(length - m) {
                    log::debug!("set marker {} for {}", &&m.account.id, &m.id);
                    environment
                        .repository
                        .set_timeline_marker(&m.account.id, &m.id);
                }
            }

            let possible_scroll = state.provider.scroll_to_item(&updates);

            let direction = state.direction();
            state.can_load_more = state
                .provider
                .process_new_data(&updates, direction, was_reload);
            state.posts = state.provider.data(direction);

            environment.platform.update_menu(window, |config| {
                config.enable_scroll = true;
            });

            if was_reload {
                return Effect::NONE;
            }

            // if we're supposed to scroll to the newest, have to do some more work
            // this is based on whether we have a scroll id *and* whether the direction
            // is down
            if direction == TimelineDirection::NewestTop {
                return Effect::NONE;
            }
            let Some(scroll_id) = possible_scroll else {
                return Effect::NONE
            };

            let dom_id = scroll_id.dom_id();
            return Effect::ui(format!(
                r#"
                setTimeout(() => {{
                    document.getElementById("{dom_id}").scrollIntoView({{ behavior: "auto", block: "end" }});
                    }}, 100);
                "#
            ));
        }
        Action::LoadedMoreData(result) => {
            state.is_loading_more = false;
            let Ok(batch) = result else {
                return Effect::NONE
            };
            if let Some(ref account) = state.account {
                environment.storage.with_mutation(|mut storage| {
                    if batch.is_empty() {
                        storage.accounts_no_older_data.insert(account.id.clone());
                    } else {
                        storage.accounts_no_older_data.remove(&account.id);
                    }
                });
            }

            let direction = state.direction();
            state.can_load_more = state.provider.process_new_data(&batch, direction, false);
            state.posts = state.provider.data(direction);
            environment
                .storage
                .with_mutation(|mut s| s.update_account_historical_data(&batch, &direction));
            Effect::NONE
        }
        Action::DataChanged => {
            state.posts = state.provider.data(state.direction());
            environment.storage.with(|data| {
                state.known_conversations = data.conversations.keys().cloned().collect();
            });
            Effect::NONE
        }
        Action::Public(action) => {
            context.send_parent(action);
            Effect::NONE
        }
        Action::AccountVisibility(account, visibility) => {
            let current = state
                .ui_settings
                .visibility
                .entry(account.0)
                .or_insert(EnumSet::all());
            if current.contains(visibility) {
                current.remove(visibility);
            } else {
                current.insert(visibility);
            }
            environment.repository.set_config(&state.ui_settings);
            Effect::NONE
        }
        Action::AppEvent(AppEvent::MenuEvent(m)) => match m {
            MainMenuEvent::ScrollDown | MainMenuEvent::ScrollUp => {
                // this is kinda ui code, but there's no good way to handle events in the UI
                // layer yet.
                // find the last post for this account
                if state.posts.is_empty() {
                    return Effect::NONE;
                }
                let (id, direction) = match m {
                    MainMenuEvent::ScrollDown => {
                        (state.posts.first().as_ref().unwrap().id.dom_id(), "end")
                    }
                    MainMenuEvent::ScrollUp => {
                        (state.posts.last().as_ref().unwrap().id.dom_id(), "start")
                    }
                    _ => panic!(),
                };
                return Effect::ui(format!(
                    "document.getElementById(\"{id}\").scrollIntoView({{ behavior: \"smooth\", block: \"{direction}\" }});"
                ));
            }
            MainMenuEvent::Reload => {
                state.provider.reset();
                state.posts = Vec::new();
                return Effect::action(Action::LoadData);
            }
            _ => Effect::NONE,
        },
        Action::AppEvent(_) => Effect::NONE,
    }
}
