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

def migrate():
    home = pathlib.Path.home()
    pickle_path = home / ".cache" / "waybar_omatunes_session.pkl"
    json_path = home / ".config" / "omatunes" / "stats.json"

    print("Starting omatunes listening history migration...")
    print(f"Reading pickle file: {pickle_path}")

    if not pickle_path.exists():
        print(f"Error: {pickle_path} does not exist. Nothing to migrate.")
        return

    try:
        with open(pickle_path, "rb") as f:
            session = pickle.load(f)
    except Exception as e:
        print(f"Failed to load pickle file: {e}")
        return

    print("Pickle file loaded successfully.")
    
    daily_buckets = {}
    legacy_tracks = session.get("total_tracks", 0)
    legacy_minutes = session.get("total_minutes", 0)

    # 1. Process daily history (up to last 60 days of detailed tracking)
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
            "track_play_counts": {} # track-level counts not detailed in old daily history
        }

    # 2. Process monthly history (synthesizing U+2014 undercount tradeoff warning)
    # Any month present in monthly_history but not represented in daily_history
    # gets consolidated on the 1st of that month.
    monthly_hist = session.get("monthly_history", {})
    print(f"Processing {len(monthly_hist)} monthly history entries...")
    
    for month_str, data in monthly_hist.items():
        # Check if we have daily buckets for this month
        has_daily_in_month = any(day.startswith(month_str) for day in daily_buckets.keys())
        if not has_daily_in_month:
            # Consolidate full month on YYYY-MM-01
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
                "track_play_counts": {}
            }

    # 3. Calculate legacy offsets (totals minus what was migrated in daily/monthly detail)
    legacy_tracks = max(0, legacy_tracks - migrated_days_tracks)
    legacy_minutes = max(0.0, legacy_minutes - migrated_days_minutes)

    # Compile all-time artist totals from the pickle file to determine legacy artist counts
    # (Since all-time leaderboard artist tracking isn't saved directly in the session, 
    # we can aggregate the monthly artist history as legacy artist totals)
    legacy_artist_minutes = {}
    legacy_artist_tracks = {}
    
    # Sum up all months
    for month_str, data in monthly_hist.items():
        for artist, mins in data.get("artists", {}).items():
            legacy_artist_minutes[artist] = legacy_artist_minutes.get(artist, 0.0) + mins
        for artist, counts in data.get("artist_tracks", {}).items():
            legacy_artist_tracks[artist] = legacy_artist_tracks.get(artist, 0) + counts

    # Subtract the daily history parts we migrated to avoid double counting
    for date_str, data in daily_hist.items():
        for artist, mins in data.get("artists", {}).items():
            if artist in legacy_artist_minutes:
                legacy_artist_minutes[artist] = max(0.0, legacy_artist_minutes[artist] - mins)
        for artist, counts in data.get("artist_tracks", {}).items():
            if artist in legacy_artist_tracks:
                legacy_artist_tracks[artist] = max(0, legacy_artist_tracks[artist] - counts)

    # Prune tiny artist values
    legacy_artist_minutes = {k: v for k, v in legacy_artist_minutes.items() if v > 0.01}
    legacy_artist_tracks = {k: v for k, v in legacy_artist_tracks.items() if v > 0}

    stats_db = {
        "daily_buckets": daily_buckets,
        "legacy_tracks": int(legacy_tracks),
        "legacy_minutes": float(legacy_minutes),
        "legacy_artist_minutes": legacy_artist_minutes,
        "legacy_artist_tracks": legacy_artist_tracks
    }

    # Ensure parent directory exists
    json_path.parent.mkdir(parents=True, exist_ok=True)
    
    with open(json_path, "w") as f:
        json.dump(stats_db, f, indent=4)

    print(f"Migration completed successfully! Saved to: {json_path}")
    print(f"Migrated daily buckets: {len(daily_buckets)}")
    print(f"Legacy Offset - Tracks: {legacy_tracks}, Minutes: {legacy_minutes:.2f}")
    print("[Streak Warning] Historical months merged on the 1st of each month will count as a single active day for streak calculation.")

if __name__ == "__main__":
    migrate()
