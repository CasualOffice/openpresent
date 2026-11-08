//! Real decks to PDF (`VS-04`): a page per slide, and the text read back out of
//! each page equals the text the slide draws. Runs on GitHub CI (owner
//! instruction 38).
//!
//! The text is read back by a small reader of this writer's own output, the way a
//! viewer does: glyph ids through each font's `ToUnicode` map, and a run's
//! `/ActualText` where it has one.

use std::collections::BTreeMap;
use std::path::Path;

use casual_slide_render::display::DisplayItem;
use casual_slide_render::{Deck, PdfMetadata};

fn open(name: &str) -> Deck {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/generated")
        .join(name);
    Deck::open(std::fs::read(path).unwrap()).unwrap()
}

fn find(hay: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    hay.get(from..)?
        .windows(needle.len())
        .position(|w| w == needle)
        .map(|p| p + from)
}

/// Every indirect object's body, by number.
fn objects(pdf: &[u8]) -> BTreeMap<usize, Vec<u8>> {
    let mut out = BTreeMap::new();
    let mut at = 0;
    while let Some(p) = find(pdf, b" 0 obj\n", at) {
        let line = pdf[..p]
            .iter()
            .rposition(|&b| b == b'\n')
            .map_or(0, |i| i + 1);
        let id: usize = std::str::from_utf8(&pdf[line..p]).unwrap().parse().unwrap();
        let start = p + b" 0 obj\n".len();
        let end = find(pdf, b"\nendobj\n", start).unwrap();
        out.insert(id, pdf[start..end].to_vec());
        at = end;
    }
    out
}

fn text(body: &[u8]) -> String {
    String::from_utf8_lossy(body).into_owned()
}

/// The object number after `key` in `body`: `key N 0 R`.
fn reference(body: &str, key: &str) -> Option<usize> {
    let at = body.find(key)? + key.len();
    body[at..].split_whitespace().next()?.parse().ok()
}

fn stream(body: &[u8]) -> Vec<u8> {
    let start = find(body, b"stream\n", 0).unwrap() + b"stream\n".len();
    let end = find(body, b"\nendstream", start).unwrap();
    body[start..end].to_vec()
}

fn hex_utf16(hex: &str) -> String {
    let units: Vec<u16> = hex
        .as_bytes()
        .chunks(4)
        .map(|c| u16::from_str_radix(std::str::from_utf8(c).unwrap(), 16).unwrap())
        .collect();
    String::from_utf16(&units).unwrap()
}

/// The text of each page, in page order.
fn extract(pdf: &[u8]) -> Vec<String> {
    let objs = objects(pdf);
    let body = |id: usize| text(&objs[&id]);
    let pages_obj = objs
        .values()
        .map(|b| text(b))
        .find(|b| b.contains("/Type /Pages"))
        .unwrap();
    let kids_at = pages_obj.find("/Kids [").unwrap() + "/Kids [".len();
    let kids_end = kids_at + pages_obj[kids_at..].find(']').unwrap();
    let kids: Vec<usize> = pages_obj[kids_at..kids_end]
        .split(" 0 R")
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    // Fonts by resource index, each its glyph-to-text map.
    let resources = objs
        .values()
        .map(|b| text(b))
        .find(|b| b.contains("/Font << "))
        .unwrap();
    let mut fonts: BTreeMap<String, BTreeMap<u32, String>> = BTreeMap::new();
    let fonts_at = resources.find("/Font << ").unwrap() + "/Font << ".len();
    let fonts_end = fonts_at + resources[fonts_at..].find(">>").unwrap();
    let tokens: Vec<&str> = resources[fonts_at..fonts_end].split_whitespace().collect();
    for entry in tokens.chunks(4) {
        let [name, id, _, _] = entry else { continue };
        let font = body(id.parse().unwrap());
        let cmap = text(&stream(&objs[&reference(&font, "/ToUnicode ").unwrap()]));
        let mut map = BTreeMap::new();
        for line in cmap.lines() {
            let Some(rest) = line.strip_prefix('<') else {
                continue;
            };
            let Some((gid, value)) = rest.split_once("> <") else {
                continue;
            };
            let value = value.trim_end_matches('>');
            map.insert(u32::from_str_radix(gid, 16).unwrap(), hex_utf16(value));
        }
        fonts.insert(name.trim_start_matches('/').to_owned(), map);
    }

    let mut out = Vec::new();
    for kid in kids {
        let page = body(kid);
        let content = text(&stream(&objs[&reference(&page, "/Contents ").unwrap()]));
        let mut page_text = String::new();
        let mut font = String::new();
        let mut actual = false;
        for line in content.lines() {
            if let Some(rest) = line.strip_prefix("/Span << /ActualText <FEFF") {
                page_text.push_str(&hex_utf16(rest.split('>').next().unwrap()));
                actual = true;
            } else if line == "EMC" {
                actual = false;
            } else if let Some(at) = line.find("BT /") {
                font = line[at + 4..].split_whitespace().next().unwrap().to_owned();
            } else if line.ends_with("TJ") && !actual {
                let map = &fonts[&font];
                for token in line.split('<').skip(1) {
                    let gid = u32::from_str_radix(token.split('>').next().unwrap(), 16).unwrap();
                    page_text.push_str(map.get(&gid).map_or("\u{fffd}", String::as_str));
                }
            }
        }
        out.push(page_text);
    }
    out
}

fn squeeze(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

/// The text each slide's display list draws, in drawing order.
fn drawn_text(deck: &Deck) -> Vec<String> {
    (0..deck.slide_count())
        .map(|i| {
            let (list, _) = deck.display_list(i).unwrap();
            list.items
                .iter()
                .filter_map(|item| match item {
                    DisplayItem::GlyphRun { text, glyphs, .. } if !glyphs.is_empty() => {
                        Some(text.as_str())
                    }
                    _ => None,
                })
                .collect()
        })
        .collect()
}

#[test]
fn the_text_read_back_from_each_page_is_the_text_the_slide_draws() {
    for name in ["area-text.pptx", "minimal.pptx", "chart-families.pptx"] {
        let mut deck = open(name);
        let expected = drawn_text(&deck);
        let (pdf, _) = deck.render_pdf(&PdfMetadata::default()).unwrap();
        let pages = extract(&pdf);
        assert_eq!(pages.len(), expected.len(), "{name}: a page per slide");
        for (i, (got, want)) in pages.iter().zip(&expected).enumerate() {
            assert_eq!(squeeze(got), squeeze(want), "{name}: slide {}", i + 1);
        }
    }
    let mut deck = open("area-text.pptx");
    let (pdf, _) = deck.render_pdf(&PdfMetadata::default()).unwrap();
    let first = &extract(&pdf)[0];
    assert!(
        squeeze(first).contains("Textproperties(1-35of100)"),
        "the slide's title extracts"
    );
}

#[test]
fn the_page_count_is_the_slide_count() {
    for name in [
        "area-text.pptx",
        "image.pptx",
        "preset-shapes.pptx",
        "rr-notes.pptx",
    ] {
        let mut deck = open(name);
        let slides = deck.slide_count();
        let (pdf, reports) = deck.render_pdf(&PdfMetadata::default()).unwrap();
        let text = String::from_utf8_lossy(&pdf);
        assert_eq!(reports.len(), slides, "{name}");
        assert_eq!(text.matches("/Type /Page ").count(), slides, "{name}");
        assert!(text.contains(&format!("/Count {slides} ")), "{name}");
        assert!(pdf.starts_with(b"%PDF-1.7\n"));
        assert!(pdf.ends_with(b"%%EOF\n"));
    }
}

#[test]
fn faces_are_embedded_as_subsets_and_the_same_deck_gives_the_same_bytes() {
    let mut deck = open("area-text.pptx");
    let (one, _) = deck.render_pdf(&PdfMetadata::default()).unwrap();
    let (two, _) = deck.render_pdf(&PdfMetadata::default()).unwrap();
    assert_eq!(one, two);
    let text = String::from_utf8_lossy(&one);
    assert!(text.contains("/FontFile2 "));
    assert!(text.contains("+OPF0"), "the first face is a subset");
    assert!(text.contains("/Subtype /CIDFontType2"));
}

#[test]
fn pictures_are_embedded_and_counted() {
    let mut deck = open("image.pptx");
    let (pdf, reports) = deck.render_pdf(&PdfMetadata::default()).unwrap();
    let drawn: u32 = reports.iter().map(|r| r.images.drawn).sum();
    let text = String::from_utf8_lossy(&pdf);
    assert_eq!(
        text.matches("/Subtype /Image").count() > 0,
        drawn > 0,
        "a drawn picture is an image object"
    );
}
