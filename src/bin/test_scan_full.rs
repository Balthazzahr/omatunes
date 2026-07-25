
use std::path::Path;
use omatunes::library::scanner::scan_folder;

fn main() {
    let p = Path::new("/home/davepople/Drives/Media/Music");
    let tracks = scan_folder(p);
    println!("scan_folder returned {} tracks!", tracks.len());
    if let Some(t) = tracks.first() {
        println!("First track: {} - {} ({:?})", t.artist, t.title, t.path);
    }
}
