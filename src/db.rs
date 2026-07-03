use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

static DB: std::sync::OnceLock<Mutex<OmatunesDb>> = std::sync::OnceLock::new();

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

fn default_table_columns() -> Vec<TableColumn> {
    vec![
        TableColumn::TrackNumber,
        TableColumn::Title,
        TableColumn::Artist,
        TableColumn::Album,
        TableColumn::Duration,
        TableColumn::Plays,
        TableColumn::Liked,
    ]
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OmatunesDb {
    pub favorites: HashSet<PathBuf>,
    pub play_counts: HashMap<PathBuf, u32>,
    pub playlists: HashMap<String, Vec<PathBuf>>,
    #[serde(default)]
    pub recently_played: Vec<(PathBuf, String)>,
    #[serde(default)]
    pub hidden_artists_albums: Vec<(String, bool)>,
    #[serde(default = "default_table_columns")]
    pub table_columns: Vec<TableColumn>,
    #[serde(default)]
    pub group_by_album: bool,
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
}

impl Default for OmatunesDb {
    fn default() -> Self {
        OmatunesDb {
            favorites: HashSet::default(),
            play_counts: HashMap::default(),
            playlists: HashMap::default(),
            recently_played: Vec::default(),
            hidden_artists_albums: Vec::default(),
            table_columns: default_table_columns(),
            group_by_album: false,
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
        }
    }
}
            recently_played: Vec::default(),
            hidden_artists_albums: Vec::default(),
            table_columns: default_table_columns(),
            group_by_album: false,
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
        }
    }
}

fn db_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(home).join(".config/omatunes/db.json")
}

impl OmatunesDb {
    pub fn load() -> Self {
        let path = db_path();
        if !path.exists() {
            return OmatunesDb::default();
        }
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        let path = db_path();
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
    guard.save();
    res
}

pub fn increment_play_count(path: PathBuf) -> u32 {
    write(|db| {
        let count = db.play_counts.entry(path).or_insert(0);
        *count += 1;
        *count
    })
}

pub fn toggle_favorite(path: PathBuf) -> bool {
    write(|db| {
        if db.favorites.contains(&path) {
            db.favorites.remove(&path);
            false
        } else {
            db.favorites.insert(path);
            true
        }
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

pub fn create_playlist(name: String) {
    write(|db| {
        db.playlists.entry(name).or_default();
    });
}

pub fn delete_playlist(name: String) {
    write(|db| {
        db.playlists.remove(&name);
    });
}

pub fn rename_playlist(old_name: String, new_name: String) {
    write(|db| {
        if let Some(list) = db.playlists.remove(&old_name) {
            db.playlists.insert(new_name, list);
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
        db.smart_playlists.insert(name, playlist);
    });
}

pub fn delete_smart_playlist(name: String) {
    write(|db| {
        db.smart_playlists.remove(&name);
    });
}

