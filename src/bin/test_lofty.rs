use lofty::tag::ItemKey;

fn main() {
    let key1 = ItemKey::Lyrics;
    let key2 = ItemKey::UnsynchronizedLyrics;
    println!("Compiled: {:?}, {:?}", key1, key2);
}
