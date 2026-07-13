use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};

static DB: std::sync::OnceLock<Mutex<OmatunesDb>> = std::sync::OnceLock::new();
static DB_DIRTY: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GroupBy {
    None,
    Album,
    Artist,
    Genre,
    Year,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TableColumn {
    TrackNumber,
    Title,
    Artist,
    Album,
    Genre,
    Year,
    DiscNumber,
    Duration,
    Plays,
    DatePlayed,
    Liked,
}

impl TableColumn {
    pub fn all() -> &'static [TableColumn] {
        &[
            TableColumn::TrackNumber,
            TableColumn::Title,
            TableColumn::Artist,
            TableColumn::Album,
            TableColumn::Genre,
            TableColumn::Year,
            TableColumn::DiscNumber,
            TableColumn::Duration,
            TableColumn::Plays,
            TableColumn::DatePlayed,
            TableColumn::Liked,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            TableColumn::TrackNumber => "#",
            TableColumn::Title => "Title",
            TableColumn::Artist => "Artist",
            TableColumn::Album => "Album",
            TableColumn::Genre => "Genre",
            TableColumn::Year => "Year",
            TableColumn::DiscNumber => "Disc #",
            TableColumn::Duration => "Duration",
            TableColumn::Plays => "Plays",
            TableColumn::DatePlayed => "Date Played",
            TableColumn::Liked => "Liked",
        }
    }
}

pub fn default_table_columns() -> Vec<TableColumn> {
    vec![
        TableColumn::TrackNumber,
        TableColumn::Title,
        TableColumn::Artist,
        TableColumn::Album,
        TableColumn::Plays,
        TableColumn::Liked,
    ]
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OmatunesDb {
    pub play_counts: HashMap<PathBuf, u32>,
    pub playlists: HashMap<String, Vec<PathBuf>>,
    #[serde(default)]
    pub recently_played: Vec<(PathBuf, String)>,
    #[serde(default)]
    pub hidden_artists_albums: Vec<(String, bool)>,
    #[serde(default = "default_table_columns")]
    pub table_columns: Vec<TableColumn>,
    #[serde(default)]
    pub hidden_columns: Vec<TableColumn>,
    #[serde(default)]
    pub group_by_album: bool,
    #[serde(default)]
    pub group_by: Option<GroupBy>,
    #[serde(default)]
    pub sidebar_width: Option<f32>,
    #[serde(default)]
    pub playlist_height: Option<f32>,
    #[serde(default)]
    pub right_panel_width: Option<f32>,
    #[serde(default)]
    pub right_panel_tab: Option<crate::app::RightPanelTab>,
    #[serde(default)]
    pub player_height: Option<f32>,
    #[serde(default)]
    pub last_view_mode: Option<crate::app::ViewMode>,
    #[serde(default)]
    pub last_selected_playlist: Option<String>,
    #[serde(default)]
    pub last_selected_folder: Option<PathBuf>,
    #[serde(default)]
    pub last_selected_artist: Option<String>,
    #[serde(default)]
    pub last_selected_album: Option<String>,
    #[serde(default)]
    pub last_selected_genre: Option<String>,
    #[serde(default)]
    pub last_track_path: Option<PathBuf>,
    #[serde(default)]
    pub last_queue_paths: Vec<PathBuf>,
    #[serde(default)]
    pub last_position_secs: u64,
    #[serde(default)]
    pub smart_playlists: HashMap<String, crate::library::smart_playlist::SmartPlaylist>,
    #[serde(default)]
    pub playlist_order: Vec<String>,
    #[serde(default)]
    pub smart_playlist_order: Vec<String>,
    #[serde(default)]
    pub smart_playlist_song_order: HashMap<String, Vec<PathBuf>>,
    #[serde(default)]
    pub auto_playlist_song_order: HashMap<String, Vec<PathBuf>>,
}

impl Default for OmatunesDb {
    fn default() -> Self {
        OmatunesDb {
            play_counts: HashMap::default(),
            playlists: HashMap::default(),
            recently_played: Vec::default(),
            hidden_artists_albums: Vec::default(),
            table_columns: default_table_columns(),
            group_by_album: false,
            group_by: Some(GroupBy::None),
            sidebar_width: None,
            playlist_height: None,
            right_panel_width: None,
            right_panel_tab: None,
            player_height: None,
            last_view_mode: None,
            last_selected_playlist: None,
            last_selected_folder: None,
            last_selected_artist: None,
            last_selected_album: None,
            last_selected_genre: None,
            last_track_path: None,
            last_queue_paths: Vec::default(),
            last_position_secs: 0,
            smart_playlists: HashMap::default(),
            playlist_order: Vec::default(),
            smart_playlist_order: Vec::default(),
            smart_playlist_song_order: HashMap::default(),
            auto_playlist_song_order: HashMap::default(),
        }
    }
}

impl OmatunesDb {
    pub fn load() -> Self {
        let path = crate::paths::db();
        if !path.exists() {
            return OmatunesDb::default();
        }
        let mut db: OmatunesDb = std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        if db.group_by.is_none() {
            if db.group_by_album {
                db.group_by = Some(GroupBy::Album);
            } else {
                db.group_by = Some(GroupBy::None);
            }
        }
        db
    }

    pub fn save(&self) {
        let path = crate::paths::db();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            std::fs::write(path, json).ok();
        }
    }
}

pub fn init() {
    let db = OmatunesDb::load();
    DB.get_or_init(|| Mutex::new(db));
}

pub fn get<F, R>(f: F) -> R
where
    F: FnOnce(&OmatunesDb) -> R,
{
    let guard = DB.get_or_init(|| Mutex::new(OmatunesDb::load())).lock().unwrap();
    f(&guard)
}

pub fn write<F, R>(f: F) -> R
where
    F: FnOnce(&mut OmatunesDb) -> R,
{
    let mut guard = DB.get_or_init(|| Mutex::new(OmatunesDb::load())).lock().unwrap();
    let res = f(&mut guard);
    DB_DIRTY.store(true, Ordering::Release);
    res
}

pub fn flush() {
    if DB_DIRTY.swap(false, Ordering::Acquire) {
        if let Ok(guard) = DB.get_or_init(|| Mutex::new(OmatunesDb::load())).lock() {
            guard.save();
        }
    }
}

pub fn increment_play_count(path: PathBuf) -> u32 {
    write(|db| {
        let count = db.play_counts.entry(path).or_insert(0);
        *count += 1;
        *count
    })
}

pub fn add_to_playlist(name: String, path: PathBuf) {
    write(|db| {
        let list = db.playlists.entry(name).or_default();
        if !list.contains(&path) {
            list.push(path);
        }
    });
}

pub fn remove_from_playlist(name: String, path: PathBuf) {
    write(|db| {
        if let Some(list) = db.playlists.get_mut(&name) {
            list.retain(|p| p != &path);
        }
    });
}

pub fn create_playlist(name: String) {
    write(|db| {
        db.playlists.entry(name.clone()).or_default();
        if !db.playlist_order.contains(&name) {
            db.playlist_order.push(name);
        }
    });
}

pub fn delete_playlist(name: String) {
    write(|db| {
        db.playlists.remove(&name);
        db.playlist_order.retain(|n| n != &name);
    });
}

pub fn rename_playlist(old_name: String, new_name: String) {
    write(|db| {
        if let Some(list) = db.playlists.remove(&old_name) {
            db.playlists.insert(new_name.clone(), list);
            if let Some(pos) = db.playlist_order.iter().position(|n| n == &old_name) {
                db.playlist_order[pos] = new_name;
            }
        }
    });
}

pub fn add_to_recently_played(path: PathBuf) {
    write(|db| {
        db.recently_played.retain(|(p, _)| p != &path);
        let now_str = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
        db.recently_played.insert(0, (path, now_str));
        if db.recently_played.len() > 100 {
            db.recently_played.truncate(100);
        }
    });
}

pub fn save_smart_playlist(name: String, playlist: crate::library::smart_playlist::SmartPlaylist) {
    write(|db| {
        db.smart_playlists.insert(name.clone(), playlist);
        if !db.smart_playlist_order.contains(&name) {
            db.smart_playlist_order.push(name);
        }
    });
}

pub fn delete_smart_playlist(name: String) {
    write(|db| {
        db.smart_playlists.remove(&name);
        db.smart_playlist_order.retain(|n| n != &name);
    });
}

