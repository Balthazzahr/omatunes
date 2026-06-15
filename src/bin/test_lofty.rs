use lofty::tag::{ItemKey, TagExt};

fn test_read(tag: &dyn TagExt) {
    if let Some(lyrics) = tag.get_string(&ItemKey::Lyrics) {
        println!("Lyrics: {}", lyrics);
    }
}

fn main() {}
