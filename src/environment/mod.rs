pub mod menu;
pub mod storage;
pub mod types;

mod native;
pub use native::*;

use std::rc::Rc;

use self::types::AppEvent;

pub trait OpenWindowState: Clone {
    type Action;
    fn window<'a, 'b>(
        &'a self,
        cx: dioxus::core::Scope<'b>,
        environment: &'a Environment,
        receiver: flume::Receiver<AppEvent>,
        parent_handler: Rc<dyn Fn(Self::Action)>,
    ) -> dioxus::core::Element<'b>
    where
        'a: 'b;
}

pub trait UploadMediaExt {
    fn id(&self) -> &str;
}
