use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::Result;
use lofty::prelude::*;
use lofty::probe::Probe;
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

/// Escaneia `dir` recursivamente e retorna as faixas ordenadas por álbum/número/título.
/// `cover_data` é sempre `None` — carregado sob demanda via `load_cover`.
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
        let (play_count, liked) = crate::db::get(|db| {
            let pc = db.play_counts.get(&path).copied().unwrap_or(0);
            let l = db.favorites.contains(&path);
            (pc, l)
        });
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
            liked,
            date_played: None,
            lyrics: info.lyrics,
        }
    }).collect()
}

/// Carrega a capa de uma faixa: tag embutida primeiro, depois cover.jpg na pasta.
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

// ── Internos ───────────────────────────────────────────────────────────────────

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

    Ok(TrackInfo { title, artist, album, track_number, disc_number, duration_ms, genre, year, lyrics })
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

