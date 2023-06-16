#![allow(non_snake_case)]

use std::path::PathBuf;
use std::rc::Rc;

use crate::environment::model::Account;
use crate::environment::{types::AppEvent, Environment};

use crate::components::post::*;

use dioxus::prelude::*;

// Allow opening post as a window as well
use crate::environment::OpenWindowState;

#[derive(Clone, Debug)]
pub struct PostWindowState {
    kind: PostKind,
    dropped_images: Vec<PathBuf>,
    account: Account,
}

impl PostWindowState {
    pub fn new(kind: PostKind, dropped_images: Vec<PathBuf>, account: Account) -> Self {
        Self {
            kind,
            dropped_images,
            account,
        }
    }
}

impl OpenWindowState for PostWindowState {
    type Action = PostAction;
    fn window<'a, 'b>(
        &'a self,
        cx: Scope<'b>,
        environment: &'a Environment,
        receiver: flume::Receiver<AppEvent>,
        _parent_handler: Rc<dyn Fn(Self::Action)>,
    ) -> Element<'b>
    where
        'a: 'b,
    {
        let kind = self.kind.clone();
        let config = environment.repository.config().ok().unwrap_or_default();

        let state = State {
            account: self.account.clone(),
            kind,
            is_window: true,
            images: Vec::new(),
            image_paths: self.dropped_images.clone(),
            posting: false,
            error_message: None,
            dropping_file: false,
            visibility: None,
            text: String::new(),
            validity: (false, 0, 500),
            config,
        };

        let store: navicula::ViewStore<PostReducer> =
            navicula::root(cx, &[], environment, || state);

        for event in receiver.try_iter() {
            store.send(PostAction::AppEvent(event));
        }

        render! {
            PostView {
                store: store
            }
        }
    }
}
