use std::collections::{HashMap, HashSet, BTreeMap};
use std::path::PathBuf;
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
    #[serde(default)]
    pub genre_track_counts: HashMap<String, u32>,
    #[serde(default)]
    pub album_track_counts: HashMap<String, u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct EarnedAchievement {
    pub entity_type: String,     // "Artist", "Album", or "Genre"
    pub entity_name: String,
    pub period: String,          // "Daily", "Weekly", "Monthly", "Yearly", "All-Time"
    pub tier: String,            // "Bronze", "Silver", "Gold", "Platinum", "Legendary"
    pub date_earned: String,     // "YYYY-MM-DD"
}

pub type YearlyStats = DayStats;

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

    // Running all-time artist/genre play counts (updated on every track play)
    #[serde(default)]
    pub all_time_artist_tracks: HashMap<String, u32>,
    #[serde(default)]
    pub all_time_genre_tracks: HashMap<String, u32>,

    // Awarded milestones to avoid re-awarding
    #[serde(default)]
    pub artist_milestones_awarded: HashMap<String, HashSet<String>>,
    #[serde(default)]
    pub genre_milestones_awarded: HashMap<String, HashSet<String>>,
    #[serde(default)]
    pub daily_milestones_awarded: HashMap<String, HashSet<u32>>,

    // Top-10 snapshot for ladder change detection
    #[serde(default)]
    pub previous_top_10_snapshot: Vec<(String, f64)>,

    // One-time backfill: legacy album data split evenly across albums
    #[serde(default)]
    pub legacy_albums_populated_v2: bool,
    #[serde(default)]
    pub legacy_album_minutes: HashMap<String, f64>,
    #[serde(default)]
    pub legacy_album_tracks: HashMap<String, u32>,

    // One-time backfill: infer historical track-level play counts
    #[serde(default)]
    pub legacy_track_counts_populated: bool,

    // New Achievement System and Yearly buckets
    #[serde(default)]
    pub yearly_buckets: HashMap<u32, YearlyStats>,
    #[serde(default)]
    pub earned_achievements: Vec<EarnedAchievement>,
    #[serde(default)]
    pub legacy_genre_minutes: HashMap<String, f64>,
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
    prune_old_buckets();
}

pub fn prune_old_buckets() {
    write(|db| {
        let cutoff = chrono::Local::now().naive_local().date() - chrono::Duration::days(35);
        let mut days_to_prune = Vec::new();

        for (date_str, day) in &db.daily_buckets {
            if let Some(date) = parse_date(date_str) {
                if date < cutoff {
                    if !day.artist_minutes.is_empty() || !day.track_play_counts.is_empty() {
                        days_to_prune.push(date_str.clone());
                    }
                }
            }
        }

        if days_to_prune.is_empty() {
            return;
        }

        for date_str in days_to_prune {
            if let Some(mut day) = db.daily_buckets.remove(&date_str) {
                db.legacy_tracks += day.track_play_count;
                db.legacy_minutes += day.total_minutes;

                for (artist, mins) in &day.artist_minutes {
                    *db.legacy_artist_minutes.entry(artist.clone()).or_default() += mins;
                }
                for (artist, count) in &day.artist_track_counts {
                    *db.legacy_artist_tracks.entry(artist.clone()).or_default() += count;
                }
                for (album, mins) in &day.album_minutes {
                    *db.legacy_album_minutes.entry(album.clone()).or_default() += mins;
                }
                for (album, count) in &day.album_track_counts {
                    *db.legacy_album_tracks.entry(album.clone()).or_default() += count;
                }

                // Clear maps to reclaim disk/memory footprint
                day.artist_minutes.clear();
                day.artist_track_counts.clear();
                day.track_play_counts.clear();
                day.genre_minutes.clear();
                day.album_minutes.clear();
                day.genre_track_counts.clear();
                day.album_track_counts.clear();

                // Re-insert bucket container with only overall day-minutes/plays to preserve streak logic
                db.daily_buckets.insert(date_str, day);
            }
        }
    });
    flush();
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

pub fn add_playback_time(artist: &str, album: &str, genre: &str, secs: f64) {
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
        
        if genre.contains("; ") {
            for g in genre.split("; ") {
                let clean = if g.trim().is_empty() { "Unknown" } else { g.trim() };
                let genre_entry = day.genre_minutes.entry(clean.to_string()).or_default();
                *genre_entry += minutes;
            }
        } else {
            let clean_genre = if genre.trim().is_empty() { "Unknown" } else { genre.trim() };
            let genre_entry = day.genre_minutes.entry(clean_genre.to_string()).or_default();
            *genre_entry += minutes;
        }

        let clean_album = if album.trim().is_empty() { "Unknown" } else { album.trim() };
        let album_entry = day.album_minutes.entry(clean_album.to_string()).or_default();
        *album_entry += minutes;

        let session_mins = db.current_session_accum_secs / 60.0;
        if session_mins > day.longest_session_minutes {
            day.longest_session_minutes = session_mins;
        }
    });
}

// ── One-Time Album Data Backfill ────────────────────────────────────────────

pub fn backfill_album_data(tracks: &[crate::library::models::Track]) {
    let mut artist_albums: HashMap<String, Vec<String>> = HashMap::new();
    for track in tracks {
        if !track.artist.is_empty() && !track.album.is_empty() {
            let clean = if track.album.trim().is_empty() { "Unknown".to_string() } else { track.album.trim().to_string() };
            let albums = artist_albums.entry(track.artist.clone()).or_default();
            if !albums.contains(&clean) {
                albums.push(clean);
            }
        }
    }

    write(|db| {
        if db.legacy_albums_populated_v2 {
            return;
        }

        // Backfill legacy artist minutes → split evenly across albums
        for (artist, mins) in &db.legacy_artist_minutes.clone() {
            if let Some(albums) = artist_albums.get(artist) {
                if !albums.is_empty() {
                    let per_album = mins / albums.len() as f64;
                    for album in albums {
                        *db.legacy_album_minutes.entry(album.clone()).or_default() += per_album;
                    }
                }
            }
        }

        // Backfill legacy artist tracks → split evenly across albums
        for (artist, count) in &db.legacy_artist_tracks.clone() {
            if let Some(albums) = artist_albums.get(artist) {
                if !albums.is_empty() {
                    let per_album = *count as f64 / albums.len() as f64;
                    for album in albums {
                        *db.legacy_album_tracks.entry(album.clone()).or_default() += per_album.round() as u32;
                    }
                }
            }
        }

        // Backfill day buckets: handle album_minutes and album_track_counts separately
        for (_, day) in &mut db.daily_buckets {
            if day.album_minutes.is_empty() {
                for (artist, mins) in &day.artist_minutes.clone() {
                    if let Some(albums) = artist_albums.get(artist) {
                        if !albums.is_empty() {
                            let per_album = mins / albums.len() as f64;
                            for album in albums {
                                *day.album_minutes.entry(album.clone()).or_default() += per_album;
                            }
                        }
                    }
                }
            }
            if day.album_track_counts.is_empty() {
                for (artist, count) in &day.artist_track_counts.clone() {
                    if let Some(albums) = artist_albums.get(artist) {
                        if !albums.is_empty() {
                            let per_album = *count as f64 / albums.len() as f64;
                            for album in albums {
                                *day.album_track_counts.entry(album.clone()).or_default() += per_album.round() as u32;
                            }
                        }
                    }
                }
            }
        }

        db.legacy_albums_populated_v2 = true;

        // One-time backfill: infer historical track_play_counts from artist_track_counts
        if !db.legacy_track_counts_populated {
            // Build artist→tracks mapping from library
            let mut artist_tracks_map: HashMap<String, Vec<std::path::PathBuf>> = HashMap::new();
            for track in tracks {
                if !track.artist.is_empty() {
                    artist_tracks_map
                        .entry(track.artist.clone())
                        .or_default()
                        .push(track.path.clone());
                }
            }

            // Backfill day buckets
            for (_, day) in &mut db.daily_buckets {
                if day.track_play_counts.is_empty() && !day.artist_track_counts.is_empty() {
                    for (artist, count) in &day.artist_track_counts.clone() {
                        if *count > 0 {
                            if let Some(track_paths) = artist_tracks_map.get(artist) {
                                if !track_paths.is_empty() {
                                    let per_track = *count / track_paths.len() as u32;
                                    let remainder = *count % track_paths.len() as u32;
                                    for (j, path) in track_paths.iter().enumerate() {
                                        let this_track = if (j as u32) < remainder { per_track + 1 } else { per_track };
                                        if this_track > 0 {
                                            *day.track_play_counts.entry(path.clone()).or_default() += this_track;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            db.legacy_track_counts_populated = true;
        }
    });
    flush();
}

pub fn on_track_play(
    artist: &str,
    genre: &str,
    album: &str,
    track_path: PathBuf,
    all_tracks: &[crate::library::models::Track],
) -> Vec<(String, String)> {
    let mut toasts: Vec<(String, String)> = Vec::new();
    let date_str = chrono::Local::now().format("%Y-%m-%d").to_string();

    write(|db| {
        // 1. Daily bucket
        let day = db.daily_buckets.entry(date_str.clone()).or_default();
        day.track_play_count += 1;
        *day.artist_track_counts.entry(artist.to_string()).or_default() += 1;
        *day.track_play_counts.entry(track_path).or_default() += 1;

        // 2. Running all-time artist count
        let artist_total = {
            let c = db.all_time_artist_tracks.entry(artist.to_string()).or_default();
            *c += 1;
            *c
        };

        // 3. Running all-time genre counts + daily genre/album track counts
        let genre_names: Vec<String> = if genre.contains("; ") {
            genre.split("; ").map(|g| {
                let clean = g.trim();
                if clean.is_empty() { "Unknown".to_string() } else { clean.to_string() }
            }).collect()
        } else {
            let clean = genre.trim();
            vec![if clean.is_empty() { "Unknown".to_string() } else { clean.to_string() }]
        };
        let mut genre_totals: Vec<(String, u32)> = Vec::new();
        for gn in &genre_names {
            let c = db.all_time_genre_tracks.entry(gn.clone()).or_default();
            *c += 1;
            genre_totals.push((gn.clone(), *c));
            *day.genre_track_counts.entry(gn.clone()).or_default() += 1;
        }
        let clean_album_track = album.trim();
        if !clean_album_track.is_empty() {
            *day.album_track_counts.entry(clean_album_track.to_string()).or_default() += 1;
        }

        // 4. Daily milestone check
        let daily_awarded = db.daily_milestones_awarded.entry(date_str).or_default();
        for &thresh in &[25u32, 50, 100] {
            if day.track_play_count == thresh && !daily_awarded.contains(&thresh) {
                daily_awarded.insert(thresh);
                let (title_suffix, icon) = match thresh {
                    25 => ("Bronze", "\u{f025}"),
                    50 => ("Silver", "\u{f005}"),
                    100 => ("Gold", "\u{f091}"),
                    _ => unreachable!(),
                };
                toasts.push((
                    format!("{} Milestone", title_suffix),
                    format!("You have listened to {} songs today! {}", thresh, icon),
                ));
            }
        }

        // 5. Artist milestone check
        let artist_awarded = db.artist_milestones_awarded.entry(artist.to_string()).or_default();
        for &thresh in &[50u32, 100, 500, 1000] {
            if artist_total == thresh {
                let key = thresh.to_string();
                if !artist_awarded.contains(&key) {
                    artist_awarded.insert(key);
                    let (tier, icon) = match thresh {
                        50 => ("Bronze", "\u{f025}"),
                        100 => ("Silver", "\u{f005}"),
                        500 => ("Gold", "\u{f091}"),
                        1000 => ("Platinum", "\u{f053f}"),
                        _ => unreachable!(),
                    };
                    toasts.push((
                        format!("{} Milestone \u{2013} {}", tier, artist),
                        format!("{} has been played {} times! {}", artist, thresh, icon),
                    ));
                }
            }
        }

        // 6. Genre milestone check
        for (gn, total) in &genre_totals {
            let genre_awarded = db.genre_milestones_awarded.entry(gn.clone()).or_default();
            for &thresh in &[50u32, 100, 500, 1000] {
                if *total == thresh {
                    let key = thresh.to_string();
                    if !genre_awarded.contains(&key) {
                        genre_awarded.insert(key);
                        let (tier, icon) = match thresh {
                            50 => ("Bronze", "\u{f025}"),
                            100 => ("Silver", "\u{f005}"),
                            500 => ("Gold", "\u{f091}"),
                            1000 => ("Platinum", "\u{f053f}"),
                            _ => unreachable!(),
                        };
                        toasts.push((
                            format!("{} Milestone \u{2013} {}", tier, gn),
                            format!("{} genre has been played {} times! {}", gn, thresh, icon),
                        ));
                    }
                }
            }
        }

        // 7. Top-10 ladder change check
        // Build current all-time leaderboard by minutes
        let mut all_artist_minutes: HashMap<String, f64> = db.legacy_artist_minutes.clone();
        for (_, day) in &db.daily_buckets {
            for (a, m) in &day.artist_minutes {
                *all_artist_minutes.entry(a.clone()).or_default() += m;
            }
        }
        let mut ranked: Vec<(String, f64)> = all_artist_minutes.into_iter().collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        ranked.truncate(10);

        let prev = &db.previous_top_10_snapshot;
        if !prev.is_empty() && prev != &ranked {
            let prev_artists: Vec<&str> = prev.iter().map(|(a, _)| a.as_str()).collect();
            let cur_artists: Vec<&str> = ranked.iter().map(|(a, _)| a.as_str()).collect();

            for (i, artist_name) in cur_artists.iter().enumerate() {
                if i >= prev_artists.len() {
                    // New entry beyond previous top 10's length - shouldn't happen since both are length 10
                    break;
                }
                if *artist_name != prev_artists[i] {
                    // Found first position where they differ
                    let new_pos = i + 1;
                    let displaced = prev_artists[i];

                    // Find where the mover was before
                    let old_pos = prev_artists.iter().position(|&a| a == *artist_name)
                        .map(|p| p + 1);

                    // Find where the displaced went
                    let displaced_new_pos = cur_artists.iter().position(|&a| a == displaced)
                        .map(|p| p + 1);

                    let ladder_title = "LADDER CHANGE".to_string();
                    let mut msg_parts = Vec::new();

                    match old_pos {
                        Some(_from) => {
                            msg_parts.push(format!(
                                "{} has knocked {} out of the #{} spot",
                                artist_name, displaced, new_pos
                            ));
                            msg_parts.push(format!(
                                "  \u{f062} {} \u{2192} #{}",
                                artist_name, new_pos
                            ));
                            match displaced_new_pos {
                                Some(dp) => {
                                    msg_parts.push(format!("  \u{f063} {} \u{2192} #{}", displaced, dp));
                                }
                                None => {
                                    msg_parts.push(format!("  \u{f063} {} out of Top 10", displaced));
                                }
                            }
                        }
                        None => {
                            // New entry to top 10
                            msg_parts.push(format!(
                                "{} has entered the Top 10 at #{}!",
                                artist_name, new_pos
                            ));
                            match displaced_new_pos {
                                Some(dp) => {
                                    msg_parts.push(format!(
                                        "  {} has been pushed to #{}",
                                        displaced, dp
                                    ));
                                }
                                None => {
                                    msg_parts.push(format!("  {} has dropped out of Top 10", displaced));
                                }
                            }
                        }
                    }

                    toasts.push((ladder_title, msg_parts.join("\n")));
                    break; // Only one ladder toast per track play
                }
            }
        }
        db.previous_top_10_snapshot = ranked;
    });

    toasts
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
    pub artist_minutes: Vec<(String, f64, u32)>,
    pub genre_minutes: Vec<(String, f64, u32)>,
    pub album_minutes: Vec<(String, f64, u32)>,
}

const TOP_N_BREAKDOWN: usize = 10;

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
        let mut album_minutes: HashMap<String, f64> = HashMap::new();

        let mut artist_tracks_count: HashMap<String, u32> = HashMap::new();
        let mut genre_tracks_count: HashMap<String, u32> = HashMap::new();
        let mut album_tracks_count: HashMap<String, u32> = HashMap::new();

        if *label == "All-Time" {
            total_plays += db.legacy_tracks;
            total_minutes += db.legacy_minutes;
            for (a, m) in &db.legacy_artist_minutes {
                *artist_minutes.entry(a.clone()).or_default() += m;
            }
            for (a, t) in &db.legacy_artist_tracks {
                *artist_tracks_count.entry(a.clone()).or_default() += t;
            }
        }

        // BTreeMap for deterministic iteration order (fixes cycling)
        let mut artist_album_list: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for track in tracks {
            if !track.artist.is_empty() && !track.album.is_empty() {
                let clean = if track.album.trim().is_empty() { "Unknown".to_string() } else { track.album.trim().to_string() };
                artist_album_list.entry(track.artist.clone()).or_default().push(clean);
            }
        }
        // Deduplicate album lists
        for albums in artist_album_list.values_mut() {
            albums.sort();
            albums.dedup();
        }

        for (date_str, day) in &db.daily_buckets {
            if filter(date_str) {
                total_plays += day.track_play_count;
                total_minutes += day.total_minutes;
                for (a, m) in &day.artist_minutes {
                    *artist_minutes.entry(a.clone()).or_default() += m;
                }
                for (a, t) in &day.artist_track_counts {
                    *artist_tracks_count.entry(a.clone()).or_default() += t;
                }
                if day.genre_minutes.is_empty() {
                    let mut artist_genre_counts: HashMap<String, HashMap<String, usize>> = HashMap::new();
                    for track in tracks {
                        if !track.artist.is_empty() && !track.genre.is_empty() {
                            for g in track.genres() {
                                let clean = if g.trim().is_empty() { "Unknown" } else { g.trim() };
                                *artist_genre_counts
                                    .entry(track.artist.clone())
                                    .or_default()
                                    .entry(clean.to_string())
                                    .or_default() += 1;
                            }
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
                for (g, t) in &day.genre_track_counts {
                    *genre_tracks_count.entry(g.clone()).or_default() += t;
                }
                if day.album_minutes.is_empty() {
                    // Split artist minutes evenly across all albums for that artist
                    for (a, m) in &day.artist_minutes {
                        if let Some(albums) = artist_album_list.get(a) {
                            if !albums.is_empty() {
                                let per_album = m / albums.len() as f64;
                                for album in albums {
                                    *album_minutes.entry(album.clone()).or_default() += per_album;
                                }
                            }
                        }
                    }
                } else {
                    for (al, m) in &day.album_minutes {
                        *album_minutes.entry(al.clone()).or_default() += m;
                    }
                }
                for (al, t) in &day.album_track_counts {
                    *album_tracks_count.entry(al.clone()).or_default() += t;
                }
            }
        }

        if *label == "All-Time" {
            for (al, m) in &db.legacy_album_minutes {
                *album_minutes.entry(al.clone()).or_default() += m;
            }
            for (al, t) in &db.legacy_album_tracks {
                *album_tracks_count.entry(al.clone()).or_default() += t;
            }
        }

        let mut artist_list: Vec<(String, f64, u32)> = artist_minutes.into_iter()
            .map(|(name, mins)| {
                let count = artist_tracks_count.get(&name).cloned().unwrap_or(0);
                (name, mins, count)
            })
            .collect();
        artist_list.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        artist_list.truncate(TOP_N_BREAKDOWN);
        let avg_song_mins: f64 = 4.0;
        let mut genre_list: Vec<(String, f64, u32)> = genre_minutes.into_iter()
            .map(|(name, mins)| {
                let count = genre_tracks_count.get(&name).cloned().unwrap_or(0);
                let estimated = if mins > 0.0 { (mins / avg_song_mins).ceil() as u32 } else { 0 };
                (name, mins, count.max(estimated))
            })
            .collect();
        genre_list.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        genre_list.truncate(TOP_N_BREAKDOWN);
        let mut album_list: Vec<(String, f64, u32)> = album_minutes.into_iter()
            .map(|(name, mins)| {
                let count = album_tracks_count.get(&name).cloned().unwrap_or(0);
                let estimated = if mins > 0.0 { (mins / avg_song_mins).ceil() as u32 } else { 0 };
                (name, mins, count.max(estimated))
            })
            .collect();
        album_list.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        album_list.truncate(TOP_N_BREAKDOWN);

        PeriodBreakdown {
            period_label: label.to_string(),
            total_minutes,
            total_plays,
            artist_minutes: artist_list,
            genre_minutes: genre_list,
            album_minutes: album_list,
        }
    })
}

// ── Song Breakdown (per-artist/album/genre) ──────────────────────────────────

#[derive(Debug, Clone)]
pub struct SongBreakdownItem {
    pub track_path: PathBuf,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub play_count: u32,
}

pub fn get_song_breakdown(
    category: &str,
    name: &str,
    period_idx: usize,
    tracks: &[crate::library::models::Track],
) -> Vec<SongBreakdownItem> {
    get(|db| {
        let now = chrono::Local::now().naive_local().date();
        let today_str = now.format("%Y-%m-%d").to_string();
        let days_from_monday = now.weekday().num_days_from_monday();
        let monday = now - chrono::Duration::days(days_from_monday as i64);
        let this_month_prefix = now.format("%Y-%m").to_string();

        let periods: Vec<(&str, Box<dyn Fn(&str) -> bool>)> = vec![
            ("Today", Box::new(move |d: &str| d == today_str)),
            ("This Week", Box::new(move |d: &str| {
                if let Some(date) = parse_date(d) { date >= monday && date <= now } else { false }
            })),
            ("This Month", Box::new(move |d: &str| d.starts_with(&this_month_prefix))),
            ("All-Time", Box::new(move |_d: &str| true)),
        ];

        let mut items: Vec<SongBreakdownItem> = Vec::new();

        if period_idx == 3 {
            // "All-Time": pull play counts directly from tracks slice loaded from db.json
            for track in tracks {
                if track.play_count > 0 {
                    let matches = match category {
                        "Artist" => track.artist == name,
                        "Album" => track.album == name,
                        "Genre" => track.genres().iter().any(|g| g.trim() == name),
                        _ => false,
                    };
                    if matches {
                        items.push(SongBreakdownItem {
                            track_path: track.path.clone(),
                            title: track.title.clone(),
                            artist: track.artist.clone(),
                            album: track.album.clone(),
                            play_count: track.play_count,
                        });
                    }
                }
            }
        } else {
            let (_label, filter) = &periods[period_idx];

            // Aggregate track_play_counts across filtered days
            let mut path_plays: HashMap<PathBuf, u32> = HashMap::new();
            for (date_str, day) in &db.daily_buckets {
                if filter(date_str) {
                    for (path, count) in &day.track_play_counts {
                        *path_plays.entry(path.clone()).or_default() += count;
                    }
                }
            }

            // Build track lookup from library
            let mut track_map: HashMap<PathBuf, &crate::library::models::Track> = HashMap::new();
            for track in tracks {
                track_map.insert(track.path.clone(), track);
            }

            for (path, count) in path_plays {
                if let Some(track) = track_map.get(&path) {
                    let matches = match category {
                        "Artist" => track.artist == name,
                        "Album" => track.album == name,
                        "Genre" => track.genres().iter().any(|g| g.trim() == name),
                        _ => false,
                    };
                    if matches {
                        items.push(SongBreakdownItem {
                            track_path: path,
                            title: track.title.clone(),
                            artist: track.artist.clone(),
                            album: track.album.clone(),
                            play_count: count,
                        });
                    }
                }
            }
        }

        // Sort by play_count descending, then alphabetically
        items.sort_by(|a, b| b.play_count.cmp(&a.play_count).then(a.title.cmp(&b.title)));
        items.truncate(100);
        items
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
                for g in track.genres() {
                    let clean = if g.trim().is_empty() { "Unknown" } else { g.trim() };
                    *artist_genre_counts
                        .entry(track.artist.clone())
                        .or_default()
                        .entry(clean.to_string())
                        .or_default() += 1;
                }
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
