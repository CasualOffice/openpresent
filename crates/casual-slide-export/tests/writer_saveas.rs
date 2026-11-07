//! The save-as grid of `docs/36` (`WR-05`), one row per pair in `fixtures/rr/sa/manifest.tsv`,
//! with the written deck beside it where a pair writes one. `RR_WRITE=1` regenerates them.
//! The grid in the code is held to the grid in the document. None of this says anything
//! about PowerPoint until `PV-01` has opened them.
//!
//! Also the replaced-media path (`T1N-08`): a replacement needing a new relationship saves.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

use casual_slide_export::media::replace_media;
use casual_slide_export::writer::{
    FileFormat, HostInputs, MediaBytes, SaveAsCell, SavePlan, media_replacement_op,
    new_from_template, new_media_for, save_as, save_as_cell,
};
use casual_slide_import::presentation::import_deck;
use casual_slide_import::properties::ImportLimits;
use casual_slide_import::slide::{SlideLimits, import_slide_parts};
use casual_slide_model::shape::ObjectId;
use casual_slide_model::slide::{SlidePart, SlideParts};
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage, XmlTree};

const MAIN_TYPES: &[(&str, &str)] = &[
    (
        "pptx",
        "application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml",
    ),
    (
        "ppsx",
        "application/vnd.openxmlformats-officedocument.presentationml.slideshow.main+xml",
    ),
    (
        "potx",
        "application/vnd.openxmlformats-officedocument.presentationml.template.main+xml",
    ),
    (
        "pptm",
        "application/vnd.ms-powerpoint.presentation.macroEnabled.main+xml",
    ),
    (
        "ppsm",
        "application/vnd.ms-powerpoint.slideshow.macroEnabled.main+xml",
    ),
    (
        "potm",
        "application/vnd.ms-powerpoint.template.macroEnabled.main+xml",
    ),
];
const VBA: &str = "ppt/vbaProject.bin";
const TYPES: &str = "[Content_Types].xml";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn input(format: FileFormat) -> Vec<u8> {
    let ext = format.extension();
    let rel = if MAIN_TYPES.iter().any(|(e, _)| *e == ext) {
        format!("fixtures/generated/rr-sa.{ext}")
    } else {
        format!("fixtures/rr/inputs/sa.{ext}")
    };
    std::fs::read(root().join(&rel)).unwrap_or_else(|_| panic!("{rel}"))
}

fn open(bytes: &[u8]) -> PresentationPackage {
    PresentationPackage::open(bytes.to_vec(), OoxmlLimits::default()).unwrap()
}

fn parts_of(bytes: &[u8]) -> BTreeMap<String, Vec<u8>> {
    let mut pkg = open(bytes);
    pkg.part_names()
        .into_iter()
        .map(|n| {
            let b = pkg.read_part(&n).unwrap();
            (n, b)
        })
        .collect()
}

fn text(parts: &BTreeMap<String, Vec<u8>>, name: &str) -> String {
    String::from_utf8(parts[name].clone()).unwrap()
}

/// The grid as `docs/36` writes it.
fn documented_grid() -> Vec<(String, Vec<String>)> {
    let doc = std::fs::read_to_string(root().join("docs/36-EXPORT-AND-SAVE.md")).unwrap();
    let cells = |line: &str| -> Vec<String> {
        line.trim()
            .trim_matches('|')
            .split('|')
            .map(|c| {
                c.trim()
                    .trim_matches('*')
                    .trim_start_matches('.')
                    .to_owned()
            })
            .collect()
    };
    let mut lines = doc.lines().skip_while(|l| !l.starts_with("| Open \\ Save"));
    let head = cells(lines.next().unwrap());
    let wanted: Vec<&str> = FileFormat::ALL.iter().map(|f| f.extension()).collect();
    assert_eq!(&head[1..], &wanted[..], "the grid's columns");
    lines
        .skip(1)
        .take_while(|l| l.starts_with('|'))
        .map(|l| {
            let row = cells(l);
            (row[0].clone(), row[1..].to_vec())
        })
        .collect()
}

#[test]
fn the_grid_in_the_code_is_the_grid_in_the_document() {
    let grid = documented_grid();
    assert_eq!(grid.len(), 11);
    for (open, (name, row)) in FileFormat::ALL.iter().zip(&grid) {
        assert_eq!(name, open.extension());
        assert_eq!(row.len(), 11, "{name}");
        for (save, cell) in FileFormat::ALL.iter().zip(row) {
            assert_eq!(
                save_as_cell(*open, *save).code(),
                cell,
                "{} to {}",
                open.extension(),
                save.extension()
            );
        }
    }
}

/// What a written retype must be: the type asked for, and no other part changed but the
/// types stream and, with VBA dropped, what the project's removal takes.
fn check_written(source: &[u8], written: &[u8], save: FileFormat, drop_vba: bool, id: &str) {
    let before = parts_of(source);
    let after = parts_of(written);
    let wanted = MAIN_TYPES
        .iter()
        .find(|(e, _)| *e == save.extension())
        .unwrap()
        .1;
    let types = text(&after, TYPES);
    assert!(
        types.contains(&format!(
            "PartName=\"/ppt/presentation.xml\" ContentType=\"{wanted}\""
        )),
        "{id}: {types}"
    );
    // INV-SA-03, INV-SA-05.
    assert_eq!(
        after.contains_key(VBA),
        before.contains_key(VBA) && !drop_vba,
        "{id}"
    );
    // INV-SA-02.
    for (name, bytes) in &before {
        if name == TYPES {
            continue;
        }
        let allowed = drop_vba && (name == VBA || name == "ppt/_rels/presentation.xml.rels");
        if !allowed {
            assert_eq!(after.get(name), Some(bytes), "{id}: {name}");
        }
    }
    assert!(
        after.keys().all(|k| before.contains_key(k)),
        "{id}: a part was added"
    );
}

#[test]
fn every_pair_does_what_its_cell_says() {
    let write = std::env::var_os("RR_WRITE").is_some();
    let dir = root().join("fixtures/rr/sa");
    let mut manifest = String::from("id\topen\tsave\tcell\toutcome\tfile\n");
    let mut count = 0usize;
    for open_as in FileFormat::ALL {
        let source = input(open_as);
        for save in FileFormat::ALL {
            let cell = save_as_cell(open_as, save);
            let id = format!("RR-SA-{}-{}", open_as.extension(), save.extension());
            let result = save_as(
                &source,
                open_as,
                save,
                SavePlan::new(),
                true,
                OoxmlLimits::default(),
            );
            let outcome = match (cell, result) {
                (SaveAsCell::Same, Ok(done)) => {
                    assert_eq!(done.bytes, source, "{id}: F1");
                    Ok(done.bytes)
                }
                (
                    SaveAsCell::Retype | SaveAsCell::RetypeKeepVba | SaveAsCell::RetypeDropVba,
                    Ok(done),
                ) => {
                    let drop_vba = cell == SaveAsCell::RetypeDropVba;
                    check_written(&source, &done.bytes, save, drop_vba, &id);
                    if drop_vba {
                        assert!(
                            done.report.iter().any(|e| e.code == "CS-SAV-0002"),
                            "{id}: {:?}",
                            done.report
                        );
                        // Without the confirmation, refused.
                        let err = save_as(
                            &source,
                            open_as,
                            save,
                            SavePlan::new(),
                            false,
                            OoxmlLimits::default(),
                        )
                        .unwrap_err();
                        assert_eq!(err.code(), "CS-SAV-0003", "{id}");
                    }
                    Ok(done.bytes)
                }
                (_, Err(err)) => {
                    // `INV-SA-10`: `.ppt` is refused by its own code until Phase 10.
                    let want = match (open_as, cell) {
                        (FileFormat::Ppt, SaveAsCell::Convert) => "CS-OPC-0005",
                        _ => "CS-SAV-0013",
                    };
                    assert_eq!(err.code(), want, "{id}: {err}");
                    assert!(
                        !matches!(
                            cell,
                            SaveAsCell::Same
                                | SaveAsCell::Retype
                                | SaveAsCell::RetypeKeepVba
                                | SaveAsCell::RetypeDropVba
                        ),
                        "{id}: {err}"
                    );
                    Err(format!("refused {}", err.code()))
                }
                (cell, Ok(_)) => panic!("{id}: {} wrote a file", cell.code()),
            };
            count += 1;
            let (result_text, file) = match outcome {
                Ok(bytes) => {
                    let file = format!("{id}.{}", save.extension());
                    let path = dir.join(&file);
                    if write {
                        std::fs::create_dir_all(&dir).unwrap();
                        std::fs::write(&path, &bytes).unwrap();
                    } else {
                        let held = std::fs::read(&path)
                            .unwrap_or_else(|_| panic!("{id}: run with RR_WRITE=1"));
                        assert_eq!(held, bytes, "{id}: the writer has moved");
                    }
                    ("written".to_owned(), file)
                }
                Err(why) => (why, "-".to_owned()),
            };
            writeln!(
                manifest,
                "{id}\t{}\t{}\t{}\t{result_text}\t{file}",
                open_as.extension(),
                save.extension(),
                cell.code()
            )
            .unwrap();
        }
    }
    assert_eq!(count, 121);
    let path = dir.join("manifest.tsv");
    if write {
        std::fs::write(&path, &manifest).unwrap();
    } else {
        let held = std::fs::read_to_string(&path)
            .unwrap()
            .replace("\r\n", "\n");
        assert_eq!(held, manifest);
    }
}

/// F4 on every deck the grid writes.
#[test]
fn every_written_pair_validates() {
    let have = |tool: &str, arg: &str| Command::new(tool).arg(arg).output().is_ok();
    if !have("python3", "--version") || !have("xmllint", "--version") {
        assert!(
            std::env::var_os("F4_REQUIRED").is_none(),
            "F4_REQUIRED is set, and python3 or xmllint is missing"
        );
        eprintln!("xmllint or python3 not installed: F4 skipped");
        return;
    }
    let out = Command::new("python3")
        .arg("-I")
        .arg(root().join("tools/f4/validate.py"))
        .arg(root().join("fixtures/rr/sa"))
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
}

/// `INV-SA-07`: two verbs. *Save as* keeps VBA in a macro type; *new from template* never
/// inherits it, starts with a new `created`, and needs no confirmation.
#[test]
fn a_new_deck_from_a_template_has_no_project_and_a_new_created() {
    let source = input(FileFormat::Potm);
    let kept = save_as(
        &source,
        FileFormat::Potm,
        FileFormat::Pptm,
        SavePlan::new(),
        false,
        OoxmlLimits::default(),
    )
    .unwrap();
    assert!(parts_of(&kept.bytes).contains_key(VBA));
    let made = new_from_template(&source, SavePlan::new(), OoxmlLimits::default()).unwrap();
    let parts = parts_of(&made.bytes);
    assert!(!parts.contains_key(VBA));
    assert!(text(&parts, TYPES).contains(MAIN_TYPES[0].1));

    let deck = std::fs::read(root().join("fixtures/generated/rr-app.pptx")).unwrap();
    let template = save_as(
        &deck,
        FileFormat::Pptx,
        FileFormat::Potx,
        SavePlan::new(),
        false,
        OoxmlLimits::default(),
    )
    .unwrap();
    let when = "2026-11-07T10:00:00Z";
    let made = new_from_template(
        &template.bytes,
        SavePlan::new().host(HostInputs {
            modified: Some(when.to_owned()),
            ..HostInputs::default()
        }),
        OoxmlLimits::default(),
    )
    .unwrap();
    let core = text(&parts_of(&made.bytes), "docProps/core.xml");
    assert!(!core.contains("2020-01-01"), "{core}");
    assert_eq!(core.matches(when).count(), 2, "{core}");
}

/// `INV-SA-08`, `-09`, `-11`: these return their own bytes, and are not edited here.
#[test]
fn a_format_that_is_not_edited_refuses_an_edit() {
    for format in [FileFormat::Ppam, FileFormat::Thmx, FileFormat::Odp] {
        let source = input(format);
        let plan = SavePlan::new().op(casual_slide_export::writer::Op::DropVba);
        let err = save_as(&source, format, format, plan, false, OoxmlLimits::default());
        assert_eq!(err.unwrap_err().code(), "CS-SAV-0009", "{format:?}");
    }
}

/// `INV-SA-10`, `-12`: opening these is refused, each by its own code. `INV-SA-13`: a
/// slideshow opens like any deck, and nothing here can start a show.
#[test]
fn a_pdf_and_a_ppt_do_not_open_and_a_slideshow_does() {
    let refused = |format, code: &str| {
        let err = PresentationPackage::open(input(format), OoxmlLimits::default()).unwrap_err();
        assert_eq!(err.code(), code);
    };
    refused(FileFormat::Pdf, "CS-PKG-0005");
    refused(FileFormat::Ppt, "CS-OPC-0005");
    open(&input(FileFormat::Ppsx));
}

/// The decks and inputs of this tier against their rows in `fixtures/manifest.tsv`.
#[test]
fn every_input_of_this_tier_meets_its_manifest_row() {
    let manifest = std::fs::read_to_string(root().join("fixtures/manifest.tsv")).unwrap();
    let mut seen = 0usize;
    let mut wrong = Vec::new();
    for line in manifest.lines().skip(1) {
        let cols: Vec<&str> = line.split('\t').collect();
        let is_ours = [
            "rr-app",
            "rr-core",
            "rr-signed",
            "rr-thumb",
            "rr-macro",
            "rr-activex",
            "rr-sa.",
        ]
        .iter()
        .any(|p| cols[0].contains(p))
            || cols[0].starts_with("rr/inputs/");
        if !is_ours {
            continue;
        }
        seen += 1;
        let bytes = std::fs::read(root().join("fixtures").join(cols[0])).unwrap();
        let result = PresentationPackage::open_strict(bytes, OoxmlLimits::default());
        if let Some(code) = cols[2].strip_prefix("refuse:") {
            assert_eq!(result.unwrap_err().code(), code, "{}", cols[0]);
            continue;
        }
        let pkg = result.unwrap_or_else(|e| panic!("{}: {e}", cols[0]));
        let mut codes: Vec<&str> = pkg.findings().iter().map(|f| f.kind.code()).collect();
        codes.sort_unstable();
        codes.dedup();
        let wanted: Vec<&str> = cols[2].split(' ').skip(1).collect();
        if codes != wanted {
            wrong.push(format!("{}: {codes:?}", cols[0]));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    assert_eq!(seen, 19);
}

fn model(bytes: &[u8]) -> SlideParts {
    let mut pkg = open(bytes);
    let deck = import_deck(&mut pkg, &ImportLimits::default())
        .unwrap()
        .deck;
    import_slide_parts(
        &mut pkg,
        &deck.presentation,
        &SlideLimits::default(),
        &ImportLimits::default(),
    )
    .unwrap()
    .parts
}

fn slide_model(parts: &SlideParts, name: &str) -> SlidePart {
    parts
        .structure
        .parts
        .iter()
        .find(|e| e.part == name)
        .and_then(|e| parts.get(e.id.part))
        .unwrap()
        .clone()
}

fn node_named(part: &SlidePart, name: &str) -> ObjectId {
    part.shape_tree()
        .unwrap()
        .descendants()
        .find(|n| {
            n.non_visual
                .as_ref()
                .is_some_and(|v| v.name.as_deref() == Some(name))
        })
        .unwrap()
        .id
}

fn blip_embed(part: &SlidePart, name: &str) -> String {
    let id = node_named(part, name);
    part.shape_tree()
        .unwrap()
        .descendants()
        .find(|n| n.id == id)
        .unwrap()
        .properties
        .picture
        .as_ref()
        .unwrap()
        .blip
        .as_ref()
        .unwrap()
        .embed
        .clone()
        .unwrap()
}

/// `T1N-08`. Slide 1's picture shares its image with slide 2. Replacing it needs a new
/// relationship and a new part; the saved deck, opened again, shows the new image there and
/// the old one on slide 2. Then the writer's own preflight and F4 judge the file.
#[test]
fn a_replacement_needing_a_new_relationship_saves_and_reopens_with_the_new_image() {
    const SLIDE: &str = "ppt/slides/slide1.xml";
    const OLD: &str = "ppt/media/image1.png";
    let new_image: &[u8] = b"\x89PNG\r\n\x1a\nreplacement";
    let source = std::fs::read(root().join("fixtures/generated/rr-shared-media.pptx")).unwrap();
    let mut pkg = open(&source);
    let rels = pkg.relationships_of(SLIDE).to_vec();
    let bytes = pkg.read_part(SLIDE).unwrap();
    let mut tree = XmlTree::parse(&bytes, SLIDE, &OoxmlLimits::default()).unwrap();
    let parts = model(&source);
    let mut part = slide_model(&parts, SLIDE);
    let node = node_named(&part, "Picture 4");
    let media = MediaBytes {
        bytes: new_image.to_vec(),
        extension: "png".to_owned(),
        content_type: "image/png".to_owned(),
    };
    let new = new_media_for(&mut pkg, SLIDE, &media).unwrap();
    let done = replace_media(
        &mut tree,
        SLIDE,
        false,
        &rels,
        &mut part,
        node,
        "rId2",
        &new,
        &SlideLimits::default(),
        &ImportLimits::default(),
        &OoxmlLimits::default(),
    )
    .unwrap();
    assert!(done.added.is_some(), "the test needs a new relationship");

    let op = media_replacement_op(SLIDE, &done, tree.to_bytes(), Some(media));
    let saved = SavePlan::new().op(op).save(&mut pkg).unwrap();

    // Opened again, by a reader that has not seen any of this.
    let mut again = open(&saved.bytes);
    let found = model(&saved.bytes);
    let slide1 = slide_model(&found, SLIDE);
    let embed = blip_embed(&slide1, "Picture 4");
    let rel = again
        .relationships_of(SLIDE)
        .iter()
        .find(|r| r.id == embed)
        .cloned()
        .unwrap();
    let target = format!("ppt/media/{}", rel.target.rsplit('/').next().unwrap());
    assert_eq!(again.read_part(&target).unwrap(), new_image);
    // Slide 2 still reaches the old image, so the part stayed.
    let slide2 = slide_model(&found, "ppt/slides/slide2.xml");
    let embed2 = blip_embed(&slide2, "Picture 4");
    let rel2 = again
        .relationships_of("ppt/slides/slide2.xml")
        .iter()
        .find(|r| r.id == embed2)
        .cloned()
        .unwrap();
    assert!(rel2.target.ends_with("image1.png"), "{rel2:?}");
    assert!(again.read_part(OLD).is_ok());

    let f4 = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("media-replaced");
    std::fs::create_dir_all(&f4).unwrap();
    std::fs::write(f4.join("replaced.pptx"), &saved.bytes).unwrap();
    let have = Command::new("xmllint").arg("--version").output().is_ok();
    if have {
        let out = Command::new("python3")
            .arg("-I")
            .arg(root().join("tools/f4/validate.py"))
            .arg(&f4)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stdout)
        );
    }
}
