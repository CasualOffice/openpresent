//! OPC part-name equivalence, per ECMA-376 Part 2.
//!
//! A relationship target and a ZIP item name for one part can be spelled
//! differently: `../media/my image.png` against `ppt/media/my%20image.png`, or
//! `é.png` against `%C3%A9.png`. Part 2 maps ZIP item names to Unicode part names
//! by un-percent-encoding (7.3.5), and compares them by ASCII case-insensitive
//! matching over code points (6.2.2.3). So `É` and `é` are different parts, and so
//! are the NFC and NFD spellings of one letter, which the standard only recommends
//! a package avoid.
//!
//! Decoding is exactly the step where a disguised path turns into a real one, so
//! everything that decodes to traversal is refused rather than normalized, at
//! every layer of encoding, for whichever reader decodes once more than we do.
//!
//! This is the one rule: the package's own index, its rewrites, and the part graph
//! in `casual-slide-ooxml` all compare names through it (`PKG-02`).

/// More layers of percent-encoding than this is not a name anyone wrote.
pub(crate) const MAX_ENCODING_LAYERS: usize = 8;

/// Marks an entry with no valid part name, so no target can ever reach it.
const UNREACHABLE: char = '\0';

/// Characters a later normalizer or a file system may turn into a separator.
const SLASH_LOOKALIKES: [char; 6] = [
    '\u{2044}', '\u{2215}', '\u{29F8}', '\u{FF0F}', '\u{FE68}', '\u{FF3C}',
];

/// Characters compatibility normalization turns into one or more dots.
const DOT_LOOKALIKES: [char; 5] = ['\u{2024}', '\u{2025}', '\u{2026}', '\u{FE52}', '\u{FF0E}'];

/// The form two equivalent part names share, or `None` for a name to be refused.
///
/// Takes a ZIP item name, a part name with its leading slash, or a target already
/// resolved against its source. Percent-encoding is removed once, as the standard
/// has it, then ASCII letters are lowered. Refused: percent-encoded dots and
/// separators in any layer (`%2e`, `%252e`), bytes that are not UTF-8, which covers
/// overlong forms such as `%c0%ae`, empty segments, segments ending in a dot,
/// backslashes, control characters, and look-alikes of `/` and `..`.
pub fn part_name_key(name: &str) -> Option<String> {
    let name = name.strip_prefix('/').unwrap_or(name);
    if has_encoded_structure(name) {
        return None;
    }
    let decoded = percent_decode(name)?;
    if !segments_are_safe(&decoded) {
        return None;
    }
    let mut layer = decoded.clone();
    let mut layers = 0;
    while layer.contains('%') {
        if has_encoded_structure(&layer) {
            return None;
        }
        let next = percent_decode(&layer)?;
        if next == layer {
            break;
        }
        if !segments_are_safe(&next) {
            return None;
        }
        layers += 1;
        if layers >= MAX_ENCODING_LAYERS {
            return None;
        }
        layer = next;
    }
    Some(decoded.to_ascii_lowercase())
}

/// Whether two names denote one part. A refused name is equivalent to nothing.
pub fn part_names_equivalent(a: &str, b: &str) -> bool {
    match (part_name_key(a), part_name_key(b)) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}

/// [`part_name_key`] for a name that has to be stored somewhere: an entry with no
/// valid name still has to be retained, read by its own name, and reported, while
/// no target resolves to it. The empty name is the package itself.
pub fn part_entry_key(name: &str) -> String {
    if name.is_empty() {
        return String::new();
    }
    part_name_key(name).unwrap_or_else(|| format!("{UNREACHABLE}{}", name.to_ascii_lowercase()))
}

fn has_encoded_structure(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    ["%2e", "%2f", "%5c"].iter().any(|e| lower.contains(e))
}

/// One layer of percent-decoding. A `%` not followed by two hex digits stands for
/// itself, as Part 2's Annex A has `/%XY.xml` and `/%25XY.xml` name one part.
fn percent_decode(name: &str) -> Option<String> {
    let bytes = name.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut rest = bytes;
    while let Some((&byte, tail)) = rest.split_first() {
        if byte == b'%'
            && let [high, low, after @ ..] = tail
            && let (Some(high), Some(low)) = (hex(*high), hex(*low))
        {
            out.push(high * 16 + low);
            rest = after;
        } else {
            out.push(byte);
            rest = tail;
        }
    }
    String::from_utf8(out).ok()
}

fn hex(byte: u8) -> Option<u8> {
    char::from(byte)
        .to_digit(16)
        .and_then(|d| u8::try_from(d).ok())
}

fn segments_are_safe(name: &str) -> bool {
    name.split('/').all(|segment| {
        !segment.is_empty()
            && !segment.ends_with('.')
            && !segment.contains(SLASH_LOOKALIKES)
            && !segment.chars().any(|c| c == '\\' || c.is_control())
            // Windows drops trailing spaces and dots, so `.. ` is `..` to it.
            && !segment
                .chars()
                .all(|c| matches!(c, '.' | ' ') || DOT_LOOKALIKES.contains(&c))
    })
}
