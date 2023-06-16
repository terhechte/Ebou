use crate::components::component_stack::RootTimelineKind;
use crate::components::profiles::ProfilesKind;
use crate::components::sidebar::MoreSelection;
use crate::environment::model::Account;
use crate::environment::types::{AppEvent, UiConfig};
use crate::environment::Environment;
use crate::view_model::AccountViewModel;

use navicula::Effect;

pub type ViewStore<'a> = navicula::ViewStore<'a, super::MoreReducer>;

#[derive(Clone, Debug, Default)]
pub struct Providers {
    pub classic_timeline: Option<RootTimelineKind>,
    pub bookmarks: Option<RootTimelineKind>,
    pub favorites: Option<RootTimelineKind>,
    pub account: Option<RootTimelineKind>,
    pub local: Option<RootTimelineKind>,
    pub public: Option<RootTimelineKind>,
    pub follows: Option<RootTimelineKind>,
    pub following: Option<RootTimelineKind>,
}

#[derive(Clone, Debug)]
pub enum Message {
    AppEvent(AppEvent),
    Selection(MoreSelection, bool),
}

#[derive(Clone, Debug)]
pub enum PublicAction {
    Timeline(crate::PublicAction),
}

#[derive(Clone, Debug)]
pub enum Action {
    Initial,
    Selection(MoreSelection),
    AppEvent(AppEvent),
    Conversation(crate::PublicAction),
}

#[derive(Clone, Debug)]
pub struct State {
    pub selection: MoreSelection,
    pub providers: Providers,
    pub ui_settings: UiConfig,
    pub user_account: Account,
}

impl State {
    pub fn new(selection: MoreSelection, user_account: Account) -> Self {
        Self {
            selection,
            // selected_conversation,
            providers: Providers::default(),
            ui_settings: Default::default(),
            user_account,
        }
    }
}

pub fn reduce<'a>(
    context: &'a impl navicula::types::MessageContext<Action, PublicAction, Message>,
    action: Action,
    state: &'a mut State,
    environment: &'a Environment,
) -> Effect<'static, Action> {
    log::trace!("{action:?}");
    match action {
        Action::Initial => {
            state.ui_settings = environment.repository.config().unwrap_or_default();
            handle_selection(
                state.selection,
                &mut state.providers,
                AccountViewModel::new(&state.user_account),
            );
        }
        Action::Selection(a) => {
            let re_selected = state.selection == a;
            state.selection = a;
            handle_selection(
                state.selection,
                &mut state.providers,
                AccountViewModel::new(&state.user_account),
            );
            context.send_children(Message::Selection(a, re_selected))
        }
        Action::AppEvent(a) => context.send_children(Message::AppEvent(a)),
        Action::Conversation(a) => context.send_parent(PublicAction::Timeline(a)),
    }
    Effect::NONE
}

/// Create a new timeline provider or get an existing one
fn handle_selection(
    selection: MoreSelection,
    providers: &mut Providers,
    account: AccountViewModel,
) {
    match selection {
        MoreSelection::Classic => {
            providers.classic_timeline = Some(RootTimelineKind::Account(account))
        }
        MoreSelection::Yours => providers.account = Some(RootTimelineKind::UserProfile(account)),
        MoreSelection::Bookmarks => {
            providers.bookmarks = Some(RootTimelineKind::Bookmarks(account))
        }
        MoreSelection::Favorites => {
            providers.favorites = Some(RootTimelineKind::Favorites(account))
        }
        MoreSelection::Federated => providers.public = Some(RootTimelineKind::Federated(account)),
        MoreSelection::Local => providers.local = Some(RootTimelineKind::Local(account)),
        MoreSelection::Followers => {
            providers.follows = Some(RootTimelineKind::Relationship(
                account.clone(),
                ProfilesKind::Followers(account.id),
            ));
        }
        MoreSelection::Following => {
            providers.following = Some(RootTimelineKind::Relationship(
                account.clone(),
                ProfilesKind::Following(account.id),
            ));
        }
        MoreSelection::Posts => todo!(),
        MoreSelection::Hashtags => todo!(),
    }
}
