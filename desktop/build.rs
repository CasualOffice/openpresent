//! The build script, and the one thing it refuses to let through.
//!
//! Copied from OpenCalc's `desktop/build.rs` at `5d42a07` (desktop/PROVENANCE.md).
//!
//! `frontendDist` is `web-dist`, staged from `../webapp` below, and
//! `tauri::generate_context!` embeds that directory into the binary. Everything
//! the webview needs travels with the app, except `webapp/pkg/`, which is
//! `wasm-pack` output, git-ignored, and missing from every fresh checkout.
//! Nothing errors when it is absent: the build is green, the bundle is produced,
//! the window opens, and the worker's import of the engine 404s. A blank
//! window, from a passing build.
//!
//! So a release build missing the engine fails here. A debug build only warns,
//! so `cargo check` and `cargo test` do not need `wasm-pack`.

// The list of required files, shared with the library rather than restated.
#[path = "src/assets.rs"]
mod assets;

use std::path::Path;

/// Set to skip the check, for a build nobody will run.
const OVERRIDE: &str = "OPENPRESENT_SKIP_ASSET_CHECK";

/// Where `frontendDist` points.
const STAGED: &str = "web-dist";

/// Never copied into the app.
const LEFT_OUT: &[&str] = &["__pycache__"];

fn main() {
    let dist = Path::new("../webapp");
    println!("cargo:rerun-if-changed=src/assets.rs");
    println!("cargo:rerun-if-env-changed={OVERRIDE}");
    for rel in assets::REQUIRED_WEB_ASSETS {
        println!("cargo:rerun-if-changed=../webapp/{rel}");
    }

    let missing = assets::missing_web_assets(dist);
    if !missing.is_empty() && std::env::var_os(OVERRIDE).is_none() {
        let list = missing
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let how = assets::how_to_build_web_assets();
        if std::env::var("PROFILE").as_deref() == Ok("release") {
            panic!(
                "the webview assets this app embeds are missing: {list}\n\
                 A release build without them opens a blank window. Build them first:\n  {how}\n\
                 Set {OVERRIDE}=1 to build anyway."
            );
        }
        println!(
            "cargo:warning=missing webview assets ({list}); the app will open a blank window. \
             Build them with: {how}"
        );
    }

    stage_web_dist(dist, Path::new(STAGED));
    tauri_build::build();
}

/// Stage `webapp/` into `web-dist/` here rather than in a `beforeBuildCommand`,
/// because `generate_context!` reads `frontendDist` at compile time for `cargo
/// check` and `cargo clippy` too, and a `beforeBuildCommand` runs only under
/// `cargo tauri build` (OpenCalc learned this from a red CI run).
fn stage_web_dist(src: &Path, dest: &Path) {
    println!("cargo:rerun-if-changed=../webapp");
    if dest.exists() {
        std::fs::remove_dir_all(dest).expect("clear desktop/web-dist");
    }
    std::fs::create_dir_all(dest).expect("create desktop/web-dist");
    copy_tree(src, dest, Path::new(""));
}

fn copy_tree(src: &Path, dest: &Path, rel: &Path) {
    let Ok(entries) = std::fs::read_dir(src.join(rel)) else {
        return;
    };
    let mut entries: Vec<_> = entries.flatten().collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let rel = rel.join(entry.file_name());
        let shown = rel.to_string_lossy().replace('\\', "/");
        if shown.split('/').any(|part| LEFT_OUT.contains(&part)) {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            copy_tree(src, dest, &rel);
        } else {
            let target = dest.join(&rel);
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent).expect("create a web-dist directory");
            }
            std::fs::copy(&path, &target).expect("copy a web asset into web-dist");
        }
    }
}
