<p align="center">
  <img src="assets/OmaTUNES Logo Transparent.png" alt="OmaTUNES Logo" width="400">
</p>

# omaTUNES

omaTUNES is a native Wayland music player and library manager built in Rust for Hyprland and Omarchy systems. omaTUNES is built to be as lightweight as possible while matching Omarchy's opinionated UI style — it picks up your system theme automatically, so your player always matches the rest of your desktop. omaTUNES is 100% offline, has wide support for most popular music codecs, and is designed to manage very large libraries with robust playlist and library management tools.

`omatunes` is a customized fork of [sheep-farm/lavanda](https://github.com/sheep-farm/lavanda) by [Balthazzahr](https://github.com/Balthazzahr).

<p align="center">
  <img src="assets/Main UI.png" alt="OmaTUNES Main UI Screenshot" width="800">
</p>

---

## Key Features

- **Wide Format Support.** MP3, FLAC, OGG, Opus, WAV, AAC, M4A, AIFF — all decoded natively through [Symphonia](https://github.com/pdeljanov/Symphonia), no plugins or codecs to hunt down.
- **Fully Offline & Private.** No telemetry, no accounts, no internet requirement to play a song. Your play counts, likes, and playlists live in one plain JSON file on disk, fully yours.
- **Smart Playlists.** Build iTunes-style rule-based playlists — mix criteria like artist, genre, play count, liked status, or "last played within 2 weeks" — and let them keep themselves up to date as your library changes. No manual curation required.
- **Customizable Library View.** Sort, filter, and browse by Artist, Album, or Genre. Drag columns into whatever order makes sense to you, group tracks dynamically by Album, Artist, Genre, or Year via a sleek fold-out capsule control, and let the table gracefully shrink its own columns as you resize the window instead of turning into a wall of wrapped text.
- **Playlist Management.** Drag songs into the order you want, drag whole playlists around in the sidebar to prioritize your favorites, and add a song to any playlist right from the player controls — no need to dig through menus mid-listen.
- **Synced Lyrics.** Synced LRC lyrics scroll and highlight in time with the track, and you can click any line to jump straight to that moment in the song.
- **Listening Statistics & Leaderboards.** Complete rewrite of listening stats — a toolbar button opens a dashboard modal with Day, Week, Month, and All-Time tabs. View three-column leaderboards (Artists, Albums, Genres) ranking your top 10 by minutes played, complete with gold, silver, and bronze highlights and custom tier icons. Drill down into any leaderboard item to open a sub-window displaying the top 100 songs by play count, and receive persistent milestone toasts for daily listening counts (25/50/100 tracks), artist plays (50/100/500/1000), and genre milestones.
- **Audio Visualizer & Motion Trails.** Real-time FFT spectrum analysis with 5 visually distinct canvas rendering modes (Mirrored Spectrograph, Radial Pulse, Liquid Silk Waveform Ribbon, Particle Constellation Starburst, and Hyperdrive Waterfall Depth Tunnel). Features customizable decaying ghost motion trails, audio reaction sensitivity, and dynamic HSL spectrum color shifting via the Visualizer Settings menu.
- **Theming System.** Follow your Omarchy system theme live, pick from built-in presets (Nord, Catppuccin, Dracula, Gruvbox, Everforest, Monokai), or build your own — omaTUNES derives the supporting shades automatically using proper WCAG contrast math, so your custom theme never ends up with unreadable text.
- **Bulk Metadata Editing.** Select a stack of tracks, check only the fields you want to change, and apply edits across an entire album in one go — with autocomplete pulled straight from your existing library tags.
- **Desktop Integration.** Full MPRIS2 support and a ready-to-go Waybar module with playback controls, live track info, and listening-history stats baked right into your bar.
- **Non-Destructive Library Handling.** No forced re-organization, no renaming your folders, no importing into some walled-off library format. omaTUNES reads your music exactly where it already lives.

---

## User Manual

The [USER_MANUAL.md](USER_MANUAL.md) covers everything in detail — every keybinding, every menu, how the database is structured, and how to get the Waybar integration looking sharp.

---

## System Requirements

| Requirement | Notes |
|---|---|
| Rust ≥ 1.75 | `rustup` is the easiest way to get it |
| A Nerd Font | Ships configured for `JetBrainsMono Nerd Font Mono`, but any Nerd Font works |
| PipeWire or PulseAudio | Audio output runs through cpal |
| D-Bus session bus | Needed for MPRIS2 — make sure `DBUS_SESSION_BUS_ADDRESS` is set |
| Wayland or X11 | Built and tested on Hyprland, but plays nicely on GNOME, KDE, and standard X11 window managers too |

---

## Install Instructions

### 1. Get the binary

**Easiest: grab a pre-built release.**
```bash
mkdir -p ~/.local/bin
curl -L -o ~/.local/bin/omatunes https://github.com/Balthazzahr/omatunes/releases/latest/download/omatunes
chmod +x ~/.local/bin/omatunes
```

**Prefer to build it yourself?**
```bash
git clone https://github.com/Balthazzahr/omatunes
cd omatunes
cargo build --release
mkdir -p ~/.local/bin
cp target/release/omatunes ~/.local/bin/omatunes
```

### 2. Wire up the Waybar module (optional, but worth it)

If you want the Waybar integration — playback controls, track info, and listening stats right in your bar — copy the scripts over and make them executable:

```bash
mkdir -p ~/.local/bin/omatunes_scripts
cp scripts/omatunes_text.py ~/.local/bin/omatunes_scripts/omatunes_text.py
cp scripts/omatunes_volume.sh ~/.local/bin/omatunes_scripts/omatunes_volume.sh
chmod +x ~/.local/bin/omatunes_scripts/omatunes_text.py
chmod +x ~/.local/bin/omatunes_scripts/omatunes_volume.sh
```

Full Waybar config below, and CSS styling details in the [User Manual](USER_MANUAL.md#waybar-integration).

---

## Configuration and Settings

omaTUNES writes out `~/.config/omatunes/config.toml` the first time you run it. Open it up and point it at your music:

```toml
# ~/.config/omatunes/config.toml

# Path to your music library
music_dir = "~/Music"

# Initial volume (0.0 = mute, 1.0 = 100%)
volume = 0.8

# Start session with shuffle/repeat
shuffle = false
repeat = false

# Seek / Volume steps
seek_step = 5
volume_step = 0.05

# Scale factor for UI text sizes (float)
font_scale = 1.0

# Theming source: "System", "Preset", or "Custom"
theme_source = "Preset"

# Selected built-in preset theme name
theme_preset = "Nord"

# Custom theme colors (used when theme_source = "Custom")
[custom_theme]
base = "#1e1e2e"       # Background
text = "#cdd6f4"       # Primary Text
accent = "#cba6f7"     # Accent
green = "#a6e3a1"      # Green highlight
red = "#f38ba8"        # Red highlight
yellow = "#f9e2af"     # Yellow highlight
blue = "#89b4fa"       # Blue highlight
# mantle, surface0, overlay0, and subtext are derived automatically from
# the colors above using WCAG contrast targets, and get written back to
# this file when you save your theme in-app.
```

Most of this is also editable straight from the in-app Settings panel — you don't need to touch this file by hand unless you want to.

---

## Waybar Integration

The Waybar setup uses a single `custom/omatunes` module that shows a music note icon, artist, and
track name in the bar, with a tooltip summarising the track and your available mouse controls.

Add this to `~/.config/waybar/config.jsonc`:

```jsonc
  "modules-left": [
    ...
    "custom/omatunes"
  ],

  "custom/omatunes": {
    "exec": "/home/yourname/.local/bin/omatunes_scripts/omatunes_text.py",
    "interval": 1,
    "return-type": "json",
    "format": "{}",
    "markup": "pango",
    "tooltip": true,
    "on-click": "/home/yourname/.local/bin/omatunes_scripts/omatunes_text.py --click play",
    "on-click-middle": "/home/yourname/.local/bin/omatunes_scripts/omatunes_text.py --click like",
    "on-click-right": "/home/yourname/.local/bin/omatunes_scripts/omatunes_text.py --click next",
    "on-scroll-up": "/home/yourname/.local/bin/omatunes_scripts/omatunes_volume.sh up",
    "on-scroll-down": "/home/yourname/.local/bin/omatunes_scripts/omatunes_volume.sh down"
  }
```

> Replace `/home/yourname/` with your actual home path — Waybar requires fully expanded paths.

For CSS styling, see the [Waybar Integration section](USER_MANUAL.md#waybar-integration) in the User Manual.

---

## Keybindings

These fire whenever the omaTUNES window has focus:

| Key | Action |
|---|---|
| `Space` | Play / Pause |
| `→` / `←` | Seek +5s / −5s |
| `n` / `p` | Next / Previous track |
| `s` | Toggle Shuffle |
| `r` | Toggle Repeat |
| `+` or `=` | Volume +5% |
| `-` | Volume −5% |
| `E` | Edit metadata for the selected track(s) |

The full list — including focus navigation and dialog shortcuts — is in the [Keybinding Reference](USER_MANUAL.md#keybinding-reference) in the User Manual.

---

## License

MIT
