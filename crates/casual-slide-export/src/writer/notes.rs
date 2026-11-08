//! Parts the writer authors: a new slide from its layout, a notes slide, and the notes
//! master a deck gets with its first notes (`INV-NM-02`, `INV-NM-03`).
//!
//! **The notes master's exact form is Unverified**: it has the root and `clrMap` ECMA-376
//! documents, and the six placeholders a third-party report says PowerPoint expects, with
//! geometry scaled from `notesSz`. `F-007` is to compare it with one PowerPoint wrote.

use casual_slide_ooxml::{Conformance, ElementId, XmlTree};

use super::draft::{DECL, Draft};
use super::error::WriterError;
use super::names::rels_part_for;
use super::ops::SlideRef;
use super::view::{pml_child, pml_children};
use crate::properties::ExportError;

const PML: &str = "application/vnd.openxmlformats-officedocument.presentationml.";
const THEME_TYPE: &str = "application/vnd.openxmlformats-officedocument.theme+xml";
const PML_NS: &str = "http://schemas.openxmlformats.org/presentationml/2006/main";
const DML_NS: &str = "http://schemas.openxmlformats.org/drawingml/2006/main";
const R_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

/// `xmlns` declarations in the file's own form (`INV-SA-14`).
fn namespaces(conformance: Conformance) -> String {
    format!(
        "xmlns:a=\"{}\" xmlns:r=\"{}\" xmlns:p=\"{}\"",
        conformance.spell(DML_NS),
        conformance.spell(R_NS),
        conformance.spell(PML_NS),
    )
}

/// A character XML 1.0 cannot carry refuses the text, rather than the part going bad
/// when it is read back.
pub(crate) fn check_text(text: &str, part: &str) -> Result<(), WriterError> {
    let bad = text
        .chars()
        .any(|c| matches!(c, '\u{0}'..='\u{8}' | '\u{b}' | '\u{c}' | '\u{e}'..='\u{1f}' | '\u{fffe}' | '\u{ffff}'));
    if bad {
        return Err(WriterError::Export(ExportError::InvalidCharacter {
            part: part.to_owned(),
        }));
    }
    Ok(())
}

const TREE_HEAD: &str =
    "<p:nvGrpSpPr><p:cNvPr id=\"1\" name=\"\"/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/>";

/// Placeholder types a new slide's copy holds text in. Picture and table placeholders
/// and the like take no text body.
fn takes_text(kind: Option<&str>) -> bool {
    matches!(
        kind,
        None | Some("title" | "body" | "ctrTitle" | "subTitle" | "obj")
    )
}

/// One placeholder of a layout, as the new slide's `p:sp`.
fn placeholder_copy(next_id: u64, layout: &XmlTree, ph: ElementId) -> Option<String> {
    let kind = layout.plain_attribute(ph, "type");
    // A slide shows the footer placeholders only when asked (Inferred, what PowerPoint's
    // own new slides look like in the corpus).
    if matches!(kind.as_deref(), Some("dt" | "ftr" | "sldNum")) {
        return None;
    }
    let mut attrs = String::new();
    for name in ["type", "orient", "sz", "idx"] {
        if let Some(value) = layout.plain_attribute(ph, name) {
            let safe = value.chars().all(|c| c.is_ascii_alphanumeric());
            if safe {
                attrs.push_str(&format!(" {name}=\"{value}\""));
            }
        }
    }
    let body = if takes_text(kind.as_deref()) {
        "<p:txBody><a:bodyPr/><a:lstStyle/><a:p/></p:txBody>"
    } else {
        ""
    };
    Some(format!(
        "<p:sp><p:nvSpPr><p:cNvPr id=\"{next_id}\" name=\"Placeholder {next_id}\"/>\
         <p:cNvSpPr><a:spLocks noGrp=\"1\"/></p:cNvSpPr><p:nvPr><p:ph{attrs}/></p:nvPr></p:nvSpPr>\
         <p:spPr/>{body}</p:sp>"
    ))
}

/// The part a new slide on this layout starts as, exactly as [`super::Op::AddSlide`]
/// writes it, for a host that shows the slide before it is saved (`VS-08`).
pub fn new_slide_part(conformance: Conformance, layout: &XmlTree) -> Vec<u8> {
    new_slide_xml(conformance, layout).into_bytes()
}

/// A new slide on `layout`, with an empty copy of each of its placeholders. Layout
/// placeholders inside a group or an `mc:AlternateContent` are not copied, since a slide
/// is free to leave any placeholder out.
pub(crate) fn new_slide_xml(conformance: Conformance, layout: &XmlTree) -> String {
    let mut shapes = String::new();
    let mut next_id = 2u64;
    if let Some(tree) =
        pml_child(layout, layout.root(), "cSld").and_then(|c| pml_child(layout, c, "spTree"))
    {
        for sp in pml_children(layout, tree, "sp") {
            let ph = pml_child(layout, sp, "nvSpPr")
                .and_then(|nv| pml_child(layout, nv, "nvPr"))
                .and_then(|nv| pml_child(layout, nv, "ph"));
            if let Some(ph) = ph
                && let Some(copy) = placeholder_copy(next_id, layout, ph)
            {
                shapes.push_str(&copy);
                next_id = next_id.saturating_add(1);
            }
        }
    }
    format!(
        "{DECL}<p:sld {}><p:cSld><p:spTree>{TREE_HEAD}{shapes}</p:spTree></p:cSld>\
         <p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr></p:sld>",
        namespaces(conformance)
    )
}

/// The part a new notes slide starts as, exactly as [`super::Op::AddNotes`] writes it
/// with no text, for a host that edits the notes before they are saved (`UX-011`).
pub fn new_notes_part(conformance: Conformance) -> Vec<u8> {
    notes_slide_xml(conformance).into_bytes()
}

fn notes_slide_xml(conformance: Conformance) -> String {
    format!(
        "{DECL}<p:notes {}><p:cSld><p:spTree>{TREE_HEAD}\
         <p:sp><p:nvSpPr><p:cNvPr id=\"2\" name=\"Slide Image Placeholder 1\"/>\
         <p:cNvSpPr><a:spLocks noGrp=\"1\" noRot=\"1\" noChangeAspect=\"1\"/></p:cNvSpPr>\
         <p:nvPr><p:ph type=\"sldImg\"/></p:nvPr></p:nvSpPr><p:spPr/></p:sp>\
         <p:sp><p:nvSpPr><p:cNvPr id=\"3\" name=\"Notes Placeholder 2\"/>\
         <p:cNvSpPr><a:spLocks noGrp=\"1\"/></p:cNvSpPr>\
         <p:nvPr><p:ph type=\"body\" idx=\"1\"/></p:nvPr></p:nvSpPr><p:spPr/>\
         <p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:rPr lang=\"en-US\"/><a:t></a:t></a:r></a:p></p:txBody></p:sp>\
         </p:spTree></p:cSld><p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr></p:notes>",
        namespaces(conformance)
    )
}

/// `value * num / den`, all in EMU and held in range: `notesSz` comes from the file.
fn scale(value: i64, num: i64, den: i64) -> i64 {
    i128::from(value)
        .saturating_mul(i128::from(num))
        .checked_div(i128::from(den))
        .and_then(|v| i64::try_from(v).ok())
        .unwrap_or(0)
}

fn notes_master_xml(conformance: Conformance, cx: i64, cy: i64) -> String {
    // Fractions of the default 6858000 by 9144000 notes page.
    const W: i64 = 6_858_000;
    const H: i64 = 9_144_000;
    let frame = |x: i64, y: i64, w: i64, h: i64| {
        format!(
            "<p:spPr><a:xfrm><a:off x=\"{}\" y=\"{}\"/><a:ext cx=\"{}\" cy=\"{}\"/></a:xfrm>\
             <a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom></p:spPr>",
            scale(cx, x, W),
            scale(cy, y, H),
            scale(cx, w, W),
            scale(cy, h, H),
        )
    };
    let text =
        "<p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:endParaRPr lang=\"en-US\"/></a:p></p:txBody>";
    let shape = |id: u32, name: &str, ph: &str, spatial: String, body: &str, locks: &str| {
        format!(
            "<p:sp><p:nvSpPr><p:cNvPr id=\"{id}\" name=\"{name}\"/>\
             <p:cNvSpPr><a:spLocks noGrp=\"1\"{locks}/></p:cNvSpPr><p:nvPr><p:ph {ph}/></p:nvPr></p:nvSpPr>\
             {spatial}{body}</p:sp>"
        )
    };
    let shapes = [
        shape(
            2,
            "Header Placeholder 1",
            "type=\"hdr\" sz=\"quarter\"",
            frame(0, 0, 2_971_800, 458_788),
            text,
            "",
        ),
        shape(
            3,
            "Date Placeholder 2",
            "type=\"dt\" idx=\"1\"",
            frame(3_884_613, 0, 2_971_800, 458_788),
            text,
            "",
        ),
        shape(
            4,
            "Slide Image Placeholder 3",
            "type=\"sldImg\" idx=\"2\"",
            frame(685_800, 1_143_000, 5_486_400, 3_086_100),
            "",
            " noRot=\"1\" noChangeAspect=\"1\"",
        ),
        shape(
            5,
            "Notes Placeholder 4",
            "type=\"body\" sz=\"quarter\" idx=\"3\"",
            frame(685_800, 4_400_550, 5_486_400, 3_600_450),
            text,
            "",
        ),
        shape(
            6,
            "Footer Placeholder 5",
            "type=\"ftr\" sz=\"quarter\" idx=\"4\"",
            frame(0, 8_685_213, 2_971_800, 458_787),
            text,
            "",
        ),
        shape(
            7,
            "Slide Number Placeholder 6",
            "type=\"sldNum\" sz=\"quarter\" idx=\"5\"",
            frame(3_884_613, 8_685_213, 2_971_800, 458_787),
            text,
            "",
        ),
    ]
    .concat();
    format!(
        "{DECL}<p:notesMaster {}><p:cSld><p:spTree>{TREE_HEAD}{shapes}</p:spTree></p:cSld>\
         <p:clrMap bg1=\"lt1\" tx1=\"dk1\" bg2=\"lt2\" tx2=\"dk2\" accent1=\"accent1\" accent2=\"accent2\" \
         accent3=\"accent3\" accent4=\"accent4\" accent5=\"accent5\" accent6=\"accent6\" hlink=\"hlink\" folHlink=\"folHlink\"/>\
         </p:notesMaster>",
        namespaces(conformance)
    )
}

/// The notes master's own theme, authored here so it depends on nothing in the deck.
fn notes_theme_xml(conformance: Conformance) -> String {
    let dml = conformance.spell(DML_NS);
    let colors = [
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
    .map(|(n, v)| format!("<a:{n}><a:srgbClr val=\"{v}\"/></a:{n}>"))
    .concat();
    let fill = "<a:solidFill><a:schemeClr val=\"phClr\"/></a:solidFill>";
    let line = format!("<a:ln w=\"6350\">{fill}</a:ln>");
    let effect = "<a:effectStyle><a:effectLst/></a:effectStyle>";
    let font = "<a:latin typeface=\"Arial\"/><a:ea typeface=\"\"/><a:cs typeface=\"\"/>";
    format!(
        "{DECL}<a:theme xmlns:a=\"{dml}\" name=\"Notes\"><a:themeElements>\
         <a:clrScheme name=\"Notes\">{colors}</a:clrScheme>\
         <a:fontScheme name=\"Notes\"><a:majorFont>{font}</a:majorFont><a:minorFont>{font}</a:minorFont></a:fontScheme>\
         <a:fmtScheme name=\"Notes\"><a:fillStyleLst>{f}</a:fillStyleLst><a:lnStyleLst>{l}</a:lnStyleLst>\
         <a:effectStyleLst>{e}</a:effectStyleLst><a:bgFillStyleLst>{f}</a:bgFillStyleLst></a:fmtScheme>\
         </a:themeElements></a:theme>",
        f = fill.repeat(3),
        l = line.repeat(3),
        e = effect.repeat(3),
    )
}

impl Draft<'_> {
    /// The notes master, created if the deck has none, in one change: part, own theme,
    /// the presentation's relationship, `notesMasterIdLst`, both `Override`s, and a report
    /// entry (`INV-NM-02`, Inferred).
    pub(crate) fn ensure_notes_master(&mut self) -> Result<String, WriterError> {
        let main = self.main.clone();
        for rel in self.relationships(&main)? {
            if super::draft::rel_kind(&rel.rel_type) == "notesMaster"
                && let Some(part) = self.resolve(&main, &rel)
            {
                return Ok(part);
            }
        }
        let (cx, cy) = {
            let tree = self.tree(&main)?;
            let root = tree.root();
            let size = pml_child(tree, root, "notesSz");
            let read = |name: &str| {
                size.and_then(|s| tree.plain_attribute(s, name))
                    .and_then(|v| v.parse::<i64>().ok())
                    .filter(|v| *v > 0)
            };
            (
                read("cx").unwrap_or(6_858_000),
                read("cy").unwrap_or(9_144_000),
            )
        };
        let dir = "ppt/notesMasters";
        let master = self.families.mint(dir, "notesMaster", "xml")?;
        let theme = self.families.mint("ppt/theme", "theme", "xml")?;
        let conformance = self.conformance;
        self.add_part(
            &master,
            Some(&format!("{PML}notesMaster+xml")),
            notes_master_xml(conformance, cx, cy).into_bytes(),
        )?;
        self.add_part(
            &theme,
            Some(THEME_TYPE),
            notes_theme_xml(conformance).into_bytes(),
        )?;
        self.add_relationship(&master, "theme", &theme)?;
        let rel_id = self.add_relationship(&main, "notesMaster", &master)?;
        let list = self.ensure_list("notesMasterIdLst")?;
        self.append_id_entry(&main, list, "notesMasterId", None, &rel_id)?;
        self.log.created_notes_master = Some(master.clone());
        self.note(
            "CS-SAV-0010",
            "INV-NM-02",
            &master,
            "notes master created with the deck's first notes",
        );
        Ok(master)
    }

    /// Give a slide notes (`INV-NM-04`, Documented).
    pub(crate) fn add_notes(&mut self, slide: &SlideRef, text: &str) -> Result<(), WriterError> {
        let (_, entry) = self.slide(slide)?;
        let part = Self::slide_part(&entry)?;
        check_text(text, &part)?;
        for rel in self.relationships(&part)? {
            if super::draft::rel_kind(&rel.rel_type) == "notesSlide" {
                return Err(WriterError::BadOperation {
                    what: "the slide already has notes",
                    subject: part,
                });
            }
        }
        let master = self.ensure_notes_master()?;
        let notes = self.families.mint("ppt/notesSlides", "notesSlide", "xml")?;
        let conformance = self.conformance;
        self.add_part(
            &notes,
            Some(&format!("{PML}notesSlide+xml")),
            notes_slide_xml(conformance).into_bytes(),
        )?;
        {
            let tree = self.tree(&notes)?;
            let mut stack = vec![tree.root()];
            while let Some(el) = stack.pop() {
                if tree.local_name(el) == "t" {
                    tree.set_text(el, text);
                    break;
                }
                stack.extend(tree.child_elements(el));
            }
        }
        self.add_relationship(&notes, "notesMaster", &master)?;
        self.add_relationship(&notes, "slide", &part)?;
        self.add_relationship(&part, "notesSlide", &notes)?;
        let _ = rels_part_for(&notes);
        Ok(())
    }

    /// Take a slide's notes away. The master stays, present stays present (`INV-NM-05`,
    /// Inferred).
    pub(crate) fn remove_notes(&mut self, slide: &SlideRef) -> Result<(), WriterError> {
        let (_, entry) = self.slide(slide)?;
        let part = Self::slide_part(&entry)?;
        let mut found = false;
        for rel in self.relationships(&part)? {
            if super::draft::rel_kind(&rel.rel_type) == "notesSlide" {
                self.remove_relationship(&part, &rel.id)?;
                found = true;
            }
        }
        if !found {
            return Err(WriterError::BadOperation {
                what: "the slide has no notes",
                subject: part,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use casual_slide_ooxml::OoxmlLimits;

    #[test]
    fn every_authored_part_parses_in_both_forms() {
        for conformance in [Conformance::Transitional, Conformance::Strict] {
            for xml in [
                notes_slide_xml(conformance),
                notes_master_xml(conformance, 6_858_000, 9_144_000),
                notes_theme_xml(conformance),
            ] {
                XmlTree::parse(xml.as_bytes(), "t", &OoxmlLimits::default()).unwrap();
            }
        }
    }

    #[test]
    fn a_hostile_notes_page_size_cannot_overflow() {
        let xml = notes_master_xml(Conformance::Transitional, i64::MAX, i64::MAX);
        XmlTree::parse(xml.as_bytes(), "t", &OoxmlLimits::default()).unwrap();
    }

    #[test]
    fn text_xml_cannot_carry_is_refused() {
        assert!(check_text("a\u{1}b", "p").is_err());
        assert!(check_text("a\tb\nc", "p").is_ok());
    }
}
