#!/usr/bin/env python3
import subprocess
import json
import os
import re
import pathlib
import pickle
import sys
import time
from datetime import datetime, timedelta

# -------------------
# Helper functions
# -------------------
try:
    import tomllib
except ImportError:
    tomllib = None

def get(cmd):
    try:
        return subprocess.check_output(cmd, stderr=subprocess.DEVNULL, text=True).strip()
    except:
        return ""

def escape(text):
    if text:
        return text.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")
    return ""

def truncate_text(text, max_length):
    return text[:max_length] + "…" if len(text) > max_length else text

def alacritty_color_to_hex(c):
    if isinstance(c, str):
        return c
    if isinstance(c, dict) and {"r", "g", "b"} <= c.keys():
        return "#{:02x}{:02x}{:02x}".format(c["r"], c["g"], c["b"])
    return "#ffffff"

import random

def send_notify(title, message, icon="multimedia-audio-player", sound_file=None):
    try:
        # Visual notification
        subprocess.Popen(["notify-send", "-a", "OmaTunes Stats", "-i", icon, title, message])
        # Sound notification
        if sound_file:
            sound_path = f"/usr/share/sounds/freedesktop/stereo/{sound_file}"
            if os.path.exists(sound_path):
                # Using paplay for PulseAudio/PipeWire
                subprocess.Popen(["paplay", sound_path])
    except:
        pass



# -------------------
# Session tracking
# -------------------
CACHE_DIR = pathlib.Path.home() / ".cache"
CACHE_DIR.mkdir(exist_ok=True)
SESSION_FILE = CACHE_DIR / "waybar_omatunes_session.pkl"

# ─── FORWARD-LOOKING NOTES — DO NOT MODIFY WITHOUT READING ─────────────────────
# 1. STATISTICS FEATURE: The session data accumulated here (daily_history,
#    weekly_history, monthly_history, total_tracks, total_minutes, artist data,
#    records) is intentionally retained for a planned in-app Statistics feature
#    inside the Rust omaTUNES application. Do NOT remove or restructure these
#    fields without first verifying whether that feature depends on the current
#    field layout.
#
# 2. PICKLE-TO-JSON MIGRATION: The storage format (Python pickle) is planned to
#    migrate to JSON in a follow-up task, to allow the Rust app to read the file
#    directly. Do NOT perform that migration here — this comment is a marker for
#    future work only.
# ────────────────────────────────────────────────────────────────────────────────

def load_session():
    now = datetime.now()
    today_str = now.strftime("%Y-%m-%d")
    month_str = now.strftime("%Y-%m")
    
    # Calculate week start (Monday)
    days_since_monday = now.weekday()
    week_start = (now - timedelta(days=days_since_monday)).replace(hour=0, minute=0, second=0, microsecond=0)
    week_str = week_start.strftime("%Y-%m-%d")
    
    defaults = {
        "last_track": "",
        "last_update": now,
        "last_track_milestone": 0,
        "last_hour_milestone": 0,
        "daily_history": {},
        "weekly_history": {}, 
        "monthly_history": {},
        "records": {"max_tracks_day": 0, "max_minutes_day": 0},
        "total_tracks": 0,
        "total_minutes": 0,
        "ignored_artists": []
    }
    
    session = defaults.copy()
    try:
        if os.path.exists(SESSION_FILE):
            with open(SESSION_FILE, "rb") as f:
                loaded = pickle.load(f)
                if isinstance(loaded, dict):
                    session.update(loaded)
    except:
        pass
        
    # Ensure all keys exist
    for key in ["daily_history", "weekly_history", "monthly_history", "records", "ignored_artists"]:
        if key not in session:
            session[key] = defaults[key]
            
    # Ensure current periods exist
    if today_str not in session["daily_history"]:
        session["daily_history"][today_str] = {"tracks": 0, "minutes": 0, "artists": {}, "artist_tracks": {}}
    if week_str not in session["weekly_history"]:
        session["weekly_history"][week_str] = {"tracks": 0, "minutes": 0, "artists": {}, "artist_tracks": {}}
    if month_str not in session["monthly_history"]:
        session["monthly_history"][month_str] = {"tracks": 0, "minutes": 0, "artists": {}, "artist_tracks": {}}
        
    return session

def save_session(session):
    try:
        # Cleanup old history (keep last 60 days)
        if len(session["daily_history"]) > 60:
            sorted_days = sorted(session["daily_history"].keys())
            for day in sorted_days[:-60]:
                del session["daily_history"][day]
        
        # Keep last 12 months
        if len(session["monthly_history"]) > 12:
            sorted_months = sorted(session["monthly_history"].keys())
            for month in sorted_months[:-12]:
                del session["monthly_history"][month]

        with open(SESSION_FILE, "wb") as f:
            pickle.dump(session, f)
    except:
        pass

# -------------------
# Load Theme & Colors (Cached)
# -------------------
def load_omarchy_colors():
    theme_path = pathlib.Path.home() / ".config/omarchy/current/theme/alacritty.toml"
    try:
        if not tomllib:
            raise ImportError
        data = tomllib.loads(theme_path.read_text())
        colors = data.get("colors", {})
        normal = colors.get("normal", {})
        bright = colors.get("bright", {})
        return {
            "green": alacritty_color_to_hex(normal.get("green")),
            "yellow": alacritty_color_to_hex(normal.get("yellow")),
            "cyan": alacritty_color_to_hex(normal.get("cyan")),
            "white": alacritty_color_to_hex(normal.get("white")),
            "red": alacritty_color_to_hex(normal.get("red")),
            "blue": alacritty_color_to_hex(bright.get("blue")),
        }
    except:
        return {"green": "#00ff00", "yellow": "#ffff00", "cyan": "#00ffff", "white": "#ffffff", "red": "#ff0000", "blue": "#0000ff"}

# Note: get_theme_colors is defined here
THEME_CACHE_FILE = pathlib.Path.home() / ".cache" / "waybar_omatunes_theme_cache.json"

def get_theme_colors():
    alacritty_path = pathlib.Path.home() / ".config/omarchy/current/theme/alacritty.toml"
    waybar_path = pathlib.Path.home() / ".config/waybar/style.css"
    
    alacritty_mtime = 0.0
    waybar_mtime = 0.0
    try:
        if alacritty_path.exists():
            alacritty_mtime = os.path.getmtime(alacritty_path)
    except:
        pass
    try:
        if waybar_path.exists():
            waybar_mtime = os.path.getmtime(waybar_path)
    except:
        pass

    cache_valid = False
    cache_data = {}
    if THEME_CACHE_FILE.exists():
        try:
            with open(THEME_CACHE_FILE, "r") as f:
                cache_data = json.load(f)
            if (cache_data.get("alacritty_mtime") == alacritty_mtime and 
                cache_data.get("waybar_mtime") == waybar_mtime):
                cache_valid = True
        except:
            pass

    if cache_valid:
        return cache_data["COLORS"], cache_data["theme_colors"]

    # Compute and save
    global tomllib
    colors_dict = load_omarchy_colors()

    theme_colors_dict = {
        "artist": colors_dict.get("green"),
        "song": colors_dict.get("white"),
        "album": colors_dict.get("cyan"),
        "omatunes_brand": colors_dict.get("cyan"),
    }
    
    try:
        with open(THEME_CACHE_FILE, "w") as f:
            json.dump({
                "alacritty_mtime": alacritty_mtime,
                "waybar_mtime": waybar_mtime,
                "COLORS": colors_dict,
                "theme_colors": theme_colors_dict
            }, f)
    except:
        pass

    return colors_dict, theme_colors_dict

COLORS, theme_colors = get_theme_colors()

if len(sys.argv) > 1:
    arg = sys.argv[1]
    if arg == "--click" and len(sys.argv) > 2:
        button = sys.argv[2]
        import socket
        try:
            s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
            if button == "play":
                s.sendto(b"play-pause", ("127.0.0.1", 18888))
            elif button == "next":
                s.sendto(b"next", ("127.0.0.1", 18888))
            elif button == "like":
                s.sendto(b"like", ("127.0.0.1", 18888))
            s.close()
        except:
            pass
        # --click focus: focus the omaTUNES window via hyprctl.
        # Not wired to any Waybar action currently (on-click-backward requires
        # Waybar >= a later release). Kept here for potential future use.
        if button == "focus":
            try:
                subprocess.Popen(
                    ["hyprctl", "dispatch", "focuswindow", "class:^omatunes$"],
                    stderr=subprocess.DEVNULL
                )
            except:
                pass
        sys.exit(0)

# -------------------
# Main OmaTunes Logic
# -------------------
cmd = [
    "playerctl",
    "-f",
    "{{status}}||{{title}}||{{artist}}||{{album}}",
    "--player=omatunes",
    "metadata"
]

raw_output = get(cmd)
parts = raw_output.split("||")

if len(parts) != 4:
    print(json.dumps({}))
    sys.exit(0)

status = parts[0].lower().strip()

if not status or status == "stopped":
    print(json.dumps({}))
    sys.exit(0)

title_raw = parts[1].strip()
if not title_raw:
    print(json.dumps({}))
    sys.exit(0)

title = escape(title_raw)
artist_raw = parts[2].strip()
artist = escape(artist_raw)
album = escape(parts[3].strip())

# -------------------
# Session & Notifications
# -------------------
session = load_session()
now = datetime.now()
today_str = now.strftime("%Y-%m-%d")
month_str = now.strftime("%Y-%m")
days_since_monday = now.weekday()
week_str = (now - timedelta(days=days_since_monday)).strftime("%Y-%m-%d")

# Update logic
track_id = f"{artist}-{title}"
if track_id != session["last_track"]:
    # Update all active periods
    for period_key in [("daily_history", today_str), ("weekly_history", week_str), ("monthly_history", month_str)]:
        hist_type, key = period_key
        session[hist_type][key]["tracks"] += 1
        if "artist_tracks" not in session[hist_type][key]: 
            session[hist_type][key]["artist_tracks"] = {}
        session[hist_type][key]["artist_tracks"][artist_raw] = session[hist_type][key]["artist_tracks"].get(artist_raw, 0) + 1
        
    session["total_tracks"] += 1
    session["last_track"] = track_id

if status == "playing":
    delta_seconds = (now - session["last_update"]).total_seconds()
    if 0 < delta_seconds < 20:
        delta_minutes = delta_seconds / 60
        
        # Update all active periods
        for period_key in [("daily_history", today_str), ("weekly_history", week_str), ("monthly_history", month_str)]:
            hist_type, key = period_key
            session[hist_type][key]["minutes"] += delta_minutes
            if "artists" not in session[hist_type][key]: session[hist_type][key]["artists"] = {}
            artists = session[hist_type][key]["artists"]
            artists[artist_raw] = artists.get(artist_raw, 0) + delta_minutes
        
        # Update totals
        session["total_minutes"] += delta_minutes
        
        # Check Records
        if session["daily_history"][today_str]["tracks"] > session["records"].get("max_tracks_day", 0):
            session["records"]["max_tracks_day"] = session["daily_history"][today_str]["tracks"]
            
        if session["daily_history"][today_str]["minutes"] > session["records"].get("max_minutes_day", 0):
            session["records"]["max_minutes_day"] = session["daily_history"][today_str]["minutes"]

# Milestone Logic
t_count = session["daily_history"][today_str]["tracks"]
last_t = session["last_track_milestone"]
triggered_t = 0

if t_count >= 10 and last_t < 10: triggered_t = 10
elif t_count >= 50 and last_t < 50: triggered_t = 50
elif t_count >= 100 and (t_count // 100 > last_t // 100):
    triggered_t = (t_count // 100) * 100

if triggered_t > 0:
    send_notify("Music Milestone!", f"You've listened to {t_count} tracks today! ", sound_file="message.oga")
    session["last_track_milestone"] = triggered_t

# Hourly Logic
current_hours = int(session["daily_history"][today_str]["minutes"] // 60)
if current_hours > session["last_hour_milestone"]:
    send_notify("Time Flies!", f"You've been vibing for {current_hours} hour{'s' if current_hours > 1 else ''} today! ", icon="appointment-soon", sound_file="complete.oga")
    session["last_hour_milestone"] = current_hours

session["last_update"] = now
save_session(session)

# -------------------
# Visuals & Tooltip
# -------------------
# Hint-list glyphs — all verified PRESENT in bundled font via cmap format-12 parse:
#   U+F001  nf-fa-music       (bar text icon + header)
#   U+F040A nf-md-play        (Play/Pause)
#   U+F02D1 nf-md-heart       (Like)
#   U+F04AD nf-md-skip-next   (Next Track)
#   U+F057E nf-md-volume-high (Scroll/Volume)

tooltip_lines = [
    f"<span font='Montserrat Bold' foreground='{theme_colors['omatunes_brand']}' size='27500'>\uf001  OmaTunes</span>",
    f"<span font='Montserrat' size='10000'> </span>",
    f"<span font='Montserrat' foreground='{theme_colors['artist']}'>\uf007   {truncate_text(artist, 40)}</span>",
    f"<span font='Montserrat' foreground='{theme_colors['song']}'>\uf001   {truncate_text(title, 40)}</span>",
    f"<span font='Montserrat' foreground='{theme_colors['album']}'>\U000f0025   {truncate_text(album, 40)}</span>",
    "",
    "\U000f040a  Left Click \u2014 Play/Pause",
    "\U000f02d1  Middle Click \u2014 Like Song",
    "\U000f04ad  Right Click \u2014 Next Track",
    "\U000f057e  Scroll \u2014 Volume Up/Down",
]

icon_color = COLORS.get("cyan") if status == "playing" else "#565f89"

if status == "playing":
    artist_color = theme_colors['artist']
    song_color = theme_colors['song']
else:
    artist_color = "#565f89"
    song_color = "#565f89"

display_text = (
    f"<span foreground='{icon_color}'>\uf001 </span>"
    f"<span foreground='{artist_color}'><b>{artist}</b></span> - "
    f"<span foreground='{song_color}'><i>{truncate_text(title, 24)}</i></span>"
)

print(json.dumps({
    "text": display_text,
    "tooltip": "\n".join(tooltip_lines),
    "markup": "pango",
    "class": status,
}))
