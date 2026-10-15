use std::path::PathBuf;
use std::sync::{OnceLock, Mutex};

use serde::{Deserialize, Serialize};

static CONFIG: OnceLock<Mutex<Config>> = OnceLock::new();

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(default)]
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
#[serde(default)]
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
#[serde(default)]
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
#[serde(default)]
pub struct AutoScanConfig {
    pub mode: String,
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
    pub playback_defaults: PlaybackDefaults,
    pub auto_scan: AutoScanConfig,
    pub show_achievements_in_ui: bool,
    pub show_toasts: bool,
    pub visualizer_mode: usize,
    pub visualizer_sensitivity: f32,
    pub ghost_trail_length: usize,
    pub ghost_decay: f32,
    pub color_shift_speed: f32,
    pub spectrograph_bar_count: usize,
    pub visualizer_bg_mode: usize, // 0 = Theme, 1 = Custom RGB, 2 = Muted Reactive
    pub visualizer_bg_color: String,
    pub aurora_preset: usize, // 0 = Emerald Glow, 1 = Solar Flare, 2 = Polar Pink & Blue
    pub depth_warp_speed: f32,
    pub kaleidoscope_axes: usize,
    pub font_family: String,
    /// Ordered font fallback chain. Empty means "auto-discover".
    pub font_fallback: Vec<String>,
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
            font_family: default_font_family(),
            font_fallback: Vec::new(),
            theme_source: "System".into(),
            theme_preset: "Nord".into(),
            custom_theme: None,
            playback_defaults: PlaybackDefaults::default(),
            auto_scan: AutoScanConfig::default(),
            show_achievements_in_ui: true,
            show_toasts: true,
            visualizer_mode: 0,
            visualizer_sensitivity: 1.0,
            ghost_trail_length: 8,
            ghost_decay: 0.45,
            color_shift_speed: 1.0,
            spectrograph_bar_count: 64,
            visualizer_bg_mode: 0,
            visualizer_bg_color: "#0f0f1a".into(),
            aurora_preset: 0,
            depth_warp_speed: 1.0,
            kaleidoscope_axes: 8,
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

/// Borrow the font settings without cloning the whole `Config`.
///
/// `Config` has 26 fields and 5 heap allocations, so calling [`get`] per
/// rendered text element allocates every frame. The font resolver runs in
/// exactly that path, so it borrows instead.
pub fn with_font_settings<R>(f: impl FnOnce(&str, &[String]) -> R) -> R {
    let guard = CONFIG.get_or_init(|| Mutex::new(read_or_default())).lock().unwrap();
    f(&guard.font_family, &guard.font_fallback)
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
    save(current)
}

pub fn update_font_family(family: &str) {
    let mut current = get();
    current.font_family = family.to_string();
    save(current)
}

pub fn update_font_fallback(chain: &[String]) {
    let mut current = get();
    current.font_fallback = chain.to_vec();
    save(current)
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
# font_family = "JetBrainsMono Nerd Font Mono"

# Ordered font fallback chain, first match wins. Empty = auto-detect.
# font_fallback = ["Noto Sans CJK JP", "Noto Color Emoji"]
"#;

fn default_font_family() -> String {
    "JetBrainsMono Nerd Font Mono".to_string()
}

#[cfg(test)]
mod tests {
    use super::Config;

    /// A pre-existing `~/.config/omatunes/config.toml` has no `font_fallback`
    /// key. It must still deserialize, yielding an empty (auto-discover) chain.
    #[test]
    fn legacy_config_without_font_fallback_still_parses() {
        let legacy = r#"
music_dir = "~/Music"
volume = 0.8
language = "auto"
seek_step = 5
volume_step = 0.05
theme_source = "System"
theme_preset = "Nord"
font_family = "JetBrainsMono Nerd Font Mono"
"#;

        let cfg: Config = toml::from_str(legacy).expect("legacy config must deserialize");
        assert_eq!(cfg.font_family, "JetBrainsMono Nerd Font Mono");
        assert!(cfg.font_fallback.is_empty());
        assert_eq!(cfg.ghost_trail_length, 8);
        assert_eq!(cfg.spectrograph_bar_count, 64);
        assert_eq!(cfg.visualizer_sensitivity, 1.0);
        assert_eq!(cfg.auto_scan.mode, "manual");
        assert_eq!(cfg.auto_scan.interval_minutes, 15);
    }

    /// An explicit chain round-trips through serialize/deserialize.
    #[test]
    fn font_fallback_round_trips() {
        let chain = vec!["Noto Sans CJK JP".to_string(), "Noto Color Emoji".to_string()];
        let mut cfg = Config::default();
        cfg.font_fallback = chain.clone();

        let encoded = toml::to_string_pretty(&cfg).expect("config serializes");
        assert!(encoded.contains("font_fallback"), "encoded config: {encoded}");

        let decoded: Config = toml::from_str(&encoded).expect("config deserializes");
        assert_eq!(decoded.font_fallback, chain);
    }

    #[test]
    fn partial_auto_scan_deserialization_defaults() {
        let toml_str = r#"
music_dir = "~/Music"
[auto_scan]
mode = "periodic"
"#;
        let cfg: Config = toml::from_str(toml_str).expect("config must parse");
        assert_eq!(cfg.auto_scan.mode, "periodic");
        assert_eq!(cfg.auto_scan.interval_minutes, 15);
    }

    #[test]
    fn partial_custom_theme_defaults() {
        let toml_str = r##"
music_dir = "~/Music"
[custom_theme]
accent = "#ff007f"
"##;
        let cfg: Config = toml::from_str(toml_str).expect("config must parse");
        let ct = cfg.custom_theme.expect("custom theme should be present");
        assert_eq!(ct.accent, "#ff007f");
        assert_eq!(ct.base, "#11111b");
        assert_eq!(ct.text, "#cdd6f4");
    }
}
