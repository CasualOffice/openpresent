//! Patch a frame's table and the `tableStyles` part in place (ADR-013, Tier 1G).
//!
//! Rows, cells and grid columns match by position, and only what differs is touched: a
//! cell's own attributes, its `tcPr` through Tier 1E's writer, its `txBody` through Tier
//! 1F's. So `a16:rowId`, `a16:colId`, every `extLst` and whatever else the model does not
//! see stay where they are, and an unedited cell is not looked at.
//!
//! Adding or removing a table, a row, a cell or a grid column is refused as
//! `CS-EXP-0009`, and nothing is saved: those are edit operations, which change ids and
//! merges together, and belong to the writer tier and Phase 6.

use casual_slide_import::properties::{Elem, Findings, ImportLimits, ns};
use casual_slide_import::slide::SlideLimits;
use casual_slide_import::table::{TABLE_STYLES_RELATIONSHIP, import_table_styles};
use casual_slide_model::table::{
    Cell3D, Headers, Table, TableBackgroundStyle, TableBorder, TableCell, TableCellBorderStyle,
    TableCellProperties, TableCol, TablePartKind, TablePartStyle, TableProperties, TableRow,
    TableStyle, TableStyleCellStyle, TableStyleChoice, TableStyleEffect, TableStyleFill,
    TableStyleList, TableStyleTextStyle,
};
use casual_slide_model::theme::{FontCollection, FontRef, StyleMatrixRef, TextFont, ThemeableFont};
use casual_slide_ooxml::{Conformance, OoxmlLimits, PresentationPackage, Relationship, XmlTree};

use casual_slide_import::presentation::{PartLocation, locate_part};

use crate::d3::{bevel, light_rig};
use crate::drawing::{Node, Space, patch};
use crate::effect::effect_properties_node;
use crate::fill::fill_node;
use crate::line::line_node;
use crate::presentation::SaveOutcome;
use crate::properties::{ExportError, Writer};
use crate::slide::{SlideExportError, sync_ordered};
use crate::text::patch_text_body;

type Result<T = ()> = std::result::Result<T, ExportError>;

const TABLE_ORDER: &[&str] = &["tblPr", "tblGrid", "tr"];
const CELL_ORDER: &[&str] = &["txBody", "tcPr", "extLst"];
const TABLE_PROPERTIES_ORDER: &[&str] = &[
    "noFill",
    "solidFill",
    "gradFill",
    "blipFill",
    "pattFill",
    "grpFill",
    "effectLst",
    "effectDag",
    "tableStyle",
    "tableStyleId",
    "extLst",
];
const CELL_PROPERTIES_ORDER: &[&str] = &[
    "lnL",
    "lnR",
    "lnT",
    "lnB",
    "lnTlToBr",
    "lnBlToTr",
    "cell3D",
    "noFill",
    "solidFill",
    "gradFill",
    "blipFill",
    "pattFill",
    "grpFill",
    "headers",
    "extLst",
];
const STYLE_ORDER: &[&str] = &[
    "tblBg", "wholeTbl", "band1H", "band2H", "band1V", "band2V", "lastCol", "firstCol", "lastRow",
    "seCell", "swCell", "firstRow", "neCell", "nwCell", "extLst",
];
const PART_ORDER: &[&str] = &["tcTxStyle", "tcStyle"];
const TEXT_STYLE_ORDER: &[&str] = &[
    "fontRef",
    "font",
    "scrgbClr",
    "srgbClr",
    "hslClr",
    "sysClr",
    "schemeClr",
    "prstClr",
    "extLst",
];
const CELL_STYLE_ORDER: &[&str] = &["tcBdr", "fill", "fillRef", "cell3D"];
const BORDER_ORDER: &[&str] = &[
    "left", "right", "top", "bottom", "insideH", "insideV", "tl2br", "tr2bl", "extLst",
];
const BACKGROUND_ORDER: &[&str] = &["fill", "fillRef", "effect", "effectRef"];
const CELL3D_ORDER: &[&str] = &["bevel", "lightRig", "extLst"];

/// The schema's sequence for a table element, for a new child among its siblings.
pub(crate) fn order_in(parent: &str) -> Option<&'static [&'static str]> {
    Some(match parent {
        "tblPr" => TABLE_PROPERTIES_ORDER,
        "tcPr" => CELL_PROPERTIES_ORDER,
        "tblStyle" | "tableStyle" => STYLE_ORDER,
        "tcTxStyle" => TEXT_STYLE_ORDER,
        "tcStyle" => CELL_STYLE_ORDER,
        "tcBdr" => BORDER_ORDER,
        "tblBg" => BACKGROUND_ORDER,
        "cell3D" => CELL3D_ORDER,
        _ if TablePartKind::from_xml(parent).is_some() => PART_ORDER,
        _ => return None,
    })
}

/// Children of a table element of which one may be present, so changing which replaces.
/// By parent, since `effect`, `fill` and `ln` mean other things elsewhere.
pub(crate) fn choices_in(parent: &str) -> &'static [&'static [&'static str]] {
    match parent {
        "tblPr" => &[&["tableStyle", "tableStyleId"]],
        "tcStyle" => &[&["fill", "fillRef"]],
        "tblBg" => &[&["fill", "fillRef"], &["effect", "effectRef"]],
        "tcTxStyle" => &[&["font", "fontRef"]],
        "left" | "right" | "top" | "bottom" | "insideH" | "insideV" | "tl2br" | "tr2bl" => {
            &[&["ln", "lnRef"]]
        }
        _ => &[],
    }
}

fn unsupported(w: &Writer<'_>, what: &'static str) -> ExportError {
    ExportError::UnsupportedEdit {
        what,
        part: w.part.to_owned(),
    }
}

fn malformed(w: &Writer<'_>) -> ExportError {
    ExportError::MalformedElement {
        part: w.part.to_owned(),
    }
}

// spec-audit: owner=tcPr,tcStyle
fn cell3d(c: &Cell3D) -> Node {
    Node::a("cell3D")
        .lex("prstMaterial", c.prst_material.as_ref())
        .opt(c.bevel.as_ref().map(|b| bevel("bevel", b)))
        .opt(c.light_rig.as_ref().map(light_rig))
}

// spec-audit: owner=tcPr
fn headers(h: &Headers) -> Node {
    Node::a("headers").list(
        h.header
            .iter()
            .map(|h| Node::a("header").text(Some(h.clone()))),
    )
}

fn tc_pr(p: &TableCellProperties) -> Node {
    let line =
        |local, l: &Option<Box<_>>| l.as_ref().map(|l: &Box<_>| line_node(l, Space::Dml, local));
    let list = p.headers.as_ref().map(headers);
    Node::a("tcPr")
        .lex("marL", p.mar_l.as_ref())
        .lex("marR", p.mar_r.as_ref())
        .lex("marT", p.mar_t.as_ref())
        .lex("marB", p.mar_b.as_ref())
        .lex("vert", p.vert.as_ref())
        .lex("anchor", p.anchor.as_ref())
        .lex("anchorCtr", p.anchor_ctr.as_ref())
        .lex("horzOverflow", p.horz_overflow.as_ref())
        .opt(line("lnL", &p.ln_l))
        .opt(line("lnR", &p.ln_r))
        .opt(line("lnT", &p.ln_t))
        .opt(line("lnB", &p.ln_b))
        .opt(line("lnTlToBr", &p.ln_tl_to_br))
        .opt(line("lnBlToTr", &p.ln_bl_to_tr))
        .opt(p.cell3d.as_ref().map(cell3d))
        .opt(p.fill.as_ref().map(fill_node))
        .opt(list)
}

fn matrix_ref(local: &'static str, r: &StyleMatrixRef) -> Node {
    Node::a(local)
        .lex("idx", r.idx.as_ref())
        .color(r.color.as_ref())
}

fn font_ref(r: &FontRef) -> Node {
    let idx = r.idx.map(|i| i.as_xml().to_owned());
    Node::a("fontRef")
        .lex("idx", idx.as_ref())
        .color(r.color.as_ref())
}

fn text_font(local: &'static str, f: &TextFont) -> Node {
    Node::a(local)
        .lex("typeface", f.typeface.as_ref())
        .lex("panose", f.panose.as_ref())
        .lex("pitchFamily", f.pitch_family.as_ref())
        .lex("charset", f.charset.as_ref())
}

fn font_collection(f: &FontCollection) -> Node {
    Node::a("font")
        .opt(f.latin.as_ref().map(|x| text_font("latin", x)))
        .opt(f.ea.as_ref().map(|x| text_font("ea", x)))
        .opt(f.cs.as_ref().map(|x| text_font("cs", x)))
        .list(f.fonts.iter().map(|s| {
            Node::a("font")
                .lex("script", s.script.as_ref())
                .lex("typeface", s.typeface.as_ref())
        }))
}

fn style_fill(f: &TableStyleFill) -> Node {
    match f {
        TableStyleFill::Inline(fill) => Node::a("fill").kid(fill_node(fill)),
        TableStyleFill::Ref(r) => matrix_ref("fillRef", r),
    }
}

fn style_effect(e: &TableStyleEffect) -> Node {
    match e {
        TableStyleEffect::Inline(effects) => Node::a("effect").kid(effect_properties_node(effects)),
        TableStyleEffect::Ref(r) => matrix_ref("effectRef", r),
    }
}

fn border(local: &'static str, b: &TableBorder) -> Node {
    let node = Node::a(local);
    match b {
        TableBorder::Empty => node,
        TableBorder::Line(l) => node.kid(line_node(l, Space::Dml, "ln")),
        TableBorder::Ref(r) => node.kid(matrix_ref("lnRef", r)),
    }
}

fn tc_bdr(b: &TableCellBorderStyle) -> Node {
    let one = |local, b: &Option<TableBorder>| b.as_ref().map(|b| border(local, b));
    Node::a("tcBdr")
        .opt(one("left", &b.left))
        .opt(one("right", &b.right))
        .opt(one("top", &b.top))
        .opt(one("bottom", &b.bottom))
        .opt(one("insideH", &b.inside_h))
        .opt(one("insideV", &b.inside_v))
        .opt(one("tl2br", &b.tl2br))
        .opt(one("tr2bl", &b.tr2bl))
}

// spec-audit: owner=wholeTbl,band1H,band2H,band1V,band2V,lastCol,firstCol,lastRow,seCell,swCell,firstRow,neCell,nwCell
fn tc_style(c: &TableStyleCellStyle) -> Node {
    Node::a("tcStyle")
        .opt(c.tc_bdr.as_ref().map(tc_bdr))
        .opt(c.fill.as_ref().map(style_fill))
        .opt(c.cell3d.as_ref().map(cell3d))
}

// spec-audit: owner=wholeTbl,band1H,band2H,band1V,band2V,lastCol,firstCol,lastRow,seCell,swCell,firstRow,neCell,nwCell
fn tc_tx_style(t: &TableStyleTextStyle) -> Node {
    let b = t.bold.map(|v| v.as_xml().to_owned());
    let i = t.italic.map(|v| v.as_xml().to_owned());
    let font = t.font.as_ref().map(|f| match f {
        ThemeableFont::Inline(c) => font_collection(c),
        ThemeableFont::Ref(r) => font_ref(r),
    });
    Node::a("tcTxStyle")
        .lex("b", b.as_ref())
        .lex("i", i.as_ref())
        .opt(font)
        .color(t.color.as_ref())
}

fn part_style(local: &'static str, p: &TablePartStyle) -> Node {
    Node::a(local)
        .opt(p.tc_tx_style.as_ref().map(tc_tx_style))
        .opt(p.tc_style.as_ref().map(tc_style))
}

fn tbl_bg(b: &TableBackgroundStyle) -> Node {
    Node::a("tblBg")
        .opt(b.fill.as_ref().map(style_fill))
        .opt(b.effect.as_ref().map(style_effect))
}

// spec-audit: a:CT_TableStyle/wholeTbl
// spec-audit: a:CT_TableStyle/band1H
// spec-audit: a:CT_TableStyle/band2H
// spec-audit: a:CT_TableStyle/band1V
// spec-audit: a:CT_TableStyle/band2V
// spec-audit: a:CT_TableStyle/lastCol
// spec-audit: a:CT_TableStyle/firstCol
// spec-audit: a:CT_TableStyle/lastRow
// spec-audit: a:CT_TableStyle/seCell
// spec-audit: a:CT_TableStyle/swCell
// spec-audit: a:CT_TableStyle/firstRow
// spec-audit: a:CT_TableStyle/neCell
// spec-audit: a:CT_TableStyle/nwCell
fn tbl_style(local: &'static str, s: &TableStyle) -> Node {
    Node::a(local)
        .lex("styleId", s.style_id.as_ref())
        .lex("styleName", s.style_name.as_ref())
        .opt(s.tbl_bg.as_ref().map(tbl_bg))
        .list(
            TablePartKind::ALL
                .iter()
                .filter_map(|k| s.part(*k).map(|p| part_style(k.as_xml(), p))),
        )
}

fn tbl_pr(p: &TableProperties) -> Node {
    let style = p.style.as_ref().map(|s| match s {
        TableStyleChoice::Inline(style) => tbl_style("tableStyle", style),
        TableStyleChoice::Id(id) => Node::a("tableStyleId").text(Some(id.clone())),
    });
    Node::a("tblPr")
        .lex("rtl", p.rtl.as_ref())
        .lex("firstRow", p.first_row.as_ref())
        .lex("firstCol", p.first_col.as_ref())
        .lex("lastRow", p.last_row.as_ref())
        .lex("lastCol", p.last_col.as_ref())
        .lex("bandRow", p.band_row.as_ref())
        .lex("bandCol", p.band_col.as_ref())
        .opt(p.fill.as_ref().map(fill_node))
        .opt(p.effects.as_ref().map(effect_properties_node))
        .opt(style)
}

fn grid_col(c: &TableCol) -> Node {
    Node::a("gridCol").lex("w", c.width.as_ref())
}

fn tc(c: &TableCell) -> Node {
    Node::a("tc")
        .lex("rowSpan", c.row_span.as_ref())
        .lex("gridSpan", c.grid_span.as_ref())
        .lex("hMerge", c.h_merge.as_ref())
        .lex("vMerge", c.v_merge.as_ref())
        .lex("id", c.id.as_ref())
}

/// The elements of this name among a parent's, which must be as many as the model read.
fn counted(w: &Writer<'_>, parent: &Elem, local: &str, expected: usize) -> Result<Vec<Elem>> {
    let found: Vec<Elem> = parent
        .children(w.tree, w.part)?
        .into_iter()
        .filter(|c| c.is(ns::DML, local))
        .collect();
    if found.len() == expected {
        Ok(found)
    } else {
        Err(malformed(w))
    }
}

// spec-audit: owner=tc
fn patch_cell(w: &mut Writer<'_>, el: &Elem, b: &TableCell, a: &TableCell) -> Result {
    patch(w, el, Some(&tc(b)), &tc(a))?;
    sync_ordered(
        w,
        el,
        ns::DML,
        "a",
        CELL_ORDER,
        "txBody",
        b.tx_body.as_ref(),
        a.tx_body.as_ref(),
        |w, el, before, after| patch_text_body(w, el, before.map(|b| &**b), after),
    )?;
    sync_ordered(
        w,
        el,
        ns::DML,
        "a",
        CELL_ORDER,
        "tcPr",
        b.tc_pr.as_ref(),
        a.tc_pr.as_ref(),
        |w, el, before, after| {
            let before = before.map(|b| tc_pr(b));
            patch(w, el, before.as_ref(), &tc_pr(after))
        },
    )
}

/// A `tr`'s `tc` elements.
fn cells_of(w: &Writer<'_>, tr: &Elem, expected: usize) -> Result<Vec<Elem>> {
    match tr.local.as_str() {
        "tr" => counted(w, tr, "tc", expected),
        _ => Err(malformed(w)),
    }
}

fn tr(r: &TableRow) -> Node {
    Node::a("tr").lex("h", r.height.as_ref())
}

fn patch_tr(w: &mut Writer<'_>, el: &Elem, b: &TableRow, a: &TableRow) -> Result {
    if b.cells.len() != a.cells.len() {
        return Err(unsupported(w, "a cell added to or removed from a row"));
    }
    patch(w, el, Some(&tr(b)), &tr(a))?;
    if b.cells == a.cells {
        return Ok(());
    }
    let cells = cells_of(w, el, b.cells.len())?;
    for ((element, before), after) in cells.iter().zip(&b.cells).zip(&a.cells) {
        if before != after {
            patch_cell(w, element, before, after)?;
        }
    }
    Ok(())
}

// spec-audit: a:<tbl>
fn patch_table(w: &mut Writer<'_>, el: &Elem, b: &Table, a: &Table) -> Result {
    if b.rows.len() != a.rows.len() {
        return Err(unsupported(w, "a row added to or removed from a table"));
    }
    sync_ordered(
        w,
        el,
        ns::DML,
        "a",
        TABLE_ORDER,
        "tblPr",
        b.tbl_pr.as_ref(),
        a.tbl_pr.as_ref(),
        |w, el, before, after| {
            let before = before.map(|b| tbl_pr(b));
            patch(w, el, before.as_ref(), &tbl_pr(after))
        },
    )?;
    if b.tbl_grid != a.tbl_grid {
        let (Some(before), Some(after)) = (&b.tbl_grid, &a.tbl_grid) else {
            return Err(unsupported(w, "a table's grid added or removed"));
        };
        let (before, after) = (&before.grid_col, &after.grid_col);
        if before.len() != after.len() {
            return Err(unsupported(w, "a column added to or removed from a grid"));
        }
        let grid = el
            .first_child(w.tree, w.part, ns::DML, "tblGrid")?
            .ok_or_else(|| malformed(w))?;
        let columns = counted(w, &grid, "gridCol", before.len())?;
        for ((col, x), y) in columns.iter().zip(before).zip(after) {
            patch(w, col, Some(&grid_col(x)), &grid_col(y))?;
        }
    }
    if b.rows != a.rows {
        let rows = counted(w, el, "tr", b.rows.len())?;
        for ((element, before), after) in rows.iter().zip(&b.rows).zip(&a.rows) {
            if before != after {
                patch_tr(w, element, before, after)?;
            }
        }
    }
    Ok(())
}

/// A frame's table, inside `a:graphic/a:graphicData`.
pub(crate) fn patch_frame_table(
    w: &mut Writer<'_>,
    frame: &Elem,
    b: Option<&Table>,
    a: Option<&Table>,
) -> Result {
    let (Some(b), Some(a)) = (b, a) else {
        return Err(unsupported(w, "a table added to or removed from a frame"));
    };
    let find = |w: &Writer<'_>, parent: &Elem, uris: &[&str], local: &str| {
        parent
            .first_child(w.tree, w.part, uris, local)?
            .ok_or_else(|| malformed(w))
    };
    let graphic = find(w, frame, ns::DML, "graphic")?;
    let data = find(w, &graphic, ns::DML, "graphicData")?;
    let table = find(w, &data, ns::DML, "tbl")?;
    patch_table(w, &table, b, a)
}

// spec-audit: owner=tblStyleLst
// spec-audit: a:<tblStyleLst>
fn list_node(l: &TableStyleList) -> Node {
    Node::a("tblStyleLst")
        .lex("def", l.def.as_ref())
        .list(l.tbl_style.iter().map(|s| tbl_style("tblStyle", s)))
}

/// Patch a `tableStyles` tree to say `after`. Returns the nodes changed; with 0, the tree
/// is as it was. What was asked is read back before returning.
#[allow(clippy::too_many_arguments)]
pub fn patch_table_styles(
    tree: &mut XmlTree,
    part: &str,
    strict: bool,
    rels: &[Relationship],
    after: &TableStyleList,
    limits: &SlideLimits,
    import_limits: &ImportLimits,
    xml_limits: &OoxmlLimits,
) -> std::result::Result<usize, SlideExportError> {
    let read = |tree: &XmlTree| {
        let mut findings = Findings::new(import_limits.max_findings);
        let mut color = casual_slide_import::color::Findings::default();
        import_table_styles(
            tree,
            part,
            rels,
            limits,
            import_limits,
            &mut findings,
            &mut color,
        )
    };
    let before = read(tree)?;
    if &before == after {
        return Ok(0);
    }
    let root = Elem::root(tree, part).map_err(ExportError::from)?;
    let mut w = Writer {
        tree,
        part,
        strict,
        known_rels: Some(rels.iter().map(|r| r.id.clone()).collect()),
        changes: 0,
    };
    patch(&mut w, &root, Some(&list_node(&before)), &list_node(after))?;
    let changes = w.changes;
    let again = XmlTree::parse(&tree.to_bytes(), part, xml_limits).map_err(ExportError::from)?;
    if &read(&again)? != after {
        return Err(SlideExportError::Export(ExportError::RoundTrip {
            part: part.to_owned(),
        }));
    }
    Ok(changes)
}

/// Save a table style list over the package it came from. Only its part changes; with
/// nothing changed, the input bytes come back. A deck with no such part is refused
/// as `CS-EXP-0005`'s sibling: adding a part is the writer tier's.
pub fn save_table_styles(
    package: &mut PresentationPackage,
    after: &TableStyleList,
    limits: &SlideLimits,
    import_limits: &ImportLimits,
) -> std::result::Result<SaveOutcome, SlideExportError> {
    let main = package.main_part().to_owned();
    let mut findings = Findings::new(import_limits.max_findings);
    let PartLocation::At(part) =
        locate_part(package, &main, TABLE_STYLES_RELATIONSHIP, &mut findings)
    else {
        return Err(SlideExportError::Unsupported {
            what: "a table style list where the deck has no such part",
            part: main,
        });
    };
    let strict = package.conformance() == Conformance::Strict;
    let xml = *package.limits();
    let bytes = package.read_part(&part).map_err(ExportError::from)?;
    let mut tree = XmlTree::parse(&bytes, &part, &xml).map_err(ExportError::from)?;
    let rels = package.relationships_of(&part).to_vec();
    let changes = patch_table_styles(
        &mut tree,
        &part,
        strict,
        &rels,
        after,
        limits,
        import_limits,
        &xml,
    )?;
    let out = tree.to_bytes();
    if changes == 0 || out == bytes {
        return Ok(SaveOutcome {
            bytes: package.save_unedited(),
            changed_parts: Vec::new(),
            signature_invalidated: false,
        });
    }
    // Through the writer, so a signature, a thumbnail and a count are what any edit leaves.
    let outcome = crate::writer::SavePlan::new()
        .acknowledge_signature()
        .replace_part(part, out)
        .save(package)
        .map_err(|err| match err {
            crate::writer::WriterError::Export(err) => err,
            other => ExportError::Refused(Box::new(other)),
        })?;
    Ok(SaveOutcome {
        bytes: outcome.bytes,
        changed_parts: outcome.changed_parts,
        signature_invalidated: outcome.signature_invalidated,
    })
}
