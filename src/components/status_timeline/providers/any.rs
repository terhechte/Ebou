use megalodon::entities::Status;

use crate::view_model::{StatusId, StatusViewModel};

use super::TimelineProvider;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::Arc,
};

// A container that wraps the TimelineProvider and provides Eq / PartialEq
#[derive(Clone)]
pub struct AnyTimelineProvider {
    provider:
        Arc<dyn TimelineProvider<Id = StatusId, Element = Status, ViewModel = StatusViewModel>>,
    equatable: u64,
}

impl AnyTimelineProvider {
    pub fn new<T: Hash>(
        provider: impl TimelineProvider<Id = StatusId, Element = Status, ViewModel = StatusViewModel>
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

impl PartialEq for AnyTimelineProvider {
    /// Thing never changes
    fn eq(&self, other: &Self) -> bool {
        self.equatable == other.equatable
    }
}

impl std::fmt::Debug for AnyTimelineProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnyTimelineProvider")
            .field("provider", &self.provider)
            .field("eq", &self.equatable)
            .finish()
    }
}

impl Eq for AnyTimelineProvider {}

impl std::ops::Deref for AnyTimelineProvider {
    type Target =
        Arc<dyn TimelineProvider<Id = StatusId, Element = Status, ViewModel = StatusViewModel>>;

    fn deref(&self) -> &Self::Target {
        &self.provider
    }
}
