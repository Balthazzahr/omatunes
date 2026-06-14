use lofty::prelude::*;
use lofty::probe::Probe;
use std::path::Path;

fn main() {
    let path = Path::new("/tmp/test2.mp3");
    println!("Opening file {:?}", path);
    let mut tagged_file = match Probe::open(path).and_then(|p| p.read()) {
        Ok(tf) => tf,
        Err(e) => {
            println!("Error opening: {:?}", e);
            return;
        }
    };
    println!("FileType: {:?}", tagged_file.file_type());
    println!("All tags: {:?}", tagged_file.tags().iter().map(|t| t.tag_type()).collect::<Vec<_>>());
    if let Some(tag) = tagged_file.primary_tag_mut() {
        println!("Primary tag type: {:?}", tag.tag_type());
        tag.set_genre("Alternative Rock".to_string());
    } else {
        println!("No primary tag!");
    }
    
    tagged_file.remove(lofty::tag::TagType::Id3v1);
    
    println!("Saving...");
    match tagged_file.save_to_path(path, Default::default()) {
        Ok(_) => println!("Save OK!"),
        Err(e) => {
            println!("Save Error: {:?}", e);
        }
    }
}
