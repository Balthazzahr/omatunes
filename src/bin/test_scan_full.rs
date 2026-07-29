
use std::path::Path;
use omatunes::library::scanner::scan_folder;

fn main() {
    let music_dir = dirs::audio_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let tracks = scan_folder(&music_dir);
    println!("scan_folder returned {} tracks!", tracks.len());
    if let Some(t) = tracks.first() {
        println!("First track: {} - {} ({:?})", t.artist, t.title, t.path);
    }
}
