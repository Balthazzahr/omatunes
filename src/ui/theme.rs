use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

use iced::widget::container;
use iced::{Border, Color};

// ── Palette ──────────────────────────────────────────────────────────────────

static PALETTE: OnceLock<Mutex<Palette>> = OnceLock::new();
static THEME_LOAD_FAILED: AtomicBool = AtomicBool::new(false);

pub fn did_theme_load_fail() -> bool {
    THEME_LOAD_FAILED.load(Ordering::Relaxed)
}

#[derive(Clone, Copy, Debug)]
pub struct Palette {
    pub base:     Color,
    pub mantle:   Color,
    pub surface0: Color,
    pub overlay0: Color,
    pub text:     Color,
    pub subtext:  Color,
    pub accent:   Color,
    pub green:    Color,
    pub red:      Color,
    pub yellow:   Color,
    pub blue:     Color,
}

impl Palette {
    pub fn default_lavender() -> Self {
        Palette {
            base:     hex(0x11, 0x11, 0x1b),
            mantle:   hex(0x18, 0x18, 0x25),
            surface0: hex(0x31, 0x32, 0x44),
            overlay0: hex(0x6c, 0x70, 0x86),
            text:     hex(0xcd, 0xd6, 0xf4),
            subtext:  hex(0xa6, 0xad, 0xc8),
            accent:   hex(0xcb, 0xa6, 0xf7),
            green:    hex(0xa6, 0xe3, 0xa1),
            red:      hex(0xf3, 0x8b, 0xa8),
            yellow:   hex(0xf9, 0xe2, 0xaf),
            blue:     hex(0x89, 0xb4, 0xfa),
        }
    }
}

pub fn get_preset_palette(name: &str) -> Option<Palette> {
    match name {
        "Nord" => Some(Palette {
            base:     hex_to_color("#2e3440").unwrap(),
            mantle:   hex_to_color("#242933").unwrap(),
            surface0: hex_to_color("#3b4252").unwrap(),
            overlay0: hex_to_color("#4c566a").unwrap(),
            text:     hex_to_color("#eceff4").unwrap(),
            subtext:  hex_to_color("#d8dee9").unwrap(),
            accent:   hex_to_color("#88c0d0").unwrap(),
            green:    hex_to_color("#a3be8c").unwrap(),
            red:      hex_to_color("#bf616a").unwrap(),
            yellow:   hex_to_color("#ebcb8b").unwrap(),
            blue:     hex_to_color("#81a1c1").unwrap(),
        }),
        "Catppuccin Mocha" => Some(Palette::default_lavender()),
        "Catppuccin Latte" => Some(Palette {
            base:     hex_to_color("#eff1f5").unwrap(),
            mantle:   hex_to_color("#e6e9ef").unwrap(),
            surface0: hex_to_color("#ccd0da").unwrap(),
            overlay0: hex_to_color("#9ca0b0").unwrap(),
            text:     hex_to_color("#4c4f69").unwrap(),
            subtext:  hex_to_color("#5c5f77").unwrap(),
            accent:   hex_to_color("#8839ef").unwrap(),
            green:    hex_to_color("#40a02b").unwrap(),
            red:      hex_to_color("#d20f39").unwrap(),
            yellow:   hex_to_color("#df8e1d").unwrap(),
            blue:     hex_to_color("#1e66f5").unwrap(),
        }),
        "Dracula" => Some(Palette {
            base:     hex_to_color("#282a36").unwrap(),
            mantle:   hex_to_color("#1e1f29").unwrap(),
            surface0: hex_to_color("#44475a").unwrap(),
            overlay0: hex_to_color("#6272a4").unwrap(),
            text:     hex_to_color("#f8f8f2").unwrap(),
            subtext:  hex_to_color("#a4b9ef").unwrap(),
            accent:   hex_to_color("#bd93f9").unwrap(),
            green:    hex_to_color("#50fa7b").unwrap(),
            red:      hex_to_color("#ff5555").unwrap(),
            yellow:   hex_to_color("#f1fa8c").unwrap(),
            blue:     hex_to_color("#8be9fd").unwrap(),
        }),
        "Gruvbox (Dark)" => Some(Palette {
            base:     hex_to_color("#282828").unwrap(),
            mantle:   hex_to_color("#1d2021").unwrap(),
            surface0: hex_to_color("#3c3836").unwrap(),
            overlay0: hex_to_color("#7c6f64").unwrap(),
            text:     hex_to_color("#ebdbb2").unwrap(),
            subtext:  hex_to_color("#a89984").unwrap(),
            accent:   hex_to_color("#fe8019").unwrap(),
            green:    hex_to_color("#b8bb26").unwrap(),
            red:      hex_to_color("#fb4934").unwrap(),
            yellow:   hex_to_color("#fabd2f").unwrap(),
            blue:     hex_to_color("#83a598").unwrap(),
        }),
        "Everforest (Dark)" => Some(Palette {
            base:     hex_to_color("#2d353b").unwrap(),
            mantle:   hex_to_color("#232a2e").unwrap(),
            surface0: hex_to_color("#3d484d").unwrap(),
            overlay0: hex_to_color("#859289").unwrap(),
            text:     hex_to_color("#d3c6aa").unwrap(),
            subtext:  hex_to_color("#9da9a0").unwrap(),
            accent:   hex_to_color("#a7c080").unwrap(),
            green:    hex_to_color("#8db573").unwrap(),
            red:      hex_to_color("#e67e80").unwrap(),
            yellow:   hex_to_color("#dbbc7f").unwrap(),
            blue:     hex_to_color("#7fbbb3").unwrap(),
        }),
        "Monokai" => Some(Palette {
            base:     hex_to_color("#272822").unwrap(),
            mantle:   hex_to_color("#1e1f1c").unwrap(),
            surface0: hex_to_color("#3e3d32").unwrap(),
            overlay0: hex_to_color("#75715e").unwrap(),
            text:     hex_to_color("#f8f8f2").unwrap(),
            subtext:  hex_to_color("#a59f85").unwrap(),
            accent:   hex_to_color("#f92672").unwrap(),
            green:    hex_to_color("#a6e22e").unwrap(),
            red:      hex_to_color("#f92672").unwrap(),
            yellow:   hex_to_color("#e6db74").unwrap(),
            blue:     hex_to_color("#66d9ef").unwrap(),
        }),
        _ => None,
    }
}

pub fn hex_to_color(s: &str) -> Option<Color> {
    let clean = s.trim().trim_start_matches('#');
    if clean.len() != 6 { return None; }
    let r = u8::from_str_radix(&clean[0..2], 16).ok()?;
    let g = u8::from_str_radix(&clean[2..4], 16).ok()?;
    let b = u8::from_str_radix(&clean[4..6], 16).ok()?;
    Some(hex(r, g, b))
}

pub fn color_to_hex(c: Color) -> String {
    let r = (c.r * 255.0).round() as u8;
    let g = (c.g * 255.0).round() as u8;
    let b = (c.b * 255.0).round() as u8;
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

fn palette_mutex() -> &'static Mutex<Palette> {
    PALETTE.get_or_init(|| {
        Mutex::new(load_palette_from_config())
    })
}

pub fn load_palette_from_config() -> Palette {
    let cfg = crate::config::get();
    match cfg.theme_source.as_str() {
        "Preset" => {
            get_preset_palette(&cfg.theme_preset).unwrap_or_else(|| {
                get_preset_palette("Nord").unwrap()
            })
        }
        "Custom" => {
            if let Some(ref custom) = cfg.custom_theme {
                let base = hex_to_color(&custom.base).unwrap_or_else(|| hex(0x11, 0x11, 0x1b));
                let text = hex_to_color(&custom.text).unwrap_or_else(|| hex(0xcd, 0xd6, 0xf4));
                let is_dark = luminance(base) < 0.5;
                
                let mantle = derive_mantle(base, is_dark);
                let surface0 = derive_surface0(base, is_dark);
                let overlay0 = derive_overlay0(base, is_dark);
                let subtext = derive_subtext(text, is_dark);

                Palette {
                    base,
                    mantle,
                    surface0,
                    overlay0,
                    text,
                    subtext,
                    accent:   hex_to_color(&custom.accent).unwrap_or_else(|| hex(0xcb, 0xa6, 0xf7)),
                    green:    hex_to_color(&custom.green).unwrap_or_else(|| hex(0xa6, 0xe3, 0xa1)),
                    red:      hex_to_color(&custom.red).unwrap_or_else(|| hex(0xf3, 0x8b, 0xa8)),
                    yellow:   hex_to_color(&custom.yellow).unwrap_or_else(|| hex(0xf9, 0xe2, 0xaf)),
                    blue:     hex_to_color(&custom.blue).unwrap_or_else(|| hex(0x89, 0xb4, 0xfa)),
                }
            } else {
                Palette::default_lavender()
            }
        }
        _ => { // "System"
            match try_load_omarchy_theme() {
                Some(p) => {
                    THEME_LOAD_FAILED.store(false, Ordering::Relaxed);
                    p
                }
                None => {
                    THEME_LOAD_FAILED.store(true, Ordering::Relaxed);
                    eprintln!("omatunes: Omarchy theme not found at ~/.local/state/omarchy/current/theme/colors.toml, using default lavender — check omarchy theme current and file permissions");
                    Palette::default_lavender()
                }
            }
        }
    }
}

/// Initialize palette on first run.
pub fn load_system_theme() {
    let _ = palette_mutex();
}

pub fn apply_palette(new_palette: Palette) {
    *palette_mutex().lock().unwrap() = new_palette;
}

/// Re-read the Omarchy theme from disk and update the palette without restarting.
pub fn reload_system_theme() {
    *palette_mutex().lock().unwrap() = load_palette_from_config();
}

fn get_homes() -> Vec<PathBuf> {
    let mut homes = Vec::new();
    if let Some(h) = home_dir() {
        homes.push(h);
    }
    if let Ok(rh) = std::env::var("REAL_HOME") {
        let p = PathBuf::from(rh);
        if !homes.contains(&p) {
            homes.push(p);
        }
    }
    homes
}

/// Return the name of the currently configured Omarchy theme (for change detection).
pub fn read_current_theme_name() -> String {
    let homes = get_homes();
    for home in &homes {
        let state_home = std::env::var("XDG_STATE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home.join(".local/state"));
        if let Ok(name) = std::fs::read_to_string(state_home.join("omarchy/current/theme.name"))
            .or_else(|_| std::fs::read_to_string(home.join(".config/omarchy/current/theme.name")))
        {
            let trimmed = name.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }
    String::new()
}

fn try_load_omarchy_theme() -> Option<Palette> {
    let homes = get_homes();
    let theme_name_raw = read_current_theme_name();
    let theme_name = if theme_name_raw.is_empty() {
        "current".to_string()
    } else {
        theme_name_raw.to_lowercase()
    };

    let mut candidate_paths = Vec::new();
    for home in &homes {
        let state_home = std::env::var("XDG_STATE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home.join(".local/state"));
        candidate_paths.push(state_home.join("omarchy/current/theme/colors.toml"));
        candidate_paths.push(home.join(".config/omarchy/current/theme/colors.toml"));
        candidate_paths.push(home.join(format!(".config/omarchy/themes/{}/colors.toml", theme_name)));
        candidate_paths.push(home.join(format!(".local/share/omarchy/themes/{}/colors.toml", theme_name)));
    }
    candidate_paths.push(PathBuf::from(format!("/usr/share/omarchy/themes/{}/colors.toml", theme_name)));

    for path in candidate_paths {
        if let Ok(content) = std::fs::read_to_string(&path) {
            eprintln!("omatunes: loading theme \"{}\" from {}", theme_name, path.display());
            if let Some(pal) = parse_colors_toml(&content) {
                return Some(pal);
            }
        }
    }
    None
}

fn parse_colors_toml(content: &str) -> Option<Palette> {
    let mut map: HashMap<String, Color> = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }

        let Some((key, val)) = line.split_once('=') else { continue };
        let key = key.trim().to_string();
        let val = val.trim();

        let hex_str = if let Some(pos) = val.find('#') {
            let after = &val[pos + 1..];
            let end = after.find(|c: char| !c.is_ascii_hexdigit()).unwrap_or(after.len());
            after[..end].to_string()
        } else {
            val.trim_matches('"').to_string()
        };

        if let Some(c) = parse_hex_str(&hex_str) {
            map.insert(key, c);
        }
    }

    let bg     = *map.get("background")?;
    let fg     = *map.get("foreground")?;
    let accent = *map.get("accent")?;

    let c8 = map.get("color8")
        .or_else(|| map.get("muted"))
        .or_else(|| map.get("selection"))
        .copied()
        .unwrap_or_else(|| lerp_color(bg, fg, 0.3));

    let is_dark = luminance(bg) < 0.5;
    let (mantle, surface0) = if is_dark {
        (
            map.get("dark_background")
                .or_else(|| map.get("darker_background"))
                .copied()
                .unwrap_or_else(|| lerp_color(bg, c8, 0.10)),
            map.get("lighter_background")
                .copied()
                .unwrap_or_else(|| lerp_color(bg, c8, 0.40)),
        )
    } else {
        (lerp_color(bg, fg, 0.05), lerp_color(bg, fg, 0.18))
    };

    Some(Palette {
        base: bg,
        mantle,
        surface0,
        overlay0: c8,
        text: fg,
        subtext: map.get("color15")
            .or_else(|| map.get("light_foreground"))
            .copied()
            .unwrap_or_else(|| lerp_color(fg, c8, 0.3)),
        accent,
        red:    map.get("color1").or_else(|| map.get("red")).copied().unwrap_or_else(|| hex(0xf3, 0x8b, 0xa8)),
        green:  map.get("color2").or_else(|| map.get("green")).copied().unwrap_or_else(|| hex(0xa6, 0xe3, 0xa1)),
        yellow: map.get("color3").or_else(|| map.get("yellow")).copied().unwrap_or_else(|| hex(0xf9, 0xe2, 0xaf)),
        blue:   map.get("color4").or_else(|| map.get("blue")).copied().unwrap_or_else(|| hex(0x89, 0xb4, 0xfa)),
    })
}

fn parse_hex_str(s: &str) -> Option<Color> {
    if s.len() == 3 {
        let r = u8::from_str_radix(&s[0..1], 16).ok()?;
        let g = u8::from_str_radix(&s[1..2], 16).ok()?;
        let b = u8::from_str_radix(&s[2..3], 16).ok()?;
        return Some(hex(r * 17, g * 17, b * 17));
    }
    if s.len() >= 6 {
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        return Some(hex(r, g, b));
    }
    None
}

pub fn luminance(c: Color) -> f32 {
    0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b
}

pub fn contrast_ratio(c1: Color, c2: Color) -> f32 {
    let l1 = luminance(c1);
    let l2 = luminance(c2);
    if l1 > l2 {
        (l1 + 0.05) / (l2 + 0.05)
    } else {
        (l2 + 0.05) / (l1 + 0.05)
    }
}

pub fn derive_color_at_contrast_ratio(source: Color, target_ratio: f32, make_lighter: bool) -> Color {
    let l_s = luminance(source);
    let l_d = if make_lighter {
        target_ratio * (l_s + 0.05) - 0.05
    } else {
        (l_s + 0.05) / target_ratio - 0.05
    };
    let l_d = l_d.clamp(0.0, 1.0);
    
    if make_lighter {
        let denom = 1.0 - l_s;
        let t = if denom.abs() < 1e-5 { 0.0 } else { ((l_d - l_s) / denom).clamp(0.0, 1.0) };
        Color {
            r: source.r + t * (1.0 - source.r),
            g: source.g + t * (1.0 - source.g),
            b: source.b + t * (1.0 - source.b),
            a: source.a,
        }
    } else {
        let t = if l_s.abs() < 1e-5 { 0.0 } else { (1.0 - l_d / l_s).clamp(0.0, 1.0) };
        Color {
            r: source.r * (1.0 - t),
            g: source.g * (1.0 - t),
            b: source.b * (1.0 - t),
            a: source.a,
        }
    }
}

pub fn derive_mantle(base: Color, is_dark: bool) -> Color {
    // Mantle target contrast is 1.20
    // applied in direction that makes Mantle slightly closer to black in dark / closer to white in light
    derive_color_at_contrast_ratio(base, 1.20, !is_dark)
}

pub fn derive_surface0(base: Color, is_dark: bool) -> Color {
    // Surface0 target contrast is 1.40
    // applied as structural highlight: lighter than Base in dark / darker than Base in light
    derive_color_at_contrast_ratio(base, 1.40, is_dark)
}

pub fn derive_overlay0(base: Color, is_dark: bool) -> Color {
    // Overlay0 target contrast is 2.80
    // applied as content-adjacent: lighter than Base in dark / darker than Base in light
    derive_color_at_contrast_ratio(base, 2.80, is_dark)
}

pub fn derive_subtext(text: Color, is_dark: bool) -> Color {
    // Subtext target contrast is 1.25 against Text
    derive_color_at_contrast_ratio(text, 1.25, !is_dark)
}

pub fn check_custom_contrast_warnings(custom: &crate::config::CustomThemeConfig) -> Vec<(String, f32, f32)> {
    let mut warnings = Vec::new();
    let base = hex_to_color(&custom.base);
    let surface0 = hex_to_color(&custom.surface0);
    let text = hex_to_color(&custom.text);
    let subtext = hex_to_color(&custom.subtext);
    let accent = hex_to_color(&custom.accent);

    if let (Some(b), Some(t)) = (base, text) {
        let cr = contrast_ratio(b, t);
        if cr < 4.5 {
            warnings.push(("Text vs Base".to_string(), cr, 4.5));
        }
    }
    if let (Some(b), Some(s)) = (base, subtext) {
        let cr = contrast_ratio(b, s);
        if cr < 3.0 {
            warnings.push(("Subtext vs Base".to_string(), cr, 3.0));
        }
    }
    if let (Some(b), Some(a)) = (base, accent) {
        let cr = contrast_ratio(b, a);
        if cr < 3.0 {
            warnings.push(("Accent vs Base".to_string(), cr, 3.0));
        }
    }
    if let (Some(s0), Some(t)) = (surface0, text) {
        let cr = contrast_ratio(s0, t);
        if cr < 3.0 {
            warnings.push(("Text vs Surface0".to_string(), cr, 3.0));
        }
    }
    warnings
}

fn home_dir() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
}

// ── Color accessors ──────────────────────────────────────────────────────────

macro_rules! color_fn {
    ($name:ident, $field:ident) => {
        pub fn $name() -> Color { palette_mutex().lock().unwrap().$field }
    };
}

color_fn!(base,     base);
color_fn!(mantle,   mantle);
color_fn!(surface0, surface0);
color_fn!(overlay0, overlay0);
color_fn!(text,     text);
color_fn!(subtext,  subtext);
color_fn!(accent,   accent);
color_fn!(green,    green);
color_fn!(red,      red);
color_fn!(yellow,   yellow);
color_fn!(blue,     blue);

// ── Utilities ──────────────────────────────────────────────────────────────────

pub fn with_alpha(c: Color, a: f32) -> Color {
    Color { a, ..c }
}

fn hex(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}

pub fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    Color {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: 1.0,
    }
}

// ── Container Styles ─────────────────────────────────────────────────────────

pub fn card(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(mantle())),
        border: Border { color: surface0(), width: 1.0, radius: 0.0.into() },
        ..Default::default()
    }
}

pub fn header(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(mantle())),
        ..Default::default()
    }
}

pub fn sidebar(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(mantle())),
        border: Border { color: surface0(), width: 1.0, radius: 0.0.into() },
        ..Default::default()
    }
}

pub fn selected_row(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(with_alpha(accent(), 0.18))),
        border: Border { color: iced::Color::TRANSPARENT, width: 0.0, radius: 4.0.into() },
        ..Default::default()
    }
}

pub fn player_panel(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(mantle())),
        border: Border { color: surface0(), width: 1.0, radius: 0.0.into() },
        ..Default::default()
    }
}

pub fn album_header(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(with_alpha(surface0(), 0.5))),
        border: Border { color: with_alpha(accent(), 0.2), width: 0.0, radius: 0.0.into() },
        ..Default::default()
    }
}

pub fn album_header_active(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(with_alpha(accent(), 0.15))),
        border: Border { color: with_alpha(accent(), 0.3), width: 1.0, radius: 4.0.into() },
        ..Default::default()
    }
}

pub fn spectrum_bar_color(amplitude: f32) -> Color {
    if amplitude < 0.5 {
        lerp_color(green(), accent(), amplitude * 2.0)
    } else {
        lerp_color(accent(), red(), (amplitude - 0.5) * 2.0)
    }
}

// ── Button Styles ──────────────────────────────────────────────────────────────

pub fn primary_button(_: &iced::Theme, status: iced::widget::button::Status) -> iced::widget::button::Style {
    let is_hovered = status == iced::widget::button::Status::Hovered || status == iced::widget::button::Status::Pressed;
    iced::widget::button::Style {
        background: Some(iced::Background::Color(if is_hovered { lerp_color(accent(), text(), 0.15) } else { accent() })),
        text_color: base(),
        border: Border {
            radius: 4.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        ..Default::default()
    }
}

pub fn secondary_button(_: &iced::Theme, status: iced::widget::button::Status) -> iced::widget::button::Style {
    let is_hovered = status == iced::widget::button::Status::Hovered || status == iced::widget::button::Status::Pressed;
    iced::widget::button::Style {
        background: Some(iced::Background::Color(if is_hovered { surface0() } else { mantle() })),
        text_color: text(),
        border: Border {
            radius: 4.0.into(),
            width: 1.0,
            color: surface0(),
        },
        ..Default::default()
    }
}

pub fn save_button(_: &iced::Theme, status: iced::widget::button::Status) -> iced::widget::button::Style {
    match status {
        iced::widget::button::Status::Pressed => {
            iced::widget::button::Style {
                background: Some(iced::Background::Color(green())),
                text_color: base(),
                border: Border {
                    radius: 4.0.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                ..Default::default()
            }
        }
        iced::widget::button::Status::Hovered => {
            iced::widget::button::Style {
                background: Some(iced::Background::Color(accent())),
                text_color: base(),
                border: Border {
                    radius: 4.0.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                ..Default::default()
            }
        }
        iced::widget::button::Status::Disabled => {
            iced::widget::button::Style {
                background: Some(iced::Background::Color(surface0())),
                text_color: subtext(),
                border: Border {
                    radius: 4.0.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                ..Default::default()
            }
        }
        _ => {
            iced::widget::button::Style {
                background: Some(iced::Background::Color(surface0())),
                text_color: accent(),
                border: Border {
                    radius: 4.0.into(),
                    width: 1.0,
                    color: accent(),
                },
                ..Default::default()
            }
        }
    }
}

pub fn save_button_saved(_: &iced::Theme, _status: iced::widget::button::Status) -> iced::widget::button::Style {
    iced::widget::button::Style {
        background: Some(iced::Background::Color(green())),
        text_color: base(),
        border: Border {
            radius: 4.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        ..Default::default()
    }
}

pub fn sidebar_list(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(base())),
        border: Border {
            color: surface0(),
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_omarchy_v4_colors_toml() {
        let content = r##"
            mode = "dark"
            accent = "#99d1db"
            selection = "#f2d5cf"
            muted = "#626880"
            background = "#303446"
            dark_background = "#35394b"
            lighter_background = "#51576d"
            foreground = "#f2d5cf"
            light_foreground = "#c6d0f5"
            red = "#f28fad"
            green = "#a6d189"
            yellow = "#e5c890"
            blue = "#8caaee"
        "##;
        let pal = parse_colors_toml(content).expect("Should parse Omarchy v4 theme");
        assert_eq!(pal.base, hex(0x30, 0x34, 0x46));
        assert_eq!(pal.text, hex(0xf2, 0xd5, 0xcf));
        assert_eq!(pal.accent, hex(0x99, 0xd1, 0xdb));
        assert_eq!(pal.overlay0, hex(0x62, 0x68, 0x80));
        assert_eq!(pal.mantle, hex(0x35, 0x39, 0x4b));
        assert_eq!(pal.surface0, hex(0x51, 0x57, 0x6d));
        assert_eq!(pal.subtext, hex(0xc6, 0xd0, 0xf5));
        assert_eq!(pal.red, hex(0xf2, 0x8f, 0xad));
        assert_eq!(pal.green, hex(0xa6, 0xd1, 0x89));
        assert_eq!(pal.yellow, hex(0xe5, 0xc8, 0x90));
        assert_eq!(pal.blue, hex(0x8c, 0xaa, 0xee));
    }

    #[test]
    fn test_parse_3digit_hex() {
        let content = r##"
            background = "#123"
            foreground = "#fff"
            accent = "#f00"
        "##;
        let pal = parse_colors_toml(content).expect("Should parse 3-digit hex");
        assert_eq!(pal.base, hex(0x11, 0x22, 0x33));
        assert_eq!(pal.text, hex(0xff, 0xff, 0xff));
        assert_eq!(pal.accent, hex(0xff, 0x00, 0x00));
    }
}






