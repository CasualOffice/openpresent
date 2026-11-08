//! What a distributable has to carry, and how to notice when it does not.
//!
//! Copied from OpenCalc's `desktop/src/assets.rs` at `5d42a07`
//! (desktop/PROVENANCE.md), with the engine's file names changed.
//!
//! `tauri::generate_context!` embeds the staged `webapp/` at compile time, and
//! the one thing not in the source tree is `webapp/pkg/`, the WebAssembly
//! engine. Without it the build succeeds and the app opens a window that cannot
//! open a deck. `build.rs` includes this file directly, so the list is kept once.

use std::path::{Path, PathBuf};

/// Files the webview cannot work without, relative to `webapp/`.
///
/// The page, its entry module, the worker, and the engine it loads. A missing
/// stylesheet is ugly; a missing `.wasm` is a viewer that opens nothing.
pub const REQUIRED_WEB_ASSETS: &[&str] = &[
    "editor.html",
    "editor.js",
    "render.worker.js",
    "pkg/casual_slide_wasm.js",
    "pkg/casual_slide_wasm_bg.wasm",
];

/// Which of [`REQUIRED_WEB_ASSETS`] are not in `dist`.
///
/// Returned rather than reported, so the caller decides whether a missing file
/// is a warning (a developer build) or an error (a release build).
pub fn missing_web_assets(dist: &Path) -> Vec<PathBuf> {
    REQUIRED_WEB_ASSETS
        .iter()
        .map(|rel| dist.join(rel))
        .filter(|path| !path.exists())
        .collect()
}

/// The command that builds the missing engine, copy-pasteable.
pub fn how_to_build_web_assets() -> &'static str {
    "wasm-pack build crates/casual-slide-wasm --release --target web \
     --out-dir \"$PWD/webapp/pkg\" (see webapp/README.md)"
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn scratch(tag: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("openpresent-desktop-{tag}-{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_tree_without_the_wasm_build_is_caught() {
        // A fresh checkout: every hand-written file present, `pkg/` absent.
        let dir = scratch("no-pkg");
        for rel in ["editor.html", "editor.js", "render.worker.js"] {
            fs::write(dir.join(rel), "x").unwrap();
        }
        let names: Vec<String> = missing_web_assets(&dir)
            .iter()
            .map(|p| {
                p.strip_prefix(&dir)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect();
        assert_eq!(
            names,
            ["pkg/casual_slide_wasm.js", "pkg/casual_slide_wasm_bg.wasm"]
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_complete_tree_is_not_complained_about() {
        let dir = scratch("complete");
        fs::create_dir_all(dir.join("pkg")).unwrap();
        for rel in REQUIRED_WEB_ASSETS {
            fs::write(dir.join(rel), "x").unwrap();
        }
        assert!(missing_web_assets(&dir).is_empty());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn the_remedy_names_this_engine() {
        assert!(how_to_build_web_assets().starts_with("wasm-pack build crates/casual-slide-wasm"));
    }
}
