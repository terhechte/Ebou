#![allow(non_snake_case)]

use crate::environment::{model::Model, repository::Repository, Environment};
use dioxus::prelude::*;
use navicula::types::AppWindow;

use crate::behaviours::{Behaviour, ChangeTextsizeBehaviour};
use crate::environment::platform::default_window;

use crate::style::STYLE;

use dioxus_desktop::{Config, WindowCloseBehaviour};

pub fn run() {
    use env_logger::Env;
    use std::io::Write;
    //#[cfg(debug_assertions)]
    env_logger::Builder::from_env(Env::default().default_filter_or("debug"))
        .format(|buf, record| {
            writeln!(
                buf,
                "{}:{} {} [{}] - {}",
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .target(env_logger::Target::Stdout)
        .init();

    let style = STYLE;
    let script = include_str!("../public/script.js");
    let config = Config::new()
        .with_close_behaviour(WindowCloseBehaviour::LastWindowHides)
        .with_custom_head(format!(
            r#"
        <title>Ebou</title>
        <style>{style}</style>
        <script>{script}</script>
        "#
        ))
        .with_file_drop_handler(move |_window, file| {
            let Ok(updater) = crate::components::loggedin::SCOPE_UPDATER.lock() else { return true };
            if let Some(o) = updater.as_ref() {
                crate::environment::handle_file_event(file, o)
            } else {
                true
            }
        })
        .with_window(default_window());

    dioxus_desktop::launch_with_props(RootApp, RootAppProps {}, config);
}

pub struct RootAppProps {}

pub fn RootApp(cx: Scope<'_, RootAppProps>) -> Element<'_> {
    log::trace!("rerender root-app");
    let repository = use_state(cx, Repository::new);
    let user = repository
        .users()
        .ok()
        .and_then(|users| users.first().cloned());
    let has_user = user.is_some();

    let environment_state = use_state(cx, || {
        let model = if let Some(user) = user {
            Model::new(user.instance_url.clone(), Some(user.token_access_token))
        } else {
            Model::default()
        };

        Environment::new(model, repository.get().clone())
    });
    let environment = environment_state.get();

    let window = AppWindow::retrieve(cx);
    cx.use_hook(|| {
        crate::environment::platform::apply_window_background(&window);
        environment.platform.setup_toolbar(&window);
    });

    ChangeTextsizeBehaviour::setup(cx, environment);

    let should_show_login = use_state(cx, || !has_user);

    cx.render(rsx! {
        environment.model.has_token.then(||
            rsx!(crate::components::loggedin::LoggedInApp {
                environment: environment_state,
                should_show_login: should_show_login,
            })
        ),
        should_show_login.then(|| rsx!(crate::components::login::LoginApp {
            environment: environment_state,
            should_show_login: should_show_login
        }))
    })
}
