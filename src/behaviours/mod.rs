use dioxus::prelude::*;
use navicula::{types::AppWindow, Effect};

mod change_textsize;
pub use change_textsize::ChangeTextsizeBehaviour;

pub trait Behaviour {
    type InputAction;
    type Environment;
    type InputState;
    fn setup<'a, T>(cx: Scope<'a, T>, environment: &'a Self::Environment);
    fn handle<'a, 'b, OutputAction>(
        window: &'a AppWindow<'a>,
        action: Self::InputAction,
        state: &'a mut Self::InputState,
        environment: &'a Self::Environment,
    ) -> Effect<'b, OutputAction>;
}
