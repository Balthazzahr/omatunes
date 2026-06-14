use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use lofty::prelude::*;
use lofty::probe::Probe;

const AUDIO_EXTENSIONS: &[&str] = &["mp3", "flac", "ogg", "opus", "wav", "aac", "m4a", "wma", "aiff"];

fn main() {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
    let music_dir = PathBuf::from(home).join("Music");
    
    let mut tracks = Vec::new();
    
    for entry in WalkDir::new(&music_dir)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| !e.file_name().to_string_lossy().starts_with('.'))
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path().to_path_buf();
        let ext = match path.extension().and_then(|s| s.to_str()) {
            Some(e) => e.to_lowercase(),
            None => continue,
        };
        
        if !AUDIO_EXTENSIONS.contains(&ext.as_str()) {
            continue;
        }
        
        if let Ok(tagged) = Probe::open(&path).and_then(|p| p.read()) {
            let tags = tagged.primary_tag();
            let artist = tags
                .and_then(|t| t.artist())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "Unknown Artist".to_string());
            
            if artist.to_lowercase() == "ben harper" {
                let album = tags
                    .and_then(|t| t.album())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "Unknown Album".to_string());
                
                let title = tags
                    .and_then(|t| t.title())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "Unknown Title".to_string());
                
                let genre = tags
                    .and_then(|t| t.genre())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "Unknown".to_string());
                
                tracks.push((path, album, title, genre));
            }
        }
    }
    
    tracks.sort_by(|a, b| a.1.cmp(&b.1).then(a.2.cmp(&b.2)));
    
    for (path, album, title, genre) in tracks {
        println!("Album: \"{}\" | Title: \"{}\" | Current Genre: \"{}\" | Path: {:?}", album, title, genre, path);
    }
}
