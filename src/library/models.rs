use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct Track {
    pub id: i64,
    pub path: PathBuf,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_id: i64,
    pub track_number: Option<u32>,
    pub disc_number: Option<u32>,
    pub duration: Duration,
    pub cover_data: Option<Vec<u8>>,
    pub genre: String,
    pub year: Option<u32>,
    pub play_count: u32,
    pub liked: bool,
    pub date_played: Option<String>,
    pub lyrics: String,
}

impl Track {
    pub fn duration_str(&self) -> String {
        let secs = self.duration.as_secs();
        let m = secs / 60;
        let s = secs % 60;
        format!("{m}:{s:02}")
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Album {
    pub id: i64,
    pub title: String,
    pub artist: String,
    pub year: Option<u32>,
    pub cover_data: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct Artist {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Playlist {
    pub id: i64,
    pub name: String,
    pub track_count: usize,
}
