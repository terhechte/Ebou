mod action;
mod reducer;
mod state;
mod view;

pub use action::PostAction;
pub use state::{PostKind, State, Visibility};

pub use reducer::{reduce, ViewStore};

pub use view::PostView;

pub struct PostReducer;
use navicula::reducer::Reducer;

impl Reducer for PostReducer {
    type Message = ();

    type DelegateMessage = action::PostAction;

    type Action = action::PostAction;

    type State = state::State;

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
        Some(PostAction::Open(Vec::new()))
    }
}
