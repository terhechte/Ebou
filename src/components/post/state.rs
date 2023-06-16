use std::path::PathBuf;
use std::str::FromStr;

use crate::environment::model::{Account, StatusVisibility};
use crate::environment::types::UiConfig;
use crate::view_model::{AttachmentMedia, StatusViewModel};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PostKind {
    Post,
    Reply(StatusViewModel),
    ReplyPrivate(StatusViewModel),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct State {
    pub account: Account,
    pub kind: PostKind,
    pub is_window: bool,
    pub image_paths: Vec<PathBuf>,
    pub images: Vec<AttachmentMedia>,
    pub posting: bool,
    pub dropping_file: bool,
    pub error_message: Option<String>,
    pub visibility: Option<Visibility>,
    pub text: String,
    pub validity: (bool, u32, u32),
    pub config: UiConfig,
}

impl State {
    pub fn new(account: Account, kind: PostKind, is_window: bool, paths: Vec<PathBuf>) -> Self {
        Self {
            account,
            kind,
            is_window,
            images: Default::default(),
            image_paths: paths,
            posting: Default::default(),
            dropping_file: Default::default(),
            error_message: Default::default(),
            visibility: Default::default(),
            text: Default::default(),
            validity: Default::default(),
            config: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Visibility {
    Public,
    Unlisted,
    Private,
    Direct,
}

impl FromStr for Visibility {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "public" => Ok(Visibility::Public),
            "unlisted" => Ok(Visibility::Unlisted),
            "private" => Ok(Visibility::Private),
            "direct" => Ok(Visibility::Direct),
            _ => Err("INVALID".to_string()),
        }
    }
}

impl From<&Visibility> for StatusVisibility {
    fn from(value: &Visibility) -> Self {
        match value {
            Visibility::Public => StatusVisibility::Public,
            Visibility::Unlisted => StatusVisibility::Unlisted,
            Visibility::Private => StatusVisibility::Private,
            Visibility::Direct => StatusVisibility::Direct,
        }
    }
}
