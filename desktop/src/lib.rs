//! The desktop shell's testable half: what the window's commands call, kept
//! apart from Tauri so `cargo test` proves it without a webview.
//!
//! Shaped after OpenCalc's `desktop/src/lib.rs` at `5d42a07`
//! (desktop/PROVENANCE.md). Unlike OpenCalc's, this shell holds no document:
//! the engine runs in the webview as WebAssembly, as on the web, and the shell
//! only reads the bytes of a file the platform named and hands them over.

pub mod assets;
pub mod launch;

/// The base name of a path, for the window and the editor to show.
///
/// Split on both separators whatever the platform, because a path from a
/// Windows panel and one from `argv` may differ in which they use.
#[must_use]
pub fn base_name(path: &str) -> String {
    path.rsplit(['/', '\\'])
        .next()
        .filter(|name| !name.is_empty())
        .unwrap_or("Presentation")
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_name_shown_is_the_last_component() {
        assert_eq!(base_name("/Users/a/Q3 review.pptx"), "Q3 review.pptx");
        assert_eq!(base_name(r"C:\Users\a\deck.pptx"), "deck.pptx");
        assert_eq!(base_name("deck.pptx"), "deck.pptx");
        assert_eq!(base_name("/"), "Presentation");
    }
}
