use lofty::prelude::*;
use lofty::probe::Probe;
use std::path::Path;

fn main() {
    let path = Path::new("/tmp/test.flac");
    println!("Opening file {:?}", path);
    let mut tagged_file = match Probe::open(path).and_then(|p| p.read()) {
        Ok(tf) => tf,
        Err(e) => {
            println!("Error opening: {:?}", e);
            return;
        }
    };
    println!("FileType: {:?}", tagged_file.file_type());
    if let Some(tag) = tagged_file.primary_tag_mut() {
        println!("Primary tag type: {:?}", tag.tag_type());
        tag.set_genre("Alternative Rock".to_string());
    } else {
        println!("No primary tag!");
    }
    
    println!("Saving...");
    if let Some(tag) = tagged_file.primary_tag() {
        match tag.save_to_path(path, Default::default()) {
            Ok(_) => println!("Save OK!"),
            Err(e) => {
                println!("Save Error: {:?}", e);
            }
        }
    } else {
        println!("No primary tag to save!");
    }
}
