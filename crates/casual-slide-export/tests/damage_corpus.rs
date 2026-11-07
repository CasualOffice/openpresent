//! The damage corpus (`REC-05`, `docs/29-RECOVERY.md`). Real decks from `fixtures/real`, each
//! damaged deterministically in every way docs/29 names, at test time, so the repository
//! stays small. Every damaged deck must:
//!
//! - open, or be refused only by docs/29's rules;
//! - never panic, and never hang;
//! - give the input back byte for byte from an unedited save;
//! - read through import, with slides that could not be read marked and in their places;
//! - after an edit, write a file the writer's preflight passes, that opens under `strict`,
//!   and that `tools/f4/validate.py` passes.
//!
//! `DAMAGE_FULL=1` runs every deck at every offset, and is what the numbers in docs/29 come
//! from; the default samples the smaller ones, so `cargo test` stays usable. Counts are
//! printed: run with `--nocapture` to read them. A deck is seeded by name, so a failure
//! found in a run is found again.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use casual_slide_export::presentation::save_deck;
use casual_slide_import::presentation::import_deck;
use casual_slide_import::properties::ImportLimits;
use casual_slide_import::slide::{SlideLimits, import_slide_parts};
use casual_slide_model::properties::CoreProperties;
use casual_slide_ooxml::{FindingKind, OoxmlLimits, PresentationPackage};
use casual_slide_package::{Package, PackageLimits, PartEdits};

/// A deck over this much time on one damage would be a hang in all but name. Generous,
/// since this runs unoptimized alongside other tests.
const MAX_ONE_DECK: Duration = Duration::from_secs(60);

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// As many threads as the host allows. `RUST_TEST_THREADS` is how a loaded machine says so.
fn workers() -> usize {
    std::env::var("RUST_TEST_THREADS")
        .ok()
        .and_then(|n| n.parse().ok())
        .unwrap_or_else(|| std::thread::available_parallelism().map_or(4, |n| n.get().min(8)))
}

/// A generator of its own, so the corpus is one set of bytes everywhere: SplitMix64.
struct Rng(u64);

impl Rng {
    fn for_deck(name: &str, kind: &str) -> Self {
        // FNV-1a, for a seed that does not move with a hasher's defaults.
        let mut seed = 0xcbf2_9ce4_8422_2325u64;
        for b in name.bytes().chain(kind.bytes()) {
            seed ^= u64::from(b);
            seed = seed.wrapping_mul(0x0100_0000_01b3);
        }
        Self(seed)
    }

    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            return 0;
        }
        (self.next() % n as u64) as usize
    }
}

/// One damaged deck, and the kind of damage, which is what the counts are by.
struct Damaged {
    kind: &'static str,
    label: String,
    bytes: Vec<u8>,
}

fn package(bytes: &[u8]) -> Package {
    Package::open(bytes.to_vec(), PackageLimits::default()).unwrap()
}

/// The deck, rewritten with some parts replaced or gone. The ZIP stays sound.
fn with_edits(deck: &[u8], edit: impl FnOnce(&mut PartEdits)) -> Vec<u8> {
    let mut edits = PartEdits::default();
    edit(&mut edits);
    package(deck).rewrite(&edits).unwrap()
}

fn find_last(bytes: &[u8], signature: &[u8; 4]) -> Option<usize> {
    bytes.windows(4).rposition(|w| w == signature)
}

fn first_central_record(bytes: &[u8]) -> Option<usize> {
    bytes.windows(4).position(|w| w == [0x50, 0x4b, 0x01, 0x02])
}

fn container_damage(name: &str, deck: &[u8], scale: usize, out: &mut Vec<Damaged>) {
    let mut rng = Rng::for_deck(name, "container");
    let len = deck.len();
    let mut push = |kind: &'static str, label: String, bytes: Vec<u8>| {
        out.push(Damaged { kind, label, bytes });
    };
    // Truncations: at fixed fractions, right at the ends, and at seeded places.
    for fraction in [1, 2, 5, 10, 25, 50, 75, 90, 95, 99] {
        let at = len * fraction / 100;
        push("truncated", format!("{fraction}%"), deck[..at].to_vec());
    }
    for back in [1, 4, 22, 23, 100] {
        push("truncated", format!("-{back}"), deck[..len - back].to_vec());
    }
    for _ in 0..scale * 2 {
        let at = rng.below(len);
        push("truncated", format!("@{at}"), deck[..at].to_vec());
    }
    // Flipped bytes: one, and a burst, anywhere in the file.
    for _ in 0..scale * 6 {
        let at = rng.below(len);
        let mut bytes = deck.to_vec();
        bytes[at] ^= 1 << rng.below(8);
        push("flipped-byte", format!("@{at}"), bytes);
    }
    for _ in 0..scale * 2 {
        let at = rng.below(len);
        let mut bytes = deck.to_vec();
        for b in bytes.iter_mut().skip(at).take(32) {
            *b ^= 0xA5;
        }
        push("flipped-burst", format!("@{at}"), bytes);
    }
    // What surrounds the ZIP.
    let mut prepended = b"#!/bin/sh\nexec unzip \"$0\"\n".to_vec();
    prepended.extend_from_slice(deck);
    push("prepended", "script".to_owned(), prepended);
    let mut trailing = deck.to_vec();
    trailing.extend_from_slice(&[0x7F; 4096]);
    push("trailing", "bytes".to_owned(), trailing);
    // The directory: gone, its end record broken, its count wrong, its offsets wrong.
    if let Some(central) = first_central_record(deck) {
        push("no-directory", "cut".to_owned(), deck[..central].to_vec());
    }
    if let Some(end) = find_last(deck, &[0x50, 0x4b, 0x05, 0x06]) {
        let mut bytes = deck.to_vec();
        bytes[end] = 0;
        push("directory-broken", "end-signature".to_owned(), bytes);
        let mut bytes = deck.to_vec();
        bytes[end + 10] ^= 0x7F;
        push("directory-broken", "count".to_owned(), bytes);
        let mut bytes = deck.to_vec();
        bytes[end + 16] ^= 0x7F;
        push("directory-broken", "offset".to_owned(), bytes);
        let mut bytes = deck.to_vec();
        bytes[end + 12] ^= 0x7F;
        push("directory-broken", "size".to_owned(), bytes);
        let mut bytes = deck.to_vec();
        bytes[end + 20] = 0xFF;
        push("directory-broken", "comment".to_owned(), bytes);
    }
    // A local header wiped: the directory says one thing, the data another.
    if let Some(second) = deck
        .windows(4)
        .enumerate()
        .skip(4)
        .find(|(_, w)| *w == [0x50, 0x4b, 0x03, 0x04])
        .map(|(i, _)| i)
    {
        let mut bytes = deck.to_vec();
        for b in bytes.iter_mut().skip(second).take(30) {
            *b = 0;
        }
        push("local-header-wiped", "second".to_owned(), bytes);
    }
}

/// The names to do something to, as the deck spells them.
struct Parts {
    names: Vec<String>,
}

impl Parts {
    fn of(deck: &[u8]) -> Self {
        Self {
            names: package(deck).entry_names(),
        }
    }

    fn first(&self, pattern: impl Fn(&str) -> bool) -> Option<&String> {
        self.names.iter().find(|n| pattern(&n.to_ascii_lowercase()))
    }

    /// One of each kind of part, for damage to reach everything a deck has.
    fn representatives(&self) -> Vec<String> {
        let wanted: [&dyn Fn(&str) -> bool; 14] = [
            &|n| n == "[content_types].xml",
            &|n| n == "_rels/.rels",
            &|n| n.ends_with("presentation.xml"),
            &|n| n.ends_with("presentation.xml.rels"),
            &|n| n.contains("slides/slide") && n.ends_with(".xml"),
            &|n| n.contains("slides/_rels/") && n.ends_with(".rels"),
            &|n| n.contains("slidelayouts/slidelayout") && n.ends_with(".xml"),
            &|n| n.contains("slidemasters/slidemaster") && n.ends_with(".xml"),
            &|n| n.contains("slidemasters/_rels/"),
            &|n| n.contains("theme/theme") && n.ends_with(".xml"),
            &|n| n.ends_with("core.xml"),
            &|n| n.ends_with("app.xml"),
            &|n| n.ends_with("presprops.xml"),
            &|n| n.ends_with("viewprops.xml"),
        ];
        let mut out = Vec::new();
        for test in wanted {
            if let Some(name) = self.first(test) {
                out.push(name.clone());
            }
        }
        // And one part of any other kind: a chart or diagram, or notes.
        for extra in [
            &(|n: &str| n.contains("media/")) as &dyn Fn(&str) -> bool,
            &|n| n.contains("notesslides/") && n.ends_with(".xml"),
            &|n| n.contains("charts/") && n.ends_with(".xml"),
            &|n| n.contains("diagrams/") && n.ends_with(".xml"),
            &|n| n.contains("customxml/") && n.ends_with(".xml"),
        ] {
            if let Some(name) = self.first(extra) {
                out.push(name.clone());
            }
        }
        out
    }
}

/// Damage only well-formedness can see: each of these leaves a part that `walk` refuses, so
/// it is what "broken XML" means here. Damage that leaves XML well-formed and merely invalid
/// is another matter: docs/29 says so, and `well_formed_but_invalid` covers it.
fn break_xml(xml: &[u8], at: usize, how: usize) -> Vec<u8> {
    let at = at.min(xml.len());
    let mut bytes = xml.to_vec();
    match how {
        0 => bytes.truncate(at),
        1 => bytes.insert(at, b'<'),
        2 => {
            let tag = bytes
                .iter()
                .skip(at)
                .position(|b| *b == b'>')
                .map_or(at, |i| at + i);
            if tag < bytes.len() {
                bytes.remove(tag);
            }
        }
        3 => bytes
            .splice(at..at, *b"</p:notAnElementHere>")
            .for_each(drop),
        4 => bytes.splice(at..at, *b"<unclosedElement>").for_each(drop),
        5 => bytes.splice(at..at, *b"&undefinedEntity;").for_each(drop),
        6 => bytes.insert(at, 0),
        7 => bytes.insert(at, 0xFF),
        8 => {
            let quote = bytes
                .iter()
                .skip(at)
                .position(|b| *b == b'"')
                .map_or(at, |i| at + i);
            if quote < bytes.len() {
                bytes.remove(quote);
            }
        }
        _ => bytes
            .splice(at..at, *b"<a b=\"1\" b=\"2\" c=3 d/>")
            .for_each(drop),
    }
    bytes
}

const BREAKS: usize = 10;

fn part_damage(name: &str, deck: &[u8], scale: usize, full: bool, out: &mut Vec<Damaged>) {
    let parts = Parts::of(deck);
    let reps = parts.representatives();

    // Removed parts, one at a time.
    for part in &reps {
        let bytes = with_edits(deck, |e| e.remove(part));
        out.push(Damaged {
            kind: "removed-part",
            label: part.clone(),
            bytes,
        });
    }

    // Broken XML at many offsets, in each kind of part.
    let mut rng = Rng::for_deck(name, "xml");
    let mut source = package(deck);
    for part in &reps {
        if !(part.ends_with(".xml") || part.ends_with(".rels")) {
            continue;
        }
        let xml = source.read_part(part).unwrap();
        // A full run does each way of breaking in every part. A sample does one in each,
        // and takes them in turn, so a few parts see them all.
        let ways = if full { 0..BREAKS } else { 0..1 };
        for how in ways {
            let how = if full { how } else { rng.below(BREAKS) };
            for _ in 0..scale {
                let at = rng.below(xml.len());
                let broken = break_xml(&xml, at, how);
                let bytes = with_edits(deck, |e| e.replace(part.clone(), broken));
                out.push(Damaged {
                    kind: "broken-xml",
                    label: format!("{part}#{how}@{at}"),
                    bytes,
                });
            }
        }
    }

    // A part that is not XML at all, and one that is empty.
    for part in reps
        .iter()
        .filter(|p| p.ends_with(".xml") || p.ends_with(".rels"))
    {
        out.push(Damaged {
            kind: "not-xml",
            label: format!("{part}#junk"),
            bytes: with_edits(deck, |e| {
                e.replace(part.clone(), b"this is not XML at all".to_vec());
            }),
        });
        out.push(Damaged {
            kind: "not-xml",
            label: format!("{part}#empty"),
            bytes: with_edits(deck, |e| e.replace(part.clone(), Vec::new())),
        });
    }

    // Hostile additions a repair must not obey.
    let main = parts
        .first(|n| n.ends_with("presentation.xml"))
        .cloned()
        .unwrap_or_default();
    let main_xml = source.read_part(&main).unwrap();
    let start = main_xml
        .windows(2)
        .position(|w| w == b"?>")
        .map_or(0, |i| i + 2);
    let bomb = "<!DOCTYPE p [<!ENTITY a \"aaaaaaaa\"><!ENTITY b \"&a;&a;&a;&a;\">\
                <!ENTITY c SYSTEM \"http://127.0.0.1:9/never\">]>";
    let mut with_doctype = main_xml.clone();
    with_doctype
        .splice(start..start, bomb.bytes())
        .for_each(drop);
    out.push(Damaged {
        kind: "hostile-xml",
        label: format!("{main}#doctype"),
        bytes: with_edits(deck, |e| e.replace(main.clone(), with_doctype)),
    });

    // Wrong encodings.
    let mut declared = main_xml.clone();
    if let Some(at) = declared.windows(5).position(|w| w == b"UTF-8") {
        declared.splice(at..at + 5, *b"UTF16").for_each(drop);
        out.push(Damaged {
            kind: "wrong-encoding",
            label: format!("{main}#declared"),
            bytes: with_edits(deck, |e| e.replace(main.clone(), declared)),
        });
    }
    let mut double_mark = vec![0xEF, 0xBB, 0xBF, 0xEF, 0xBB, 0xBF];
    double_mark.extend_from_slice(&main_xml);
    out.push(Damaged {
        kind: "wrong-encoding",
        label: format!("{main}#two-marks"),
        bytes: with_edits(deck, |e| e.replace(main.clone(), double_mark)),
    });

    // Wrong content types.
    let types_name = parts.first(|n| n == "[content_types].xml").cloned();
    if let Some(types_name) = types_name {
        let types = String::from_utf8_lossy(&source.read_part(&types_name).unwrap()).into_owned();
        for (label, wrong) in [
            (
                "main",
                types.replace(
                    "presentationml.presentation.main+xml",
                    "spreadsheetml.sheet.main+xml",
                ),
            ),
            (
                "slides",
                types.replace("presentationml.slide+xml", "presentationml.nonsense+xml"),
            ),
            (
                "layouts",
                types.replace("presentationml.slideLayout+xml", "application/xml"),
            ),
            (
                "masters",
                types.replace("presentationml.slideMaster+xml", "application/xml"),
            ),
            (
                "theme",
                types.replace("officedocument.theme+xml", "officedocument.nonsense+xml"),
            ),
            (
                "no-rels",
                types.replace("package.relationships+xml", "nonsense+xml"),
            ),
            ("no-overrides", {
                let head = types.find("<Override").unwrap_or(types.len());
                let tail = types.rfind("</Types>").unwrap_or(types.len());
                format!("{}{}", &types[..head], &types[tail..])
            }),
            (
                "empty",
                "<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"/>"
                    .to_owned(),
            ),
        ] {
            out.push(Damaged {
                kind: "wrong-content-types",
                label: label.to_owned(),
                bytes: with_edits(deck, |e| {
                    e.replace(types_name.clone(), wrong.into_bytes());
                }),
            });
        }
    }

    // Relationships that lie.
    if let Some(root_rels) = parts.first(|n| n == "_rels/.rels").cloned() {
        let rels = String::from_utf8_lossy(&source.read_part(&root_rels).unwrap()).into_owned();
        for (label, wrong) in [
            (
                "spoofed-type",
                rels.replace(
                    "relationships/officeDocument",
                    "relationships/officeDokument",
                ),
            ),
            (
                "no-target",
                rels.replace("ppt/presentation.xml", "ppt/nothing.xml"),
            ),
            (
                "escapes",
                rels.replace("ppt/presentation.xml", "../../../etc/passwd"),
            ),
        ] {
            out.push(Damaged {
                kind: "wrong-relationships",
                label: label.to_owned(),
                bytes: with_edits(deck, |e| e.replace(root_rels.clone(), wrong.into_bytes())),
            });
        }
    }
}

fn corpus(name: &str, deck: &[u8], scale: usize, full: bool) -> Vec<Damaged> {
    let mut out = Vec::new();
    container_damage(name, deck, scale, &mut out);
    part_damage(name, deck, scale, full, &mut out);
    out
}

/// What became of a damaged deck.
struct Outcome {
    refused: Option<&'static str>,
    edited: Option<Vec<u8>>,
    repaired: Vec<&'static str>,
}

/// The gate for one deck. An `Err` is a broken rule, with what it was.
fn run_one(bytes: &[u8]) -> Result<Outcome, String> {
    let limits = OoxmlLimits::default();
    let import_limits = ImportLimits::default();
    let mut pkg = match PresentationPackage::open(bytes.to_vec(), limits) {
        Ok(pkg) => pkg,
        // Only what docs/29 lists: nothing is recoverable, the whole-file limit, Compound File.
        Err(err) => {
            let code = err.code();
            return if matches!(code, "CS-PKG-0005" | "CS-PKG-0001" | "CS-OPC-0005") {
                Ok(Outcome {
                    refused: Some(code),
                    edited: None,
                    repaired: Vec::new(),
                })
            } else {
                Err(format!("refused outside docs/29: {err}"))
            };
        }
    };
    if pkg.save_unedited() != bytes {
        return Err("an unedited save changed the bytes, before reading".to_owned());
    }
    let imported =
        import_deck(&mut pkg, &import_limits).map_err(|err| format!("import_deck: {err}"))?;
    let slides = import_slide_parts(
        &mut pkg,
        &imported.deck.presentation,
        &SlideLimits::default(),
        &import_limits,
    )
    .map_err(|err| format!("import_slide_parts: {err}"))?;
    if slides.parts.structure.slides.iter().any(|ordinal| {
        slides
            .parts
            .get(*ordinal)
            .and_then(casual_slide_model::slide::SlidePart::shape_tree)
            .is_none()
    }) {
        return Err("a slide is not there to look at".to_owned());
    }
    if pkg.save_unedited() != bytes {
        return Err("an unedited save changed the bytes, after reading".to_owned());
    }
    let mut after = imported.deck.clone();
    after
        .properties
        .core
        .get_or_insert_with(CoreProperties::default)
        .title = Some("Edited after recovery".to_owned());
    let edited = save_deck(&mut pkg, &after, &import_limits)
        .map_err(|err| format!("the edit was refused: {err}"))?;
    if edited.bytes == bytes {
        return Err("an edit left the bytes as they were".to_owned());
    }
    // The writer read it back strictly already. This is that this is what it did.
    PresentationPackage::open_strict(edited.bytes.clone(), limits)
        .map_err(|err| format!("the output does not open strictly: {err}"))?;
    let mut repaired: Vec<&'static str> = pkg
        .findings()
        .iter()
        .filter(|f| matches!(f.kind, FindingKind::Repaired(_)))
        .map(|f| f.kind.code())
        .collect();
    repaired.sort_unstable();
    repaired.dedup();
    Ok(Outcome {
        refused: None,
        edited: Some(edited.bytes),
        repaired,
    })
}

/// Run F4 over `dir`, by batches on as many threads as there are cores. Every failure, by
/// file name.
fn f4_failures(dir: &Path) -> Vec<String> {
    let files: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    let threads = workers();
    let chunk = files.len().div_ceil(threads).max(1);
    let script = root().join("tools/f4/validate.py");
    std::thread::scope(|scope| {
        let workers: Vec<_> = files
            .chunks(chunk)
            .map(|batch| {
                let script = &script;
                scope.spawn(move || {
                    let done = Command::new("python3")
                        .arg("-I")
                        .arg(script)
                        .args(batch)
                        .output()
                        .expect("python3 runs");
                    let code = done.status.code();
                    let stdout = String::from_utf8_lossy(&done.stdout).into_owned();
                    if code == Some(2) || code.is_none() {
                        panic!(
                            "F4 could not run: {stdout} {}",
                            String::from_utf8_lossy(&done.stderr)
                        );
                    }
                    failures_in(&stdout)
                })
            })
            .collect();
        workers
            .into_iter()
            .flat_map(|w| w.join().expect("an F4 batch finished"))
            .collect()
    })
}

/// F4's text output, reduced to what failed: `deck: part: first message`. A deck's line says
/// `name: PASS` or `name: FAIL`; a part's is indented two, and a message six.
fn failures_in(stdout: &str) -> Vec<String> {
    let mut deck = "";
    let mut failing_part = false;
    let mut out = Vec::new();
    for line in stdout.lines() {
        if !line.starts_with(' ') {
            deck = line.split(':').next().unwrap_or_default();
            if line.contains("(refused") {
                out.push(line.to_owned());
            }
            failing_part = false;
        } else if line.starts_with("      ") {
            if failing_part {
                out.push(format!("{deck}:   {}", line.trim()));
                failing_part = false;
            }
        } else if let Some(body) = line.strip_prefix("  ") {
            let status = body.split_whitespace().next().unwrap_or_default();
            failing_part =
                !matches!(status, "valid" | "no-schema" | "recorded") && !body.starts_with('(');
            if failing_part {
                out.push(format!("{deck}: {}", body.trim()));
            }
        }
    }
    out
}

#[test]
fn every_damaged_deck_opens_or_is_refused_only_by_the_rules() {
    let full = std::env::var_os("DAMAGE_FULL").is_some();
    let dir = root().join("fixtures/real");
    let mut decks: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "pptx"))
        .collect();
    decks.sort();
    if !full {
        // The smaller ones, one of them Strict. `DAMAGE_FULL=1` is every deck.
        decks.retain(|p| std::fs::metadata(p).unwrap().len() < 50_000);
    }
    let scale = if full { 2 } else { 1 };
    let out = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("damage-corpus");
    let _ = std::fs::remove_dir_all(&out);
    std::fs::create_dir_all(&out).unwrap();

    let mut opened: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut refused: BTreeMap<(&'static str, &'static str), usize> = BTreeMap::new();
    let mut codes: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut broken_rules = Vec::new();
    let mut slowest = Duration::ZERO;
    let mut total = 0usize;
    let mut ordinal = 0usize;
    // Each output carries its source's name, which F4's table of recorded failures reads.
    let mut sources = BTreeMap::new();
    let threads = workers();
    let mut all: Vec<(String, Damaged)> = Vec::new();
    for path in &decks {
        let name = path.file_stem().unwrap().to_string_lossy().into_owned();
        let deck = std::fs::read(path).unwrap();
        for damaged in corpus(&name, &deck, scale, full) {
            all.push((name.clone(), damaged));
        }
    }
    // Read in parallel, reported in order, so a failure's place does not depend on timing.
    let chunk = all.len().div_ceil(threads).max(1);
    let results: Vec<_> = std::thread::scope(|scope| {
        let workers: Vec<_> = all
            .chunks(chunk)
            .map(|batch| {
                scope.spawn(move || {
                    batch
                        .iter()
                        .map(|(_, damaged)| {
                            let started = Instant::now();
                            let outcome = std::panic::catch_unwind(|| run_one(&damaged.bytes));
                            (outcome, started.elapsed())
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        workers
            .into_iter()
            .flat_map(|w| w.join().expect("a worker finished"))
            .collect()
    });
    {
        for ((name, damaged), (outcome, took)) in all.iter().zip(results) {
            total += 1;
            slowest = slowest.max(took);
            let label = format!("{name} {} {}", damaged.kind, damaged.label);
            if took > MAX_ONE_DECK {
                broken_rules.push(format!("{label}: took {took:?}"));
            }
            let outcome = match outcome {
                Ok(outcome) => outcome,
                Err(_) => {
                    broken_rules.push(format!("{label}: PANIC"));
                    continue;
                }
            };
            match outcome {
                Err(what) => broken_rules.push(format!("{label}: {what}")),
                Ok(Outcome {
                    refused: Some(code),
                    ..
                }) => {
                    *refused.entry((damaged.kind, code)).or_default() += 1;
                }
                Ok(outcome) => {
                    *opened.entry(damaged.kind).or_default() += 1;
                    for code in &outcome.repaired {
                        *codes.entry(code).or_default() += 1;
                    }
                    if let Some(bytes) = outcome.edited {
                        ordinal += 1;
                        let file = format!("{name}--{ordinal:05}.pptx");
                        std::fs::write(out.join(&file), bytes).unwrap();
                        sources.insert(file, label);
                    }
                }
            }
        }
    }

    println!("damage corpus: {} decks, {total} damaged", decks.len());
    let opens: usize = opened.values().sum();
    let refusals: usize = refused.values().sum();
    println!("opened {opens}, refused {refusals}, slowest {slowest:?}");
    for (kind, n) in &opened {
        println!("  open   {kind}: {n}");
    }
    for ((kind, code), n) in &refused {
        println!("  refuse {kind} {code}: {n}");
    }
    for (code, n) in &codes {
        println!("  repair {code}: {n}");
    }
    assert!(total > 100, "only {total} damaged decks");
    assert!(
        broken_rules.is_empty(),
        "{} of {total} broke a rule:\n{}",
        broken_rules.len(),
        broken_rules.join("\n")
    );

    // F4 needs `python3` and `xmllint`. `F4_REQUIRED` means a missing tool fails.
    let have = |tool: &str, arg: &str| Command::new(tool).arg(arg).output().is_ok();
    if !have("python3", "--version") || !have("xmllint", "--version") {
        assert!(
            std::env::var_os("F4_REQUIRED").is_none(),
            "F4_REQUIRED is set, and python3 or xmllint is missing"
        );
        eprintln!("xmllint or python3 not installed: F4 skipped");
        return;
    }
    let failures = f4_failures(&out);
    println!(
        "F4: {} outputs, {} failing lines",
        sources.len(),
        failures.len()
    );
    if !failures.is_empty() {
        let shown: Vec<String> = failures
            .iter()
            .take(60)
            .map(|line| {
                let file = line.split(':').next().unwrap_or_default();
                let source = sources.get(file).map_or("?", String::as_str);
                format!("[{source}] {line}")
            })
            .collect();
        panic!(
            "{} F4 failures in {} outputs:\n{}",
            failures.len(),
            sources.len(),
            shown.join("\n")
        );
    }
    let _ = std::fs::remove_dir_all(&out);
}
