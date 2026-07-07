use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use serde::{Serialize, Deserialize};

static STATS: std::sync::OnceLock<Mutex<StatsDb>> = std::sync::OnceLock::new();
static STATS_DIRTY: AtomicBool = AtomicBool::new(false);

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
    pub album_minutes: HashMap<String, f64>,
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
    pub current_session_accum_secs: f64,
}

impl StatsDb {
    pub fn load() -> Self {
        let path = crate::paths::stats();
        if !path.exists() {
            return StatsDb::default();
        }
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        let path = crate::paths::stats();
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
    STATS_DIRTY.store(true, Ordering::Release);
    res
}

pub fn flush() {
    if STATS_DIRTY.swap(false, Ordering::Acquire) {
        if let Ok(guard) = STATS.get_or_init(|| Mutex::new(StatsDb::load())).lock() {
            guard.save();
        }
    }
}

// ── Accumulation Helper ───────────────────────────────────────────────────────

pub fn add_playback_time(artist: &str, genre: &str, secs: f64) {
    let now_dt = chrono::Local::now();
    let date_str = now_dt.format("%Y-%m-%d").to_string();
    let now_ts = now_dt.timestamp();
    let minutes = secs / 60.0;
    
    write(|db| {
        // Handle Session Closing Check (30 minutes = 1800 seconds)
        if let Some(last_ts) = db.last_active_timestamp {
            if now_ts - last_ts > 1800 {
                db.current_session_accum_secs = 0.0;
            }
        } else {
            db.current_session_accum_secs = 0.0;
        }
        
        db.current_session_accum_secs += secs;
        db.last_active_timestamp = Some(now_ts);
        
        let day = db.daily_buckets.entry(date_str).or_default();
        day.total_minutes += minutes;
        
        let artist_entry = day.artist_minutes.entry(artist.to_string()).or_default();
        *artist_entry += minutes;
        
        let clean_genre = if genre.trim().is_empty() { "Unknown" } else { genre };
        let genre_entry = day.genre_minutes.entry(clean_genre.to_string()).or_default();
        *genre_entry += minutes;
        
        let session_mins = db.current_session_accum_secs / 60.0;
        if session_mins > day.longest_session_minutes {
            day.longest_session_minutes = session_mins;
        }
    });
}

pub fn add_track_play(artist: &str, track_path: PathBuf) {
    let date_str = chrono::Local::now().format("%Y-%m-%d").to_string();
    
    write(|db| {
        let day = db.daily_buckets.entry(date_str).or_default();
        day.track_play_count += 1;
        
        let artist_count = day.artist_track_counts.entry(artist.to_string()).or_default();
        *artist_count += 1;
        
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

pub fn get_combined_monthly_leaderboard() -> Vec<(String, f64, u32)> {
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
        
        let mut combined: Vec<(String, f64, u32)> = artists_minutes.into_iter().map(|(name, mins)| {
            let count = artists_tracks.get(&name).cloned().unwrap_or(0);
            (name, mins, count)
        }).collect();
        
        combined.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        combined.truncate(5);
        combined
    })
}

pub fn get_combined_all_time_leaderboard() -> Vec<(String, f64, u32)> {
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
        
        let mut combined: Vec<(String, f64, u32)> = artists_minutes.into_iter().map(|(name, mins)| {
            let count = artists_tracks.get(&name).cloned().unwrap_or(0);
            (name, mins, count)
        }).collect();
        
        combined.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        combined.truncate(10);
        combined
    })
}

#[derive(Debug, Clone)]
pub struct RowStats {
    pub period_label: String,
    pub songs: u32,
    pub minutes: f64,
    pub top_genre: String,
    pub top_artist: String,
    pub longest_session: f64,
}

#[derive(Debug, Clone)]
pub struct PeriodBreakdown {
    pub period_label: String,
    pub total_minutes: f64,
    pub total_plays: u32,
    pub artist_minutes: Vec<(String, f64)>,
    pub genre_minutes: Vec<(String, f64)>,
}

pub fn get_period_breakdown(period_idx: usize, tracks: &[crate::library::models::Track]) -> PeriodBreakdown {
    get(|db| {
        let now = chrono::Local::now().naive_local().date();
        let today_str = now.format("%Y-%m-%d").to_string();
        let days_from_monday = now.weekday().num_days_from_monday();
        let monday = now - chrono::Duration::days(days_from_monday as i64);
        let this_month_prefix = now.format("%Y-%m").to_string();

        let periods: Vec<(&str, Box<dyn Fn(&str) -> bool>)> = vec![
            ("Today", Box::new(move |d: &str| d == today_str)),
            ("This Week", Box::new(move |d: &str| {
                if let Some(date) = parse_date(d) {
                    date >= monday && date <= now
                } else { false }
            })),
            ("This Month", Box::new(move |d: &str| d.starts_with(&this_month_prefix))),
            ("All-Time", Box::new(move |_d: &str| true)),
        ];

        let (label, filter) = &periods[period_idx];
        let mut total_minutes = 0.0;
        let mut total_plays = 0;
        let mut artist_minutes: HashMap<String, f64> = HashMap::new();
        let mut genre_minutes: HashMap<String, f64> = HashMap::new();

        if *label == "All-Time" {
            total_plays += db.legacy_tracks;
            total_minutes += db.legacy_minutes;
            for (a, m) in &db.legacy_artist_minutes {
                *artist_minutes.entry(a.clone()).or_default() += m;
            }
        }

        for (date_str, day) in &db.daily_buckets {
            if filter(date_str) {
                total_plays += day.track_play_count;
                total_minutes += day.total_minutes;
                for (a, m) in &day.artist_minutes {
                    *artist_minutes.entry(a.clone()).or_default() += m;
                }
                if day.genre_minutes.is_empty() {
                    // Build artist_to_genre mapping as fallback
                    let mut artist_genre_counts: HashMap<String, HashMap<String, usize>> = HashMap::new();
                    for track in tracks {
                        if !track.artist.is_empty() && !track.genre.is_empty() {
                            *artist_genre_counts
                                .entry(track.artist.clone())
                                .or_default()
                                .entry(track.genre.clone())
                                .or_default() += 1;
                        }
                    }
                    for (a, m) in &day.artist_minutes {
                        if let Some(best_genre) = artist_genre_counts.get(a)
                            .and_then(|gmap| gmap.iter().max_by_key(|(_, count)| **count).map(|(g, _)| g))
                        {
                            *genre_minutes.entry(best_genre.clone()).or_default() += m;
                        }
                    }
                } else {
                    for (g, m) in &day.genre_minutes {
                        *genre_minutes.entry(g.clone()).or_default() += m;
                    }
                }
            }
        }

        let mut artist_list: Vec<(String, f64)> = artist_minutes.into_iter().collect();
        artist_list.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let mut genre_list: Vec<(String, f64)> = genre_minutes.into_iter().collect();
        genre_list.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        PeriodBreakdown {
            period_label: label.to_string(),
            total_minutes,
            total_plays,
            artist_minutes: artist_list,
            genre_minutes: genre_list,
        }
    })
}

pub fn get_restructured_stats(tracks: &[crate::library::models::Track]) -> Vec<RowStats> {
    get(|db| {
        let now = chrono::Local::now().naive_local().date();
        let today_str = now.format("%Y-%m-%d").to_string();
        
        let days_from_monday = now.weekday().num_days_from_monday();
        let monday = now - chrono::Duration::days(days_from_monday as i64);
        
        let this_month_prefix = now.format("%Y-%m").to_string();
        
        // Build artist to genre mapping from the library tracks
        let mut artist_genre_counts: HashMap<String, HashMap<String, usize>> = HashMap::new();
        for track in tracks {
            if !track.artist.is_empty() && !track.genre.is_empty() {
                *artist_genre_counts
                    .entry(track.artist.clone())
                    .or_default()
                    .entry(track.genre.clone())
                    .or_default() += 1;
            }
        }
        
        let mut artist_to_genre: HashMap<String, String> = HashMap::new();
        for (artist, genres_map) in artist_genre_counts {
            if let Some(best_genre) = genres_map.into_iter()
                .max_by_key(|(_, count)| *count)
                .map(|(genre, _)| genre) {
                artist_to_genre.insert(artist, best_genre);
            }
        }
        
        let periods = vec![
            ("Today".to_string(), Box::new(move |d: &str| d == today_str) as Box<dyn Fn(&str) -> bool>),
            ("This Week".to_string(), Box::new(move |d: &str| {
                if let Some(date) = parse_date(d) {
                    date >= monday && date <= now
                } else {
                    false
                }
            })),
            ("This Month".to_string(), Box::new(move |d: &str| d.starts_with(&this_month_prefix))),
            ("All-Time".to_string(), Box::new(move |_d: &str| true)),
        ];
        
        let mut rows = Vec::new();
        
        for (label, filter) in periods {
            let mut songs = 0;
            let mut minutes = 0.0;
            let mut longest_session = 0.0;
            let mut artists: HashMap<String, f64> = HashMap::new();
            let mut genres: HashMap<String, f64> = HashMap::new();
            
            if label == "All-Time" {
                songs += db.legacy_tracks;
                minutes += db.legacy_minutes;
                for (a, m) in &db.legacy_artist_minutes {
                    *artists.entry(a.clone()).or_default() += m;
                    if let Some(g) = artist_to_genre.get(a) {
                        *genres.entry(g.clone()).or_default() += m;
                    }
                }
            }
            
            for (date_str, day) in &db.daily_buckets {
                if filter(date_str) {
                    songs += day.track_play_count;
                    minutes += day.total_minutes;
                    if day.longest_session_minutes > longest_session {
                        longest_session = day.longest_session_minutes;
                    }
                    for (a, m) in &day.artist_minutes {
                        *artists.entry(a.clone()).or_default() += m;
                    }
                    if day.genre_minutes.is_empty() {
                        for (a, m) in &day.artist_minutes {
                            if let Some(g) = artist_to_genre.get(a) {
                                *genres.entry(g.clone()).or_default() += m;
                            }
                        }
                    } else {
                        for (g, m) in &day.genre_minutes {
                            *genres.entry(g.clone()).or_default() += m;
                        }
                    }
                }
            }
            
            let top_artist = artists.into_iter()
                .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(name, _)| name)
                .unwrap_or_else(|| "-".to_string());
                
            let top_genre = genres.into_iter()
                .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(name, _)| name)
                .unwrap_or_else(|| "-".to_string());
                
            rows.push(RowStats {
                period_label: label,
                songs,
                minutes,
                top_genre,
                top_artist,
                longest_session,
            });
        }
        
        rows
    })
}
