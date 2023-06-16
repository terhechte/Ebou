#![allow(unused)]
use std::rc::Rc;

use crate::environment::{types::AppEvent, Environment, OpenWindowState};
use dioxus::prelude::*;

#[derive(Clone, Copy)]
pub enum ImageWindowKind {
    Image,
    Video,
}

#[derive(Clone)]
pub struct ImageWindowState(pub String, pub ImageWindowKind);

impl OpenWindowState for ImageWindowState {
    type Action = ();
    fn window<'a, 'b>(
        &'a self,
        cx: Scope<'b>,
        environment: &'a Environment,
        receiver: flume::Receiver<AppEvent>,
        parent_handler: Rc<dyn Fn(Self::Action)>,
    ) -> Element<'b>
    where
        'a: 'b,
    {
        let url = self.0.clone();
        cx.render(rsx!(div {
            match self.1 {
                ImageWindowKind::Image => rsx!(img {
                    style: "object-fit: contain; width: 100%; height: 100vh; object-position: center center;",
                    width: "100%",
                    height: "100%",
                    src: "{url}",
                }),
                ImageWindowKind::Video => rsx!(video {
                    // style: "width: 448px;",
                    width: "100%",
                    height: "100%",
                    controls: "true",
                    source {
                        src: "{url}"
                    }
                })
            }
        }))
    }
}
