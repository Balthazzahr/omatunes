#!/usr/bin/env python3
import subprocess
import json
import os
import re
import pathlib
import pickle
import urllib.request
import urllib.parse
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
        return subprocess.check_output(cmd, shell=True, stderr=subprocess.DEVNULL, text=True).strip()
    except:
        return ""

def escape(text):
    if text:
        return text.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")
    return ""

def truncate_text(text, max_length):
    return text[:max_length] + "…" if len(text) > max_length else text

def wrap_text(text, width):
    words = text.split()
    lines = []
    current_line = []
    current_length = 0
    for word in words:
        if current_length + len(word) + 1 <= width:
            current_line.append(word)
            current_length += len(word) + 1
        else:
            if current_line:
                lines.append(" ".join(current_line))
            current_line = [word]
            current_length = len(word)
    if current_line:
        lines.append(" ".join(current_line))
    return lines

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
# Load Theme & Colors
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

COLORS = load_omarchy_colors()

def get_css_color(var):
    css_path = pathlib.Path.home() / ".config/waybar/style.css"
    try:
        content = css_path.read_text()
        m = re.search(rf"@define-color\s+{var}\s+([#\w]+);", content)
        return m.group(1) if m else None
    except:
        return None

theme_colors = {
    "artist": COLORS.get("green"),
    "song": COLORS.get("white"),
    "album": COLORS.get("cyan"),
    "time": COLORS.get("white"),
    "volume": get_css_color("volume") or COLORS.get("cyan"),
    "status_playing": COLORS.get("green"),
    "status_stopped": COLORS.get("red"),
    "omatunes_brand": COLORS.get("cyan"),
    "progress": COLORS.get("blue"),
}

# -------------------
# Main OmaTunes Logic
# -------------------
status = get("playerctl --player=omatunes status").lower()

if not status or status == "stopped":
    print(json.dumps({}))
    exit()

title_raw = get("playerctl --player=omatunes metadata title")
artist_raw = get("playerctl --player=omatunes metadata artist")

if not title_raw:
    print(json.dumps({}))
    exit()

title = escape(title_raw)
artist = escape(artist_raw)
album = escape(get("playerctl --player=omatunes metadata album"))
volume = float(get("playerctl --player=omatunes volume") or 0)
position = int(float(get("playerctl --player=omatunes position") or 0))
length = int(get("playerctl --player=omatunes metadata mpris:length") or 0) // 1_000_000
shuffle = get("playerctl --player=omatunes shuffle").lower()
loop = get("playerctl --player=omatunes loop")

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
    send_notify("Music Milestone!", f"You've listened to {t_count} tracks today! ", sound_file="message.oga")
    session["last_track_milestone"] = triggered_t

# Hourly Logic
current_hours = int(session["daily_history"][today_str]["minutes"] // 60)
if current_hours > session["last_hour_milestone"]:
    send_notify("Time Flies!", f"You've been vibing for {current_hours} hour{'s' if current_hours > 1 else ''} today! ", icon="appointment-soon", sound_file="complete.oga")
    session["last_hour_milestone"] = current_hours

session["last_update"] = now
save_session(session)

# -------------------
# Visuals & Tooltip
# -------------------
def track_bar(pos, total, width=32, color=None):
    pos_t = f"{pos//60}:{pos%60:02d}"
    len_t = f"{total//60}:{total%60:02d}"
    avail = width - len(pos_t) - len(len_t) - 2
    filled = int((pos / total) * avail) if total else 0
    filled = min(max(0, filled), avail - 1)
    bar = "━" * filled + "●" + "┄" * (avail - filled - 1)
    return f"{pos_t} <span foreground='{color}'>{bar}</span> {len_t}"

def format_time(minutes):
    h, m = divmod(int(minutes), 60)
    return f"{h}h {m:02d}m" if h else f"{m}m"

tooltip_pre = [
    f"<span font='Montserrat Bold' foreground='{theme_colors['omatunes_brand']}' size='27500'>  OmaTunes</span>",
    f"<span font='Montserrat' size='10000'> </span>",  # vertical gap
    f"<span font='Montserrat' foreground='{theme_colors['artist']}'>   {truncate_text(artist, 40)}</span>",
    f"<span font='Montserrat' foreground='{theme_colors['song']}'>   {truncate_text(title, 40)}</span>",
    f"<span font='Montserrat' foreground='{theme_colors['album']}'>󰀥   {truncate_text(album, 40)}</span>",
    "",
]

# Statistics Section
stats_lines = []
stats_lines.append(f"<span foreground='{COLORS['white']}'> Listening History</span>:")
stats_lines.append("<span size='5000'> </span>")

# Helper for aligned stats
def format_stat_line(glyph, label, minutes, tracks, color_time, color_tracks):
    time_str = format_time(minutes)
    return (f"<span font_family='monospace'> {glyph} {label:<11} "
            f"<span foreground='{color_time}'>{time_str:>8}</span>  "
            f"<span foreground='{color_tracks}'>{tracks:>4} tracks</span></span>")

day_data = session["daily_history"].get(today_str, {"tracks": 0, "minutes": 0})
stats_lines.append(format_stat_line("󰃭", "Today:", day_data['minutes'], day_data['tracks'], theme_colors['artist'], theme_colors['song']))

week_data = session["weekly_history"].get(week_str, {"tracks": 0, "minutes": 0})
if week_data["tracks"] < day_data["tracks"]: week_data = day_data
stats_lines.append(format_stat_line("󰃮", "This Week:", week_data['minutes'], week_data['tracks'], theme_colors['artist'], theme_colors['song']))

month_data = session["monthly_history"].get(month_str, {"tracks": 0, "minutes": 0})
if month_data["tracks"] < day_data["tracks"]: month_data = day_data
stats_lines.append(format_stat_line("󰸗", "This Month:", month_data['minutes'], month_data['tracks'], theme_colors['artist'], theme_colors['song']))

# Last Month Logic
last_month_dt = (now.replace(day=1) - timedelta(days=1))
last_month_str = last_month_dt.strftime("%Y-%m")
last_month_data = session["monthly_history"].get(last_month_str, {"tracks": 0, "minutes": 0})
stats_lines.append(format_stat_line("", "Last Month:", last_month_data['minutes'], last_month_data['tracks'], theme_colors['artist'], theme_colors['song']))

# All-Time Logic
stats_lines.append(format_stat_line("󰓃", "All-Time:", session['total_minutes'], session['total_tracks'], COLORS['yellow'], COLORS['yellow']))

# Leaderboards
extra_lines = []

# 1. Monthly Leaderboard (Top 5)
month_history = session["monthly_history"].get(month_str, {})
m_artists_time = month_history.get("artists", {})
m_artists_tracks = month_history.get("artist_tracks", {})

if m_artists_time:
    extra_lines.append("")
    extra_lines.append(f"<span foreground='{COLORS['white']}'> Monthly Leaderboard (Top 5)</span>:")
    extra_lines.append("<span size='5000'> </span>")
    m_top = sorted(m_artists_time.items(), key=lambda x: x[1], reverse=True)[:5]
    for i, (name, mins) in enumerate(m_top):
        rank = i + 1
        t_count = m_artists_tracks.get(name, 0)
        rank_str = f"{rank}."
        name_trunc = truncate_text(name, 18)
        name_esc = escape(name_trunc)
        
        name_padding = " " * (18 - len(name_trunc))
        
        extra_lines.append(f"<span font_family='monospace'> {rank_str:>3}  {name_esc}{name_padding} <span foreground='{theme_colors['artist']}'>{format_time(mins):>7}</span> <span foreground='#aaaaaa' size='x-small'>({t_count} tracks)</span></span>")

# 2. All-Time Leaderboard (Top 10)
all_time_time = {}
all_time_tracks = {}
for m_data in session.get("monthly_history", {}).values():
    for name, mins in m_data.get("artists", {}).items():
        all_time_time[name] = all_time_time.get(name, 0) + mins
    for name, count in m_data.get("artist_tracks", {}).items():
        all_time_tracks[name] = all_time_tracks.get(name, 0) + count

if all_time_time:
    extra_lines.append("")
    extra_lines.append(f"<span foreground='{COLORS['white']}'>󰓃 All-Time Legends (Top 10)</span>:")
    extra_lines.append("<span size='5000'> </span>")
    a_top = sorted(all_time_time.items(), key=lambda x: x[1], reverse=True)[:10]
    a_medal_colors = ["#FFD700", "#C0C0C0", "#CD7F32"]
    for i, (name, mins) in enumerate(a_top):
        rank = i + 1
        t_count = all_time_tracks.get(name, 0)
        rank_str = f"{rank}."
        
        name_trunc = truncate_text(name, 18)
        name_esc = escape(name_trunc)
        
        if rank <= 3:
            name_part = f"<span foreground='{a_medal_colors[i]}'>{name_esc}</span>"
        else:
            name_part = name_esc
            
        name_padding = " " * (18 - len(name_trunc))
            
        extra_lines.append(f"<span font_family='monospace'> {rank_str:>3}  {name_part}{name_padding} <span foreground='{COLORS['blue']}'>{format_time(mins):>7}</span> <span foreground='#aaaaaa' size='x-small'>({t_count} tracks)</span></span>")

# Footer definitions
footer_text = "󰍽 L: Play  󰍽 M: Prev  󰍽 R: Next  󰍽 Scrl: Vol"
f_size = "9000"
f_size_val = float(f_size)

# Calculate natural max width from content
all_pre_lines = tooltip_pre + stats_lines + extra_lines
clean_lines = [re.sub(r'<.*?>', '', line) for line in all_pre_lines if line]
max_w = max(len(line) for line in clean_lines) if clean_lines else 35
max_w = min(max_w, 45)

# Build final tooltip list
tooltip = tooltip_pre
tooltip.append(f"<span>{'▶' if status == 'playing' else '⏸'} {track_bar(position, length, width=max_w-2, color=theme_colors['progress'])}</span>")
tooltip.append("<span size='10000'> </span>")

# Shuffle/Repeat with centering
shuffle_color = theme_colors["status_playing"] if shuffle == "on" else theme_colors["status_stopped"]
loop_color = theme_colors["status_playing"] if loop.lower() != "none" else theme_colors["status_stopped"] 
text_content_raw = f"  Repeat: {loop}    Shuffle: {shuffle.title()}"
padding = max(0, (max_w - len(text_content_raw)) // 2)
tooltip.append(f"<span font_family='monospace'>{' ' * padding}<span foreground='{loop_color}'>  Repeat: {loop}</span>  <span foreground='{shuffle_color}'>  Shuffle: {shuffle.title()}</span></span>")
tooltip.append("<span size='10000'> </span>")

vol_filled = int(volume * (max_w - 8))
vol_filled = max(1, min(vol_filled, max_w - 8))
vol_bar = "█" * vol_filled + "░" * (max_w - 8 - vol_filled)
tooltip.append(f"<span>  <span foreground='{theme_colors['volume']}'>{vol_bar}</span> {int(volume * 100)}%</span>")

tooltip.append("")
tooltip.extend(stats_lines)
tooltip.extend(extra_lines)

tooltip.append("")
tooltip.append(f"<span foreground='{COLORS['white']}'>{'┈' * max_w}</span>")

ratio = 10000.0 / f_size_val
def get_pad(text, target_w, r):
    visible_len = len(text)
    return int(max(0, (target_w * r - visible_len) / 2))

pad = get_pad(footer_text, max_w, ratio)

tooltip.append(f"<span font_family='monospace' size='{f_size}' foreground='{COLORS['white']}'>{' ' * pad}{footer_text}</span>")

status_icon = "" if status == "playing" else ""
icon_color = COLORS.get("cyan") if status == "playing" else theme_colors['artist']

if status == "playing":
    status_icon = ""
    icon_color = COLORS.get("cyan")
    artist_color = theme_colors['artist']
    song_color = theme_colors['song']
else:
    status_icon = ""
    icon_color = "#565f89" 
    artist_color = "#565f89"
    song_color = "#565f89"

display_text = (
    f"<span foreground='{icon_color}'>{status_icon} </span>"
    f"<span foreground='{artist_color}'><b>{artist}</b></span> - "
    f"<span foreground='{song_color}'><i>{truncate_text(title, 24)}</i></span>"
)

print(json.dumps({
    "text": display_text,
    "tooltip": "\n".join(tooltip),
    "markup": "pango",
    "class": status,
    "on-click": "playerctl --player=omatunes play-pause",
    "on-right-click": "playerctl --player=omatunes next",
    "on-middle-click": "playerctl --player=omatunes previous",
}))
