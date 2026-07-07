use std::sync::OnceLock;

static LOCALE: OnceLock<&'static Strings> = OnceLock::new();

pub struct Strings {
    pub sidebar_folders:  &'static str,
    pub no_track:         &'static str,
    pub no_tracks_found:  &'static str,
    pub select_folder:    &'static str,
    pub unknown:          &'static str,
    track_singular:       &'static str,
    track_plural:         &'static str,
}

impl Strings {
    pub fn track_count(&self, n: usize) -> String {
        if n == 1 {
            format!("1 {}", self.track_singular)
        } else {
            format!("{} {}", n, self.track_plural)
        }
    }
}

// ── Translations ───────────────────────────────────────────────────────────────

static EN: Strings = Strings {
    sidebar_folders:  "Folders",
    no_track:         "No track",
    no_tracks_found:  "No tracks found",
    select_folder:    "Select a folder",
    unknown:          "Unknown",
    track_singular:   "track",
    track_plural:     "tracks",
};

static PT_BR: Strings = Strings {
    sidebar_folders:  "Pastas",
    no_track:         "Nenhuma faixa",
    no_tracks_found:  "Nenhuma faixa encontrada",
    select_folder:    "Selecione uma pasta",
    unknown:          "Desconhecido",
    track_singular:   "faixa",
    track_plural:     "faixas",
};

static ES: Strings = Strings {
    sidebar_folders:  "Carpetas",
    no_track:         "Sin pista",
    no_tracks_found:  "Sin pistas",
    select_folder:    "Selecciona una carpeta",
    unknown:          "Desconocido",
    track_singular:   "pista",
    track_plural:     "pistas",
};

// ── Initialization ─────────────────────────────────────────────────────────────

pub fn load() {
    LOCALE.get_or_init(detect);
}

pub fn get() -> &'static Strings {
    LOCALE.get_or_init(detect)
}

fn detect() -> &'static Strings {
    let override_lang = crate::config::get().language.clone();
    let lang = if override_lang == "auto" || override_lang.is_empty() {
        std::env::var("LANG")
            .or_else(|_| std::env::var("LANGUAGE"))
            .or_else(|_| std::env::var("LC_ALL"))
            .unwrap_or_default()
    } else {
        override_lang
    };
    let lang = lang.split('.').next().unwrap_or("").to_lowercase();
    if lang.starts_with("pt") { &PT_BR }
    else if lang.starts_with("es") { &ES }
    else { &EN }
}
