use chrono::{DateTime, Utc};
use enumset::EnumSet;
use serde::{Deserialize, Serialize};

// Repository Types

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub last_login: DateTime<Utc>,
    pub instance_url: String,

    pub token_access_token: String,
    pub token_scope: String,
    pub token_created: u64,
    pub token_expires: Option<u64>,
    pub token_refresh_token: Option<String>,

    pub app_id: String,
    pub app_name: String,
    pub app_website: Option<String>,
    pub app_client_id: String,
    pub app_client_secret: String,
    pub app_auth_url: Option<String>,
    pub app_redirect_uri: String,
}

impl User {
    pub fn new(
        instance_url: String,
        account: super::model::Account,
        token: super::model::TokenData,
        data: super::model::AppData,
    ) -> Self {
        Self {
            id: account.id.clone(),
            name: account.display_name.clone(),
            last_login: Utc::now(),
            instance_url,
            token_access_token: token.access_token.clone(),
            token_scope: token.scope.clone(),
            token_created: token.created_at,
            token_expires: token.expires_in,
            token_refresh_token: token.refresh_token.clone(),
            app_id: data.id.clone(),
            app_name: data.name.clone(),
            app_website: data.website.clone(),
            app_client_id: data.client_id.clone(),
            app_client_secret: data.client_secret.clone(),
            app_auth_url: data.url.clone(),
            app_redirect_uri: data.redirect_uri.clone(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Marker {
    /// When was the marker set
    pub set: DateTime<Utc>,
    /// The last / highest status the user saw
    pub id: String,
    /// The id for which this marker is saved
    pub marker_id: String,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct UiConfig {
    pub visibility: im::HashMap<String, EnumSet<crate::view_model::AccountVisibility>>,
    pub last_notification_id: Option<StatusId>,
    pub zoom: UiZoom,
    #[serde(default)]
    pub direction: TimelineDirection,
    #[serde(default)]
    pub post_window_inline: bool,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Default)]
pub enum TimelineDirection {
    NewestBottom,
    #[default]
    NewestTop,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq, FromRepr)]
#[repr(u8)]
#[derive(Default)]
pub enum UiZoom {
    Z90 = 90,
    #[default]
    Z100 = 100,
    Z110 = 110,
    Z120 = 120,
    Z130 = 130,
    Z140 = 140,
    Z150 = 150,
}

impl UiZoom {
    pub fn css_class(&self) -> String {
        let value: u8 = *self as u8;
        format!("zoom{value}")
    }
}

impl UiZoom {
    const CHANGE: u8 = 10;

    pub fn increase(&self) -> Option<Self> {
        let mut v: u8 = *self as u8;
        v += Self::CHANGE;
        Self::from_repr(v)
    }

    pub fn decrease(&self) -> Option<Self> {
        let mut v: u8 = *self as u8;
        v -= Self::CHANGE;
        Self::from_repr(v)
    }
}

// Instance Types

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Instance {
    pub id: String,
    pub name: String,
    pub users: String,
    pub thumbnail: Option<String>,
}

impl Instance {
    pub fn url(&self) -> String {
        format!("https://{}", self.name)
    }
}

// Menu

use strum_macros::Display;
use strum_macros::EnumIter;
use strum_macros::FromRepr;
use strum_macros::IntoStaticStr;

use crate::view_model::StatusId;

#[derive(IntoStaticStr, EnumIter, Display, Debug, Clone, Copy, Eq, PartialEq)]
pub enum MainMenuEvent {
    NewPost,
    Logout,
    Reload,
    ScrollUp,
    ScrollDown,
    TextSizeIncrease,
    TextSizeDecrease,
    TextSizeReset,
    Timeline,
    Mentions,
    Messages,
    More,
    PostWindowSubmit,
    PostWindowAttachFile,
    EbouHelp,
    Settings,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub struct MainMenuConfig {
    pub logged_in: bool,
    pub enable_scroll: bool,
    pub enable_postwindow: bool,
}

pub trait ActionFromEvent {
    fn make_focus_event(focus: bool) -> Option<Self>
    where
        Self: Sized;
    fn make_menu_event(event: MainMenuEvent) -> Option<Self>
    where
        Self: Sized;
    fn make_close_window_event() -> Option<Self>
    where
        Self: Sized;
}

#[derive(Clone, Debug)]
pub enum AppEvent {
    FocusChange(bool),
    MenuEvent(crate::environment::types::MainMenuEvent),
    FileEvent(FileEvent),
    ClosingWindow,
}

impl ActionFromEvent for AppEvent {
    fn make_focus_event(focus: bool) -> Option<Self>
    where
        Self: Sized,
    {
        Some(AppEvent::FocusChange(focus))
    }
    fn make_menu_event(event: MainMenuEvent) -> Option<Self>
    where
        Self: Sized,
    {
        Some(AppEvent::MenuEvent(event))
    }
    fn make_close_window_event() -> Option<Self>
    where
        Self: Sized,
    {
        Some(AppEvent::ClosingWindow)
    }
}

#[derive(Clone, Debug)]
pub enum FileEvent {
    Hovering(bool),
    Dropped(Vec<std::path::PathBuf>),
    Cancelled,
}
