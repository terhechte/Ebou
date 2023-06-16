mod reducer;
mod view;

pub use reducer::{MoreSelection, SidebarAction, SidebarDelegateAction, SidebarState};
pub use view::SidebarComponent;

pub struct SidebarReducer;
use navicula::reducer::Reducer;

impl Reducer for SidebarReducer {
    type Message = ();

    type DelegateMessage = reducer::SidebarDelegateAction;

    type Action = reducer::SidebarAction;

    type State = reducer::SidebarState;

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
        Some(SidebarAction::Initial)
    }
}
