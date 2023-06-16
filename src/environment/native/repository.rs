use crate::environment::types::UiConfig;
use chrono::{DateTime, Utc};
use navicula::publisher::RefPublisher;

// use lazy_static::__Deref;
use super::super::types::{Marker, User};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{from_slice, to_string_pretty};
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};

const USERS_PATH: &str = "users.json";
const MARKERS_PATH: &str = "markers.json";
const UICONFIG_PATH: &str = "uiconfig.json";
const FAVORITES_PATH: &str = "favorites.json";

#[derive(Clone)]
pub struct Repository {
    users: Arc<Mutex<Vec<User>>>,
    markers: Arc<Mutex<Markers>>,
    pub favorites: RefPublisher<HashSet<String>>,
    ui_config: Arc<Mutex<UiConfig>>,
}

impl Repository {
    pub fn new() -> Self {
        let users = read(USERS_PATH)
            .ok()
            .flatten().unwrap_or_default();
        let markers = read(MARKERS_PATH)
            .ok()
            .flatten().unwrap_or_default();
        let ui_config = read(UICONFIG_PATH)
            .ok()
            .flatten().unwrap_or_default();
        let favorites = read(FAVORITES_PATH).ok().flatten().unwrap_or_default();

        // Try to write the users. Otherwise fail early
        write(USERS_PATH, &users)
            .expect("Expect to be able to write access token and logged in users");

        Self {
            users,
            markers,
            // FIXME: Make all items in here RefPublishers!
            favorites: RefPublisher::new(favorites),
            ui_config,
        }
    }

    pub fn update_or_insert_user(&self, new_user: User) -> Result<(), String> {
        let mut users = self
            .users
            .lock()
            .map_err(|e| format!("Accounts Data Error: {e:?}"))?;
        let mut found = false;
        for user in users.iter_mut() {
            if user.id == new_user.id {
                *user = new_user.clone();
                found = true;
                break;
            }
        }

        if !found {
            users.push(new_user);
        }

        if let Err(e) = write(USERS_PATH, users.deref()) {
            log::error!("Could not save users: {e:?}");
        }

        Ok(())
    }

    pub fn remove_user(&self, id: String) -> Result<(), String> {
        let mut users = self
            .users
            .lock()
            .map_err(|e| format!("Accounts Data Error: {e:?}"))?;
        let Some(id) = users.iter().position(|user| user.id == id) else {
            return Err(format!("Unknown User {id}"))
        };

        users.remove(id);

        if let Err(e) = write(USERS_PATH, users.deref()) {
            log::error!("Could not save users: {e:?}");
        }

        Ok(())
    }

    pub fn users(&self) -> Result<Vec<User>, String> {
        Ok(self
            .users
            .lock()
            .map_err(|e| format!("Accounts Data Error: {e:?}"))?
            .clone())
    }

    pub fn get_timeline_marker(&self, account: &str) -> Option<(String, DateTime<Utc>)> {
        let markers = &self
            .markers
            .lock()
            .map_err(|e| format!("Accounts Data Error: {e:?}"))
            .ok()?
            .timeline_markers;
        markers
            .get(account)
            .map(|marker| (marker.id.clone(), marker.set))
    }

    pub fn set_timeline_marker(&self, account: &str, status: &str) -> Option<()> {
        let mut markers = self
            .markers
            .lock()
            .map_err(|e| format!("Accounts Data Error: {e:?}"))
            .ok()?;
        markers.timeline_markers.insert(
            account.to_string(),
            Marker {
                set: Utc::now(),
                id: status.to_string(),
                marker_id: account.to_string(),
            },
        );
        if let Err(e) = write(MARKERS_PATH, markers.deref()) {
            log::error!("Could not save markers: {e:?}");
        }
        None
    }

    pub fn config(&self) -> Result<UiConfig, String> {
        Ok(self
            .ui_config
            .lock()
            .map_err(|e| format!("UiConfig Data Error: {e:?}"))?
            .clone())
    }

    pub fn set_config(&self, config: &UiConfig) -> Option<()> {
        let mut ui_config = self
            .ui_config
            .lock()
            .map_err(|e| format!("UiConfig Data Error: {e:?}"))
            .ok()?;
        *ui_config = config.clone();
        if let Err(e) = write(UICONFIG_PATH, config.deref()) {
            log::error!("Could not save config: {e:?}");
        }
        None
    }

    /// Return favorites, if they exists. Will return `None` if there're
    /// no favorites in the `HashSet` yet.
    pub fn favorites(&self) -> Option<HashSet<String>> {
        self.favorites
            .with(|m| if m.is_empty() { None } else { Some(m.clone()) })
    }

    pub fn toggle_favorite(&self, id: String) -> Result<(), String> {
        self.favorites.with_mutation(|mut favs| {
            if favs.contains(&id) {
                favs.remove(&id);
            } else {
                favs.insert(id);
            }
            if let Err(e) = write(FAVORITES_PATH, &*favs) {
                log::error!("Could not save favorites: {e:?}");
            }
        });
        Ok(())
    }

    pub fn is_favorite(&self, id: &str) -> Result<bool, String> {
        Ok(self.favorites.with(|favs| favs.contains(id)))
    }

    pub fn map_config<T>(
        &self,
        action: impl FnOnce(&mut MutexGuard<UiConfig>) -> T,
    ) -> Result<T, String> {
        let mut ui_config = self
            .ui_config
            .lock()
            .map_err(|e| format!("UiConfig Data Error: {e:?}"))?;
        let o = action(&mut ui_config);
        if let Err(e) = write(UICONFIG_PATH, ui_config.deref()) {
            log::error!("Could not save config: {e:?}");
        }
        Ok(o)
    }
}

type UserId = String;
// type ConversationId = String;

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
struct Markers {
    timeline_markers: HashMap<UserId, Marker>,
}

fn read<T: DeserializeOwned>(name: &str) -> Result<Option<T>, String> {
    let data_path = data_directory().join(name);
    if !data_path.exists() {
        return Ok(None);
    };
    let data = std::fs::read(&data_path)
        .map_err(|e| format!("Could not read {}: {e:?}", data_path.display()))?;
    let obj: T =
        from_slice(&data).map_err(|e| format!("Could not parse {}: {e:?}", data_path.display()))?;
    Ok(Some(obj))
}

fn write<T: Serialize>(name: &str, value: &T) -> Result<(), String> {
    let data_path = data_directory().join(name);
    let data = to_string_pretty(&value).map_err(|e| format!("Could not parse value:{e:?}"))?;
    std::fs::write(&data_path, data)
        .map_err(|e| format!("Could not write to {}: {e:?}", data_path.display()))?;
    Ok(())
}

fn data_directory() -> PathBuf {
    use directories_next::ProjectDirs;
    if let Some(proj_dirs) = ProjectDirs::from("com", "stylemac", "ebou") {
        let dirs = proj_dirs.config_dir().to_path_buf();
        if !dirs.exists() {
            if let Err(e) = std::fs::create_dir_all(&dirs) {
                log::error!("Could not create directory {}: {e:?}", dirs.display());
                panic!("Couldn't find a folder to save data")
            }
        }
        dirs
    } else {
        panic!("Couldn't find a folder to save data")
    }
}
