use super::TimelineProvider;
use crate::{
    environment::{types::TimelineDirection, Environment},
    view_model::{AccountId, StatusId, StatusViewModel},
};
use futures_util::Future;
use megalodon::entities::Status;
use std::pin::Pin;

/// A provider that shows notifications which were already loaded
pub struct NotificationsTimelineProvider {
    environment: Environment,
    account: AccountId,
}

impl NotificationsTimelineProvider {
    pub fn new(environment: Environment, account: AccountId) -> Self {
        Self {
            environment,
            account,
        }
    }
}

impl std::fmt::Debug for NotificationsTimelineProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NotificationsTimelineProvider")
            .field("account", &self.account)
            .finish()
    }
}

impl TimelineProvider for NotificationsTimelineProvider {
    type Id = StatusId;
    type Element = Status;
    type ViewModel = StatusViewModel;
    fn should_auto_reload(&self) -> bool {
        false
    }

    fn identifier(&self) -> &str {
        std::any::type_name::<Self>()
    }

    fn forced_direction(&self) -> Option<TimelineDirection> {
        Some(TimelineDirection::NewestTop)
    }

    fn reset(&self) {}

    fn scroll_to_item(&self, _updates: &[Status]) -> Option<StatusId> {
        None
    }

    fn request_data(
        &self,
        _after: Option<StatusId>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Status>, String>> + Send>> {
        Box::pin(async move { Ok(Vec::new()) })
    }

    fn process_new_data(
        &self,
        updates: &[Status],
        _direction: TimelineDirection,
        _is_reload: bool,
    ) -> bool {
        !updates.is_empty()
    }

    fn data(&self, _direction: TimelineDirection) -> Vec<StatusViewModel> {
        self.environment
            .storage
            .with(|storage| storage.notification_posts.get(&self.account).cloned())
            .unwrap_or_default()
            .iter()
            .map(|e| e.status.clone())
            .collect()
    }
}
