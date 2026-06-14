use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use lofty::prelude::*;
use lofty::probe::Probe;

const AUDIO_EXTENSIONS: &[&str] = &["mp3", "flac", "ogg", "opus", "wav", "aac", "m4a", "wma", "aiff"];

fn main() {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
    let music_dir = PathBuf::from(home).join("Music");
    
    println!("Scanning library at {:?}...", music_dir);
    
    let mut artist_genres: HashMap<String, HashSet<String>> = HashMap::new();
    let mut files_by_artist: HashMap<String, Vec<PathBuf>> = HashMap::new();
    
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
            
            let genre = tags
                .and_then(|t| t.genre())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "Unknown".to_string());
            
            artist_genres.entry(artist.clone()).or_default().insert(genre);
            files_by_artist.entry(artist).or_default().push(path);
        }
    }
    
    println!("\n--- Artist & Genre Report ---");
    let mut sorted_artists: Vec<String> = artist_genres.keys().cloned().collect();
    sorted_artists.sort();
    
    for artist in sorted_artists {
        let genres: Vec<String> = artist_genres.get(&artist).unwrap().iter().cloned().collect();
        let file_count = files_by_artist.get(&artist).unwrap().len();
        println!("Artist: \"{}\" | Tracks: {} | Current Genres: {:?}", artist, file_count, genres);
    }
}
