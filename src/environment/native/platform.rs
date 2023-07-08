use dioxus::prelude::Scope;
use dioxus_desktop::tao::menu::MenuId;

use crate::environment::storage::UiTab;

use super::{super::types::MainMenuEvent, toolbar::ToolbarSelection};

use crate::view_model::{self, AttachmentMedia};
pub use navicula::types::AppWindow;
use std::path::PathBuf;
use strum::IntoEnumIterator;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use self::windows::*;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use self::linux::*;

#[cfg(target_os = "ios")]
mod ios;
#[cfg(target_os = "ios")]
pub use self::ios::*;

pub fn is_fullscreen<'a>(window: &'a AppWindow<'a>) -> bool {
    window.fullscreen().is_some()
}

impl MainMenuEvent {
    pub fn menu_id(&self) -> MenuId {
        MenuId::new(self.into())
    }

    pub fn resolve(menu_id: &MenuId) -> Option<MainMenuEvent> {
        for f in MainMenuEvent::iter() {
            let m = MenuId::new(f.into());
            if &m == menu_id {
                return Some(f);
            }
        }
        None
    }
}

#[allow(unused)]
fn tab_index(tab: &UiTab) -> ToolbarSelection {
    match tab {
        UiTab::Timeline => ToolbarSelection::Timeline,
        UiTab::Mentions => ToolbarSelection::Notifications,
        UiTab::Messages => ToolbarSelection::Messages,
        UiTab::More => ToolbarSelection::More,
    }
}

/// Sorta cross-platform way of opening a file
pub fn open_file(path: impl AsRef<std::path::Path>) {
    use std::process::Command;
    let Some(path) = path.as_ref().to_str() else {
        return
    };

    #[cfg(target_os = "windows")]
    {
        Command::new("explorer").arg(path).spawn().ok();
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open").arg(path).spawn().ok();
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open").args(["-R", path]).spawn().ok();
    }
}

pub fn temporary_directory() -> Option<std::path::PathBuf> {
    let dir = std::env::temp_dir();
    let folder = "com.stylemac.ebou";
    let path = dir.join(folder);
    if !path.exists() {
        if let Err(e) = std::fs::create_dir_all(&path) {
            log::error!("Could not create directory: {e:?}");
            return None;
        }
    }
    Some(path)
}

#[cfg(not(target_os = "ios"))]
pub fn open_file_dialog(directory: &str) -> Option<view_model::AttachmentMedia> {
    use rfd::FileDialog;
    let file = FileDialog::new()
        .add_filter("image", SUPPORTED_IMAGE_TYPES)
        .add_filter("video", SUPPORTED_VIDEO_TYPES)
        .set_directory(directory)
        .pick_file();

    let file = file?;

    read_file_to_attachment(&file)
}

#[cfg(target_os = "ios")]
pub fn open_file_dialog(directory: &str) -> Option<view_model::AttachmentMedia> {
    None
}

pub const SUPPORTED_IMAGE_TYPES: &[&str] = &["png", "jpg", "jpeg", "gif"];
pub const SUPPORTED_VIDEO_TYPES: &[&str] = &["mp4", "mov"];

/// filter out only the supported types
pub fn supported_file_types(files: &[PathBuf]) -> Vec<PathBuf> {
    let mut collected = Vec::new();
    for f in files {
        let Some(ext) = f.extension().and_then(|e| e.to_str()) else { continue };
        if SUPPORTED_IMAGE_TYPES.contains(&ext) || SUPPORTED_VIDEO_TYPES.contains(&ext) {
            collected.push(f.clone());
        }
    }
    collected
}

pub fn read_file_to_attachment(path: &PathBuf) -> Option<view_model::AttachmentMedia> {
    let is_image = if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        ["png", "jpeg", "jpg", "gif"].contains(&ext)
    } else {
        false
    };

    let name = path
        .file_name()
        .and_then(|e| e.to_str().map(|e| e.to_string()))
        .unwrap_or("unknown".to_string());

    let preview = if is_image {
        if let Ok(img) = image::open(path) {
            let resized =
                image::imageops::resize(&img, 64, 64, image::imageops::FilterType::Lanczos3);
            let mut buffer = std::io::Cursor::new(Vec::new());
            if resized
                .write_to(&mut buffer, image::ImageFormat::Png)
                .is_ok()
            {
                let v = buffer.into_inner();

                use base64::{engine::general_purpose, Engine as _};
                let string: String = general_purpose::STANDARD_NO_PAD.encode(v);
                Some(format!("data:image/jpeg;base64, {string}"))
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // copy the actual file to a temporary place
    // if that fails, just use the current path and hope for the best
    let new_path = if let Some(base_path) = temporary_directory() {
        let new_path = base_path.join(&name);
        if let Err(e) = std::fs::copy(path, &new_path) {
            log::error!("Could not copy file: {e:?}");
            path.clone()
        } else {
            new_path
        }
    } else {
        path.clone()
    };

    // if the data is an image, provide a preview
    // also, we upload by filename, so store it in a temporary folder
    let media = AttachmentMedia {
        preview,
        path: new_path,
        filename: name,
        description: None,
        is_uploaded: false,
        server_id: None,
    };
    Some(media)
}

pub fn copy_to_clipboard(content: impl AsRef<str>) {
    use copypasta::ClipboardContext;
    use copypasta::ClipboardProvider;
    let Ok(mut ctx) = ClipboardContext::new() else {
        log::error!("Could not write to clipboard");
        return;
    };
    if let Err(e) = ctx.set_contents(content.as_ref().to_string()) {
        log::error!("Could not write to clipboard: {e:?}");
    }
}

#[cfg(not(target_os = "macos"))]
pub fn format_datetime(datetime: &chrono::DateTime<chrono::Utc>) -> (String, String) {
    use chrono::Locale;

    let locale: Locale = {
        let current = current_locale::current_locale().unwrap_or("en_US".to_string());
        current.as_str().try_into().unwrap_or(Locale::en_US)
    };
    let detailed = datetime.format_localized("%x %r", locale).to_string();

    // if it is today, show only the time
    // if it is within the lst 7 days, show the weekday name
    // otherwise, the date
    let duration = chrono::Utc::now().signed_duration_since(*datetime);
    let human = if duration.num_hours() <= 24 {
        datetime.format_localized("%r", locale)
    } else if duration.num_days() <= 6 {
        datetime.format_localized("%A", locale)
    } else {
        datetime.format_localized("%x", locale)
    };
    (human.to_string(), detailed)
}

pub fn execute_js_once<T>(cx: &Scope<'_, T>, js: &str) {
    let js = js.to_string();
    let single = dioxus::prelude::use_state(cx, || false);
    let ev = dioxus_desktop::use_eval(cx).clone();
    if !(*single.get()) {
        cx.push_future(async move {
            ev(js);
        });
        single.set(true);
    }
}
