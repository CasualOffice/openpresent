//! Japanese, Chinese and Korean faces for the editor (`T2D1-04`).
//!
//! Started from OpenCalc's `crates/casual-calc-wasm/src/fonts.rs` (owner
//! instructions 42 and 54): `register_font_slice` and `set_cjk_default_region` as
//! there. The faces are **not in this binary**: they are WOFF2 slices under
//! `fonts/cjk/`, served as static files. OpenCalc's canvas draws cell text with the
//! browser, which fetches slices through `@font-face`; here the engine draws the
//! slide, so the page asks [`slide_font_needs`] after drawing which slices the
//! slide wanted, fetches those files, hands them to [`register_font_slice`] and
//! draws again: OpenCalc's PDF-export loop (`session_export_pdf_font_needs`).

use super::*;
use casual_slide_text::cjk::{Region, Variant, set_default_region};

/// Set the region Han characters in a Latin-font run default to, as the system
/// locale does for PowerPoint. Accepts `jp`/`sc`/`tc`/`kr` or a language tag (`ja`,
/// `zh-CN`, `zh-TW`, `ko`). Returns whether it was understood; an unknown value
/// leaves the default as it was.
#[wasm_bindgen]
#[must_use]
pub fn set_cjk_default_region(code: &str) -> bool {
    Region::parse(code).map(set_default_region).is_some()
}

/// Register one CJK font slice, as fetched: `variant` is its directory
/// (`jp-sans-400`), `bytes` the WOFF2 file. Returns how many characters it maps.
///
/// # Errors
///
/// An unknown variant, bytes that are not a readable face (a 404 page, a truncated
/// download), or a registry past its size cap: each refused rather than stored.
#[wasm_bindgen]
pub fn register_font_slice(variant: &str, bytes: &[u8]) -> Result<u32, JsError> {
    let variant =
        Variant::from_key(variant).ok_or_else(|| JsError::new("unknown CJK font variant"))?;
    let chars = casual_slide_text::cjk::register_slice(variant, bytes)
        .map_err(|e| JsError::new(&e.to_string()))?;
    Ok(u32::try_from(chars).unwrap_or(u32::MAX))
}

/// The CJK slices the last drawn slide wanted and did not have, as JSON
/// `[{"chain":["jp-sans-400","sc-sans-400",…],"chars":"日本語"}]`, OpenCalc's shape.
///
/// `chain` is where to look for each character, in order: the variant meant for
/// it, then the same style and weight in the other regions. A host fetches, for
/// each character, the slice of the first variant in the chain whose ranges hold
/// it, registers them, and draws again; an empty array means every character was
/// drawn in the face meant for it. Empty, too, when there is no deck.
#[wasm_bindgen]
#[must_use]
pub fn slide_font_needs() -> String {
    let wanted = with_viewer(|v| Ok(v.cjk_wanted.clone())).unwrap_or_default();
    font_needs_json(&wanted)
}

/// [`slide_font_needs`]'s JSON for a set of wanted characters.
pub(crate) fn font_needs_json(wanted: &std::collections::BTreeSet<(Variant, char)>) -> String {
    let mut by_variant: BTreeMap<Variant, String> = BTreeMap::new();
    for &(variant, ch) in wanted {
        by_variant.entry(variant).or_default().push(ch);
    }
    let items: Vec<String> = by_variant
        .iter()
        .map(|(variant, chars)| {
            let chain: Vec<String> = variant
                .chain()
                .iter()
                .map(|v| format!("\"{}\"", v.key()))
                .collect();
            format!(
                "{{\"chain\":[{}],\"chars\":{}}}",
                chain.join(","),
                format::json_string(chars)
            )
        })
        .collect();
    format!("[{}]", items.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ink(png: &[u8], x0: u32, x1: u32, y0: u32, y1: u32) -> usize {
        let image = tiny_skia::Pixmap::decode_png(png).unwrap();
        (y0..y1)
            .step_by(2)
            .flat_map(|y| (x0..x1).step_by(2).map(move |x| (x, y)))
            .filter(|&(x, y)| image.pixel(x, y).unwrap().demultiply().red() < 100)
            .count()
    }

    /// The editor draws Japanese once it loads the slices the drawn slide asked for
    /// (`T2D1-04`). The browser never registered a slice, so `z21464709-souzou-ja`'s
    /// grouped title drew nothing in the editor, while `render-deck`, which loads
    /// every slice the deck's characters name, drew it.
    #[test]
    fn a_japanese_slide_names_the_slices_it_wants_and_draws_once_they_are_registered() {
        let path = format!(
            "{}/../../fixtures/real/z21464709-souzou-ja.pptx",
            env!("CARGO_MANIFEST_DIR")
        );
        let mut v = Viewer::open(std::fs::read(path).unwrap()).unwrap();
        let png = v.render_png(0, 960).unwrap();
        assert!(!v.cjk_wanted.is_empty(), "the slide names no slice");
        let json = font_needs_json(&v.cjk_wanted);
        assert!(
            json.starts_with("[{\"chain\":[\"jp-sans-") && json.contains("\"chars\":\""),
            "{json}"
        );
        let before = ink(&png, 200, 370, 70, 120);

        let dir = casual_slide_text::cjk::SliceDirectory::open(format!(
            "{}/../../webapp/fonts/cjk",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        assert!(dir.load(v.cjk_wanted.clone()) > 0);
        let png = v.render_png(0, 960).unwrap();
        let after = ink(&png, 200, 370, 70, 120);
        assert!(
            after > 100 && after > before + 50,
            "title ink {before} before the slices, {after} after"
        );
        assert!(v.cjk_wanted.is_empty(), "still wanted: {:?}", v.cjk_wanted);
    }
}
