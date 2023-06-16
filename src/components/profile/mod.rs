//! A single profile with posts, follows and followers
use crate::components::profiles::{FollowersTimelineProvider, ProfilesKind};
use crate::components::status_timeline::{TimelineContents, UserProfileTimelineProvider};
use crate::environment::types::{AppEvent, UiConfig};
use crate::environment::Environment;
use crate::view_model::AccountViewModel;
use crate::PublicAction;
use dioxus::prelude::*;
use navicula::{self, reducer::ChildReducer, types::MessageContext, Effect, Reducer};

use super::profile_preview::SelectedProfileTab;
use super::profiles::{AnyProfilesTimelineProvider, ProfilesReducer, ProfilesState, ProfilesView};
use super::status_timeline::{AnyTimelineProvider, TimelineReducer};
use crate::components::profile_preview::{ProfilePageHeader, ProfilePreviewReducer, ProfileState};
use crate::widgets::*;

#[derive(Debug, Clone)]
pub enum Action {
    Initial,
    ChangeTab(SelectedProfileTab),
    TimelineAction(PublicAction),
    AppEvent(AppEvent),
}

#[derive(Debug, Clone)]
pub struct State {
    ui_settings: UiConfig,
    selected_tab: SelectedProfileTab,
    profile: AccountViewModel,
    timeline_provider: Option<AnyTimelineProvider>,
    following_provider: Option<AnyProfilesTimelineProvider>,
    followers_provider: Option<AnyProfilesTimelineProvider>,
}

impl State {
    fn providers(
        &self,
    ) -> Option<(
        &AnyTimelineProvider,
        &AnyProfilesTimelineProvider,
        &AnyProfilesTimelineProvider,
    )> {
        self.timeline_provider
            .as_ref()
            .zip(self.following_provider.as_ref())
            .zip(self.followers_provider.as_ref())
            .map(|((a, b), c)| (a, b, c))
    }
}

impl State {
    pub fn new(profile: AccountViewModel) -> Self {
        Self {
            ui_settings: UiConfig::default(),
            selected_tab: SelectedProfileTab::Posts,
            profile,
            timeline_provider: None,
            following_provider: None,
            followers_provider: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    AppEvent(AppEvent),
}

#[derive(Debug, Clone)]
pub enum DelegateMessage {
    TimelineAction(PublicAction),
}

pub struct ProfileReducer;
impl Reducer for ProfileReducer {
    type Message = Message;

    type DelegateMessage = DelegateMessage;

    type Action = Action;

    type State = State;

    type Environment = crate::environment::Environment;

    fn reduce<'a, 'b>(
        context: &'a impl MessageContext<Self::Action, Self::DelegateMessage, Self::Message>,
        action: Self::Action,
        state: &'a mut Self::State,
        environment: &'a Self::Environment,
    ) -> navicula::effect::Effect<'b, Self::Action> {
        reduce(context, action, state, environment)
    }

    fn initial_action() -> Option<Self::Action> {
        Some(Action::Initial)
    }
}

pub fn reduce<'a>(
    context: &'a impl MessageContext<Action, DelegateMessage, Message>,
    action: Action,
    state: &'a mut State,
    environment: &'a Environment,
) -> Effect<'static, Action> {
    log::trace!("{action:?}");
    match action {
        Action::Initial => {
            let p = UserProfileTimelineProvider::new(environment.clone(), state.profile.id.clone());
            state.timeline_provider = Some(AnyTimelineProvider::new(p, &state.profile.id));
            state.followers_provider = {
                let kind = ProfilesKind::Followers(state.profile.id.clone());
                let p = FollowersTimelineProvider::new(kind, environment);
                Some(AnyProfilesTimelineProvider::new(p, &state.profile.id))
            };
            state.following_provider = {
                let kind = ProfilesKind::Following(state.profile.id.clone());
                let p = FollowersTimelineProvider::new(kind, environment);
                Some(AnyProfilesTimelineProvider::new(p, &state.profile.id))
            };
        }
        Action::ChangeTab(tab) => {
            state.selected_tab = tab;
        }
        Action::TimelineAction(a) => context.send_parent(DelegateMessage::TimelineAction(a)),
        Action::AppEvent(a) => context.send_children(Message::AppEvent(a)),
    }
    Effect::NONE
}

pub type ViewStore<'a> = navicula::ViewStore<'a, ProfileReducer>;

#[inline_props]
pub fn Profile<'a>(cx: Scope<'a>, store: ViewStore<'a>) -> Element<'a> {
    let profile = &store.profile;
    let Some((tp, fp1, fp2)) = store.providers() else {
        return render!(Error {error: "No Timeline"})
    };
    render! {
        div {
            style: "height: 100%; overflow: scroll",
            ProfileHeaderView {
                key: "a{p.id.0}",
                store: store,
                profile: profile,
            }
            ProfilePostTimelineView {
                store: store,
                provider: tp,
                hidden: store.selected_tab != SelectedProfileTab::Posts
            }
            ProfileFollowersView {
                store: store,
                provider: fp2,
                hidden: store.selected_tab != SelectedProfileTab::Followers
            }
            ProfileFollowersView {
                store: store,
                provider: fp1,
                hidden: store.selected_tab != SelectedProfileTab::Following
            }
        }
    }
}

#[inline_props]
pub fn Error<'a>(cx: Scope<'a>, error: &'a str) -> Element<'a> {
    render! {
        div {
            "{error}"
        }
    }
}

#[inline_props]
pub fn ProfileHeaderView<'a>(
    cx: Scope<'a>,
    store: &'a ViewStore<'a>,
    profile: &'a AccountViewModel,
) -> Element<'a> {
    render! {
        ProfilePageHeader {
            key: "b{profile.id.0}",
            store: store.host_with(
                cx,
                *profile,
                |account| ProfileState::new(account, true),
            ),
            selected: store.selected_tab,
            onclick: |a| {
                store.send(Action::ChangeTab(a));
            },
        }
    }
}

impl ChildReducer<ProfileReducer> for ProfilePreviewReducer {
    fn to_child(
        _message: <ProfileReducer as navicula::Reducer>::Message,
    ) -> Option<<Self as navicula::Reducer>::Action> {
        // ProfilePreviewReducer::Message is `()`. We don't need app events there
        None
    }

    fn from_child(
        message: <Self as navicula::Reducer>::DelegateMessage,
    ) -> Option<<ProfileReducer as navicula::Reducer>::Action> {
        Some(crate::components::profile::Action::TimelineAction(message))
    }
}

#[inline_props]
pub fn ProfilePostTimelineView<'a>(
    cx: Scope<'a>,
    store: &'a ViewStore<'a>,
    provider: &'a AnyTimelineProvider,
    hidden: bool,
) -> Element<'a> {
    log::debug!(
        "rerender ProfilePostTimelineView for {}",
        provider.identifier()
    );
    use crate::components::status_timeline::State;
    render!(HideableView {
        class: "flex-direction-column",
        hidden: *hidden,
        TimelineContents {
            store: store.host_with(
                cx,
                *provider,
                |a| State::new(a, store.ui_settings.clone(), None),
            ),
            account_settings: None
            show_profile: false
        }
    })
}

impl ChildReducer<ProfileReducer> for TimelineReducer {
    fn to_child(
        message: <ProfileReducer as navicula::Reducer>::Message,
    ) -> Option<<Self as navicula::Reducer>::Action> {
        use crate::components::status_timeline::Action as TimelineAction;
        match message {
            Message::AppEvent(a) => Some(TimelineAction::AppEvent(a)),
        }
    }

    fn from_child(
        message: <Self as navicula::Reducer>::DelegateMessage,
    ) -> Option<<ProfileReducer as navicula::Reducer>::Action> {
        log::debug!("ChildReducer<ProfileReducer>:TimelineReducer: {message:?}");
        Some(Action::TimelineAction(message))
    }
}

#[inline_props]
pub fn ProfileFollowersView<'a>(
    cx: Scope<'a>,
    store: &'a ViewStore<'a>,
    provider: &'a AnyProfilesTimelineProvider,
    hidden: bool,
) -> Element<'a> {
    render!(HideableView {
        hidden: *hidden,
        ProfilesView {
            store: store.host_with(
                cx,
                *provider,
                |a| ProfilesState::new(a, true)
            )
        }
    })
}

impl ChildReducer<ProfileReducer> for ProfilesReducer {
    fn to_child(
        message: <ProfileReducer as Reducer>::Message,
    ) -> Option<<Self as Reducer>::Action> {
        match message {
            Message::AppEvent(a) => Some(super::profiles::ProfilesAction::AppEvent(a)),
        }
    }

    fn from_child(
        message: <Self as Reducer>::DelegateMessage,
    ) -> Option<<ProfileReducer as Reducer>::Action> {
        match message {
            super::profiles::ProfilesDelegate::TimelineAction(a) => Some(Action::TimelineAction(a)),
        }
    }
}
