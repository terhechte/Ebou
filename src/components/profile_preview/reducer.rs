use crate::environment::model::Relationship;

use crate::environment::Environment;
use crate::view_model::AccountViewModel;
use crate::PublicAction;
use navicula::Effect;

pub type ViewStore<'a> = navicula::ViewStore<'a, super::ProfilePreviewReducer>;

#[derive(Clone, Debug)]
pub enum ProfileAction {
    Initial,
    Public(PublicAction),
    LoadRelationship,
    LoadedRelationship(Result<Relationship, String>),
    ToggleFollow,
    ToggleFollowResult(Result<bool, String>),
    ToggleFavourite,
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ProfileState {
    pub account: AccountViewModel,
    // should we initially load the relationship?
    pub load_initial: bool,
    pub has_loaded: bool,
    pub is_loading: bool,
    pub is_favorite: bool,
    /// Are we following them
    pub following: bool,
    /// Are they following us
    pub followed_by: bool,
    /// Did we mute them?
    pub muting: bool,
    pub blocking: bool,
    pub error: Option<String>,
}

impl ProfileState {
    pub fn new(account: AccountViewModel, load_initial: bool) -> Self {
        Self {
            account,
            load_initial,
            ..Default::default()
        }
    }
}

pub fn reduce<'a>(
    context: &'a impl navicula::types::MessageContext<ProfileAction, crate::PublicAction, ()>,
    action: ProfileAction,
    state: &'a mut ProfileState,
    environment: &'a Environment,
) -> Effect<'static, ProfileAction> {
    log::trace!("{action:?}");
    match action {
        ProfileAction::Initial => {
            let id = state.account.id.0.clone();
            state.is_favorite = environment.repository.is_favorite(&id).unwrap_or_default();
            if state.load_initial {
                return Effect::action(ProfileAction::LoadRelationship);
            }
        }
        ProfileAction::LoadRelationship => {
            let id = state.account.id.0.clone();
            let model = environment.model.clone();
            state.is_loading = true;
            return Effect::future(
                async move { model.relationship(id).await },
                ProfileAction::LoadedRelationship,
            );
        }
        ProfileAction::LoadedRelationship(result) => {
            state.is_loading = false;
            match result {
                Ok(r) => {
                    state.following = r.following;
                    state.followed_by = r.followed_by;
                    state.blocking = r.blocking;
                    state.muting = r.muting;
                }
                Err(e) => {
                    state.error = Some(e);
                }
            }
        }
        ProfileAction::ToggleFollow => {
            // if we have a loading error, do nothing
            if state.error.is_some() {
                return Effect::NONE;
            };
            state.is_loading = true;
            let id = state.account.id.0.clone();
            let model = environment.model.clone();
            if state.following {
                return Effect::future(
                    async move { model.unfollow(id).await },
                    ProfileAction::ToggleFollowResult,
                );
            } else {
                return Effect::future(
                    async move { model.follow(id).await },
                    ProfileAction::ToggleFollowResult,
                );
            }
        }
        ProfileAction::ToggleFollowResult(result) => {
            state.is_loading = false;
            match result {
                // Follow was successful
                Ok(true) => {
                    state.following = true;
                }
                // Unfollow was successful
                Ok(false) => {
                    state.following = false;
                }
                Err(e) => {
                    log::error!("Could not follow/unfollow {e:?}");
                }
            }
        }
        ProfileAction::ToggleFavourite => {
            let _ = environment
                .repository
                .toggle_favorite(state.account.id.0.clone());
            state.is_favorite = !state.is_favorite;
        }
        ProfileAction::Public(action) => {
            context.send_parent(action);
        }
    }
    Effect::NONE
}
