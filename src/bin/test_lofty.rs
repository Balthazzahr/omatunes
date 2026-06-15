use lofty::tag::{ItemKey, Tag, TagExt};

fn test_read(tag: &Tag) {
    if let Some(lyrics) = tag.get_string(&ItemKey::Lyrics) {
        println!("Lyrics: {}", lyrics);
    }
}

fn main() {}
