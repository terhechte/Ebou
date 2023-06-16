mod providers;
mod reducer;
mod view;

use crate::PublicAction;
pub use providers::*;
pub use reducer::reduce as timeline_reducer;
pub use reducer::{reduce, Action, State, ViewStore};
pub use view::{TimelineComponent, TimelineContents};

pub struct TimelineReducer;

use navicula::reducer::Reducer;

impl Reducer for TimelineReducer {
    type Message = ();

    type DelegateMessage = PublicAction;

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
