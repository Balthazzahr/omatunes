use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::Result;
use lofty::config::WriteOptions;
use lofty::file::TaggedFileExt;
use lofty::prelude::*;
use lofty::probe::Probe;
use lofty::tag::{ItemKey, ItemValue, Tag, TagItem};
use walkdir::WalkDir;

use super::models::Track;

const AUDIO_EXTENSIONS: &[&str] = &["mp3", "flac", "ogg", "opus", "wav", "aac", "m4a", "wma", "aiff"];

const COVER_FILENAMES: &[&str] = &[
    "cover.jpg", "Cover.jpg",
    "cover.png", "Cover.png",
    "cover.webp", "Cover.webp",
    "folder.jpg", "Folder.jpg",
    "folder.png", "Folder.png",
];

/// Write like/rating metadata to an audio file using the unified generic tag API.
///
/// Writes POPM (rating=5), RATING=100, LIKE=1.
/// When `is_liked` is false, all of the above are removed.
pub fn write_like_status(path: &Path, is_liked: bool) -> Result<()> {
    let mut tagged_file = Probe::open(path)?.read()?;

    if tagged_file.primary_tag_mut().is_none() {
        tagged_file.insert_tag(Tag::new(tagged_file.primary_tag_type()));
    }
    let tag = tagged_file.primary_tag_mut().unwrap();

    tag.remove_key(&ItemKey::Popularimeter);
    let rating_key = ItemKey::Unknown("RATING".to_string());
    let like_key = ItemKey::Unknown("LIKE".to_string());
    tag.retain(|i| i.key() != &rating_key && i.key() != &like_key);

    if is_liked {
        tag.insert_unchecked(TagItem::new(
            ItemKey::Popularimeter,
            ItemValue::Text("5".to_string()),
        ));
        tag.insert_unchecked(TagItem::new(
            ItemKey::Unknown("RATING".to_string()),
            ItemValue::Text("100".to_string()),
        ));
        tag.insert_unchecked(TagItem::new(
            ItemKey::Unknown("LIKE".to_string()),
            ItemValue::Text("1".to_string()),
        ));
    }

    tagged_file.save_to_path(path, WriteOptions::default())?;
    Ok(())
}

/// Scan `dir` recursively and return tracks sorted by album/number/title.
/// `cover_data` is always `None` — loaded on demand via `load_cover`.
pub fn scan_folder(dir: &Path) -> Vec<Track> {
    let mut pairs: Vec<(PathBuf, TrackInfo)> = WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| !e.file_name().to_string_lossy().starts_with('.'))
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|entry| {
            let path = entry.path().to_path_buf();
            let ext = path.extension()?.to_str()?.to_lowercase();
            if !AUDIO_EXTENSIONS.contains(&ext.as_str()) {
                return None;
            }
            read_tags(&path).ok().map(|info| (path, info))
        })
        .collect();

    pairs.sort_by(|(_, a), (_, b)| {
        a.album.cmp(&b.album)
            .then(a.disc_number.cmp(&b.disc_number))
            .then(a.track_number.cmp(&b.track_number))
            .then(a.title.cmp(&b.title))
    });

    pairs.into_iter().enumerate().map(|(_i, (path, info))| {
        let play_count = crate::db::get(|db| db.play_counts.get(&path).copied().unwrap_or(0));
        Track {
            id: {
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                path.hash(&mut hasher);
                hasher.finish() as i64
            },
            path,
            title: info.title,
            artist: info.artist,
            album: info.album,
            album_id: 0,
            track_number: info.track_number,
            disc_number: info.disc_number,
            duration: Duration::from_millis(info.duration_ms),
            cover_data: None,
            genre: info.genre,
            year: info.year,
            play_count,
            liked: info.liked,
            date_played: None,
            lyrics: info.lyrics,
        }
    }).collect()
}

/// Load cover art for a track: embedded tag first, then cover.jpg in the folder.
pub fn load_cover(path: &Path) -> Option<Vec<u8>> {
    let tagged = Probe::open(path).ok()?.read().ok()?;
    let embedded = tagged.primary_tag().and_then(|t| {
        t.pictures().iter().find(|p| {
            matches!(
                p.pic_type(),
                lofty::picture::PictureType::CoverFront | lofty::picture::PictureType::Other
            )
        })
        .map(|p| p.data().to_vec())
    });
    embedded.or_else(|| cover_from_folder(path))
}

// ── Internal ───────────────────────────────────────────────────────────────────

struct TrackInfo {
    title: String,
    artist: String,
    album: String,
    track_number: Option<u32>,
    disc_number: Option<u32>,
    duration_ms: u64,
    genre: String,
    year: Option<u32>,
    lyrics: String,
    liked: bool,
}

fn read_tags(path: &Path) -> Result<TrackInfo> {
    let tagged = Probe::open(path)?.read()?;
    let duration_ms = tagged.properties().duration().as_millis() as u64;
    let tags = tagged.primary_tag();

    let unknown = crate::locale::get().unknown;

    let title = tags
        .and_then(|t| t.title())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(unknown)
                .to_string()
        });

    let folder_artist = path.parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .or_else(|| {
            path.parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
        })
        .unwrap_or(unknown)
        .to_string();

    let folder_album = path.parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or(unknown)
        .to_string();

    let artist = tags
        .and_then(|t| t.artist())
        .map(|s| s.to_string())
        .unwrap_or(folder_artist);

    let album = tags
        .and_then(|t| t.album())
        .map(|s| s.to_string())
        .unwrap_or(folder_album);

    let track_number = tags.and_then(|t| t.track());
    let disc_number = tags.and_then(|t| t.disk());
    let year = tags.and_then(|t| t.year());

    let genre = tags
        .and_then(|t| t.genre())
        .map(|s| s.to_string())
        .unwrap_or_else(|| unknown.to_string());

    let lyrics = tags
        .and_then(|t| t.get_string(&lofty::tag::ItemKey::Lyrics))
        .map(|s| s.to_string())
        .unwrap_or_default();

    let liked = tags
        .and_then(|t| t.get_string(&ItemKey::Unknown("LIKE".to_string())))
        .map(|s| s == "1")
        .unwrap_or(false);

    Ok(TrackInfo { title, artist, album, track_number, disc_number, duration_ms, genre, year, lyrics, liked })
}

fn cover_from_folder(path: &Path) -> Option<Vec<u8>> {
    let dir = path.parent()?;
    for name in COVER_FILENAMES {
        if let Ok(data) = std::fs::read(dir.join(name)) {
            return Some(data);
        }
    }
    None
}

pub fn write_tags(
    path: &Path,
    title: &str,
    artist: &str,
    album: &str,
    genre: &str,
    track_number: Option<u32>,
    disc_number: Option<u32>,
    cover_path: Option<&str>,
    year: Option<u32>,
    lyrics: Option<&str>,
) -> Result<()> {
    let mut tagged_file = Probe::open(path)?.read()?;

    if tagged_file.primary_tag_mut().is_none() {
        tagged_file.insert_tag(lofty::tag::Tag::new(tagged_file.primary_tag_type()));
    }
    let tag = tagged_file.primary_tag_mut().unwrap();

    tag.set_title(title.to_string());
    tag.set_artist(artist.to_string());
    tag.set_album(album.to_string());
    tag.set_genre(genre.to_string());
    if let Some(num) = track_number {
        tag.set_track(num);
    } else {
        tag.remove_track();
    }
    if let Some(num) = disc_number {
        tag.set_disk(num);
    } else {
        tag.remove_disk();
    }
    if let Some(yr) = year {
        tag.set_year(yr);
    } else {
        tag.remove_year();
    }

    if let Some(lyr) = lyrics {
        tag.insert_text(lofty::tag::ItemKey::Lyrics, lyr.to_string());
    }

    if let Some(cp) = cover_path {
        if let Ok(cover_data) = std::fs::read(cp) {
            let mime = if cp.to_lowercase().ends_with(".png") {
                "image/png".to_string()
            } else {
                "image/jpeg".to_string()
            };
            let picture = lofty::picture::Picture::new_unchecked(
                lofty::picture::PictureType::CoverFront,
                Some(lofty::picture::MimeType::Unknown(mime)),
                None,
                cover_data,
            );
            while !tag.pictures().is_empty() {
                tag.remove_picture(0);
            }
            tag.push_picture(picture);
        }
    }

    tagged_file.remove(lofty::tag::TagType::Id3v1);
    tagged_file.save_to_path(path, Default::default())?;
    Ok(())
}

