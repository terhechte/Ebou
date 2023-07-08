#![allow(non_snake_case)]
#[cfg(any(target_os = "macos", target_os = "ios"))]
#[macro_use]
extern crate objc;

mod app;
mod behaviours;
mod components;
mod environment;
mod helper;
mod icons;
mod style;
mod view_model;
mod widgets;
mod windows;

pub use app::run;
pub use environment::Instances;
pub use helper::clean_html;

/// Handy macro for future localization
#[macro_export]
macro_rules! loc {
    ($x:expr $(,)?) => {
        $x
    };
}

pub struct Defer(Box<dyn Fn()>);

impl Defer {
    pub fn action<A: Clone + 'static>(action: &A, handler: impl Fn(&A) + 'static) -> Self {
        let cloned = action.clone();
        let boxed = Box::new(move || handler(&cloned));
        Self(boxed)
    }
}

impl Drop for Defer {
    fn drop(&mut self) {
        (self.0)()
    }
}

use components::post::PostKind;
use view_model::{AccountViewModel, StatusViewModel};

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug)]
pub enum PublicAction {
    StatusMutation(StatusMutation, StatusViewModel),
    Conversation(StatusViewModel),
    OpenLink(String),
    OpenTag(String),
    OpenVideo(String),
    OpenImage(String),
    OpenProfile(AccountViewModel),
    /// Resolve a profile via a link
    OpenProfileLink(String),
    Copy(String),
    Post(PostKind),
    /// Close the current conversation
    Close,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum StatusMutation {
    Bookmark(bool),
    Favourite(bool),
    Boost(bool),
}

impl StatusMutation {
    fn new_status(&self) -> bool {
        match self {
            StatusMutation::Bookmark(a) => *a,
            StatusMutation::Favourite(a) => *a,
            StatusMutation::Boost(a) => *a,
        }
    }
}
