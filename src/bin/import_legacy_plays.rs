use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
struct OmatunesDb {
    play_counts: HashMap<PathBuf, u32>,
    recently_played: Vec<(PathBuf, String)>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct DayStats {
    total_minutes: f64,
    track_play_count: u32,
    artist_minutes: HashMap<String, f64>,
    artist_track_counts: HashMap<String, u32>,
    track_play_counts: HashMap<PathBuf, u32>,
    genre_minutes: HashMap<String, f64>,
    album_minutes: HashMap<String, f64>,
    longest_session_minutes: f64,
    genre_track_counts: HashMap<String, u32>,
    album_track_counts: HashMap<String, u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct StatsDb {
    daily_buckets: HashMap<String, DayStats>,
    legacy_tracks: u32,
    legacy_minutes: f64,
    legacy_artist_minutes: HashMap<String, f64>,
    legacy_artist_tracks: HashMap<String, u32>,
    last_active_timestamp: Option<i64>,
    current_session_accum_secs: f64,
    all_time_artist_tracks: HashMap<String, u32>,
    all_time_genre_tracks: HashMap<String, u32>,
    artist_milestones_awarded: HashMap<String, HashSet<String>>,
    genre_milestones_awarded: HashMap<String, HashSet<String>>,
    daily_milestones_awarded: HashMap<String, HashSet<u32>>,
    previous_top_10_snapshot: Vec<(String, f64)>,
    legacy_albums_populated_v2: bool,
    legacy_album_minutes: HashMap<String, f64>,
    legacy_album_tracks: HashMap<String, u32>,
    legacy_track_counts_populated: bool,
}

struct TrackMeta {
    title: String,
    artist: String,
    album: String,
    genres: Vec<String>,
    duration_mins: f64,
}

fn get_track_metadata(path: &Path) -> TrackMeta {
    let unknown = "Unknown";
    
    // Fallback info from paths
    let file_title = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(unknown)
        .to_string();

    let folder_artist = path.parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or(unknown)
        .to_string();

    let folder_album = path.parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or(unknown)
        .to_string();

    if path.exists() {
        if let Ok(tagged_file) = lofty::probe::Probe::open(path).and_then(|p| p.read()) {
            let duration_mins = tagged_file.properties().duration().as_secs_f64() / 60.0;
            let tags = tagged_file.primary_tag();
            
            let title = tags
                .and_then(|t| t.title())
                .map(|s| s.to_string())
                .unwrap_or(file_title);

            let artist = tags
                .and_then(|t| t.artist())
                .map(|s| s.to_string())
                .unwrap_or(folder_artist);

            let album = tags
                .and_then(|t| t.album())
                .map(|s| s.to_string())
                .unwrap_or(folder_album);

            let genre_str = tags
                .and_then(|t| t.genre())
                .map(|s| s.to_string())
                .unwrap_or_else(|| unknown.to_string());

            let genres: Vec<String> = if genre_str.contains("; ") {
                genre_str.split("; ").map(|g| {
                    let clean = g.trim();
                    if clean.is_empty() { unknown.to_string() } else { clean.to_string() }
                }).collect()
            } else {
                let clean = genre_str.trim();
                vec![if clean.is_empty() { unknown.to_string() } else { clean.to_string() }]
            };

            return TrackMeta {
                title,
                artist,
                album,
                genres,
                duration_mins,
            };
        }
    }

    // Default fallback if file doesn't exist or isn't readable
    TrackMeta {
        title: file_title,
        artist: folder_artist,
        album: folder_album,
        genres: vec![unknown.to_string()],
        duration_mins: 4.0, // default 4 minutes
    }
}

fn main() {
    let home = std::env::var("HOME").expect("HOME env var not set");
    let db_path = PathBuf::from(&home).join(".config/omatunes/db.json");
    let stats_path = PathBuf::from(&home).join(".config/omatunes/stats.json");

    println!("Starting import of play counts from {}...", db_path.display());

    if !db_path.exists() {
        eprintln!("Error: db.json does not exist at {}", db_path.display());
        std::process::exit(1);
    }

    let db_content = std::fs::read_to_string(&db_path).expect("Failed to read db.json");
    let db: OmatunesDb = serde_json::from_str(&db_content).expect("Failed to parse db.json");

    println!("Loaded db.json. Found {} play count entries and {} recently played items.", 
             db.play_counts.len(), db.recently_played.len());

    // Map track paths to their recently played date strings
    let mut recent_map: HashMap<PathBuf, String> = HashMap::new();
    for (path, date_str) in &db.recently_played {
        // recently_played contains format: "YYYY-MM-DD HH:MM"
        // We only need the date part: "YYYY-MM-DD"
        let date_only = date_str.split(' ').next().unwrap_or(date_str).to_string();
        recent_map.insert(path.clone(), date_only);
    }

    let mut stats = StatsDb::default();
    stats.legacy_albums_populated_v2 = true;
    stats.legacy_track_counts_populated = true;

    // Use "2026-05-01" as a historical dummy date for legacy plays
    let legacy_date = "2026-05-01".to_string();

    let mut processed_count = 0;
    let mut total_plays_migrated = 0;

    for (path, play_count) in &db.play_counts {
        if *play_count == 0 {
            continue;
        }

        processed_count += 1;
        let meta = get_track_metadata(path);

        // Partition play count
        let recent_date = recent_map.get(path);
        
        let (recent_plays, legacy_plays) = match recent_date {
            Some(_) => (1, play_count - 1),
            None => (0, *play_count),
        };

        total_plays_migrated += play_count;

        // 1. Record Recent Play
        if recent_plays > 0 {
            if let Some(date_str) = recent_date {
                let day = stats.daily_buckets.entry(date_str.clone()).or_default();
                day.track_play_count += recent_plays;
                day.total_minutes += meta.duration_mins * (recent_plays as f64);
                *day.track_play_counts.entry(path.clone()).or_default() += recent_plays;
                *day.artist_track_counts.entry(meta.artist.clone()).or_default() += recent_plays;
                *day.artist_minutes.entry(meta.artist.clone()).or_default() += meta.duration_mins * (recent_plays as f64);
                *day.album_track_counts.entry(meta.album.clone()).or_default() += recent_plays;
                *day.album_minutes.entry(meta.album.clone()).or_default() += meta.duration_mins * (recent_plays as f64);
                for g in &meta.genres {
                    *day.genre_track_counts.entry(g.clone()).or_default() += recent_plays;
                    *day.genre_minutes.entry(g.clone()).or_default() += meta.duration_mins * (recent_plays as f64);
                }
            }
        }

        // 2. Record Legacy Plays
        if legacy_plays > 0 {
            let day = stats.daily_buckets.entry(legacy_date.clone()).or_default();
            day.track_play_count += legacy_plays;
            day.total_minutes += meta.duration_mins * (legacy_plays as f64);
            *day.track_play_counts.entry(path.clone()).or_default() += legacy_plays;
            *day.artist_track_counts.entry(meta.artist.clone()).or_default() += legacy_plays;
            *day.artist_minutes.entry(meta.artist.clone()).or_default() += meta.duration_mins * (legacy_plays as f64);
            *day.album_track_counts.entry(meta.album.clone()).or_default() += legacy_plays;
            *day.album_minutes.entry(meta.album.clone()).or_default() += meta.duration_mins * (legacy_plays as f64);
            for g in &meta.genres {
                *day.genre_track_counts.entry(g.clone()).or_default() += legacy_plays;
                *day.genre_minutes.entry(g.clone()).or_default() += meta.duration_mins * (legacy_plays as f64);
            }
        }

        // 3. Accumulate all-time counts directly for fast reference
        *stats.all_time_artist_tracks.entry(meta.artist.clone()).or_default() += play_count;
        for g in &meta.genres {
            *stats.all_time_genre_tracks.entry(g.clone()).or_default() += play_count;
        }

        // Init milestone map keys to match application's expected format
        stats.artist_milestones_awarded.entry(meta.artist.clone()).or_default();
        for g in &meta.genres {
            stats.genre_milestones_awarded.entry(g.clone()).or_default();
        }
    }

    // 4. Build previous top 10 snapshots by minutes
    let mut all_artist_minutes: HashMap<String, f64> = HashMap::new();
    for (_, day) in &stats.daily_buckets {
        for (artist, mins) in &day.artist_minutes {
            *all_artist_minutes.entry(artist.clone()).or_default() += mins;
        }
    }
    let mut ranked: Vec<(String, f64)> = all_artist_minutes.into_iter().collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    ranked.truncate(10);
    stats.previous_top_10_snapshot = ranked;

    // Save to stats.json
    let stats_json = serde_json::to_string_pretty(&stats).expect("Failed to serialize stats.json");
    std::fs::write(&stats_path, stats_json).expect("Failed to write to stats.json");

    println!("Successfully processed {} tracks with {} total plays.", processed_count, total_plays_migrated);
    println!("Saved listening stats database to {}.", stats_path.display());
}
