//! Tier 1D export: edit a node, save, reopen; only that node changed, every other
//! part is byte-identical, a second save is a fixed point, and the result validates.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use casual_slide_export::slide::save_slide_parts;
use casual_slide_import::presentation::import_deck;
use casual_slide_import::properties::ImportLimits;
use casual_slide_import::slide::{SlideLimits, import_slide_parts};
use casual_slide_model::presentation::{Coordinate, Point, PositiveSize};
use casual_slide_model::shape::{Locks, PlaceholderType, ShapeNode, Transform};
use casual_slide_model::slide::{HeaderFooter, SlidePart, SlideParts};
use casual_slide_model::theme::{ColorMapOverride, ColorMapSlot, ColorSchemeIndex};
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage};

const RICH: &str = "generated/slides-masters-layouts.pptx";
const STRICT: &str = "generated/slides-strict.pptx";
const SLIDE2: &str = "ppt/slides/slide2.xml";

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn read(path: &str) -> Vec<u8> {
    std::fs::read(fixtures().join(path)).unwrap()
}

fn open(bytes: Vec<u8>) -> PresentationPackage {
    PresentationPackage::open(bytes, OoxmlLimits::default()).unwrap()
}

fn model(bytes: &[u8]) -> SlideParts {
    let mut pkg = open(bytes.to_vec());
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

fn parts(bytes: &[u8]) -> BTreeMap<String, Vec<u8>> {
    let mut pkg = open(bytes.to_vec());
    pkg.part_names()
        .into_iter()
        .map(|n| {
            let b = pkg.read_part(&n).unwrap();
            (n, b)
        })
        .collect()
}

fn named_mut<'a>(part: &'a mut SlidePart, name: &str) -> &'a mut ShapeNode {
    let id = part
        .shape_tree()
        .unwrap()
        .descendants()
        .find(|n| n.non_visual.as_ref().and_then(|v| v.name.as_deref()) == Some(name))
        .unwrap_or_else(|| panic!("no {name}"))
        .id;
    part.node_mut(id).unwrap()
}

/// Save these parts of `source` after `change`. Returns the outcome's bytes, having
/// checked the edit reads back, a second save changes nothing, and every part not
/// named in the outcome is byte-identical.
fn try_edit(
    source: &[u8],
    ordinals: &[u32],
    change: impl FnOnce(&mut SlideParts),
) -> Result<Vec<u8>, String> {
    let mut wanted = model(source);
    change(&mut wanted);
    let mut pkg = open(source.to_vec());
    let edited: Vec<&SlidePart> = ordinals.iter().map(|&o| wanted.get(o).unwrap()).collect();
    let outcome = save_slide_parts(
        &mut pkg,
        &wanted.structure,
        &edited,
        &SlideLimits::default(),
        &ImportLimits::default(),
    )
    .map_err(|e| e.code().to_owned())?;
    assert_eq!(model(&outcome.bytes), wanted, "the edit reads back");

    let mut again = open(outcome.bytes.clone());
    let second = save_slide_parts(
        &mut again,
        &wanted.structure,
        &edited,
        &SlideLimits::default(),
        &ImportLimits::default(),
    )
    .unwrap();
    assert_eq!(second.bytes, outcome.bytes, "fixed point");
    assert!(second.changed_parts.is_empty());

    let (old, new) = (parts(source), parts(&outcome.bytes));
    assert_eq!(old.len(), new.len(), "parts were lost or invented");
    for (name, bytes) in &old {
        if !outcome.changed_parts.contains(name) {
            assert_eq!(new.get(name), Some(bytes), "{name} changed");
        }
    }
    Ok(outcome.bytes)
}

fn edit(source: &[u8], ordinals: &[u32], change: impl FnOnce(&mut SlideParts)) -> Vec<u8> {
    try_edit(source, ordinals, change).unwrap()
}

/// `from` occurs once in `part`; the edit turned it into `to`, and changed no other
/// byte of any part.
fn assert_only_this_changed(source: &[u8], edited: &[u8], part: &str, from: &str, to: &str) {
    let old = String::from_utf8(parts(source)[part].clone()).unwrap();
    let new = String::from_utf8(parts(edited)[part].clone()).unwrap();
    assert_eq!(old.matches(from).count(), 1, "{from} in {part}");
    assert_eq!(new, old.replace(from, to));
    let (a, b) = (parts(source), parts(edited));
    let changed: Vec<&String> = b
        .iter()
        .filter(|(n, v)| a.get(*n) != Some(v))
        .map(|(n, _)| n)
        .collect();
    assert_eq!(changed, [part]);
}

fn at(x: i64, y: i64) -> Point {
    Point {
        x: Coordinate(x),
        y: Coordinate(y),
    }
}

#[test]
fn editing_a_shape_transform_changes_only_that_node() {
    let source = read(RICH);
    let edited = edit(&source, &[5], |m| {
        let node = named_mut(m.get_mut(5).unwrap(), "Child B");
        node.transform.as_mut().unwrap().offset = Some(at(1_000_000, 0));
    });
    assert_only_this_changed(
        &source,
        &edited,
        SLIDE2,
        r#"<a:off x="914400" y="0"/>"#,
        r#"<a:off x="1000000" y="0"/>"#,
    );
}

type Change = fn(&mut SlideParts);

/// One edit of each kind this tier writes, on the deck full of every construct.
#[test]
fn each_kind_of_edit_changes_only_its_node() {
    let cases: &[(u32, &str, &str, &str, Change)] = &[
        (
            5,
            SLIDE2,
            r#"<a:xfrm rot="5400000" flipH="1">"#,
            r#"<a:xfrm rot="10800000">"#,
            |m| {
                let t = named_mut(m.get_mut(5).unwrap(), "Group 9")
                    .transform
                    .as_mut()
                    .unwrap();
                t.rotation = Some(10_800_000);
                t.flip_h = None;
            },
        ),
        (
            5,
            SLIDE2,
            r#"<a:chExt cx="1828800" cy="914400"/>"#,
            r#"<a:chExt cx="914400" cy="914400"/>"#,
            |m| {
                let t = named_mut(m.get_mut(5).unwrap(), "Group 9")
                    .transform
                    .as_mut()
                    .unwrap();
                t.child_extent = Some(PositiveSize {
                    cx: 914_400,
                    cy: 914_400,
                });
            },
        ),
        (
            5,
            SLIDE2,
            r#"<a:off x="6000000" y="1825625"/>"#,
            r#"<a:off x="6000000" y="2000000"/>"#,
            |m| {
                named_mut(m.get_mut(5).unwrap(), "Table 16")
                    .transform
                    .as_mut()
                    .unwrap()
                    .offset = Some(at(6_000_000, 2_000_000));
            },
        ),
        (
            5,
            SLIDE2,
            r#"<p:cNvPr id="16" name="Picture 15" descr="A checker"/>"#,
            r#"<p:cNvPr id="16" name="Picture 15" descr="A grid" title="Grid"/>"#,
            |m| {
                let nv = named_mut(m.get_mut(5).unwrap(), "Picture 15")
                    .non_visual
                    .as_mut()
                    .unwrap();
                nv.descr = Some("A grid".to_owned());
                nv.title = Some("Grid".to_owned());
            },
        ),
        (
            5,
            SLIDE2,
            r#"<p:cNvPr id="5" name="Hidden" hidden="1">"#,
            r#"<p:cNvPr id="5" name="Hidden">"#,
            |m| {
                named_mut(m.get_mut(5).unwrap(), "Hidden")
                    .non_visual
                    .as_mut()
                    .unwrap()
                    .hidden = None;
            },
        ),
        (
            5,
            SLIDE2,
            r#"tooltip="To slide 3""#,
            r#"tooltip="Onward""#,
            |m| {
                let nv = named_mut(m.get_mut(5).unwrap(), "Title 1")
                    .non_visual
                    .as_mut()
                    .unwrap();
                nv.hyperlink_click.as_mut().unwrap().tooltip = Some("Onward".to_owned());
            },
        ),
        (
            5,
            SLIDE2,
            r#"<a:spLocks noMove="1" noResize="1" noTextEdit="1"/>"#,
            r#"<a:spLocks noResize="1" noTextEdit="1" noRot="1"/>"#,
            |m| {
                let locks = named_mut(m.get_mut(5).unwrap(), "Child A")
                    .locks
                    .as_mut()
                    .unwrap();
                locks.no_move = None;
                locks.no_rot = Some(true);
            },
        ),
        (
            5,
            SLIDE2,
            r#"<p:cNvSpPr></p:cNvSpPr><p:nvPr><p:ph idx="1"/>"#,
            r#"<p:cNvSpPr><a:spLocks noGrp="1"/></p:cNvSpPr><p:nvPr><p:ph idx="1"/>"#,
            |m| {
                named_mut(m.get_mut(5).unwrap(), "Content Placeholder 2").locks = Some(Locks {
                    no_grp: Some(true),
                    ..Locks::default()
                });
            },
        ),
        (
            5,
            SLIDE2,
            r#"<p:ph type="body" idx="4294967295"/>"#,
            r#"<p:ph type="body" idx="2"/>"#,
            |m| {
                let node = named_mut(m.get_mut(5).unwrap(), "Orphan Placeholder");
                node.app.as_mut().unwrap().placeholder.as_mut().unwrap().idx = Some(2);
            },
        ),
        (5, SLIDE2, r#"<p:sp useBgFill="1">"#, r#"<p:sp>"#, |m| {
            named_mut(m.get_mut(5).unwrap(), "Child B").use_bg_fill = None;
        }),
        (
            5,
            SLIDE2,
            r#"<p:spPr bwMode="gray">"#,
            r#"<p:spPr bwMode="white">"#,
            |m| {
                named_mut(m.get_mut(5).unwrap(), "Child B").bw_mode =
                    Some(casual_slide_model::shape::BlackWhiteMode::White);
            },
        ),
        (
            5,
            SLIDE2,
            r#"<p:cSld name="Content">"#,
            r#"<p:cSld name="Renamed">"#,
            |m| {
                m.get_mut(5).unwrap().common.as_mut().unwrap().name = Some("Renamed".to_owned());
            },
        ),
        (
            6,
            "ppt/slides/slide3.xml",
            r#"<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" show="0" showMasterSp="0""#,
            r#"<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" show="1" showMasterSp="0""#,
            |m| {
                m.get_mut(6).unwrap().show = Some(true);
            },
        ),
        (
            6,
            "ppt/slides/slide3.xml",
            r#"bg1="dk1" tx1="lt1""#,
            r#"bg1="lt1" tx1="lt1""#,
            |m| {
                if let Some(ColorMapOverride::Override(map)) =
                    &mut m.get_mut(6).unwrap().color_map_override
                {
                    map.slots[ColorMapSlot::Bg1 as usize] = Some(ColorSchemeIndex::Lt1);
                }
            },
        ),
        (
            4,
            "ppt/slides/slide1.xml",
            "<p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>",
            "",
            |m| {
                m.get_mut(4).unwrap().color_map_override = None;
            },
        ),
        (
            1,
            "ppt/slideLayouts/slideLayout1.xml",
            r#"<p:hf sldNum="0"/>"#,
            r#"<p:hf sldNum="0" ftr="0"/>"#,
            |m| {
                m.get_mut(1).unwrap().header_footer = Some(HeaderFooter {
                    slide_number: Some(false),
                    footer: Some(false),
                    ..HeaderFooter::default()
                });
            },
        ),
        (
            1,
            "ppt/slideLayouts/slideLayout1.xml",
            r#"matchingName="Title""#,
            r#"matchingName="Cover""#,
            |m| {
                m.get_mut(1).unwrap().matching_name = Some("Cover".to_owned());
            },
        ),
        (
            2,
            "ppt/slideLayouts/slideLayout2.xml",
            r#"<p:ph type="pic" sz="quarter" idx="13"/>"#,
            r#"<p:ph type="media" sz="quarter" idx="13"/>"#,
            |m| {
                let node = named_mut(m.get_mut(2).unwrap(), "Picture Placeholder 3");
                node.app
                    .as_mut()
                    .unwrap()
                    .placeholder
                    .as_mut()
                    .unwrap()
                    .placeholder_type = Some(PlaceholderType::Media);
            },
        ),
        (
            0,
            "ppt/slideMasters/slideMaster1.xml",
            r#"accent1="accent1" accent2="accent2""#,
            r#"accent1="accent2" accent2="accent2""#,
            |m| {
                let map = m.get_mut(0).unwrap().color_map.as_mut().unwrap();
                map.slots[ColorMapSlot::Accent1 as usize] = Some(ColorSchemeIndex::Accent2);
            },
        ),
        (
            8,
            "ppt/notesMasters/notesMaster1.xml",
            r#"<p:hf dt="0"/>"#,
            r#"<p:hf/>"#,
            |m| {
                m.get_mut(8).unwrap().header_footer = Some(HeaderFooter::default());
            },
        ),
    ];
    let source = read(RICH);
    for (ordinal, part, from, to, change) in cases {
        let edited = edit(&source, &[*ordinal], change);
        assert_only_this_changed(&source, &edited, part, from, to);
    }
}

#[test]
fn a_placeholder_moved_off_its_layout_gains_an_xfrm_in_schema_order() {
    let source = read(RICH);
    let edited = edit(&source, &[4], |m| {
        named_mut(m.get_mut(4).unwrap(), "Title 1").transform = Some(Transform {
            offset: Some(at(100, 200)),
            extent: Some(PositiveSize { cx: 300, cy: 400 }),
            ..Transform::default()
        });
    });
    let slide = String::from_utf8(parts(&edited)["ppt/slides/slide1.xml"].clone()).unwrap();
    assert!(
        slide.contains(r#"<p:spPr><a:xfrm><a:off x="100" y="200"/><a:ext cx="300" cy="400"/></a:xfrm><a:prstGeom"#),
        "{slide}"
    );
    // And taking it away again leaves the bytes as they were.
    let back = edit(&edited, &[4], |m| {
        named_mut(m.get_mut(4).unwrap(), "Title 1").transform = None;
    });
    assert_eq!(
        parts(&back)["ppt/slides/slide1.xml"],
        parts(&source)["ppt/slides/slide1.xml"]
    );
    f4_validates(&edited);
}

#[test]
fn an_unedited_model_returns_the_input_for_every_deck() {
    for set in ["generated", "producers"] {
        for entry in std::fs::read_dir(fixtures().join(set)).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|e| e == "tsv") {
                continue;
            }
            let source = std::fs::read(&path).unwrap();
            let m = model(&source);
            let all: Vec<&SlidePart> = m.parts.iter().flatten().collect();
            let mut pkg = open(source.clone());
            let out = save_slide_parts(
                &mut pkg,
                &m.structure,
                &all,
                &SlideLimits::default(),
                &ImportLimits::default(),
            )
            .unwrap();
            assert_eq!(out.bytes, source, "{}", path.display());
            assert!(out.changed_parts.is_empty());
        }
    }
}

/// Import, edit, export, import: equal, on every deck, then a second save of the
/// same model is byte-identical.
#[test]
fn import_export_import_is_a_fixed_point_on_every_deck() {
    for set in ["generated", "producers"] {
        for entry in std::fs::read_dir(fixtures().join(set)).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|e| e == "tsv") {
                continue;
            }
            let source = std::fs::read(&path).unwrap();
            let first = model(&source).structure.slides[0];
            edit(&source, &[first], |m| {
                let slide = m.get_mut(first).unwrap();
                slide.show = Some(false);
                let tree = slide.shape_tree_mut().unwrap();
                if let Some(nv) = tree.non_visual.as_mut() {
                    nv.descr = Some("edited".to_owned());
                }
            });
        }
    }
}

#[test]
fn what_this_tier_does_not_write_is_refused_and_nothing_is_saved() {
    let source = read(RICH);
    let code = |change: Change| try_edit(&source, &[5], change).unwrap_err();
    assert_eq!(
        code(|m| {
            m.get_mut(5)
                .unwrap()
                .shape_tree_mut()
                .unwrap()
                .children
                .pop();
        }),
        "CS-SLE-0001",
        "removing a node"
    );
    assert_eq!(
        code(
            |m| m.get_mut(5).unwrap().common.as_mut().unwrap().background =
                Some(Default::default())
        ),
        "CS-SLE-0001",
        "a background is Tier 1E"
    );
    assert_eq!(
        code(
            |m| named_mut(m.get_mut(5).unwrap(), "Table 16").graphic_uri = Some("urn:x".to_owned())
        ),
        "CS-SLE-0001"
    );
    assert_eq!(
        code(|m| named_mut(m.get_mut(5).unwrap(), "Title 1")
            .non_visual
            .as_mut()
            .unwrap()
            .hyperlink_click = None),
        "CS-SLE-0001"
    );
    assert_eq!(
        code(|m| {
            named_mut(m.get_mut(5).unwrap(), "Fallback Shape")
                .transform
                .as_mut()
                .unwrap()
                .rotation = Some(1);
        }),
        "CS-SLE-0002",
        "the p14 branch would keep the old transform"
    );
    assert_eq!(
        code(|m| named_mut(m.get_mut(5).unwrap(), "Child B")
            .non_visual
            .as_mut()
            .unwrap()
            .spid = Some(11)),
        "CS-SLE-0003"
    );
    assert_eq!(
        code(|m| named_mut(m.get_mut(5).unwrap(), "Child B")
            .transform
            .as_mut()
            .unwrap()
            .extent = Some(PositiveSize { cx: -1, cy: 0 })),
        "CS-EXP-0002"
    );
    assert_eq!(
        code(
            |m| named_mut(m.get_mut(5).unwrap(), "Child B").locks = Some(Locks {
                no_ungrp: Some(true),
                ..Locks::default()
            })
        ),
        "CS-EXP-0002",
        "a shape has no noUngrp"
    );
    assert_eq!(
        code(|m| named_mut(m.get_mut(5).unwrap(), "Title 1")
            .non_visual
            .as_mut()
            .unwrap()
            .hyperlink_click
            .as_mut()
            .unwrap()
            .rel_id = Some("rId404".to_owned())),
        "CS-EXP-0003"
    );
    assert_eq!(
        code(|m| named_mut(m.get_mut(5).unwrap(), "Child B")
            .non_visual
            .as_mut()
            .unwrap()
            .name = Some("\u{1}".to_owned())),
        "CS-EXP-0001"
    );
}

#[test]
fn strict_decks_are_patched_in_their_own_namespaces() {
    let source = read(STRICT);
    let edited = edit(&source, &[5], |m| {
        let slide = m.get_mut(5).unwrap();
        named_mut(slide, "Child B")
            .transform
            .as_mut()
            .unwrap()
            .offset = Some(at(1_000_000, 0));
        named_mut(slide, "Content Placeholder 2").locks = Some(Locks {
            no_grp: Some(true),
            ..Locks::default()
        });
    });
    let slide = String::from_utf8(parts(&edited)[SLIDE2].clone()).unwrap();
    assert!(slide.contains(r#"<a:spLocks noGrp="1"/>"#));
    assert!(!slide.contains("schemas.openxmlformats.org/drawingml/2006/main"));
    f4_validates(&edited);
}

/// F4 on the edited deck: markup compatibility resolved, then `xmllint` against the
/// vendored schemas. Skipped, and said so, where the tools are not installed.
fn f4_validates(bytes: &[u8]) {
    let have = |tool: &str| Command::new(tool).arg("--version").output().is_ok();
    if !have("xmllint") || !have("python3") {
        eprintln!("xmllint or python3 not installed: F4 skipped");
        return;
    }
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("f4-slides");
    std::fs::create_dir_all(&dir).unwrap();
    let n = std::fs::read_dir(&dir).unwrap().count();
    let file = dir.join(format!("edited-{n}.pptx"));
    std::fs::write(&file, bytes).unwrap();
    let tool = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/f4/validate.py");
    let out = Command::new("python3")
        .arg("-I")
        .arg(tool)
        .arg(&file)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn edited_decks_validate() {
    let source = read(RICH);
    let edited = edit(&source, &[0, 1, 5, 6], |m| {
        m.get_mut(0).unwrap().preserve = Some(false);
        m.get_mut(1).unwrap().header_footer = None;
        named_mut(m.get_mut(5).unwrap(), "Group 9")
            .transform
            .as_mut()
            .unwrap()
            .child_offset = Some(at(5, 5));
        named_mut(m.get_mut(5).unwrap(), "Connector 14").locks = None;
        named_mut(m.get_mut(5).unwrap(), "Table 16")
            .locks
            .as_mut()
            .unwrap()
            .no_select = Some(true);
        m.get_mut(6).unwrap().color_map_override = Some(ColorMapOverride::Master);
        m.get_mut(6).unwrap().show_master_shapes = None;
    });
    f4_validates(&edited);
}
