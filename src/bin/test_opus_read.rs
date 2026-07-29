
use std::path::Path;
use lofty::probe::Probe;
use lofty::prelude::*;

fn main() {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let p = std::path::PathBuf::from(home).join("sample.opus");
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
