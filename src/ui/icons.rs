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

/// Base UI font — same family as Waybar/Omarchy.
pub const UI_FONT: Font = Font {
    family: iced::font::Family::Name("JetBrainsMono Nerd Font Mono"),
    weight: Weight::Normal,
    stretch: iced::font::Stretch::Normal,
    style: iced::font::Style::Normal,
};

pub const UI_FONT_BOLD: Font = Font {
    weight: Weight::Bold,
    ..UI_FONT
};

// Font Awesome codepoints (Nerd Fonts tier 1 — universal across any Nerd Font)
pub const ICON_PLAY:     &str = "\u{f04b}";  //
pub const ICON_PAUSE:    &str = "\u{f04c}";  //
pub const ICON_PREV:     &str = "\u{f048}";  //
pub const ICON_NEXT:     &str = "\u{f051}";  //
pub const ICON_SHUFFLE:  &str = "\u{f074}";  //
pub const ICON_REPEAT:   &str = "\u{f021}";  //
pub const ICON_REPEAT_ONE: &str = "\u{f1b8}";  //
pub const ICON_VOL_UP:   &str = "\u{f028}";  //
pub const ICON_VOL_MUTE: &str = "\u{f026}";  //
pub const ICON_MUSIC:    &str = "\u{f001}";  //
pub const ICON_LIST:       &str = "\u{f0cb8}"; // nf-md-playlist_music
pub const ICON_PLAYLIST_PLUS: &str = "\u{f0412}"; // nf-md-playlist_plus
pub const ICON_HEART:      &str = "\u{f004}";  //
pub const ICON_PLUS:       &str = "\u{f067}";  //
pub const ICON_TRASH:      &str = "\u{f1f8}";  //
pub const ICON_COPY:       &str = "\u{f0c5}";  //
pub const ICON_PODIUM:     &str = "\u{f0d25}"; //
pub const ICON_VISUALIZER: &str = "\u{f147d}"; // waveform
pub const ICON_LYRICS:     &str = "\u{f0370}"; // nf-md-microphone_variant
pub const ICON_STATS:      &str = "\u{f0126}"; // nf-md-chart-bar
pub const ICON_CALENDAR_TODAY: &str = "\u{f00ea}"; // nf-md-calendar-today
pub const ICON_CALENDAR_MONTH: &str = "\u{f0e17}"; // nf-md-calendar-month
pub const ICON_TROPHY:      &str = "\u{f053f}"; // nf-md-trophy
pub const ICON_LIBRARY:     &str = "\u{f0330}"; // nf-md-library
pub const ICON_WAND:       &str = "\u{ebcf}";  // magic wand (nf-cod-wand)
pub const ICON_BOLT:       &str = "\u{f0e7}";  // bolt/flash
pub const ICON_PERSON:     &str = "\u{f4ff}";  // artist/person icon (nf-oct-person)
pub const ICON_CD:         &str = "\u{e271}";  // compact disc (nf-md-disc)
pub const ICON_TAG:        &str = "\u{f02b}";  // tag
pub const ICON_CLOCK:      &str = "\u{f017}";  // clock (nf-fa-clock_o)
pub const ICON_CALENDAR_DAY:  &str = "\u{f00ed}"; // nf-md-calendar
pub const ICON_CALENDAR_WEEK: &str = "\u{f272}";  // nf-fa-calendar_minus_o
pub const ICON_CALENDAR_MONTH_FA: &str = "\u{f073}"; // nf-fa-calendar
pub const ICON_TROPHY_FA:     &str = "\u{f091}"; // trophy (nf-fa-trophy)
pub const ICON_AWARD:         &str = "\u{f559}"; // award ribbon (nf-fa-award)
pub const ICON_MEDAL:         &str = "\u{f5a2}"; // medal with star (nf-fa-medal)
pub const ICON_CROWN:         &str = "\u{f521}"; // crown (nf-fa-crown)
pub const ICON_GEM:           &str = "\u{f3a5}"; // gem (nf-fa-gem)


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
pub const ICON_HEADPHONES:   &str = "\u{f025}";  // nf-fa-headphones — Bronze
pub const ICON_STAR:         &str = "\u{f005}";  // nf-fa-star — Silver
pub const ICON_ARROW_UP:     &str = "\u{f062}";  // nf-fa-arrow_up
pub const ICON_ARROW_DOWN:   &str = "\u{f063}";  // nf-fa-arrow_down


