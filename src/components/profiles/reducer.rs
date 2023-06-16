use crate::environment::types::{AppEvent, TimelineDirection, UiConfig};
use crate::environment::Environment;
use crate::PublicAction;
use crate::{
    environment::model::Account,
    view_model::{AccountId, AccountViewModel},
};
use navicula::Effect;

pub type ViewStore<'a> = navicula::ViewStore<'a, super::ProfilesReducer>;

pub struct ProfilesState {
    provider: super::AnyProfilesTimelineProvider,
    // if it is nested, don't show additional paddings
    pub is_nested: bool,
    pub ui_settings: UiConfig,
    pub is_loading: bool,
    pub is_loading_more: bool,
    pub profiles: Vec<AccountViewModel>,
    pub error: Option<String>,
    pub can_load_more: bool,
    pub next_profile_id: Option<String>,
    // need this to open more providers from within. not optimal
    pub environment: Option<Environment>,
}

impl ProfilesState {
    pub fn new(provider: super::AnyProfilesTimelineProvider, is_nested: bool) -> Self {
        Self {
            is_nested,
            ui_settings: UiConfig::default(),
            is_loading: false,
            is_loading_more: false,
            provider,
            profiles: Vec::new(),
            error: None,
            can_load_more: true,
            next_profile_id: None,
            environment: None,
        }
    }
}

#[derive(Clone)]
pub enum ProfilesMessage {
    AppEvent(AppEvent),
}

#[derive(Clone)]
pub enum ProfilesDelegate {
    TimelineAction(crate::PublicAction),
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
pub enum ProfilesAction {
    Initial,
    Reload,
    LoadData,
    LoadedData(Result<Vec<Account>, String>),
    LoadMoreData,
    LoadedMoreData(Result<Vec<Account>, String>),
    Select(Option<AccountViewModel>),
    AppEvent(AppEvent),
    TimelineAction(crate::PublicAction),
}

impl std::fmt::Debug for ProfilesAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Initial => write!(f, "Initial"),
            Self::Reload => write!(f, "Reload"),
            Self::LoadData => write!(f, "LoadData"),
            Self::LoadedData(_arg0) => f.debug_tuple("LoadedData").finish(),
            Self::LoadMoreData => write!(f, "LoadMoreData"),
            Self::LoadedMoreData(_arg0) => f.debug_tuple("LoadedMoreData").finish(),
            Self::Select(arg0) => f.debug_tuple("Select").field(arg0).finish(),
            Self::AppEvent(arg0) => f.debug_tuple("AppEvent").field(arg0).finish(),
            Self::TimelineAction(arg0) => f.debug_tuple("TimelineAction").field(arg0).finish(),
        }
    }
}

pub fn reduce<'a>(
    context: &'a impl navicula::types::MessageContext<ProfilesAction, ProfilesDelegate, ProfilesMessage>,
    action: ProfilesAction,
    state: &'a mut ProfilesState,
    environment: &'a Environment,
) -> Effect<'static, ProfilesAction> {
    log::trace!("{action:?}");
    match action {
        ProfilesAction::Initial => {
            state.ui_settings = environment.repository.config().unwrap_or_default();
            state.environment = Some(environment.clone());
            return Effect::action(ProfilesAction::LoadData);
        }
        ProfilesAction::Reload => {
            state.profiles.clear();
            state.is_loading = true;
            return Effect::action(ProfilesAction::LoadData);
        }
        ProfilesAction::LoadData => {
            state.is_loading = true;
            return Effect::future(state.provider.request_data(None), |d| {
                ProfilesAction::LoadedData(d)
            });
        }
        ProfilesAction::LoadedData(data) => {
            state.is_loading = false;
            match data {
                Ok(data) => {
                    state.next_profile_id = data.last().and_then(|e| e.next.clone());
                    state
                        .provider
                        .process_new_data(&data, TimelineDirection::NewestBottom, true);
                }
                Err(e) => state.error = Some(format!("{e:?}")),
            };
            state.profiles = state.provider.data(TimelineDirection::NewestTop);
        }
        ProfilesAction::LoadMoreData => {
            state.is_loading_more = true;
            return Effect::future(
                state
                    .provider
                    .request_data(state.next_profile_id.clone().map(AccountId)),
                ProfilesAction::LoadedMoreData,
            );
        }
        ProfilesAction::LoadedMoreData(result) => {
            state.is_loading_more = false;
            match result {
                Ok(data) => {
                    state.can_load_more = !data.is_empty();
                    state.next_profile_id = data.last().and_then(|e| e.next.clone());
                    state
                        .provider
                        .process_new_data(&data, TimelineDirection::NewestBottom, false);
                }
                Err(e) => state.error = Some(format!("{e:?}")),
            };
            state.profiles = state.provider.data(TimelineDirection::NewestTop);
        }
        ProfilesAction::Select(vm) => {
            if let Some(a) = vm {
                context.send_parent(ProfilesDelegate::TimelineAction(PublicAction::OpenProfile(
                    a,
                )));
            }
        }
        ProfilesAction::TimelineAction(p) => {
            context.send_parent(ProfilesDelegate::TimelineAction(p))
        }
        ProfilesAction::AppEvent(a) => {
            use crate::environment::types::MainMenuEvent;
            match a {
                AppEvent::FocusChange(_) => todo!(),
                AppEvent::MenuEvent(m) => match m {
                    MainMenuEvent::Reload => return Effect::action(ProfilesAction::Reload),
                    MainMenuEvent::ScrollUp | MainMenuEvent::ScrollDown => {
                        if let Some(entry) = match m {
                            MainMenuEvent::ScrollUp => state.profiles.first(),
                            MainMenuEvent::ScrollDown => state.profiles.last(),
                            _ => None,
                        } {
                            let dom_id = format!("a{}", entry.id.0);
                            return Effect::ui(format!(
                                r#"
                            setTimeout(() => {{
                                document.getElementById("{dom_id}").scrollIntoView({{ behavior: "auto", block: "end" }});
                                }}, 100);
                            "#
                            ));
                        };
                    }
                    _ => (),
                },
                _ => (),
            }
            context.send_children(ProfilesMessage::AppEvent(a));
        }
    }
    Effect::NONE
}
