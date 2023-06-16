use std::collections::HashSet;

use chrono::{DateTime, Utc};
use im::HashMap;
use megalodon::streaming::Message;

use crate::components::conversation::Conversation;
use crate::environment::model::{Account, Notification, Status};
use crate::view_model::*;

use super::types::TimelineDirection;

const LOCAL_TIMELINE_KEY: &str = "";

#[derive(Clone, Copy, Eq, PartialEq, Debug, Default)]
pub enum UiTab {
    #[default]
    Timeline,
    Mentions,
    Messages,
    More,
}

impl UiTab {
    pub fn is_timeline(&self) -> bool {
        matches!(self, UiTab::Timeline)
    }

    pub fn is_mentions(&self) -> bool {
        matches!(self, UiTab::Mentions)
    }

    pub fn is_messages(&self) -> bool {
        matches!(self, UiTab::Messages)
    }

    pub fn is_more(&self) -> bool {
        matches!(self, UiTab::More)
    }
}

#[derive(Clone)]
pub struct Data {
    pub user_account: Option<Account>,
    pub selected_account: Option<AccountViewModel>,

    pub conversations: im::HashMap<StatusId, Conversation>,
    pub notification_accounts: im::Vector<AccountUpdateViewModel>,
    pub notification_posts: im::HashMap<AccountId, Vec<NotificationViewModel>>,

    pub selected_notifications: Option<AccountViewModel>,

    // The content of different lists.
    pub timelines: HashMap<String, TimelineEntry>,

    pub active_tab: UiTab,

    /// Bookmarks can quickly become out of date. We store it here, so that
    /// at least any UI bookmark action can be applied to our internal state
    pub bookmarks: Vec<StatusViewModel>,
    pub favorites: Vec<StatusViewModel>,

    pub local_timeline: Vec<StatusViewModel>,
    pub public_timeline: Vec<StatusViewModel>,
    pub classic_timeline: Vec<StatusViewModel>,
    pub account_timeline: im::HashMap<AccountId, Vec<StatusViewModel>>,

    // did we load the maximum history for something?
    pub accounts_no_older_data: im::HashSet<AccountId>,
}

impl std::fmt::Debug for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Data")
            .field("user_account", &self.user_account.as_ref().map(|e| &e.id))
            .field(
                "selected_account",
                &self.selected_account.as_ref().map(|e| &e.id),
            )
            .field("conversations", &self.conversations.len())
            .field("notification_accounts", &self.notification_accounts.len())
            .field("notification_posts", &self.notification_posts.len())
            .field(
                "selected_notifications",
                &self.selected_notifications.as_ref().map(|e| &e.id),
            )
            .field("timelines", &self.timelines)
            .field("active_tab", &self.active_tab)
            .field("bookmarks", &self.bookmarks.len())
            .field("accounts_no_older_data", &self.accounts_no_older_data)
            .finish()
    }
}

impl Default for Data {
    fn default() -> Self {
        let mut timelines = HashMap::new();
        timelines.insert(
            LOCAL_TIMELINE_KEY.to_string(),
            TimelineEntry {
                title: crate::loc!("Timeline").to_string(),
                ..Default::default()
            },
        );
        Self {
            user_account: Default::default(),
            selected_account: Default::default(),
            conversations: Default::default(),
            notification_accounts: Default::default(),
            notification_posts: Default::default(),
            selected_notifications: Default::default(),
            timelines,
            active_tab: Default::default(),
            bookmarks: Default::default(),
            favorites: Default::default(),
            local_timeline: Default::default(),
            public_timeline: Default::default(),
            classic_timeline: Default::default(),
            account_timeline: Default::default(),
            accounts_no_older_data: Default::default(),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Default)]
pub struct TimelineEntry {
    pub title: String,
    pub id: String,
    pub entries: im::Vector<AccountUpdateViewModel>,
    pub posts: HashMap<AccountId, Vec<StatusViewModel>>,
    /// Only if we're selected, will the timer force an update
    pub last_update: DateTime<Utc>,
}

impl std::fmt::Debug for TimelineEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TimelineEntry")
            .field("title", &self.title)
            .field("id", &self.id)
            .field("entries", &self.entries.len())
            .field("posts", &self.posts.len())
            .field("last_update", &self.last_update)
            .finish()
    }
}

// Data Operations

impl Data {
    pub fn accounts(&self) -> &im::Vector<AccountUpdateViewModel> {
        &self.timelines.get(LOCAL_TIMELINE_KEY).unwrap().entries
    }

    pub fn posts(&self) -> &im::HashMap<AccountId, Vec<StatusViewModel>> {
        &self.timelines.get(LOCAL_TIMELINE_KEY).unwrap().posts
    }

    pub fn merge_bookmarks(&mut self, bookmarks: &[Status], is_reload: bool) {
        Self::general_merge(&mut self.bookmarks, bookmarks, is_reload, None);
    }

    pub fn merge_favorites(&mut self, favorites: &[Status], is_reload: bool) {
        Self::general_merge(&mut self.favorites, favorites, is_reload, None);
    }

    pub fn merge_localtimeline(&mut self, posts: &[Status], is_reload: bool) {
        Self::general_merge(&mut self.local_timeline, posts, is_reload, Some(350));
    }

    pub fn merge_publictimeline(&mut self, posts: &[Status], is_reload: bool) {
        Self::general_merge(&mut self.public_timeline, posts, is_reload, Some(350));
    }

    pub fn merge_classictimeline(&mut self, posts: &[Status], is_reload: bool) {
        Self::general_merge(&mut self.classic_timeline, posts, is_reload, Some(350));
    }

    pub fn merge_account(&mut self, posts: &[Status], id: &AccountId, is_reload: bool) {
        // if we don't have an entry yet, insert a new one
        let g = self.account_timeline.entry(id.clone()).or_default();
        Self::general_merge(g, posts, is_reload, None);
    }

    fn general_merge(
        into: &mut Vec<StatusViewModel>,
        items: &[Status],
        is_reload: bool,
        limit_count: Option<usize>,
    ) {
        let existing: HashSet<_> = into.iter().map(|e| e.id.0.clone()).collect();
        for entry in items.iter() {
            if !existing.contains(entry.id.as_str()) {
                if is_reload {
                    into.insert(0, StatusViewModel::new(entry))
                } else {
                    into.push(StatusViewModel::new(entry));
                }
            }
        }
        // by default, only keep the newest LIMIT. Otherwise memory piles up
        if let Some(limit) = limit_count {
            if into.len() > limit {
                let _ = into.drain(limit..).collect::<Vec<_>>();
            }
        }
    }

    /// Will return the currently open conversation (for the current tab)
    /// if there is one
    pub fn conversation(&self, id: &StatusId) -> Option<&Conversation> {
        // let current = self.conversation_id()?;
        self.conversations.get(id)
    }

    pub fn clear_reload(&mut self) -> bool {
        self.timelines.iter_mut().for_each(|l| {
            l.1.entries.clear();
            l.1.posts.clear();
        });
        self.selected_account = None;
        self.selected_notifications = None;
        self.bookmarks.clear();
        self.favorites.clear();
        self.local_timeline.clear();
        self.public_timeline.clear();
        self.account_timeline.clear();
        self.classic_timeline.clear();
        true
    }

    pub fn handle_push_message(&mut self, message: Message, direction: TimelineDirection) {
        match message {
            Message::Update(status) => {
                log::debug!("update classic timeline data");
                self.classic_timeline
                    .insert(0, StatusViewModel::new(&status));
                self.update_account_historical_data(&[status], &direction);
            }
            Message::Notification(notification) => {
                self.update_notifications(&[notification]);
            }
            Message::Conversation(_) => {
                //
            }
            Message::Delete(_) => {
                //
            }
            Message::StatusUpdate(status) => {
                self.update_account_historical_data(&[status], &direction);
            }
            Message::Heartbeat() => {
                //
            }
        }
    }

    pub fn mutate_post(
        &mut self,
        id: StatusId,
        account_id: AccountId,
        mut action: impl FnMut(&mut StatusViewModel),
    ) -> bool {
        let mut found = false;

        if let Some(posts) = self.notification_posts.get_mut(&account_id) {
            for item in posts.iter_mut() {
                if item.status.id == id {
                    action(&mut item.status);
                    found = true;
                }
            }
        }

        // Go through all the timelines
        if let Some(posts) = self.account_timeline.get_mut(&account_id) {
            for item in posts.iter_mut() {
                if item.id == id {
                    action(item);
                    found = true;
                }
            }
        }

        // next, go through all posts & boosts
        for (_, timeline) in self.timelines.iter_mut() {
            for (_, posts) in timeline.posts.iter_mut() {
                for p in posts.iter_mut() {
                    if p.id == id {
                        action(p);
                        found = true;
                    }
                    if let Some(o) = p.reblog_status.as_mut() {
                        if o.id == id {
                            action(o);
                            found = true;
                        }
                    }
                }
            }
        }

        // bookmarks, favorites, and so on
        for posts in [
            self.bookmarks.iter_mut(),
            self.favorites.iter_mut(),
            self.local_timeline.iter_mut(),
            self.public_timeline.iter_mut(),
            self.classic_timeline.iter_mut(),
        ] {
            for p in posts {
                if p.id == id {
                    action(p);
                    found = true;
                }
                if let Some(o) = p.reblog_status.as_mut() {
                    if o.id == id {
                        action(o);
                        found = true;
                    }
                }
            }
        }

        // finally, go through the conversations
        let mut found_conv = false;
        for (_, c) in self.conversations.iter_mut() {
            let r = c.mutate_post(&id, &mut action);
            if r {
                found_conv = r;
            }
        }
        found_conv || found
    }

    /// Remove lists we don't have anymore, add new lists
    pub fn update_timelines(&mut self, timelines: &[(String, String)]) {
        let mut unknown: HashSet<_> = self.timelines.keys().cloned().collect();
        for (name, id) in timelines {
            if self.timelines.contains_key(id) {
                unknown.remove(id);
                self.timelines[id].title = name.clone();
            } else {
                self.timelines.insert(
                    id.clone(),
                    TimelineEntry {
                        title: name.clone(),
                        id: id.clone(),
                        ..Default::default()
                    },
                );
            }
        }
        // remove all unknown
        for id in unknown {
            if id != LOCAL_TIMELINE_KEY {
                self.timelines.remove(&id);
            }
        }
    }

    pub fn replied_to_status(&mut self, status_id: &str) -> bool {
        for (_, timeline) in self.timelines.iter_mut() {
            for (_, stati) in timeline.posts.iter_mut() {
                for status in stati.iter_mut() {
                    if status.id.0 == status_id {
                        status.did_reply();
                        return true;
                    }
                }
            }
        }

        for (_, stati) in self.notification_posts.iter_mut() {
            for notification in stati.iter_mut() {
                if notification.status.id.0 == status_id {
                    notification.status.did_reply();
                    return true;
                }
            }
        }
        false
    }

    pub fn changed_bookmark(&mut self, status: &StatusViewModel, added: bool) {
        if added {
            self.bookmarks.insert(0, status.clone());
        } else {
            let Some(idx) = self.bookmarks.iter().position(|a| a.id == status.id) else {
                return
            };
            self.bookmarks.remove(idx);
        }
    }

    pub fn changed_favorite(&mut self, status: &StatusViewModel, added: bool) {
        if added {
            self.favorites.insert(0, status.clone());
        } else {
            let Some(idx) = self.favorites.iter().position(|a| a.id == status.id) else {
                return
            };
            self.favorites.remove(idx);
        }
    }

    pub fn changed_boost(&mut self, _status: &StatusViewModel, _added: bool) {
        // not sure we need to do anything here
    }

    pub fn possibly_update_conversation_with_reply(&mut self, reply: &Status) {
        // If this was a reply to an update in the currently loaded conversation,
        // then inject it there
        let Some(reply_id) = &reply.in_reply_to_id.as_ref().map(|e| StatusId(e.clone())) else { return };

        for (_, conversation) in self.conversations.iter_mut() {
            conversation
                .insert_child_if(reply_id, StatusViewModel::new(reply))
                .unwrap_or_default();
        }
    }

    pub fn update_account_historical_data(
        &mut self,
        updates: &[Status],
        direction: &TimelineDirection,
    ) {
        Self::update_historical_data(
            updates,
            self.timelines.get_mut(LOCAL_TIMELINE_KEY).unwrap(),
            direction,
        );
    }

    pub fn update_timeline_historical_data(
        &mut self,
        id: &str,
        updates: &[Status],
        direction: &TimelineDirection,
    ) {
        let Some(timeline) = self.timelines.get_mut(id) else {
            return
        };
        Self::update_historical_data(updates, timeline, direction);
    }

    /// can't have to &mut. Proper solution is to abstract timelines into a struct
    /// that can be used in here (like the list struct)
    fn update_historical_data(
        updates: &[Status],
        timeline: &mut TimelineEntry,
        direction: &TimelineDirection,
    ) -> bool {
        let posts = &mut timeline.posts;
        let accounts = &mut timeline.entries;
        let mut updated = HashSet::new();
        for update in updates.iter() {
            let id = AccountId(update.account.id.clone());
            updated.insert(id.clone());
            let exists = posts.contains_key(&id);
            let new_status = StatusViewModel::new(update);
            if exists {
                // this should never fail, but still
                if let Some(account_idx) = accounts.iter().position(|o| o.id == id) {
                    // update with whatever the new status is
                    if update.created_at > accounts[account_idx].last_updated {
                        accounts[account_idx] = AccountUpdateViewModel::new(&new_status);
                    }
                }
                posts.entry(id).and_modify(|existing| {
                    // if this id already exists, we replace it
                    if let Some(ref pos) = existing.iter().position(|x| x.id.0 == update.id) {
                        existing[*pos] = new_status; // StatusViewModel::new(update);
                    } else {
                        existing.push(new_status);
                    }
                });
            } else {
                accounts.push_back(AccountUpdateViewModel::new(&new_status));
                posts.entry(id).or_insert(vec![new_status]);
            }
        }
        // Sort the accounts by date
        accounts.sort_by(|a, b| b.last_updated.cmp(&a.last_updated));

        // for all affected accounts, sort reverse
        let changed = !updated.is_empty();
        for account in updated {
            if direction == &TimelineDirection::NewestBottom {
                if let Some(e) = posts.get_mut(&account) {
                    e.sort_by(|a, b| a.created.cmp(&b.created))
                }
            } else if let Some(e) = posts.get_mut(&account) {
                e.sort_by(|b, a| a.created.cmp(&b.created))
            }
        }

        changed
    }

    pub fn update_notifications(&mut self, notifications: &[Notification]) -> bool {
        let posts = &mut self.notification_posts;
        let accounts = &mut self.notification_accounts;

        let mut updated = HashSet::new();
        for notification in notifications.iter() {
            let id = AccountId(notification.account.id.clone());
            let exists = posts.contains_key(&id);
            let Some(ref status) = notification.status else {
                continue
            };
            let Some(nm) = NotificationViewModel::new(notification) else {
                continue
            };
            updated.insert(id.clone());
            let new_status = StatusViewModel::new(status);
            if exists {
                if let Some(account_idx) = accounts.iter().position(|o| o.id == id) {
                    // update with whatever the new status is
                    if status.created_at > accounts[account_idx].last_updated {
                        accounts[account_idx] = AccountUpdateViewModel::new(&new_status);
                    }
                }
                posts.entry(id).and_modify(|existing| {
                    // if this id already exists, we replace it
                    if let Some(ref pos) = existing.iter().position(|x| x.id == notification.id) {
                        existing[*pos] = nm;
                    } else {
                        existing.push(nm)
                    }
                });
            } else {
                accounts.push_back(AccountUpdateViewModel::new(&new_status));
                posts.entry(id).or_insert(vec![nm]);
            }
        }
        // Sort the accounts by date
        accounts.sort_by(|a, b| b.last_updated.cmp(&a.last_updated));

        !updated.is_empty()
    }
}
