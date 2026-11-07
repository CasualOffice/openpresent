//! Evidence for Phase 1's closing list (`P1X-01`, `tools/phase1-evidence.py`): the 500-slide
//! budget, the whole hostile corpus under a time bound, and every corpus deck written out
//! edited, for the validators and LibreOffice to judge.
//!
//! Each test prints `PHASE1 <item> key=value ...` lines, which the evidence tool reads, so run
//! them with `--nocapture`. Times are wall clock on whatever machine ran them, and so are
//! upper bounds when it was loaded. Nothing here says anything about PowerPoint (`PV-01`).

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use casual_slide_export::presentation::save_deck;
use casual_slide_export::slide::save_slide_parts;
use casual_slide_export::writer::{Op, SavePlan, SlideRef};
use casual_slide_import::presentation::import_deck;
use casual_slide_import::properties::ImportLimits;
use casual_slide_import::slide::{ImportedSlides, SlideLimits, import_slide_parts};
use casual_slide_model::presentation::Deck;
use casual_slide_model::properties::CoreProperties;
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage};

/// The 500-slide budget, for an unoptimized build, which is what `cargo test` is. Set from what
/// that build measured, with margin, so that crossing one means something got slower and not
/// that a machine was busy: on 2026-11-08, on a 10-core laptop at load 6 to 9, open and read
/// every slide took 0.5 to 0.6 s, an unedited save under 1 ms, and a slide edit 1.7 to 2.1 s.
/// The edit is the larger, since a first save reads every part (`prepare_for_edit`).
const BUDGET_OPEN: Duration = Duration::from_secs(10);
const BUDGET_UNEDITED_SAVE: Duration = Duration::from_millis(100);
const BUDGET_EDIT: Duration = Duration::from_secs(10);

/// One hostile deck past this, through open, read and edit, is a hang in all but name.
const HOSTILE_BOUND: Duration = Duration::from_secs(60);

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn title_edit(deck: &Deck) -> Deck {
    let mut after = deck.clone();
    after
        .properties
        .core
        .get_or_insert_with(CoreProperties::default)
        .title = Some("Edited for Phase 1's exit".to_owned());
    after
}

fn read_all(pkg: &mut PresentationPackage) -> Result<(Deck, ImportedSlides), String> {
    let limits = ImportLimits::default();
    let deck = import_deck(pkg, &limits)
        .map_err(|e| format!("import_deck {}", e.code()))?
        .deck;
    let slides = import_slide_parts(pkg, &deck.presentation, &SlideLimits::default(), &limits)
        .map_err(|e| format!("import_slide_parts {}", e.code()))?;
    Ok((deck, slides))
}

/// Rename a named shape on one slide, which patches that slide's XML: the last one the writer
/// will patch, since a shape inside alternate content is refused by design (`CS-SLE-0002`).
/// `None` when the slide has no shape to rename.
fn rename_a_shape(
    pkg: &mut PresentationPackage,
    slides: &ImportedSlides,
    index: usize,
) -> Option<Result<Vec<u8>, String>> {
    let ordinal = slides.parts.structure.slide(index)?;
    let named: Vec<_> = slides
        .parts
        .get(ordinal)?
        .shape_tree()?
        .descendants()
        .filter(|n| {
            n.non_visual
                .as_ref()
                .is_some_and(|visual| visual.name.is_some())
        })
        .map(|n| n.id)
        .collect();
    let mut last = None;
    for id in named.into_iter().rev() {
        let mut wanted = slides.parts.clone();
        let part = wanted.get_mut(ordinal)?;
        part.node_mut(id)?.non_visual.as_mut()?.name = Some("Edited for Phase 1's exit".to_owned());
        let part = wanted.get(ordinal)?;
        match save_slide_parts(
            pkg,
            &slides.parts.structure,
            &[part],
            &SlideLimits::default(),
            &ImportLimits::default(),
        ) {
            Ok(outcome) => return Some(Ok(outcome.bytes)),
            Err(e) if e.code() == "CS-SLE-0002" => last = Some(e.code()),
            Err(e) => return Some(Err(format!("rename {}", e.code()))),
        }
    }
    last.map(|code| Err(format!("rename {code}")))
}

/// The three kinds of edit a user makes: a property, a slide's XML, and the slide list. Each is
/// a reopen of what the one before wrote, so each sees a file and not its own memory. A step
/// the deck gives no place for is left out and named.
fn edit_every_way(source: &[u8]) -> Result<Edited, String> {
    let mut left_out = Vec::new();
    let limits = OoxmlLimits::default();
    let import_limits = ImportLimits::default();

    let mut pkg = PresentationPackage::open(source.to_vec(), limits)
        .map_err(|e| format!("open {}", e.code()))?;
    let (deck, _) = read_all(&mut pkg)?;
    let bytes = save_deck(&mut pkg, &title_edit(&deck), &import_limits)
        .map_err(|e| format!("title {}", e.code()))?
        .bytes;

    let mut pkg =
        PresentationPackage::open(bytes, limits).map_err(|e| format!("reopen {}", e.code()))?;
    let (deck, slides) = read_all(&mut pkg)?;
    let bytes = match rename_a_shape(&mut pkg, &slides, 0) {
        Some(Ok(renamed)) => renamed,
        Some(Err(why)) if why.ends_with("CS-SLE-0002") => {
            left_out.push("rename");
            pkg.save_unedited()
        }
        Some(Err(why)) => return Err(why),
        None => {
            left_out.push("rename");
            pkg.save_unedited()
        }
    };

    let mut pkg =
        PresentationPackage::open(bytes, limits).map_err(|e| format!("reopen {}", e.code()))?;
    let count = deck.presentation.slides.map_or(0, |s| s.len());
    let mut plan = SavePlan::new().acknowledge_signature().op(Op::AddSlide {
        position: None,
        layout: None,
    });
    if count > 0 {
        plan = plan.op(Op::DuplicateSlide {
            slide: SlideRef::Index(0),
        });
    }
    let mut clones = BTreeMap::new();
    let bytes = match plan.save(&mut pkg) {
        Ok(outcome) => {
            clones = outcome.clones;
            outcome.bytes
        }
        // A construct with no safe form yet is a stated refusal (`writer_corpus.rs`).
        Err(err) if err.code() == "CS-SAV-0009" => {
            left_out.push("slide-list");
            pkg.save_unedited()
        }
        Err(err) => return Err(format!("slide list {}", err.code())),
    };
    PresentationPackage::open_strict(bytes.clone(), limits)
        .map_err(|e| format!("the output does not open strictly: {}", e.code()))?;
    Ok(Edited {
        bytes,
        left_out,
        clones,
    })
}

/// A deck edited every way, with what was left out, and each part the slide-list edit cloned,
/// by its source, so a validator's finding on a clone can be traced to the input's.
struct Edited {
    bytes: Vec<u8>,
    left_out: Vec<&'static str>,
    clones: BTreeMap<String, String>,
}

#[test]
fn a_500_slide_deck_opens_saves_and_edits_within_its_budget() {
    let source = std::fs::read(fixtures().join("generated/five-hundred-slides.pptx")).unwrap();

    let started = Instant::now();
    let mut pkg = PresentationPackage::open(source.clone(), OoxmlLimits::default()).unwrap();
    let (_, slides) = read_all(&mut pkg).unwrap();
    let open = started.elapsed();
    assert_eq!(slides.parts.structure.slides.len(), 500);

    let started = Instant::now();
    let saved = pkg.save_unedited();
    let unedited = started.elapsed();
    assert_eq!(saved, source, "an unedited save gives the input back");

    let started = Instant::now();
    let edited = rename_a_shape(&mut pkg, &slides, 250)
        .expect("slide 251 has a named shape")
        .unwrap();
    let edit = started.elapsed();
    assert_ne!(edited, source);
    PresentationPackage::open_strict(edited, OoxmlLimits::default()).unwrap();

    println!(
        "PHASE1 budget open_ms={} unedited_save_ms={} edit_ms={} budget_open_ms={} budget_unedited_save_ms={} budget_edit_ms={} optimized={}",
        open.as_millis(),
        unedited.as_millis(),
        edit.as_millis(),
        BUDGET_OPEN.as_millis(),
        BUDGET_UNEDITED_SAVE.as_millis(),
        BUDGET_EDIT.as_millis(),
        !cfg!(debug_assertions),
    );
    assert!(open <= BUDGET_OPEN, "open took {open:?}");
    assert!(unedited <= BUDGET_UNEDITED_SAVE, "save took {unedited:?}");
    assert!(edit <= BUDGET_EDIT, "edit took {edit:?}");
}

/// What one hostile deck did. Only a panic or passing the bound breaks the rule; a typed refusal
/// is how a deck is allowed to say no, and is counted by its code.
enum Hostile {
    Edited,
    Refused(String),
    Panicked,
}

/// Open, read every slide, and edit once: what a user does first. Reopening and editing again
/// is the corpus run's, and would only read a bounded-but-large part again.
fn open_read_and_edit(source: &[u8]) -> Result<(), String> {
    let limits = OoxmlLimits::default();
    let mut pkg = PresentationPackage::open(source.to_vec(), limits)
        .map_err(|e| format!("open {}", e.code()))?;
    let (deck, _) = read_all(&mut pkg)?;
    let bytes = save_deck(&mut pkg, &title_edit(&deck), &ImportLimits::default())
        .map_err(|e| format!("title {}", e.code()))?
        .bytes;
    PresentationPackage::open_strict(bytes, limits)
        .map_err(|e| format!("the output does not open strictly: {}", e.code()))?;
    Ok(())
}

fn hostile_one(bytes: Vec<u8>) -> Hostile {
    let run = std::panic::catch_unwind(move || open_read_and_edit(&bytes));
    match run {
        Ok(Ok(_)) => Hostile::Edited,
        Ok(Err(why)) => Hostile::Refused(why),
        Err(_) => Hostile::Panicked,
    }
}

/// Every file in `fixtures/hostile`, read from the folder and not from the manifest, so a
/// fixture the manifest forgot is still run. On a thread of its own, so a hang is seen as one.
/// A stack overflow takes the whole test binary with it, which the evidence tool sees.
#[test]
fn every_hostile_fixture_opens_and_edits_within_its_bound_or_is_refused() {
    let mut files: Vec<PathBuf> = std::fs::read_dir(fixtures().join("hostile"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    files.sort();
    assert!(files.len() > 100, "{}", files.len());
    let mut edited = 0usize;
    let mut refusals = BTreeMap::<String, Vec<String>>::new();
    let mut broken = Vec::new();
    let mut slowest = Duration::ZERO;
    for path in &files {
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let bytes = std::fs::read(path).unwrap();
        let (tell, wait) = mpsc::channel();
        let started = Instant::now();
        std::thread::spawn(move || {
            let _ = tell.send(hostile_one(bytes));
        });
        match wait.recv_timeout(HOSTILE_BOUND) {
            Ok(Hostile::Edited) => edited += 1,
            Ok(Hostile::Refused(why)) => refusals.entry(why).or_default().push(name),
            Ok(Hostile::Panicked) => broken.push(format!("{name}: panic")),
            Err(_) => broken.push(format!("{name}: over {HOSTILE_BOUND:?}")),
        }
        slowest = slowest.max(started.elapsed());
    }
    let refused: usize = refusals.values().map(Vec::len).sum();
    println!(
        "PHASE1 hostile total={} edited={edited} refused={refused} broken={} slowest_ms={} bound_ms={}",
        files.len(),
        broken.len(),
        slowest.as_millis(),
        HOSTILE_BOUND.as_millis(),
    );
    for (why, names) in &refusals {
        println!(
            "PHASE1 hostile-refusal {why}: {} [{}]",
            names.len(),
            names.join(" ")
        );
    }
    assert!(broken.is_empty(), "{}", broken.join("\n"));
}

/// Every deck in `fixtures/real` and `fixtures/generated`, edited each way, written to
/// `PHASE1_EXPORT_OUT` under its own name. The validators and LibreOffice run from the tool.
#[test]
fn every_corpus_deck_writes_out_edited() {
    let out = std::env::var_os("PHASE1_EXPORT_OUT").map_or_else(
        || PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("phase1-export"),
        PathBuf::from,
    );
    std::fs::create_dir_all(&out).unwrap();
    let mut total = 0usize;
    let mut left_out = Vec::new();
    let mut failures = Vec::new();
    for folder in ["real", "generated"] {
        let mut decks: Vec<PathBuf> = std::fs::read_dir(fixtures().join(folder))
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| {
                p.extension().is_some_and(|e| {
                    ["pptx", "pptm", "ppsx", "ppsm", "potx", "potm"]
                        .iter()
                        .any(|s| e == *s)
                })
            })
            .collect();
        decks.sort();
        for path in decks {
            total += 1;
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            match edit_every_way(&std::fs::read(&path).unwrap()) {
                Ok(edited) => {
                    std::fs::write(out.join(&name), edited.bytes).unwrap();
                    // As `writer_corpus.rs` writes them, which `tools/writer-reopen.py` reads.
                    let mut clones = String::new();
                    for (clone, source) in &edited.clones {
                        writeln!(clones, "{clone}\t{source}").unwrap();
                    }
                    std::fs::write(out.join(format!("{name}.clones")), clones).unwrap();
                    if !edited.left_out.is_empty() {
                        left_out.push(format!("{name}({})", edited.left_out.join("+")));
                    }
                }
                Err(why) => failures.push(format!("{name}: {why}")),
            }
        }
    }
    println!(
        "PHASE1 export-write total={total} failed={} left_out={}",
        failures.len(),
        left_out.len()
    );
    println!("PHASE1 export-left-out {}", left_out.join(" "));
    for failure in &failures {
        println!("PHASE1 export-failure {failure}");
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Found by the hostile run: a `tableStyles` part over its bound stops every slide being read
/// (`CS-TBL-0005`), where docs/29 asks that the styles be set aside and the deck open. Now the
/// list is left out of the model, kept in the package, and reported as unreadable.
#[test]
fn a_table_style_bound_leaves_the_slides_readable() {
    let bytes = std::fs::read(fixtures().join("hostile/table-limit-styles.bin")).unwrap();
    let mut pkg = PresentationPackage::open(bytes.clone(), OoxmlLimits::default()).unwrap();
    let (_, slides) = read_all(&mut pkg).unwrap();
    assert!(slides.table_styles.is_none());
    assert!(
        slides
            .findings
            .iter()
            .any(|f| f.kind.code() == "CS-IMF-0012" && f.part.contains("tableStyles")),
        "the list set aside is not reported"
    );
    edit_every_way(&bytes).unwrap();
}

/// Found by the full damage corpus: 32 bytes flipped in slideLayout1's local header. The package
/// opens clean from its directory, and the layout is found damaged only when read. That read
/// repairs the package, which drops the layout and supplies one; the slide import read on from
/// its index of the package before the repair, and failed `CS-OPC-0001` on the dropped part.
#[test]
fn a_layout_damaged_in_its_local_header_still_lets_the_slides_read() {
    let mut bytes = std::fs::read(fixtures().join("real/z17310490-vierfelder-de.pptx")).unwrap();
    for b in bytes.iter_mut().skip(9948).take(32) {
        *b ^= 0xA5;
    }
    let mut pkg = PresentationPackage::open(bytes.clone(), OoxmlLimits::default()).unwrap();
    let (_, slides) = read_all(&mut pkg).unwrap();
    let structure = &slides.parts.structure;
    assert!(!structure.slides.is_empty() && !structure.layouts.is_empty());
    for ordinal in &structure.slides {
        let slide = slides.parts.get(*ordinal).unwrap();
        assert!(slide.shape_tree().is_some(), "slide {ordinal} is not there");
    }
    edit_every_way(&bytes).unwrap();
}

/// Found by the first export run over the hostile files: adding and duplicating a slide in a
/// deck whose sections are hostile is refused as a broken invariant (`CS-SAV-0012`), where the
/// edit should save. Two causes: a new slide after one in no section went to the front of the
/// first section, out of step with the slide list; and sections sharing an id were all
/// written with one row, so the slide was in many.
#[test]
fn a_slide_list_edit_saves_on_a_deck_with_hostile_sections() {
    for name in [
        "section-bad-values.bin",
        "section-limit-sections.bin",
        "section-limit-slides.bin",
    ] {
        let bytes = std::fs::read(fixtures().join("hostile").join(name)).unwrap();
        if let Err(why) = edit_every_way(&bytes) {
            panic!("{name}: {why}");
        }
    }
}
