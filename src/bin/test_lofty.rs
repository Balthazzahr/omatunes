use lofty::tag::{ItemKey, Tag, TagExt};

fn test_write(tag: &mut Tag) {
    tag.insert_text(ItemKey::Lyrics, "test lyrics".to_string());
}

fn main() {}
