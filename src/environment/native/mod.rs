pub mod instances;
use std::rc::Rc;

use flume::{Receiver, Sender};
pub use instances::Instances;

pub mod model;
pub use model::Model;

pub mod repository;
use navicula::publisher::RefPublisher;
use navicula::types::EnvironmentType;
pub use repository::Repository;

pub mod platform;
mod toolbar;

use self::platform::AppWindow;

use super::storage::Data;

use dioxus::prelude::*;
use dioxus_desktop::{Config, LogicalSize, WindowBuilder};

use super::{types, OpenWindowState};
use crate::environment::types::AppEvent;
use crate::style::STYLE;

use crate::behaviours::{Behaviour, ChangeTextsizeBehaviour};

#[derive(Clone)]
pub struct Environment {
    // For multi-account support, have this be a hashmap: account-id -> model
    pub model: Model,
    pub repository: Repository,
    pub instances: Instances,
    pub platform: platform::Platform,
    pub storage: RefPublisher<Data>,
}

impl EnvironmentType for Environment {
    type AppEvent = types::AppEvent;
}

impl std::fmt::Debug for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Environment").finish()
    }
}

impl Environment {
    pub fn new(model: Model, repository: Repository) -> Self {
        Self {
            model,
            repository,
            instances: Instances::default(),
            platform: platform::Platform::default(),
            storage: RefPublisher::default(),
        }
    }

    pub fn update_model(&mut self, model: Model) {
        self.model = model;
    }

    pub fn open_url(&self, url: &str) {
        let _ = webbrowser::open(url);
    }

    pub fn open_window<S: OpenWindowState + 'static, Action: 'static>(
        &self,
        window: &AppWindow,
        state: S,
        width: f64,
        height: f64,
        title: impl AsRef<str>,
        parent_handler: Rc<dyn Fn(Action)>,
    ) {
        let dom = VirtualDom::new(new_window_popup::<S>);
        let style = STYLE;

        let s = LogicalSize::new(width, height);
        let builder = WindowBuilder::new()
            .with_theme(Some(dioxus_desktop::tao::window::Theme::Dark))
            .with_title(title.as_ref())
            .with_inner_size(s);

        let dom_scope = dom.base_scope();
        dom_scope.provide_context(state);

        let (sender, receiver) = flume::unbounded();

        // add a sender for file events
        dom_scope.provide_context(receiver);
        dom_scope.provide_context(sender.clone());
        dom_scope.provide_context(parent_handler.clone());

        let updater = dom_scope.schedule_update();
        let moved_sender = sender;
        let ux: Arc<dyn Fn(AppEvent) + Send + Sync> = Arc::new(move |a: AppEvent| {
            let _ = moved_sender.send(a);
            updater();
        });
        let config = Config::new()
            .with_custom_head(format!(
                r#"
        <style>{style}</style>
        <meta name='color-scheme' content='dark'>
        "#
            ))
            .with_file_drop_handler(move |_window, file| handle_file_event(file, &ux))
            .with_window(builder);

        dom_scope.provide_context(self.clone());

        window.new_window(dom, config);
    }
}

fn new_window_popup<S: OpenWindowState + 'static>(cx: Scope) -> Element {
    let Some(window_state) = cx.use_hook(|| cx.consume_context::<S>()) else {
        return cx.render(rsx!(div {format!("Failed consume_context {}", std::any::type_name::<S>())}));
    };
    let Some(receiver) = cx.use_hook(|| cx.consume_context::<Receiver<AppEvent>>()) else {
        return cx.render(rsx!(div {"Failed consume_context `DefaultEventReceiver`"}));
    };
    let Some(environment) = cx.use_hook(|| cx.consume_context::<Environment>()) else {
        return cx.render(rsx!(div {"Failed consume_context `Environment`"}));
    };
    let Some(sender) = cx.use_hook(|| cx.consume_context::<Sender<AppEvent>>()) else {
        return cx.render(rsx!(div {"Failed consume_context `Sender<AppEvent>`"}));
    };
    let Some(parent_handler) = cx.use_hook(|| cx.consume_context::<Rc<dyn Fn(S::Action)>>()) else {
        return cx.render(rsx!(div {format!("Failed consume_context Rc<dyn Fn({})>", std::any::type_name::<S::Action>())}));
    };

    let updater = cx.schedule_update();

    let cloned_sender = sender.clone();

    environment.platform.handle_menu_events(
        cx,
        Arc::new(move |a: AppEvent| {
            let _ = cloned_sender.send(a);
            updater();
        }),
    );

    ChangeTextsizeBehaviour::setup(cx, environment);

    #[cfg(target_os = "macos")]
    let window = dioxus_desktop::use_window(cx);

    cx.use_hook(move || {
        #[cfg(target_os = "macos")]
        {
            let webview = window.webview.clone();
            use dioxus_desktop::wry::webview::WebviewExtMacOS;
            unsafe {
                let native_webview = webview.webview();
                use cacao::foundation::NSNumber;
                use cacao::foundation::NSString;
                use cacao::foundation::NO;
                let n = NSNumber::bool(true);
                let s = NSString::new("drawsTransparentBackground");
                let _: () = msg_send![native_webview, setValue:n forKey:s];
                let _: () = msg_send![native_webview, setHidden: NO];
            }
        }
    });

    render! {
        rsx! {
            window_state.window(
                cx,
                environment,
                receiver.clone(),
                parent_handler.clone()
            )
        }
    }
}

use std::sync::Arc;

use dioxus_desktop::wry::webview::FileDropEvent;

pub fn handle_file_event(
    file: FileDropEvent,
    updater: &Arc<dyn Fn(AppEvent) + Send + Sync>,
) -> bool {
    use crate::{environment::platform::supported_file_types, environment::types::FileEvent};

    match file {
        FileDropEvent::Hovered { paths, .. } => {
            let files = supported_file_types(&paths);
            let allowed = !files.is_empty();
            updater(AppEvent::FileEvent(FileEvent::Hovering(allowed)));
        }
        FileDropEvent::Dropped { paths, .. } => {
            let collected_files = supported_file_types(&paths);
            if !collected_files.is_empty() {
                updater(AppEvent::FileEvent(FileEvent::Dropped(collected_files)));
            }
        }
        FileDropEvent::Cancelled => {
            updater(AppEvent::FileEvent(FileEvent::Cancelled));
        }
        _ => (),
    }
    // We always return true, otherwise the image will replace the HTML in webkit
    true
}
