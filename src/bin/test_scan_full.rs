
use std::path::Path;
use omatunes::library::scanner::scan_folder;

fn main() {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let music_dir = std::path::PathBuf::from(home).join("Music");
    let tracks = scan_folder(&music_dir);
    println!("scan_folder returned {} tracks!", tracks.len());
    if let Some(t) = tracks.first() {
        println!("First track: {} - {} ({:?})", t.artist, t.title, t.path);
    }
}
