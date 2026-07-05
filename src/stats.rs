use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use serde::{Serialize, Deserialize};

static STATS: std::sync::OnceLock<Mutex<StatsDb>> = std::sync::OnceLock::new();

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DayStats {
    pub total_minutes: f64,
    pub track_play_count: u32,
    pub artist_minutes: HashMap<String, f64>,
    pub artist_track_counts: HashMap<String, u32>,
    pub track_play_counts: HashMap<PathBuf, u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StatsDb {
    pub daily_buckets: HashMap<String, DayStats>, // Key: "YYYY-MM-DD"
    // Legacy counters from waybar_omatunes_session.pkl before day-bucket tracking
    #[serde(default)]
    pub legacy_tracks: u32,
    #[serde(default)]
    pub legacy_minutes: f64,
    #[serde(default)]
    pub legacy_artist_minutes: HashMap<String, f64>,
    #[serde(default)]
    pub legacy_artist_tracks: HashMap<String, u32>,
}

fn stats_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(home).join(".config/omatunes/stats.json")
}

impl StatsDb {
    pub fn load() -> Self {
        let path = stats_path();
        if !path.exists() {
            return StatsDb::default();
        }
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        let path = stats_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            std::fs::write(path, json).ok();
        }
    }
}

pub fn init() {
    let db = StatsDb::load();
    STATS.get_or_init(|| Mutex::new(db));
}

pub fn get<F, R>(f: F) -> R
where
    F: FnOnce(&StatsDb) -> R,
{
    let guard = STATS.get_or_init(|| Mutex::new(StatsDb::load())).lock().unwrap();
    f(&guard)
}

pub fn write<F, R>(f: F) -> R
where
    F: FnOnce(&mut StatsDb) -> R,
{
    let mut guard = STATS.get_or_init(|| Mutex::new(StatsDb::load())).lock().unwrap();
    let res = f(&mut guard);
    guard.save();
    res
}

// ── Accumulation Helper ───────────────────────────────────────────────────────

pub fn add_playback_time(artist: &str, track_path: PathBuf, secs: f64) {
    let date_str = chrono::Local::now().format("%Y-%m-%d").to_string();
    let minutes = secs / 60.0;
    
    write(|db| {
        let day = db.daily_buckets.entry(date_str).or_default();
        day.total_minutes += minutes;
        
        let artist_entry = day.artist_minutes.entry(artist.to_string()).or_default();
        *artist_entry += minutes;
    });
}

pub fn add_track_play(artist: &str, track_path: PathBuf) {
    let date_str = chrono::Local::now().format("%Y-%m-%d").to_string();
    
    write(|db| {
        let day = db.daily_buckets.entry(date_str).or_default();
        day.track_play_count += 1;
        
        let artist_count = day.artist_track_counts.entry(artist.to_string()).or_default();
        *artist_count += 1;
        
        let track_count = day.track_play_counts.entry(track_path).or_default();
        *track_count += 1;
    });
}
