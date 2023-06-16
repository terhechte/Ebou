use super::TimelineProvider;
use crate::{
    environment::{types::TimelineDirection, Environment, Model},
    view_model::{AccountId, StatusId, StatusViewModel},
};
use chrono::Utc;
use futures_util::Future;
use megalodon::entities::Status;
use std::{cell::Cell, pin::Pin};

/// A provider that loads new data but also feeds the result in a grouped fashion into the Storage
pub struct GroupedAccountTimelineProvider {
    account: AccountId,
    model: Model,
    environment: Environment,
    last_action_was_historic_data: Cell<bool>,
}

impl GroupedAccountTimelineProvider {
    pub fn new(account: AccountId, model: Model, environment: Environment) -> Self {
        Self {
            account,
            model,
            environment,
            last_action_was_historic_data: Cell::new(false),
        }
    }
}

impl std::fmt::Debug for GroupedAccountTimelineProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GroupedAccountTimelineProvider")
            .field("account", &self.account)
            .field(
                "last_action_was_historic_data",
                &self.last_action_was_historic_data,
            )
            .finish()
    }
}

impl TimelineProvider for GroupedAccountTimelineProvider {
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
        None
    }

    fn reset(&self) {}

    fn scroll_to_item(&self, updates: &[Status]) -> Option<StatusId> {
        let existing = self
            .environment
            .storage
            .with(|m| m.posts().get(&self.account).cloned())?;
        newer_than(&existing, updates)
    }

    fn request_data(
        &self,
        after: Option<StatusId>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Status>, String>> + Send>> {
        let model = self.model.clone();
        let account = self.account.clone();

        // The code is much simpler if `after` is available
        if let Some(after) = after {
            self.last_action_was_historic_data.set(true);
            return Box::pin(async move {
                model
                    .user_timeline(account.0.clone(), Some(after.0.clone()), None, Some(40))
                    .await
            });
        }
        self.last_action_was_historic_data.set(false);

        let act = self
            .environment
            .storage
            .with(|m| m.posts().get(&self.account).cloned());

        // check if we have a marker for this account
        let (after_id, since_id, post_id) = if let Some((marker, last_used)) =
            self.environment.repository.get_timeline_marker(&account.0)
        {
            // If we have posts we selected an account with updates in the sidebar
            if let Some(post) = act.and_then(|e| e.last().cloned()) {
                log::trace!("Found marker {marker:?}");
                // if the date is not too old, we use the marker and page until we reach the
                // related status. Otherwise, we just backfill for 1 page
                let distance = Utc::now().signed_duration_since(last_used);
                if distance.num_days() > 7 {
                    log::trace!("marker older 7 days {}", distance.num_days());
                    // Backfill for one page of data
                    (Some(post.id.0.clone()), None, post.id.0)
                } else {
                    log::trace!("valid marker");
                    // start from last known
                    (None, Some(marker), post.id.0)
                }
            } else {
                // if not, we selected a random account (e.g. via search)
                return Box::pin(async move {
                    model
                        .user_timeline(account.0.clone(), None, None, Some(25))
                        .await
                });
            }
        } else {
            // we have at least one post from this user (otherwise they wouldn't appear)
            // in the list.
            log::trace!("found novalid marker");
            if let Some(post) = act.and_then(|e| e.last().cloned()) {
                (Some(post.id.0.clone()), None, post.id.0)
            } else {
                return Box::pin(async move {
                    model
                        .user_timeline(account.0.clone(), None, None, Some(25))
                        .await
                });
            }
        };

        Box::pin(async move {
            // if we have a since id, continue paging until we reach the post id - or
            // until we paged for 3 pages
            if let Some(ref since) = since_id {
                let mut lp = 0;
                let mut stati = Vec::new();
                let mut cloned_since = since.clone();
                loop {
                    lp += 4;
                    log::trace!("paging {lp}");
                    if lp >= 3 {
                        log::trace!("end paging");
                        return Ok(stati);
                    }
                    match model
                        .user_timeline(
                            account.0.clone(),
                            None,
                            Some(cloned_since.clone()),
                            Some(40),
                        )
                        .await
                    {
                        Ok(mut next) => {
                            log::trace!("found more");
                            if next.is_empty() {
                                log::trace!("more empty");
                                return Ok(stati);
                            }
                            stati.append(&mut next);
                            let Some(last) = stati.last() else { return Ok(stati) };
                            let Some(cmp) = cmp_id_str_gt(&last.id, &post_id) else { return Ok(stati) };
                            if !cmp {
                                log::trace!("smp not done yet: {} >= {}", &last.id, &post_id);
                                // not done yet
                                cloned_since = last.id.clone();
                            } else {
                                log::trace!("smp done");
                                return Ok(stati);
                            }
                        }
                        Err(e) => return Err(e),
                    }
                }
            } else {
                log::trace!("nothing special");
                model
                    .user_timeline(account.0.clone(), after_id, None, Some(15))
                    .await
            }
        })
    }

    fn process_new_data(
        &self,
        updates: &[Status],
        direction: TimelineDirection,
        _is_reload: bool,
    ) -> bool {
        // For this provider, only if we did a historic data event,
        // do we check with updates == empty for load_more.
        let can_load_more = if self.last_action_was_historic_data.get() {
            !updates.is_empty()
        } else {
            true
        };
        self.environment.storage.with_mutation(|mut storage| {
            storage.update_account_historical_data(updates, &direction);
        });
        can_load_more
    }

    fn data(&self, _direction: TimelineDirection) -> Vec<StatusViewModel> {
        self.environment.storage.with(|storage| {
            storage
                .posts()
                .get(&self.account)
                .cloned()
                .unwrap_or_default()
        })
    }
}

/// check if a is >= b (converted to nr)
fn cmp_id_str_gt(a: &str, b: &str) -> Option<bool> {
    let a = a.parse::<u128>().ok()?;
    let b = b.parse::<u128>().ok()?;
    Some(a >= b)
}

fn newer_than<'a>(
    existing: &'a [StatusViewModel],
    incoming: &'a [Status],
    // newest_in: DateTime<Utc>,
) -> Option<StatusId> {
    // - get the date of the newest post in our old data
    // - update the data / recalculate
    // - find the first one newer than the newest in our old data
    // - or the last (newest) in the incoming data
    if let Some(current_newest) = existing.last() {
        if let Some(incoming_new) = incoming
            .iter()
            .find(|e| e.created_at > current_newest.created)
        {
            Some(StatusId(incoming_new.id.clone()))
        } else {
            Some(current_newest.id.clone())
        }
    } else {
        incoming.last().map(|e| StatusId(e.id.clone()))
    }
}
