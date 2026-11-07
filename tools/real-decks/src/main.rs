//! F-007: the engine over real decks, stage by stage.
//!
//! Usage: `casual-slide-real-decks <out-dir> <deck>...`
//!
//! For each deck, one JSON line on stdout, with each stage's result:
//!
//! 1. `open`: admission and the part graph, with the finding codes.
//! 2. `import`: `import_deck` and every theme part through `import_theme`.
//! 3. `unedited_save`: `save_deck` of the unedited model returns the input bytes and
//!    changes no part (F1 through the writer).
//! 4. `forced_rewrite`: every XML part parsed into the tree and written back, saved,
//!    reopened, and compared part by part with the input (WR-01). The rewritten deck is
//!    written to `<out-dir>/<name>` for F4 and the SDK validator to compare.
//!
//! A stage that panics is caught and reported as `panic`, which is itself a defect:
//! untrusted input must never panic. Messages are the engine's own, which carry part
//! names and counts, never document text (SEC-015 records where that is broken).

use std::fmt::Write as _;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;

use casual_slide_export::presentation::save_deck;
use casual_slide_import::presentation::import_deck;
use casual_slide_import::properties::ImportLimits;
use casual_slide_import::theme::{
    THEME_OVERRIDE_CONTENT_TYPE, import_theme, import_theme_override, read_theme_parts,
};
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage, XmlTree};
use casual_slide_package::PartEdits;

fn json_str(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// A stage's outcome: `ok` with a detail, `fail` with a message, or `panic`.
struct Stage {
    name: &'static str,
    status: &'static str,
    detail: String,
}

fn run<T>(
    name: &'static str,
    f: impl FnOnce() -> Result<(T, String), String>,
) -> (Option<T>, Stage) {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(Ok((value, detail))) => (
            Some(value),
            Stage {
                name,
                status: "ok",
                detail,
            },
        ),
        Ok(Err(detail)) => (
            None,
            Stage {
                name,
                status: "fail",
                detail,
            },
        ),
        Err(payload) => {
            let detail = payload
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| payload.downcast_ref::<&str>().map(|s| (*s).to_owned()))
                .unwrap_or_else(|| "panic".to_owned());
            (
                None,
                Stage {
                    name,
                    status: "panic",
                    detail,
                },
            )
        }
    }
}

fn check(path: &Path, out_dir: &Path) -> Vec<Stage> {
    let mut stages = Vec::new();
    let source = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) => {
            stages.push(Stage {
                name: "read",
                status: "fail",
                detail: e.kind().to_string(),
            });
            return stages;
        }
    };
    let limits = ImportLimits::default();

    let (pkg, stage) = run("open", || {
        let pkg = PresentationPackage::open(source.clone(), OoxmlLimits::default())
            .map_err(|e| format!("[{}] {e}", e.code()))?;
        let mut codes: Vec<&str> = pkg.findings().iter().map(|f| f.kind.code()).collect();
        codes.sort_unstable();
        codes.dedup();
        let detail = format!(
            "{} parts; findings {}",
            pkg.part_names().len(),
            codes.join(" ")
        );
        Ok((pkg, detail))
    });
    stages.push(stage);
    let Some(mut pkg) = pkg else { return stages };

    let (deck, stage) = run("import", || {
        let imported = import_deck(&mut pkg, &limits).map_err(|e| format!("[{}] {e}", e.code()))?;
        let mut theme_codes = Vec::new();
        let themes = read_theme_parts(&mut pkg).map_err(|e| format!("theme parts: {e}"))?;
        for (name, tree) in &themes {
            if name.contains("/theme/") {
                // By content type, as the crate selects them: PowerPoint keeps overrides
                // in ppt/theme/ too, so the directory cannot tell them apart.
                let codes =
                    if pkg.content_types().resolve(name) == Some(THEME_OVERRIDE_CONTENT_TYPE) {
                        import_theme_override(tree)
                            .map_err(|e| format!("{name}: {e}"))?
                            .findings
                            .codes()
                    } else {
                        import_theme(tree)
                            .map_err(|e| format!("{name}: {e}"))?
                            .findings
                            .codes()
                    };
                theme_codes.extend(codes);
            }
        }
        theme_codes.sort_unstable();
        theme_codes.dedup();
        let mut codes: Vec<&str> = imported.findings.iter().map(|f| f.kind.code()).collect();
        codes.sort_unstable();
        codes.dedup();
        let detail = format!(
            "{} slides; findings {}; {} theme parts, findings {}",
            imported
                .deck
                .presentation
                .slides
                .as_ref()
                .map_or(0, Vec::len),
            codes.join(" "),
            themes.len(),
            theme_codes.join(" ")
        );
        Ok((imported.deck, detail))
    });
    stages.push(stage);

    if let Some(deck) = deck {
        let (_, stage) = run("unedited_save", || {
            let mut fresh = PresentationPackage::open(source.clone(), OoxmlLimits::default())
                .map_err(|e| e.to_string())?;
            let outcome =
                save_deck(&mut fresh, &deck, &limits).map_err(|e| format!("[{}] {e}", e.code()))?;
            if !outcome.changed_parts.is_empty() {
                return Err(format!(
                    "changed parts: {}",
                    outcome.changed_parts.join(" ")
                ));
            }
            if outcome.bytes != source {
                return Err("bytes differ from the input".to_owned());
            }
            Ok(((), String::new()))
        });
        stages.push(stage);
    }

    let (_, stage) = run("forced_rewrite", || {
        let xml_limits = OoxmlLimits::default();
        let mut before =
            PresentationPackage::open(source.clone(), xml_limits).map_err(|e| e.to_string())?;
        let mut edits = PartEdits::default();
        let mut parts = 0;
        for name in before.part_names() {
            if name.ends_with(".xml") || name.ends_with(".rels") {
                let bytes = before
                    .read_part(&name)
                    .map_err(|e| format!("{name}: {e}"))?;
                let tree = XmlTree::parse(&bytes, &name, &xml_limits)
                    .map_err(|e| format!("[{}] {e}", e.code()))?;
                let written = tree.to_bytes();
                if written != bytes {
                    return Err(format!("{name}: the tree does not write back its input"));
                }
                edits.replace(name, written);
                parts += 1;
            }
        }
        let rewritten = before.save_with(&edits).map_err(|e| e.to_string())?;
        let mut after = PresentationPackage::open(rewritten.clone(), xml_limits)
            .map_err(|e| format!("reopen: {e}"))?;
        if before.part_names() != after.part_names() {
            return Err("part names differ after the rewrite".to_owned());
        }
        if before.findings() != after.findings() {
            return Err("findings differ after the rewrite".to_owned());
        }
        for name in before.part_names() {
            let a = before.read_part(&name).map_err(|e| e.to_string())?;
            let b = after.read_part(&name).map_err(|e| e.to_string())?;
            if a != b {
                return Err(format!("{name}: bytes differ after the rewrite"));
            }
        }
        let file = path.file_name().ok_or("no file name")?;
        std::fs::write(out_dir.join(file), &rewritten).map_err(|e| e.kind().to_string())?;
        Ok(((), format!("{parts} XML parts")))
    });
    stages.push(stage);
    stages
}

fn main() {
    // Panics are caught per stage and reported; the default hook would print them twice.
    std::panic::set_hook(Box::new(|_| {}));
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some((out, decks)) = args.split_first() else {
        eprintln!("usage: casual-slide-real-decks <out-dir> <deck>...");
        std::process::exit(2);
    };
    let out_dir = Path::new(out);
    let mut failed = false;
    for deck in decks {
        let path = Path::new(deck);
        let stages = check(path, out_dir);
        failed |= stages.iter().any(|s| s.status != "ok");
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let body: Vec<String> = stages
            .iter()
            .map(|s| {
                format!(
                    "{}: {{\"status\": {}, \"detail\": {}}}",
                    json_str(s.name),
                    json_str(s.status),
                    json_str(&s.detail)
                )
            })
            .collect();
        println!("{{\"deck\": {}, {}}}", json_str(&name), body.join(", "));
    }
    std::process::exit(i32::from(failed));
}
