use std::sync::Arc;

use dioxus::prelude::ScopeState;
use dioxus_desktop::{LogicalSize, WindowBuilder};

pub use navicula::types::AppWindow;

use crate::environment::{
    storage::UiTab,
    types::{ActionFromEvent, AppEvent, MainMenuConfig},
};

pub fn default_window() -> WindowBuilder {
    let builder = WindowBuilder::new();
    let s = LogicalSize::new(1200., 775.);

    let builder = builder
        .with_title("Ebou")
        .with_theme(Some(dioxus_desktop::tao::window::Theme::Dark))
        .with_inner_size(s);
    builder
}

#[derive(Clone, Default)]
pub struct Platform {}

impl Platform {
    pub fn setup_toolbar(&self, _window: &AppWindow) {}
    pub fn update_menu<'a>(&self, _window: &AppWindow, _mutator: impl Fn(&mut MainMenuConfig)) {}
    pub fn update_toolbar(
        &self,
        _account: &str,
        _window: &AppWindow,
        _tab: &UiTab,
        _has_notifications: bool,
    ) {
    }
    pub fn handle_menu_events<A: ActionFromEvent + 'static>(
        &self,
        _cx: &ScopeState,
        _updater: Arc<dyn Fn(A) + Send + Sync>,
    ) {
    }
    pub fn set_toolbar_handler(&self, _handler: std::sync::Arc<dyn Fn(AppEvent) + Send + Sync>) {}
    pub fn loggedout_toolbar(&self, _window: &AppWindow) {}
}

pub fn apply_window_background<'a>(window: &AppWindow) {
    let webview = window.webview.clone();
    let native_window = webview.window();

    // use window_vibrancy::apply_blur;
    // apply_blur(&native_window, Some((18, 18, 18, 125)))
    //     .expect("Unsupported platform! 'apply_blur' is only supported on Windows");
}
