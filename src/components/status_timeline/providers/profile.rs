use super::TimelineProvider;
use crate::{
    environment::{types::TimelineDirection, Environment},
    view_model::{AccountId, StatusId, StatusViewModel},
};
use futures_util::Future;
use megalodon::entities::Status;
use std::pin::Pin;

/// A provider that just loads the users own toots
pub struct UserProfileTimelineProvider {
    environment: Environment,
    account: AccountId,
    identifier: String,
}

impl UserProfileTimelineProvider {
    pub fn new(environment: Environment, account: AccountId) -> Self {
        let identifier = format!("UserProfileTimelineProvider-{}", account.0);
        Self {
            environment,
            account,
            identifier,
        }
    }
}

impl std::fmt::Debug for UserProfileTimelineProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UserProfileTimelineProvider")
            .field("account", &self.account)
            .finish()
    }
}

impl TimelineProvider for UserProfileTimelineProvider {
    type Id = StatusId;
    type Element = Status;
    type ViewModel = StatusViewModel;
    fn should_auto_reload(&self) -> bool {
        false
    }

    fn identifier(&self) -> &str {
        &self.identifier
    }

    fn forced_direction(&self) -> Option<TimelineDirection> {
        Some(TimelineDirection::NewestTop)
    }

    fn reset(&self) {
        self.environment
            .storage
            .with_mutation(|mut storage| storage.account_timeline.clear())
    }

    fn scroll_to_item(&self, _updates: &[Status]) -> Option<StatusId> {
        None
    }

    fn request_data(
        &self,
        after: Option<StatusId>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Status>, String>> + Send>> {
        let after = after.map(|e| e.0);
        let model = self.environment.model.clone();
        let id = self.account.0.clone();
        Box::pin(async move { model.user_timeline(id, after, None, Some(40)).await })
    }

    fn process_new_data(
        &self,
        updates: &[Status],
        _direction: TimelineDirection,
        is_reload: bool,
    ) -> bool {
        let can_load_more = !updates.is_empty();
        self.environment
            .storage
            .with_mutation(|mut storage| storage.merge_account(updates, &self.account, is_reload));
        can_load_more
    }

    fn data(&self, _direction: TimelineDirection) -> Vec<StatusViewModel> {
        self.environment
            .storage
            .with(|storage| {
                let o = storage.account_timeline.get(&self.account);
                o.cloned()
            }).unwrap_or_default()
    }
}
