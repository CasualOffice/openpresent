//! `P1X-01`: one deck, one operation, one process, so whoever runs this can bound it.
//!
//! ```text
//! exit_probe survive <deck>            open, import, every slide part, unedited save
//! exit_probe edit <deck> <out>         survive, then change one run's text and save to <out>
//! exit_probe budget <stage> <deck> [<out>]
//!     open      open, import, index, read the first slide: what opening a deck costs
//!     open-all  open, import, read every slide-like part
//!     save      open, import, unedited save, which must return the input bytes
//!     edit      open, import, index, read one middle slide, change a run, save to <out>
//! ```
//!
//! Prints one JSON line. A refusal is an answer, not a failure: the caller judges it
//! against docs/29. **A panic is deliberately not caught:** it ends the process with a
//! non-zero status, which the caller counts as the crash it is, and a hang is the caller's
//! to time out. The panic hook prints only a source location, so that document text in a
//! message cannot reach a log. Budget timings cover the operation, not process start.

use std::fmt::Write as _;
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

use casual_slide_export::presentation::save_deck;
use casual_slide_export::slide::save_slide_parts;
use casual_slide_import::presentation::import_deck;
use casual_slide_import::properties::{Findings, ImportLimits};
use casual_slide_import::slide::{LazySlides, SlideLimits, import_slide_parts, index_deck};
use casual_slide_model::shape::ShapeNode;
use casual_slide_model::slide::SlidePart;
use casual_slide_model::text::{TextItem, TextRun};
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage};

const EDITED: &str = "Edited by the exit run";

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

/// A refusal: where, and the stable code. Never document text.
struct Refused {
    stage: &'static str,
    code: String,
}

fn refused(stage: &'static str, code: &str) -> Refused {
    Refused {
        stage,
        code: code.to_owned(),
    }
}

fn refusal_line(r: &Refused) -> String {
    format!(
        "{{\"outcome\": \"refused\", \"stage\": {}, \"code\": {}}}",
        json_str(r.stage),
        json_str(&r.code)
    )
}

/// The first run with text under a shape, depth first.
fn first_run(node: &mut ShapeNode) -> Option<&mut TextRun> {
    if let Some(body) = node.text_body.as_mut() {
        let run = body
            .paragraphs
            .iter_mut()
            .flat_map(|p| p.items.iter_mut())
            .find_map(|item| match item {
                TextItem::Run(run) if run.text.is_some() => Some(run),
                _ => None,
            });
        if run.is_some() {
            return run;
        }
    }
    node.children.iter_mut().find_map(first_run)
}

fn edit_part(part: &mut SlidePart) -> bool {
    let Some(tree) = part.shape_tree_mut() else {
        return false;
    };
    match first_run(tree) {
        Some(run) => {
            run.text = Some(EDITED.to_owned());
            true
        }
        None => false,
    }
}

fn open(source: Vec<u8>) -> Result<PresentationPackage, Refused> {
    PresentationPackage::open(source, OoxmlLimits::default()).map_err(|e| refused("open", e.code()))
}

fn survive(source: Vec<u8>) -> Result<String, Refused> {
    let limits = ImportLimits::default();
    let mut pkg = open(source.clone())?;
    let imported = import_deck(&mut pkg, &limits).map_err(|e| refused("import", e.code()))?;
    let parts = import_slide_parts(
        &mut pkg,
        &imported.deck.presentation,
        &SlideLimits::default(),
        &limits,
    )
    .map_err(|e| refused("slides", e.code()))?;
    let slides = parts.parts.structure.slides.len();
    let mut fresh = open(source.clone())?;
    let saved = save_deck(&mut fresh, &imported.deck, &limits)
        .map_err(|e| refused("unedited_save", e.code()))?;
    Ok(format!(
        "{{\"outcome\": \"open\", \"slides\": {slides}, \"unedited_same\": {}}}",
        saved.bytes == source
    ))
}

fn edit(source: Vec<u8>, out: &Path) -> Result<String, Refused> {
    let limits = ImportLimits::default();
    let slide_limits = SlideLimits::default();
    let mut pkg = open(source.clone())?;
    let imported = import_deck(&mut pkg, &limits).map_err(|e| refused("import", e.code()))?;
    let mut parts = import_slide_parts(
        &mut pkg,
        &imported.deck.presentation,
        &slide_limits,
        &limits,
    )
    .map_err(|e| refused("slides", e.code()))?
    .parts;
    let ordinals = parts.structure.slides.clone();
    let Some(ordinal) = ordinals
        .into_iter()
        .find(|&o| parts.get_mut(o).is_some_and(edit_part))
    else {
        return Ok("{\"outcome\": \"no-text\"}".to_owned());
    };
    let Some(part) = parts.get(ordinal) else {
        return Err(refused("edit", "no-part"));
    };
    let mut fresh = open(source)?;
    let saved = save_slide_parts(&mut fresh, &parts.structure, &[part], &slide_limits, &limits)
        .map_err(|e| refused("edit_save", e.code()))?;
    let mut again = open(saved.bytes.clone()).map_err(|r| refused("reopen", &r.code))?;
    import_deck(&mut again, &limits).map_err(|e| refused("reopen_import", e.code()))?;
    std::fs::write(out, &saved.bytes).map_err(|e| refused("write", &e.kind().to_string()))?;
    Ok(format!(
        "{{\"outcome\": \"edited\", \"changed_parts\": {}}}",
        saved.changed_parts.len()
    ))
}

fn budget(stage: &str, source: Vec<u8>, out: Option<&Path>) -> Result<String, Refused> {
    let limits = ImportLimits::default();
    let slide_limits = SlideLimits::default();
    let start = Instant::now();
    let original = source.clone();
    let mut pkg = open(source)?;
    let imported = import_deck(&mut pkg, &limits).map_err(|e| refused("import", e.code()))?;
    let presentation = &imported.deck.presentation;
    let mut detail = String::new();
    match stage {
        "open" | "edit" => {
            let mut findings = Findings::new(limits.max_findings);
            let structure = index_deck(&pkg, presentation, &slide_limits, &mut findings)
                .map_err(|e| refused("index", e.code()))?;
            let count = structure.slides.len();
            let mut lazy = LazySlides::new(structure, slide_limits, limits);
            let index = if stage == "open" { 0 } else { count / 2 };
            let loaded = lazy
                .slide(&mut pkg, index)
                .map_err(|e| refused("slide", e.code()))?;
            let _ = write!(detail, "\"slides\": {count}, \"loaded\": 1");
            if stage == "edit" {
                let mut part = loaded.value.clone();
                if !edit_part(&mut part) {
                    return Err(refused("edit", "no-text"));
                }
                let saved = save_slide_parts(
                    &mut pkg,
                    &lazy.structure,
                    &[&part],
                    &slide_limits,
                    &limits,
                )
                .map_err(|e| refused("edit_save", e.code()))?;
                let _ = write!(detail, ", \"changed_parts\": {}", saved.changed_parts.len());
                if let Some(out) = out {
                    std::fs::write(out, &saved.bytes)
                        .map_err(|e| refused("write", &e.kind().to_string()))?;
                }
            }
        }
        "open-all" => {
            let parts = import_slide_parts(&mut pkg, presentation, &slide_limits, &limits)
                .map_err(|e| refused("slides", e.code()))?;
            let _ = write!(
                detail,
                "\"slides\": {}, \"loaded\": {}",
                parts.parts.structure.slides.len(),
                parts.parts.parts.len()
            );
        }
        "save" => {
            let saved = save_deck(&mut pkg, &imported.deck, &limits)
                .map_err(|e| refused("unedited_save", e.code()))?;
            let _ = write!(detail, "\"unedited_same\": {}", saved.bytes == original);
        }
        _ => return Err(refused("usage", "unknown stage")),
    }
    Ok(format!(
        "{{\"outcome\": \"ok\", \"stage\": {}, \"ms\": {}, {detail}}}",
        json_str(stage),
        start.elapsed().as_millis()
    ))
}

fn main() -> ExitCode {
    std::panic::set_hook(Box::new(|info| {
        let at = info
            .location()
            .map_or_else(|| "unknown".to_owned(), |l| format!("{}:{}", l.file(), l.line()));
        eprintln!("panic at {at}");
    }));
    let args: Vec<String> = std::env::args().skip(1).collect();
    let words: Vec<&str> = args.iter().map(String::as_str).collect();
    let read = |path: &str| std::fs::read(path).map_err(|e| e.kind().to_string());
    let (deck, result) = match words.as_slice() {
        ["survive", deck] => (*deck, read(deck).map(survive)),
        ["edit", deck, out] => (*deck, read(deck).map(|s| edit(s, Path::new(out)))),
        ["budget", stage, deck] => (*deck, read(deck).map(|s| budget(stage, s, None))),
        ["budget", stage, deck, out] => {
            (*deck, read(deck).map(|s| budget(stage, s, Some(Path::new(out)))))
        }
        _ => {
            eprintln!("usage: exit_probe survive|edit|budget ...");
            return ExitCode::from(2);
        }
    };
    let name = Path::new(deck)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let line = match result {
        Ok(Ok(line)) => line,
        Ok(Err(refusal)) => refusal_line(&refusal),
        Err(kind) => {
            eprintln!("cannot read the deck: {kind}");
            return ExitCode::from(2);
        }
    };
    let body = line.strip_prefix('{').unwrap_or(&line);
    println!("{{\"deck\": {}, {body}", json_str(&name));
    ExitCode::SUCCESS
}
