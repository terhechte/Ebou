mod reducer;
mod view;

pub use reducer::reduce as profile_reducer;
pub use reducer::{ProfileAction, ProfileState, ViewStore};
pub use view::{
    FollowProfileComponent, ListProfileComponent, ProfileComponent, ProfilePageHeader,
    SelectedProfileTab,
};

pub struct ProfilePreviewReducer;
use navicula::reducer::Reducer;

impl Reducer for ProfilePreviewReducer {
    type Message = ();

    type DelegateMessage = crate::PublicAction;

    type Action = reducer::ProfileAction;

    type State = reducer::ProfileState;

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
        Some(ProfileAction::Initial)
    }
}
