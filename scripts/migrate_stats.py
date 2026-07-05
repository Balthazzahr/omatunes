#!/usr/bin/env python3
import os
import pickle
import json
import pathlib
from datetime import datetime

# ─── STREAK TRADEOFF WARNING ──────────────────────────────────────────────────
# WARNING: Months covered only by 'monthly_history' (not 'daily_history') will
# be synthesized as a single day-bucket on the 1st of each month. This means
# active listening days will be undercounted during that historical period,
# potentially impacting streaks. This is a known tradeoff of the migration.
# ──────────────────────────────────────────────────────────────────────────────

def load_pickle(path):
    if not path.exists():
        return None
    try:
        with open(path, "rb") as f:
            return pickle.load(f)
    except Exception as e:
        print(f"Warning: Failed to load {path}: {e}")
        return None

def merge_histories(h1, h2):
    merged = {}
    all_keys = set(h1.keys()) | set(h2.keys())
    for k in all_keys:
        d1 = h1.get(k, {})
        d2 = h2.get(k, {})
        
        # Merge artists
        art1 = d1.get("artists", {})
        art2 = d2.get("artists", {})
        merged_art = {}
        for a in set(art1.keys()) | set(art2.keys()):
            merged_art[a] = art1.get(a, 0.0) + art2.get(a, 0.0)
            
        # Merge artist_tracks
        art_t1 = d1.get("artist_tracks", {})
        art_t2 = d2.get("artist_tracks", {})
        merged_art_t = {}
        for a in set(art_t1.keys()) | set(art_t2.keys()):
            merged_art_t[a] = art_t1.get(a, 0) + art_t2.get(a, 0)
            
        merged[k] = {
            "tracks": d1.get("tracks", 0) + d2.get("tracks", 0),
            "minutes": d1.get("minutes", 0.0) + d2.get("minutes", 0.0),
            "artists": merged_art,
            "artist_tracks": merged_art_t
        }
    return merged

def migrate():
    home = pathlib.Path.home()
    spotify_path = home / ".cache" / "waybar_spotify_session.pkl"
    omatunes_path = home / ".cache" / "waybar_omatunes_session.pkl"
    json_path = home / ".config" / "omatunes" / "stats.json"

    print("Starting omatunes listening history migration...")
    
    session_spotify = load_pickle(spotify_path)
    session_omatunes = load_pickle(omatunes_path)

    if not session_spotify and not session_omatunes:
        print("Error: No legacy session files found. Nothing to migrate.")
        return

    # Merge session structures
    session = {}
    if session_spotify and session_omatunes:
        print("Both Spotify and omaTUNES session pickle files found. Merging histories...")
        session["total_tracks"] = session_spotify.get("total_tracks", 0) + session_omatunes.get("total_tracks", 0)
        session["total_minutes"] = session_spotify.get("total_minutes", 0.0) + session_omatunes.get("total_minutes", 0.0)
        session["daily_history"] = merge_histories(session_spotify.get("daily_history", {}), session_omatunes.get("daily_history", {}))
        session["monthly_history"] = merge_histories(session_spotify.get("monthly_history", {}), session_omatunes.get("monthly_history", {}))
    elif session_spotify:
        print("Found legacy Spotify session file.")
        session = session_spotify
    else:
        print("Found legacy omaTUNES session file.")
        session = session_omatunes

    daily_buckets = {}
    legacy_tracks = session.get("total_tracks", 0)
    legacy_minutes = session.get("total_minutes", 0.0)

    # 1. Process daily history (detailed tracking)
    daily_hist = session.get("daily_history", {})
    migrated_days_tracks = 0
    migrated_days_minutes = 0.0

    print(f"Processing {len(daily_hist)} daily history entries...")
    for date_str, data in daily_hist.items():
        tracks = data.get("tracks", 0)
        minutes = data.get("minutes", 0.0)
        artists = data.get("artists", {})
        artist_tracks = data.get("artist_tracks", {})
        
        migrated_days_tracks += tracks
        migrated_days_minutes += minutes

        daily_buckets[date_str] = {
            "total_minutes": minutes,
            "track_play_count": tracks,
            "artist_minutes": artists,
            "artist_track_counts": artist_tracks,
            "track_play_counts": {},
            "genre_minutes": {},
            "longest_session_minutes": 0.0
        }

    # 2. Process monthly history (synthesizing)
    monthly_hist = session.get("monthly_history", {})
    print(f"Processing {len(monthly_hist)} monthly history entries...")
    
    for month_str, data in monthly_hist.items():
        has_daily_in_month = any(day.startswith(month_str) for day in daily_buckets.keys())
        if not has_daily_in_month:
            synth_date = f"{month_str}-01"
            tracks = data.get("tracks", 0)
            minutes = data.get("minutes", 0.0)
            artists = data.get("artists", {})
            artist_tracks = data.get("artist_tracks", {})
            
            print(f"  [Note] Synthesizing monthly statistics for {month_str} on {synth_date}")
            
            migrated_days_tracks += tracks
            migrated_days_minutes += minutes
            
            daily_buckets[synth_date] = {
                "total_minutes": minutes,
                "track_play_count": tracks,
                "artist_minutes": artists,
                "artist_track_counts": artist_tracks,
                "track_play_counts": {},
                "genre_minutes": {},
                "longest_session_minutes": 0.0
            }

    legacy_tracks = max(0, legacy_tracks - migrated_days_tracks)
    legacy_minutes = max(0.0, legacy_minutes - migrated_days_minutes)

    legacy_artist_minutes = {}
    legacy_artist_tracks = {}
    
    for month_str, data in monthly_hist.items():
        for artist, mins in data.get("artists", {}).items():
            legacy_artist_minutes[artist] = legacy_artist_minutes.get(artist, 0.0) + mins
        for artist, counts in data.get("artist_tracks", {}).items():
            legacy_artist_tracks[artist] = legacy_artist_tracks.get(artist, 0) + counts

    for date_str, data in daily_hist.items():
        for artist, mins in data.get("artists", {}).items():
            if artist in legacy_artist_minutes:
                legacy_artist_minutes[artist] = max(0.0, legacy_artist_minutes[artist] - mins)
        for artist, counts in data.get("artist_tracks", {}).items():
            if artist in legacy_artist_tracks:
                legacy_artist_tracks[artist] = max(0, legacy_artist_tracks[artist] - counts)

    legacy_artist_minutes = {k: v for k, v in legacy_artist_minutes.items() if v > 0.01}
    legacy_artist_tracks = {k: v for k, v in legacy_artist_tracks.items() if v > 0}

    stats_db = {
        "daily_buckets": daily_buckets,
        "legacy_tracks": int(legacy_tracks),
        "legacy_minutes": float(legacy_minutes),
        "legacy_artist_minutes": legacy_artist_minutes,
        "legacy_artist_tracks": legacy_artist_tracks,
        "last_active_timestamp": None,
        "current_session_accum_secs": 0
    }

    json_path.parent.mkdir(parents=True, exist_ok=True)
    
    with open(json_path, "w") as f:
        json.dump(stats_db, f, indent=4)

    print(f"Migration completed successfully! Saved to: {json_path}")
    print(f"Migrated daily buckets: {len(daily_buckets)}")
    print(f"Legacy Offset - Tracks: {legacy_tracks}, Minutes: {legacy_minutes:.2f}")

if __name__ == "__main__":
    migrate()
