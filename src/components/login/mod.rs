mod reducer;
mod view;

use crate::environment::Environment;
use navicula::reducer::Reducer;
use reducer::{reduce, LoginAction, LoginReducer, LoginState};

pub use view::LoginApp;

impl Reducer for LoginReducer {
    type Message = ();

    type DelegateMessage = LoginAction;

    type Action = LoginAction;

    type State = LoginState;

    type Environment = Environment;

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
        reduce(context, action, state, environment)
    }

    fn initial_action() -> Option<Self::Action> {
        Some(LoginAction::Load)
    }
}
