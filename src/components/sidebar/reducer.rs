use std::collections::HashSet;
use std::time::Duration;

use im::{HashMap, Vector};
use itertools::Itertools;

use crate::components::loggedin::Action;
use crate::environment::model::{Account, Notification, Status};
use crate::environment::storage::UiTab;
use crate::environment::types::{AppEvent, MainMenuEvent};
use crate::environment::Environment;
use crate::view_model::{AccountUpdateViewModel, AccountViewModel};
use navicula::{Debouncer, Effect};

pub type ViewStore<'a> = navicula::ViewStore<'a, super::SidebarReducer>;

#[derive(Clone)]
pub enum SidebarDelegateAction {
    SelectAccount(AccountViewModel),
    SelectedNotifications(AccountViewModel),
    Root(Action),
    AppEvent(AppEvent),
    SelectMore(MoreSelection),
}

#[derive(Clone)]
pub enum SidebarAction {
    Initial,
    ChangeTab(UiTab),
    SelectAccount(AccountViewModel),

    LoadTimeline,
    LoadTimelineData(Result<Vec<Status>, String>),
    LoadMoreTimeline,

    LoadNotifications,
    LoadedNotifications(Result<Vec<Notification>, String>),
    SelectedNotifications(AccountViewModel),

    AppEvent(AppEvent),
    Reload(bool),
    DataChanged,
    FavoritesChanged,

    Search(String),
    SearchResults(Option<Result<Vec<Account>, String>>),

    LoadLists,
    LoadedLists(Result<Vec<(String, String)>, String>),
    SelectList(String),
    LoadList(String),
    LoadListData(Result<Vec<Status>, String>, String),

    // This is a bit hackish. Move needed types into delegate
    Root(Action),

    // Switch within the more section
    MoreSelection(MoreSelection),
}

impl std::fmt::Debug for SidebarAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Initial => write!(f, "Initial"),
            Self::ChangeTab(arg0) => f.debug_tuple("ChangeTab").field(arg0).finish(),
            Self::SelectAccount(arg0) => f.debug_tuple("SelectAccount").field(arg0).finish(),
            Self::LoadTimeline => write!(f, "LoadTimeline"),
            Self::LoadTimelineData(_arg0) => f.debug_tuple("LoadTimelineData").finish(),
            Self::LoadMoreTimeline => write!(f, "LoadMoreTimeline"),
            Self::LoadNotifications => write!(f, "LoadNotifications"),
            Self::LoadedNotifications(_arg0) => f.debug_tuple("LoadedNotifications").finish(),
            Self::SelectedNotifications(arg0) => {
                f.debug_tuple("SelectedNotifications").field(arg0).finish()
            }
            Self::AppEvent(arg0) => f.debug_tuple("AppEvent").field(arg0).finish(),
            Self::Reload(arg0) => f.debug_tuple("Reload").field(arg0).finish(),
            Self::DataChanged => write!(f, "DataChanged"),
            Self::Root(arg0) => f.debug_tuple("Root").field(arg0).finish(),
            Self::Search(arg0) => f.debug_tuple("Search").field(arg0).finish(),
            Self::SearchResults(a) => write!(f, "SearchResults: {}", a.is_some()),
            Self::FavoritesChanged => write!(f, "FavoritesChanged"),
            Self::LoadLists => write!(f, "LoadLists"),
            Self::LoadedLists(_) => write!(f, "LoadedLists"),
            Self::SelectList(entry) => f.debug_tuple("SelectList").field(entry).finish(),
            Self::LoadList(id) => f.debug_tuple("LoadList").field(id).finish(),
            Self::LoadListData(_entry, id) => f.debug_tuple("LoadListData").field(id).finish(),
            Self::MoreSelection(id) => f.debug_tuple("MoreSelection").field(id).finish(),
        }
    }
}

#[allow(unused)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
pub enum MoreSelection {
    #[default]
    Classic,
    Yours,
    Local,
    Federated,
    Posts,
    Hashtags,
    Followers,
    Following,
    Bookmarks,
    Favorites,
}

#[derive(Clone, Default)]
pub struct SidebarState {
    pub list_names: Vec<(String, String)>,
    pub accounts: im::Vector<AccountUpdateViewModel>,
    pub active_tab: UiTab,
    pub selected_account: Option<AccountViewModel>,
    pub selected_notifications: Option<AccountViewModel>,
    pub notification_accounts: Vector<AccountUpdateViewModel>,
    pub user_account: Option<Account>,
    pub notification_posts_empty: bool,
    pub posts_empty: bool,
    /// For each timeline we host (e.g. main & lists) the last loaded id
    pub last_timeline_id: HashMap<String, String>,
    pub has_new_notifications: bool,

    pub loading_content: bool,
    pub loading_notifications: bool,
    pub last_notification_id: Option<String>,

    last_search_debounce: Option<Debouncer>,
    pub search_results: Vec<Account>,
    pub is_searching: bool,
    pub search_term: String,
    pub favorites: HashSet<String>,
    // The currently selected list (or none for timeline)
    pub selected_list: Option<String>,
    // for each timeline we support, note whether a "load more" (e.g. older)
    // data returned an empty result. in that case hide the button
    pub no_more_load_more: HashSet<String>,
    // The current sidebare selection if we're in the More section
    pub more_selection: MoreSelection,
}

impl SidebarState {
    pub fn has_more(&self) -> bool {
        let id = self
            .selected_list
            .as_ref()
            .map(String::clone)
            .unwrap_or_default();
        !self.no_more_load_more.contains(&id)
    }
}

pub fn reduce<'a>(
    context: &'a impl navicula::types::MessageContext<SidebarAction, SidebarDelegateAction, ()>,
    action: SidebarAction,
    state: &'a mut SidebarState,
    environment: &'a Environment,
) -> Effect<'static, SidebarAction> {
    log::trace!("{action:?}");
    let model = environment.model.clone();
    let window = context.window();

    match action {
        SidebarAction::Initial => {
            // Initial loading of the favorites.
            if let Some(n) = environment.repository.favorites() {
                state.favorites = n;
            }
            Effect::merge6(
                environment
                    .storage
                    .subscribe("sidebar_reducer_data", context, |_| {
                        SidebarAction::DataChanged
                    }),
                environment
                    .repository
                    .favorites
                    .subscribe("sidebar_favorites", context, |_| {
                        SidebarAction::FavoritesChanged
                    }),
                Effect::action(SidebarAction::LoadTimeline),
                Effect::action(SidebarAction::LoadNotifications),
                Effect::action(SidebarAction::LoadLists),
                Effect::timer(
                    Duration::from_secs(85),
                    SidebarAction::Reload(false),
                    202001,
                ),
            )
        }
        SidebarAction::DataChanged => {
            // Fetch all the data we need
            environment.storage.with(|d| {
                state.list_names = d
                    .timelines
                    .iter()
                    .filter(|e| !e.0.is_empty())
                    .map(|(key, value)| (key.clone(), value.title.clone()))
                    .collect();
                state.accounts = match state
                    .selected_list
                    .as_ref()
                    .and_then(|e| d.timelines.get(e.as_str()))
                {
                    Some(n) => &n.entries,
                    None => d.accounts(),
                }
                .clone();
                state.active_tab = d.active_tab;
                state.selected_account = d.selected_account.clone();
                state.posts_empty = d.posts().is_empty();
                state.selected_notifications = d.selected_notifications.clone();
                state.notification_accounts = d.notification_accounts.clone();
                state.notification_posts_empty = d.notification_posts.is_empty();
                state.user_account = d.user_account.clone();
            });
            // check if we have search results, in that case update apprioripately,
            // because a reload might have loaded an account that was prior to that not
            // there and would now appear twice
            if !state.search_results.is_empty() {
                update_search_results(state, state.search_results.clone());
            }
            log::trace!("data changed: {}", state.accounts.len());
            Effect::NONE
        }
        SidebarAction::FavoritesChanged => {
            if let Some(n) = environment.repository.favorites() {
                state.favorites = n;
            }
            Effect::NONE
        }
        SidebarAction::LoadTimeline => {
            state.loading_content = true;
            // FIXME: once we're done loading, find the not-known favorites and load
            // data for them as well
            return Effect::future(
                async move { model.timeline(None, 2).await },
                SidebarAction::LoadTimelineData,
            );
        }
        SidebarAction::LoadMoreTimeline => {
            let timeline_id = state
                .selected_list
                .as_ref()
                .map(|e| e.to_string())
                .unwrap_or_default();
            let cloned_id = timeline_id.clone();
            let Some(last) = state.last_timeline_id.get(&timeline_id).cloned() else {
                return Effect::NONE
            };
            state.loading_content = true;
            if timeline_id.is_empty() {
                return Effect::future(
                    async move { model.timeline(Some(last), 3).await },
                    SidebarAction::LoadTimelineData,
                );
            } else {
                return Effect::future(
                    async move {
                        model
                            .list_timeline(timeline_id.clone(), Some(last), 3)
                            .await
                    },
                    move |r| SidebarAction::LoadListData(r, cloned_id),
                );
            }
        }
        SidebarAction::LoadTimelineData(data) => {
            let direction = environment
                .repository
                .config()
                .ok()
                .map(|e| e.direction)
                .unwrap_or_default();
            environment.storage.with_mutation(|mut storage| {
                let is_initial = storage.accounts().is_empty();
                state.loading_content = false;
                let Ok(batch) = data else {
                    return Effect::NONE
                };
                let Some(last) = batch.last().map(|e| e.id.clone()) else {
                    // empty, we can't load more
                    state.no_more_load_more.insert(String::new());
                    return Effect::NONE
                };
                state.last_timeline_id.insert(String::new(), last.clone());
                storage.update_account_historical_data(&batch, &direction);

                // if this was the initial load, and nothing was selected, select the first entry
                let first_account = storage.accounts().get(0).cloned();
                match (is_initial, first_account) {
                    (true, Some(_account)) => {
                        Effect::merge2(
                            Effect::NONE,
                            // And load another two pages
                            Effect::future(
                                async move { model.timeline(Some(last), 2).await },
                                SidebarAction::LoadTimelineData,
                            ),
                        )
                    }
                    _ => Effect::NONE,
                }
            })
        }
        SidebarAction::LoadNotifications => {
            let id = state.last_notification_id.clone();
            state.loading_notifications = true;
            Effect::future(
                async move { model.notifications(id, 2).await },
                SidebarAction::LoadedNotifications,
            )
        }
        SidebarAction::LoadedNotifications(result) => {
            environment.storage.with_mutation(|mut storage| {
                state.loading_notifications = false;
                let Ok(n) = result else {
                    return Effect::NONE
                };
                if n.is_empty() {
                    return Effect::NONE;
                }
                let old = storage.notification_posts.clone();
                storage.update_notifications(&n);

                // FIXME: Notifications really need to be in their own reducer..
                // also fixme: this is the same code as in the ChangeTab reducer
                if let Ok(Some(last)) = environment
                    .repository
                    .map_config(|c| c.last_notification_id.clone())
                {
                    let Some(n) = storage.notification_accounts.head() else {
                        return Effect::NONE
                    };
                    let Some(p) = storage.notification_posts.get(&n.id)
                    .and_then(|p| p.last()) else {
                        return Effect::NONE
                    };
                    let status_id = p.status.id.clone();
                    if last != status_id {
                        state.has_new_notifications = true;
                    } else {
                        state.has_new_notifications = false;
                    }
                } else if old != storage.notification_posts {
                    state.has_new_notifications = true;
                }

                let (avt, active_tab) = (storage.user_account.clone(), storage.active_tab);

                // Only, if we're still logged in
                if let Some(avatar) = avt.map(|e| e.avatar_static.clone()) {
                    environment.platform.update_toolbar(
                        &avatar,
                        window,
                        &active_tab,
                        state.has_new_notifications,
                    );
                }

                // This is needed to only load newer notifications
                state.last_notification_id = Some(n.last().as_ref().map(|e| e.id.clone()).unwrap());
                log::error!("Should have updated notifications");
                Effect::NONE
            })
        }
        SidebarAction::SelectedNotifications(account) => {
            context.send_parent(SidebarDelegateAction::SelectedNotifications(account));
            Effect::NONE
        }
        SidebarAction::ChangeTab(tab) => {
            let avatar = environment.storage.with_mutation(|mut storage| {
                storage.active_tab = tab;
                storage.user_account.clone()
            });

            // Only, if we're still logged in

            if let Some(avatar) = avatar.map(|e| e.avatar_static.clone()) {
                environment.platform.update_toolbar(
                    &avatar,
                    window,
                    &tab,
                    state.has_new_notifications,
                );
            }

            if tab == UiTab::Mentions {
                state.has_new_notifications = false;

                // remember the uppermost status id, so that for the next start,
                // we can see if the user has indeed new notifications
                environment.storage.with(|data| {
                    let Some(n) = data.notification_accounts.head() else {
                        return;
                    };
                    let Some(p) = data.notification_posts.get(&n.id)
                    .and_then(|p| p.last()) else {
                        return;
                    };
                    let status_id = p.status.id.clone();
                    if let Err(e) = environment
                        .repository
                        .map_config(|c| c.last_notification_id = Some(status_id))
                    {
                        log::error!("Could not update notification {e:?}");
                    }
                })
            }
            Effect::NONE
        }
        SidebarAction::SelectAccount(account) => {
            environment.storage.with_mutation(|mut storage| {
                storage.selected_account = Some(account.clone());
                context.send_parent(SidebarDelegateAction::SelectAccount(account));
                Effect::NONE
            })
        }
        SidebarAction::Root(action) => {
            context.send_parent(SidebarDelegateAction::Root(action));
            Effect::NONE
        }
        SidebarAction::AppEvent(AppEvent::MenuEvent(m)) => {
            let tab = match m {
                MainMenuEvent::Timeline => UiTab::Timeline,
                MainMenuEvent::Mentions => UiTab::Mentions,
                MainMenuEvent::Messages => UiTab::Messages,
                MainMenuEvent::More => UiTab::More,
                _ => {
                    context.send_parent(SidebarDelegateAction::AppEvent(AppEvent::MenuEvent(m)));
                    return Effect::NONE;
                }
            };
            Effect::action(SidebarAction::ChangeTab(tab))
        }
        SidebarAction::AppEvent(event) => {
            log::error!("Missing Event: {event:?}");
            // context.send(Action::AppEvent(event));
            Effect::NONE
        }
        SidebarAction::Reload(hard) => {
            if hard {
                environment.storage.with_mutation(|mut s| s.clear_reload());
                state.last_timeline_id = HashMap::new();
                state.last_notification_id = None;
                state.no_more_load_more = HashSet::new();
            }
            Effect::action(SidebarAction::LoadTimeline)
        }
        SidebarAction::Search(term) => {
            state.is_searching = true;
            if let Some(e) = state.last_search_debounce.as_ref() {
                e.cancel()
            }
            let new_debounce = Debouncer::default();
            let model = environment.model.clone();
            state.last_search_debounce = Some(new_debounce.clone());
            state.search_term = term.clone();
            return Effect::debounce(
                async move { model.search_account(term, false).await },
                SidebarAction::SearchResults,
                Duration::from_secs_f64(0.12),
                new_debounce,
            );
        }
        SidebarAction::SearchResults(results) => {
            state.is_searching = false;
            match results {
                Some(Ok(data)) => {
                    update_search_results(state, data);
                }
                Some(Err(_)) => {
                    // FIXME: Error?
                }
                None => {
                    // debounced
                }
            }
            Effect::NONE
        }
        SidebarAction::LoadLists => Effect::future(
            async move {
                Ok(match model.lists().await {
                    Ok(n) => n
                        .into_iter()
                        .sorted_by(|a, b| b.title.cmp(&a.title))
                        .map(|e| (e.title, e.id))
                        .collect(),
                    Err(_) => vec![],
                })
            },
            SidebarAction::LoadedLists,
        ),
        SidebarAction::LoadedLists(result) => {
            let Ok(n) = result else {
                return Effect::NONE
            };
            environment.storage.with_mutation(|mut storage| {
                storage.update_timelines(&n);
            });
            Effect::NONE
        }
        SidebarAction::SelectList(entry) => {
            // reset the search
            state.search_results = Vec::new();
            state.search_term = String::new();
            if entry.is_empty() {
                state.selected_list = None;
                Effect::NONE
            } else {
                let Some(list) = environment.storage.with(|s| s.timelines.get(&entry).cloned()) else {
                    state.selected_list = None;
                    return Effect::NONE
                };
                state.selected_list = Some(entry.clone());
                if list.entries.is_empty() {
                    // if we don't have any data for this list, load data
                    Effect::action(SidebarAction::LoadList(entry))
                } else {
                    // otherwise, do nothing
                    Effect::NONE
                }
            }
        }
        SidebarAction::LoadList(id) => {
            let model = environment.model.clone();
            let cloned_id = id.clone();
            state.loading_content = true;
            Effect::future(
                async move { model.list_timeline(id, None, 2).await },
                move |r| SidebarAction::LoadListData(r, cloned_id),
            )
        }
        SidebarAction::LoadListData(result, id) => {
            state.loading_content = false;
            let Ok(data) = result else {
                return Effect::NONE
            };
            let Some(last) = data.last().map(|e| e.id.clone()) else {
                // empty, we can't load more
                state.no_more_load_more.insert(id);
                return Effect::NONE
            };
            state.last_timeline_id.insert(id.clone(), last);
            let direction = environment
                .repository
                .config()
                .ok()
                .map(|e| e.direction)
                .unwrap_or_default();
            environment.storage.with_mutation(|mut storage| {
                storage.update_timeline_historical_data(&id, &data, &direction);
            });
            Effect::NONE
        }
        SidebarAction::MoreSelection(s) => {
            state.more_selection = s;
            context.send_parent(SidebarDelegateAction::SelectMore(s));
            Effect::NONE
        }
    }
}

fn update_search_results(state: &mut SidebarState, results: Vec<Account>) {
    // only assign those results that aren't in our accounts
    // already

    // FIXME: This is trickier because the Mastodon search is fuzzy, so there
    // might be results that are not in our list, (because the search didn't hit for them)
    // but that *are* in the results. and so taping them makes them disappear
    // let acts: HashSet<&str> = state
    //     .data
    //     .accounts()
    //     .iter()
    //     .map(|e| e.id.0.as_str())
    //     .collect();
    // let d = results
    //     .into_iter()
    //     .filter(|e| !acts.contains(e.id.as_str()))
    //     .collect();
    // state.search_results = d;
    state.search_results = results;
}
