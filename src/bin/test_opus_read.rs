
use std::path::Path;
use lofty::probe::Probe;
use lofty::prelude::*;

fn main() {
    let p = Path::new("/home/user/Drives/Media/Music/John Mayer/Born And Raised/01 Queen of California.opus");
    match Probe::open(p) {
        Ok(probe) => {
            match probe.read() {
                Ok(tagged) => {
                    println!("SUCCESS reading tags for opus file!");
                    println!("Duration ms: {}", tagged.properties().duration().as_millis());
                    if let Some(t) = tagged.primary_tag() {
                        println!("Title: {:?}", t.title());
                        println!("Artist: {:?}", t.artist());
                    }
                }
                Err(e) => println!("ERROR in probe.read(): {:?}", e),
            }
        }
        Err(e) => println!("ERROR in Probe::open(): {:?}", e),
    }
}
