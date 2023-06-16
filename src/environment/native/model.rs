#![allow(unused)]
pub use megalodon::entities::attachment::*;
pub use megalodon::entities::{
    notification::NotificationType, Attachment, Card, Context, Emoji, Instance, Notification,
    Relationship, Status, StatusVisibility, Tag, UploadMedia,
};
use megalodon::megalodon::{
    GetArrayOptions, GetArrayWithSinceOptions, GetListTimelineInputOptions,
    GetNotificationsInputOptions, GetTimelineOptions, SearchAccountInputOptions,
};
pub use megalodon::streaming::Message;
use megalodon::{entities::List, megalodon::AccountFollowersInputOptions};

use megalodon::{
    megalodon::{
        FollowAccountInputOptions, GetAccountStatusesInputOptions, GetTimelineOptionsWithLocal,
        PostStatusInputOptions, UpdateMediaInputOptions, UploadMediaInputOptions,
    },
    Megalodon,
};
use reqwest::header::HeaderValue;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Model {
    pub url: String,
    pub has_token: bool,
    client: Arc<Box<dyn Megalodon + Send + Sync>>,
    instance: Arc<Mutex<Option<Instance>>>,
    is_logged_in: Arc<AtomicBool>,
}

impl std::fmt::Debug for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Model").finish()
    }
}

impl Default for Model {
    fn default() -> Self {
        Self::new("http://mastodon.social".to_string(), None)
    }
}

impl Model {
    pub fn new(url: String, token: Option<String>) -> Self {
        let has_token = token.is_some();
        let client = megalodon::generator(megalodon::SNS::Mastodon, url.clone(), token, None);
        Self {
            url,
            has_token,
            client: Arc::new(client),
            instance: Arc::default(),
            is_logged_in: Arc::new(AtomicBool::new(false)),
        }
    }

    #[allow(unused)]
    pub fn is_loggedin(&self) -> bool {
        self.is_logged_in.load(Ordering::SeqCst)
    }

    pub async fn register(&self) -> Result<AppData, String> {
        let scopes = "read read:accounts read:bookmarks read:favourites read:statuses write write:bookmarks write:favourites write:media write:statuses follow".split(' ')
        .map(|e| e.to_string())
        .collect();

        // read read:bookmarks read:favourites read:statuses write write:bookmarks write:favourites write:media write:statuses follow
        let options = megalodon::megalodon::AppInputOptions {
            scopes: Some(scopes),
            website: Some("https://terhech.de/ebou".to_string()),
            ..Default::default()
        };

        self.client
            .register_app("Ebou".to_string(), &options)
            .await
            .string_error("register")
            .map(AppData::from)
    }

    pub async fn authenticate(
        &self,
        client_id: String,
        client_secret: String,
        code: String,
    ) -> Result<TokenData, String> {
        log::trace!("Authenticate");
        self.client
            .fetch_access_token(
                client_id,
                client_secret,
                code.trim().to_string(),
                megalodon::default::NO_REDIRECT.to_string(),
            )
            .await
            .map(TokenData::from)
            .string_error("authenticate")
    }

    pub async fn login(&self) -> Result<Account, String> {
        let a = self.client.verify_account_credentials();
        let b = self.client.get_instance();
        let (a, b) = tokio::join!(a, b);

        let instance = b.map(|e| e.json).string_error("login")?;

        let _ = self.instance.lock().map(|mut e| e.replace(instance));

        let response = a
            .map(|e| e.json)
            .map(Account::from)
            .string_error("parse_instance");

        if response.is_ok() {
            self.is_logged_in.swap(true, Ordering::SeqCst);
        }

        response
    }

    pub fn instance(&self) -> Option<Instance> {
        self.instance.lock().ok()?.clone()
    }

    pub async fn logout(
        &self,
        client_id: String,
        client_secret: String,
        access_token: String,
    ) -> Result<(), String> {
        log::trace!("Logout");
        self.is_logged_in.swap(false, Ordering::SeqCst);
        self.client
            .revoke_access_token(client_id, client_secret, access_token)
            .await
            .string_error("logout")?;
        self.is_logged_in.swap(false, Ordering::SeqCst);
        Ok(())
    }

    pub async fn subscribe_user_stream(
        &self,
        sender: Arc<dyn Fn(Message) + Send + Sync>,
    ) -> Result<(), String> {
        log::trace!("Subscribe");
        let streaming_url = self
            .instance
            .lock()
            .map_err(|e| format!("Poison Error {e}"))?
            .as_ref()
            .map(|e| e.urls.streaming_api.clone())
            .ok_or("Could not connect to user stream: No streaming_api URL")?;

        let client = self.client.user_streaming(streaming_url);

        // this kinda sucks for multiple accounts?
        tokio::spawn(async move {
            client
                .listen(Box::new(move |message| {
                    sender(message);
                }))
                .await;
        });

        Ok(())
    }

    /// Keeps all `Status` items in the `self.posts` and returns only the new ones
    /// We load multiple pages of data
    pub async fn timeline(
        &self,
        after: Option<String>,
        pages: usize,
    ) -> Result<Vec<Status>, String> {
        log::trace!("Timeline");
        let per_page = 40;
        let total_pages = pages;
        let mut last_page = after;
        let mut all_data = Vec::with_capacity(per_page * total_pages);
        for _ in 0..=total_pages {
            let options = GetTimelineOptionsWithLocal {
                limit: Some(40),
                max_id: last_page,
                ..Default::default()
            };
            let response = self
                .client
                .get_home_timeline(Some(&options))
                .await
                .string_error("timeline")?;
            let mut data = response.json;
            let Some(last) = data.last() else { break };
            last_page = Some(last.id.clone());
            all_data.append(&mut data);
        }
        log::trace!("timeline data arrived {}", all_data.len());
        Ok(all_data)
    }

    pub async fn user_timeline(
        &self,
        id: String,
        after: Option<String>,
        since: Option<String>,
        limit: Option<u32>,
    ) -> Result<Vec<Status>, String> {
        log::trace!("User Logout");
        let options = GetAccountStatusesInputOptions {
            limit,
            max_id: after,
            since_id: since,
            exclude_replies: Some(false),
            exclude_reblogs: Some(false),
            ..Default::default()
        };
        let response = self
            .client
            .get_account_statuses(id.to_string(), Some(&options))
            .await
            .string_error("user_timeline")?;
        Ok(response.json)
    }

    pub async fn single_status(&self, id: String) -> Result<Status, String> {
        log::trace!("Single Status");
        self.client
            .get_status(id)
            .await
            .map(|e| e.json)
            .string_error("single_status")
    }

    pub async fn status_context(&self, id: String) -> Result<Context, String> {
        log::trace!("Status Context");
        self.client
            .get_status_context(id, None)
            .await
            .map(|e| e.json)
            .string_error("status_context")
    }

    pub async fn search_account(
        &self,
        term: String,
        following: bool,
    ) -> Result<Vec<Account>, String> {
        log::trace!("Search Account");
        let options = SearchAccountInputOptions {
            following: Some(following),
            resolve: Some(false),
            limit: Some(40),
            ..Default::default()
        };
        self.client
            .search_account(term, Some(&options))
            .await
            .map(|e| e.json)
            .map(|items| items.into_iter().map(Account::from).collect())
            .string_error("search_account")
    }

    /// Get the relationship for a single user
    pub async fn relationship(&self, id: String) -> Result<Relationship, String> {
        log::trace!("Relationship");
        let data = self
            .client
            .get_relationships(vec![id])
            .await
            .map(|e| e.json.first().cloned())
            .string_error("search_account")?;
        data.ok_or("No relationship found".to_string())
    }

    pub async fn lists(&self) -> Result<Vec<List>, String> {
        log::trace!("Lists");
        self.client
            .get_lists()
            .await
            .map(|e| e.json)
            .string_error("search_account")
    }

    pub async fn list_timeline(
        &self,
        id: String,
        after: Option<String>,
        pages: usize,
    ) -> Result<Vec<Status>, String> {
        log::trace!("Load list timeline for {id}");
        let per_page = 40;
        let total_pages = pages;
        let mut last_page = after;
        let mut all_data = Vec::with_capacity(per_page * total_pages);
        for _ in 0..=total_pages {
            let options = GetListTimelineInputOptions {
                limit: Some(40),
                max_id: last_page,
                ..Default::default()
            };
            let response = self
                .client
                .get_list_timeline(id.clone(), Some(&options))
                .await
                .string_error("timeline")?;
            let mut data = response.json;
            let Some(last) = data.last() else { break };
            last_page = Some(last.id.clone());
            all_data.append(&mut data);
        }
        log::trace!("Found {} list timeline entries for ", all_data.len());
        Ok(all_data)
    }

    pub async fn notifications(
        &self,
        after: Option<String>,
        pages: usize,
    ) -> Result<Vec<Notification>, String> {
        log::trace!("Notifications");
        let per_page = 40;
        let total_pages = pages;
        let mut last_page = after;
        let mut all_data = Vec::with_capacity(per_page * total_pages);
        for _ in 0..=total_pages {
            let options = GetNotificationsInputOptions {
                limit: Some(40),
                max_id: last_page,
                exclude_types: Some(vec![
                    NotificationType::Follow,
                    NotificationType::FollowRequest,
                    NotificationType::Reblog,
                    NotificationType::Favourite,
                    NotificationType::PollVote,
                    NotificationType::PollExpired,
                    NotificationType::EmojiReaction,
                ]),
                ..Default::default()
            };
            let response = match self
                .client
                .get_notifications(Some(&options))
                .await
                .string_error("notifications")
            {
                Ok(n) => n,
                Err(e) => {
                    log::error!("Notification Error: {e:?}");
                    return Ok(all_data);
                }
            };
            let mut data = response.json;
            let Some(last) = data.last() else { break };
            last_page = Some(last.id.clone());
            all_data.append(&mut data);
        }
        Ok(all_data)
    }

    pub async fn set_bookmark(&self, id: String, on: bool) -> Result<Status, String> {
        let result = if on {
            self.client.bookmark_status(id).await
        } else {
            self.client.unbookmark_status(id).await
        };
        result.map(|e| e.json).map_err(|e| format!("Error: {e:?}"))
    }

    pub async fn set_favourite(&self, id: String, on: bool) -> Result<Status, String> {
        let result = if on {
            self.client.favourite_status(id).await
        } else {
            self.client.unfavourite_status(id).await
        };
        result.map(|e| e.json).map_err(|e| format!("Error: {e:?}"))
    }

    pub async fn set_reblog(&self, id: String, on: bool) -> Result<Status, String> {
        let result = if on {
            self.client.reblog_status(id).await
        } else {
            self.client.unreblog_status(id).await
        };
        result.map(|e| e.json).map_err(|e| format!("Error: {e:?}"))
    }

    pub async fn upload_media(
        &self,
        path: &Path,
        description: Option<String>,
    ) -> Result<UploadMedia, String> {
        let Some(file_path) = path.to_str().map(|e| e.to_string()) else {
            return Err("Invalid Path".to_string())
        };
        let options = description.map(|e| UploadMediaInputOptions {
            description: Some(e),
            ..Default::default()
        });
        self.client
            .upload_media(file_path, options.as_ref())
            .await
            .map(|e| e.json)
            .string_error("upload_media")
    }

    pub async fn update_media(
        &self,
        id: String,
        description: Option<String>,
    ) -> Result<(), String> {
        let options = UpdateMediaInputOptions {
            description,
            ..Default::default()
        };
        self.client
            .update_media(id, Some(&options))
            .await
            .map(|_| ())
            .string_error("update_media")
    }

    pub async fn post_status(
        &self,
        status: String,
        media_ids: Option<Vec<String>>,
        in_reply_to_id: Option<String>,
        quote_id: Option<String>,
        visibility: Option<StatusVisibility>,
    ) -> Result<Status, String> {
        {
            let options = PostStatusInputOptions {
                media_ids,
                in_reply_to_id,
                quote_id,
                visibility,
                ..Default::default()
            };
            self.client
                .post_status(status, Some(&options))
                .await
                .map(|e| e.json)
                .string_error("post_status")
        }
    }

    pub async fn tag(&self, name: String) -> Result<Tag, String> {
        log::trace!("Tag");
        self.client
            .get_tag(name)
            .await
            .map(|e| e.json)
            .string_error("tag")
    }

    /// returns always true in the Result to distinguish from unfollow (false)
    pub async fn follow(&self, userid: String) -> Result<bool, String> {
        let options = FollowAccountInputOptions {
            reblog: Some(true),
            notify: None,
        };
        self.client
            .follow_account(userid, Some(&options))
            .await
            .map(|_| true)
            .string_error("follow")
    }

    /// returns always false in the Result to distinguish from follow (true)
    pub async fn unfollow(&self, userid: String) -> Result<bool, String> {
        self.client
            .unfollow_account(userid)
            .await
            .map(|_| false)
            .string_error("unfollow")
    }

    pub async fn followers(
        &self,
        id: String,
        after: Option<String>,
    ) -> Result<Vec<Account>, String> {
        log::trace!("Followers");
        let options = AccountFollowersInputOptions {
            limit: Some(80),
            max_id: after,
            ..Default::default()
        };
        self.client
            .get_account_followers(id, Some(&options))
            .await
            .map(|r| (r.json, parse_lheader(r.header.get("link"))))
            .map(|a| {
                a.0.into_iter()
                    .map(|e| Account::new(e, a.1.clone()))
                    .collect()
            })
            .string_error("followers")
    }

    pub async fn following(
        &self,
        id: String,
        after: Option<String>,
    ) -> Result<Vec<Account>, String> {
        log::trace!("Following");
        let options = AccountFollowersInputOptions {
            limit: Some(80),
            max_id: after,
            ..Default::default()
        };
        dbg!(&options);
        self.client
            .get_account_following(id, Some(&options))
            .await
            .map(|r| (r.json, parse_lheader(r.header.get("link"))))
            .map(|a| {
                a.0.into_iter()
                    .map(|e| Account::new(e, a.1.clone()))
                    .collect()
            })
            .string_error("following")
    }

    pub async fn bookmarks(&self, after: Option<String>) -> Result<Vec<Status>, String> {
        let options = GetArrayWithSinceOptions {
            limit: Some(40),
            max_id: after,
            ..Default::default()
        };
        self.client
            .get_bookmarks(Some(&options))
            .await
            .map(|r| r.json)
            .string_error("bookmarks")
    }

    pub async fn favorites(&self, after: Option<String>) -> Result<Vec<Status>, String> {
        let options = GetArrayOptions {
            limit: Some(40),
            max_id: after,
            ..Default::default()
        };
        self.client
            .get_favourites(Some(&options))
            .await
            .map(|r| r.json)
            .string_error("favorites")
    }

    pub async fn local_timeline(&self, after: Option<String>) -> Result<Vec<Status>, String> {
        log::trace!("Local Timeline");
        let options = GetTimelineOptions {
            limit: Some(40),
            max_id: after,
            ..Default::default()
        };
        self.client
            .get_local_timeline(Some(&options))
            .await
            .map(|r| r.json)
            .string_error("local_timeline")
    }

    pub async fn public_timeline(&self, after: Option<String>) -> Result<Vec<Status>, String> {
        log::trace!("Public Timeline");
        let options = GetTimelineOptions {
            limit: Some(40),
            max_id: after,
            ..Default::default()
        };
        self.client
            .get_public_timeline(Some(&options))
            .await
            .map(|r| r.json)
            .string_error("public_timeline")
    }

    pub async fn dm_timeline(&self, after: Option<String>) -> Result<Vec<Status>, String> {
        log::trace!("Public Timeline");
        let options = GetArrayWithSinceOptions {
            limit: Some(40),
            max_id: after,
            ..Default::default()
        };
        self.client
            .get_conversation_timeline(Some(&options))
            .await
            .map(|r| r.json)
            .string_error("dm_timeline")
    }
}

fn parse_lheader(from: Option<&HeaderValue>) -> Option<String> {
    let o = from?.to_str().ok()?;
    // quick hack. get the max_id= until the '>
    let ptn = "max_id=";
    let idx1 = o.find(ptn)? + ptn.len();
    let s = o.split_at(idx1);
    let idx2 = s.1.find('>')?;
    let s = s.1.split_at(idx2);
    Some(s.0.to_string())
}

trait ResultExt {
    type Output;
    fn string_error(self, call: &'static str) -> Result<Self::Output, String>;
}

impl<T, E: std::fmt::Debug> ResultExt for Result<T, E> {
    type Output = T;
    fn string_error(self, call: &'static str) -> Result<T, String> {
        self.map_err(|e| {
            let string_error = format!("API Error: {call} {e:?}");
            log::error!("{string_error}");
            string_error
        })
    }
}

use super::super::UploadMediaExt;
impl UploadMediaExt for UploadMedia {
    fn id(&self) -> &str {
        match self {
            UploadMedia::Attachment(a) => &a.id,
            UploadMedia::AsyncAttachment(a) => &a.id,
        }
    }
}

// Wrapp a couple of types to add Eq/PartialEq

#[derive(Debug, Clone)]
pub struct AppData(megalodon::oauth::AppData);

impl PartialEq for AppData {
    fn eq(&self, other: &Self) -> bool {
        self.0.id == other.0.id
    }
}

impl Eq for AppData {}

impl From<megalodon::oauth::AppData> for AppData {
    fn from(value: megalodon::oauth::AppData) -> Self {
        AppData(value)
    }
}

impl std::ops::Deref for AppData {
    type Target = megalodon::oauth::AppData;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct TokenData(megalodon::oauth::TokenData);

impl PartialEq for TokenData {
    fn eq(&self, other: &Self) -> bool {
        self.0.access_token == other.0.access_token
    }
}

impl Eq for TokenData {}

impl From<megalodon::oauth::TokenData> for TokenData {
    fn from(value: megalodon::oauth::TokenData) -> Self {
        TokenData(value)
    }
}

impl std::ops::Deref for TokenData {
    type Target = megalodon::oauth::TokenData;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct Account {
    d: megalodon::entities::Account,
    pub next: Option<String>,
}

impl Account {
    fn new(d: megalodon::entities::Account, next: Option<String>) -> Self {
        Self { d, next }
    }
}

impl PartialEq for Account {
    fn eq(&self, other: &Self) -> bool {
        self.d.id == other.d.id
    }
}

impl Eq for Account {}

impl From<megalodon::entities::Account> for Account {
    fn from(value: megalodon::entities::Account) -> Self {
        Account {
            d: value,
            next: None,
        }
    }
}

impl std::ops::Deref for Account {
    type Target = megalodon::entities::Account;

    fn deref(&self) -> &Self::Target {
        &self.d
    }
}

#[allow(unused)]
#[cfg(debug_assertions)]
pub mod mock {
    use chrono::Utc;
    pub use megalodon::entities::attachment::*;
    pub use megalodon::entities::*;

    pub fn make_account() -> super::Account {
        super::Account::new(Account {
            id: "asdf".to_string(),
            username: "Foone".to_string(),
            display_name: "Longfoone@alskjf".to_string(),
            acct: String::new(),
            locked: false,
            created_at: Utc::now(),
            followers_count: 0,
            following_count: 0,
            statuses_count: 0,
            note: String::new(),
            url: String::new(),
            avatar: "https://files.mastodon.social/cache/accounts/avatars/109/293/508/408/537/512/original/4d3f2a1dab779b5b.jpg".to_string(),
            avatar_static: "https://files.mastodon.social/cache/accounts/avatars/109/293/508/408/537/512/original/4d3f2a1dab779b5b.jpg".to_string(),
            header: "https://files.mastodon.social/cache/accounts/avatars/109/293/508/408/537/512/original/4d3f2a1dab779b5b.jpg".to_string(),
            header_static: "https://files.mastodon.social/cache/accounts/avatars/109/293/508/408/537/512/original/4d3f2a1dab779b5b.jpg".to_string(),
            emojis: vec![],
            moved: None,
            fields: None,
            bot: None,
            source: None
        }, None)
    }

    pub fn make_status() -> super::Status {
        Status {
            id: "my-id".to_string(),
            uri: Default::default(),
            url: None,
            account: Account {
                id: "109325706684051157".to_string(),
                username: "terhechte".to_string(),
                acct: String::new(),
                display_name: "terhechte".to_string(),
                locked: false,
                created_at: Utc::now(),
                followers_count: 0,
                following_count: 0,
                statuses_count: 0,
                note: String::new(),
                url: String::new(),
                avatar: String::new(),
                avatar_static: String::new(),
                header: String::new(),
                header_static: String::new(),
                emojis: Vec::new(),
                moved: None,
                fields: None,
                bot: None,
                source: None,
            },
            in_reply_to_id: None,
            in_reply_to_account_id: None,
            reblog: None,
            content: String::new(),
            plain_content: Some("mockmock".to_string()),
            created_at: Utc::now(),
            emojis: Vec::new(),
            replies_count: 0,
            reblogs_count: 0,
            favourites_count: 0,
            reblogged: None,
            favourited: None,
            muted: None,
            sensitive: false,
            spoiler_text: String::new(),
            visibility: StatusVisibility::Public,
            media_attachments: Vec::new(),
            mentions: Vec::new(),
            tags: Vec::new(),
            card: None,
            poll: None,
            application: None,
            language: None,
            pinned: None,
            emoji_reactions: None,
            quote: false,
            bookmarked: None,
        }
    }
}
