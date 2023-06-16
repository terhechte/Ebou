mod conversation_helpers;
mod reducer;
mod view;

use crate::PublicAction;
pub use conversation_helpers::Conversation;
pub use reducer::{Action, State, ViewStore};
pub use view::ConversationComponent;

pub struct ConversationReducer;
use navicula::reducer::Reducer;

impl Reducer for ConversationReducer {
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
