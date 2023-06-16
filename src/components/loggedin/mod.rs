pub mod reducer;
pub mod view;
use crate::environment::Environment;
use reducer::ReducerState;

pub use reducer::{Action, SCOPE_UPDATER};
pub use view::LoggedInApp;

type ViewStore<'a> = navicula::ViewStore<'a, RootReducer>;

pub struct RootReducer;
use navicula::reducer::Reducer;

impl Reducer for RootReducer {
    type Message = Action;

    type DelegateMessage = Action;

    type Action = Action;

    type State = ReducerState;

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
        reducer::reduce(context, action, state, environment)
    }

    fn initial_action() -> Option<Self::Action> {
        Some(Action::Login)
    }
}
