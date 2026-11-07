//! What a repair did, in codes and in words a user can read (`REC-05`,
//! `docs/29-RECOVERY.md`).
//!
//! **No document text.** A repair is a value of a fixed enum and a part's name, so a
//! report can go to a log (`SEC-015`). Part names come from the file; each is scrubbed of
//! control characters and bounded before it is kept, and only named parts appear.

use casual_slide_package::ContainerFaultKind;

use crate::recover_xml::XmlRepair;

/// One thing a repair did. None of these stops a deck opening.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Repair {
    /// A fault in the ZIP container.
    Container(ContainerFaultKind),
    /// `[Content_Types].xml` was missing or broken, and was rebuilt.
    ContentTypesRebuilt,
    /// A relationships part was missing or broken, and was rebuilt.
    RelationshipsRebuilt,
    /// The main part was found by its name or root, with no relationship to it.
    MainPartFound,
    /// No main part could be read, and one was built from the slides that exist.
    MainPartBuilt,
    /// A part's declared content type was wrong for what it is, and was corrected.
    TypeCorrected,
    /// A part demands a namespace this engine does not understand. The demand was set
    /// aside, and what the namespace held was left out.
    MustUnderstandSetAside,
    /// An entry on a list points at nothing, and was left off.
    ListEntryDropped,
    /// A fault in a part's XML.
    Xml(XmlRepair),
    /// A part could not be used. Its bytes are retained under `recovered/`.
    PartSetAside,
    /// A slide could not be read. It is an empty slide marked damaged, so order survives.
    SlideEmpty,
    /// A slide's layout was missing. The nearest was used, or a minimal one made.
    LayoutSupplied,
    /// A layout's master was missing. The nearest was used, or a minimal one made.
    MasterSupplied,
    /// A master's theme was missing. A minimal one was made.
    ThemeSupplied,
    /// A part another part needs was missing, and an empty one was made.
    PartSupplied,
}

impl Repair {
    /// The stable diagnostic code (`docs/20`).
    pub fn code(self) -> &'static str {
        match self {
            Self::Container(kind) => kind.code(),
            Self::ContentTypesRebuilt => "CS-REC-0201",
            Self::RelationshipsRebuilt => "CS-REC-0202",
            Self::MainPartFound => "CS-REC-0203",
            Self::MainPartBuilt => "CS-REC-0204",
            Self::TypeCorrected => "CS-REC-0205",
            Self::MustUnderstandSetAside => "CS-REC-0206",
            Self::ListEntryDropped => "CS-REC-0207",
            Self::Xml(XmlRepair::DoctypeSkipped) => "CS-REC-0301",
            Self::Xml(XmlRepair::NestingClosed) => "CS-REC-0302",
            Self::Xml(XmlRepair::AttributeRepaired) => "CS-REC-0303",
            Self::Xml(XmlRepair::TextRepaired) => "CS-REC-0304",
            Self::Xml(XmlRepair::ReferenceLiteral) => "CS-REC-0305",
            Self::Xml(XmlRepair::OutsideRootDropped) => "CS-REC-0306",
            Self::Xml(XmlRepair::LimitStopped) => "CS-REC-0307",
            Self::Xml(XmlRepair::SubtreeLeftOut) => "CS-REC-0308",
            Self::PartSetAside => "CS-REC-0309",
            Self::SlideEmpty => "CS-REC-0401",
            Self::LayoutSupplied => "CS-REC-0402",
            Self::MasterSupplied => "CS-REC-0403",
            Self::ThemeSupplied => "CS-REC-0404",
            Self::PartSupplied => "CS-REC-0405",
        }
    }

    /// What to tell the user. Fixed text.
    pub fn plain(self) -> &'static str {
        match self {
            Self::Container(kind) => kind.plain(),
            Self::ContentTypesRebuilt => {
                "the list of what each item in the file is was damaged, and was rebuilt"
            }
            Self::RelationshipsRebuilt => {
                "the links between items in the file were damaged, and were rebuilt"
            }
            Self::MainPartFound => "the presentation's main item was found without its link",
            Self::MainPartBuilt => {
                "the presentation's main item could not be read; it was rebuilt from the slides found"
            }
            Self::TypeCorrected => "this item was labelled as the wrong kind, and was corrected",
            Self::MustUnderstandSetAside => {
                "this item needs a feature this application does not have; what depends on it was left out"
            }
            Self::ListEntryDropped => {
                "a list in this item named something missing; that entry was left off"
            }
            Self::Xml(XmlRepair::DoctypeSkipped) => {
                "this item carried a document type declaration, which was ignored"
            }
            Self::Xml(XmlRepair::NestingClosed) => {
                "this item's structure was broken, and was mended"
            }
            Self::Xml(XmlRepair::AttributeRepaired) => {
                "some of this item's settings were malformed, and were mended"
            }
            Self::Xml(XmlRepair::TextRepaired) => {
                "this item held the wrong encoding or invalid characters, which were mended"
            }
            Self::Xml(XmlRepair::ReferenceLiteral) => {
                "this item referred to something undefined, which was kept as plain text and not followed"
            }
            Self::Xml(XmlRepair::OutsideRootDropped) => {
                "this item held content outside its main structure, which was left out"
            }
            Self::Xml(XmlRepair::LimitStopped) => {
                "this item is too large or too deeply nested to read in full; what was read is kept"
            }
            Self::Xml(XmlRepair::SubtreeLeftOut) => {
                "part of this item could not be trusted and was left out; the original is kept in the file"
            }
            Self::PartSetAside => {
                "this item could not be used and was left out; its original bytes are kept in the file"
            }
            Self::SlideEmpty => {
                "this slide could not be read and is shown empty, in its place; its original bytes are kept in the file"
            }
            Self::LayoutSupplied => "this slide's layout was missing, so another was used",
            Self::MasterSupplied => "this layout's master was missing, so another was used",
            Self::ThemeSupplied => "this master's theme was missing, so a plain one was used",
            Self::PartSupplied => "this item was missing, so an empty one was used",
        }
    }
}

/// One line of a report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportLine {
    /// The stable code.
    pub code: &'static str,
    /// The part, or empty for the file as a whole.
    pub part: String,
    /// What to say about it.
    pub text: &'static str,
}

/// What a recovered deck tells its user. Empty for a deck that arrived healthy.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RecoveryReport {
    /// One line per repair and part, in a stable order.
    pub lines: Vec<ReportLine>,
}

impl RecoveryReport {
    /// Whether anything was repaired.
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// The report as sentences, for a host to show. No document text.
    pub fn plain_language(&self) -> Vec<String> {
        self.lines
            .iter()
            .map(|line| {
                if line.part.is_empty() {
                    let mut text = line.text.to_owned();
                    if let Some(first) = text.get_mut(..1) {
                        first.make_ascii_uppercase();
                    }
                    format!("{text}.")
                } else {
                    format!("{}: {}.", line.part, line.text)
                }
            })
            .collect()
    }
}
