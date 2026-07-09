use std::path::PathBuf;
use std::sync::{OnceLock, Mutex};

use serde::{Deserialize, Serialize};

static CONFIG: OnceLock<Mutex<Config>> = OnceLock::new();

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct CustomThemeConfig {
    pub base: String,
    pub mantle: String,
    pub surface0: String,
    pub overlay0: String,
    pub text: String,
    pub subtext: String,
    pub accent: String,
    pub green: String,
    pub red: String,
    pub yellow: String,
    pub blue: String,
}

impl Default for CustomThemeConfig {
    fn default() -> Self {
        CustomThemeConfig {
            base:     "#11111b".into(),
            mantle:   "#181825".into(),
            surface0: "#313244".into(),
            overlay0: "#6c7086".into(),
            text:     "#cdd6f4".into(),
            subtext:  "#a6adc8".into(),
            accent:   "#cba6f7".into(),
            green:    "#a6e3a1".into(),
            red:      "#f38ba8".into(),
            yellow:   "#f9e2af".into(),
            blue:     "#89b4fa".into(),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PlaybackDefault {
    pub shuffle: bool,
    pub repeat: bool,
}

impl Default for PlaybackDefault {
    fn default() -> Self {
        Self { shuffle: false, repeat: false }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PlaybackDefaults {
    pub album: PlaybackDefault,
    pub artist: PlaybackDefault,
    pub genre: PlaybackDefault,
    pub user_playlist: PlaybackDefault,
    pub smart_playlist: PlaybackDefault,
}

impl Default for PlaybackDefaults {
    fn default() -> Self {
        Self {
            album: PlaybackDefault { shuffle: false, repeat: false },
            artist: PlaybackDefault { shuffle: true, repeat: false },
            genre: PlaybackDefault { shuffle: true, repeat: false },
            user_playlist: PlaybackDefault { shuffle: false, repeat: false },
            smart_playlist: PlaybackDefault { shuffle: false, repeat: false },
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AutoScanConfig {
    #[serde(default)]
    pub mode: String,
    #[serde(default)]
    pub interval_minutes: u64,
}

impl Default for AutoScanConfig {
    fn default() -> Self {
        Self { mode: "manual".into(), interval_minutes: 15 }
    }
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct Config {
    pub music_dir:   String,
    pub volume:      f32,
    pub language:    String,
    pub seek_step:   u64,
    pub volume_step: f32,
    pub font_scale:  Option<f32>,
    pub theme_source: String,
    pub theme_preset: String,
    pub custom_theme: Option<CustomThemeConfig>,
    #[serde(default)]
    pub playback_defaults: PlaybackDefaults,
    #[serde(default)]
    pub auto_scan: AutoScanConfig,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            music_dir:   "~/Music".into(),
            volume:      0.8,
            language:    "auto".into(),
            seek_step:   5,
            volume_step: 0.05,
            font_scale:  Some(1.0),
            theme_source: "System".into(),
            theme_preset: "Nord".into(),
            custom_theme: None,
            playback_defaults: PlaybackDefaults::default(),
            auto_scan: AutoScanConfig::default(),
        }
    }
}

impl Config {
    pub fn font_scale(&self) -> f32 {
        self.font_scale.unwrap_or(1.0)
    }
    /// Returns `music_dir` with `~` expanded to `$HOME`.
    pub fn music_path(&self) -> PathBuf {
        expand_tilde(&self.music_dir)
    }
}

// ── Initialization ─────────────────────────────────────────────────────────────

pub fn load() {
    CONFIG.get_or_init(|| Mutex::new(read_or_default()));
}

pub fn get() -> Config {
    let guard = CONFIG.get_or_init(|| Mutex::new(read_or_default())).lock().unwrap();
    guard.clone()
}

pub fn save(cfg: Config) {
    let mut guard = CONFIG.get_or_init(|| Mutex::new(read_or_default())).lock().unwrap();
    *guard = cfg.clone();
    if let Ok(toml_str) = toml::to_string_pretty(&cfg) {
        let path = config_path();
        std::fs::write(path, toml_str).ok();
    }
}

pub fn update_font_scale(scale: f32) {
    let mut current = get();
    current.font_scale = Some(scale);
    save(current);
}

fn read_or_default() -> Config {
    let path = config_path();

    if !path.exists() {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).ok();
        }
        std::fs::write(&path, DEFAULT_CONFIG).ok();
        eprintln!("omatunes: config created at {}", path.display());
        return Config::default();
    }

    let content = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("omatunes: error reading config: {e}");
            return Config::default();
        }
    };

    match toml::from_str::<Config>(&content) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("omatunes: invalid config ({e}), using defaults");
            Config::default()
        }
    }
}

fn config_path() -> PathBuf {
    crate::paths::config_toml()
}

fn expand_tilde(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        PathBuf::from(home).join(&path[2..])
    } else {
        PathBuf::from(path)
    }
}

// ── Default config generated on first run ─────────────────────────────────────

const DEFAULT_CONFIG: &str = r#"# omatunes — configuration file
# ~/.config/omatunes/config.toml
#
# All fields are optional. Missing fields use the defaults shown here.

# Path to your music library. Subdirectories are shown as folders in the sidebar.
music_dir = "~/Music"

# Initial volume (0.0 = mute, 1.0 = 100%)
volume = 0.8

# Interface language. Options: "auto", "en", "pt_BR", "es"
# "auto" detects from $LANG
language = "auto"

# Seek step in seconds for the ← → arrow keys
seek_step = 5

# Volume delta per + / - keypress
volume_step = 0.05

# UI font size scale multiplier (default: 1.0)
# font_scale = 1.0

# Theme source: "System", "Preset", or "Custom"
theme_source = "System"

# Theme preset (used when theme_source = "Preset")
theme_preset = "Nord"
"#;
