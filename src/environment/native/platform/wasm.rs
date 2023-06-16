use super::Scope;
use std::path::PathBuf;

use crate::view_model::{self, AttachmentMedia};

pub fn show_emoji_popup(cx: Scope) {}

pub fn open_file_dialog(directory: &str) -> Option<view_model::AttachmentMedia> {
    Some(crate::view_model::AttachmentMedia {
        preview: None,
        path: std::path::PathBuf::new(),
        filename: "Mockfile.mp4".to_string(),
        description: None,
        is_uploaded: false,
        server_id: None,
    })
}

pub fn read_file_to_attachment(path: PathBuf) -> Option<view_model::AttachmentMedia> {
    None
}

pub fn open_file(path: impl AsRef<std::path::Path>) {}

pub fn execute_js<'a, T>(cx: &Scope<'a, T>, js: &str) {
    let ev = dioxus_web::use_eval(cx);
    ev(js.to_string());
}

pub fn execute_js_once<'a, T>(cx: &Scope<'a, T>, js: &str) {
    use dioxus::prelude::use_state;
    let js = js.to_string();
    let single = use_state(cx, || false);
    let ev = dioxus_web::use_eval(cx).clone();
    if *single.get() == false {
        cx.push_future(async move {
            ev(js);
        });
        single.set(true);
    }
}

pub fn copy_to_clipboard(content: impl AsRef<str>) {}
