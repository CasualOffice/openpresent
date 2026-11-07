//! The repair-risk fixtures of `docs/36`, one per invariant the writer implements.
//!
//! **Good:** the deck after the stated edit, through the writer, which F4 must pass.
//! **Seeded:** the same plan with one invariant broken by a hook on the draft; preflight
//! must refuse it, naming that invariant. A rule only a written ZIP can show is seeded in
//! the bytes, and refused by `check_zip`. Given as an input deck, a seeded file must open
//! and save, with the break reported where a rule can see it in one package.
//!
//! The pairs live in `fixtures/rr`, for `PV-01` to open. `RR_WRITE=1` regenerates them; a
//! plain run holds what the writer makes to what is committed, part by part. None of this
//! says anything about PowerPoint until `PV-01` has opened them.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

use casual_slide_export::theme::set_theme_name;
use casual_slide_export::writer::{
    Draft, HostInputs, Op, SavePlan, SlideRef, WriterError, check_zip,
};
use casual_slide_ooxml::{ElementId, OoxmlLimits, PresentationPackage, XmlTree};

const THREE: &str = "generated/three-slides.pptx";
const MAIN: &str = "ppt/presentation.xml";
const MAIN_RELS: &str = "ppt/_rels/presentation.xml.rels";
const TYPES: &str = "[Content_Types].xml";
const SLIDE1_RELS: &str = "ppt/slides/_rels/slide1.xml.rels";
const T: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
const SEC_A: &str = "{11111111-AAAA-4BBB-8CCC-000000000001}";
const SEC_B: &str = "{22222222-AAAA-4BBB-8CCC-000000000002}";
const SEC_NEW: &str = "{33333333-AAAA-4BBB-8CCC-000000000003}";
const PNG: &[u8] = b"\x89PNG\r\n\x1a\nrr-new-image";
const VALID_XML: &[u8] = b"<thing xmlns=\"urn:example:seed\"/>";

type Seed = fn(&mut Draft<'_>);
type ByteSeed = fn(&[u8], &[u8]) -> (Vec<u8>, Vec<u8>, BTreeSet<String>);
type Check = fn(&BTreeMap<String, Vec<u8>>, &[String]);
type Prepare = fn(SavePlan<'static>) -> SavePlan<'static>;

/// One fixture. `seeds` is what must be refused; `arrives` says a rule can see the first
/// seed in a deck on its own, so it must be reported when given as input.
struct Case {
    id: &'static str,
    rule: &'static str,
    base: &'static str,
    plan: Vec<Op>,
    replaced: Vec<(&'static str, Vec<u8>)>,
    seeds: Vec<(&'static str, Seed)>,
    bytes: Option<(&'static str, ByteSeed)>,
    arrives: bool,
    check: Option<Check>,
    prepare: Option<Prepare>,
    no_seed: Option<&'static str>,
}

fn case(id: &'static str, rule: &'static str, base: &'static str, plan: Vec<Op>) -> Case {
    Case {
        id,
        rule,
        base,
        plan,
        replaced: Vec::new(),
        seeds: Vec::new(),
        bytes: None,
        arrives: false,
        check: None,
        prepare: None,
        no_seed: None,
    }
}

impl Case {
    fn seed(mut self, rule: &'static str, seed: Seed) -> Self {
        self.seeds.push((rule, seed));
        self
    }
    fn bytes(mut self, rule: &'static str, seed: ByteSeed) -> Self {
        self.bytes = Some((rule, seed));
        self
    }
    fn reported(mut self) -> Self {
        self.arrives = true;
        self
    }
    fn prepare(mut self, prepare: Prepare) -> Self {
        self.prepare = Some(prepare);
        self
    }
    /// No draft can hold this break; the check on the good deck is the test.
    fn no_seed(mut self, why: &'static str) -> Self {
        self.no_seed = Some(why);
        self
    }
    fn check(mut self, check: Check) -> Self {
        self.check = Some(check);
        self
    }
}

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
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

fn add_slide() -> Op {
    Op::AddSlide {
        position: None,
        layout: None,
    }
}

fn add_slide_at(position: usize) -> Op {
    Op::AddSlide {
        position: Some(position),
        layout: None,
    }
}

fn delete(i: usize) -> Op {
    Op::DeleteSlide {
        slide: SlideRef::Index(i),
    }
}

fn duplicate(i: usize) -> Op {
    Op::DuplicateSlide {
        slide: SlideRef::Index(i),
    }
}

fn picture(bytes: &[u8], extension: &str, content_type: &str) -> Op {
    Op::AddPicture {
        slide: SlideRef::Index(0),
        bytes: bytes.to_vec(),
        extension: extension.to_owned(),
        content_type: content_type.to_owned(),
    }
}

fn notes() -> Op {
    Op::AddNotes {
        slide: SlideRef::Index(0),
        text: "A note".to_owned(),
    }
}

fn elements(tree: &XmlTree) -> Vec<ElementId> {
    let mut out = Vec::new();
    let mut stack = vec![tree.root()];
    while let Some(el) = stack.pop() {
        out.push(el);
        let mut children = tree.child_elements(el);
        children.reverse();
        stack.extend(children);
    }
    out
}

fn find(tree: &XmlTree, local: &str) -> Option<ElementId> {
    elements(tree)
        .into_iter()
        .find(|&el| tree.local_name(el) == local)
}

/// Add a raw relationship, with nothing else following.
fn raw_rel(draft: &mut Draft<'_>, rels_part: &str, id: &str, rel_type: &str, target: &str) {
    draft
        .edit_part(rels_part, |tree| {
            let root = tree.root();
            let new = tree.new_element("Relationship");
            assert!(tree.set_attribute(new, "Id", id));
            assert!(tree.set_attribute(new, "Type", rel_type));
            assert!(tree.set_attribute(new, "Target", target));
            tree.insert_child(root, None, new);
        })
        .unwrap();
}

fn set_rel_target(draft: &mut Draft<'_>, rels_part: &str, kind: &str, target: &str) {
    draft
        .edit_part(rels_part, |tree| {
            for el in tree.child_elements(tree.root()) {
                let is_kind = tree
                    .plain_attribute(el, "Type")
                    .is_some_and(|t| t.ends_with(kind));
                if is_kind {
                    assert!(tree.set_attribute(el, "Target", target));
                    return;
                }
            }
            panic!("no relationship of kind {kind}");
        })
        .unwrap();
}

fn remove_rel_of(draft: &mut Draft<'_>, rels_part: &str, kind: &str) {
    draft
        .edit_part(rels_part, |tree| {
            let root = tree.root();
            for el in tree.child_elements(root) {
                if tree
                    .plain_attribute(el, "Type")
                    .is_some_and(|t| t.ends_with(kind))
                {
                    tree.remove_child(root, el);
                }
            }
        })
        .unwrap();
}

fn remove_override(draft: &mut Draft<'_>, part_name: &str) {
    draft
        .edit_part(TYPES, |tree| {
            let root = tree.root();
            for el in tree.child_elements(root) {
                if tree.plain_attribute(el, "PartName").as_deref() == Some(part_name) {
                    tree.remove_child(root, el);
                }
            }
        })
        .unwrap();
}

fn add_type_entry(draft: &mut Draft<'_>, local: &str, attrs: &[(&str, &str)]) {
    draft
        .edit_part(TYPES, |tree| {
            let root = tree.root();
            let new = tree.new_element(local);
            for (k, v) in attrs {
                assert!(tree.set_attribute(new, k, v));
            }
            tree.insert_child(root, None, new);
        })
        .unwrap();
}

fn remove_elements(draft: &mut Draft<'_>, part: &str, local: &str) {
    draft
        .edit_part(part, |tree| {
            for el in elements(tree) {
                if tree.local_name(el) == local {
                    let parent = tree.parent_element(el).unwrap();
                    tree.remove_child(parent, el);
                }
            }
        })
        .unwrap();
}

/// Set an attribute on the first `local` element.
fn set_first(draft: &mut Draft<'_>, part: &str, local: &str, attr: &str, value: &str) {
    draft
        .edit_part(part, |tree| {
            let el = find(tree, local).unwrap();
            assert!(tree.set_attribute(el, attr, value));
        })
        .unwrap();
}

/// Append a `sldId` to the `which`-th section.
fn add_to_section(draft: &mut Draft<'_>, which: usize, id: &str) {
    draft
        .edit_part(MAIN, |tree| {
            let sections: Vec<ElementId> = elements(tree)
                .into_iter()
                .filter(|&el| tree.local_name(el) == "section")
                .collect();
            let list = tree.child(sections[which], "sldIdLst").unwrap();
            let new = tree.new_element("p14:sldId");
            assert!(tree.set_attribute(new, "id", id));
            tree.insert_child(list, None, new);
        })
        .unwrap();
}

/// A section list with no section in it.
fn empty_section_list(d: &mut Draft<'_>) {
    d.edit_part(MAIN, |tree| {
        let root = tree.root();
        let ext_list = tree.new_element("p:extLst");
        let ext = tree.new_element("p:ext");
        assert!(tree.set_attribute(ext, "uri", "{521415D9-36F7-43E2-AB2F-B90AF26B5E84}"));
        let list = tree.new_element("p14:sectionLst");
        assert!(tree.set_attribute(
            list,
            "xmlns:p14",
            "http://schemas.microsoft.com/office/powerpoint/2010/main"
        ));
        tree.insert_child(ext, None, list);
        tree.insert_child(ext_list, None, ext);
        tree.insert_child(root, None, ext_list);
    })
    .unwrap();
}

/// A ZIP of stored entries, in order. All a byte-level seed needs.
fn stored_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
    fn crc32(bytes: &[u8]) -> u32 {
        let mut crc = !0u32;
        for b in bytes {
            crc ^= u32::from(*b);
            for _ in 0..8 {
                crc = if crc & 1 == 1 {
                    (crc >> 1) ^ 0xEDB8_8320
                } else {
                    crc >> 1
                };
            }
        }
        !crc
    }
    let mut out = Vec::new();
    let mut central = Vec::new();
    for (name, data) in entries {
        let offset = out.len() as u32;
        let (crc, len) = (crc32(data), data.len() as u32);
        let common = |buf: &mut Vec<u8>| {
            buf.extend(20u16.to_le_bytes());
            buf.extend(0u16.to_le_bytes());
            buf.extend(0u16.to_le_bytes());
            buf.extend(0u16.to_le_bytes());
            buf.extend(33u16.to_le_bytes());
            buf.extend(crc.to_le_bytes());
            buf.extend(len.to_le_bytes());
            buf.extend(len.to_le_bytes());
            buf.extend((name.len() as u16).to_le_bytes());
            buf.extend(0u16.to_le_bytes());
        };
        out.extend(0x0403_4B50u32.to_le_bytes());
        common(&mut out);
        out.extend(name.as_bytes());
        out.extend(*data);
        central.extend(0x0201_4B50u32.to_le_bytes());
        central.extend(20u16.to_le_bytes());
        common(&mut central);
        central.extend([0u8; 10]);
        central.extend(offset.to_le_bytes());
        central.extend(name.as_bytes());
    }
    let at = out.len() as u32;
    let size = central.len() as u32;
    out.extend(central);
    out.extend(0x0605_4B50u32.to_le_bytes());
    out.extend([0u8; 4]);
    out.extend((entries.len() as u16).to_le_bytes());
    out.extend((entries.len() as u16).to_le_bytes());
    out.extend(size.to_le_bytes());
    out.extend(at.to_le_bytes());
    out.extend(0u16.to_le_bytes());
    out
}

fn stored_copy(bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
    let mut pkg = open(bytes);
    pkg.part_names()
        .into_iter()
        .map(|n| {
            let b = pkg.read_part(&n).unwrap();
            (n, b)
        })
        .collect()
}

fn as_refs(entries: &[(String, Vec<u8>)]) -> Vec<(&str, &[u8])> {
    entries
        .iter()
        .map(|(n, b)| (n.as_str(), b.as_slice()))
        .collect()
}

fn cases() -> Vec<Case> {
    let mut all = Vec::new();

    // Part naming.
    all.push(
        case("RR-PN-01", "INV-PN-01", THREE, vec![add_slide()])
            .seed("INV-PN-01", |d| {
                d.set_part_bytes("ppt/custom/a%2Fb.xml", VALID_XML.to_vec());
            })
            .seed("INV-PN-01", |d| {
                d.set_part_bytes("ppt/custom/thing.", b"x".to_vec());
            })
            .reported(),
    );
    all.push(
        case(
            "RR-PN-02",
            "INV-PN-02",
            THREE,
            vec![picture(PNG, "png", "image/png")],
        )
        .check(|parts, _| {
            assert!(parts.contains_key("ppt/media/image1.png"));
        })
        .seed("INV-PN-02", |d| {
            d.set_part_bytes("ppt/media/é.png", b"x".to_vec());
        })
        .reported(),
    );
    all.push(
        case(
            "RR-PN-03",
            "INV-PN-03",
            "generated/case-differs.pptx",
            vec![add_slide()],
        )
        .check(|parts, _| {
            assert!(parts.contains_key("ppt/Slides/Slide1.xml"));
            assert!(parts.contains_key("ppt/slides/slide2.xml"));
        })
        .bytes("INV-PN-03", |input, _| {
            let mut entries = stored_copy(input);
            entries.push(("ppt/slides/slide1.xml".to_owned(), b"x".to_vec()));
            let zip = stored_zip(&as_refs(&entries));
            (
                stored_zip(&as_refs(&stored_copy(input))),
                zip,
                BTreeSet::new(),
            )
        })
        .reported(),
    );
    all.push(
        case("RR-PN-04", "INV-PN-04", THREE, vec![add_slide()]).seed("INV-PN-04", |d| {
            d.set_part_bytes("ppt/slides/slide1.xml/x", b"x".to_vec());
        }),
    );
    all.push(
        case("RR-PN-05", "INV-PN-05", THREE, vec![delete(1), add_slide()])
            .check(|parts, new| {
                assert_eq!(new, ["ppt/slides/slide4.xml"]);
                assert!(!parts.contains_key("ppt/slides/slide2.xml"));
            })
            .seed("INV-PN-05", |d| {
                let body = d.part_bytes("ppt/slides/slide1.xml").unwrap();
                d.set_part_bytes("ppt/slides/slide2.xml", body);
            }),
    );
    all.push(
        case("RR-PN-06", "INV-PN-06", THREE, vec![delete(1)])
            .check(|parts, _| {
                assert!(parts.contains_key("ppt/slides/slide1.xml"));
                assert!(!parts.contains_key("ppt/slides/slide2.xml"));
                assert!(parts.contains_key("ppt/slides/slide3.xml"), "the gap stays");
            })
            .seed("INV-PN-06", |d| {
                let old = d.part_bytes("ppt/slides/slide3.xml").unwrap();
                d.drop_entry("ppt/slides/slide3.xml");
                d.set_part_bytes("ppt/slides/slide2.xml", old);
            }),
    );
    all.push(
        case("RR-PN-07", "INV-PN-07", THREE, vec![delete(2)])
            .seed("INV-PN-07", |d| {
                d.set_part_bytes(
                    "ppt/slides/_rels/slide3.xml.rels",
                    b"<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"/>"
                        .to_vec(),
                );
            })
            .reported(),
    );
    all.push(
        case(
            "RR-PN-08",
            "INV-PN-08",
            "generated/percent-encoded-names.pptx",
            vec![add_slide()],
        )
        .check(|parts, _| {
            assert!(parts.contains_key("ppt/media/%C3%A9.png"), "kept as it is");
        })
        .bytes("INV-PN-08", |input, _| {
            let mut entries = stored_copy(input);
            entries.push(("ppt/media/ñ.png".to_owned(), b"x".to_vec()));
            let zip = stored_zip(&as_refs(&entries));
            (
                stored_zip(&as_refs(&stored_copy(input))),
                zip,
                BTreeSet::new(),
            )
        })
        .reported(),
    );
    all.push(
        case(
            "RR-PN-09",
            "INV-PN-09",
            "generated/stored-not-deflated.pptx",
            vec![add_slide()],
        )
        .bytes("INV-PN-09", |input, _| {
            let mut entries = stored_copy(input);
            entries.push(("ppt/slides/slide9.xml".to_owned(), b"x".to_vec()));
            let zip = stored_zip(&as_refs(&entries));
            let changed = ["ppt/slides/slide9.xml".to_owned()].into_iter().collect();
            (stored_zip(&as_refs(&stored_copy(input))), zip, changed)
        })
        .reported(),
    );
    all.push(
        case("RR-PN-10", "INV-PN-10", THREE, vec![add_slide()])
            .bytes("INV-PN-10", |input, output| {
                let mut bytes = output.to_vec();
                // The first local header's compressed size.
                bytes[18] = bytes[18].wrapping_add(1);
                (input.to_vec(), bytes, BTreeSet::new())
            })
            .reported(),
    );
    all.push(
        case("RR-PN-11", "INV-PN-11", THREE, vec![add_slide()])
            .bytes("INV-PN-11", |input, _| {
                let entries = stored_copy(input);
                let stored = stored_zip(&as_refs(&entries));
                let mut moved = entries.clone();
                let at = moved.iter().position(|(n, _)| n == TYPES).unwrap();
                let types = moved.remove(at);
                moved.push(types);
                (stored, stored_zip(&as_refs(&moved)), BTreeSet::new())
            })
            .reported(),
    );

    // Relationship ids.
    all.push(
        case(
            "RR-RI-01",
            "INV-RI-01",
            THREE,
            vec![picture(PNG, "png", "image/png")],
        )
        .seed("INV-RI-01", |d| {
            raw_rel(d, SLIDE1_RELS, "1a", &format!("{T}/hyperlink"), "#s");
        })
        .seed("INV-RI-01", |d| {
            raw_rel(d, SLIDE1_RELS, "rId1", &format!("{T}/tags"), "../x.xml");
        })
        .reported(),
    );
    all.push(
        case(
            "RR-RI-02",
            "INV-RI-02",
            "generated/rr-rich-clone.pptx",
            vec![picture(PNG, "png", "image/png")],
        )
        .check(|parts, _| {
            let rels = text(parts, SLIDE1_RELS);
            assert!(rels.contains("Id=\"rId7\""), "no id moved: {rels}");
        })
        .seed("INV-RI-02", |d| {
            d.edit_part(SLIDE1_RELS, |tree| {
                for el in tree.child_elements(tree.root()) {
                    if tree.plain_attribute(el, "Id").as_deref() == Some("rId7") {
                        assert!(tree.set_attribute(el, "Id", "rId80"));
                    }
                }
            })
            .unwrap();
        }),
    );
    all.push(
        case(
            "RR-RI-03",
            "INV-RI-03",
            "generated/rr-unknown-rel.pptx",
            vec![picture(PNG, "png", "image/png")],
        )
        .check(|parts, _| {
            assert!(
                text(parts, SLIDE1_RELS).contains("Id=\"rId8\""),
                "one above rId7, past foo"
            );
        })
        .seed("INV-RI-03", |d| {
            raw_rel(d, SLIDE1_RELS, "rId3", &format!("{T}/hyperlink"), "#s");
        }),
    );
    all.push(
        case(
            "RR-RI-04",
            "INV-RI-04",
            "generated/rr-shared-media.pptx",
            vec![Op::RemovePicture {
                slide: SlideRef::Index(0),
                shape_id: 4,
            }],
        )
        .check(|parts, _| {
            assert!(!text(parts, "ppt/slides/slide1.xml").contains("r:embed=\"rId2\""));
            assert!(!text(parts, SLIDE1_RELS).contains("image1"));
            assert!(parts.contains_key("ppt/media/image1.png"), "slide 2 has it");
        })
        .seed("INV-RI-04", |d| {
            d.edit_part("ppt/slides/slide1.xml", |tree| {
                let sp = find(tree, "sp").unwrap();
                assert!(tree.set_attribute(sp, "r:dangling", "rId99"));
            })
            .unwrap();
        }),
    );
    all.push(
        case(
            "RR-RI-05",
            "INV-RI-05",
            "generated/rr-unknown-rel.pptx",
            vec![add_slide()],
        )
        .check(|parts, _| {
            assert!(text(parts, SLIDE1_RELS).contains("relationships/retained"));
            assert!(parts.contains_key("ppt/custom/thing1.xml"));
        })
        .seed("INV-RI-05", |d| {
            remove_rel_of(d, SLIDE1_RELS, "retained");
        }),
    );
    all.push(
        case("RR-RI-06", "INV-RI-06", THREE, vec![add_slide()])
            .seed("INV-RI-06", |d| {
                d.edit_part(MAIN_RELS, |tree| {
                    let root = tree.root();
                    assert!(tree.set_attribute(root, "xml:base", "/x/"));
                })
                .unwrap();
            })
            .reported(),
    );
    all.push(
        case(
            "RR-RI-07",
            "INV-RI-07",
            "generated/rr-absolute-targets.pptx",
            vec![add_slide()],
        )
        .check(|parts, new| {
            let rels = text(parts, MAIN_RELS);
            let wanted = format!("Target=\"/{}\"", new[0]);
            assert!(rels.contains(&wanted), "{rels}");
        })
        .seed("INV-RI-07", |d| {
            set_rel_target(
                d,
                MAIN_RELS,
                "/slideMaster",
                "slideMasters/slideMaster1.xml",
            );
        }),
    );
    all.push(
        case(
            "RR-RI-08",
            "INV-RI-08",
            THREE,
            vec![Op::AddLayout {
                master: "ppt/slideMasters/slideMaster1.xml".to_owned(),
                copy_of: "ppt/slideLayouts/slideLayout1.xml".to_owned(),
            }],
        )
        .seed("INV-RI-08", |d| {
            raw_rel(
                d,
                SLIDE1_RELS,
                "rId5",
                "http://schemas.openxmlformats.org/package/2006/relationships/metadata/thumbnail",
                "../theme/theme1.xml",
            );
        })
        .reported(),
    );
    all.push(
        case(
            "RR-RI-09",
            "INV-RI-09",
            "generated/slides-strict.pptx",
            vec![add_slide()],
        )
        .check(|parts, _| {
            let rels = text(parts, MAIN_RELS);
            assert!(rels.contains("purl.oclc.org"), "{rels}");
            assert!(!rels.contains("schemas.openxmlformats.org/officeDocument"));
        })
        .seed("INV-RI-09", |d| {
            raw_rel(
                d,
                MAIN_RELS,
                "rId99",
                &format!("{T}/tags"),
                "slides/slide1.xml",
            );
        }),
    );
    all.push(
        case(
            "RR-RI-10",
            "INV-RI-10",
            THREE,
            vec![
                picture(PNG, "png", "image/png"),
                picture(PNG, "png", "image/png"),
            ],
        )
        .check(|parts, _| {
            assert_eq!(text(parts, SLIDE1_RELS).matches("image1.png").count(), 1);
            assert_eq!(
                text(parts, "ppt/slides/slide1.xml")
                    .matches("<p:pic>")
                    .count(),
                2,
                "both pictures, one relationship"
            );
        })
        .seed("INV-RI-10", |d| {
            raw_rel(
                d,
                SLIDE1_RELS,
                "rId9",
                &format!("{T}/image"),
                "../media/image1.png",
            );
        }),
    );

    // Content types.
    all.push(
        case(
            "RR-CT-01",
            "INV-CT-01",
            THREE,
            vec![picture(PNG, "webp", "image/webp")],
        )
        .check(|parts, _| {
            assert!(text(parts, TYPES).contains("PartName=\"/ppt/media/image1.webp\""));
        })
        .seed("INV-CT-01", |d| {
            remove_override(d, "/ppt/media/image1.webp");
        })
        .reported(),
    );
    all.push(
        case("RR-CT-02", "INV-CT-02", THREE, vec![add_slide()]).seed("INV-CT-02", |d| {
            add_type_entry(
                d,
                "Default",
                &[("Extension", "PNG"), ("ContentType", "image/png")],
            );
        }),
    );
    all.push(
        case(
            "RR-CT-03",
            "INV-CT-03",
            "generated/rr-conflicting-default.pptx",
            vec![
                picture(PNG, "png", "image/png"),
                Op::AttachPart {
                    source: "ppt/slides/slide1.xml".to_owned(),
                    kind: "http://example.com/relationships/retained".to_owned(),
                    dir: "ppt/custom".to_owned(),
                    stem: "blob".to_owned(),
                    extension: String::new(),
                    content_type: "application/x-example".to_owned(),
                    bytes: b"x".to_vec(),
                },
            ],
        )
        .check(|parts, _| {
            let types = text(parts, TYPES);
            assert!(types.contains("PartName=\"/ppt/media/image1.png\" ContentType=\"image/png\""));
            assert!(types.contains("PartName=\"/ppt/custom/blob1\""));
            assert!(types.contains("Extension=\"png\" ContentType=\"application/x-other\""));
        })
        .seed("INV-CT-03", |d| {
            remove_override(d, "/ppt/media/image1.png");
        }),
    );
    all.push(
        case(
            "RR-CT-04",
            "INV-CT-04",
            THREE,
            vec![add_slide(), picture(PNG, "png", "image/png")],
        )
        .check(|parts, new| {
            let types = text(parts, TYPES);
            assert!(types.contains(&format!("PartName=\"/{}\"", new[0])));
            assert!(!types.contains("PartName=\"/ppt/media/image1.png\""));
        })
        .seed("INV-CT-04", |d| {
            remove_override(d, "/ppt/slides/slide4.xml");
            d.edit_part(TYPES, |tree| {
                for el in tree.child_elements(tree.root()) {
                    if tree.plain_attribute(el, "Extension").as_deref() == Some("xml") {
                        assert!(tree.set_attribute(
                            el,
                            "ContentType",
                            "application/vnd.openxmlformats-officedocument.presentationml.slide+xml"
                        ));
                    }
                }
            })
            .unwrap();
        }),
    );
    all.push(
        case("RR-CT-05", "INV-CT-05", THREE, vec![delete(2)])
            .check(|parts, _| {
                assert!(!text(parts, TYPES).contains("slide3.xml"));
            })
            .seed("INV-CT-05", |d| {
                add_type_entry(
                    d,
                    "Override",
                    &[
                        ("PartName", "/ppt/slides/slide3.xml"),
                        (
                            "ContentType",
                            "application/vnd.openxmlformats-officedocument.presentationml.slide+xml",
                        ),
                    ],
                );
            })
            .reported(),
    );
    all.push(
        case(
            "RR-CT-06",
            "INV-CT-06",
            "generated/rr-vba.pptm",
            vec![
                Op::DropVba,
                Op::Retype {
                    extension: "pptx".to_owned(),
                },
            ],
        )
        .check(|parts, _| {
            assert!(!parts.contains_key("ppt/vbaProject.bin"));
            let types = text(parts, TYPES);
            assert!(!types.contains("vbaProject"), "{types}");
            assert!(
                types.contains("Extension=\"png\""),
                "an unused Default stays"
            );
            assert!(!types.contains("macroEnabled"));
        })
        .seed("INV-CT-06", |d| {
            add_type_entry(
                d,
                "Default",
                &[
                    ("Extension", "bin"),
                    ("ContentType", "application/vnd.ms-office.vbaProject"),
                ],
            );
        })
        .seed("INV-CT-06", |d| {
            d.edit_part(TYPES, |tree| {
                let root = tree.root();
                for el in tree.child_elements(root) {
                    if tree.plain_attribute(el, "Extension").as_deref() == Some("png") {
                        tree.remove_child(root, el);
                    }
                }
            })
            .unwrap();
        }),
    );
    all.push(
        case("RR-CT-07", "INV-CT-07", THREE, vec![add_slide()])
            .seed("INV-CT-07", |d| {
                d.edit_part(TYPES, |tree| {
                    let root = tree.root();
                    assert!(tree.set_attribute(root, "version", "1"));
                })
                .unwrap();
            })
            .reported(),
    );
    all.push(
        case(
            "RR-CT-08",
            "INV-CT-08",
            THREE,
            vec![Op::Retype {
                extension: "potx".to_owned(),
            }],
        )
        .check(|parts, _| {
            assert!(text(parts, TYPES).contains("template.main+xml"));
        })
        .seed("INV-CT-08", |d| {
            d.edit_part(TYPES, |tree| {
                for el in tree.child_elements(tree.root()) {
                    if tree.plain_attribute(el, "PartName").as_deref()
                        == Some("/ppt/slides/slide1.xml")
                    {
                        assert!(tree.set_attribute(el, "ContentType", "application/x-other+xml"));
                    }
                }
            })
            .unwrap();
        }),
    );
    all.push(
        case(
            "RR-CT-10",
            "INV-CT-10",
            THREE,
            vec![Op::Retype { extension: "ppsx".to_owned() }],
        )
        .check(|parts, _| {
            assert!(text(parts, TYPES).contains("slideshow.main+xml"));
        })
        .seed("INV-CT-10", |d| {
            d.edit_part(TYPES, |tree| {
                for el in tree.child_elements(tree.root()) {
                    if tree.plain_attribute(el, "PartName").as_deref() == Some("/ppt/presentation.xml") {
                        assert!(tree.set_attribute(
                            el,
                            "ContentType",
                            "application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"
                        ));
                    }
                }
            })
            .unwrap();
        }),
    );
    all.push(
        case("RR-CT-09", "INV-CT-09", THREE, vec![add_slide()]).seed("INV-CT-09", |d| {
            d.edit_part(TYPES, |tree| {
                let root = tree.root();
                for el in tree.child_elements(root) {
                    if tree.plain_attribute(el, "Extension").as_deref() == Some("rels") {
                        tree.remove_child(root, el);
                    }
                }
            })
            .unwrap();
        }),
    );

    // Shared parts.
    all.push(
        case(
            "RR-SP-01",
            "INV-SP-01",
            "generated/rr-orphan.pptx",
            vec![add_slide()],
        )
        .check(|parts, _| {
            assert!(parts.contains_key("ppt/custom/orphan1.xml"));
        })
        .seed("INV-SP-01", |d| {
            d.remove_part("ppt/custom/orphan1.xml").unwrap();
        }),
    );
    all.push(
        case(
            "RR-SP-02",
            "INV-SP-02",
            "generated/rr-shared-media.pptx",
            vec![delete(0)],
        )
        .check(|parts, _| {
            assert!(parts.contains_key("ppt/media/image1.png"), "slide 2 has it");
            assert!(
                parts.contains_key("ppt/media/image2.png"),
                "the theme has it"
            );
        })
        .seed("INV-SP-02", |d| {
            d.remove_part("ppt/media/image1.png").unwrap();
        }),
    );
    all.push(
        case(
            "RR-SP-03",
            "INV-SP-03",
            "generated/rr-shared-media.pptx",
            vec![{
                let png = std::fs::read(fixtures().join("generated/image.pptx")).unwrap();
                let image = parts_of(&png)["ppt/media/image1.png"].clone();
                picture(&image, "png", "image/png")
            }],
        )
        .check(|parts, _| {
            assert_eq!(
                parts.keys().filter(|k| k.starts_with("ppt/media/")).count(),
                2,
                "no new part"
            );
        })
        .seed("INV-SP-03", |d| {
            let same = d.part_bytes("ppt/media/image1.png").unwrap();
            d.set_part_bytes("ppt/media/image9.png", same);
        }),
    );
    all.push(
        case(
            "RR-SP-04",
            "INV-SP-04",
            "generated/rr-shared-media.pptx",
            vec![Op::ReplaceMedia {
                slide: SlideRef::Index(0),
                rel_id: "rId2".to_owned(),
                bytes: PNG.to_vec(),
                extension: "png".to_owned(),
                content_type: "image/png".to_owned(),
            }],
        )
        .check(|parts, _| {
            assert!(
                parts.contains_key("ppt/media/image1.png"),
                "slide 2 keeps it"
            );
            assert!(parts.contains_key("ppt/media/image3.png"));
            assert!(text(parts, SLIDE1_RELS).contains("image3.png"));
            assert!(
                text(parts, "ppt/slides/_rels/slide2.xml.rels").contains("image1.png"),
                "the edit does not leak"
            );
        })
        .seed("INV-SP-04", |d| {
            d.set_part_bytes("ppt/media/image1.png", b"changed".to_vec());
        }),
    );
    all.push(
        case("RR-SP-05", "INV-SP-05", THREE, vec![add_slide()])
            .seed("INV-SP-05", |d| {
                remove_rel_of(d, "ppt/slides/_rels/slide4.xml.rels", "/slideLayout");
            })
            .reported(),
    );
    all.push(
        case(
            "RR-SP-06",
            "INV-SP-06",
            "generated/rr-two-masters.pptx",
            vec![Op::RemoveLayout {
                layout: "ppt/slideLayouts/slideLayout3.xml".to_owned(),
            }],
        )
        .check(|parts, _| {
            assert!(!parts.contains_key("ppt/slideLayouts/slideLayout3.xml"));
            assert!(
                !text(parts, "ppt/slideMasters/slideMaster1.xml").contains("2147483650"),
                "off the list"
            );
        })
        .seed("INV-SP-06", |d| {
            remove_elements(d, "ppt/slideMasters/slideMaster1.xml", "sldLayoutId");
        })
        .seed("INV-SP-06", |d| {
            remove_elements(d, MAIN, "sldMasterId");
        })
        .seed("INV-SP-06", |d| {
            remove_elements(d, "ppt/slideMasters/slideMaster1.xml", "sldLayoutId");
            remove_elements(d, "ppt/slideMasters/slideMaster2.xml", "sldLayoutId");
        })
        .reported(),
    );
    all.push(
        case(
            "RR-SP-07",
            "INV-SP-07",
            THREE,
            vec![Op::AddMaster {
                copy_of: "ppt/slideMasters/slideMaster1.xml".to_owned(),
            }],
        )
        .check(|parts, _| {
            assert!(parts.contains_key("ppt/slideMasters/slideMaster2.xml"));
            assert!(
                parts.contains_key("ppt/theme/theme2.xml"),
                "a theme of its own"
            );
            assert!(parts.contains_key("ppt/slideLayouts/slideLayout2.xml"));
        })
        .seed("INV-SP-07", |d| {
            set_rel_target(
                d,
                "ppt/slideMasters/_rels/slideMaster2.xml.rels",
                "/theme",
                "../theme/theme1.xml",
            );
        })
        .reported(),
    );
    all.push(
        case(
            "RR-SP-08",
            "INV-SP-08",
            "generated/rr-two-masters.pptx",
            vec![Op::RemoveMaster {
                master: "ppt/slideMasters/slideMaster1.xml".to_owned(),
            }],
        )
        .check(|parts, _| {
            assert!(!parts.contains_key("ppt/slideMasters/slideMaster1.xml"));
            assert!(!parts.contains_key("ppt/theme/theme1.xml"));
            assert!(
                text(parts, MAIN_RELS).contains("theme/theme2.xml"),
                "it follows"
            );
        })
        .seed("INV-SP-08", |d| {
            set_rel_target(d, MAIN_RELS, "/theme", "theme/theme1.xml");
        })
        .reported(),
    );
    all.push(
        case(
            "RR-SP-09",
            "INV-SP-09",
            "generated/rr-notes.pptx",
            vec![delete(0)],
        )
        .check(|parts, _| {
            assert!(!parts.contains_key("ppt/notesSlides/notesSlide1.xml"));
            assert!(!parts.contains_key("ppt/comments/comment1.xml"));
            assert!(parts.contains_key("ppt/commentAuthors.xml"));
            assert!(parts.contains_key("ppt/notesMasters/notesMaster1.xml"));
        })
        .seed("INV-SP-09", |d| {
            d.set_part_bytes("ppt/notesSlides/notesSlide1.xml", VALID_XML.to_vec());
        }),
    );
    all.push(
        case(
            "RR-SP-10",
            "INV-SP-10",
            "generated/rr-rich-clone.pptx",
            vec![duplicate(0)],
        )
        .check(|parts, new| {
            assert_eq!(new, ["ppt/slides/slide3.xml"]);
            assert_eq!(
                parts["ppt/slides/slide3.xml"], parts["ppt/slides/slide1.xml"],
                "a byte copy, so all it holds comes along"
            );
            for part in [
                "ppt/charts/chart2.xml",
                "ppt/embeddings/Workbook2.xlsx",
                "ppt/embeddings/oleObject2.bin",
                "ppt/diagrams/data2.xml",
                "ppt/diagrams/drawing2.xml",
                "ppt/custom/thing2.xml",
                "ppt/ink/ink2.xml",
            ] {
                assert!(parts.contains_key(part), "{part}");
            }
            let rels = text(parts, "ppt/slides/_rels/slide3.xml.rels");
            assert!(rels.contains("../charts/chart2.xml"));
            assert!(rels.contains("../slideLayouts/slideLayout1.xml"), "shared");
            assert!(rels.contains("Id=\"rId7\""), "the id relId names");
            assert!(rels.contains("../diagrams/drawing2.xml"));
            assert!(
                text(parts, "ppt/diagrams/data2.xml").contains("relId=\"rId7\""),
                "it still resolves, to the clone's drawing"
            );
            let chart = text(parts, "ppt/charts/_rels/chart2.xml.rels");
            assert!(chart.contains("Workbook2.xlsx"));
        })
        .seed("INV-SP-10", |d| {
            set_rel_target(
                d,
                "ppt/slides/_rels/slide3.xml.rels",
                "/chart",
                "../charts/chart1.xml",
            );
        })
        .seed("INV-SP-10", |d| {
            set_rel_target(
                d,
                "ppt/slides/_rels/slide3.xml.rels",
                "/diagramDrawing",
                "../diagrams/drawing1.xml",
            );
        }),
    );

    // Ids.
    all.push(
        case("RR-ID-01", "INV-ID-01", THREE, vec![add_slide()])
            .seed("INV-ID-01", |d| set_first(d, MAIN, "sldId", "id", "255"))
            .seed("INV-ID-01", |d| {
                set_first(d, MAIN, "sldId", "id", "2147483648")
            })
            .seed("INV-ID-01", |d| set_first(d, MAIN, "sldId", "id", "257"))
            .reported(),
    );
    all.push(
        case(
            "RR-ID-02",
            "INV-ID-02",
            "generated/rr-high-ids.pptx",
            vec![add_slide()],
        )
        .check(|parts, _| {
            assert!(text(parts, MAIN).contains("id=\"257\""), "the lowest free");
        })
        .seed("INV-ID-02", |d| set_first(d, MAIN, "sldId", "id", "999")),
    );
    all.push(
        case(
            "RR-ID-03",
            "INV-ID-03",
            THREE,
            vec![Op::AddLayout {
                master: "ppt/slideMasters/slideMaster1.xml".to_owned(),
                copy_of: "ppt/slideLayouts/slideLayout1.xml".to_owned(),
            }],
        )
        .check(|parts, _| {
            assert!(text(parts, "ppt/slideMasters/slideMaster1.xml").contains("2147483650"));
        })
        .seed("INV-ID-03", |d| {
            set_first(
                d,
                "ppt/slideMasters/slideMaster1.xml",
                "sldLayoutId",
                "id",
                "2147483648",
            );
        })
        .reported(),
    );
    all.push(
        case(
            "RR-ID-04",
            "INV-ID-04",
            "generated/rr-high-ids.pptx",
            vec![Op::AddLayout {
                master: "ppt/slideMasters/slideMaster1.xml".to_owned(),
                copy_of: "ppt/slideLayouts/slideLayout1.xml".to_owned(),
            }],
        )
        .check(|parts, _| {
            assert!(
                text(parts, "ppt/slideMasters/slideMaster1.xml").contains("2147483648"),
                "the lowest free, at the cap"
            );
        })
        .seed("INV-ID-04", |d| {
            set_first(
                d,
                "ppt/slideMasters/slideMaster1.xml",
                "sldLayoutId",
                "id",
                "4294967296",
            );
        }),
    );
    all.push(
        case(
            "RR-ID-05",
            "INV-ID-05",
            "generated/rr-sections.pptx",
            vec![
                delete(1),
                Op::AddCustomShow {
                    name: "Added".to_owned(),
                    slides: vec![SlideRef::Index(0)],
                },
            ],
        )
        .check(|parts, _| {
            let main = text(parts, MAIN);
            assert!(main.contains("<p:sldLst></p:sldLst>"), "{main}");
            assert!(!main.contains("id=\"257\""));
            assert!(
                main.contains("name=\"Added\" id=\"1\""),
                "consecutive: {main}"
            );
        })
        .seed("INV-ID-05", |d| {
            set_first(d, "ppt/presProps.xml", "custShow", "id", "99");
        })
        .seed("INV-ID-05", |d| {
            d.edit_part(MAIN, |tree| {
                let shows: Vec<ElementId> = elements(tree)
                    .into_iter()
                    .filter(|&el| tree.local_name(el) == "custShow")
                    .collect();
                assert!(tree.set_attribute(*shows.last().unwrap(), "id", "50"));
            })
            .unwrap();
        })
        .reported(),
    );
    all.push(
        case(
            "RR-ID-06",
            "INV-ID-06",
            "generated/rr-animated.pptx",
            vec![
                Op::CopyShape {
                    slide: SlideRef::Index(0),
                    shape_id: 4,
                    creation_id: Some("{BBBBBBBB-0000-4000-8000-000000000002}".to_owned()),
                },
                duplicate(0),
            ],
        )
        .check(|parts, _| {
            let slide = text(parts, "ppt/slides/slide1.xml");
            assert!(
                slide.contains("<p:cNvPr id=\"5\" name=\"Animated\""),
                "{slide}"
            );
            assert!(slide.contains("spid=\"5\""), "the copy is animated");
            assert!(slide.contains("spid=\"4\""), "and so is the original");
            assert!(slide.contains("BBBBBBBB"), "a new creationId");
            assert_eq!(slide.matches("AAAAAAAA").count(), 1);
            assert_eq!(
                parts["ppt/slides/slide2.xml"],
                parts["ppt/slides/slide1.xml"]
            );
        })
        .seed("INV-ID-06", |d| {
            set_first(d, "ppt/slides/slide1.xml", "spTgt", "spid", "9999");
        })
        .seed("INV-ID-06", |d| {
            // The copy's animation left naming the original.
            d.edit_part("ppt/slides/slide1.xml", |tree| {
                for el in elements(tree) {
                    if tree.local_name(el) == "spTgt"
                        && tree.plain_attribute(el, "spid").as_deref() == Some("5")
                    {
                        assert!(tree.set_attribute(el, "spid", "4"));
                    }
                }
            })
            .unwrap();
        })
        .seed("INV-ID-06", |d| {
            d.edit_part("ppt/slides/slide1.xml", |tree| {
                let ids: Vec<ElementId> = elements(tree)
                    .into_iter()
                    .filter(|&el| tree.local_name(el) == "cNvPr")
                    .collect();
                assert!(tree.set_attribute(ids[2], "id", "2"));
            })
            .unwrap();
        })
        .seed("INV-ID-06", |d| {
            d.edit_part("ppt/slides/slide1.xml", |tree| {
                for el in elements(tree) {
                    if tree.local_name(el) == "creationId"
                        && tree
                            .plain_attribute(el, "id")
                            .is_some_and(|v| v.starts_with("{BB"))
                    {
                        assert!(tree.set_attribute(
                            el,
                            "id",
                            "{AAAAAAAA-0000-4000-8000-000000000001}"
                        ));
                    }
                }
            })
            .unwrap();
        }),
    );
    all.push(
        case(
            "RR-ID-07",
            "INV-ID-07",
            "generated/rr-notes.pptx",
            vec![Op::AddCommentAuthor {
                name: "B".to_owned(),
                initials: "B".to_owned(),
            }],
        )
        .check(|parts, _| {
            assert!(text(parts, "ppt/commentAuthors.xml").contains("id=\"1\""));
        })
        .seed("INV-ID-07", |d| {
            d.edit_part("ppt/commentAuthors.xml", |tree| {
                let root = tree.root();
                let last = *tree.child_elements(root).last().unwrap();
                assert!(tree.set_attribute(last, "id", "0"));
            })
            .unwrap();
        })
        .reported(),
    );
    all.push(
        case("RR-ID-08", "INV-ID-08", THREE, vec![notes()])
            .check(|parts, _| {
                let main = text(parts, MAIN);
                let masters = main.find("<p:sldMasterIdLst").unwrap();
                let notes = main.find("<p:notesMasterIdLst").unwrap();
                let slides = main.find("<p:sldIdLst").unwrap();
                assert!(masters < notes && notes < slides, "{main}");
            })
            .seed("INV-ID-08", |d| {
                d.edit_part(MAIN, |tree| {
                    let root = tree.root();
                    let list = tree.child(root, "notesMasterIdLst").unwrap();
                    tree.remove_child(root, list);
                    tree.insert_child(root, None, list);
                })
                .unwrap();
            })
            .reported(),
    );
    all.push(first_slide_num_case());
    all.extend(bookkeeping_cases());

    // Sections.
    all.push(
        case("RR-SL-01", "INV-SL-01", THREE, vec![add_slide()])
            .check(|parts, _| {
                assert!(
                    !text(parts, MAIN).contains("sectionLst"),
                    "absent stays absent"
                );
            })
            .seed("INV-SL-01", empty_section_list),
    );
    all.push(
        case(
            "RR-SL-02",
            "INV-SL-02",
            "generated/rr-one-section.pptx",
            vec![Op::RemoveSection {
                id: SEC_A.to_owned(),
            }],
        )
        .check(|parts, _| {
            assert!(!text(parts, MAIN).contains("sectionLst"));
        })
        .seed("INV-SL-02", empty_section_list)
        .reported(),
    );
    all.push(
        case(
            "RR-SL-03",
            "INV-SL-03",
            "generated/rr-sections.pptx",
            vec![Op::MoveSlideToSection {
                slide: SlideRef::Id(258),
                section: SEC_A.to_owned(),
                index: 2,
            }],
        )
        .check(|parts, _| {
            assert!(text(parts, MAIN).contains(SEC_B), "an empty section stays");
        })
        .seed("INV-SL-03", |d| {
            d.edit_part(MAIN, |tree| {
                for el in elements(tree) {
                    if tree.plain_attribute(el, "id").as_deref() == Some(SEC_B) {
                        let parent = tree.parent_element(el).unwrap();
                        tree.remove_child(parent, el);
                    }
                }
            })
            .unwrap();
        }),
    );
    all.push(
        case(
            "RR-SL-04",
            "INV-SL-04",
            "generated/rr-sections.pptx",
            vec![delete(1)],
        )
        .check(|parts, _| {
            assert!(!text(parts, MAIN).contains("p14:sldId id=\"257\""));
        })
        .seed("INV-SL-04", |d| add_to_section(d, 0, "257"))
        .reported(),
    );
    all.push(
        case(
            "RR-SL-05",
            "INV-SL-05",
            "generated/rr-sections.pptx",
            vec![Op::MoveSlideToSection {
                slide: SlideRef::Id(256),
                section: SEC_B.to_owned(),
                index: 0,
            }],
        )
        .check(|parts, _| {
            let main = text(parts, MAIN);
            let (a, b, c) = (
                main.find("<p:sldId id=\"257\"").unwrap(),
                main.find("<p:sldId id=\"256\"").unwrap(),
                main.find("<p:sldId id=\"258\"").unwrap(),
            );
            assert!(a < b && b < c, "sldIdLst follows the sections: {main}");
        })
        .seed("INV-SL-05", |d| add_to_section(d, 1, "257"))
        .seed("INV-SL-05", |d| {
            d.edit_part(MAIN, |tree| {
                for el in elements(tree) {
                    if tree.qualified_name(el) == "p14:sldId"
                        && tree.plain_attribute(el, "id").as_deref() == Some("257")
                    {
                        let parent = tree.parent_element(el).unwrap();
                        tree.remove_child(parent, el);
                    }
                }
            })
            .unwrap();
        })
        .reported(),
    );
    all.push(
        case(
            "RR-SL-06",
            "INV-SL-06",
            "generated/rr-sections.pptx",
            vec![add_slide_at(0), add_slide_at(3)],
        )
        .check(|parts, _| {
            let main = text(parts, MAIN);
            let first = main.find("<p14:section ").unwrap();
            let second = main.rfind("<p14:section ").unwrap();
            assert_eq!(main.matches("p14:sldId id=\"259\"").count(), 1);
            let at = |id: &str| main.find(&format!("p14:sldId id=\"{id}\"")).unwrap();
            assert!(
                first < at("259") && at("259") < at("256"),
                "the first section"
            );
            assert!(at("260") < second, "with the slide before it: {main}");
        })
        .seed("INV-SL-06", |d| {
            // Slide 260 moved into the next section, keeping the order that reads.
            d.edit_part(MAIN, |tree| {
                let sections: Vec<ElementId> = elements(tree)
                    .into_iter()
                    .filter(|&el| tree.local_name(el) == "section")
                    .collect();
                let first = tree.child(sections[0], "sldIdLst").unwrap();
                let second = tree.child(sections[1], "sldIdLst").unwrap();
                let moved = *tree.child_elements(first).last().unwrap();
                tree.remove_child(first, moved);
                let front = tree.child_elements(second).first().copied();
                tree.insert_child(second, front, moved);
            })
            .unwrap();
        }),
    );
    all.push(
        case(
            "RR-SL-07",
            "INV-SL-07",
            "generated/rr-sections.pptx",
            vec![Op::AddSection {
                name: "New".to_owned(),
                id: SEC_NEW.to_owned(),
                at: None,
            }],
        )
        .seed("INV-SL-07", |d| {
            set_first(
                d,
                MAIN,
                "section",
                "id",
                "{33333333-aaaa-4bbb-8ccc-000000000003}",
            );
        })
        .seed("INV-SL-07", |d| {
            set_first(d, MAIN, "section", "id", SEC_B);
        })
        .reported(),
    );

    // Notes master.
    all.push(
        case(
            "RR-NM-01",
            "INV-NM-01",
            "generated/rr-notes.pptx",
            vec![Op::AddNotes {
                slide: SlideRef::Index(1),
                text: "Second".to_owned(),
            }],
        )
        .check(|parts, _| {
            assert!(parts.contains_key("ppt/notesSlides/notesSlide2.xml"));
            assert!(!parts.contains_key("ppt/notesMasters/notesMaster2.xml"));
        })
        .seed("INV-NM-01", |d| {
            let body = d.part_bytes("ppt/notesMasters/notesMaster1.xml").unwrap();
            d.set_part_bytes("ppt/notesMasters/notesMaster2.xml", body);
            add_type_entry(
                d,
                "Override",
                &[
                    ("PartName", "/ppt/notesMasters/notesMaster2.xml"),
                    (
                        "ContentType",
                        "application/vnd.openxmlformats-officedocument.presentationml.notesMaster+xml",
                    ),
                ],
            );
        })
        .reported(),
    );
    all.push(
        case("RR-NM-02", "INV-NM-02", THREE, vec![notes()])
            .check(|parts, _| {
                assert!(parts.contains_key("ppt/notesMasters/notesMaster1.xml"));
                assert!(parts.contains_key("ppt/theme/theme2.xml"), "its own theme");
                let types = text(parts, TYPES);
                assert!(types.contains("/ppt/notesMasters/notesMaster1.xml"));
                assert!(types.contains("/ppt/theme/theme2.xml"));
                assert!(text(parts, MAIN).contains("notesMasterIdLst"));
                assert!(text(parts, "ppt/notesSlides/notesSlide1.xml").contains("A note"));
            })
            .seed("INV-NM-02", |d| {
                set_rel_target(
                    d,
                    "ppt/notesMasters/_rels/notesMaster1.xml.rels",
                    "/theme",
                    "../theme/theme1.xml",
                );
            })
            .seed("INV-NM-02", |d| {
                remove_override(d, "/ppt/theme/theme2.xml");
            }),
    );
    all.push(
        case(
            "RR-NM-03",
            "INV-NM-03",
            "generated/rr-four-three.pptx",
            vec![notes()],
        )
        .check(|parts, _| {
            let master = text(parts, "ppt/notesMasters/notesMaster1.xml");
            for kind in ["hdr", "dt", "sldImg", "body", "ftr", "sldNum"] {
                assert!(master.contains(&format!("type=\"{kind}\"")), "{kind}");
            }
            assert!(master.contains("<p:clrMap "));
            assert!(
                master.contains("<a:off x=\"342900\" y=\"571500\"/>"),
                "sized from notesSz: {master}"
            );
        })
        .seed("INV-NM-03", |d| {
            remove_elements(d, "ppt/notesMasters/notesMaster1.xml", "clrMap");
        })
        .seed("INV-NM-03", |d| {
            d.edit_part("ppt/notesMasters/notesMaster1.xml", |tree| {
                let sps: Vec<ElementId> = elements(tree)
                    .into_iter()
                    .filter(|&el| tree.local_name(el) == "sp")
                    .collect();
                let parent = tree.parent_element(sps[0]).unwrap();
                tree.remove_child(parent, sps[0]);
            })
            .unwrap();
        })
        .reported(),
    );
    all.push(
        case("RR-NM-04", "INV-NM-04", THREE, vec![notes()])
            .seed("INV-NM-04", |d| {
                remove_rel_of(
                    d,
                    "ppt/notesSlides/_rels/notesSlide1.xml.rels",
                    "/notesMaster",
                );
            })
            .reported(),
    );
    all.push(
        case(
            "RR-NM-05",
            "INV-NM-05",
            "generated/rr-notes.pptx",
            vec![Op::RemoveNotes {
                slide: SlideRef::Index(0),
            }],
        )
        .check(|parts, _| {
            assert!(!parts.contains_key("ppt/notesSlides/notesSlide1.xml"));
            assert!(parts.contains_key("ppt/notesMasters/notesMaster1.xml"));
        })
        .seed("INV-NM-05", |d| {
            remove_elements(d, MAIN, "notesMasterIdLst");
        }),
    );
    all
}

/// `INV-ID-09`, through a replaced main part, as a model patch would reach it.
const APP: &str = "docProps/app.xml";
const CORE: &str = "docProps/core.xml";
const RR_APP: &str = "generated/rr-app.pptx";
const RR_SIGNED: &str = "generated/rr-signed.pptx";
const RR_THUMB: &str = "generated/rr-thumbnail.pptx";
const WHEN: &str = "2026-11-07T10:00:00Z";
const CERT: &str = "_xmlsignatures/certs/cert1.cer";
const ORIGIN: &str =
    "http://schemas.openxmlformats.org/package/2006/relationships/digital-signature/origin";
const THUMB: &str =
    "http://schemas.openxmlformats.org/package/2006/relationships/metadata/thumbnail";
const CORE_REL: &str =
    "http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties";

fn host() -> HostInputs {
    HostInputs {
        modified: Some(WHEN.to_owned()),
        user: Some("Editor".to_owned()),
        application: Some("OpenPresent".to_owned()),
        thumbnail: None,
    }
}

fn with_host(plan: SavePlan<'static>) -> SavePlan<'static> {
    plan.host(host())
}

fn with_png(plan: SavePlan<'static>) -> SavePlan<'static> {
    plan.host(HostInputs {
        thumbnail: Some(PNG.to_vec()),
        ..host()
    })
}

fn ack(plan: SavePlan<'static>) -> SavePlan<'static> {
    plan.acknowledge_signature()
}

fn retype(extension: &str) -> Op {
    Op::Retype {
        extension: extension.to_owned(),
    }
}

fn author() -> Op {
    Op::AddCommentAuthor {
        name: "Editor".to_owned(),
        initials: "E".to_owned(),
    }
}

/// Set the text of a child of `part`'s root.
/// Set the `size` of the vector under `list` in `app.xml`.
fn set_vector_size(d: &mut Draft<'_>, list: &str, size: &str) {
    d.edit_part(APP, |tree| {
        let held = tree.child(tree.root(), list).unwrap();
        let vector = tree.child(held, "vector").unwrap();
        assert!(tree.set_attribute(vector, "size", size));
    })
    .unwrap();
}

/// Set the count of the `group`th pair in `HeadingPairs`.
fn set_group_count(d: &mut Draft<'_>, group: usize, count: &str) {
    d.edit_part(APP, |tree| {
        let held = tree.child(tree.root(), "HeadingPairs").unwrap();
        let vector = tree.child(held, "vector").unwrap();
        let counts: Vec<ElementId> = tree
            .child_elements(vector)
            .into_iter()
            .filter_map(|variant| tree.child(variant, "i4"))
            .collect();
        tree.set_text(counts[group], count);
    })
    .unwrap();
}

/// A thumbnail must have no relationships of its own (`INV-TH-01`).
const THUMB_RELS: &[u8] = b"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
<Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" \
Target=\"../../ppt/presentation.xml\"/></Relationships>";

fn set_root_text(d: &mut Draft<'_>, part: &str, local: &str, value: &str) {
    d.edit_part(part, |tree| {
        let el = tree.child(tree.root(), local).unwrap();
        tree.set_text(el, value);
    })
    .unwrap();
}

/// Add a child of `part`'s root.
fn add_root_child(d: &mut Draft<'_>, part: &str, name: &str) {
    d.edit_part(part, |tree| {
        let root = tree.root();
        let el = tree.new_element(name);
        tree.insert_child(root, None, el);
    })
    .unwrap();
}

fn main_type_is(parts: &BTreeMap<String, Vec<u8>>, wanted: &str) {
    let types = text(parts, TYPES);
    let found = format!("PartName=\"/ppt/presentation.xml\" ContentType=\"{wanted}\"");
    assert!(types.contains(&found), "{types}");
}

fn bookkeeping_cases() -> Vec<Case> {
    const POTX: &str =
        "application/vnd.openxmlformats-officedocument.presentationml.template.main+xml";
    const PPSX: &str =
        "application/vnd.openxmlformats-officedocument.presentationml.slideshow.main+xml";

    vec![
        case("RR-AP-01", "INV-AP-01", RR_APP, vec![add_slide(), add_slide(), delete(0)])
            .check(|parts, _| {
                let app = text(parts, APP);
                assert!(app.contains("<Slides>4</Slides>"), "{app}");
                assert!(app.contains("<Notes>0</Notes>"), "{app}");
                assert!(app.contains("<HiddenSlides>1</HiddenSlides>"), "{app}");
            })
            .seed("INV-AP-01", |d| set_root_text(d, APP, "Slides", "9"))
            .seed("INV-AP-01", |d| set_root_text(d, APP, "HiddenSlides", "9"))
            .seed("INV-AP-01", |d| set_root_text(d, APP, "Notes", "9"))
            .reported(),
        case("RR-AP-02", "INV-AP-02", RR_APP, vec![add_slide()])
            .check(|parts, _| {
                let app = text(parts, APP);
                assert!(app.contains("<vt:i4>4</vt:i4>"), "{app}");
                assert!(app.contains("size=\"5\""), "{app}");
                assert_eq!(app.matches("<vt:lpstr>Fixture").count(), 4, "{app}");
                assert!(app.contains("PowerPoint Presentation"), "{app}");
            })
            .seed("INV-AP-02", |d| {
                d.edit_part(APP, |tree| {
                    let list = tree.child(tree.root(), "TitlesOfParts").unwrap();
                    let vector = tree.child(list, "vector").unwrap();
                    let el = tree.new_element("vt:lpstr");
                    tree.insert_child(vector, None, el);
                })
                .unwrap();
            })
            // Each of the rule's three checks alone.
            .seed("INV-AP-02", |d| set_vector_size(d, "HeadingPairs", "99"))
            .seed("INV-AP-02", |d| set_vector_size(d, "TitlesOfParts", "99"))
            .seed("INV-AP-02", |d| set_group_count(d, 0, "0"))
            .seed("INV-AP-03", |d| set_group_count(d, 1, "5"))
            .reported(),
        case(
            "RR-AP-03",
            "INV-AP-03",
            "generated/rr-app-unknown-groups.pptx",
            vec![add_slide()],
        )
        .check(|parts, _| {
            let app = text(parts, APP);
            assert!(!app.contains("HeadingPairs"), "{app}");
            assert!(!app.contains("TitlesOfParts"), "{app}");
        })
        .seed("INV-AP-03", |d| {
            d.edit_part(APP, |tree| {
                let root = tree.root();
                let held = tree.new_element("HeadingPairs");
                let vector = tree.new_element("vt:vector");
                tree.insert_child(held, None, vector);
                tree.insert_child(root, None, held);
            })
            .unwrap();
        }),
        case("RR-AP-04", "INV-AP-04", RR_APP, vec![author()])
            .seed("INV-AP-04", |d| set_root_text(d, APP, "AppVersion", "1.2.3"))
            .reported(),
        case("RR-AP-05", "INV-AP-05", RR_APP, vec![author()])
            .prepare(with_host)
            .check(|parts, _| {
                let app = text(parts, APP);
                assert!(app.contains("<Application>OpenPresent</Application>"), "{app}");
                assert!(app.contains("<TotalTime>5</TotalTime>"), "{app}");
            })
            .seed("INV-AP-05", |d| {
                set_root_text(d, APP, "Application", "Microsoft Office PowerPoint");
            }),
        case("RR-AP-06", "INV-AP-06", RR_SIGNED, vec![author()])
            .prepare(ack)
            .check(|parts, _| assert!(!text(parts, APP).contains("DigSig")))
            .seed("INV-AP-06", |d| add_root_child(d, APP, "DigSig")),
        case("RR-AP-07", "INV-AP-07", RR_APP, vec![author()])
            .seed("INV-AP-07", |d| {
                raw_rel(d, "_rels/.rels", "rId90", CORE_REL, "docProps/core.xml");
            }),
        case("RR-AP-08", "INV-AP-08", RR_APP, vec![author()])
            .prepare(with_host)
            .check(|parts, _| {
                let core = text(parts, CORE);
                assert!(core.contains(WHEN), "{core}");
                assert!(core.contains("2020-01-01T00:00:00Z"), "created stays: {core}");
            })
            .seed("INV-AP-08", |d| {
                d.edit_part(CORE, |tree| {
                    let el = tree.child(tree.root(), "modified").unwrap();
                    assert!(tree.remove_attribute(el, "xsi:type"));
                })
                .unwrap();
            })
            .seed("INV-AP-08", |d| set_root_text(d, CORE, "created", WHEN))
            .seed("INV-AP-09", |d| set_root_text(d, CORE, "lastModifiedBy", "Other")),
        case(
            "RR-AP-09",
            "INV-AP-09",
            "generated/rr-core-text-revision.pptx",
            vec![author()],
        )
        .prepare(with_host)
        .check(|parts, _| {
            let core = text(parts, CORE);
            assert!(core.contains(">draft<"), "text stays: {core}");
            assert!(core.contains(">Editor<"), "{core}");
        })
        .seed("INV-AP-09", |d| set_root_text(d, CORE, "revision", "1")),
        case("RR-AP-10", "INV-AP-10", RR_APP, vec![author()])
            .prepare(with_host)
            .check(|parts, _| {
                let core = text(parts, CORE);
                assert!(core.contains("<cp:value xml:lang=\"fr-FR\">deux</cp:value>"), "{core}");
                assert!(core.contains("<cp:revision>8</cp:revision>"), "{core}");
            })
            .seed("INV-AP-10", |d| set_root_text(d, CORE, "keywords", "flat")),
        case("RR-TH-01", "INV-TH-01", RR_THUMB, vec![author()])
            .prepare(with_png)
            .seed("INV-TH-01", |d| {
                raw_rel(d, "_rels/.rels", "rId90", THUMB, "docProps/thumbnail.png");
            })
            .seed("INV-TH-01", |d| {
                d.set_part_bytes("docProps/_rels/thumbnail.png.rels", THUMB_RELS.to_vec());
            }),
        case("RR-TH-02", "INV-TH-02", RR_THUMB, vec![author()])
            .check(|parts, _| {
                assert!(!parts.contains_key("docProps/thumbnail.jpeg"));
                assert!(!text(parts, "_rels/.rels").contains("thumbnail"));
            })
            .seed("INV-TH-02", |d| {
                d.set_part_bytes("docProps/thumbnail.jpeg", b"\xff\xd8\xff\xd9".to_vec());
                raw_rel(d, "_rels/.rels", "rId90", THUMB, "docProps/thumbnail.jpeg");
            }),
        case("RR-TH-03", "INV-TH-03", RR_THUMB, vec![author()])
            .prepare(with_png)
            .check(|parts, _| {
                assert_eq!(parts["docProps/thumbnail.png"], PNG);
                assert!(!parts.contains_key("docProps/thumbnail.jpeg"));
            })
            .seed("INV-TH-03", |d| {
                d.set_part_bytes("docProps/thumbnail.jpeg", b"\xff\xd8\xff\xd9".to_vec());
            }),
        case("RR-SG-01", "INV-SG-01", RR_SIGNED, vec![author()])
            .prepare(ack)
            .seed("INV-SG-01", |d| {
                d.set_part_bytes("_xmlsignatures/sig1.xml", b"<Signature/>".to_vec());
                add_type_entry(
                    d,
                    "Override",
                    &[
                        ("PartName", "/_xmlsignatures/sig1.xml"),
                        (
                            "ContentType",
                            "application/vnd.openxmlformats-package.digital-signature-xmlsignature+xml",
                        ),
                    ],
                );
            })
            .reported(),
        case("RR-SG-02", "INV-SG-02", RR_SIGNED, vec![author()])
            .prepare(ack)
            .seed("INV-SG-02", |d| d.forget_acknowledgement()),
        case("RR-SG-03", "INV-SG-03", RR_SIGNED, vec![author()])
            .prepare(ack)
            .check(|parts, _| {
                assert!(!parts.keys().any(|k| k.starts_with("_xmlsignatures")));
                assert!(!text(parts, "_rels/.rels").contains("digital-signature"));
                assert!(!text(parts, TYPES).contains("digital-signature"));
            })
            .seed("INV-SG-03", |d| d.set_part_bytes(CERT, b"x".to_vec())),
        case("RR-SG-04", "INV-SG-04", RR_SIGNED, vec![author()])
            .prepare(ack)
            .seed("INV-SG-04", |d| d.forget_report("CS-SAV-0001")),
        case("RR-SG-05", "INV-SG-05", RR_SIGNED, vec![author()])
            .prepare(ack)
            .seed("INV-SG-05", |d| {
                raw_rel(d, "_rels/.rels", "rId90", ORIGIN, "_xmlsignatures/origin.sigs");
            }),
        case("RR-SG-06", "INV-SG-06", RR_SIGNED, vec![])
            .check(|parts, _| {
                assert!(parts.contains_key(CERT), "nothing edited, nothing dropped");
            })
            .no_seed("nothing dirty means the input bytes; a_signed_deck_with_its_edit_undone_keeps_its_bytes"),
        case("RR-SG-07", "INV-SG-07", RR_SIGNED, vec![retype("potx")])
            .prepare(ack)
            .check(|parts, _| {
                main_type_is(parts, POTX);
                assert!(!parts.contains_key(CERT));
            })
            .seed("INV-SG-05", |d| {
                raw_rel(d, "_rels/.rels", "rId90", ORIGIN, "_xmlsignatures/origin.sigs");
            }),
        case("RR-SG-08", "INV-SG-08", "generated/rr-signed-vba.pptm", vec![author()])
            .prepare(ack)
            .check(|parts, _| {
                assert!(parts.contains_key("ppt/vbaProjectSignature.bin"));
                assert!(!parts.contains_key(CERT));
            })
            .seed("INV-SG-08", |d| d.drop_entry("ppt/vbaProjectSignature.bin")),
        case("RR-SA-01", "INV-SA-01", THREE, vec![retype("ppsx")])
            .check(|parts, _| main_type_is(parts, PPSX))
            .seed("INV-SA-01", |d| {
                d.edit_part(TYPES, |tree| {
                    for el in tree.child_elements(tree.root()) {
                        if tree.plain_attribute(el, "PartName").as_deref()
                            == Some("/ppt/presentation.xml")
                        {
                            assert!(tree.set_attribute(el, "ContentType", "application/wrong"));
                        }
                    }
                })
                .unwrap();
            }),
        case("RR-SA-02", "INV-SA-02", THREE, vec![retype("potx")])
            .seed("INV-SA-02", |d| {
                set_first(d, "ppt/slides/slide1.xml", "cNvPr", "name", "Changed");
            }),
        case("RR-SA-03", "INV-SA-03", "generated/rr-vba.pptm", vec![Op::DropVba, retype("pptx")])
            .check(|parts, _| assert!(!parts.contains_key("ppt/vbaProject.bin")))
            .seed("INV-SA-03", |d| {
                d.set_part_bytes("ppt/vbaProject.bin", b"x".to_vec());
                raw_rel(
                    d,
                    MAIN_RELS,
                    "rId90",
                    "http://schemas.microsoft.com/office/2006/relationships/vbaProject",
                    "vbaProject.bin",
                );
            })
            .reported(),
        case(
            "RR-SA-04",
            "INV-SA-04",
            "generated/rr-macro-action.pptm",
            vec![Op::DropVba, retype("pptx")],
        )
        .check(|parts, _| {
            assert!(!text(parts, "ppt/slides/slide1.xml").contains("ppaction"));
        })
        .seed("INV-SA-04", |d| {
            set_first(
                d,
                "ppt/slides/slide1.xml",
                "cNvPr",
                "descr",
                "ppaction://macro?name=Run",
            );
        }),
        case("RR-SA-05", "INV-SA-05", THREE, vec![retype("pptm")])
            .check(|parts, _| assert!(!parts.contains_key("ppt/vbaProject.bin")))
            .seed("INV-SA-05", |d| {
                d.set_part_bytes("ppt/custom/extra.xml", VALID_XML.to_vec());
            }),
        case("RR-SA-06", "INV-SA-06", "generated/rr-activex.pptx", vec![retype("ppsx")])
            .check(|parts, _| {
                assert!(parts.contains_key("customUI/customUI.xml"));
                assert!(parts.contains_key("ppt/activeX/activeX1.xml"));
            })
            .seed("INV-SA-06", |d| d.drop_entry("customUI/customUI.xml")),
        case("RR-SA-14", "INV-SA-14", "generated/slides-strict.pptx", vec![retype("potx")])
            .check(|parts, _| {
                assert!(text(parts, MAIN_RELS).contains("purl.oclc.org"));
            })
            .seed("INV-RI-09", |d| {
                raw_rel(d, MAIN_RELS, "rId90", &format!("{T}/tags"), "tags/t.xml");
                d.set_part_bytes("ppt/tags/t.xml", VALID_XML.to_vec());
            }),
    ]
}

/// `INV-SG-06`: an edit undone is nothing dirty, so the signature stands.
#[test]
fn a_signed_deck_with_its_edit_undone_keeps_its_bytes() {
    let source = std::fs::read(fixtures().join(RR_SIGNED)).unwrap();
    let mut pkg = open(&source);
    let same = pkg.read_part("ppt/slides/slide1.xml").unwrap();
    let outcome = SavePlan::new()
        .replace_part("ppt/slides/slide1.xml", same)
        .save(&mut pkg)
        .unwrap();
    assert_eq!(outcome.bytes, source);
}

/// `INV-SG-02`: an edit is refused until the host has told the user, and leaves no trace.
#[test]
fn an_edit_of_a_signed_deck_needs_an_acknowledgement() {
    let source = std::fs::read(fixtures().join(RR_SIGNED)).unwrap();
    let mut pkg = open(&source);
    let err = SavePlan::new().op(author()).save(&mut pkg).unwrap_err();
    assert_eq!(err.code(), "CS-SAV-0015", "{err}");
    let outcome = SavePlan::new()
        .acknowledge_signature()
        .op(author())
        .save(&mut pkg)
        .unwrap();
    assert!(outcome.signature_invalidated);
    let entry = outcome
        .report
        .iter()
        .find(|e| e.code == "CS-SAV-0001")
        .unwrap();
    assert_eq!(entry.detail, "1 signatures removed");
}

/// A stale count that arrived, with nothing here changing it, is reported and left (F2).
#[test]
fn a_count_that_arrived_stale_stays_until_the_slides_change() {
    let source = std::fs::read(fixtures().join(RR_APP)).unwrap();
    let mut pkg = open(&source);
    let app = String::from_utf8(pkg.read_part(APP).unwrap()).unwrap();
    let mut seeded = casual_slide_package::PartEdits::default();
    seeded.replace(APP, app.replace("<Slides>3<", "<Slides>9<").into_bytes());
    let stale = pkg.save_with(&seeded).unwrap();
    let mut pkg = open(&stale);
    let kept = SavePlan::new().op(author()).save(&mut pkg).unwrap();
    assert!(text(&parts_of(&kept.bytes), APP).contains("<Slides>9<"));
    assert!(kept.report.iter().any(|e| e.rule == "INV-AP-01"));
    let mut pkg = open(&stale);
    let fixed = SavePlan::new().op(add_slide()).save(&mut pkg).unwrap();
    assert!(text(&parts_of(&fixed.bytes), APP).contains("<Slides>4<"));
}

fn first_slide_num_case() -> Case {
    let bytes = std::fs::read(fixtures().join(THREE)).unwrap();
    let main = String::from_utf8(parts_of(&bytes)[MAIN].clone()).unwrap();
    let main = main.replacen(
        "<p:presentation ",
        "<p:presentation firstSlideNum=\"9999\" ",
        1,
    );
    let mut c = case("RR-ID-09", "INV-ID-09", THREE, Vec::new())
        .check(|parts, _| {
            assert!(text(parts, MAIN).contains("firstSlideNum=\"9999\""));
        })
        .seed("INV-ID-09", |d| {
            d.edit_part(MAIN, |tree| {
                let root = tree.root();
                assert!(tree.set_attribute(root, "firstSlideNum", "10000"));
            })
            .unwrap();
        })
        .reported();
    c.replaced.push((MAIN, main.into_bytes()));
    c
}

fn plan_of(case: &Case) -> SavePlan<'static> {
    let mut plan = SavePlan::new();
    for (part, bytes) in &case.replaced {
        plan = plan.replace_part(*part, bytes.clone());
    }
    for op in &case.plan {
        plan = plan.op(op.clone());
    }
    if let Some(prepare) = case.prepare {
        plan = prepare(plan);
    }
    plan
}

fn stage_and_seed(source: &[u8], case: &Case, seed: Seed) -> Result<(), WriterError> {
    let mut pkg = open(source);
    let mut draft = plan_of(case).stage(&mut pkg).unwrap();
    seed(&mut draft);
    draft.preflight().map(|_| ())
}

fn rr_dir() -> PathBuf {
    fixtures().join("rr")
}

/// The seeded file for a draft seed, or why there is none.
fn write_seeded(source: &[u8], case: &Case, seed: Seed) -> Result<Vec<u8>, String> {
    let mut pkg = open(source);
    let mut draft = plan_of(case).stage(&mut pkg).unwrap();
    seed(&mut draft);
    let edits = draft.into_part_edits().unwrap();
    pkg.save_with(&edits).map_err(|e| e.code().to_owned())
}

/// Given as an input deck, a seeded file opens, saves after an unrelated edit, and reports
/// the break where one package can show it: from the writer, or, for what the reader had to
/// repair, in the recovery report (`docs/36`, `docs/29`). What became of it.
fn benign_save(seeded: &[u8], case: &Case) -> String {
    let mut pkg = PresentationPackage::open(seeded.to_vec(), OoxmlLimits::default())
        .unwrap_or_else(|err| panic!("{}: open refused: {err}", case.id));
    let repaired = !pkg.recovery_report().is_empty();
    let outcome = SavePlan::new()
        .acknowledge_signature()
        .op(Op::AddCommentAuthor {
            name: "Editor".to_owned(),
            initials: "E".to_owned(),
        })
        .save(&mut pkg)
        .unwrap_or_else(|err| panic!("{}: a seeded deck must still save: {err}", case.id));
    let reported = outcome
        .report
        .iter()
        .any(|e| e.code == "CS-SAV-0021" && e.rule == case.rule);
    assert!(
        reported || repaired || !case.arrives,
        "{}: as input the break is reported: {:?}",
        case.id,
        outcome.report
    );
    if repaired {
        String::from("opens by repair, saves, reports")
    } else if reported {
        String::from("opens, saves, reports")
    } else {
        String::from("opens and saves; one package does not show it")
    }
}

/// Every invariant of the families `T1N-01` to `T1N-03` own has its fixture.
#[test]
fn no_invariant_in_these_rows_is_left_without_a_fixture() {
    let families = [
        ("PN", 11),
        ("RI", 10),
        ("CT", 10),
        ("SP", 10),
        ("ID", 9),
        ("SL", 7),
        ("NM", 5),
        ("AP", 10),
        ("TH", 3),
        ("SG", 8),
    ];
    let have: BTreeSet<String> = cases().iter().map(|c| c.id.to_owned()).collect();
    for (family, count) in families {
        for n in 1..=count {
            let id = format!("RR-{family}-{n:02}");
            assert!(have.contains(&id), "{id} has no fixture");
        }
    }
}

#[test]
fn every_fixture_saves_its_good_deck_and_refuses_its_seed() {
    let write = std::env::var_os("RR_WRITE").is_some();
    let dir = rr_dir();
    let mut manifest = String::from("id\tinvariant\tbase\tgood\tseeded\n");
    let mut ids = BTreeSet::new();
    for case in cases() {
        assert!(ids.insert(case.id), "{} twice", case.id);
        let source = std::fs::read(fixtures().join(case.base))
            .unwrap_or_else(|_| panic!("{}: {}", case.id, case.base));

        // Good: through the writer.
        let mut pkg = open(&source);
        let good = plan_of(&case)
            .save(&mut pkg)
            .unwrap_or_else(|e| panic!("{}: {e}", case.id));
        if let Some(check) = case.check {
            check(&parts_of(&good.bytes), &good.new_slides);
        }

        // Seeded: refused, naming the invariant.
        let mut seeded_file = None;
        let mut seeded_note = String::from("no seeded deck: the seed is in the ZIP");
        for (rule, seed) in &case.seeds {
            let err = stage_and_seed(&source, &case, *seed)
                .expect_err(&format!("{}: {rule} was not refused", case.id));
            assert_eq!(err.code(), "CS-SAV-0012", "{}: {err}", case.id);
            assert!(
                err.names_rule(rule),
                "{}: {rule} not named by {err}",
                case.id
            );
        }
        if let Some((_, seed)) = case.seeds.first() {
            match write_seeded(&source, &case, *seed) {
                Ok(bytes) => {
                    seeded_note = benign_save(&bytes, &case);
                    seeded_file = Some(bytes);
                }
                Err(code) => seeded_note = format!("cannot be written: {code}"),
            }
        }
        if let Some((rule, seed)) = &case.bytes {
            let (input, output, changed) = seed(&source, &good.bytes);
            let err = check_zip(&input, &output, &changed)
                .expect_err(&format!("{}: {rule} was not refused", case.id));
            assert_eq!(err.code(), "CS-SAV-0020", "{}: {err}", case.id);
            assert!(
                err.names_rule(rule),
                "{}: {rule} not named by {err}",
                case.id
            );
            seeded_note = format!("seeded in the ZIP; {}", benign_save(&output, &case));
            seeded_file = Some(output);
        }
        if let Some(why) = case.no_seed {
            seeded_note = format!("no seed: {why}");
        }
        assert!(
            !case.seeds.is_empty() || case.bytes.is_some() || case.no_seed.is_some(),
            "{} has no seed",
            case.id
        );

        // The committed pair.
        let good_path = dir.join("good").join(format!("{}.pptx", case.id));
        let seeded_path = dir.join("seeded").join(format!("{}.pptx", case.id));
        if write {
            std::fs::create_dir_all(good_path.parent().unwrap()).unwrap();
            std::fs::create_dir_all(seeded_path.parent().unwrap()).unwrap();
            std::fs::write(&good_path, &good.bytes).unwrap();
            if let Some(bytes) = &seeded_file {
                std::fs::write(&seeded_path, bytes).unwrap();
            }
        } else {
            let committed = std::fs::read(&good_path).unwrap_or_else(|_| {
                panic!("{}: not committed; run with RR_WRITE=1", case.id);
            });
            assert_eq!(
                parts_of(&committed),
                parts_of(&good.bytes),
                "{}: the writer has moved; review, then RR_WRITE=1",
                case.id
            );
        }
        writeln!(
            manifest,
            "{}\t{}\t{}\t{}\t{}",
            case.id,
            case.rule,
            case.base,
            // The manifest is the same on every OS.
            good_path
                .strip_prefix(&dir)
                .unwrap()
                .display()
                .to_string()
                .replace('\\', "/"),
            seeded_note
        )
        .unwrap();
    }
    let manifest_path = dir.join("manifest.tsv");
    if write {
        std::fs::write(&manifest_path, &manifest).unwrap();
    } else {
        // A Windows checkout may hold it with CRLF.
        let held = std::fs::read_to_string(&manifest_path)
            .unwrap()
            .replace("\r\n", "\n");
        assert_eq!(held, manifest);
    }
}

/// `emit` checks what it wrote. Preflight cannot see a wrong ZIP, and the reader that opens
/// the result reads the central directory, so a local header that disagrees reaches the
/// user unless `check_zip` runs.
#[test]
fn a_written_zip_with_disagreeing_headers_is_refused_by_emit() {
    let source = std::fs::read(fixtures().join(THREE)).unwrap();
    let mut pkg = open(&source);
    let err = SavePlan::new()
        .op(add_slide())
        .stage(&mut pkg)
        .unwrap()
        .emit_tampered(|bytes| bytes[18] = bytes[18].wrapping_add(1))
        .unwrap_err();
    assert_eq!(err.code(), "CS-SAV-0020", "{err}");
    assert!(err.names_rule("INV-PN-10"), "{err}");
}

/// Two relationships of one id: the reader repairs it, the save works, and the report says.
#[test]
fn a_duplicate_relationship_id_opens_by_repair_saves_and_reports() {
    let source = std::fs::read(fixtures().join(THREE)).unwrap();
    let case = cases().into_iter().find(|c| c.id == "RR-RI-01").unwrap();
    let (_, duplicate) = case.seeds[1];
    let bytes = write_seeded(&source, &case, duplicate).unwrap();
    assert_eq!(
        benign_save(&bytes, &case),
        "opens by repair, saves, reports"
    );
}

/// `RR-PN-01`: `Package::rewrite` will not write these names, so they are put in the ZIP by
/// hand. The deck opens, saves, and says so.
#[test]
fn a_part_name_breaking_opc_opens_saves_and_reports() {
    let source = std::fs::read(fixtures().join(THREE)).unwrap();
    let case = cases().into_iter().find(|c| c.id == "RR-PN-01").unwrap();
    for name in ["ppt/custom/thing.", "ppt/custom/a%2Fb.xml"] {
        let mut entries = stored_copy(&source);
        entries.push((name.to_owned(), VALID_XML.to_vec()));
        let bytes = stored_zip(&as_refs(&entries));
        let note = benign_save(&bytes, &case);
        assert_ne!(
            note, "opens and saves; one package does not show it",
            "{name}"
        );
    }
}

/// F4 on every good deck.
#[test]
fn every_good_deck_validates() {
    let have = |tool: &str, arg: &str| Command::new(tool).arg(arg).output().is_ok();
    if !have("python3", "--version") || !have("xmllint", "--version") {
        assert!(
            std::env::var_os("F4_REQUIRED").is_none(),
            "F4_REQUIRED is set, and python3 or xmllint is missing"
        );
        eprintln!("xmllint or python3 not installed: F4 skipped");
        return;
    }
    let dir = rr_dir().join("good");
    assert!(dir.is_dir(), "run with RR_WRITE=1 first");
    let tool = fixtures().join("../tools/f4/validate.py");
    let out = Command::new("python3")
        .arg("-I")
        .arg(tool)
        .arg(&dir)
        .output()
        .unwrap();
    let report = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "{report}");
}

/// Every proposed theme path goes through the same edit: a theme shared on arrival stays
/// shared until edited, then the edit gets its own part (`INV-SP-04`, `INV-SP-07`).
#[test]
fn editing_a_shared_theme_gives_the_master_its_own() {
    let source = std::fs::read(fixtures().join("generated/rr-shared-theme.pptx")).unwrap();
    let mut pkg = open(&source);
    let bytes = pkg.read_part("ppt/theme/theme1.xml").unwrap();
    let mut tree = XmlTree::parse(&bytes, "t", &OoxmlLimits::default()).unwrap();
    assert!(set_theme_name(&mut tree, "Mine").unwrap());
    let outcome = SavePlan::new()
        .op(Op::EditTheme {
            master: "ppt/slideMasters/slideMaster1.xml".to_owned(),
            bytes: tree.to_bytes(),
        })
        .save(&mut pkg)
        .unwrap();
    let after = parts_of(&outcome.bytes);
    assert_eq!(after["ppt/theme/theme1.xml"], bytes, "the other referrer's");
    let master_rels = text(&after, "ppt/slideMasters/_rels/slideMaster1.xml.rels");
    assert!(master_rels.contains("theme2.xml"), "{master_rels}");
    assert!(text(&after, "ppt/theme/theme2.xml").contains("name=\"Mine\""));
    assert!(
        text(&after, MAIN_RELS).contains("theme/theme2.xml"),
        "the presentation follows the first master"
    );

    // Overwriting the shared part in place is what the rule refuses.
    let mut pkg = open(&source);
    let err = SavePlan::new()
        .replace_part("ppt/theme/theme1.xml", tree.to_bytes())
        .save(&mut pkg)
        .unwrap_err();
    assert!(err.names_rule("INV-SP-04"), "{err}");
}

#[test]
fn a_new_section_can_split_one_and_then_go_again() {
    let source = std::fs::read(fixtures().join("generated/rr-sections.pptx")).unwrap();
    let mut pkg = open(&source);
    let outcome = SavePlan::new()
        .op(Op::AddSection {
            name: "Split".to_owned(),
            id: SEC_NEW.to_owned(),
            at: Some(SlideRef::Id(257)),
        })
        .save(&mut pkg)
        .unwrap();
    let main = text(&parts_of(&outcome.bytes), MAIN);
    let new_at = main.find(SEC_NEW).unwrap();
    let b_at = main.find(SEC_B).unwrap();
    assert!(new_at < b_at, "after the section it split: {main}");
    assert!(main.find("p14:sldId id=\"257\"").unwrap() > new_at);

    let mut pkg = open(&outcome.bytes);
    let again = SavePlan::new()
        .op(Op::RemoveSection {
            id: SEC_NEW.to_owned(),
        })
        .save(&mut pkg)
        .unwrap();
    assert!(!text(&parts_of(&again.bytes), MAIN).contains(SEC_NEW));
}

#[test]
fn a_new_section_list_holds_every_slide() {
    let source = std::fs::read(fixtures().join(THREE)).unwrap();
    let mut pkg = open(&source);
    let outcome = SavePlan::new()
        .op(Op::AddSection {
            name: "All".to_owned(),
            id: SEC_NEW.to_owned(),
            at: None,
        })
        .save(&mut pkg)
        .unwrap();
    let main = text(&parts_of(&outcome.bytes), MAIN);
    assert_eq!(main.matches("<p14:sldId ").count(), 3, "{main}");
}

#[test]
fn vba_in_a_macro_free_type_needs_confirmation() {
    let source = std::fs::read(fixtures().join("generated/rr-vba.pptm")).unwrap();
    let mut pkg = open(&source);
    let err = SavePlan::new()
        .op(Op::Retype {
            extension: "pptx".to_owned(),
        })
        .save(&mut pkg)
        .unwrap_err();
    assert_eq!(err.code(), "CS-SAV-0003");
    let kept = SavePlan::new()
        .op(Op::Retype {
            extension: "potm".to_owned(),
        })
        .save(&mut pkg)
        .unwrap();
    assert!(parts_of(&kept.bytes).contains_key("ppt/vbaProject.bin"));
}

#[test]
fn a_shape_with_a_creation_id_is_not_copied_without_a_new_one() {
    let source = std::fs::read(fixtures().join("generated/rr-animated.pptx")).unwrap();
    let mut pkg = open(&source);
    let err = SavePlan::new()
        .op(Op::CopyShape {
            slide: SlideRef::Index(0),
            shape_id: 4,
            creation_id: None,
        })
        .save(&mut pkg)
        .unwrap_err();
    assert_eq!(err.code(), "CS-SAV-0017");
}

#[test]
fn a_layout_in_use_and_a_last_master_are_refused() {
    let source = std::fs::read(fixtures().join(THREE)).unwrap();
    let mut pkg = open(&source);
    let err = SavePlan::new()
        .op(Op::RemoveLayout {
            layout: "ppt/slideLayouts/slideLayout1.xml".to_owned(),
        })
        .save(&mut pkg)
        .unwrap_err();
    assert_eq!(err.code(), "CS-SAV-0017");
    let err = SavePlan::new()
        .op(Op::RemoveMaster {
            master: "ppt/slideMasters/slideMaster1.xml".to_owned(),
        })
        .save(&mut pkg)
        .unwrap_err();
    assert_eq!(err.code(), "CS-SAV-0017");
}

#[test]
fn removing_a_notes_master_is_never_asked_for_and_a_handout_is_never_minted() {
    let source = std::fs::read(fixtures().join("generated/rr-notes.pptx")).unwrap();
    let mut pkg = open(&source);
    let outcome = SavePlan::new()
        .op(delete(0))
        .op(delete(0))
        .save(&mut pkg)
        .unwrap();
    let after = parts_of(&outcome.bytes);
    assert!(after.contains_key("ppt/notesMasters/notesMaster1.xml"));
    assert!(!after.keys().any(|k| k.contains("handoutMaster")));
    assert!(
        text(&after, MAIN).contains("<p:sldIdLst"),
        "absent stays absent"
    );
}

#[test]
fn notes_text_a_part_cannot_carry_is_refused() {
    let source = std::fs::read(fixtures().join(THREE)).unwrap();
    let mut pkg = open(&source);
    let err = SavePlan::new()
        .op(Op::AddNotes {
            slide: SlideRef::Index(0),
            text: "a\u{1}b".to_owned(),
        })
        .save(&mut pkg)
        .unwrap_err();
    assert_eq!(err.code(), "CS-EXP-0001");
}

#[test]
fn notes_text_is_escaped_and_never_reaches_a_message() {
    let source = std::fs::read(fixtures().join(THREE)).unwrap();
    let mut pkg = open(&source);
    let outcome = SavePlan::new()
        .op(Op::AddNotes {
            slide: SlideRef::Index(0),
            text: "<script>&".to_owned(),
        })
        .save(&mut pkg)
        .unwrap();
    assert!(
        text(&parts_of(&outcome.bytes), "ppt/notesSlides/notesSlide1.xml")
            .contains("&lt;script&gt;&amp;")
    );
    assert!(!format!("{:?}", outcome.report).contains("script"));
}

/// `INV-AP-08`: a host that supplies no time leaves `dcterms:modified` alone, so a draft
/// that changes it anyway is refused.
#[test]
fn a_modified_time_changed_with_no_time_supplied_is_refused() {
    let source = std::fs::read(fixtures().join(RR_APP)).unwrap();
    let mut pkg = open(&source);
    let plan = SavePlan::new().op(author()).host(HostInputs {
        user: Some("Editor".to_owned()),
        ..HostInputs::default()
    });
    let mut draft = plan.stage(&mut pkg).unwrap();
    set_root_text(&mut draft, CORE, "modified", WHEN);
    let err = draft.preflight().map(|_| ()).unwrap_err();
    assert_eq!(err.code(), "CS-SAV-0012", "{err}");
    assert!(err.names_rule("INV-AP-08"), "{err}");
}

/// Every seed of the bookkeeping rows is refused naming its rule, each one alone, so a
/// failure elsewhere in the fixture walk cannot hide one.
#[test]
fn every_bookkeeping_seed_is_refused_naming_its_rule() {
    for case in cases()
        .into_iter()
        .filter(|c| c.id.starts_with("RR-AP-") || c.id.starts_with("RR-TH-"))
    {
        let source = std::fs::read(fixtures().join(case.base)).unwrap();
        for (rule, seed) in &case.seeds {
            let err = stage_and_seed(&source, &case, *seed)
                .expect_err(&format!("{}: {rule} was not refused", case.id));
            assert_eq!(err.code(), "CS-SAV-0012", "{}: {err}", case.id);
            assert!(
                err.names_rule(rule),
                "{}: {rule} not named by {err}",
                case.id
            );
        }
    }
}
