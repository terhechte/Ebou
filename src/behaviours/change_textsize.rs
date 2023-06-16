use dioxus::prelude::*;

use crate::environment::types::{MainMenuEvent, UiConfig, UiZoom};
use navicula::{types::AppWindow, Effect};

use super::Behaviour;

pub struct ChangeTextsizeBehaviour {}

impl Behaviour for ChangeTextsizeBehaviour {
    type InputAction = MainMenuEvent;
    type InputState = UiConfig;
    type Environment = crate::environment::Environment;

    fn setup<T>(cx: Scope<'_, T>, environment: &Self::Environment) {
        if let Ok(config) = environment.repository.config() {
            let zoom = config.zoom.css_class();
            log::debug!("startup zoom {zoom}");
            crate::environment::platform::execute_js_once(
                &cx,
                &format!(
                    r#"
                document.documentElement.classList.add("{zoom}");
                "#
                ),
            );
        }
    }

    fn handle<'a, 'b, OutputAction>(
        _window: &'a AppWindow<'a>,
        action: MainMenuEvent,
        state: &'a mut Self::InputState,
        _environment: &'a Self::Environment,
    ) -> Effect<'b, OutputAction> {
        match action {
            MainMenuEvent::TextSizeIncrease => {
                let new = state.zoom.increase();
                change_textsize(state, new)
            }
            MainMenuEvent::TextSizeDecrease => {
                let new = state.zoom.decrease();
                change_textsize(state, new)
            }
            MainMenuEvent::TextSizeReset => change_textsize(state, Some(UiZoom::Z100)),
            _ => Effect::NONE,
        }
    }
}

fn change_textsize<'b, A>(state: &mut UiConfig, new: Option<UiZoom>) -> Effect<'b, A> {
    let current = state.zoom.css_class();
    let Some(new) = new else {
        return Effect::NONE
    };
    log::debug!("change: old zoom {:?}", state.zoom);
    log::debug!("change: new zoom {:?}", new);
    state.zoom = new;
    let new = new.css_class();
    Effect::ui(format!(
        r#"
        document.documentElement.classList.remove("{current}");
        document.documentElement.classList.add("{new}");
    "#
    ))
}
