//! Package and model repair: from a sound container holding damaged parts, to a clean
//! image the strict reader admits (`REC-02`, `REC-04`, `docs/29-RECOVERY.md`).
//!
//! The image is the whole design. Every reader and the writer see one clean package, so
//! none of them grows a second way to read damage; the bytes as they arrived are kept
//! apart, for an unedited save. So a part synthesized here exists in memory until a user's
//! edit writes it out, and a save with no edit returns the damage untouched.
//!
//! A damaged part's original bytes go to a retained entry under `recovered/`, and what
//! stands in its place is the best reading that can still be trusted to be valid. Nothing
//! here follows a target out of the package, or takes a number from the file as a size.

use std::collections::{BTreeMap, BTreeSet};

use casual_slide_package::{
    Package, PartEdits, RETAINED_DIR, RETAINED_EXTENSION, part_name_key, retained_name, scrub_name,
};

use crate::content_types::{CONTENT_TYPES_PART, ContentTypes};
use crate::error::OoxmlError;
use crate::graph::{Conformance, Finding, FindingKind};
use crate::limits::OoxmlLimits;
use crate::mce::resolve_compatibility;
use crate::namespaces::{MARKUP_COMPATIBILITY, is_understood};
use crate::part_name::entry_key;
use crate::recover_xml::{Prune, XmlRepair, declaration_disagrees, recover_xml};
use crate::recovery::Repair;
use crate::rels::{
    Relationship, TargetMode, parse_relationships, rels_part_for, resolve_internal, source_of_rels,
};
use crate::xml::{ElementId, XmlTree, escape_attribute, walk};

pub(crate) const TRANSITIONAL_RELS: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/";
pub(crate) const STRICT_RELS: &str = "http://purl.oclc.org/ooxml/officeDocument/relationships/";
const RELS_NS: &str = "http://schemas.openxmlformats.org/package/2006/relationships";
const TYPES_NS: &str = "http://schemas.openxmlformats.org/package/2006/content-types";
const PML_NS: &str = "http://schemas.openxmlformats.org/presentationml/2006/main";
const DML_NS: &str = "http://schemas.openxmlformats.org/drawingml/2006/main";
const R_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
const EP_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/extended-properties";
const CP_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/custom-properties";
const VT_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes";
const CORE_NS: &str = "http://schemas.openxmlformats.org/package/2006/metadata/core-properties";
const DC_NS: &str = "http://purl.org/dc/elements/1.1/";
const DCTERMS_NS: &str = "http://purl.org/dc/terms/";
const XSI_NS: &str = "http://www.w3.org/2001/XMLSchema-instance";
const CORE_REL: &str =
    "http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties";
const DECL: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n";
const RELS_TYPE: &str = "application/vnd.openxmlformats-package.relationships+xml";
const PML_TYPE: &str = "application/vnd.openxmlformats-officedocument.presentationml.";
const THEME_TYPE: &str = "application/vnd.openxmlformats-officedocument.theme+xml";
const FIRST_MASTER_ID: u64 = 2_147_483_648;
const FIRST_SLIDE_ID: u64 = 256;
/// A slide size and notes size to fall back on: 16:9, and portrait letter.
const DEFAULT_SLIDE_SIZE: (u64, u64) = (12_192_000, 6_858_000);
const DEFAULT_NOTES_SIZE: (u64, u64) = (6_858_000, 9_144_000);

/// A role a part plays. Role decides what can stand in for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Role {
    Main,
    Slide,
    Layout,
    Master,
    Notes,
    NotesMaster,
    HandoutMaster,
    Theme,
    PresProps,
    ViewProps,
    TableStyles,
    Core,
    App,
    Custom,
    CommentAuthors,
    Comments,
    Tags,
    Other,
}

impl Role {
    fn of_rel_kind(kind: &str) -> Option<Self> {
        Some(match kind {
            "officeDocument" => Self::Main,
            "slide" => Self::Slide,
            "slideLayout" => Self::Layout,
            "slideMaster" => Self::Master,
            "notesSlide" => Self::Notes,
            "notesMaster" => Self::NotesMaster,
            "handoutMaster" => Self::HandoutMaster,
            "theme" => Self::Theme,
            "presProps" => Self::PresProps,
            "viewProps" => Self::ViewProps,
            "tableStyles" => Self::TableStyles,
            "extended-properties" | "extendedProperties" => Self::App,
            "custom-properties" | "customProperties" => Self::Custom,
            "commentAuthors" => Self::CommentAuthors,
            "comments" => Self::Comments,
            "tags" => Self::Tags,
            _ => return None,
        })
    }

    /// The role a relationship type gives its target.
    fn of_rel(rel_type: &str) -> Option<Self> {
        if rel_type.ends_with("/metadata/core-properties") {
            return Some(Self::Core);
        }
        rel_kind(rel_type).and_then(Self::of_rel_kind)
    }

    fn of_root(local: &str) -> Option<Self> {
        Some(match local {
            "presentation" => Self::Main,
            "sld" => Self::Slide,
            "sldLayout" => Self::Layout,
            "sldMaster" => Self::Master,
            "notes" => Self::Notes,
            "notesMaster" => Self::NotesMaster,
            "handoutMaster" => Self::HandoutMaster,
            "theme" => Self::Theme,
            "presentationPr" => Self::PresProps,
            "viewPr" => Self::ViewProps,
            "tblStyleLst" => Self::TableStyles,
            "coreProperties" => Self::Core,
            "cmAuthorLst" => Self::CommentAuthors,
            "cmLst" => Self::Comments,
            "tagLst" => Self::Tags,
            _ => return None,
        })
    }

    fn of_content_type(content_type: &str) -> Option<Self> {
        if content_type == THEME_TYPE || content_type.ends_with(".theme+xml") {
            return Some(Self::Theme);
        }
        if content_type == "application/vnd.openxmlformats-package.core-properties+xml" {
            return Some(Self::Core);
        }
        if content_type.ends_with(".extended-properties+xml") {
            return Some(Self::App);
        }
        if content_type.ends_with(".custom-properties+xml") {
            return Some(Self::Custom);
        }
        let last = content_type.strip_prefix(PML_TYPE)?;
        Some(match last {
            "presentation.main+xml"
            | "slideshow.main+xml"
            | "template.main+xml"
            | "presentation.macroEnabled.main+xml"
            | "slideshow.macroEnabled.main+xml"
            | "template.macroEnabled.main+xml" => Self::Main,
            "slide+xml" => Self::Slide,
            "slideLayout+xml" => Self::Layout,
            "slideMaster+xml" => Self::Master,
            "notesSlide+xml" => Self::Notes,
            "notesMaster+xml" => Self::NotesMaster,
            "handoutMaster+xml" => Self::HandoutMaster,
            "presProps+xml" => Self::PresProps,
            "viewProps+xml" => Self::ViewProps,
            "tableStyles+xml" => Self::TableStyles,
            "commentAuthors+xml" => Self::CommentAuthors,
            "comments+xml" => Self::Comments,
            "tags+xml" => Self::Tags,
            _ => return None,
        })
    }

    /// The conventional folder and stem, for a part with no other evidence.
    fn of_name(name: &str) -> Option<Self> {
        let lower = name.to_ascii_lowercase();
        let (dir, file) = lower.rsplit_once('/').unwrap_or(("", &lower));
        if !file.ends_with(".xml") {
            return None;
        }
        let in_dir = |d: &str| dir == d || dir.ends_with(&format!("/{d}"));
        Some(match file {
            "presentation.xml" => Self::Main,
            "presprops.xml" => Self::PresProps,
            "viewprops.xml" => Self::ViewProps,
            "tablestyles.xml" => Self::TableStyles,
            "commentauthors.xml" => Self::CommentAuthors,
            "core.xml" if in_dir("docprops") => Self::Core,
            "app.xml" if in_dir("docprops") => Self::App,
            "custom.xml" if in_dir("docprops") => Self::Custom,
            _ if in_dir("slides") => Self::Slide,
            _ if in_dir("slidelayouts") => Self::Layout,
            _ if in_dir("slidemasters") => Self::Master,
            _ if in_dir("notesslides") => Self::Notes,
            _ if in_dir("notesmasters") => Self::NotesMaster,
            _ if in_dir("handoutmasters") => Self::HandoutMaster,
            _ if in_dir("theme") => Self::Theme,
            _ if in_dir("comments") => Self::Comments,
            _ if in_dir("tags") => Self::Tags,
            _ => return None,
        })
    }

    fn content_type(self) -> Option<String> {
        let pml = |last: &str| Some(format!("{PML_TYPE}{last}"));
        match self {
            Self::Main => pml("presentation.main+xml"),
            Self::Slide => pml("slide+xml"),
            Self::Layout => pml("slideLayout+xml"),
            Self::Master => pml("slideMaster+xml"),
            Self::Notes => pml("notesSlide+xml"),
            Self::NotesMaster => pml("notesMaster+xml"),
            Self::HandoutMaster => pml("handoutMaster+xml"),
            Self::Theme => Some(THEME_TYPE.to_owned()),
            Self::PresProps => pml("presProps+xml"),
            Self::ViewProps => pml("viewProps+xml"),
            Self::TableStyles => pml("tableStyles+xml"),
            Self::Core => {
                Some("application/vnd.openxmlformats-package.core-properties+xml".to_owned())
            }
            Self::App => Some(
                "application/vnd.openxmlformats-officedocument.extended-properties+xml".to_owned(),
            ),
            Self::Custom => Some(
                "application/vnd.openxmlformats-officedocument.custom-properties+xml".to_owned(),
            ),
            Self::CommentAuthors => pml("commentAuthors+xml"),
            Self::Comments => pml("comments+xml"),
            Self::Tags => pml("tags+xml"),
            Self::Other => None,
        }
    }

    /// The element a slide-like part's root must be.
    fn slide_like_root(self) -> Option<&'static str> {
        match self {
            Self::Slide => Some("sld"),
            Self::Layout => Some("sldLayout"),
            Self::Master => Some("sldMaster"),
            Self::Notes => Some("notes"),
            Self::NotesMaster => Some("notesMaster"),
            Self::HandoutMaster => Some("handoutMaster"),
            _ => None,
        }
    }

    /// The root's children, in schema order: `(local name, rank)`.
    fn root_children(self) -> &'static [(&'static str, u8)] {
        match self {
            Self::Slide => &[
                ("cSld", 0),
                ("clrMapOvr", 1),
                ("transition", 2),
                ("AlternateContent", 2),
                ("timing", 3),
                ("extLst", 4),
            ],
            Self::Layout => &[
                ("cSld", 0),
                ("clrMapOvr", 1),
                ("transition", 2),
                ("AlternateContent", 2),
                ("timing", 3),
                ("hf", 4),
                ("extLst", 5),
            ],
            Self::Master => &[
                ("cSld", 0),
                ("clrMap", 1),
                ("sldLayoutIdLst", 2),
                ("transition", 3),
                ("AlternateContent", 3),
                ("timing", 4),
                ("hf", 5),
                ("txStyles", 6),
                ("extLst", 7),
            ],
            Self::Notes => &[("cSld", 0), ("clrMapOvr", 1), ("extLst", 2)],
            Self::NotesMaster => &[
                ("cSld", 0),
                ("clrMap", 1),
                ("hf", 2),
                ("notesStyle", 3),
                ("extLst", 4),
            ],
            Self::HandoutMaster => &[("cSld", 0), ("clrMap", 1), ("hf", 2), ("extLst", 3)],
            _ => &[],
        }
    }

    fn needs_color_map(self) -> bool {
        matches!(self, Self::Master | Self::NotesMaster | Self::HandoutMaster)
    }
}

const COLOR_MAP: [(&str, &str); 12] = [
    ("bg1", "lt1"),
    ("tx1", "dk1"),
    ("bg2", "lt2"),
    ("tx2", "dk2"),
    ("accent1", "accent1"),
    ("accent2", "accent2"),
    ("accent3", "accent3"),
    ("accent4", "accent4"),
    ("accent5", "accent5"),
    ("accent6", "accent6"),
    ("hlink", "hlink"),
    ("folHlink", "folHlink"),
];

/// The namespaces a new part declares, in the form the file is in.
fn pml_decls(conformance: Conformance) -> String {
    format!(
        "xmlns:a=\"{}\" xmlns:r=\"{}\" xmlns:p=\"{}\"",
        conformance.spell(DML_NS),
        conformance.spell(R_NS),
        conformance.spell(PML_NS)
    )
}

const EMPTY_TREE: &str = "<p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id=\"1\" name=\"\"/>\
    <p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/></p:spTree></p:cSld>";
const COLOR_OVERRIDE: &str = "<p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>";

fn color_map() -> String {
    let mut out = String::from("<p:clrMap");
    for (k, v) in COLOR_MAP {
        out.push_str(&format!(" {k}=\"{v}\""));
    }
    out.push_str("/>");
    out
}

fn solid() -> &'static str {
    "<a:solidFill><a:schemeClr val=\"phClr\"/></a:solidFill>"
}

fn theme_skeleton(conformance: Conformance) -> String {
    let dml = conformance.spell(DML_NS);
    let srgb = |name: &str, hex: &str| format!("<a:{name}><a:srgbClr val=\"{hex}\"/></a:{name}>");
    let colors: String = [
        ("dk1", "000000"),
        ("lt1", "FFFFFF"),
        ("dk2", "44546A"),
        ("lt2", "E7E6E6"),
        ("accent1", "4472C4"),
        ("accent2", "ED7D31"),
        ("accent3", "A5A5A5"),
        ("accent4", "FFC000"),
        ("accent5", "5B9BD5"),
        ("accent6", "70AD47"),
        ("hlink", "0563C1"),
        ("folHlink", "954F72"),
    ]
    .iter()
    .map(|(n, h)| srgb(n, h))
    .collect();
    let font = "<a:latin typeface=\"\"/><a:ea typeface=\"\"/><a:cs typeface=\"\"/>";
    let fill = solid().repeat(3);
    let line = format!("<a:ln>{}</a:ln>", solid()).repeat(3);
    let effect = "<a:effectStyle><a:effectLst/></a:effectStyle>".repeat(3);
    format!(
        "{DECL}<a:theme xmlns:a=\"{dml}\" name=\"Recovered\"><a:themeElements>\
         <a:clrScheme name=\"Recovered\">{colors}</a:clrScheme>\
         <a:fontScheme name=\"Recovered\"><a:majorFont>{font}</a:majorFont>\
         <a:minorFont>{font}</a:minorFont></a:fontScheme>\
         <a:fmtScheme name=\"Recovered\"><a:fillStyleLst>{fill}</a:fillStyleLst>\
         <a:lnStyleLst>{line}</a:lnStyleLst><a:effectStyleLst>{effect}</a:effectStyleLst>\
         <a:bgFillStyleLst>{fill}</a:bgFillStyleLst></a:fmtScheme></a:themeElements></a:theme>"
    )
}

/// A valid, empty part for a role. `None` for a role with no such thing.
fn skeleton(role: Role, conformance: Conformance) -> Option<String> {
    let ns = pml_decls(conformance);
    let map = color_map();
    let spell = |uri: &'static str| conformance.spell(uri).to_owned();
    Some(match role {
        Role::Slide => format!("{DECL}<p:sld {ns}>{EMPTY_TREE}{COLOR_OVERRIDE}</p:sld>"),
        Role::Layout => {
            format!("{DECL}<p:sldLayout {ns}>{EMPTY_TREE}{COLOR_OVERRIDE}</p:sldLayout>")
        }
        Role::Master => {
            format!("{DECL}<p:sldMaster {ns}>{EMPTY_TREE}{map}<p:sldLayoutIdLst/></p:sldMaster>")
        }
        Role::Notes => format!("{DECL}<p:notes {ns}>{EMPTY_TREE}{COLOR_OVERRIDE}</p:notes>"),
        Role::NotesMaster => format!("{DECL}<p:notesMaster {ns}>{EMPTY_TREE}{map}</p:notesMaster>"),
        Role::HandoutMaster => {
            format!("{DECL}<p:handoutMaster {ns}>{EMPTY_TREE}{map}</p:handoutMaster>")
        }
        Role::Theme => theme_skeleton(conformance),
        Role::PresProps => format!("{DECL}<p:presentationPr {ns}/>"),
        Role::ViewProps => format!("{DECL}<p:viewPr {ns}/>"),
        Role::TableStyles => format!(
            "{DECL}<a:tblStyleLst xmlns:a=\"{}\" def=\"{{5C22544A-7EE6-4342-B048-85BDC9FD1C3A}}\"/>",
            spell(DML_NS)
        ),
        Role::Core => format!(
            "{DECL}<cp:coreProperties xmlns:cp=\"{CORE_NS}\" xmlns:dc=\"{DC_NS}\" \
             xmlns:dcterms=\"{DCTERMS_NS}\" xmlns:xsi=\"{XSI_NS}\"/>"
        ),
        Role::App => format!(
            "{DECL}<Properties xmlns=\"{}\" xmlns:vt=\"{}\"/>",
            spell(EP_NS),
            spell(VT_NS)
        ),
        Role::Custom => format!(
            "{DECL}<Properties xmlns=\"{}\" xmlns:vt=\"{}\"/>",
            spell(CP_NS),
            spell(VT_NS)
        ),
        Role::CommentAuthors => format!("{DECL}<p:cmAuthorLst {ns}/>"),
        Role::Comments => format!("{DECL}<p:cmLst {ns}/>"),
        Role::Tags => format!("{DECL}<p:tagLst {ns}/>"),
        Role::Main | Role::Other => return None,
    })
}

/// Bring a recovered slide-like part to the shape its schema requires, around whatever
/// survived: required frames put back, children out of order or unknown left out. `None` if
/// it is not this kind of part at all.
fn normalize_slide_like(bytes: &[u8], role: Role, limits: &OoxmlLimits) -> Option<Vec<u8>> {
    let mut tree = XmlTree::parse(bytes, "part", limits).ok()?;
    let root = tree.root();
    if Some(tree.local_name(root)) != role.slide_like_root() {
        return None;
    }
    let prefix = tree
        .qualified_name(root)
        .split_once(':')
        .map_or("", |(p, _)| p)
        .to_owned();
    let name = |local: &str| {
        if prefix.is_empty() {
            local.to_owned()
        } else {
            format!("{prefix}:{local}")
        }
    };
    keep_in_order(&mut tree, root, role.root_children());
    let csld = match tree.child(root, "cSld") {
        Some(c) => c,
        None => {
            let first = tree.child_elements(root).first().copied();
            tree.insert_element(root, first, &name("cSld"), &[])
        }
    };
    keep_in_order(
        &mut tree,
        csld,
        &[
            ("bg", 0),
            ("spTree", 1),
            ("custDataLst", 2),
            ("controls", 3),
            ("extLst", 4),
        ],
    );
    let sptree = match tree.child(csld, "spTree") {
        Some(s) => s,
        None => {
            let before = tree
                .child_elements(csld)
                .into_iter()
                .find(|c| tree.local_name(*c) != "bg");
            tree.insert_element(csld, before, &name("spTree"), &[])
        }
    };
    keep_in_order(
        &mut tree,
        sptree,
        &[
            ("nvGrpSpPr", 0),
            ("grpSpPr", 1),
            ("sp", 2),
            ("grpSp", 2),
            ("graphicFrame", 2),
            ("cxnSp", 2),
            ("pic", 2),
            ("contentPart", 2),
            ("AlternateContent", 2),
            ("extLst", 3),
        ],
    );
    if tree.child(sptree, "grpSpPr").is_none() {
        let before = tree
            .child_elements(sptree)
            .into_iter()
            .find(|c| tree.local_name(*c) != "nvGrpSpPr");
        tree.insert_element(sptree, before, &name("grpSpPr"), &[]);
    }
    if tree.child(sptree, "nvGrpSpPr").is_none() {
        let first = tree.child_elements(sptree).first().copied();
        let nv = tree.insert_element(sptree, first, &name("nvGrpSpPr"), &[]);
        tree.insert_element(nv, None, &name("cNvPr"), &[("id", "1"), ("name", "")]);
        tree.insert_element(nv, None, &name("cNvGrpSpPr"), &[]);
        tree.insert_element(nv, None, &name("nvPr"), &[]);
    }
    if role.needs_color_map() && tree.child(root, "clrMap").is_none() {
        let after = tree
            .child_elements(root)
            .into_iter()
            .find(|c| tree.local_name(*c) != "cSld");
        tree.insert_element(root, after, &name("clrMap"), &COLOR_MAP);
    }
    Some(tree.to_bytes())
}

/// Keep the children `order` names, in its order, and the first of a rank that repeats only
/// where the rank is one that may repeat. Everything else goes.
fn keep_in_order(tree: &mut XmlTree, parent: ElementId, order: &[(&str, u8)]) {
    let mut last_rank: Option<u8> = None;
    let mut seen: BTreeSet<String> = BTreeSet::new();
    // Rank 2 under spTree is the one that repeats; elsewhere each name is once.
    let repeats = order.iter().filter(|(_, r)| *r == 2).count() > 3;
    for child in tree.child_elements(parent) {
        let local = tree.local_name(child).to_owned();
        let rank = order.iter().find(|(n, _)| *n == local).map(|(_, r)| *r);
        let keep = rank.is_some_and(|r| {
            let in_order = last_rank.is_none_or(|l| r > l || (r == l && repeats));
            in_order && (repeats && r == 2 || seen.insert(local.clone()))
        });
        if keep {
            last_rank = rank;
        } else {
            tree.remove_child(parent, child);
        }
    }
}

/// Take every element in one of `namespaces` out of the tree. No recursion on the input.
fn remove_elements_in(tree: &mut XmlTree, namespaces: &BTreeSet<String>) {
    let mut stack = vec![tree.root()];
    while let Some(id) = stack.pop() {
        for child in tree.child_elements(id) {
            let inside = tree
                .namespace(child)
                .is_some_and(|n| namespaces.contains(&n));
            if inside {
                tree.remove_child(id, child);
            } else {
                stack.push(child);
            }
        }
    }
}

/// Drop every `mc:MustUnderstand`, by whichever prefix it has.
fn remove_must_understand(tree: &mut XmlTree) {
    let mut stack = vec![tree.root()];
    while let Some(id) = stack.pop() {
        let names: Vec<String> = tree
            .declared_attributes(id)
            .into_iter()
            .map(|(n, _)| n)
            .collect();
        for name in names {
            if let Some((prefix, "MustUnderstand")) = name.split_once(':') {
                let bound = tree.prefix_for_namespace(id, MARKUP_COMPATIBILITY);
                if bound.as_deref() == Some(prefix) {
                    tree.remove_attribute(id, &name);
                }
            }
        }
        stack.extend(tree.child_elements(id));
    }
}

/// `walk` admits it, and the declaration does not lie about the encoding.
pub(crate) fn part_is_clean(bytes: &[u8], name: &str, limits: &OoxmlLimits) -> bool {
    !declaration_disagrees(bytes) && walk(bytes, name, limits, |_, _| Ok(())).is_ok()
}

fn must_understand_namespaces(bytes: &[u8], name: &str, limits: &OoxmlLimits) -> BTreeSet<String> {
    // Cheap to rule out, and almost no part has it.
    let needle: &[u8] = b"MustUnderstand";
    if !bytes.windows(needle.len()).any(|w| w == needle) {
        return BTreeSet::new();
    }
    XmlTree::parse(bytes, name, limits)
        .map(|tree| resolve_compatibility(&tree).must_understand)
        .map(|set| set.into_iter().collect())
        .unwrap_or_default()
}

/// Whether a part holds what the strict reader refuses and a repair must mend.
pub(crate) fn part_needs_repair(bytes: &[u8], name: &str, limits: &OoxmlLimits) -> bool {
    !part_is_clean(bytes, name, limits)
        || !must_understand_namespaces(bytes, name, limits).is_empty()
}

fn is_xml_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.ends_with(".xml") || lower.ends_with(".rels")
}

/// `name`'s part is one XML is expected of, and the bytes are good: which of two entries
/// naming one part to read.
pub(crate) fn entry_is_good(name: &str, bytes: &[u8], limits: &OoxmlLimits) -> bool {
    if !is_xml_name(name) {
        return true;
    }
    let mut root = None;
    let clean = !declaration_disagrees(bytes)
        && walk(bytes, name, limits, |e, depth| {
            if depth == 1 {
                root = Some(String::from_utf8_lossy(e.local_name().as_ref()).into_owned());
            }
            Ok(())
        })
        .is_ok();
    // A name that says what a part is, and a root that says otherwise, is not the part.
    clean
        && Role::of_name(name).is_none_or(|expected| {
            root.as_deref()
                .and_then(Role::of_root)
                .is_none_or(|found| found == expected)
                && root.as_deref().is_some_and(|r| Role::of_root(r).is_some())
        })
}

fn rel_kind(rel_type: &str) -> Option<&str> {
    rel_type
        .strip_prefix(TRANSITIONAL_RELS)
        .or_else(|| rel_type.strip_prefix(STRICT_RELS))
}

/// Digits compare as numbers, so `slide10` follows `slide9`.
fn natural_key(name: &str) -> (Vec<(String, u64)>, String) {
    let mut out = Vec::new();
    let mut text = String::new();
    let mut digits = String::new();
    for c in name.chars() {
        if c.is_ascii_digit() {
            digits.push(c);
        } else {
            if !digits.is_empty() {
                out.push((
                    std::mem::take(&mut text),
                    digits.parse().unwrap_or(u64::MAX),
                ));
                digits.clear();
            }
            text.push(c);
        }
    }
    out.push((text, digits.parse().unwrap_or(0)));
    (out, name.to_owned())
}

fn relative_target(source: &str, part: &str) -> String {
    let from: Vec<&str> = source.split('/').collect();
    let to: Vec<&str> = part.split('/').collect();
    let dirs = from.len().saturating_sub(1);
    let common = from
        .iter()
        .take(dirs)
        .zip(to.iter())
        .take_while(|(a, b)| a.eq_ignore_ascii_case(b))
        .count();
    let mut out = "../".repeat(dirs - common);
    out.push_str(&to.get(common..).unwrap_or_default().join("/"));
    out
}

fn is_ncname(id: &str) -> bool {
    let mut chars = id.chars();
    chars.next().is_some_and(|c| c.is_alphabetic() || c == '_')
        && chars.all(|c| c.is_alphanumeric() || matches!(c, '_' | '-' | '.'))
}

/// A token of a content type, by OPC's grammar.
fn is_type_token(token: &str) -> bool {
    !token.is_empty()
        && token.bytes().all(|b| {
            b.is_ascii_graphic()
                && !matches!(
                    b,
                    b'(' | b')'
                        | b'<'
                        | b'>'
                        | b'@'
                        | b','
                        | b';'
                        | b':'
                        | b'\\'
                        | b'"'
                        | b'/'
                        | b'['
                        | b']'
                        | b'?'
                        | b'='
                        | b'{'
                        | b'}'
                )
        })
}

fn is_content_type(value: &str) -> bool {
    value
        .split_once('/')
        .is_some_and(|(a, b)| is_type_token(a) && is_type_token(b))
}

fn is_extension(ext: &str) -> bool {
    !ext.is_empty()
        && ext.bytes().all(|b| {
            b.is_ascii_alphanumeric()
                || matches!(
                    b,
                    b'!' | b'$'
                        | b'&'
                        | b'\''
                        | b'('
                        | b')'
                        | b'*'
                        | b'+'
                        | b','
                        | b':'
                        | b'='
                        | b'@'
                        | b'-'
                        | b'_'
                        | b'~'
                )
        })
}

/// A content type for a file with this extension, with no other evidence.
fn type_for_extension(ext: &str) -> Option<&'static str> {
    Some(match ext {
        "rels" => RELS_TYPE,
        "xml" => "application/xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        "tif" | "tiff" => "image/tiff",
        "svg" => "image/svg+xml",
        "emf" => "image/x-emf",
        "wmf" => "image/x-wmf",
        "wdp" => "image/vnd.ms-photo",
        "mp4" | "m4v" => "video/mp4",
        "mp3" => "audio/mpeg",
        "m4a" => "audio/mp4",
        "wav" => "audio/wav",
        "wmv" => "video/x-ms-wmv",
        "bin" => "application/vnd.openxmlformats-officedocument.oleObject",
        "vml" => "application/vnd.openxmlformats-officedocument.vmlDrawing",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "fntdata" => "application/x-fontdata",
        "sigs" => "application/vnd.openxmlformats-package.digital-signature-origin",
        RETAINED_EXTENSION => "application/octet-stream",
        _ => return None,
    })
}

/// What repairing produced.
#[derive(Debug)]
pub(crate) struct Repaired {
    /// A clean package.
    pub image: Vec<u8>,
    /// What was done.
    pub findings: BTreeSet<Finding>,
    /// Parts whose content is not what arrived, in the name the image gives them.
    pub damaged: BTreeSet<String>,
}

/// Repair everything that needs it in a sound container. `aside` names parts a reader above
/// could not use: they are set aside whatever they hold.
pub(crate) fn repair_package(
    package: &mut Package,
    limits: &OoxmlLimits,
    aside: &BTreeSet<String>,
    list_cap: usize,
) -> Result<Repaired, OoxmlError> {
    let names: Vec<String> = package
        .entry_names()
        .into_iter()
        .filter(|n| !n.ends_with('/'))
        .collect();
    let mut keys = BTreeMap::new();
    for name in &names {
        keys.insert(entry_key(name), name.clone());
    }
    let work = Work {
        package,
        limits,
        names,
        keys,
        out: BTreeMap::new(),
        gone: BTreeSet::new(),
        rels: BTreeMap::new(),
        rels_dirty: BTreeSet::new(),
        types: ContentTypes::default(),
        types_dirty: false,
        conformance: Conformance::Transitional,
        findings: BTreeSet::new(),
        damaged: BTreeSet::new(),
        roots: BTreeMap::new(),
        link_roles: BTreeMap::new(),
        retained_count: 0,
        total_rels: 0,
        next_master_id: FIRST_MASTER_ID,
        list_cap,
    };
    work.run(aside)
}

/// A damaged part's bytes, and whether any could be read. `None` is a part over a bound.
type DamagedParts = BTreeMap<String, Option<Vec<u8>>>;

struct Work<'p> {
    package: &'p mut Package,
    limits: &'p OoxmlLimits,
    names: Vec<String>,
    keys: BTreeMap<String, String>,
    out: BTreeMap<String, Vec<u8>>,
    gone: BTreeSet<String>,
    rels: BTreeMap<String, Vec<Relationship>>,
    rels_dirty: BTreeSet<String>,
    types: ContentTypes,
    types_dirty: bool,
    conformance: Conformance,
    findings: BTreeSet<Finding>,
    damaged: BTreeSet<String>,
    /// Root elements of parts that parsed.
    roots: BTreeMap<String, String>,
    /// The role each relationship gives its target, by key. First link wins.
    link_roles: BTreeMap<String, Role>,
    retained_count: usize,
    total_rels: usize,
    next_master_id: u64,
    /// The most a rebuilt list of slides or masters may hold.
    list_cap: usize,
}

impl Work<'_> {
    fn note(&mut self, repair: Repair, part: &str) {
        self.findings.insert(Finding {
            kind: FindingKind::Repaired(repair),
            part: scrub_name(part),
            rel_id: None,
        });
    }

    fn note_xml(&mut self, repairs: &BTreeSet<XmlRepair>, part: &str) {
        for repair in repairs {
            self.note(Repair::Xml(*repair), part);
        }
    }

    fn actual(&self, name: &str) -> Option<String> {
        self.keys
            .get(&entry_key(name))
            .filter(|n| !self.gone.contains(*n))
            .cloned()
    }

    fn exists(&self, name: &str) -> bool {
        self.actual(name).is_some()
    }

    /// Read a part for the repair. `None` if it is over a bound or will not read.
    fn read(&mut self, name: &str) -> Option<Vec<u8>> {
        if let Some(bytes) = self.out.get(name) {
            return Some(bytes.clone());
        }
        self.package
            .read_part_capped(name, self.limits.max_xml_part_bytes as u64)
            .ok()
    }

    fn run(mut self, aside: &BTreeSet<String>) -> Result<Repaired, OoxmlError> {
        self.read_types();
        self.read_all_rels();
        let damaged = self.scan_parts(aside);
        let main = self.locate_main(&damaged);
        self.index_link_roles();
        self.apply_damage(damaged, &main, aside);
        self.fix_main(&main);
        self.fix_chain(&main);
        self.finish_types(&main);
        self.finish_rels();

        let mut edits = PartEdits::default();
        for name in &self.gone {
            edits.remove(name);
        }
        for (name, bytes) in std::mem::take(&mut self.out) {
            edits.replace(name, bytes);
        }
        let image = self.package.rewrite(&edits)?;
        Ok(Repaired {
            image,
            findings: self.findings,
            damaged: self.damaged,
        })
    }

    fn read_types(&mut self) {
        let Some(actual) = self.actual(CONTENT_TYPES_PART) else {
            self.types_dirty = true;
            self.note(Repair::ContentTypesRebuilt, CONTENT_TYPES_PART);
            return;
        };
        let bytes = self.read(&actual);
        let strict = bytes
            .as_deref()
            .filter(|b| !declaration_disagrees(b))
            .and_then(|b| ContentTypes::parse(b, self.limits).ok());
        if let Some(types) = strict {
            self.types = types;
            return;
        }
        self.types_dirty = true;
        self.note(Repair::ContentTypesRebuilt, CONTENT_TYPES_PART);
        let Some(bytes) = bytes else { return };
        let recovered = recover_xml(&bytes, self.limits, Prune::Nothing);
        self.note_xml(&recovered.repairs, CONTENT_TYPES_PART);
        let Ok(tree) = XmlTree::parse(&recovered.bytes, CONTENT_TYPES_PART, self.limits) else {
            return;
        };
        let mut count = 0usize;
        for child in tree.child_elements(tree.root()) {
            if count >= self.limits.max_content_types {
                self.note(Repair::Xml(XmlRepair::LimitStopped), CONTENT_TYPES_PART);
                break;
            }
            let Some(value) = tree
                .plain_attribute(child, "ContentType")
                .filter(|v| is_content_type(v))
            else {
                continue;
            };
            let added = match tree.local_name(child) {
                "Default" => tree
                    .plain_attribute(child, "Extension")
                    .filter(|e| is_extension(e))
                    .is_some_and(|e| self.types.add_default(&e, &value)),
                "Override" => tree
                    .plain_attribute(child, "PartName")
                    .is_some_and(|p| self.types.add_override(&p, &value)),
                _ => false,
            };
            count += usize::from(added);
        }
    }

    fn read_all_rels(&mut self) {
        let names: Vec<String> = self.names.clone();
        for name in &names {
            let key = entry_key(name);
            let Some(source_key) = source_of_rels(&key) else {
                continue;
            };
            let source = if source_key.is_empty() {
                String::new()
            } else {
                match self.keys.get(&source_key) {
                    Some(actual) => actual.clone(),
                    // Relationships of nothing. If it reads it is left alone, and if not
                    // it is set aside like any other part.
                    None => {
                        let bad = self.read(name).is_none_or(|b| {
                            parse_relationships(&b, name, self.limits).is_err()
                                || declaration_disagrees(&b)
                        });
                        if bad {
                            self.set_aside_bytes(name);
                            self.gone.insert(name.clone());
                            self.note(Repair::PartSetAside, name);
                        }
                        continue;
                    }
                }
            };
            let bytes = self.read(name);
            let strict = bytes
                .as_deref()
                .filter(|b| !declaration_disagrees(b))
                .and_then(|b| parse_relationships(b, name, self.limits).ok());
            let mut list = match strict {
                Some(list) => list,
                None => {
                    self.rels_dirty.insert(source.clone());
                    self.note(Repair::RelationshipsRebuilt, name);
                    self.set_aside_bytes(name);
                    self.tolerant_rels(bytes.as_deref(), name)
                }
            };
            // Over the whole-package bound, what fits is kept.
            let room = self
                .limits
                .max_total_relationships
                .saturating_sub(self.total_rels);
            if list.len() > room {
                list.truncate(room);
                self.rels_dirty.insert(source.clone());
                self.note(Repair::Xml(XmlRepair::LimitStopped), name);
            }
            self.total_rels = self.total_rels.saturating_add(list.len());
            self.rels.insert(source, list);
        }
    }

    fn tolerant_rels(&mut self, bytes: Option<&[u8]>, name: &str) -> Vec<Relationship> {
        let Some(bytes) = bytes else {
            return Vec::new();
        };
        let recovered = recover_xml(bytes, self.limits, Prune::Nothing);
        self.note_xml(&recovered.repairs, name);
        let Ok(tree) = XmlTree::parse(&recovered.bytes, name, self.limits) else {
            return Vec::new();
        };
        let mut out: Vec<Relationship> = Vec::new();
        let mut ids = BTreeSet::new();
        for child in tree.child_elements(tree.root()) {
            if out.len() >= self.limits.max_part_relationships {
                self.note(Repair::Xml(XmlRepair::LimitStopped), name);
                break;
            }
            if tree.local_name(child) != "Relationship" {
                continue;
            }
            let (Some(id), Some(rel_type), Some(target)) = (
                tree.plain_attribute(child, "Id"),
                tree.plain_attribute(child, "Type"),
                tree.plain_attribute(child, "Target"),
            ) else {
                continue;
            };
            if !is_ncname(&id) || !ids.insert(id.clone()) || target.is_empty() {
                continue;
            }
            // An unknown mode is never believed to be inside the package.
            let mode = match tree.plain_attribute(child, "TargetMode").as_deref() {
                None | Some("Internal") => TargetMode::Internal,
                Some(_) => TargetMode::External,
            };
            out.push(Relationship {
                id,
                rel_type,
                target,
                mode,
            });
        }
        out
    }

    /// Find every part that needs mending, and the root of every part that does not.
    fn scan_parts(&mut self, aside: &BTreeSet<String>) -> DamagedParts {
        let mut damaged = DamagedParts::new();
        let names = self.names.clone();
        for name in &names {
            if self.gone.contains(name) || !is_xml_name(name) {
                continue;
            }
            let key = entry_key(name);
            if key == entry_key(CONTENT_TYPES_PART) || source_of_rels(&key).is_some() {
                continue;
            }
            let forced = aside.contains(name);
            let Some(bytes) = self.read(name) else {
                damaged.insert(name.clone(), None);
                continue;
            };
            if forced {
                damaged.insert(name.clone(), Some(bytes));
                continue;
            }
            let mut root = None;
            let clean = !declaration_disagrees(&bytes)
                && walk(&bytes, name, self.limits, |e, depth| {
                    if depth == 1 {
                        root = Some(String::from_utf8_lossy(e.local_name().as_ref()).into_owned());
                    }
                    Ok(())
                })
                .is_ok();
            if !clean {
                damaged.insert(name.clone(), Some(bytes));
                continue;
            }
            if let Some(root) = root {
                self.roots.insert(name.clone(), root);
            }
            if !must_understand_namespaces(&bytes, name, self.limits).is_empty() {
                damaged.insert(name.clone(), Some(bytes));
            }
        }
        damaged
    }

    fn role_of(&self, name: &str, main: &str) -> Role {
        if entry_key(name) == entry_key(main) {
            return Role::Main;
        }
        if let Some(role) = self.link_roles.get(&entry_key(name)) {
            return *role;
        }
        if let Some(role) = self.types.resolve(name).and_then(Role::of_content_type) {
            return role;
        }
        if let Some(role) = self.roots.get(name).and_then(|r| Role::of_root(r)) {
            return role;
        }
        Role::of_name(name).unwrap_or(Role::Other)
    }

    /// What each relationship says its target is. Bounded by the relationship limit, and
    /// looked up once per part afterward, so a flood of parts and links stays linear.
    fn index_link_roles(&mut self) {
        let mut roles = BTreeMap::new();
        for (source, list) in &self.rels {
            for r in list {
                if r.mode != TargetMode::Internal {
                    continue;
                }
                let Some(role) = Role::of_rel(&r.rel_type) else {
                    continue;
                };
                if let Some(key) =
                    resolve_internal(source, &r.target).and_then(|t| part_name_key(&t))
                {
                    roles.entry(key).or_insert(role);
                }
            }
        }
        self.link_roles = roles;
    }

    /// Choose the main part. Exact relationship namespaces, then the conventional name, then
    /// a type or root that says so; a part that is something else is not believed.
    fn locate_main(&mut self, damaged: &DamagedParts) -> String {
        let root_rels: Vec<Relationship> = self.rels.get("").cloned().unwrap_or_default();
        let is_main_rel = |r: &Relationship| {
            r.mode == TargetMode::Internal && rel_kind(&r.rel_type) == Some("officeDocument")
        };
        // What the relationship names, if the part there can be a presentation.
        let named = root_rels.iter().find(|r| is_main_rel(r)).and_then(|r| {
            let strict = r.rel_type.starts_with(STRICT_RELS);
            resolve_internal("", &r.target).map(|t| (t, strict))
        });
        if let Some((target, strict)) = &named {
            self.conformance = if *strict {
                Conformance::Strict
            } else {
                Conformance::Transitional
            };
            let usable = part_name_key(target).is_some_and(|k| match self.keys.get(&k).cloned() {
                Some(actual) => {
                    damaged.contains_key(&actual)
                        || self.roots.get(&actual).is_some_and(|r| r == "presentation")
                }
                None => is_safe_new_name(target),
            });
            if usable {
                return self.actual(target).unwrap_or_else(|| target.clone());
            }
        }
        let names = self.names.clone();
        let found = names
            .iter()
            .filter(|n| !self.gone.contains(*n))
            .find(|n| entry_key(n) == "ppt/presentation.xml" && !self.is_not_a_presentation(n))
            .or_else(|| {
                names.iter().filter(|n| !self.gone.contains(*n)).find(|n| {
                    self.roots.get(*n).is_some_and(|r| r == "presentation")
                        || self.types.resolve(n).and_then(Role::of_content_type) == Some(Role::Main)
                            && !self.is_not_a_presentation(n)
                })
            })
            .cloned();
        match found {
            Some(main) => {
                self.note(Repair::MainPartFound, &main);
                self.set_main_rel(&main);
                self.conformance = self.conformance_from_rels(&main);
                main
            }
            None => {
                let main = self.free_name("ppt/presentation", "xml");
                self.conformance = self.conformance_from_rels(&main);
                self.note(Repair::MainPartBuilt, &main);
                self.set_main_rel(&main);
                main
            }
        }
    }

    /// A part with a root, and that root is not a presentation's.
    fn is_not_a_presentation(&self, name: &str) -> bool {
        self.roots.get(name).is_some_and(|r| r != "presentation")
    }

    /// With no main relationship to say, the form of the standard shows in any part.
    fn conformance_from_rels(&self, main: &str) -> Conformance {
        let strict_type = self
            .rels
            .values()
            .flatten()
            .any(|r| r.rel_type.starts_with(STRICT_RELS));
        if strict_type {
            return Conformance::Strict;
        }
        let _ = main;
        Conformance::Transitional
    }

    fn set_main_rel(&mut self, main: &str) {
        let spelled = self.rel_type("officeDocument");
        let mut list = self.rels.remove("").unwrap_or_default();
        list.retain(|r| rel_kind(&r.rel_type) != Some("officeDocument"));
        let id = next_rel_id(&list);
        list.push(Relationship {
            id,
            rel_type: spelled,
            target: main.to_owned(),
            mode: TargetMode::Internal,
        });
        self.rels.insert(String::new(), list);
        self.rels_dirty.insert(String::new());
    }

    fn rel_type(&self, kind: &str) -> String {
        match self.conformance {
            Conformance::Transitional => format!("{TRANSITIONAL_RELS}{kind}"),
            Conformance::Strict => format!("{STRICT_RELS}{kind}"),
        }
    }

    /// A name that is not taken, as `stem` + a number + `.ext`, or `stem.ext` if free.
    fn free_name(&self, stem: &str, ext: &str) -> String {
        let bare = format!("{stem}.{ext}");
        if !self.keys.contains_key(&entry_key(&bare)) {
            return bare;
        }
        (1u64..)
            .map(|n| format!("{stem}{n}.{ext}"))
            .find(|n| !self.keys.contains_key(&entry_key(n)))
            .unwrap_or(bare)
    }

    fn numbered_name(&self, stem: &str, ext: &str) -> String {
        (1u64..)
            .map(|n| format!("{stem}{n}.{ext}"))
            .find(|n| !self.keys.contains_key(&entry_key(n)))
            .unwrap_or_else(|| format!("{stem}.{ext}"))
    }

    /// Keep a part's bytes as they stand, as a retained entry nothing reaches.
    fn set_aside_bytes(&mut self, name: &str) {
        let Some(actual) = self.actual(name) else {
            return;
        };
        let bytes = match self
            .package
            .read_part_capped(&actual, self.limits.max_xml_part_bytes as u64)
        {
            Ok(bytes) => Some(bytes),
            // Over the bound, so not decompressed: its compressed bytes are what is kept.
            Err(_) => self.package.compressed_bytes(&actual).ok(),
        };
        if let Some(bytes) = bytes {
            self.retain(&actual, bytes);
        }
    }

    fn retain(&mut self, original: &str, bytes: Vec<u8>) {
        let mut ordinal = self.names.len().saturating_add(self.retained_count);
        let name = loop {
            let candidate = retained_name(ordinal, original);
            if !self.keys.contains_key(&entry_key(&candidate)) {
                break candidate;
            }
            ordinal = ordinal.saturating_add(1);
        };
        self.retained_count = self.retained_count.saturating_add(1);
        self.keys.insert(entry_key(&name), name.clone());
        self.out.insert(name, bytes);
    }

    fn apply_damage(&mut self, damaged: DamagedParts, main: &str, aside: &BTreeSet<String>) {
        for (name, bytes) in damaged {
            let forced = aside.contains(&name);
            self.set_aside_bytes(&name);
            self.damaged.insert(name.clone());
            let role = self.role_of(&name, main);
            if role == Role::Main {
                // Mended with the lists, once it is known what they should hold.
                continue;
            }
            let understood = bytes
                .as_deref()
                .filter(|_| !forced)
                .filter(|b| part_is_clean(b, &name, self.limits))
                .is_some();
            if understood && let Some(bytes) = &bytes {
                self.mend_must_understand(&name, bytes);
                continue;
            }
            let recovered = bytes
                .as_deref()
                .filter(|_| !forced)
                .map(|b| recover_xml(b, self.limits, prune_for(role)));
            if let Some(recovered) = &recovered {
                self.note_xml(&recovered.repairs, &name);
            }
            let served = recovered.as_ref().and_then(|r| {
                role.slide_like_root()?;
                normalize_slide_like(&r.bytes, role, self.limits)
            });
            if let Some(served) = served {
                self.out.insert(name.clone(), served);
                continue;
            }
            if let Some(stand_in) = skeleton(role, self.conformance) {
                self.out.insert(name.clone(), stand_in.into_bytes());
                let what = if role == Role::Slide {
                    Repair::SlideEmpty
                } else {
                    Repair::PartSetAside
                };
                self.note(what, &name);
                continue;
            }
            // A part with no stand-in: readable only if nothing validates its namespace.
            let unbound = recovered.as_ref().filter(|r| r.has_root()).filter(|r| {
                XmlTree::parse(&r.bytes, &name, self.limits)
                    .ok()
                    .is_some_and(|t| t.namespace(t.root()).is_none_or(|ns| !is_understood(&ns)))
            });
            if let Some(unbound) = unbound {
                self.out.insert(name.clone(), unbound.bytes.clone());
            } else {
                self.remove_part(&name);
                self.note(Repair::PartSetAside, &name);
            }
        }
    }

    fn mend_must_understand(&mut self, name: &str, bytes: &[u8]) {
        let namespaces = must_understand_namespaces(bytes, name, self.limits);
        let Ok(mut tree) = XmlTree::parse(bytes, name, self.limits) else {
            return;
        };
        remove_must_understand(&mut tree);
        remove_elements_in(&mut tree, &namespaces);
        self.note(Repair::MustUnderstandSetAside, name);
        self.out.insert(name.to_owned(), tree.to_bytes());
    }

    /// Take a part out of the image, with its `.rels` and every link to it.
    fn remove_part(&mut self, name: &str) {
        self.gone.insert(name.to_owned());
        self.out.remove(name);
        let key = entry_key(name);
        let mut dirtied = Vec::new();
        for (source, list) in &mut self.rels {
            let before = list.len();
            list.retain(|r| {
                r.mode != TargetMode::Internal
                    || resolve_internal(source, &r.target)
                        .and_then(|t| part_name_key(&t))
                        .is_none_or(|k| k != key)
            });
            if list.len() != before {
                dirtied.push(source.clone());
            }
        }
        self.rels_dirty.extend(dirtied);
        if let Some(rels_name) = self.actual(&rels_part_for(name)) {
            self.gone.insert(rels_name);
        }
        self.rels.remove(name);
        self.rels_dirty.remove(name);
    }

    /// Parts of a role that exist, in the order a person would read them.
    fn discover(&self, role: Role, main: &str) -> Vec<String> {
        let mut found: Vec<String> = self
            .keys
            .values()
            .filter(|n| !self.gone.contains(*n) && is_xml_name(n) && !n.starts_with(RETAINED_DIR))
            .filter(|n| source_of_rels(&entry_key(n)).is_none())
            .filter(|n| self.role_of(n, main) == role)
            .cloned()
            .collect();
        found.sort_by_key(|n| natural_key(n));
        found
    }

    /// The part a relationship reaches, if it is one that exists.
    fn target_of(&self, source: &str, r: &Relationship) -> Option<String> {
        if r.mode != TargetMode::Internal {
            return None;
        }
        let target = resolve_internal(source, &r.target)?;
        part_name_key(&target)?;
        self.actual(&target)
    }

    fn rels_of(&self, source: &str) -> Vec<Relationship> {
        self.rels.get(source).cloned().unwrap_or_default()
    }

    fn targets(&self, source: &str, kind: &str) -> Vec<String> {
        self.rels_of(source)
            .iter()
            .filter(|r| rel_kind(&r.rel_type) == Some(kind))
            .filter_map(|r| self.target_of(source, r))
            .collect()
    }

    /// Add a relationship unless one of this kind to this part is there. Its id.
    fn link(&mut self, source: &str, kind: &str, part: &str) -> String {
        let mut list = self.rels.remove(source).unwrap_or_default();
        let existing = list.iter().find(|r| {
            rel_kind(&r.rel_type) == Some(kind)
                && r.mode == TargetMode::Internal
                && resolve_internal(source, &r.target).and_then(|t| part_name_key(&t))
                    == part_name_key(part)
        });
        let id = match existing {
            Some(r) => r.id.clone(),
            None => {
                let id = next_rel_id(&list);
                if let (Some(role), Some(key)) = (Role::of_rel_kind(kind), part_name_key(part)) {
                    self.link_roles.entry(key).or_insert(role);
                }
                list.push(Relationship {
                    id: id.clone(),
                    rel_type: self.rel_type(kind),
                    target: relative_target(source, part),
                    mode: TargetMode::Internal,
                });
                self.rels_dirty.insert(source.to_owned());
                id
            }
        };
        self.rels.insert(source.to_owned(), list);
        id
    }

    /// Make a new part from its role's stand-in.
    fn create_part(&mut self, name: &str, role: Role) {
        let bytes = skeleton(role, self.conformance).unwrap_or_default();
        self.keys.insert(entry_key(name), name.to_owned());
        self.out.insert(name.to_owned(), bytes.into_bytes());
    }

    /// Bring the main part and its lists to what exists.
    fn fix_main(&mut self, main: &str) {
        let main_damaged = self.damaged.contains(main);
        let exists = self.exists(main);
        let bytes = if exists { self.read(main) } else { None };

        // Mended of a must-understand alone, it is still whole.
        let clean_tree = bytes
            .as_deref()
            .filter(|b| part_is_clean(b, main, self.limits))
            .and_then(|b| {
                let mut tree = XmlTree::parse(b, main, self.limits).ok()?;
                if tree.local_name(tree.root()) != "presentation" {
                    return None;
                }
                if !must_understand_namespaces(b, main, self.limits).is_empty() {
                    let namespaces = must_understand_namespaces(b, main, self.limits);
                    remove_must_understand(&mut tree);
                    remove_elements_in(&mut tree, &namespaces);
                    ensure_required_main_children(&mut tree);
                    self.findings.insert(Finding {
                        kind: FindingKind::Repaired(Repair::MustUnderstandSetAside),
                        part: scrub_name(main),
                        rel_id: None,
                    });
                    self.out.insert(main.to_owned(), tree.to_bytes());
                }
                Some(tree)
            });

        let main_rels_rebuilt = self.rels_dirty.contains(main) || !self.rels.contains_key(main);
        if let Some(tree) = clean_tree.filter(|_| !main_damaged || self.out.contains_key(main)) {
            let unresolved = self.unresolved_list_entries(&tree, main);
            if unresolved.is_empty() && !main_rels_rebuilt {
                return;
            }
            if !main_rels_rebuilt {
                self.drop_list_entries(tree, main, &unresolved);
                return;
            }
            // The relationships are gone, so what its ids meant is too: its lists are
            // made again, and everything else in the part stays.
            self.rewrite_lists(tree, main);
            return;
        }
        // Nothing whole to keep. The reading tells the order of the slides, and sizes.
        let reading = bytes
            .as_deref()
            .map(|b| recover_xml(b, self.limits, Prune::Nothing));
        if let Some(reading) = &reading {
            self.note_xml(&reading.repairs, main);
        }
        let tree = reading
            .as_ref()
            .filter(|r| r.has_root())
            .and_then(|r| XmlTree::parse(&r.bytes, main, self.limits).ok())
            .filter(|t| t.local_name(t.root()) == "presentation");
        self.build_main(tree.as_ref(), main);
    }

    /// `r:id`s on the lists with no relationship.
    fn unresolved_list_entries(&self, tree: &XmlTree, main: &str) -> Vec<ElementId> {
        let rels = self.rels_of(main);
        let mut out = Vec::new();
        for (list, entry) in [
            ("sldMasterIdLst", "sldMasterId"),
            ("notesMasterIdLst", "notesMasterId"),
            ("handoutMasterIdLst", "handoutMasterId"),
            ("sldIdLst", "sldId"),
        ] {
            let Some(list) = tree.child(tree.root(), list) else {
                continue;
            };
            for el in tree.child_elements(list) {
                if tree.local_name(el) != entry {
                    continue;
                }
                let id = rel_id_of(tree, el);
                if !id.is_some_and(|id| rels.iter().any(|r| r.id == id)) {
                    out.push(el);
                }
            }
        }
        out
    }

    fn drop_list_entries(&mut self, mut tree: XmlTree, main: &str, entries: &[ElementId]) {
        for &el in entries {
            if let Some(parent) = tree.parent_element(el) {
                tree.remove_child(parent, el);
            }
        }
        self.note(Repair::ListEntryDropped, main);
        self.out.insert(main.to_owned(), tree.to_bytes());
        self.damaged.insert(main.to_owned());
    }

    /// The parts its lists should hold, by discovery: the kinds in the deck, in a person's
    /// order, ahead of what the part once said.
    fn rebuild_main_rels(&mut self, main: &str, order: &[String]) -> (Vec<String>, Vec<String>) {
        let masters = self.discover(Role::Master, main);
        let mut slides = self.discover(Role::Slide, main);
        // The order the old list held, where the part still says.
        let position = |name: &String| {
            order
                .iter()
                .position(|o| entry_key(o) == entry_key(name))
                .unwrap_or(usize::MAX)
        };
        slides.sort_by_key(|n| (position(n), natural_key(n)));
        slides.truncate(self.list_cap);
        let masters: Vec<String> = masters.into_iter().take(self.list_cap).collect();
        let mut master_ids = Vec::with_capacity(masters.len());
        for m in &masters {
            master_ids.push(self.link(main, "slideMaster", m));
        }
        let mut slide_ids = Vec::with_capacity(slides.len());
        for s in &slides {
            slide_ids.push(self.link(main, "slide", s));
        }
        for role in [
            (Role::Theme, "theme"),
            (Role::PresProps, "presProps"),
            (Role::ViewProps, "viewProps"),
            (Role::TableStyles, "tableStyles"),
        ] {
            // The theme a master has is the master's. Only one with no owner is the deck's.
            let parts = self.discover(role.0, main);
            let free = parts.iter().find(|p| {
                role.0 != Role::Theme
                    || !masters
                        .iter()
                        .any(|m| self.targets(m, "theme").iter().any(|t| t == *p))
            });
            if let Some(part) = free.cloned() {
                self.link(main, role.1, &part);
            }
        }
        (master_ids, slide_ids)
    }

    fn list_markup(&mut self, master_ids: &[String], slide_ids: &[String]) -> (String, String) {
        let masters: String = master_ids
            .iter()
            .map(|id| {
                let n = self.next_master_id;
                self.next_master_id = self.next_master_id.saturating_add(1);
                format!("<p:sldMasterId id=\"{n}\" r:id=\"{id}\"/>")
            })
            .collect();
        let slides: String = slide_ids
            .iter()
            .enumerate()
            .map(|(i, id)| {
                let n = FIRST_SLIDE_ID.saturating_add(i as u64);
                format!("<p:sldId id=\"{n}\" r:id=\"{id}\"/>")
            })
            .collect();
        (masters, slides)
    }

    /// The presentation part anew: lists from what exists, sizes from the reading if sound.
    fn build_main(&mut self, reading: Option<&XmlTree>, main: &str) {
        self.note(Repair::MainPartBuilt, main);
        self.damaged.insert(main.to_owned());
        let order = reading
            .map(|t| self.old_slide_order(t, main))
            .unwrap_or_default();
        let (master_ids, slide_ids) = self.rebuild_main_rels(main, &order);
        let (masters, slides) = self.list_markup(&master_ids, &slide_ids);
        let size = |local: &str, default: (u64, u64), min: u64, max: u64| {
            reading
                .and_then(|t| {
                    let el = t.child(t.root(), local)?;
                    let cx: u64 = t.plain_attribute(el, "cx")?.parse().ok()?;
                    let cy: u64 = t.plain_attribute(el, "cy")?.parse().ok()?;
                    ((min..=max).contains(&cx) && (min..=max).contains(&cy)).then_some((cx, cy))
                })
                .unwrap_or(default)
        };
        let slide_size = size("sldSz", DEFAULT_SLIDE_SIZE, 914_400, 51_206_400);
        let notes_size = size("notesSz", DEFAULT_NOTES_SIZE, 1, 27_273_042_316);
        let xml = format!(
            "{DECL}<p:presentation {ns}>{master_list}{slide_list}\
             <p:sldSz cx=\"{}\" cy=\"{}\"/><p:notesSz cx=\"{}\" cy=\"{}\"/></p:presentation>",
            slide_size.0,
            slide_size.1,
            notes_size.0,
            notes_size.1,
            ns = pml_decls(self.conformance),
            master_list = if masters.is_empty() {
                String::new()
            } else {
                format!("<p:sldMasterIdLst>{masters}</p:sldMasterIdLst>")
            },
            slide_list = if slides.is_empty() {
                String::new()
            } else {
                format!("<p:sldIdLst>{slides}</p:sldIdLst>")
            },
        );
        self.keys.insert(entry_key(main), main.to_owned());
        self.gone.remove(main);
        self.out.insert(main.to_owned(), xml.into_bytes());
    }

    fn old_slide_order(&self, tree: &XmlTree, main: &str) -> Vec<String> {
        let rels = self.rels_of(main);
        let Some(list) = tree.child(tree.root(), "sldIdLst") else {
            return Vec::new();
        };
        tree.child_elements(list)
            .into_iter()
            .filter_map(|el| rel_id_of(tree, el))
            .filter_map(|id| rels.iter().find(|r| r.id == id))
            .filter_map(|r| self.target_of(main, r))
            .collect()
    }

    /// Keep the part, and replace only the lists its relationships meant.
    fn rewrite_lists(&mut self, mut tree: XmlTree, main: &str) {
        let order = self.old_slide_order(&tree, main);
        let (master_ids, slide_ids) = self.rebuild_main_rels(main, &order);
        let root = tree.root();
        for local in ["sldMasterIdLst", "sldIdLst"] {
            if let Some(list) = tree.child(root, local) {
                tree.remove_child(root, list);
            }
        }
        // Notes and handout masters have no id to mend them with.
        for local in ["notesMasterIdLst", "handoutMasterIdLst"] {
            let dangling = tree.child(root, local).is_some_and(|list| {
                tree.child_elements(list).iter().any(|el| {
                    !rel_id_of(&tree, *el)
                        .is_some_and(|id| self.rels_of(main).iter().any(|r| r.id == id))
                })
            });
            if dangling && let Some(list) = tree.child(root, local) {
                tree.remove_child(root, list);
            }
        }
        let prefix = tree
            .qualified_name(root)
            .split_once(':')
            .map_or("p", |(p, _)| p)
            .to_owned();
        let r = tree
            .prefix_for_namespace(root, self.conformance.spell(R_NS))
            .unwrap_or_else(|| "r".to_owned());
        if !master_ids.is_empty() || !slide_ids.is_empty() {
            let declared = tree.prefix_for_namespace(root, self.conformance.spell(R_NS));
            if declared.is_none() {
                let uri = self.conformance.spell(R_NS).to_owned();
                let _ = tree.set_attribute(root, "xmlns:r", &uri);
            }
        }
        let first = tree.child_elements(root).first().copied();
        if !master_ids.is_empty() {
            let list = tree.insert_element(root, first, &format!("{prefix}:sldMasterIdLst"), &[]);
            for id in &master_ids {
                let n = self.next_master_id.to_string();
                self.next_master_id = self.next_master_id.saturating_add(1);
                tree.insert_element(
                    list,
                    None,
                    &format!("{prefix}:sldMasterId"),
                    &[("id", &n), (&format!("{r}:id"), id)],
                );
            }
        }
        if !slide_ids.is_empty() {
            let before = tree.child_elements(root).into_iter().find(|c| {
                !matches!(
                    tree.local_name(*c),
                    "sldMasterIdLst" | "notesMasterIdLst" | "handoutMasterIdLst"
                )
            });
            let list = tree.insert_element(root, before, &format!("{prefix}:sldIdLst"), &[]);
            for (i, id) in slide_ids.iter().enumerate() {
                let n = FIRST_SLIDE_ID.saturating_add(i as u64).to_string();
                tree.insert_element(
                    list,
                    None,
                    &format!("{prefix}:sldId"),
                    &[("id", &n), (&format!("{r}:id"), id)],
                );
            }
        }
        self.note(Repair::ListEntryDropped, main);
        self.damaged.insert(main.to_owned());
        self.out.insert(main.to_owned(), tree.to_bytes());
    }

    /// The first master, made if there is none, with a theme and its place on the list.
    fn first_master(&mut self, main: &str) -> String {
        if let Some(first) = self.targets(main, "slideMaster").into_iter().next() {
            return first;
        }
        let name = self.numbered_name("ppt/slideMasters/slideMaster", "xml");
        self.create_part(&name, Role::Master);
        self.note(Repair::MasterSupplied, &name);
        self.add_master_to_main(main, &name);
        name
    }

    fn add_master_to_main(&mut self, main: &str, master: &str) {
        let id = self.link(main, "slideMaster", master);
        let Some(bytes) = self.read(main) else { return };
        let Ok(mut tree) = XmlTree::parse(&bytes, main, self.limits) else {
            return;
        };
        let root = tree.root();
        let r = tree
            .prefix_for_namespace(root, self.conformance.spell(R_NS))
            .unwrap_or_else(|| "r".to_owned());
        if tree
            .prefix_for_namespace(root, self.conformance.spell(R_NS))
            .is_none()
        {
            let uri = self.conformance.spell(R_NS).to_owned();
            let _ = tree.set_attribute(root, "xmlns:r", &uri);
        }
        let prefix = tree
            .qualified_name(root)
            .split_once(':')
            .map_or("p", |(p, _)| p)
            .to_owned();
        let list = match tree.child(root, "sldMasterIdLst") {
            Some(list) => list,
            None => {
                let first = tree.child_elements(root).first().copied();
                tree.insert_element(root, first, &format!("{prefix}:sldMasterIdLst"), &[])
            }
        };
        let n = self.fresh_master_id(&tree, list);
        tree.insert_element(
            list,
            None,
            &format!("{prefix}:sldMasterId"),
            &[("id", &n), (&format!("{r}:id"), &id)],
        );
        self.out.insert(main.to_owned(), tree.to_bytes());
    }

    fn fresh_master_id(&mut self, tree: &XmlTree, list: ElementId) -> String {
        for el in tree.child_elements(list) {
            if let Some(n) = tree
                .plain_attribute(el, "id")
                .and_then(|v| v.parse::<u64>().ok())
            {
                self.next_master_id = self.next_master_id.max(n.saturating_add(1));
            }
        }
        let n = self.next_master_id;
        self.next_master_id = n.saturating_add(1);
        n.to_string()
    }

    /// The part a relationship of `kind` from `source` names, made if it is missing.
    /// `Some` if there is a part at the end of one, and `None` if there is no such link.
    fn linked_or_made(
        &mut self,
        source: &str,
        kind: &str,
        role: Role,
        repair: Repair,
    ) -> Option<String> {
        let rels = self.rels_of(source);
        let of_kind = rels.iter().find(|r| rel_kind(&r.rel_type) == Some(kind))?;
        if let Some(actual) = self.target_of(source, of_kind) {
            return Some(actual);
        }
        if of_kind.mode != TargetMode::Internal {
            return None;
        }
        let target = resolve_internal(source, &of_kind.target)?;
        if !is_safe_new_name(&target) {
            return None;
        }
        self.create_part(&target, role);
        self.note(repair, &target);
        Some(target)
    }

    /// Every slide with a layout, every layout with a master, every master with a theme.
    fn fix_chain(&mut self, main: &str) {
        // What the main part lists and is missing, made in its place, so order survives.
        for kind in [
            ("slide", Role::Slide, Repair::SlideEmpty),
            ("slideMaster", Role::Master, Repair::MasterSupplied),
            ("theme", Role::Theme, Repair::ThemeSupplied),
            ("presProps", Role::PresProps, Repair::PartSupplied),
            ("viewProps", Role::ViewProps, Repair::PartSupplied),
            ("tableStyles", Role::TableStyles, Repair::PartSupplied),
            ("notesMaster", Role::NotesMaster, Repair::PartSupplied),
            ("handoutMaster", Role::HandoutMaster, Repair::PartSupplied),
        ] {
            let rels = self.rels_of(main);
            for r in rels
                .iter()
                .filter(|r| rel_kind(&r.rel_type) == Some(kind.0))
            {
                if self.target_of(main, r).is_none()
                    && r.mode == TargetMode::Internal
                    && let Some(target) = resolve_internal(main, &r.target)
                    && is_safe_new_name(&target)
                {
                    self.create_part(&target, kind.1);
                    self.damaged.insert(target.clone());
                    self.note(kind.2, &target);
                }
            }
        }

        for kind in [
            ("extended-properties", Role::App),
            ("extendedProperties", Role::App),
            ("custom-properties", Role::Custom),
            ("customProperties", Role::Custom),
        ] {
            self.make_missing_root_target(kind.0, kind.1);
        }
        self.make_missing_root_target("metadata/core-properties", Role::Core);
        self.relink_root_properties(main);

        let slides = self.targets(main, "slide");
        let mut orphans = Vec::new();
        let mut layouts: Vec<String> = Vec::new();
        for slide in &slides {
            match self.linked_or_made(slide, "slideLayout", Role::Layout, Repair::LayoutSupplied) {
                Some(layout) => {
                    if !layouts.contains(&layout) {
                        layouts.push(layout);
                    }
                }
                None => orphans.push(slide.clone()),
            }
        }
        for master in self.targets(main, "slideMaster") {
            for layout in self.targets(&master, "slideLayout") {
                if !layouts.contains(&layout) {
                    layouts.push(layout);
                }
            }
        }
        // The layouts each master holds, against a fresh look after anything was added.
        let mut touched_masters: BTreeSet<String> = BTreeSet::new();
        for layout in &layouts {
            let has =
                self.linked_or_made(layout, "slideMaster", Role::Master, Repair::MasterSupplied);
            let master = match has {
                Some(master) => {
                    if !self.targets(main, "slideMaster").contains(&master) {
                        self.add_master_to_main(main, &master);
                    }
                    master
                }
                None => {
                    let master = self.first_master(main);
                    self.link(layout, "slideMaster", &master);
                    self.note(Repair::MasterSupplied, layout);
                    master
                }
            };
            if !self.targets(&master, "slideLayout").contains(layout) {
                self.link(&master, "slideLayout", layout);
                touched_masters.insert(master);
            }
        }
        for slide in orphans {
            let master = self.first_master(main);
            let existing = self.targets(&master, "slideLayout").into_iter().next();
            let layout = match existing {
                Some(layout) => layout,
                None => {
                    let name = self.numbered_name("ppt/slideLayouts/slideLayout", "xml");
                    self.create_part(&name, Role::Layout);
                    self.link(&name, "slideMaster", &master);
                    self.link(&master, "slideLayout", &name);
                    touched_masters.insert(master.clone());
                    name
                }
            };
            self.link(&slide, "slideLayout", &layout);
            self.note(Repair::LayoutSupplied, &slide);
        }
        for master in self.targets(main, "slideMaster") {
            self.fix_theme(&master, main);
            self.sync_master_list(&master, touched_masters.contains(&master));
        }
    }

    /// The package's relationships were rebuilt, so the property parts that exist go back on
    /// them. Without this, an edit to a property would make a second part beside the first.
    fn relink_root_properties(&mut self, main: &str) {
        if !self.rels_dirty.contains("") {
            return;
        }
        let spelled = |this: &Self, transitional: &str, strict: &str| match this.conformance {
            Conformance::Transitional => format!("{TRANSITIONAL_RELS}{transitional}"),
            Conformance::Strict => format!("{STRICT_RELS}{strict}"),
        };
        let wanted = [
            (Role::Core, CORE_REL.to_owned()),
            (
                Role::App,
                spelled(self, "extended-properties", "extendedProperties"),
            ),
            (
                Role::Custom,
                spelled(self, "custom-properties", "customProperties"),
            ),
        ];
        for (role, rel_type) in wanted {
            let linked = self
                .rels_of("")
                .iter()
                .any(|r| Role::of_rel(&r.rel_type) == Some(role));
            if linked {
                continue;
            }
            if let Some(part) = self.discover(role, main).into_iter().next() {
                let mut list = self.rels.remove("").unwrap_or_default();
                let id = next_rel_id(&list);
                list.push(Relationship {
                    id,
                    rel_type,
                    target: part,
                    mode: TargetMode::Internal,
                });
                self.rels.insert(String::new(), list);
            }
        }
    }

    /// A document property part the package's relationships name, and that is gone. An
    /// edit to the property would otherwise have nowhere to go.
    fn make_missing_root_target(&mut self, kind: &str, role: Role) {
        let rels = self.rels_of("");
        let suffix = format!("/{kind}");
        for r in rels.iter().filter(|r| r.rel_type.ends_with(&suffix)) {
            if self.target_of("", r).is_none()
                && r.mode == TargetMode::Internal
                && let Some(target) = resolve_internal("", &r.target)
                && is_safe_new_name(&target)
            {
                self.create_part(&target, role);
                self.note(Repair::PartSupplied, &target);
            }
        }
    }

    fn fix_theme(&mut self, master: &str, main: &str) {
        if self
            .linked_or_made(master, "theme", Role::Theme, Repair::ThemeSupplied)
            .is_some()
        {
            return;
        }
        // A theme belongs to one master, so one no master has is the only one to take.
        let masters = self.targets(main, "slideMaster");
        let taken: Vec<String> = masters
            .iter()
            .flat_map(|m| self.targets(m, "theme"))
            .collect();
        let free = self
            .discover(Role::Theme, main)
            .into_iter()
            .find(|t| !taken.contains(t));
        let theme = free.unwrap_or_else(|| {
            let name = self.numbered_name("ppt/theme/theme", "xml");
            self.create_part(&name, Role::Theme);
            name
        });
        self.link(master, "theme", &theme);
        self.note(Repair::ThemeSupplied, master);
    }

    /// A master's `sldLayoutIdLst` to its relationships, when it was damaged, new or changed,
    /// or names what is not there. A healthy one is not touched.
    fn sync_master_list(&mut self, master: &str, touched: bool) {
        let Some(bytes) = self.read(master) else {
            return;
        };
        let Ok(mut tree) = XmlTree::parse(&bytes, master, self.limits) else {
            return;
        };
        let root = tree.root();
        let rels = self.rels_of(master);
        let layout_ids: Vec<String> = rels
            .iter()
            .filter(|r| rel_kind(&r.rel_type) == Some("slideLayout"))
            .filter(|r| self.target_of(master, r).is_some())
            .map(|r| r.id.clone())
            .collect();
        let list = tree.child(root, "sldLayoutIdLst");
        let listed: Vec<String> = list
            .map(|l| {
                tree.child_elements(l)
                    .iter()
                    .filter_map(|el| rel_id_of(&tree, *el))
                    .collect()
            })
            .unwrap_or_default();
        let dangling = listed.iter().any(|id| !layout_ids.contains(id));
        let unlisted = layout_ids.iter().any(|id| !listed.contains(id));
        let was_mended = self.damaged.contains(master)
            || self.out.contains_key(master)
            || self.rels_dirty.contains(master);
        let needed = dangling || touched || (unlisted && was_mended);
        if !needed {
            return;
        }
        let prefix = tree
            .qualified_name(root)
            .split_once(':')
            .map_or("p", |(p, _)| p)
            .to_owned();
        let list = match list {
            Some(list) => list,
            None => {
                let after = tree
                    .child_elements(root)
                    .into_iter()
                    .find(|c| !matches!(tree.local_name(*c), "cSld" | "clrMap"));
                tree.insert_element(root, after, &format!("{prefix}:sldLayoutIdLst"), &[])
            }
        };
        if tree
            .prefix_for_namespace(root, self.conformance.spell(R_NS))
            .is_none()
        {
            let uri = self.conformance.spell(R_NS).to_owned();
            let _ = tree.set_attribute(root, "xmlns:r", &uri);
        }
        let r = tree
            .prefix_for_namespace(root, self.conformance.spell(R_NS))
            .unwrap_or_else(|| "r".to_owned());
        for el in tree.child_elements(list) {
            if !rel_id_of(&tree, el).is_some_and(|id| layout_ids.contains(&id)) {
                tree.remove_child(list, el);
            }
        }
        for id in &layout_ids {
            if !listed.contains(id) {
                let n = self.fresh_master_id(&tree, list);
                tree.insert_element(
                    list,
                    None,
                    &format!("{prefix}:sldLayoutId"),
                    &[("id", &n), (&format!("{r}:id"), id)],
                );
            }
        }
        self.out.insert(master.to_owned(), tree.to_bytes());
    }

    /// Every part in the image, once.
    fn final_names(&self) -> Vec<String> {
        self.keys
            .values()
            .filter(|n| !self.gone.contains(*n))
            .cloned()
            .collect()
    }

    /// Declare whatever is not declared, and correct what a role makes plain is wrong.
    fn finish_types(&mut self, main: &str) {
        let mut wanted: Vec<(String, Role)> = Vec::new();
        for name in self.final_names() {
            let key = entry_key(&name);
            if key == entry_key(CONTENT_TYPES_PART) {
                continue;
            }
            let extension = key.rsplit_once('.').map(|(_, e)| e.to_owned());
            if extension.as_deref() == Some("rels") {
                if self.types.resolve(&name) != Some(RELS_TYPE) {
                    self.declare_default("rels", RELS_TYPE, &name);
                }
                continue;
            }
            let role = self.role_of(&name, main);
            if role != Role::Other {
                wanted.push((name.clone(), role));
                continue;
            }
            if self.types.resolve(&name).is_some() {
                continue;
            }
            let known = extension.as_deref().and_then(type_for_extension);
            match (extension, known) {
                (Some(ext), Some(known)) if is_extension(&ext) => {
                    self.declare_default(&ext, known, &name);
                }
                (Some(ext), None) if is_extension(&ext) => {
                    self.declare_default(&ext, "application/octet-stream", &name);
                }
                _ => {
                    self.declare_override(&name, "application/octet-stream");
                }
            }
        }
        for (name, role) in wanted {
            let current = self.types.resolve(&name).map(str::to_owned);
            // A main part may be any of the kinds; `Other` never reaches here.
            let fits = current
                .as_deref()
                .is_some_and(|t| Role::of_content_type(t) == Some(role));
            if !fits
                && let Some(wanted_type) = role.content_type()
                && self.declare_override(&name, &wanted_type)
            {
                self.note(Repair::TypeCorrected, &name);
            }
        }
        if !self.types_dirty {
            return;
        }
        let mut xml = format!("{DECL}<Types xmlns=\"{TYPES_NS}\">");
        for (ext, value) in self.types.defaults() {
            if is_extension(ext) && is_content_type(value) {
                xml.push_str(&format!(
                    "<Default Extension=\"{}\" ContentType=\"{}\"/>",
                    escape_attribute(ext),
                    escape_attribute(value)
                ));
            }
        }
        let names = self.final_names();
        for name in names {
            let key = entry_key(&name);
            if let Some(value) = self.types.override_for(&key)
                && is_content_type(value)
            {
                xml.push_str(&format!(
                    "<Override PartName=\"/{}\" ContentType=\"{}\"/>",
                    escape_attribute(&name),
                    escape_attribute(value)
                ));
            }
        }
        xml.push_str("</Types>");
        let name = self
            .actual(CONTENT_TYPES_PART)
            .unwrap_or_else(|| CONTENT_TYPES_PART.to_owned());
        self.keys.insert(entry_key(&name), name.clone());
        self.out.insert(name, xml.into_bytes());
    }

    fn declare_default(&mut self, ext: &str, value: &str, part: &str) -> bool {
        if self.types.len() >= self.limits.max_content_types {
            self.note(Repair::Xml(XmlRepair::LimitStopped), part);
            return false;
        }
        self.types.set_default(ext, value);
        self.types_dirty = true;
        true
    }

    fn declare_override(&mut self, part: &str, value: &str) -> bool {
        if self.types.len() >= self.limits.max_content_types {
            self.note(Repair::Xml(XmlRepair::LimitStopped), part);
            return false;
        }
        self.types.set_override(part, value);
        self.types_dirty = true;
        true
    }

    fn finish_rels(&mut self) {
        let dirty: Vec<String> = self.rels_dirty.iter().cloned().collect();
        for source in dirty {
            if !source.is_empty() && !self.exists(&source) {
                continue;
            }
            let list = self.rels_of(&source);
            let mut xml = format!("{DECL}<Relationships xmlns=\"{RELS_NS}\">");
            for r in &list {
                xml.push_str(&format!(
                    "<Relationship Id=\"{}\" Type=\"{}\" Target=\"{}\"{}/>",
                    escape_attribute(&r.id),
                    escape_attribute(&r.rel_type),
                    escape_attribute(&r.target),
                    if r.mode == TargetMode::External {
                        " TargetMode=\"External\""
                    } else {
                        ""
                    }
                ));
            }
            xml.push_str("</Relationships>");
            let name = rels_part_for(&source);
            let name = self.actual(&name).unwrap_or(name);
            self.keys.insert(entry_key(&name), name.clone());
            self.out.insert(name, xml.into_bytes());
        }
    }
}

/// A presentation part being written needs `notesSz`, and it is the one thing it needs.
fn ensure_required_main_children(tree: &mut XmlTree) {
    let root = tree.root();
    if tree.child(root, "notesSz").is_some() {
        return;
    }
    let prefix = tree
        .qualified_name(root)
        .split_once(':')
        .map_or("", |(p, _)| p)
        .to_owned();
    let name = if prefix.is_empty() {
        "notesSz".to_owned()
    } else {
        format!("{prefix}:notesSz")
    };
    let before = tree.child_elements(root).into_iter().find(|c| {
        !matches!(
            tree.local_name(*c),
            "sldMasterIdLst" | "notesMasterIdLst" | "handoutMasterIdLst" | "sldIdLst" | "sldSz"
        )
    });
    let cx = DEFAULT_NOTES_SIZE.0.to_string();
    let cy = DEFAULT_NOTES_SIZE.1.to_string();
    tree.insert_element(root, before, &name, &[("cx", &cx), ("cy", &cy)]);
}

fn prune_for(role: Role) -> Prune {
    if role.slide_like_root().is_some() {
        Prune::SlideLike
    } else {
        Prune::Nothing
    }
}

/// A name a new part may take: one with a valid part name, safe as a path, that is not a
/// relationships part. A missing part's own target is usually this.
fn is_safe_new_name(name: &str) -> bool {
    casual_slide_package::is_safe_part_path(name)
        && part_name_key(name).is_some()
        && source_of_rels(&entry_key(name)).is_none()
        && name.is_ascii()
        && name.split('/').all(|s| {
            s.bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'-'))
        })
}

/// An element's relationship attribute, whichever prefix it has.
fn rel_id_of(tree: &XmlTree, el: ElementId) -> Option<String> {
    tree.declared_attributes(el)
        .into_iter()
        .find(|(name, _)| {
            name.split_once(':')
                .is_some_and(|(prefix, local)| local == "id" && prefix != "xmlns")
        })
        .map(|(_, value)| value)
}

fn next_rel_id(list: &[Relationship]) -> String {
    let highest = list
        .iter()
        .filter_map(|r| r.id.strip_prefix("rId")?.parse::<u64>().ok())
        .max()
        .unwrap_or(0);
    let mut n = highest.saturating_add(1);
    loop {
        let id = format!("rId{n}");
        if !list.iter().any(|r| r.id == id) {
            return id;
        }
        n = n.saturating_add(1);
    }
}

/// Whether what strict admission let through still needs mending: a slide, layout or master
/// with nothing to inherit from. Relationships alone say so, with no part read.
pub(crate) fn chain_is_broken(
    main: &str,
    relationships: &BTreeMap<String, Vec<Relationship>>,
    names: &[String],
    types: &ContentTypes,
) -> bool {
    let keys: BTreeMap<String, &String> = names
        .iter()
        .filter_map(|n| Some((part_name_key(n)?, n)))
        .collect();
    let target = |source: &str, r: &Relationship| {
        (r.mode == TargetMode::Internal)
            .then(|| resolve_internal(source, &r.target))
            .flatten()
    };
    let exists = |t: &str| part_name_key(t).is_some_and(|k| keys.contains_key(&k));
    let linked = |source: &str, kind: &str| -> Option<Vec<String>> {
        let rels = relationships.get(source)?;
        Some(
            rels.iter()
                .filter(|r| rel_kind(&r.rel_type) == Some(kind))
                .filter_map(|r| target(source, r))
                .collect(),
        )
    };
    let needs = |source: &str, kind: &str| match linked(source, kind) {
        None => true,
        Some(targets) => targets.is_empty() || targets.iter().any(|t| !exists(t)),
    };
    let actual = |t: &str| part_name_key(t).and_then(|k| keys.get(&k).map(|n| (*n).clone()));
    let right_type =
        |part: &str, role: Role| types.resolve(part).and_then(Role::of_content_type) == Some(role);
    let property_gone = relationships.get("").is_some_and(|rels| {
        rels.iter().any(|r| {
            let property = r.rel_type.ends_with("/metadata/core-properties")
                || rel_kind(&r.rel_type)
                    .and_then(Role::of_rel_kind)
                    .is_some_and(|role| matches!(role, Role::App | Role::Custom));
            property && target("", r).is_some_and(|t| !exists(&t))
        })
    });
    if property_gone {
        return true;
    }
    let mut layouts_seen = BTreeSet::new();
    let mut masters_seen = BTreeSet::new();
    let slides = linked(main, "slide").unwrap_or_default();
    for slide in &slides {
        let Some(slide) = actual(slide) else {
            return true;
        };
        if needs(&slide, "slideLayout") || !right_type(&slide, Role::Slide) {
            return true;
        }
        for layout in linked(&slide, "slideLayout").unwrap_or_default() {
            let Some(layout) = actual(&layout) else {
                return true;
            };
            if !layouts_seen.insert(layout.clone()) {
                continue;
            }
            if needs(&layout, "slideMaster") || !right_type(&layout, Role::Layout) {
                return true;
            }
            for master in linked(&layout, "slideMaster").unwrap_or_default() {
                let Some(master) = actual(&master) else {
                    return true;
                };
                if masters_seen.insert(master.clone())
                    && (needs(&master, "theme") || !right_type(&master, Role::Master))
                {
                    return true;
                }
            }
        }
    }
    false
}
