use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use lofty::prelude::*;
use lofty::probe::Probe;

const AUDIO_EXTENSIONS: &[&str] = &["mp3", "flac", "ogg", "opus", "wav", "aac", "m4a", "wma", "aiff"];

fn get_target_genre(artist: &str, album: &str, year: Option<u32>) -> Option<&'static str> {
    let artist_lower = artist.to_lowercase();
    let album_lower = album.to_lowercase();
    
    // Deftones special rule:
    // Older albums prior to self-titled (2003) -> Nu-Metal, after that -> Metal
    if artist_lower == "deftones" {
        // Match by known album names or year
        if album_lower == "adrenaline" || album_lower == "around the fur" || album_lower == "white pony" {
            return Some("Nu-Metal");
        }
        if let Some(y) = year {
            if y < 2003 {
                return Some("Nu-Metal");
            }
        }
        // Fallback for Deftones
        return Some("Metal");
    }
    
    // Ben Harper special rule
    if artist_lower == "ben harper" || artist_lower == "ben harper & the blind boys of alabama" {
        if album_lower.contains("fight for your mind") {
            return Some("Folk Rock");
        } else if album_lower.contains("the will to live") {
            return Some("Blues Rock");
        } else if album_lower.contains("there will be a light") {
            return Some("Soul");
        } else if album_lower.contains("welcome to the cruel world") {
            return Some("Folk Rock");
        }
    }
    
    match artist_lower.as_str() {
        "a perfect circle" => Some("Alternative Rock"),
        "at the drive-in" => Some("Post-Hardcore"),
        "bad religion" => Some("Punk Rock"),
        "beastie boys" => Some("Hip-Hop"),
        "blink-182" => Some("Pop-Punk"),
        "bob dylan" => Some("Folk Rock"),
        "box car racer" => Some("Pop-Punk"),
        "bracket" => Some("Pop-Punk"),
        "burning blue sky" => Some("Rock"),
        "cold chisel" => Some("Classic Rock"),
        "counting crows" => Some("Alternative Rock"),
        "creedence clearwater revival" => Some("Classic Rock"),
        
        "crowded house" => Some("Rock"),
        "dave matthews band" => Some("Alternative Rock"),
        "dave pople" => Some("Rock"),
        "dead letter circus" => Some("Progressive Rock"),
        "dieselboy" => Some("Punk Rock"),
        "faith no more" => Some("Alternative Metal"),
        "fear factory" => Some("Metal"),
        "frenzal rhomb" => Some("Punk Rock"),
        "george harrison" => Some("Classic Rock"),
        "good riddance" => Some("Punk Rock"),
        "gotye" => Some("Indie Rock"),
        "green day" => Some("Pop Punk"),
        "hi-standard" => Some("Punk"),
        "incubus" => Some("Alternative Rock"),
        "jack johnson" => Some("Rock"),
        "jars of clay" => Some("Rock"),
        "jason mraz" => Some("Rock"),
        "jeff buckley" => Some("Rock"),
        "jewel" => Some("Rock"),
        "jimmy eat world" => Some("Rock"),
        "john mayer" => Some("Rock"),
        "john mayer trio" => Some("Blues"),
        "karnivool" => Some("Metal"),
        "lagwagon" => Some("Punk"),
        "limp bizkit" => Some("Alternative"),
        "me first & the gimme gimmes" => Some("Punk"),
        "method" => Some("Rock"),
        "millencolin" => Some("Punk"),
        "nofx" => Some("Punk"),
        "nirvana" => Some("Alternative"),
        "no use for a name" => Some("Punk"),
        "one eye open" => Some("Rock"),
        "operation ivy" => Some("Punk"),
        "pearl jam" => Some("Rock"),
        "pink floyd" => Some("Rock"),
        "pitchshifter" => Some("Metal"),
        "propagandhi" => Some("Punk"),
        "rage against the machine" => Some("Alternative"),
        "rancid" => Some("Punk"),
        "red hot chili peppers" => Some("Alternative"),
        "ryan adams" => Some("Rock"),
        "sepultura" => Some("Metal"),
        "sevendust" => Some("Metal"),
        "silverchair" => Some("Alternative"),
        "slipknot" => Some("Metal"),
        "snuff" => Some("Punk"),
        "something for kate" => Some("Rock"),
        "soulfly" => Some("Metal"),
        "soundgarden" => Some("Alternative"),
        "stevie ray vaughan and double trouble" | "stevie ray vaughan & double trouble" => Some("Blues"),
        "strung out" => Some("Punk"),
        "temple of the dog" => Some("Alternative"),
        "the beatles" => Some("Rock"),
        "the clash" => Some("Punk"),
        "the cure" => Some("Alternative"),
        "the jimi hendrix experience" => Some("Rock"),
        "the smashing pumpkins" => Some("Alternative"),
        "the smiths" => Some("Alternative"),
        "the string quartet" => Some("Classical / Instrumental"),
        "third eye blind" => Some("Rock"),
        "tilt" => Some("Punk"),
        "tonic" => Some("Rock"),
        "tool" => Some("Metal"),
        "u2" => Some("Rock"),
        "ugly kid joe" => Some("Rock"),
        "unknown artist" => Some("Rock"),
        "vast" => Some("Alternative"),
        "wizo" => Some("Punk"),
        "insurge" => Some("Rock"),
        _ => None
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
    let music_dir = PathBuf::from(home).join("Music");
    
    println!("Scanning and updating genres in {:?}...", music_dir);
    
    let mut updated_count = 0;
    let mut total_files = 0;
    
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
        
        total_files += 1;
        
        let mut tagged_file = match Probe::open(&path).and_then(|p| p.read()) {
            Ok(tf) => tf,
            Err(_) => continue,
        };
        
        let (artist, album, year, current_genre) = {
            let tags = tagged_file.primary_tag();
            let artist = tags.and_then(|t| t.artist()).map(|s| s.to_string()).unwrap_or_default();
            let album = tags.and_then(|t| t.album()).map(|s| s.to_string()).unwrap_or_default();
            let year = tags.and_then(|t| t.year());
            let genre = tags.and_then(|t| t.genre()).map(|s| s.to_string()).unwrap_or_default();
            (artist, album, year, genre)
        };
        
        if artist.is_empty() {
            continue;
        }
        
        if let Some(target_genre) = get_target_genre(&artist, &album, year) {
            if current_genre != target_genre {
                println!(
                    "Updating \"{}\" - \"{}\": genre \"{}\" -> \"{}\" ({:?})",
                    artist, album, current_genre, target_genre, path.file_name().unwrap_or_default()
                );
                
                if let Some(tag) = tagged_file.primary_tag_mut() {
                    tag.set_genre(target_genre.to_string());
                }
                // Remove ID3v1 to avoid lofty panic crashes
                tagged_file.remove(lofty::tag::TagType::Id3v1);
                tagged_file.save_to_path(&path, Default::default())?;
                updated_count += 1;
            }
        }
    }
    
    println!("\nSuccessfully updated {} out of {} files.", updated_count, total_files);
    Ok(())
}
