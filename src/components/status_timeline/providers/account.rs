use super::TimelineProvider;
use crate::{
    environment::{types::TimelineDirection, Environment},
    view_model::{StatusId, StatusViewModel},
};
use futures_util::Future;
use megalodon::entities::Status;
use std::pin::Pin;

/// A provider that loads new data but just by ungrouping the grouped data (by default) but groups new data
pub struct AccountTimelineProvider {
    environment: Environment,
}

impl AccountTimelineProvider {
    pub fn new(environment: Environment) -> Self {
        Self { environment }
    }
}

impl std::fmt::Debug for AccountTimelineProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccountTimelineProvider").finish()
    }
}

impl TimelineProvider for AccountTimelineProvider {
    type Id = StatusId;
    type Element = Status;
    type ViewModel = StatusViewModel;
    fn should_auto_reload(&self) -> bool {
        true
    }

    fn identifier(&self) -> &str {
        std::any::type_name::<Self>()
    }

    fn forced_direction(&self) -> Option<TimelineDirection> {
        Some(TimelineDirection::NewestTop)
    }

    fn reset(&self) {
        // self.page.set(PER_PAGE)
    }

    fn scroll_to_item(&self, _updates: &[Status]) -> Option<StatusId> {
        None
    }

    fn request_data(
        &self,
        after: Option<StatusId>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Status>, String>> + Send>> {
        let model = self.environment.model.clone();
        let after = after.map(|e| e.0);
        Box::pin(async move { model.timeline(after, 1).await })
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
            .with_mutation(|mut storage| storage.merge_classictimeline(updates, is_reload));
        can_load_more
    }

    fn data(&self, _direction: TimelineDirection) -> Vec<StatusViewModel> {
        log::trace!("called classic timeline data");
        self.environment
            .storage
            .with(|storage| storage.classic_timeline.clone())
    }
}
