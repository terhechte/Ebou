mod reducer;
mod view;

pub use reducer::{ProfilesAction, ProfilesDelegate, ProfilesState};
pub use view::ProfilesView;

pub struct ProfilesReducer;
use std::cell::RefCell;

use megalodon::entities::List;
use navicula::reducer::Reducer;

use crate::{
    environment::{model::Account, Environment},
    view_model::{AccountId, AccountViewModel},
};

use super::status_timeline::TimelineProvider;

impl Reducer for ProfilesReducer {
    type Message = reducer::ProfilesMessage;

    type DelegateMessage = reducer::ProfilesDelegate;

    type Action = reducer::ProfilesAction;

    type State = reducer::ProfilesState;

    type Environment = crate::environment::Environment;

    fn reduce<'a, 'b>(
        context: &'a impl navicula::types::MessageContext<
            Self::Action,
            Self::DelegateMessage,
            Self::Message,
        >,
        action: Self::Action,
        state: &'a mut Self::State,
        environment: &'a Self::Environment,
    ) -> navicula::effect::Effect<'b, Self::Action> {
        reducer::reduce(context, action, state, environment)
    }

    fn initial_action() -> Option<Self::Action> {
        Some(reducer::ProfilesAction::Initial)
    }
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum ProfilesKind {
    Followers(AccountId),
    Following(AccountId),
    List(AccountId, List),
}

impl PartialEq for ProfilesKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Followers(l0), Self::Followers(r0)) => l0 == r0,
            (Self::Following(l0), Self::Following(r0)) => l0 == r0,
            (Self::List(l0, _), Self::List(r0, _)) => l0 == r0,
            _ => false,
        }
    }
}

impl Eq for ProfilesKind {}

impl ProfilesKind {
    pub fn id(&self) -> &AccountId {
        match self {
            ProfilesKind::Followers(a) => a,
            ProfilesKind::Following(a) => a,
            ProfilesKind::List(a, _) => a,
        }
    }
}

// Implement a timeline-provider for follows and followers
#[derive(Debug, Clone)]
pub struct FollowersTimelineProvider {
    /// a unique id of this entry
    id: String,
    kind: ProfilesKind,
    /// Followers of this account id
    data: RefCell<Vec<AccountViewModel>>,
    environment: Environment,
}

impl FollowersTimelineProvider {
    pub fn new(kind: ProfilesKind, environment: &Environment) -> Self {
        Self {
            id: format!("followers-{}", &kind.id().0),
            kind,
            data: RefCell::default(),
            environment: environment.clone(),
        }
    }
}

impl TimelineProvider for FollowersTimelineProvider {
    type Id = AccountId;
    type Element = Account;
    type ViewModel = AccountViewModel;
    fn should_auto_reload(&self) -> bool {
        false
    }

    fn identifier(&self) -> &str {
        &self.id
    }

    fn reset(&self) {
        self.data.borrow_mut().clear();
    }

    fn forced_direction(&self) -> Option<crate::environment::types::TimelineDirection> {
        None
    }

    fn request_data(
        &self,
        after: Option<AccountId>,
    ) -> std::pin::Pin<Box<dyn futures_util::Future<Output = Result<Vec<Account>, String>> + Send>>
    {
        let after = after.map(|e| e.0);
        let model = self.environment.model.clone();
        match &self.kind {
            ProfilesKind::Followers(a) => {
                let id = a.clone();
                Box::pin(async move { model.followers(id.0, after).await })
            }
            ProfilesKind::Following(a) => {
                let id = a.clone();
                Box::pin(async move { model.following(id.0, after).await })
            }
            ProfilesKind::List(_, _) => todo!(),
        }
    }

    fn process_new_data(
        &self,
        updates: &[Account],
        _direction: crate::environment::types::TimelineDirection,
        is_reload: bool,
    ) -> bool {
        let mut converted = updates.iter().map(AccountViewModel::new).collect();
        if is_reload {
            self.data.replace(converted);
        } else {
            self.data.borrow_mut().append(&mut converted);
        }
        false
    }

    fn data(
        &self,
        _direction: crate::environment::types::TimelineDirection,
    ) -> Vec<AccountViewModel> {
        self.data.borrow().clone()
    }

    fn scroll_to_item(&self, _updates: &[megalodon::entities::Status]) -> Option<AccountId> {
        todo!()
    }
}

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::Arc,
};

#[derive(Clone)]
pub struct AnyProfilesTimelineProvider {
    provider:
        Arc<dyn TimelineProvider<Id = AccountId, Element = Account, ViewModel = AccountViewModel>>,
    equatable: u64,
}

impl AnyProfilesTimelineProvider {
    pub fn new<T: Hash>(
        provider: impl TimelineProvider<Id = AccountId, Element = Account, ViewModel = AccountViewModel>
            + 'static,
        id: &T,
    ) -> Self {
        let mut hasher = DefaultHasher::new();
        id.hash(&mut hasher);
        let equatable = hasher.finish();
        Self {
            provider: Arc::new(provider),
            equatable,
        }
    }
}

impl PartialEq for AnyProfilesTimelineProvider {
    /// Thing never changes
    fn eq(&self, other: &Self) -> bool {
        self.equatable == other.equatable
    }
}

impl std::fmt::Debug for AnyProfilesTimelineProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnyAccountTimelineProvider")
            .field("provider", &self.provider)
            .field("eq", &self.equatable)
            .finish()
    }
}

impl Eq for AnyProfilesTimelineProvider {}

impl std::ops::Deref for AnyProfilesTimelineProvider {
    type Target =
        Arc<dyn TimelineProvider<Id = AccountId, Element = Account, ViewModel = AccountViewModel>>;

    fn deref(&self) -> &Self::Target {
        &self.provider
    }
}
