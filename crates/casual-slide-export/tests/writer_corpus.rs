//! `T1N-04`: add, duplicate and delete at file level on every deck in `fixtures/generated`,
//! `fixtures/producers` and `fixtures/real`. Each result reopens through the engine, with
//! every other part byte-identical, and no part the input had as valid fails F4.
//! `fixtures/hostile` is never in here.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

use casual_slide_export::writer::{Op, SavePlan, SlideRef};
use casual_slide_import::presentation::import_deck;
use casual_slide_import::properties::ImportLimits;
use casual_slide_import::slide::{SlideLimits, import_slide_parts};
use casual_slide_ooxml::{FindingKind, OoxmlLimits, PresentationPackage};

const DIRS: &[&str] = &["generated", "producers", "real"];
/// Slides duplicated at once, in the variant that copies many.
const MANY: usize = 16;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn decks() -> Vec<(String, PathBuf)> {
    let mut out = Vec::new();
    for dir in DIRS {
        for entry in std::fs::read_dir(root().join("fixtures").join(dir)).unwrap() {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            if [".pptx", ".ppsx", ".potx", ".pptm", ".potm", ".ppsm"]
                .iter()
                .any(|s| name.ends_with(s))
            {
                out.push((format!("{dir}/{name}"), path));
            }
        }
    }
    out.sort();
    out
}

fn open(bytes: &[u8]) -> PresentationPackage {
    PresentationPackage::open(bytes.to_vec(), OoxmlLimits::default()).unwrap()
}

fn slide_count(bytes: &[u8]) -> usize {
    let mut pkg = open(bytes);
    import_deck(&mut pkg, &ImportLimits::default())
        .unwrap()
        .deck
        .presentation
        .slides
        .map_or(0, |s| s.len())
}

fn parts(bytes: &[u8]) -> BTreeMap<String, Vec<u8>> {
    let mut pkg = open(bytes);
    pkg.part_names()
        .into_iter()
        .filter(|n| !n.ends_with('/'))
        .map(|n| {
            let b = pkg.read_part(&n).unwrap();
            (n, b)
        })
        .collect()
}

/// Findings that mean a package is structurally worse, which a save must not add.
fn structural(bytes: &[u8]) -> BTreeSet<(String, String)> {
    open(bytes)
        .findings()
        .iter()
        .filter(|f| {
            matches!(
                f.kind,
                FindingKind::DanglingTarget
                    | FindingKind::OrphanPart
                    | FindingKind::UndeclaredContentType
            )
        })
        .map(|f| (f.kind.code().to_owned(), f.part.clone()))
        .collect()
}

fn plans(slides: usize) -> Vec<(&'static str, usize, Vec<Op>)> {
    let add = Op::AddSlide {
        position: None,
        layout: None,
    };
    let add_first = Op::AddSlide {
        position: Some(0),
        layout: None,
    };
    let mut out = vec![("add", slides + 2, vec![add.clone(), add_first])];
    if slides > 0 {
        out.push((
            "duplicate",
            slides + 1,
            vec![Op::DuplicateSlide {
                slide: SlideRef::Index(0),
            }],
        ));
        out.push((
            "delete",
            slides - 1,
            vec![Op::DeleteSlide {
                slide: SlideRef::Index(slides - 1),
            }],
        ));
        // Every slide copied, or the first sixteen, then the first slide deleted and a new
        // one added: what a real session's plan looks like at its worst.
        let copied = slides.min(MANY);
        let mut many: Vec<Op> = (0..copied)
            .rev()
            .map(|i| Op::DuplicateSlide {
                slide: SlideRef::Index(i),
            })
            .collect();
        many.push(Op::DeleteSlide {
            slide: SlideRef::Index(0),
        });
        many.push(add);
        out.push(("many", slides + copied, many));
    }
    out
}

fn out_dir() -> PathBuf {
    std::env::var_os("WRITER_CORPUS_OUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_TARGET_TMPDIR")).join("writer-corpus"))
}

#[test]
fn add_duplicate_and_delete_reopen_on_every_deck() {
    let out = out_dir();
    let _ = std::fs::remove_dir_all(&out);
    let mut failures = Vec::new();
    let mut summary = String::new();
    let mut refused = String::new();
    let mut count = 0usize;
    for (name, path) in decks() {
        let source = std::fs::read(&path).unwrap();
        let slides = slide_count(&source);
        let input_findings = structural(&source);
        let old = parts(&source);
        for (variant, want, ops) in plans(slides) {
            let mut pkg = open(&source);
            let mut plan = SavePlan::new();
            for op in &ops {
                plan = plan.op(op.clone());
            }
            let outcome = match plan.save(&mut pkg) {
                Ok(outcome) => outcome,
                Err(err) => {
                    // A construct with no safe form yet is a refusal, stated and counted.
                    if err.code() == "CS-SAV-0009" {
                        writeln!(refused, "{name}\t{variant}\t{err}").unwrap();
                        continue;
                    }
                    failures.push(format!("{name} {variant}: {err}"));
                    continue;
                }
            };
            count += 1;
            let new = parts(&outcome.bytes);
            for (part, bytes) in &old {
                if !outcome.changed_parts.contains(part) && new.get(part) != Some(bytes) {
                    failures.push(format!("{name} {variant}: {part} changed"));
                }
            }
            let got = slide_count(&outcome.bytes);
            if got != want {
                failures.push(format!("{name} {variant}: {got} slides, not {want}"));
            }
            let mut reopened = open(&outcome.bytes);
            let deck = import_deck(&mut reopened, &ImportLimits::default()).unwrap();
            if let Err(err) = import_slide_parts(
                &mut reopened,
                &deck.deck.presentation,
                &SlideLimits::default(),
                &ImportLimits::default(),
            ) {
                failures.push(format!("{name} {variant}: slides do not import: {err}"));
            }
            for finding in structural(&outcome.bytes).difference(&input_findings) {
                let origin = outcome
                    .clones
                    .get(&finding.1)
                    .cloned()
                    .unwrap_or_else(|| finding.1.clone());
                if !input_findings.contains(&(finding.0.clone(), origin)) {
                    failures.push(format!("{name} {variant}: new {finding:?}"));
                }
            }
            for entry in &outcome.report {
                writeln!(
                    summary,
                    "{name}\t{variant}\t{}\t{}\t{}\t{}",
                    entry.code, entry.rule, entry.part, entry.detail
                )
                .unwrap();
            }
            // Beside the deck, so the validators can judge a clone by its source.
            let dir = out.join(variant);
            std::fs::create_dir_all(&dir).unwrap();
            let file = name.rsplit('/').next().unwrap();
            std::fs::write(dir.join(file), &outcome.bytes).unwrap();
            let clones: String = outcome
                .clones
                .iter()
                .map(|(clone, source)| format!("{clone}\t{source}\n"))
                .collect();
            std::fs::write(dir.join(format!("{file}.clones")), clones).unwrap();
        }
    }
    std::fs::create_dir_all(&out).unwrap();
    std::fs::write(out.join("report.tsv"), summary).unwrap();
    std::fs::write(out.join("refused.tsv"), refused).unwrap();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    assert!(count > 100, "only {count} results");

    // F4, each result against its input, so what a deck arrived with is not ours.
    let have = |tool: &str, arg: &str| Command::new(tool).arg(arg).output().is_ok();
    if !have("python3", "--version") || !have("xmllint", "--version") {
        assert!(
            std::env::var_os("F4_REQUIRED").is_none(),
            "F4_REQUIRED is set, and python3 or xmllint is missing"
        );
        eprintln!("xmllint or python3 not installed: F4 skipped");
        return;
    }
    let result = Command::new("python3")
        .arg("-I")
        .arg(root().join("tools/writer-reopen.py"))
        .arg("f4")
        .arg(&out)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}
