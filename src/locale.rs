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

static EN: Strings = Strings {
    sidebar_folders:  "Folders",
    no_track:         "No track",
    no_tracks_found:  "No tracks found",
    select_folder:    "Select a folder",
    unknown:          "Unknown",
    track_singular:   "track",
    track_plural:     "tracks",
};

pub fn load() {
    LOCALE.get_or_init(|| &EN);
}

pub fn get() -> &'static Strings {
    LOCALE.get_or_init(|| &EN)
}
