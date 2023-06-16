#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use self::windows::*;

#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum ToolbarSelection {
    Timeline = 0,
    Notifications = 1,
    Messages = 2,
    More = 3,
}
