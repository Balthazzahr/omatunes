<p align="center">
  <img src="assets/OmaTUNES Logo Transparent.png" alt="OmaTUNES Logo" width="400">
</p>

# omatunes

A native Wayland music player written in Rust, built for [Omarchy](https://omarchy.org/) / Hyprland rices. Follows the active Omarchy theme automatically — colors update live when you switch themes.

`omatunes` is a customized fork of [sheep-farm/lavanda](https://github.com/sheep-farm/lavanda) by [Balthazzahr](https://github.com/Balthazzahr).


<p align="center">
  <img src="assets/Main UI.png" alt="OmaTUNES Main UI Screenshot" width="800">
</p>

---

## Key Features

- **Wide Audio Format Support**: Plays MP3, FLAC, OGG, Opus, WAV, AAC, M4A, AIFF, and more natively via the high-performance [Symphonia](https://github.com/pdeljanov/Symphonia) library.
- **100% Offline & Privacy-First**: Zero tracking, zero background telemetry, and no network requirements. Logs and saves play counts, stats, and custom user playlists locally.
- **JSON Database**: Stores favorites, play counts, recently played tracks, hidden artists/albums, column settings, and user playlists in a single portable JSON file (`~/.config/omatunes/db.json`).
- **Synchronized LRC Lyrics & Interactivity**: Parses LRC metadata to highlight and auto-scroll the active lyric line, supporting interactive seek-on-click to any lyric's timestamp.
- **Logarithmic Audio Spectrum Visualizer**: Computes real-time 2048-point Hann-windowed FFT to render 64 logarithmic bands colored with an amplitude gradient.
- **Resizable Panel UI**: Toggle vertical tabs on the right side to open lyrics or spectrum views, adjustable with a click-and-drag handle and width state persistence.
- **Live Omarchy Theme Switching**: Automatically maps your active system theme (`Catppuccin`, `Nord`, `Gruvbox`, etc.) to the UI palette live in under 3 seconds. No app restarts required.
- **Native Wayland/X11 & Lightweight**: Built in native Rust using the Iced GUI toolkit. Runs on any Wayland compositor (Hyprland, GNOME, KDE) or traditional X11 window managers. Features extremely fast startup and low resource consumption.
- **Rich Waybar Integration**: Pre-packaged with local Waybar status scripts (`scripts/omatunes_text.py`) and a control group mapping play, next, and like controls. Provides styled progress bars, listening history milestones, and interactive tooltip stats.
- **Folder-Based Music Library**: No forced file re-organization. Respects and reads your existing music library subdirectories exactly as they are.
- **Advanced Bulk Metadata (ID3) Editing**: Select multiple tracks, edit fields selectively using checkboxes, utilize predictive library-based autocomplete suggestions, and apply tag updates across entire albums.
- **Customizable Columns**: Toggle visibilities or re-order columns (Track #, Title, Artist, Album, Genre, Year, Plays, Duration) via a right-click header menu, saving preferences to your local database.
- **Playlists & Smart Autoplaylists**: Build custom playlists on the fly, or use automatic smart lists (`Liked Songs`, `Recently Played`, `Most Played`) that update live as you listen.
- **MPRIS2 Server Support**: Integrates natively with `playerctl` and other system D-Bus audio widgets.

---

## 📖 User Manual 

<img width="512" height="512" alt="image" src="https://github.com/user-attachments/assets/c9d355f6-6f3f-4b16-96db-4aba15baf73e" />

For an in-depth reference covering all application features, including playback controls, keybindings, live lyrics scroll-seeking, database details, bulk ID3 metadata editing, and advanced Waybar integration styling, please see the [USER_MANUAL.md](USER_MANUAL.md).

---

## Requirements

| Requirement | Notes |
|---|---|
| Rust &geq; 1.75 | `rustup` recommended |
| A Nerd Font | `JetBrainsMono Nerd Font Mono` by default; any Nerd Font works |
| PipeWire or PulseAudio | Audio output via cpal |
| D-Bus session bus | For MPRIS2 (`DBUS_SESSION_BUS_ADDRESS` must be set) |
| Wayland or X11 | Tested on Hyprland; works on GNOME, KDE, and any standard Wayland/X11 window manager |

---

## Installation & Setup

### 1. Install the Player Binary

#### Option A: Download Pre-compiled Release (Recommended)
Download the pre-compiled binary directly from the latest GitHub release:
```bash
mkdir -p ~/.local/bin
curl -L -o ~/.local/bin/omatunes https://github.com/Balthazzahr/omatunes/releases/latest/download/omatunes
chmod +x ~/.local/bin/omatunes
```

#### Option B: Compile from Source
If you prefer to compile manually:
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

---

## Waybar Integration

The Waybar integration uses four discrete modules grouped together in `config.jsonc`. This provides specific button components for controlling playback while displaying active track details and listening history statistics.

Add the following to your `~/.config/waybar/config.jsonc` file:

```jsonc
  "modules-left": [
    ...
    "group/omatunes-group"
  ],

  "group/omatunes-group": {
    "orientation": "horizontal",
    "modules": [
      "custom/omatunes-play",
      "custom/omatunes-next",
      "custom/omatunes-text",
      "custom/omatunes-like"
    ]
  },

  "custom/omatunes-play": {
    "exec": "/home/user/.local/bin/omatunes_scripts/omatunes_text.py --button play",
    "interval": 1,
    "return-type": "json",
    "format": "{}",
    "on-click": "/home/user/.local/bin/omatunes_scripts/omatunes_text.py --click play"
  },
  "custom/omatunes-next": {
    "exec": "/home/user/.local/bin/omatunes_scripts/omatunes_text.py --button next",
    "interval": 1,
    "return-type": "json",
    "format": "{}",
    "on-click": "/home/user/.local/bin/omatunes_scripts/omatunes_text.py --click next"
  },
  "custom/omatunes-like": {
    "exec": "/home/user/.local/bin/omatunes_scripts/omatunes_text.py --button like",
    "interval": 1,
    "return-type": "json",
    "format": "{}",
    "on-click": "/home/user/.local/bin/omatunes_scripts/omatunes_text.py --click like"
  },
  "custom/omatunes-text": {
    "exec": "/home/user/.local/bin/omatunes_scripts/omatunes_text.py",
    "interval": 1,
    "return-type": "json",
    "format": "{}",
    "markup": "pango",
    "on-click": "hyprctl dispatch focuswindow class:^omatunes$ || hyprctl dispatch focuswindow title:^omatunes$",
    "on-scroll-up": "/home/user/.local/bin/omatunes_scripts/omatunes_volume.sh up",
    "on-scroll-down": "/home/user/.local/bin/omatunes_scripts/omatunes_volume.sh down",
    "tooltip": true
  }
```

For the CSS styling details to combine these modules into a single pill layout that collapses cleanly when OmaTunes is closed, refer to the **Waybar Integration** section in the [USER_MANUAL.md](USER_MANUAL.md).

---

## Keybindings

These work when the omatunes window is focused:

| Key | Action |
|---|---|
| `Space` | Play / Pause |
| `&rightarrow;` / `&leftarrow;` | Seek &plus;5s / &minus;5s |
| `n` / `p` | Next / Previous track |
| `s` | Toggle Shuffle |
| `r` | Toggle Repeat |
| `+` or `=` | Volume &plus;5% |
| `-` | Volume &minus;5% |
| `E` | Edit metadata for selected tracks |

For the complete list of keyboard shortcuts (including focus navigation), see the **Full Keybinding Reference** in the [USER_MANUAL.md](USER_MANUAL.md).

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
