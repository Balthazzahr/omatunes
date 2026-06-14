# omatunes

A native Wayland music player written in Rust, built for [Omarchy](https://omarchy.org/) / Hyprland rices. Follows the active Omarchy theme automatically — colors update live when you switch themes.

`omatunes` is a customized fork of [sheep-farm/lavanda](https://github.com/sheep-farm/lavanda) by [Balthazzahr](https://github.com/Balthazzahr).

---

## Key Enhancements & Features

- **Multi-Track Metadata Editing**: Support for bulk editing ID3 tags on multiple selected tracks. Checked fields apply only to selections (or entire albums if toggled). Year tag is fully integrated.
- **Smart Autocomplete Suggestions**: Dynamic pills suggest matching Artists, Albums, and Genres from your library as you type. Editing a field or selecting a suggestion automatically ticks the checkbox.
- **Customizable Track Columns**: Right-click track table headers to customize shown columns and re-order them (Move Left / Move Right). Layouts persist to the library database.
- **Refactored Playlist Views**: Simplified two-tab playlist section:
  - *User Playlists*: Custom folders with the `New Playlist` button placed cleanly at the bottom.
  - *Autoplaylists*: Smart lists (`Liked Songs`, `Recently Played`, `Most Played`) rendered as standard list items.
- **Clean Folder Tabs**: Redesigned Artist/Album/Genre and Playlist tabs to span the full sidebar width with a thin `28px` profile. Features a `4.0` border radius and `0` spacing to prevent anti-aliasing gaps.
- **Upscaled UI Layout**: 20% larger cover art (`216x216` px) with corresponding upscaled player panel typography (Title: 24, Artist/Album: 16).
- **Cover Art Lock**: Album art strictly follows the active playing track instead of changing to selected tracks.
- **Privacy-First Waybar Integration**: Includes a status/statistics script (`scripts/omatunes_text.py`) with all Last.fm network requests stripped. Fetches listening milestones, stats, and leaderboards locally.
- **Audio formats** — MP3, FLAC, OGG, Opus, WAV, AAC, M4A, AIFF and more via [Symphonia](https://github.com/pdeljanov/Symphonia).
- **MPRIS2** — full D-Bus integration; works with `playerctl`, Waybar, AGS, EWW, etc.

---

## Requirements

| Requirement | Notes |
|---|---|
| Rust ≥ 1.75 | `rustup` recommended |
| A Nerd Font | `JetBrainsMono Nerd Font Mono` by default; any Nerd Font works |
| PipeWire or PulseAudio | Audio output via cpal |
| D-Bus session bus | For MPRIS2 (`DBUS_SESSION_BUS_ADDRESS` must be set) |
| Wayland compositor | Tested on Hyprland; works on any wlroots compositor |

---

## Installation & Setup

### 1. Install the Player Binary (Compile from Source)
Clone the repository, compile the release binary, and copy it to your local path:
```bash
git clone https://github.com/Balthazzahr/omatunes
cd omatunes
cargo build --release
mkdir -p ~/.local/bin
cp target/release/omatunes ~/.local/bin/omatunes
```

### 2. Install Waybar Integration Scripts
To set up the Waybar module and stats dashboard, copy the scripts to your scripts folder and make them executable:
```bash
mkdir -p ~/.local/bin/omatunes_scripts
cp scripts/omatunes_text.py ~/.local/bin/omatunes_scripts/omatunes_text.py
cp scripts/omatunes_volume.sh ~/.local/bin/omatunes_scripts/omatunes_volume.sh
chmod +x ~/.local/bin/omatunes_scripts/omatunes_text.py
chmod +x ~/.local/bin/omatunes_scripts/omatunes_volume.sh
```

### 3. (Optional) Setup Auto-Sync Service
If you want to push local code edits automatically to your GitHub fork:
```bash
mkdir -p ~/.local/bin/omatunes_scripts
cp scripts/git_sync.sh ~/.local/bin/omatunes_scripts/git_sync.sh
chmod +x ~/.local/bin/omatunes_scripts/git_sync.sh

mkdir -p ~/.config/systemd/user
cp scripts/omatunes-sync.service ~/.config/systemd/user/omatunes-sync.service
systemctl --user daemon-reload
systemctl --user enable --now omatunes-sync.service
```

---

## Configuration

omatunes generates `~/.config/omatunes/config.toml` on first run. Edit it to configure paths and behaviors:

```toml
# ~/.config/omatunes/config.toml

# Path to your music library
music_dir = "~/Music"

# Initial volume (0.0 = mute, 1.0 = 100%)
volume = 0.8

# Start session with shuffle/repeat
shuffle = false
repeat = false

# Language ("auto", "en", "pt_BR", "es")
language = "auto"

# Seek / Volume steps
seek_step = 5
volume_step = 0.05
```

The library database is stored at `~/.local/share/omatunes/omatunes.db`. Delete this file to force a clean re-scan.

---

## Waybar Integration

Use the provided script under `/scripts/omatunes_text.py` for Waybar status and hover-tooltip details. 

To connect clicks, add custom commands to the module configuration in `~/.config/waybar/config.jsonc`:

```jsonc
"custom/omatunes": {
    "exec": "~/.local/bin/omatunes_scripts/omatunes_text.py",
    "return-type": "json",
    "format": "{text}",
    "on-click": "~/.local/bin/omatunes_scripts/omatunes_text.py --click left",
    "on-click-right": "~/.local/bin/omatunes_scripts/omatunes_text.py --click right",
    "on-click-middle": "~/.local/bin/omatunes_scripts/omatunes_text.py --click middle",
    "on-scroll-up": "~/.local/bin/omatunes_scripts/omatunes_volume.sh up",
    "on-scroll-down": "~/.local/bin/omatunes_scripts/omatunes_volume.sh down",
    "interval": 2
}
```

---

## Keybindings

These work when the omatunes window is focused:

| Key | Action |
|---|---|
| `Space` | Play / Pause |
| `→` / `←` | Seek +5s / −5s |
| `n` / `p` | Next / Previous track |
| `s` | Toggle Shuffle |
| `r` | Toggle Repeat |
| `+` or `=` | Volume +5% |
| `-` | Volume −5% |
| `E` | Edit metadata for selected tracks |

---

## Auto-Sync local changes to GitHub
A script is provided at `scripts/git_sync.sh` which watches the local codebase and automatically pushes updates to your GitHub repository in the background.

To activate, ensure your SSH key is added to GitHub, then run:
```bash
systemctl --user daemon-reload
systemctl --user enable --now omatunes-sync.service
```

---

## License

MIT
