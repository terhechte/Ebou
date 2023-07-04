use crate::environment::model::*;
use crate::helper::HtmlItem;
use chrono::{DateTime, Utc};
use enumset::EnumSetType;

use crate::icons::*;

use crate::helper::{clean_html, format_number};

use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct AccountViewModel {
    pub id: AccountId,
    pub image: String,
    pub image_header: String,
    pub username: String,
    pub display_name: String,
    pub display_name_html: String,
    pub acct: String,
    pub note_plain: String,
    pub note_html: Vec<HtmlItem>,
    pub joined_human: String,
    pub joined_full: String,
    pub joined: DateTime<Utc>,
    pub url: String,
    pub followers: u32,
    pub followers_str: String,
    pub following: u32,
    pub following_str: String,
    pub statuses: u32,
    pub statuses_str: String,
    pub header: String,
    pub fields: Vec<AccountField>,
}

impl PartialEq for AccountViewModel {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for AccountViewModel {}

impl AccountViewModel {
    pub fn new(account: &Account) -> Self {
        let (h, f) = crate::environment::platform::format_datetime(&account.created_at);
        let (mut plain, html) = match replace_emoji(&account.note, &account.emojis) {
            Some(n) => clean_html(&n),
            None => clean_html(&account.note),
        };
        if plain.len() > 140 {
            plain = plain.chars().take(140).collect();
            plain.push('…');
        }

        let fields: Vec<_> = account
            .fields
            .iter()
            .map(|f| AccountField::new(&f.name, &f.value, f.verified_at))
            .collect();

        let display_name_html = match replace_emoji(&account.display_name, &account.emojis) {
            Some(n) => n,
            None => account.display_name.clone(),
        };

        Self {
            id: AccountId(account.id.clone()),
            image: account.avatar_static.clone(),
            image_header: account.header_static.clone(),
            username: account.username.clone(),
            display_name: account.display_name.clone(),
            display_name_html,
            acct: account.acct.clone(),
            note_plain: plain,
            note_html: html,
            joined_human: h,
            joined_full: f,
            joined: account.created_at,
            url: account.url.clone(),
            followers: account.followers_count,
            followers_str: format_number(account.followers_count as i64),
            following: account.following_count,
            following_str: format_number(account.following_count as i64),
            statuses: account.statuses_count,
            statuses_str: format_number(account.statuses_count as i64),
            header: account.header_static.clone(),
            fields,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AccountField {
    /// The original name
    pub name: String,
    /// The original value
    pub value: String,
    /// A parsed value obtained by stripping HTML (or value)
    pub value_parsed: String,
    // If the field value contains a link, this is the parsed link
    pub link: Option<url::Url>,
    pub verified_at: Option<DateTime<Utc>>,
}

impl AccountField {
    /// Parse a Field, also try to find links in the `value` field
    pub fn new(name: &str, value: &str, verified_at: Option<DateTime<Utc>>) -> AccountField {
        let cleaned = clean_html(value);
        let parsed = cleaned
            .1
            .into_iter()
            .filter_map(|e| match e {
                HtmlItem::Link { url, .. } => url::Url::parse(&url)
                    .ok()
                    .map(|url| (url.host_str().unwrap_or("Link").to_string(), url)),
                HtmlItem::Mention { url, name } => {
                    url::Url::parse(&url).ok().map(|url| (name, url))
                }
                _ => None,
            })
            .next();
        let (value_parsed, link) = parsed.unzip();
        AccountField {
            name: name.to_string(),
            value: value.to_string(),
            value_parsed: value_parsed.unwrap_or_else(|| value.to_string()),
            link,
            verified_at,
        }
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct AccountUpdateViewModel {
    pub id: AccountId,
    pub favorited: bool,
    pub account: AccountViewModel,
    pub content: String,
    pub last_updated_human: String,
    pub last_updated_full: String,
    pub last_updated: DateTime<Utc>,
}

impl AccountUpdateViewModel {
    pub fn new(status: &StatusViewModel) -> Self {
        let account = status.account.clone();
        let mut content = if let Some(ref boosted_content) = status.reblog_status {
            format!("{} boosted: {}", account.username, boosted_content.text)
        } else {
            status.text.clone()
        };
        if content.len() > 140 {
            content = content.chars().take(140).collect();
            content.push('…');
        }
        Self {
            id: account.id.clone(),
            favorited: false,
            account,
            content,
            last_updated_human: status.created_human.clone(),
            last_updated_full: status.created_full.clone(),
            last_updated: status.created,
        }
    }
}

#[derive(EnumSetType, Debug, Serialize, Deserialize)]
pub enum AccountVisibility {
    Toots,
    Replies,
    Boosts,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct StatusViewModel {
    pub id: StatusId,
    pub uri: String,
    pub account: AccountViewModel,
    // pub username: String,
    // pub user_id: AccountId,
    // pub display_name: String,
    // pub user_url: String,
    // #[serde(default)]
    // pub user_acct: String,
    pub status_images: Vec<(String, String, String)>,
    pub created: DateTime<Utc>,
    pub created_human: String,
    pub created_full: String,
    pub reblog_status: Option<Box<StatusViewModel>>,
    pub content: Vec<HtmlItem>,
    pub card: Option<Card>,
    pub replies: String,
    pub replies_title: String,
    #[serde(default)]
    pub replies_count: u32,
    /// Is this a reply, except if it is a reply to ourselves
    pub is_reply: bool,
    /// Has the *current user* reblogged this
    #[serde(default)]
    pub has_reblogged: bool,
    /// Is this a reblog
    #[serde(default)]
    pub is_reblog: bool,
    #[serde(default)]
    pub reblog_count: u32,
    pub reblog: String,
    pub reblog_title: String,
    pub is_favourited: bool,
    pub favourited: String,
    #[serde(default)]
    pub favourited_count: u32,
    pub favourited_title: String,
    pub bookmarked_title: String,
    pub is_bookmarked: bool,
    pub share_title: String,
    pub mentions: Vec<String>,
    pub has_conversation: Option<StatusId>,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub media: Vec<VideoMedia>,
}

impl PartialEq for StatusViewModel {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.account == other.account
            && self.is_bookmarked == other.is_bookmarked
            && self.is_favourited == other.is_favourited
            && self.is_reblog == other.is_reblog
            && self.is_reply == other.is_reply
            && self.replies_count == other.replies_count
            && self.reblog_status == other.reblog_status
            && self.reblog == other.reblog
            && self.favourited_count == other.favourited_count
            && self.favourited == other.favourited
    }
}

impl Eq for StatusViewModel {}

impl std::fmt::Debug for StatusViewModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StatusViewModel")
            .field("id", &self.id)
            .field("user", &self.account.username)
            .finish()
    }
}

impl StatusViewModel {
    pub fn new(status: &Status) -> Self {
        let (h, f) = crate::environment::platform::format_datetime(&status.created_at);
        let reblog_status = status
            .reblog
            .as_ref()
            .map(|status| Box::new(StatusViewModel::new(status)));

        let has_reblogged = status.reblogged.unwrap_or_default();
        let is_reblog = reblog_status.is_some();
        let is_favourited = status.favourited.unwrap_or_default();
        let is_bookmarked = status.bookmarked.unwrap_or_default();
        let is_reply = status
            .in_reply_to_id
            .as_ref()
            .or_else(|| {
                // replies to ourselves, for this app, are not considered replies.
                // maybe we need a better name for this (has replied self, has replied others)
                let s = status.in_reply_to_account_id.as_ref();
                if s == Some(&status.account.id) {
                    None
                } else {
                    s
                }
            })
            .is_some();

        let status_images = status_images(status);

        let mentions: Vec<_> = status
            .mentions
            .iter()
            .map(|e| format!("@{}", e.acct))
            .collect();

        let media: Vec<_> = status
            .media_attachments
            .iter()
            .filter(|a| matches!(a.r#type, AttachmentType::Video | AttachmentType::Gifv))
            .map(|attachment| VideoMedia {
                preview_url: attachment.preview_url.clone(),
                video_url: attachment.url.clone(),
                description: attachment.description.as_ref().cloned().unwrap_or_default(),
            })
            .collect();

        // if we replied to a conversation, or if we were replied to,
        // then we have a conversation that can be loaded
        let has_conversation = status
            .in_reply_to_id
            .as_ref()
            .map(|e| StatusId(e.clone()))
            .or((status.replies_count > 0).then(|| StatusId(status.id.clone())));

        let (text, content) = match replace_emoji(&status.content, &status.emojis) {
            Some(n) => clean_html(&n),
            None => clean_html(&status.content),
        };

        // unfortunate clone :/. We need the `Account` wrapper for PartialEq until
        // we use the AccountViewModel everywhere. Here, status contains a
        // megalodon.account, but we need the wrapper. So right now we clone it just
        // for forwarding a ref. Once we stop using the account-wrapper in lieu of accountviewmodel
        // we should be fine
        let wrapped_account: Account = status.account.clone().into();
        let account = AccountViewModel::new(&wrapped_account);

        StatusViewModel {
            id: StatusId(status.id.clone()),
            account,
            uri: status.uri.clone(),
            status_images,
            created: status.created_at,
            created_human: h,
            created_full: f,
            reblog_status,
            content,
            card: status.card.clone(),
            replies: format_number(status.replies_count as i64),
            replies_title: "Reply to this status".to_owned(),
            replies_count: status.replies_count,
            is_reply,
            has_reblogged,
            is_reblog,
            reblog_count: status.reblogs_count,
            reblog: format_number(status.reblogs_count as i64),
            reblog_title: format!(
                "Reblogs{}",
                has_reblogged
                    .then_some(": You boosted this")
                    .unwrap_or_default()
            ),
            is_favourited,
            favourited: format_number(status.favourites_count as i64),
            favourited_title: format!(
                "Favorites{}",
                is_favourited
                    .then_some(": You favourited this")
                    .unwrap_or_default()
            ),
            favourited_count: status.favourites_count,
            is_bookmarked,
            bookmarked_title: format!(
                "Bookmark{}",
                is_bookmarked
                    .then_some(": You bookmarked this")
                    .unwrap_or_default()
            ),
            share_title: "Share this status".to_string(),
            mentions,
            has_conversation,
            text,
            media,
        }
    }

    /// Mutate the reply status. This happens when be mutate,
    /// before the backend sends back an update
    pub fn did_reply(&mut self) {
        self.is_reply = true;
        self.replies_count += 1;
        self.replies = format_number(self.replies_count as i64);
    }

    pub fn is_reblogged<T>(&self, action: impl Fn(bool, &'static str) -> T) -> T {
        let value = self.has_reblogged;
        let icon = if value { ICON_BOOST2 } else { ICON_BOOST1 };
        action(value, icon)
    }

    pub fn update_reblog(&mut self, on: bool) {
        // Se `update_favorited` below
        if !on && self.reblog_count > 0 {
            self.reblog_count -= 1;
        }
        self.reblog = format_number(self.reblog_count as i64);
    }

    pub fn is_bookmarked<T>(&self, action: impl Fn(bool, &'static str) -> T) -> T {
        let value = self.is_bookmarked;
        let icon = if value {
            ICON_BOOKMARK2
        } else {
            ICON_BOOKMARK1
        };
        action(value, icon)
    }

    pub fn is_favourited<T>(&self, action: impl Fn(bool, &'static str) -> T) -> T {
        let value = self.is_favourited;
        let icon = if value { ICON_STAR2 } else { ICON_STAR1 };
        action(value, icon)
    }

    pub fn update_favorited(&mut self, on: bool) {
        // This looks weird, but it seems like mastodon
        // doesn't return the updated favorite count when I
        // unfavourite something
        if !on && self.favourited_count > 0 {
            self.favourited_count -= 1;
        }
        self.favourited = format_number(self.favourited_count as i64);
    }
}

pub fn status_images(status: &Status) -> Vec<(String, String, String)> {
    status
        .media_attachments
        .iter()
        .filter_map(|item| match item.r#type {
            AttachmentType::Image => Some((
                item.description.as_deref().unwrap_or_default().to_string(),
                item.preview_url.as_deref().unwrap_or(&item.url).to_string(),
                item.url.clone(),
            )),
            _ => None,
        })
        .collect()
}

#[derive(Debug, Eq, PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct NotificationViewModel {
    pub id: String,
    pub message: String,
    pub status: StatusViewModel,
    pub date: DateTime<Utc>,
}

impl NotificationViewModel {
    pub fn new(notification: &Notification) -> Option<Self> {
        let status = notification.status.as_ref()?;
        let mut content = status
            .plain_content
            .clone()
            .unwrap_or_else(|| clean_html(&status.content).0);
        if content.len() > 140 {
            content = content.chars().take(140).collect();
            content.push('…');
        }
        let message = match notification.r#type {
            NotificationType::Mention => {
                format!("{} mentioned you: {content}", notification.account.username)
            }
            NotificationType::Status => {
                format!(
                    "{} shared an update: {content}",
                    notification.account.username
                )
            }
            _ => return None,
        };
        let status = StatusViewModel::new(status);
        let id = notification.id.clone();
        Some(Self {
            id,
            message,
            status,
            date: notification.created_at,
        })
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Default, Serialize, Deserialize)]
pub struct AccountId(pub String);

#[derive(Debug, Eq, PartialEq, Hash, Clone, Default, Serialize, Deserialize)]
pub struct StatusId(pub String);

impl StatusId {
    pub fn dom_id(&self) -> String {
        format!("status-{}", self.0)
    }
}

impl std::fmt::Display for StatusId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("StatusID:{}", self.0))
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct AttachmentMedia {
    /// Base 64 image preview
    pub preview: Option<String>,
    /// Path to the data on disk
    pub path: std::path::PathBuf,
    pub filename: String,
    pub description: Option<String>,
    pub is_uploaded: bool,
    pub server_id: Option<String>,
}

impl PartialEq for AttachmentMedia {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path && self.server_id == other.server_id
    }
}

impl Eq for AttachmentMedia {}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct VideoMedia {
    pub preview_url: Option<String>,
    pub video_url: String,
    pub description: String,
}

fn replace_emoji(input: &str, emojis: &[Emoji]) -> Option<String> {
    if emojis.is_empty() {
        return None;
    }
    if !input.contains(':') {
        return None;
    }
    let mut string = input.to_string();
    for emoji in emojis.iter() {
        let image = format!(
            "<img src=\"{}\" class=\"emoji-entry\" />",
            &emoji.static_url
        );
        string = string.replace(&format!(":{}:", emoji.shortcode), &image);
    }
    Some(string)
}
