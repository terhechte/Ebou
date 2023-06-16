//! Control a ZStack of Components for drilling down a navigation hierachy
// use crate::components::profiles::SelectedProfile;
use crate::components::profile::{Profile, State as ProfileState};
use crate::components::profiles::ProfilesView;
use crate::components::status_timeline::TimelineComponent;
use crate::environment::types::{AppEvent, UiConfig};
use crate::environment::Model;
use crate::view_model::{AccountViewModel, StatusId};
use crate::widgets::*;
use crate::PublicAction;
use crate::{environment::Environment, loc};
use dioxus::prelude::*;
use navicula::{self, reducer::ChildReducer, types::MessageContext, Effect, Reducer};

use super::conversation::ConversationReducer;
use super::profile::ProfileReducer;
use super::profiles::{
    AnyProfilesTimelineProvider, FollowersTimelineProvider, ProfilesKind, ProfilesReducer,
};
use super::status_timeline::{
    AccountTimelineProvider, AnyTimelineProvider, BookmarkTimelineProvider,
    FavoritesTimelineProvider, GroupedAccountTimelineProvider, LocalTimelineProvider,
    NotificationsTimelineProvider, PublicTimelineProvider, TimelineReducer,
    UserProfileTimelineProvider,
};

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum Action {
    Initial,
    PushProfile(AccountViewModel),
    ResolvePushProfile(String),
    SelectConversation(StatusId),
    CloseConversation,
    AppEvent(AppEvent),
    CloseCurrent,
    PublicAction(PublicAction),
    ConversationAction(PublicAction),
}

/// Fixme: Move to the status_timeline to provide a better anytimeline provider
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RootTimelineKind {
    GroupedAccount(AccountViewModel),
    Notifications(AccountViewModel),
    Account(AccountViewModel),
    UserProfile(AccountViewModel),
    Bookmarks(AccountViewModel),
    Favorites(AccountViewModel),
    Federated(AccountViewModel),
    Local(AccountViewModel),
    Relationship(AccountViewModel, ProfilesKind),
}

#[derive(Debug, Clone)]
pub enum ProviderKind {
    Timeline(AnyTimelineProvider),
    Profile(AnyProfilesTimelineProvider),
}

impl From<AnyTimelineProvider> for ProviderKind {
    fn from(value: AnyTimelineProvider) -> Self {
        Self::Timeline(value)
    }
}

impl From<AnyProfilesTimelineProvider> for ProviderKind {
    fn from(value: AnyProfilesTimelineProvider) -> Self {
        Self::Profile(value)
    }
}

impl RootTimelineKind {
    pub fn as_provider(&self, environment: &Environment, model: &Model) -> ProviderKind {
        match self {
            RootTimelineKind::GroupedAccount(a) => AnyTimelineProvider::new(
                GroupedAccountTimelineProvider::new(
                    a.id.clone(),
                    model.clone(),
                    environment.clone(),
                ),
                &a.id,
            )
            .into(),
            RootTimelineKind::Account(a) => {
                AnyTimelineProvider::new(AccountTimelineProvider::new(environment.clone()), &a.id)
                    .into()
            }
            RootTimelineKind::UserProfile(a) => AnyTimelineProvider::new(
                UserProfileTimelineProvider::new(environment.clone(), a.id.clone()),
                &a.id,
            )
            .into(),
            RootTimelineKind::Bookmarks(a) => {
                AnyTimelineProvider::new(BookmarkTimelineProvider::new(environment.clone()), &a.id)
                    .into()
            }
            RootTimelineKind::Favorites(a) => {
                AnyTimelineProvider::new(FavoritesTimelineProvider::new(environment.clone()), &a.id)
                    .into()
            }
            RootTimelineKind::Federated(a) => {
                AnyTimelineProvider::new(PublicTimelineProvider::new(environment.clone()), &a.id)
                    .into()
            }
            RootTimelineKind::Local(a) => {
                AnyTimelineProvider::new(LocalTimelineProvider::new(environment.clone()), &a.id)
                    .into()
            }
            RootTimelineKind::Relationship(a, k) => AnyProfilesTimelineProvider::new(
                FollowersTimelineProvider::new(k.clone(), environment),
                &a.id,
            )
            .into(),
            RootTimelineKind::Notifications(a) => AnyTimelineProvider::new(
                NotificationsTimelineProvider::new(environment.clone(), a.id.clone()),
                &a.id,
            )
            .into(),
        }
    }

    pub fn model(&self) -> AccountViewModel {
        match self {
            RootTimelineKind::GroupedAccount(a) => a.clone(),
            RootTimelineKind::Notifications(a) => a.clone(),
            RootTimelineKind::Account(a) => a.clone(),
            RootTimelineKind::UserProfile(a) => a.clone(),
            RootTimelineKind::Bookmarks(a) => a.clone(),
            RootTimelineKind::Favorites(a) => a.clone(),
            RootTimelineKind::Federated(a) => a.clone(),
            RootTimelineKind::Local(a) => a.clone(),
            RootTimelineKind::Relationship(a, _) => a.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct State {
    root_timeline_kind: RootTimelineKind,
    root_provider: Option<ProviderKind>,
    stack: Vec<AccountViewModel>,
    // Each stack can always have *one* conversation open
    current_conversation: Option<StatusId>,
    pub ui_settings: UiConfig,
}

impl State {
    pub fn new(kind: RootTimelineKind) -> Self {
        Self {
            root_timeline_kind: kind,
            root_provider: None,
            stack: Vec::new(),
            current_conversation: None,
            ui_settings: UiConfig::default(),
        }
    }
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum Message {
    AppEvent(AppEvent),
    SelectConversation(StatusId),
}

#[derive(Debug, Clone)]
pub enum DelegateMessage {
    PublicAction(PublicAction),
    ConversationAction(crate::PublicAction),
}

pub struct StackReducer;
impl Reducer for StackReducer {
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
            state.ui_settings = environment.repository.config().unwrap_or_default();
            state.root_provider = Some(
                state
                    .root_timeline_kind
                    .as_provider(environment, &environment.model),
            );
        }
        Action::PushProfile(p) => {
            state.stack.push(p);
        }
        Action::ResolvePushProfile(p) => {
            let Some(g) = crate::helper::parse_user_url(&p) else {
                return Effect::NONE
            };
            let model = environment.model.clone();
            return Effect::future(
                async move {
                    model
                        .search_account(g, false)
                        .await
                        .ok()
                        .and_then(|a| a.first().cloned())
                },
                |account| {
                    if let Some(n) = account.map(|a| AccountViewModel::new(&a)) {
                        Action::PushProfile(n)
                    } else {
                        // If we couldn't resolve it, open in browser
                        Action::PublicAction(PublicAction::OpenLink(p))
                    }
                },
            );
        }
        Action::CloseCurrent => {
            state.stack.pop();
        }
        Action::SelectConversation(c) => {
            state.current_conversation = Some(c);
        }
        Action::CloseConversation => {
            state.current_conversation = None;
        }
        Action::AppEvent(e) => {
            // FIXME: Should only send to the most recent child..
            context.send_children(Message::AppEvent(e));
            // For whatever reason this doesn't work
            // if let Some(last) = context.children().first() {
            //     last(Message::AppEvent(e));
            // }
        }
        Action::PublicAction(e) => match e {
            PublicAction::Conversation(c) => {
                state.current_conversation = Some(c.id);
            }
            _ => context.send_parent(DelegateMessage::PublicAction(e)),
        },
        Action::ConversationAction(e) => {
            context.send_parent(DelegateMessage::ConversationAction(e))
        }
    }
    Effect::NONE
}

pub type ViewStore<'a> = navicula::ViewStore<'a, StackReducer>;

#[inline_props]
pub fn Stack<'a>(cx: Scope<'a>, store: ViewStore<'a>) -> Element<'a> {
    let Some(provider) = store.root_provider.as_ref() else {
        return render!(div {})
    };
    render! {
        div {
            style: "position: relative; flex-basis: 520px; max-width: 520px; flex-shrink: 0;",
            match provider {
                ProviderKind::Timeline(t) => rsx!(TimelineInStack {
                    store: store,
                    provider: t.clone()
                }),
                ProviderKind::Profile(t) => rsx!(ProfilesInStack {
                    store: store,
                    provider: t.clone()
                })
            }

            for profile in store.stack.iter() {
                StackEntry {
                    store: store,
                    profile: profile
                }
            }
        }
        store
            .current_conversation
            .as_ref()
            .map(|c| rsx!(ConversationComponent { conversation: c.clone(), store: store }))
    }
}

#[inline_props]
pub fn StackEntry<'a>(
    cx: Scope<'a>,
    store: &'a ViewStore<'a>,
    profile: &'a AccountViewModel,
) -> Element<'a> {
    // only animate the stack the first time it appears
    let first_state = use_state(cx, || false);
    let is_first_time = first_state.get() == &false;
    if is_first_time {
        first_state.set(true);
    }
    let cls = if is_first_time {
        "slide-in-blurred-bottom"
    } else {
        ""
    };
    render! {
        div {
            class: "popup {cls}",
            ProfileStackEntry {
                store: store,
                profile: profile
            }
        }
    }
}

#[inline_props]
pub fn TimelineInStack<'a>(
    cx: Scope<'a>,
    store: &'a ViewStore<'a>,
    provider: AnyTimelineProvider,
) -> Element<'a> {
    use crate::components::status_timeline::State as TimelineState;
    render! {
        TimelineComponent {
            store: store.host_with(
                cx,
                provider,
                |p| TimelineState::new(p, store.ui_settings.clone(), Some(store.root_timeline_kind.model())))
        }
    }
}

impl ChildReducer<StackReducer> for TimelineReducer {
    fn to_child(message: <StackReducer as Reducer>::Message) -> Option<<Self as Reducer>::Action> {
        use crate::components::status_timeline::Action as TimelineAction;
        match message {
            Message::AppEvent(a) => Some(TimelineAction::AppEvent(a)),
            Message::SelectConversation(_) => None,
        }
    }

    fn from_child(
        message: <Self as Reducer>::DelegateMessage,
    ) -> Option<<StackReducer as Reducer>::Action> {
        match message {
            PublicAction::Conversation(s) => Some(Action::SelectConversation(s.id)),
            PublicAction::OpenProfile(p) => Some(Action::PushProfile(p)),
            PublicAction::OpenProfileLink(p) => Some(Action::ResolvePushProfile(p)),
            _ => Some(Action::PublicAction(message)),
        }
    }
}

#[inline_props]
pub fn ProfilesInStack<'a>(
    cx: Scope<'a>,
    store: &'a ViewStore<'a>,
    provider: AnyProfilesTimelineProvider,
) -> Element<'a> {
    use crate::components::profiles;
    render! {
        ProfilesView {
            store: store.host_with(
                cx,
                provider,
                |a| profiles::ProfilesState::new(a, false),
            )
        }
    }
}

impl ChildReducer<StackReducer> for ProfilesReducer {
    fn to_child(message: <StackReducer as Reducer>::Message) -> Option<<Self as Reducer>::Action> {
        match message {
            Message::AppEvent(a) => Some(super::profiles::ProfilesAction::AppEvent(a)),
            _ => None,
        }
    }

    fn from_child(
        message: <Self as Reducer>::DelegateMessage,
    ) -> Option<<StackReducer as Reducer>::Action> {
        match message {
            super::profiles::ProfilesDelegate::TimelineAction(p) => match p {
                PublicAction::Close => Some(Action::CloseConversation),
                PublicAction::OpenProfile(a) => Some(Action::PushProfile(a)),
                PublicAction::OpenProfileLink(a) => Some(Action::ResolvePushProfile(a)),
                _ => Some(Action::PublicAction(p)),
            },
        }
    }
}

#[inline_props]
fn ConversationComponent<'a>(
    cx: Scope<'a>,
    conversation: StatusId,
    store: &'a ViewStore<'a>,
) -> Element<'a> {
    use crate::components::conversation::{ConversationComponent, State as ConversationState};
    render! {
        ConversationComponent {
            store: store.host_with(cx, conversation, ConversationState::new)
        }
    }
}

impl ChildReducer<StackReducer> for ConversationReducer {
    fn to_child(message: <StackReducer as Reducer>::Message) -> Option<<Self as Reducer>::Action> {
        match message {
            Message::SelectConversation(a) => Some(
                crate::components::conversation::Action::SelectConversation(a),
            ),
            _ => None,
        }
    }

    fn from_child(
        message: <Self as Reducer>::DelegateMessage,
    ) -> Option<<StackReducer as Reducer>::Action> {
        match message {
            PublicAction::Close => Some(Action::CloseConversation),
            PublicAction::OpenProfile(a) => Some(Action::PushProfile(a)),
            PublicAction::OpenProfileLink(a) => Some(Action::ResolvePushProfile(a)),
            _ => Some(Action::ConversationAction(message)),
        }
    }
}

#[inline_props]
fn ProfileStackEntry<'a>(
    cx: Scope<'a>,
    store: &'a ViewStore<'a>,
    profile: &'a AccountViewModel,
) -> Element<'a> {
    render! {
        VStack {
            class: "height-100",
            div {
                class: "hstack toolbar me-auto",
                style: "margin: 0px; margin-bottom: 16px; align-items: baseline;",
                IconButton {
                    icon: crate::icons::ICON_CANCEL,
                    title: loc!("Close"),
                    onclick: move |_| {
                        store.send(Action::CloseCurrent);
                    }
                }
                Label {
                    "{profile.username}"
                }
            }
            InnerProfileStackEntry {
                store: store,
                profile: profile
            }
        }
    }
}

#[inline_props]
fn InnerProfileStackEntry<'a>(
    cx: Scope<'a>,
    store: &'a ViewStore<'a>,
    profile: &'a AccountViewModel,
) -> Element<'a> {
    render! {
        // "bam"
        Profile {
            store: store.host_with(
                cx,
                *profile,
                ProfileState::new
            )
        }
    }
}

impl ChildReducer<StackReducer> for ProfileReducer {
    fn to_child(message: <StackReducer as Reducer>::Message) -> Option<<Self as Reducer>::Action> {
        use crate::components::profile::Action;
        match message {
            Message::AppEvent(a) => Some(Action::AppEvent(a)),
            Message::SelectConversation(_) => todo!(),
        }
    }

    fn from_child(
        message: <Self as Reducer>::DelegateMessage,
    ) -> Option<<StackReducer as Reducer>::Action> {
        match message {
            super::profile::DelegateMessage::TimelineAction(PublicAction::OpenProfile(a)) => {
                Some(Action::PushProfile(a))
            }
            super::profile::DelegateMessage::TimelineAction(PublicAction::OpenProfileLink(a)) => {
                Some(Action::ResolvePushProfile(a))
            }
            super::profile::DelegateMessage::TimelineAction(PublicAction::Conversation(a)) => {
                Some(Action::SelectConversation(a.id))
            }
            super::profile::DelegateMessage::TimelineAction(a) => Some(Action::PublicAction(a)),
        }
    }
}
