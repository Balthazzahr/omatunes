//! Installed-font enumeration and glyph-coverage index.
//!
//! iced 0.13 has no font fallback: `iced_graphics::text::paragraph` turns one
//! `iced::Font` into a single cosmic-text `Attrs`, so a text widget gets
//! exactly one family and anything that family does not cover renders as tofu.
//! The only escape hatch available to us is therefore *whole-string font
//! selection by coverage*: if the primary family does not cover every codepoint
//! in a string, switch the entire string to a family that does.
//!
//! Everything here is built lazily behind `OnceLock`s so that app launch never
//! pays for cmap parsing, and the resolver is a handful of hash/binary searches
//! on the hot rendering path.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock, RwLock};

use fontdb::{Database, ID, Source};
use ttf_parser::Face as TtfFace;

/// A coarse script/coverage category a font can provide.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ScriptClass {
    Latin,
    Cjk,
    Hangul,
    Arabic,
    Cyrillic,
    Greek,
    Hebrew,
    Devanagari,
    Thai,
    Emoji,
    NerdSymbols,
}

impl ScriptClass {
    /// Short human label for the settings UI, e.g. "Japanese/Chinese".
    pub fn label(&self) -> &'static str {
        match self {
            ScriptClass::Latin => "Latin",
            ScriptClass::Cjk => "Japanese/Chinese",
            ScriptClass::Hangul => "Korean",
            ScriptClass::Arabic => "Arabic",
            ScriptClass::Cyrillic => "Cyrillic",
            ScriptClass::Greek => "Greek",
            ScriptClass::Hebrew => "Hebrew",
            ScriptClass::Devanagari => "Devanagari",
            ScriptClass::Thai => "Thai",
            ScriptClass::Emoji => "Emoji",
            ScriptClass::NerdSymbols => "Symbols/Icons",
        }
    }

    /// Single-char sample to render next to the label in the UI.
    #[allow(dead_code)]
    pub fn sample(&self) -> &'static str {
        match self {
            ScriptClass::Latin => "A",
            ScriptClass::Cjk => "漢",
            ScriptClass::Hangul => "한",
            ScriptClass::Arabic => "ع",
            ScriptClass::Cyrillic => "Ж",
            ScriptClass::Greek => "Ω",
            ScriptClass::Hebrew => "א",
            ScriptClass::Devanagari => "क",
            ScriptClass::Thai => "ก",
            ScriptClass::Emoji => "\u{1F600}",
            ScriptClass::NerdSymbols => "\u{E0B0}",
        }
    }
}

/// Metadata about one installed font family.
#[derive(Clone, Debug)]
pub struct FontInfo {
    pub family: String,
    /// Sorted deterministically (declaration order of [`ScriptClass`]).
    pub classes: Vec<ScriptClass>,
    /// Path of one representative face. Empty for in-memory fonts.
    pub path: PathBuf,
}

// ---------------------------------------------------------------------------
// Codepoint -> script classification
// ---------------------------------------------------------------------------

/// Inclusive codepoint ranges, most specific first. Nothing below U+0370 is
/// classified, so ASCII/Latin-1 never triggers a fallback.
const RANGES: &[(u32, u32, ScriptClass)] = &[
    // Nerd Font / Material Icons private use areas.
    (0xE000, 0xF8FF, ScriptClass::NerdSymbols),
    (0xF0000, 0xF01FF, ScriptClass::NerdSymbols),
    // Emoji and the symbol blocks that ship with emoji fonts.
    (0x1F300, 0x1FAFF, ScriptClass::Emoji),
    (0x2600, 0x27BF, ScriptClass::Emoji),
    (0x2B00, 0x2BFF, ScriptClass::Emoji),
    // CJK ideographs (all extension planes) before Hangul, because CJK
    // extension G (U+30000..=U+3134A) overlaps Hangul compatibility Jamo.
    (0x3400, 0x4DBF, ScriptClass::Cjk),
    (0x4E00, 0x9FFF, ScriptClass::Cjk),
    (0xF900, 0xFAFF, ScriptClass::Cjk),
    (0x20000, 0x2A6DF, ScriptClass::Cjk),
    (0x2A700, 0x2EBEF, ScriptClass::Cjk),
    (0x30000, 0x3134F, ScriptClass::Cjk),
    // Kana.
    (0x3040, 0x309F, ScriptClass::Cjk),
    (0x30A0, 0x30FF, ScriptClass::Cjk),
    (0x31F0, 0x31FF, ScriptClass::Cjk),
    (0xFF66, 0xFF9D, ScriptClass::Cjk),
    // Hangul.
    (0x1100, 0x11FF, ScriptClass::Hangul),
    (0x3130, 0x318F, ScriptClass::Hangul),
    (0xA960, 0xA97F, ScriptClass::Hangul),
    (0xAC00, 0xD7AF, ScriptClass::Hangul),
    (0xD7B0, 0xD7FF, ScriptClass::Hangul),
    (0xFFA0, 0xFFDC, ScriptClass::Hangul),
    // Arabic.
    (0x0600, 0x06FF, ScriptClass::Arabic),
    (0x0750, 0x077F, ScriptClass::Arabic),
    (0x08A0, 0x08FF, ScriptClass::Arabic),
    (0xFB50, 0xFDFF, ScriptClass::Arabic),
    (0xFE70, 0xFEFF, ScriptClass::Arabic),
    // Cyrillic, Greek, Hebrew, Devanagari, Thai.
    (0x0400, 0x052F, ScriptClass::Cyrillic),
    (0x0370, 0x03FF, ScriptClass::Greek),
    (0x1F00, 0x1FFF, ScriptClass::Greek),
    (0x0590, 0x05FF, ScriptClass::Hebrew),
    (0xFB1D, 0xFB4F, ScriptClass::Hebrew),
    (0x0900, 0x097F, ScriptClass::Devanagari),
    (0xA8E0, 0xA8FF, ScriptClass::Devanagari),
    (0x0E00, 0x0E7F, ScriptClass::Thai),
];

/// Lowest codepoint covered by [`RANGES`]; anything below is unclassified.
const RANGE_FLOOR: u32 = 0x0370;

/// Classify a single char. Returns `None` for ASCII/basic Latin and
/// other unclassified chars, so they don't force a fallback.
pub fn script_class_of(c: char) -> Option<ScriptClass> {
    let cp = c as u32;
    if cp < RANGE_FLOOR {
        return None;
    }
    RANGES
        .iter()
        .find(|(lo, hi, _)| cp >= *lo && cp <= *hi)
        .map(|(_, _, class)| *class)
}

/// Representative codepoints used to decide which classes a family provides.
/// A family gets the class when it covers *at least one* probe.
const PROBES: &[(ScriptClass, &[char])] = &[
    (ScriptClass::Latin, &['A', 'a', 'é']),
    (ScriptClass::Cjk, &['漢', 'あ', 'ア']),
    (ScriptClass::Hangul, &['한', '가']),
    (ScriptClass::Arabic, &['م', 'ا']),
    (ScriptClass::Cyrillic, &['Ж', 'ж']),
    (ScriptClass::Greek, &['α', 'Ω']),
    (ScriptClass::Hebrew, &['א']),
    (ScriptClass::Devanagari, &['न', 'अ']),
    (ScriptClass::Thai, &['ส']),
    (ScriptClass::Emoji, &['\u{1F600}', '❤']),
    (ScriptClass::NerdSymbols, &['\u{E0B0}', '\u{F0001}']),
];

// ---------------------------------------------------------------------------
// Font database + family index
// ---------------------------------------------------------------------------

/// One entry per family, holding every face that belongs to it.
struct FamilyEntry {
    family: String,
    path: PathBuf,
    faces: Vec<ID>,
}

impl FamilyEntry {
    /// A single representative face, preferring a non-italic, non-condensed
    /// style. Weight/style variants of a family share the same cmap, so one
    /// face is enough for coverage and it keeps a `.ttc` with 7 faces from
    /// being mapped and enumerated 7 times.
    fn representative_face(&self) -> Option<fontdb::ID> {
        let db = db();
        self.faces
            .iter()
            .copied()
            .find(|id| {
                db.face(*id).is_some_and(|f| {
                    f.style == fontdb::Style::Normal && f.stretch == fontdb::Stretch::Normal
                })
            })
            .or_else(|| self.faces.first().copied())
    }
}

fn db() -> &'static Database {
    static DB: OnceLock<Database> = OnceLock::new();
    DB.get_or_init(|| {
        let mut db = Database::new();
        db.load_system_fonts();
        db
    })
}

/// All families, de-duplicated, sorted by family name.
fn families() -> &'static [FamilyEntry] {
    static FAMILIES: OnceLock<Vec<FamilyEntry>> = OnceLock::new();
    FAMILIES.get_or_init(|| {
        let mut out: Vec<FamilyEntry> = Vec::new();
        let mut index: HashMap<String, usize> = HashMap::new();

        for face in db().faces() {
            let Some((family, _)) = face.families.first() else {
                continue;
            };
            let family = family.trim();
            if family.is_empty() {
                continue;
            }
            let path = match &face.source {
                Source::File(p) => p.clone(),
                _ => PathBuf::new(),
            };
            match index.get(family) {
                Some(&i) => out[i].faces.push(face.id),
                None => {
                    index.insert(family.to_string(), out.len());
                    out.push(FamilyEntry {
                        family: family.to_string(),
                        path,
                        faces: vec![face.id],
                    });
                }
            }
        }

        out.sort_by(|a, b| a.family.cmp(&b.family));
        out
    })
}

fn entry(family: &str) -> Option<&'static FamilyEntry> {
    static BY_NAME: OnceLock<HashMap<String, usize>> = OnceLock::new();
    let map = BY_NAME.get_or_init(|| {
        families()
            .iter()
            .enumerate()
            .map(|(i, e)| (e.family.clone(), i))
            .collect()
    });
    map.get(family).map(|&i| &families()[i])
}

// ---------------------------------------------------------------------------
// Coverage: lazy, cached per family
// ---------------------------------------------------------------------------

/// Per-family coverage verdicts.
///
/// Deliberately *not* a full glyph set: enumerating a CJK cmap means touching
/// every page of a ~19 MB `.ttc` and sorting ~174k codepoints, which measured
/// at ~185 ms per family. Instead we memoize the verdict for each codepoint we
/// are actually asked about, answering a batch of unknowns with a single
/// `with_face_data` pass. Repeat queries are then a hash lookup, and a cold
/// query costs one mmap plus a handful of cmap binary searches.
struct Coverage {
    /// The one face we query. `None` means the family has no usable face and
    /// therefore covers nothing.
    face: Option<ID>,
    verdicts: RwLock<HashMap<char, bool>>,
}

impl Coverage {
    /// Resolve a batch of codepoints against this family.
    ///
    /// Answers every `false` (no coverage) if the face cannot be parsed, so an
    /// unreadable font degrades to "covers nothing" rather than panicking.
    fn answer(&self, chars: &[char]) -> bool {
        let unknown: Vec<char> = {
            let cached = self.verdicts.read().unwrap_or_else(|e| e.into_inner());
            let mut unknown: Vec<char> = chars
                .iter()
                .copied()
                .filter(|c| !cached.contains_key(c))
                .collect();
            unknown.sort_unstable();
            unknown.dedup();
            unknown
        };

        if !unknown.is_empty() {
            let mut answers: Vec<Option<bool>> = vec![None; unknown.len()];
            if let Some(id) = self.face {
                let _ = db().with_face_data(id, |data, index| {
                    // `index` is the face index inside a `.ttc` collection. Noto
                    // CJK ships as collections, so it must be threaded through
                    // or coverage silently fails for the fonts that matter most.
                    let Ok(face) = TtfFace::parse(data, index) else {
                        return;
                    };
                    for (slot, c) in answers.iter_mut().zip(&unknown) {
                        *slot = Some(face.glyph_index(*c).is_some());
                    }
                });
            }
            let mut cache = self.verdicts.write().unwrap_or_else(|e| e.into_inner());
            for (slot, c) in answers.iter().zip(&unknown) {
                // An unreadable face covers nothing.
                cache.insert(*c, slot.unwrap_or(false));
            }
        }

        let cache = self.verdicts.read().unwrap_or_else(|e| e.into_inner());
        chars.iter().all(|c| cache.get(c).copied().unwrap_or(false))
    }
}

fn classes_cache() -> &'static RwLock<HashMap<String, Arc<[ScriptClass]>>> {
    static CACHE: OnceLock<RwLock<HashMap<String, Arc<[ScriptClass]>>>> = OnceLock::new();
    CACHE.get_or_init(|| RwLock::new(HashMap::new()))
}

fn coverage_cache() -> &'static RwLock<HashMap<String, Arc<Coverage>>> {
    static CACHE: OnceLock<RwLock<HashMap<String, Arc<Coverage>>>> = OnceLock::new();
    CACHE.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Cheap probe-based classification: ~11 cmap lookups, no enumeration.
fn classes_of(entry: &FamilyEntry) -> Arc<[ScriptClass]> {
    if let Ok(cache) = classes_cache().read() {
        if let Some(hit) = cache.get(&entry.family) {
            return Arc::clone(hit);
        }
    }

    let mut found: Vec<ScriptClass> = Vec::new();
    if let Some(id) = entry.representative_face() {
        let _ = db().with_face_data(id, |data, index| {
            let Ok(face) = TtfFace::parse(data, index) else {
                return;
            };
            for (class, probes) in PROBES {
                if found.contains(class) {
                    continue;
                }
                if probes.iter().any(|p| face.glyph_index(*p).is_some()) {
                    found.push(*class);
                }
            }
        });
    }
    found.sort_unstable();
    found.dedup();

    let shared: Arc<[ScriptClass]> = Arc::from(found.into_boxed_slice());
    if let Ok(mut cache) = classes_cache().write() {
        cache.insert(entry.family.clone(), Arc::clone(&shared));
    }
    shared
}

/// The family's coverage memo. Built at most once, on first use.
fn coverage_of(entry: &FamilyEntry) -> Arc<Coverage> {
    if let Ok(cache) = coverage_cache().read() {
        if let Some(hit) = cache.get(&entry.family) {
            return Arc::clone(hit);
        }
    }
    let fresh = Arc::new(Coverage {
        face: entry.representative_face(),
        verdicts: RwLock::new(HashMap::new()),
    });
    if let Ok(mut cache) = coverage_cache().write() {
        cache.insert(entry.family.clone(), Arc::clone(&fresh));
    }
    fresh
}

// ---------------------------------------------------------------------------
// Public query API
// ---------------------------------------------------------------------------

/// Every installed font family, sorted by family name, de-duplicated.
pub fn fonts() -> &'static Vec<FontInfo> {
    static LIST: OnceLock<Vec<FontInfo>> = OnceLock::new();
    LIST.get_or_init(|| {
        families()
            .iter()
            .map(|e| FontInfo {
                family: e.family.clone(),
                classes: classes_of(e).to_vec(),
                path: e.path.clone(),
            })
            .collect()
    })
}

/// Does `family` contain a glyph for `c`?
pub fn family_covers(family: &str, c: char) -> bool {
    let Some(entry) = entry(family) else {
        return false;
    };
    coverage_of(entry).answer(&[c])
}

/// Which script classes in `text` are NOT covered by `family`.
pub fn missing_scripts(text: &str, family: &str) -> Vec<ScriptClass> {
    let mut missing: Vec<ScriptClass> = Vec::new();
    for c in text.chars() {
        let Some(class) = script_class_of(c) else {
            continue;
        };
        if !missing.contains(&class) && !family_covers(family, c) {
            missing.push(class);
        }
    }
    missing.sort_unstable();
    missing
}

fn covers_all(family: &str, chars: &[char]) -> bool {
    let Some(entry) = entry(family) else {
        return false;
    };
    coverage_of(entry).answer(chars)
}

/// The core resolver. Returns the family to actually use for `text`:
///   - if `primary` covers every codepoint, return `primary`
///   - else walk `chain` in order, return the first entry covering all codepoints
///   - else fall back to an auto-discovered covering font
///   - else return `primary` unchanged
pub fn pick_font_for(text: &str, primary: &str, chain: &[String]) -> String {
    // Fast path: nothing in the string can force a fallback.
    let needed: Vec<char> = text
        .chars()
        .filter(|c| script_class_of(*c).is_some())
        .collect();
    if needed.is_empty() {
        return primary.to_string();
    }

    if covers_all(primary, &needed) {
        return primary.to_string();
    }

    // Resolution order, region-aware:
    //   1. the primary font, if it covers everything
    //   2. a chain entry matching the text's region (an explicit user choice)
    //   3. the region-biased shortlist (auto-discovery, correct glyph forms)
    //   4. any chain entry, then any shortlist entry
    // Region-correct auto-discovery deliberately outranks a region-MISMATCHED
    // chain entry: a chain is a preference list, and the order entries were
    // saved in says nothing about which regional glyph forms the text wants.
    // `suggested_chain()` sorts alphabetically, so "…HK" can precede "…JP",
    // and a KR variant may be absent from the chain entirely. Two passes, so
    // this allocates nothing on the render path.
    let region = region_of(text);
    if let Some(token) = region {
        for family in chain {
            if name_has_region(family, token) && covers_all(family, &needed) {
                return family.clone();
            }
        }
        for family in region_first(region).iter() {
            if covers_all(family, &needed) {
                return family.clone();
            }
        }
    }
    for family in chain {
        if covers_all(family, &needed) {
            return family.clone();
        }
    }
    for family in region_first(region).iter().chain(shortlist()) {
        if covers_all(family, &needed) {
            return family.clone();
        }
    }
    primary.to_string()
}

/// Does this family name advertise the given regional CJK variant?
fn name_has_region(family: &str, region: &str) -> bool {
    family.to_lowercase().contains(region)
}

// ---------------------------------------------------------------------------
// Auto-discovery shortlist
// ---------------------------------------------------------------------------

/// Family-name substrings that mark a font as a likely fallback provider,
/// mapped to a bonus score. Higher wins.
const NAME_HINTS: &[(&str, u32)] = &[
    // Strong multi-script providers.
    ("cjk", 12),
    ("source han", 12),
    ("unifont", 10),
    ("droid sans fallback", 10),
    ("noto sans symbols2", 9),
    ("noto sans symbols", 9),
    ("noto color symbols", 9),
    ("noto color emoji", 9),
    ("emoji", 8),
    ("symbola", 8),
    ("twemoji", 8),
    // Regional CJK families.
    ("noto sans jp", 7),
    ("noto sans kr", 7),
    ("noto sans sc", 7),
    ("noto sans tc", 7),
    ("noto sans hk", 7),
    ("noto serif jp", 7),
    ("noto serif kr", 7),
    ("noto serif sc", 7),
    ("ipa gothic", 6),
    ("ipa mincho", 6),
    ("meiryo", 6),
    ("malgun", 6),
    ("nanum", 6),
    ("batang", 6),
    ("gulim", 6),
    ("ms gothic", 5),
    ("ms mincho", 5),
    ("yahei", 5),
    ("simsun", 5),
    ("micro hei", 5),
    ("wenquanyi", 5),
    ("wqy", 5),
    // Script-specific Noto families.
    ("noto naskh", 5),
    ("noto kufi", 5),
    ("noto sans arabic", 5),
    ("noto sans hebrew", 5),
    ("noto sans devanagari", 5),
    ("noto sans thai", 5),
    ("noto sans armenian", 4),
    ("noto sans georgian", 4),
    ("noto sans tamil", 4),
    ("noto sans telugu", 4),
    ("noto sans kannada", 4),
    ("noto sans malayalam", 4),
    ("noto sans sinhala", 4),
    ("noto sans lao", 4),
    ("noto sans khmer", 4),
    ("noto sans tibetan", 4),
    ("noto sans myanmar", 4),
    ("noto sans bengali", 4),
    ("noto sans gujarati", 4),
    ("noto sans kannada ", 4),
    ("noto sans oriya", 4),
    ("noto sans tamil ", 4),
    ("amiri", 4),
    ("scheherazade", 4),
    // Icon fonts.
    ("nerd font", 3),
    ("nerd fonts", 3),
    ("material icons", 3),
    // Generic multi-script fallbacks.
    ("dejavu", 2),
    ("liberation", 2),
    ("noto sans", 1),
    ("noto serif", 1),
];

/// Cap on how many name-matched families get probed when building the list, so
/// shortlist construction can never turn into an O(all fonts) cmap scan.
const SHORTLIST_PROBE_CAP: usize = 400;

/// How many families the resolver may fall back onto when the chain fails.
const SHORTLIST_LEN: usize = 12;

fn name_bonus(lower_family: &str) -> u32 {
    let mut best = 0;
    for (hint, bonus) in NAME_HINTS {
        if lower_family.contains(hint) && *bonus > best {
            best = *bonus;
        }
    }
    best
}

/// Precomputed, short candidate list for the "nothing in the chain covers
/// this" case. Built at most once, and only from name-matched families.
fn shortlist() -> &'static [String] {
    static SHORTLIST: OnceLock<Vec<String>> = OnceLock::new();
    SHORTLIST.get_or_init(|| {
        let mut candidates: Vec<&FamilyEntry> = families()
            .iter()
            .filter(|e| name_bonus(&e.family.to_lowercase()) > 0)
            .collect();
        // `families()` is already sorted by name, so this is deterministic.
        candidates.truncate(SHORTLIST_PROBE_CAP);

        let mut scored: Vec<(u32, String)> = candidates
            .iter()
            .filter_map(|e| {
                let classes = classes_of(e);
                if classes.is_empty() {
                    return None;
                }
                let bonus = name_bonus(&e.family.to_lowercase());
                Some((bonus * 100 + classes.len() as u32, e.family.clone()))
            })
            .collect();
        scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
        scored.dedup_by(|a, b| a.1 == b.1);

        scored
            .into_iter()
            .take(SHORTLIST_LEN)
            .map(|(_, family)| family)
            .collect()
    })
}

/// Which regional flavour of a shared CJK font the text calls for.
///
/// A single `Noto Sans CJK` family is installed once per locale and its family
/// names differ only by an `HK`/`JP`/`KR`/`SC`/`TC` suffix, but the glyph forms
/// inside are region-specific. Picking the wrong one renders Japanese kanji with
/// Hong Kong forms, so text that actually contains kana (or Hangul) gets to
/// prefer the matching region over the alphabetical tie-break.
/// Han-only text carries no such signal and keeps the existing behaviour.
fn region_of(text: &str) -> Option<&'static str> {
    let mut hangul = false;
    for c in text.chars() {
        match c as u32 {
            // Hiragana, Katakana, Katakana phonetic extensions, halfwidth Katakana.
            0x3041..=0x30FF | 0x31F0..=0x31FF | 0xFF66..=0xFF9D => return Some("jp"),
            0x1100..=0x11FF | 0x3130..=0x318F | 0xA960..=0xA97F | 0xAC00..=0xD7AF
            | 0xD7B0..=0xD7FF | 0xFFA0..=0xFFDC => hangul = true,
            _ => {}
        }
    }
    if hangul { Some("kr") } else { None }
}

/// The shortlist with families whose name mentions `token` hoisted to the front.
///
/// This is a pure reordering: it never drops a candidate, so a region-mismatched
/// family is still tried (and still returns a covering font) if the preferred one
/// does not exist on this machine.
fn hoisted_shortlist(token: &str) -> Vec<String> {
    let mut hoisted = Vec::new();
    let mut rest = Vec::new();
    for family in shortlist() {
        if family.to_lowercase().contains(token) {
            hoisted.push(family.clone());
        } else {
            rest.push(family.clone());
        }
    }
    hoisted.extend(rest);
    hoisted
}

/// Region-preferred shortlist for `region`, or empty to use [`shortlist`] as-is.
fn region_first(region: Option<&'static str>) -> &'static [String] {
    match region {
        Some("jp") => {
            static LIST: OnceLock<Vec<String>> = OnceLock::new();
            LIST.get_or_init(|| hoisted_shortlist("jp"))
        }
        Some("kr") => {
            static LIST: OnceLock<Vec<String>> = OnceLock::new();
            LIST.get_or_init(|| hoisted_shortlist("kr"))
        }
        _ => &[],
    }
}

/// A proposed default chain for the settings UI, built by inspecting
/// what is actually installed (e.g. a CJK font if one is present).
pub fn suggested_chain() -> Vec<String> {
    let mut chain: Vec<String> = Vec::new();
    for wanted in [
        ScriptClass::Cjk,
        ScriptClass::Hangul,
        ScriptClass::Emoji,
        ScriptClass::NerdSymbols,
        ScriptClass::Arabic,
        ScriptClass::Devanagari,
        ScriptClass::Thai,
    ] {
        for family in shortlist() {
            let Some(entry) = entry(family) else {
                continue;
            };
            if classes_of(entry).contains(&wanted) && !chain.contains(family) {
                chain.push(family.clone());
                break;
            }
        }
    }
    chain
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_is_never_classified() {
        for c in ('\0'..='\u{7F}').filter(|c| c.is_ascii()) {
            assert_eq!(script_class_of(c), None, "ASCII {:?} must not classify", c);
        }
        assert_eq!(script_class_of('é'), None, "Latin-1 must not classify");
        assert_eq!(script_class_of('\u{200B}'), None, "zero-width must not classify");
        assert_eq!(script_class_of('—'), None, "general punctuation must not classify");
    }

    #[test]
    fn every_class_has_a_representative() {
        // `Latin` is deliberately absent: it is a property a *font* can have
        // (see PROBES) but no codepoint is classified as it, so ordinary text
        // never triggers a fallback.
        let samples = [
            (ScriptClass::Cjk, '漢'),
            (ScriptClass::Cjk, 'あ'),
            (ScriptClass::Cjk, 'ア'),
            (ScriptClass::Cjk, '\u{20000}'),
            (ScriptClass::Hangul, '한'),
            (ScriptClass::Hangul, '\u{1100}'),
            (ScriptClass::Arabic, 'م'),
            (ScriptClass::Cyrillic, 'Ж'),
            (ScriptClass::Greek, 'Ω'),
            (ScriptClass::Hebrew, 'א'),
            (ScriptClass::Devanagari, 'क'),
            (ScriptClass::Thai, 'ก'),
            (ScriptClass::Emoji, '\u{1F600}'),
            (ScriptClass::Emoji, '❤'),
            (ScriptClass::NerdSymbols, '\u{E0B0}'),
            (ScriptClass::NerdSymbols, '\u{F0001}'),
        ];
        for (expected, c) in samples {
            assert_eq!(script_class_of(c), Some(expected), "{:?} U+{:04X}", expected, c as u32);
        }
    }

    #[test]
    fn every_enum_variant_has_a_label_and_sample() {
        let all = [
            ScriptClass::Latin,
            ScriptClass::Cjk,
            ScriptClass::Hangul,
            ScriptClass::Arabic,
            ScriptClass::Cyrillic,
            ScriptClass::Greek,
            ScriptClass::Hebrew,
            ScriptClass::Devanagari,
            ScriptClass::Thai,
            ScriptClass::Emoji,
            ScriptClass::NerdSymbols,
        ];
        for class in all {
            assert!(!class.label().is_empty());
            assert!(!class.sample().is_empty());
            assert_eq!(class.sample().chars().count(), 1);
        }
        assert_eq!(ScriptClass::Cjk.label(), "Japanese/Chinese");
    }

    #[test]
    fn latin_only_string_has_no_missing_scripts() {
        assert!(missing_scripts("Hello, world! 123", "NoSuchFontFamily").is_empty());
        assert!(missing_scripts("naïve café Ærø", "NoSuchFontFamily").is_empty());
        assert!(missing_scripts("", "NoSuchFontFamily").is_empty());
    }

    #[test]
    fn unknown_family_covers_nothing() {
        assert!(!family_covers("Definitely Not Installed 12345", 'A'));
        assert!(!family_covers("Definitely Not Installed 12345", '漢'));
    }

    #[test]
    fn pick_font_for_returns_primary_when_chain_is_empty() {
        assert_eq!(pick_font_for("Hello", "Some Primary", &[]), "Some Primary");
        assert_eq!(pick_font_for("", "Some Primary", &[]), "Some Primary");
        assert_eq!(
            pick_font_for("Hello", "Some Primary", &["A".into(), "B".into()]),
            "Some Primary"
        );
    }

    #[test]
    fn empty_chain_returns_primary_or_a_font_that_really_covers() {
        // With no chain entries the resolver may still auto-discover a covering
        // family, which is machine-dependent. Assert the contract instead of a
        // fixed string: whatever comes back must be the primary, or a family
        // that genuinely covers the text.
        let primary = "Definitely Not Installed 12345";
        for text in ["音楽", "한글", "Привет", "😀"] {
            let picked = pick_font_for(text, primary, &[]);
            assert!(
                picked == primary || family_covers(&picked, text.chars().next().unwrap()),
                "picked {:?} for {:?} but it does not cover it",
                picked,
                text
            );
        }
    }

    #[test]
    fn chain_entry_beats_auto_discovery() {
        let Some(covering) = fonts()
            .iter()
            .find(|f| family_covers(&f.family, '音'))
            .map(|f| f.family.clone())
        else {
            return; // No CJK font on this machine; nothing to assert.
        };
        let chain = vec!["Definitely Not Installed 12345".to_string(), covering.clone()];
        assert_eq!(pick_font_for("音楽", "Some Primary", &chain), covering);
    }

    #[test]
    fn fonts_are_sorted_and_deduplicated() {
        let list = fonts();
        for pair in list.windows(2) {
            assert!(
                pair[0].family < pair[1].family,
                "fonts() not strictly sorted: {:?} then {:?}",
                pair[0].family,
                pair[1].family
            );
        }
        for info in list {
            assert!(!info.family.is_empty());
            for pair in info.classes.windows(2) {
                assert!(pair[0] < pair[1], "classes not sorted for {:?}", info.family);
            }
        }
    }

    #[test]
    fn suggested_chain_is_small_and_deduplicated() {
        let chain = suggested_chain();
        assert!(chain.len() <= SHORTLIST_LEN);
        let mut sorted = chain.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), chain.len());
    }

    #[test]
    fn shortlist_is_bounded() {
        assert!(shortlist().len() <= SHORTLIST_LEN);
    }
}

#[cfg(test)]
mod region_regression {
    use super::*;

    /// The chain walk used to ignore region entirely, so an alphabetically
    /// sorted chain (`suggested_chain()` yields "…HK" before "…JP") rendered
    /// Japanese kanji with Hong Kong forms. These assertions are pure name
    /// matching, so they hold on machines with no CJK font installed.
    #[test]
    fn region_detection_reads_kana_and_hangul() {
        assert_eq!(region_of("君が代"), Some("jp"));
        assert_eq!(region_of("Song 君が代"), Some("jp"));
        assert_eq!(region_of("한글 노래"), Some("kr"));
        // Han-only text carries no regional signal and must not force a choice.
        assert_eq!(region_of("茉莉花"), None);
        assert_eq!(region_of("Bohemian Rhapsody"), None);
    }

    #[test]
    fn family_names_match_their_region() {
        assert!(name_has_region("Noto Sans CJK JP", "jp"));
        assert!(name_has_region("Noto Sans CJK KR", "kr"));
        assert!(!name_has_region("Noto Sans CJK HK", "jp"));
        assert!(!name_has_region("Noto Sans CJK HK", "kr"));
        assert!(name_has_region("Source Han Sans KR", "kr"));
    }
}
