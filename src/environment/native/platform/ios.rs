use dioxus_desktop::wry::webview::WebviewExtIOS;
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

    builder
        .with_title("Ebou")
        .with_theme(Some(dioxus_desktop::tao::window::Theme::Dark))
        .with_inner_size(s)
}

#[derive(Clone, Default)]
pub struct Platform {}

impl Platform {
    pub fn setup_toolbar(&self, _window: &AppWindow) {}
    pub fn update_menu(&self, _window: &AppWindow, _mutator: impl Fn(&mut MainMenuConfig)) {}
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

pub fn apply_window_background(window: &AppWindow) {
    let webview = window.webview.clone();
    let mv = webview.webview();
    unsafe {
        let _: () = msg_send![mv, setInspectable: true];
    }
}
