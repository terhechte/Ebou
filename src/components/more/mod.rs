mod reducer;
mod view;

pub use reducer::{reduce, Action, PublicAction, State, ViewStore};
pub use view::MoreViewComponent;

pub struct MoreReducer;
use navicula::reducer::Reducer;

impl Reducer for MoreReducer {
    type Message = reducer::Message;

    type DelegateMessage = reducer::PublicAction;

    type Action = reducer::Action;

    type State = reducer::State;

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
        Some(Action::Initial)
    }
}
