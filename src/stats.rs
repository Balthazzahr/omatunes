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
    #[serde(default)]
    pub genre_minutes: HashMap<String, f64>,
    #[serde(default)]
    pub longest_session_minutes: f64,
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
    #[serde(default)]
    pub last_active_timestamp: Option<i64>,
    #[serde(default)]
    pub current_session_accum_secs: u64,
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

// ── Aggregation & Query Functions ─────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct PeriodStats {
    pub today_minutes: f64,
    pub yesterday_minutes: f64,
    pub this_week_minutes: f64,
    pub last_week_minutes: f64,
    pub this_month_minutes: f64,
    pub last_month_minutes: f64,
    pub this_year_minutes: f64,
    pub last_year_minutes: f64,
    pub all_time_minutes: f64,
}

#[derive(Debug, Clone, Default)]
pub struct StreakStats {
    pub current_streak: u32,
    pub longest_streak: u32,
}

#[derive(Debug, Clone, Default)]
pub struct UniqueStats {
    pub unique_tracks: usize,
    pub unique_artists: usize,
    pub unique_albums: usize,
}

fn parse_date(s: &str) -> Option<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

pub fn get_period_stats() -> PeriodStats {
    get(|db| {
        let now = chrono::Local::now().naive_local().date();
        let today_str = now.format("%Y-%m-%d").to_string();
        
        let yesterday = now - chrono::Duration::days(1);
        let yesterday_str = yesterday.format("%Y-%m-%d").to_string();
        
        // Weeks start on Monday
        let days_from_monday = now.weekday().num_days_from_monday();
        let monday = now - chrono::Duration::days(days_from_monday as i64);
        let prev_sunday = monday - chrono::Duration::days(1);
        let prev_monday = prev_sunday - chrono::Duration::days(6);
        
        let this_month_prefix = now.format("%Y-%m").to_string();
        let prev_month = if now.month() == 1 {
            format!("{}-12", now.year() - 1)
        } else {
            format!("{}-{:02}", now.year(), now.month() - 1)
        };
        
        let this_year_prefix = now.format("%Y").to_string();
        let prev_year_prefix = (now.year() - 1).to_string();
        
        let mut stats = PeriodStats::default();
        stats.all_time_minutes = db.legacy_minutes;

        for (date_str, day) in &db.daily_buckets {
            stats.all_time_minutes += day.total_minutes;
            
            if date_str == &today_str {
                stats.today_minutes += day.total_minutes;
            }
            if date_str == &yesterday_str {
                stats.yesterday_minutes += day.total_minutes;
            }
            
            if let Some(d) = parse_date(date_str) {
                if d >= monday && d <= now {
                    stats.this_week_minutes += day.total_minutes;
                }
                if d >= prev_monday && d <= prev_sunday {
                    stats.last_week_minutes += day.total_minutes;
                }
            }
            
            if date_str.starts_with(&this_month_prefix) {
                stats.this_month_minutes += day.total_minutes;
            }
            if date_str.starts_with(&prev_month) {
                stats.last_month_minutes += day.total_minutes;
            }
            
            if date_str.starts_with(&this_year_prefix) {
                stats.this_year_minutes += day.total_minutes;
            }
            if date_str.starts_with(&prev_year_prefix) {
                stats.last_year_minutes += day.total_minutes;
            }
        }
        
        stats
    })
}

use chrono::Datelike;

pub fn get_streak_stats() -> StreakStats {
    get(|db| {
        let mut active_dates: Vec<chrono::NaiveDate> = db.daily_buckets.iter()
            .filter(|(_, day)| day.total_minutes > 0.0)
            .filter_map(|(date_str, _)| parse_date(date_str))
            .collect();
            
        if active_dates.is_empty() {
            return StreakStats::default();
        }
        
        active_dates.sort();
        active_dates.dedup();
        
        let now = chrono::Local::now().naive_local().date();
        let yesterday = now - chrono::Duration::days(1);
        
        // Compute current streak
        let mut current_streak = 0;
        let mut has_today_or_yesterday = active_dates.contains(&now) || active_dates.contains(&yesterday);
        
        if has_today_or_yesterday {
            let mut check_date = if active_dates.contains(&now) { now } else { yesterday };
            while active_dates.contains(&check_date) {
                current_streak += 1;
                check_date -= chrono::Duration::days(1);
            }
        }
        
        // Compute longest streak
        let mut longest_streak = 0;
        let mut current_run = 0;
        let mut prev_date: Option<chrono::NaiveDate> = None;
        
        for date in active_dates {
            match prev_date {
                Some(prev) => {
                    if date == prev + chrono::Duration::days(1) {
                        current_run += 1;
                    } else if date != prev {
                        current_run = 1;
                    }
                }
                None => {
                    current_run = 1;
                }
            }
            if current_run > longest_streak {
                longest_streak = current_run;
            }
            prev_date = Some(date);
        }
        
        StreakStats {
            current_streak,
            longest_streak,
        }
    })
}

pub fn get_unique_stats(all_tracks: &[crate::library::models::Track]) -> UniqueStats {
    get(|db| {
        let mut tracks_played = std::collections::HashSet::new();
        for (_, day) in &db.daily_buckets {
            for path in day.track_play_counts.keys() {
                tracks_played.insert(path.clone());
            }
        }
        
        let unique_tracks_count = tracks_played.len() + db.legacy_tracks as usize;
        
        // Deduplicate artists and albums from played track paths
        let mut artists = std::collections::HashSet::new();
        let mut albums = std::collections::HashSet::new();
        
        // Load legacy ones
        for artist in db.legacy_artist_minutes.keys() {
            artists.insert(artist.clone());
        }
        
        for path in &tracks_played {
            if let Some(track) = all_tracks.iter().find(|t| &t.path == path) {
                if !track.artist.trim().is_empty() {
                    artists.insert(track.artist.clone());
                }
                if !track.album.trim().is_empty() {
                    albums.insert(track.album.clone());
                }
            } else {
                // Fallback to parent dir/filename parsing if missing from current library
                if let Some(parent) = path.parent() {
                    if let Some(album_name) = parent.file_name().and_then(|f| f.to_str()) {
                        albums.insert(album_name.to_string());
                    }
                }
            }
        }
        
        UniqueStats {
            unique_tracks: unique_tracks_count,
            unique_artists: artists.len(),
            unique_albums: albums.len(),
        }
    })
}

pub fn get_monthly_leaderboards() -> (Vec<(String, f64)>, Vec<(String, u32)>) {
    get(|db| {
        let now = chrono::Local::now().naive_local().date();
        let this_month_prefix = now.format("%Y-%m").to_string();
        
        let mut artists_minutes: HashMap<String, f64> = HashMap::new();
        let mut artists_tracks: HashMap<String, u32> = HashMap::new();
        
        for (date_str, day) in &db.daily_buckets {
            if date_str.starts_with(&this_month_prefix) {
                for (artist, mins) in &day.artist_minutes {
                    *artists_minutes.entry(artist.clone()).or_default() += mins;
                }
                for (artist, count) in &day.artist_track_counts {
                    *artists_tracks.entry(artist.clone()).or_default() += count;
                }
            }
        }
        
        let mut top_minutes: Vec<(String, f64)> = artists_minutes.into_iter().collect();
        top_minutes.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        top_minutes.truncate(5);
        
        let mut top_tracks: Vec<(String, u32)> = artists_tracks.into_iter().collect();
        top_tracks.sort_by(|a, b| b.1.cmp(&a.1));
        top_tracks.truncate(5);
        
        (top_minutes, top_tracks)
    })
}

pub fn get_all_time_leaderboards() -> (Vec<(String, f64)>, Vec<(String, u32)>) {
    get(|db| {
        let mut artists_minutes: HashMap<String, f64> = db.legacy_artist_minutes.clone();
        let mut artists_tracks: HashMap<String, u32> = db.legacy_artist_tracks.clone();
        
        for (_, day) in &db.daily_buckets {
            for (artist, mins) in &day.artist_minutes {
                *artists_minutes.entry(artist.clone()).or_default() += mins;
            }
            for (artist, count) in &day.artist_track_counts {
                *artists_tracks.entry(artist.clone()).or_default() += count;
            }
        }
        
        let mut top_minutes: Vec<(String, f64)> = artists_minutes.into_iter().collect();
        top_minutes.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        top_minutes.truncate(10);
        
        let mut top_tracks: Vec<(String, u32)> = artists_tracks.into_iter().collect();
        top_tracks.sort_by(|a, b| b.1.cmp(&a.1));
        top_tracks.truncate(10);
        
        (top_minutes, top_tracks)
    })
}
