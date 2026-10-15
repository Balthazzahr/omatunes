use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

use iced::{Font, font::Weight};

/// Nerd Font loaded from the system.
/// Any installed Nerd Font (JetBrainsMono, FiraCode, Hack, etc.) works.
pub const NERD_FONT: Font = Font {
    family: iced::font::Family::Name("JetBrainsMono Nerd Font"),
    weight: Weight::Normal,
    stretch: iced::font::Stretch::Normal,
    style: iced::font::Style::Normal,
};

pub const NERD_FONT_MONO: Font = Font {
    family: iced::font::Family::Name("JetBrainsMono Nerd Font Mono"),
    ..NERD_FONT
};

/// System interface font family (default: JetBrainsMono Nerd Font Mono).
pub const SYSTEM_FONT_FAMILY: &str = "JetBrainsMono Nerd Font Mono";

/// Available font families for user selection in Settings > Display.
pub const AVAILABLE_FONT_FAMILIES: &[&str] = &[
    "JetBrainsMono Nerd Font Mono",
    "JetBrainsMono Nerd Font",
    "GeistMono Nerd Font Mono",
    "BlexMono Nerd Font Mono",
    "Adwaita Mono",
    "DejaVu Sans Mono",
    "Monospace",
];

pub const ICON_FONT: &str = "\u{f031}"; // nf-fa-font

/// Resolve a font family name to its static interned family string.
pub fn resolve_font_family(name: &str) -> &'static str {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        SYSTEM_FONT_FAMILY
    } else {
        intern_family(trimmed)
    }
}

/// Interned family-name cache.
///
/// `iced::font::Family::Name` wants a `&'static str`, but the fallback resolver
/// hands back a runtime `String`. Leaking each render call would grow without
/// bound in a per-frame path, so every distinct name is leaked exactly once and
/// memoized here. The key space is bounded by the number of *distinct installed
/// font family names* (~295 on a typical desktop), so the total leak is
/// bounded by that count plus the name lengths — kilobytes, once per process.
fn family_cache() -> &'static RwLock<HashMap<&'static str, &'static str>> {
    static CACHE: OnceLock<RwLock<HashMap<&'static str, &'static str>>> = OnceLock::new();
    CACHE.get_or_init(|| {
        // Seed with the hardcoded constants so interning never produces a
        // duplicate pointer for a name we already hold statically.
        RwLock::new(
            AVAILABLE_FONT_FAMILIES
                .iter()
                .map(|f| (*f, *f))
                .collect(),
        )
    })
}

/// Return a `&'static str` equal to `name`, interning it on first sight.
///
/// Cache hit (the overwhelmingly common path) is one `RwLock` read plus a hash
/// lookup and allocates nothing.
pub fn intern_family(name: &str) -> &'static str {
    if let Ok(cache) = family_cache().read() {
        if let Some(hit) = cache.get(name) {
            return hit;
        }
    }

    // Miss: intern once. Two racers may both leak here; the loser's copy is
    // unreachable, which costs at most one extra bounded allocation.
    let leaked: &'static str = Box::leak(name.to_string().into_boxed_str());
    if let Ok(mut cache) = family_cache().write() {
        if let Some(existing) = cache.get(name) {
            return existing;
        }
        cache.insert(leaked, leaked);
    }
    leaked
}

/// Font for rendering `text`, falling back to a covering family when the
/// configured font lacks glyphs.
pub fn ui_font_for(text: &str) -> iced::Font {
    let family = crate::config::with_font_settings(|primary, chain| {
        crate::font_index::pick_font_for(text, resolve_font_family(primary), chain)
    });
    let family = intern_family(&family);
    Font {
        family: iced::font::Family::Name(family),
        weight: Weight::Normal,
        stretch: iced::font::Stretch::Normal,
        style: iced::font::Style::Normal,
    }
}

/// Bold counterpart of [`ui_font_for`], mirroring [`ui_font_bold`]'s weight.
pub fn ui_font_for_bold(text: &str) -> iced::Font {
    let family = crate::config::with_font_settings(|primary, chain| {
        crate::font_index::pick_font_for(text, resolve_font_family(primary), chain)
    });
    let family = intern_family(&family);
    Font {
        family: iced::font::Family::Name(family),
        weight: Weight::Bold,
        stretch: iced::font::Stretch::Normal,
        style: iced::font::Style::Normal,
    }
}

/// UI font, family resolved from config at runtime.
pub fn ui_font() -> iced::Font {
    let family = resolve_font_family(&crate::config::get().font_family);
    Font {
        family: iced::font::Family::Name(family),
        weight: Weight::Normal,
        stretch: iced::font::Stretch::Normal,
        style: iced::font::Style::Normal,
    }
}

/// Bold UI font, family resolved from config at runtime.
pub fn ui_font_bold() -> iced::Font {
    let family = resolve_font_family(&crate::config::get().font_family);
    Font {
        family: iced::font::Family::Name(family),
        weight: Weight::Bold,
        stretch: iced::font::Stretch::Normal,
        style: iced::font::Style::Normal,
    }
}

// Font Awesome codepoints (Nerd Fonts tier 1 — universal across any Nerd Font)
pub const ICON_PLAY:     &str = "\u{f04b}";  //
pub const ICON_PAUSE:    &str = "\u{f04c}";  //
pub const ICON_PREV:     &str = "\u{f048}";  //
pub const ICON_NEXT:     &str = "\u{f051}";  //
pub const ICON_SHUFFLE:  &str = "\u{f074}";  //
pub const ICON_REPEAT:   &str = "\u{f021}";  //
pub const ICON_VOL_UP:   &str = "\u{f028}";  //
pub const ICON_MUSIC:    &str = "\u{f001}";  //
pub const ICON_LIST:       &str = "\u{f0cb8}"; // nf-md-playlist_music
pub const ICON_PLAYLIST_PLUS: &str = "\u{f0412}"; // nf-md-playlist_plus
pub const ICON_HEART:      &str = "\u{f004}";  //
pub const ICON_PLUS:       &str = "\u{f067}";  //
pub const ICON_PODIUM:     &str = "\u{f0d25}"; //
pub const ICON_VISUALIZER: &str = "\u{f147d}"; // waveform
pub const ICON_LYRICS:     &str = "\u{f0370}"; // nf-md-microphone_variant
pub const ICON_TROPHY:      &str = "\u{f091}"; // trophy (nf-fa-trophy)
pub const ICON_LIBRARY:     &str = "\u{f0330}"; // nf-md-library
pub const ICON_WAND:       &str = "\u{ebcf}";  // magic wand (nf-cod-wand)
pub const ICON_BOLT:       &str = "\u{f0e7}";  // bolt/flash
pub const ICON_PERSON:     &str = "\u{f4ff}";  // artist/person icon (nf-oct-person)
pub const ICON_CD:         &str = "\u{e271}";  // compact disc (nf-md-disc)
pub const ICON_TAG:        &str = "\u{f02b}";  // tag
pub const ICON_CALENDAR_DAY:  &str = "\u{f00ed}"; // nf-md-calendar
pub const ICON_CALENDAR_WEEK: &str = "\u{f272}";  // nf-fa-calendar_minus_o
pub const ICON_CALENDAR_MONTH_FA: &str = "\u{f073}"; // nf-fa-calendar
pub const ICON_CALENDAR_YEAR:  &str = "\u{ee46}"; // globe/earth
pub const ICON_ALL_TIME:      &str = "\u{f0d25}"; // podium (nf-fa-podium)
pub const ICON_CROWN:         &str = "\u{edeb}"; // nf-cod-crown
pub const ICON_GEM:           &str = "\u{f219}"; // nf-fa-diamond


// Settings nav icons
pub const ICON_KEYBOARD:      &str = "\u{f11c}";  // nf-fa-keyboard
pub const ICON_PALETTE:       &str = "\u{f03e5}"; // nf-md-palette
pub const ICON_MONITOR:       &str = "\u{f0990}"; // nf-md-monitor
pub const ICON_SLIDERS:       &str = "\u{f0de6}"; // nf-md-tune — for Playback settings
pub const ICON_FOLDER:        &str = "\u{f024b}"; // nf-md-folder
pub const ICON_AUTO_SCAN:     &str = "\u{f0498}"; // nf-md-sync — auto-scan
pub const ICON_VOLUME_HIGH:   &str = "\u{f028}";  // nf-fa-volume-up
pub const ICON_CHECK:         &str = "\u{f00c}";  // nf-fa-check
pub const ICON_TIMES:         &str = "\u{f00d}";  // nf-fa-times
pub const ICON_GLOBE:         &str = "\u{f0ac}";  // nf-fa-globe

// Toast / milestone icons
pub const ICON_ARROW_UP:     &str = "\u{f062}";  // nf-fa-arrow_up
pub const ICON_FIRE:         &str = "\u{f06d}";  // nf-fa-fire

// Table sort icons (FontAwesome — always in Nerd Fonts)
pub const ICON_SORT_UP:      &str = "\u{f0de}";  // nf-fa-sort_asc / caret up
pub const ICON_SORT_DOWN:    &str = "\u{f0dd}";  // nf-fa-sort_desc / caret down

pub fn get_award_image_bytes(period: &str, tier: &str) -> &'static [u8] {
    let (p, t) = match tier {
        "Bronze Trophy" => ("Yearly", "Bronze"),
        "Silver Trophy" => ("Yearly", "Silver"),
        "Gold Trophy" => ("Yearly", "Gold"),
        "Platinum Crown" => ("Monthly", "Platinum"),
        "Legendary Gem" => ("All-Time", "Legendary"),
        _ => (period, tier),
    };
    match (p, t) {
        ("Daily", "Bronze") => include_bytes!("../../assets/award_daily_bronze.png"),
        ("Daily", "Silver") => include_bytes!("../../assets/award_daily_silver.png"),
        ("Daily", "Gold") => include_bytes!("../../assets/award_daily_gold.png"),
        ("Daily", "Platinum") => include_bytes!("../../assets/award_daily_platinum.png"),
        ("Daily", "Legendary") => include_bytes!("../../assets/award_daily_legendary.png"),
        
        ("Weekly", "Bronze") => include_bytes!("../../assets/award_weekly_bronze.png"),
        ("Weekly", "Silver") => include_bytes!("../../assets/award_weekly_silver.png"),
        ("Weekly", "Gold") => include_bytes!("../../assets/award_weekly_gold.png"),
        ("Weekly", "Platinum") => include_bytes!("../../assets/award_weekly_platinum.png"),
        ("Weekly", "Legendary") => include_bytes!("../../assets/award_weekly_legendary.png"),
        
        ("Monthly", "Bronze") => include_bytes!("../../assets/award_monthly_bronze.png"),
        ("Monthly", "Silver") => include_bytes!("../../assets/award_monthly_silver.png"),
        ("Monthly", "Gold") => include_bytes!("../../assets/award_monthly_gold.png"),
        ("Monthly", "Platinum") => include_bytes!("../../assets/award_monthly_platinum.png"),
        ("Monthly", "Legendary") => include_bytes!("../../assets/award_monthly_legendary.png"),
        
        ("Yearly", "Bronze") => include_bytes!("../../assets/award_yearly_bronze.png"),
        ("Yearly", "Silver") => include_bytes!("../../assets/award_yearly_silver.png"),
        ("Yearly", "Gold") => include_bytes!("../../assets/award_yearly_gold.png"),
        ("Yearly", "Platinum") => include_bytes!("../../assets/award_yearly_platinum.png"),
        ("Yearly", "Legendary") => include_bytes!("../../assets/award_yearly_legendary.png"),
        
        ("All-Time", "Bronze") => include_bytes!("../../assets/award_all_time_bronze.png"),
        ("All-Time", "Silver") => include_bytes!("../../assets/award_all_time_silver.png"),
        ("All-Time", "Gold") => include_bytes!("../../assets/award_all_time_gold.png"),
        ("All-Time", "Platinum") => include_bytes!("../../assets/award_all_time_platinum.png"),
        ("All-Time", "Legendary") => include_bytes!("../../assets/award_all_time_legendary.png"),
        
        _ => include_bytes!("../../assets/award_daily_bronze.png"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_font_family_dynamic() {
        assert_eq!(resolve_font_family(""), SYSTEM_FONT_FAMILY);
        assert_eq!(resolve_font_family("   "), SYSTEM_FONT_FAMILY);
        assert_eq!(resolve_font_family("JetBrainsMono Nerd Font Mono"), "JetBrainsMono Nerd Font Mono");
        assert_eq!(resolve_font_family("Noto Sans CJK JP"), "Noto Sans CJK JP");
        assert_eq!(resolve_font_family("Custom Installed Font"), "Custom Installed Font");
    }
}


