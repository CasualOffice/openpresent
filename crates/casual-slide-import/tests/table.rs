//! Tier 1G import: every row of `docs/spec/TABLE.md`, read from the checklist at test
//! time; merges; the `tableStyles` part; an unknown GUID; hostile tables; nothing from a
//! document in a diagnostic.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use casual_slide_import::color::Findings as ColorFindings;
use casual_slide_import::presentation::import_deck;
use casual_slide_import::properties::{Findings, ImportFindingKind as K, ImportLimits};
use casual_slide_import::slide::{
    ImportedSlides, SlideError, SlideLimits, import_slide_part, import_slide_parts,
};
use casual_slide_import::table::{TableBound, TableFinding, TableLimits, import_table_styles};
use casual_slide_model::fill::Fill;
use casual_slide_model::shape::{NodeKind, ShapeNode};
use casual_slide_model::slide::{SlideKind, SlidePart};
use casual_slide_model::table::{
    MergeFault, Table, TableBorder, TableCell, TableMerge, TablePartKind, TablePartStyle,
    TableStyle, TableStyleChoice, TableStyleEffect, TableStyleFill, TableStyleList,
};
use casual_slide_model::text::TextItem;
use casual_slide_model::theme::ThemeableFont;
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage, XmlTree};

const NS: &str = concat!(
    r#"xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" "#,
    r#"xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" "#,
    r#"xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main""#
);
const EVERY: &str = "generated/table-every-property.pptx";
const STYLE_ID: &str = "{1C8F3A2E-5B6D-4E7F-8A9B-0C1D2E3F4A5B}";

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn deck_with(path: &str, limits: &SlideLimits) -> Result<ImportedSlides, SlideError> {
    let bytes = std::fs::read(fixtures().join(path)).unwrap();
    let mut pkg = PresentationPackage::open(bytes, OoxmlLimits::default()).unwrap();
    let import = ImportLimits::default();
    let presentation = import_deck(&mut pkg, &import).unwrap().deck.presentation;
    import_slide_parts(&mut pkg, &presentation, limits, &import)
}

fn deck(path: &str) -> ImportedSlides {
    deck_with(path, &SlideLimits::default()).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn first_slide(deck: &ImportedSlides) -> &SlidePart {
    deck.parts
        .get(*deck.parts.structure.slides.first().unwrap())
        .unwrap()
}

fn frame<'a>(part: &'a SlidePart, name: &str) -> &'a ShapeNode {
    part.shape_tree()
        .unwrap()
        .descendants()
        .find(|n| n.non_visual.as_ref().and_then(|v| v.name.as_deref()) == Some(name))
        .unwrap_or_else(|| panic!("no {name}"))
}

fn table_of<'a>(part: &'a SlidePart, name: &str) -> &'a Table {
    frame(part, name).table.as_ref().unwrap()
}

fn slide_with_table(table: &str) -> String {
    format!(
        r#"<p:sld {NS}><p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/><p:graphicFrame><p:nvGraphicFramePr><p:cNvPr id="2" name="t"/><p:cNvGraphicFramePr/><p:nvPr/></p:nvGraphicFramePr><p:xfrm/><a:graphic><a:graphicData uri="u">{table}</a:graphicData></a:graphic></p:graphicFrame></p:spTree></p:cSld></p:sld>"#
    )
}

fn read_table(table: &str) -> (Table, Findings) {
    let xml = slide_with_table(table);
    let tree = XmlTree::parse(xml.as_bytes(), "s.xml", &OoxmlLimits::default()).unwrap();
    let limits = ImportLimits::default();
    let mut findings = Findings::new(limits.max_findings);
    let mut color = ColorFindings::default();
    let part = import_slide_part(
        &tree,
        "s.xml",
        &[],
        SlideKind::Slide,
        0,
        &SlideLimits::default(),
        &limits,
        &mut findings,
        &mut color,
    )
    .unwrap();
    let node = &part.shape_tree().unwrap().children[0];
    assert_eq!(node.kind, NodeKind::GraphicFrame);
    ((**node.table.as_ref().unwrap()).clone(), findings)
}

fn read_styles(body: &str) -> (TableStyleList, Findings) {
    let xml = format!(r#"<a:tblStyleLst {NS} def="{STYLE_ID}">{body}</a:tblStyleLst>"#);
    let tree = XmlTree::parse(xml.as_bytes(), "t.xml", &OoxmlLimits::default()).unwrap();
    let limits = ImportLimits::default();
    let mut findings = Findings::new(limits.max_findings);
    let mut color = ColorFindings::default();
    let list = import_table_styles(
        &tree,
        "t.xml",
        &[],
        &SlideLimits::default(),
        &limits,
        &mut findings,
        &mut color,
    )
    .unwrap();
    (list, findings)
}

fn table_rows() -> Vec<BTreeMap<String, String>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/spec/checklist.tsv");
    let text = std::fs::read_to_string(path).unwrap();
    let mut lines = text.lines();
    let names: Vec<String> = lines
        .next()
        .unwrap()
        .split('\t')
        .map(str::to_owned)
        .collect();
    lines
        .map(|l| {
            names
                .iter()
                .cloned()
                .zip(l.split('\t').map(str::to_owned))
                .collect::<BTreeMap<_, _>>()
        })
        .filter(|r| r["area"] == "TABLE")
        .collect()
}

/// The `extLst` of this owner, on its own, is reported and stays out of the model.
fn extension_is_reported(owner: &str) -> bool {
    const EXT: &str = r#"<a:extLst><a:ext uri="u"/></a:extLst>"#;
    let cell = |inner: &str| {
        format!(
            r#"<a:tbl><a:tblGrid><a:gridCol w="1"/></a:tblGrid><a:tr h="1">{inner}</a:tr></a:tbl>"#
        )
    };
    let in_table = |table: String| read_table(&table).1.has(K::RetainedElement, "extLst");
    let in_style = |style: String| read_styles(&style).1.has(K::RetainedElement, "extLst");
    match owner {
        "CT_TableCellProperties" => in_table(cell(&format!("<a:tc><a:tcPr>{EXT}</a:tcPr></a:tc>"))),
        "CT_TableCell" => in_table(cell(&format!("<a:tc>{EXT}</a:tc>"))),
        "CT_TableRow" => in_table(cell(&format!("<a:tc/>{EXT}"))),
        "CT_TableCol" => in_table(format!(
            r#"<a:tbl><a:tblGrid><a:gridCol w="1">{EXT}</a:gridCol></a:tblGrid></a:tbl>"#
        )),
        "CT_TableProperties" => in_table(format!(
            r#"<a:tbl><a:tblPr>{EXT}</a:tblPr><a:tblGrid/></a:tbl>"#
        )),
        "CT_Cell3D" => in_table(cell(&format!(
            "<a:tc><a:tcPr><a:cell3D><a:bevel/>{EXT}</a:cell3D></a:tcPr></a:tc>"
        ))),
        "CT_TableStyleTextStyle" => in_style(format!(
            r#"<a:tblStyle><a:wholeTbl><a:tcTxStyle>{EXT}</a:tcTxStyle></a:wholeTbl></a:tblStyle>"#
        )),
        "CT_TableCellBorderStyle" => in_style(format!(
            r#"<a:tblStyle><a:wholeTbl><a:tcStyle><a:tcBdr>{EXT}</a:tcBdr></a:tcStyle></a:wholeTbl></a:tblStyle>"#
        )),
        "CT_TableStyle" => in_style(format!("<a:tblStyle>{EXT}</a:tblStyle>")),
        other => panic!("an extLst on {other}, with no case here"),
    }
}

struct Every<'a> {
    table: &'a Table,
    inline: &'a Table,
    list: &'a TableStyleList,
}

impl Every<'_> {
    fn cell(&self) -> &TableCell {
        &self.table.rows[0].cells[0]
    }
    fn style(&self) -> &TableStyle {
        &self.list.styles[0]
    }
    fn whole(&self) -> &TablePartStyle {
        self.style().part(TablePartKind::WholeTable).unwrap()
    }
    fn border(&self, name: &str) -> &TableBorder {
        let b = self
            .whole()
            .cell
            .as_ref()
            .unwrap()
            .borders
            .as_ref()
            .unwrap();
        match name {
            "left" => b.left.as_ref(),
            "right" => b.right.as_ref(),
            "top" => b.top.as_ref(),
            "bottom" => b.bottom.as_ref(),
            "insideH" => b.inside_horizontal.as_ref(),
            "insideV" => b.inside_vertical.as_ref(),
            "tl2br" => b.top_left_to_bottom_right.as_ref(),
            _ => b.top_right_to_bottom_left.as_ref(),
        }
        .unwrap()
    }
}

/// Whether the model holds this row's construct, as the fixture states it. Unknown rows
/// panic, so a row added to the checklist fails this until it is handled.
fn modelled(e: &Every<'_>, owner: &str, name: &str) -> bool {
    let cell = e.cell();
    let p = cell.properties.as_ref().unwrap();
    let t = e.table.properties.as_ref().unwrap();
    let td = e.inline.properties.as_ref().unwrap();
    let text = e.whole().text.as_ref().unwrap();
    let tc_style = e.whole().cell.as_ref().unwrap();
    let list = e.list;
    match (owner, name) {
        ("CT_TableCellProperties", "lnL") => p.line_left.is_some(),
        ("CT_TableCellProperties", "lnR") => p.line_right.is_some(),
        ("CT_TableCellProperties", "lnT") => p.line_top.is_some(),
        ("CT_TableCellProperties", "lnB") => p.line_bottom.is_some(),
        ("CT_TableCellProperties", "lnTlToBr") => p.line_top_left_to_bottom_right.is_some(),
        ("CT_TableCellProperties", "lnBlToTr") => p.line_bottom_left_to_top_right.is_some(),
        ("CT_TableCellProperties", "cell3D") => p.cell3d.is_some(),
        ("CT_TableCellProperties", "EG_FillProperties") => matches!(p.fill, Some(Fill::Solid(_))),
        ("CT_TableCellProperties", "headers") => p.headers.is_some(),
        ("CT_TableCellProperties", "@marL") => p.margin_left.is_some(),
        ("CT_TableCellProperties", "@marR") => {
            p.margin_right.as_ref().and_then(|m| m.written()) == Some("1in")
        }
        ("CT_TableCellProperties", "@marT") => p.margin_top.is_some(),
        ("CT_TableCellProperties", "@marB") => {
            p.margin_bottom.as_ref().and_then(|m| m.written()) == Some("2.5cm")
        }
        ("CT_TableCellProperties", "@vert") => p.vertical.is_some(),
        ("CT_TableCellProperties", "@anchor") => p.anchor.is_some(),
        ("CT_TableCellProperties", "@anchorCtr") => p.anchor_center == Some(true),
        ("CT_TableCellProperties", "@horzOverflow") => p.horizontal_overflow.is_some(),
        ("CT_Headers", "header") => p.headers.as_ref().is_some_and(|h| h == &["h1", "h2"]),
        ("CT_TableCol", "@w") => e.table.grid.as_ref().unwrap()[0].width.is_some(),
        ("CT_TableGrid", "gridCol") => e.table.column_count() == 2,
        ("CT_TableCell", "txBody") => cell.text.is_some(),
        ("CT_TableCell", "tcPr") => cell.properties.is_some(),
        ("CT_TableCell", "@rowSpan") => cell.row_span == Some(1),
        ("CT_TableCell", "@gridSpan") => cell.grid_span == Some(1),
        ("CT_TableCell", "@hMerge") => cell.h_merge == Some(false),
        ("CT_TableCell", "@vMerge") => cell.v_merge == Some(false),
        ("CT_TableCell", "@id") => cell.id.as_deref() == Some("cell-1"),
        ("CT_TableRow", "tc") => e.table.rows[0].cells.len() == 2,
        ("CT_TableRow", "@h") => e.table.rows[0].height.is_some(),
        ("CT_TableProperties", "EG_FillProperties") => {
            matches!(t.fill, Some(Fill::Solid(_))) && matches!(td.fill, Some(Fill::None))
        }
        ("CT_TableProperties", "EG_EffectProperties") => {
            t.effects.is_some() && td.effects.is_some()
        }
        ("CT_TableProperties", "tableStyle") => {
            matches!(td.style, Some(TableStyleChoice::Inline(_)))
        }
        ("CT_TableProperties", "tableStyleId") => {
            matches!(&t.style, Some(TableStyleChoice::Id(id)) if id == STYLE_ID)
        }
        ("CT_TableProperties", "@rtl") => t.rtl == Some(true),
        ("CT_TableProperties", "@firstRow") => t.first_row == Some(true),
        ("CT_TableProperties", "@firstCol") => t.first_column == Some(true),
        ("CT_TableProperties", "@lastRow") => t.last_row == Some(true),
        ("CT_TableProperties", "@lastCol") => t.last_column == Some(true),
        ("CT_TableProperties", "@bandRow") => t.band_row == Some(true),
        ("CT_TableProperties", "@bandCol") => t.band_column == Some(true),
        ("CT_Table", "tblPr") => e.table.properties.is_some(),
        ("CT_Table", "tblGrid") => e.table.grid.is_some(),
        ("CT_Table", "tr") => e.table.rows.len() == 2,
        ("CT_Cell3D", "bevel") => p.cell3d.is_some_and(|c| c.bevel.is_some()),
        ("CT_Cell3D", "lightRig") => p.cell3d.is_some_and(|c| c.light_rig.is_some()),
        ("CT_Cell3D", "@prstMaterial") => p.cell3d.is_some_and(|c| c.material.is_some()),
        ("CT_ThemeableLineStyle", "ln") => matches!(e.border("left"), TableBorder::Line(_)),
        ("CT_ThemeableLineStyle", "lnRef") => matches!(e.border("right"), TableBorder::Ref(_)),
        ("CT_TableStyleTextStyle", "EG_ThemeableFontStyles") => {
            let plain = e.style().part(TablePartKind::Band1Horizontal).unwrap();
            let plain = plain.text.as_ref().unwrap();
            matches!(text.font, Some(ThemeableFont::Inline(_)))
                && matches!(plain.font, Some(ThemeableFont::Ref(_)))
        }
        ("CT_TableStyleTextStyle", "EG_ColorChoice") => text.color.is_some(),
        ("CT_TableStyleTextStyle", "@b") => text.bold.is_some(),
        ("CT_TableStyleTextStyle", "@i") => text.italic.is_some(),
        ("CT_TableCellBorderStyle", n @ ("left" | "right" | "top" | "bottom" | "insideH")) => {
            !matches!(e.border(n), TableBorder::Empty)
        }
        ("CT_TableCellBorderStyle", n @ ("insideV" | "tl2br" | "tr2bl")) => {
            !matches!(e.border(n), TableBorder::Empty)
        }
        ("CT_TableBackgroundStyle", "EG_ThemeableFillStyle") => {
            let one = e.style().background.as_ref().unwrap();
            let two = list.styles[1].background.as_ref().unwrap();
            matches!(one.fill, Some(TableStyleFill::Inline(_)))
                && matches!(two.fill, Some(TableStyleFill::Ref(_)))
        }
        ("CT_TableBackgroundStyle", "EG_ThemeableEffectStyle") => {
            let one = e.style().background.as_ref().unwrap();
            let two = list.styles[1].background.as_ref().unwrap();
            matches!(one.effect, Some(TableStyleEffect::Ref(_)))
                && matches!(two.effect, Some(TableStyleEffect::Inline(_)))
        }
        ("CT_TableStyleCellStyle", "tcBdr") => tc_style.borders.is_some(),
        ("CT_TableStyleCellStyle", "EG_ThemeableFillStyle") => {
            let plain = e.style().part(TablePartKind::Band1Horizontal).unwrap();
            matches!(tc_style.fill, Some(TableStyleFill::Ref(_)))
                && matches!(
                    plain.cell.as_ref().unwrap().fill,
                    Some(TableStyleFill::Inline(_))
                )
        }
        ("CT_TableStyleCellStyle", "cell3D") => tc_style.cell3d.is_some(),
        ("CT_TablePartStyle", "tcTxStyle") => e.whole().text.is_some(),
        ("CT_TablePartStyle", "tcStyle") => e.whole().cell.is_some(),
        ("CT_TableStyle", "tblBg") => e.style().background.is_some(),
        ("CT_TableStyle", "@styleId") => e.style().id.as_deref() == Some(STYLE_ID),
        ("CT_TableStyle", "@styleName") => e.style().name.as_deref() == Some("Fixture Style"),
        ("CT_TableStyle", part) => {
            let kind = TablePartKind::from_xml(part)
                .unwrap_or_else(|| panic!("CT_TableStyle/{part}, with no case here"));
            e.style().part(kind).is_some()
        }
        ("CT_TableStyleList", "tblStyle") => list.styles.len() == 2,
        ("CT_TableStyleList", "@def") => list.default_style.as_deref() == Some(STYLE_ID),
        other => panic!("{other:?} in the TABLE rows, with no case here"),
    }
}

/// The acceptance test. Rows come from the checklist, so one added there with no case
/// here fails. A row is either in the model or, for `extLst`, reported.
#[test]
fn every_table_row_in_the_checklist_is_read_or_reported() {
    let d = deck(EVERY);
    let slide = first_slide(&d);
    let every = Every {
        table: table_of(slide, "Every Property"),
        inline: table_of(slide, "Inline Style"),
        list: d.table_styles.as_ref().unwrap(),
    };
    let rows = table_rows();
    assert_eq!(rows.len(), 113, "docs/spec/TABLE.md says 113");
    let (mut read, mut reported, mut declared) = (0, 0, 0);
    for row in &rows {
        let (id, owner, name, kind) = (&row["id"], &row["owner"], &row["name"], &row["kind"]);
        match kind.as_str() {
            // A type, or a root: its members are rows too. Say it is one this reads.
            "type" | "root" => {
                let known = [
                    "CT_TableCellProperties",
                    "CT_Headers",
                    "CT_TableCol",
                    "CT_TableGrid",
                    "CT_TableCell",
                    "CT_TableRow",
                    "CT_TableProperties",
                    "CT_Table",
                    "CT_Cell3D",
                    "CT_ThemeableLineStyle",
                    "CT_TableStyleTextStyle",
                    "CT_TableCellBorderStyle",
                    "CT_TableBackgroundStyle",
                    "CT_TableStyleCellStyle",
                    "CT_TablePartStyle",
                    "CT_TableStyle",
                    "CT_TableStyleList",
                    "<tbl>",
                    "<tblStyleLst>",
                ];
                assert!(known.contains(&name.as_str()), "{id}");
                declared += 1;
            }
            _ if name == "extLst" => {
                assert!(extension_is_reported(owner), "{id} is not reported");
                reported += 1;
            }
            _ => {
                assert!(modelled(&every, owner, name), "{id} is not in the model");
                read += 1;
            }
        }
    }
    assert_eq!(read + reported + declared, 113);
    assert_eq!(reported, 9, "extLst rows");
}

/// `docs/21` quotes this figure.
#[test]
fn a_cell_costs_no_more_than_docs_21_says() {
    assert!(
        std::mem::size_of::<TableCell>() <= 96,
        "{}",
        std::mem::size_of::<TableCell>()
    );
}

#[test]
fn absent_stays_absent_in_a_bare_table() {
    let (t, findings) = read_table(
        r#"<a:tbl><a:tblGrid><a:gridCol w="5"/></a:tblGrid><a:tr h="5"><a:tc/></a:tr></a:tbl>"#,
    );
    assert_eq!(t.properties, None);
    let c = &t.rows[0].cells[0];
    assert_eq!(*c, TableCell::default());
    assert!(!findings.iter().any(|f| matches!(f.kind, K::Table(_))));
}

#[test]
fn an_empty_header_list_and_no_header_list_differ() {
    let (t, _) = read_table(
        r#"<a:tbl><a:tblGrid><a:gridCol w="5"/></a:tblGrid><a:tr h="5"><a:tc><a:tcPr><a:headers/></a:tcPr></a:tc></a:tr></a:tbl>"#,
    );
    let p = t.rows[0].cells[0].properties.as_ref().unwrap();
    assert_eq!(p.headers.as_deref(), Some(&[][..]));
}

#[test]
fn a_cells_text_is_tier_1fs_and_its_fill_and_lines_tier_1es() {
    let d = deck(EVERY);
    let t = table_of(first_slide(&d), "Every Property");
    let c = &t.rows[0].cells[0];
    let body = c.text.as_ref().unwrap();
    assert!(matches!(
        &body.paragraphs[0].items[0],
        TextItem::Run(r) if r.text.as_deref() == Some("Full")
    ));
    let line = c.properties.as_ref().unwrap().line_left.as_ref().unwrap();
    assert_eq!(line.width, Some(12700));
    assert!(matches!(line.fill, Some(Fill::Solid(_))));
}

#[test]
fn colid_and_rowid_are_reported_and_not_in_the_model() {
    let d = deck(EVERY);
    assert!(d.findings.has(K::RetainedElement, "extLst"));
    // They are read from nowhere: a second read of the same deck gives an equal table.
    let e = deck(EVERY);
    assert_eq!(
        table_of(first_slide(&d), "Every Property"),
        table_of(first_slide(&e), "Every Property")
    );
}

#[test]
fn merged_cells_read_as_mso_oi29500_describes_them() {
    let d = deck("generated/table-merged-cells.pptx");
    let t = table_of(first_slide(&d), "Merged");
    let map = t.merges();
    assert_eq!(map.faults, [], "this fixture is what 2.1.1424 d describes");
    assert_eq!(
        map.merges,
        [
            TableMerge {
                row: 0,
                column: 0,
                rows: 2,
                columns: 2
            },
            TableMerge {
                row: 0,
                column: 2,
                rows: 1,
                columns: 2
            },
            TableMerge {
                row: 1,
                column: 2,
                rows: 2,
                columns: 1
            },
        ]
    );
    assert_eq!(map.origins[&(1, 1)], (0, 0));
    assert_eq!(map.origins[&(0, 3)], (0, 2));
    assert_eq!(map.origins[&(2, 2)], (1, 2));
    assert!(!map.origins.contains_key(&(1, 3)));
    // Each covered cell keeps its own text.
    let covered = t.rows[1].cells[1].text.as_ref().unwrap();
    assert!(matches!(
        &covered.paragraphs[0].items[0],
        TextItem::Run(r) if r.text.as_deref() == Some("c")
    ));
    assert!(!d.findings.iter().any(|f| matches!(f.kind, K::Table(_))));
}

fn has_table(findings: &Findings, finding: TableFinding) -> bool {
    findings.iter().any(|f| f.kind == K::Table(finding))
}

#[test]
fn an_unknown_style_guid_is_reported_and_never_crashes() {
    for path in [
        "generated/table-unknown-style.pptx",
        "generated/table-unknown-style-with-list.pptx",
    ] {
        let d = deck(path);
        assert!(
            d.findings
                .has(K::Table(TableFinding::UnknownStyle), "tableStyleId"),
            "{path}: {:?}",
            d.findings
        );
        let t = table_of(first_slide(&d), "Unknown");
        assert!(t.properties.as_ref().unwrap().style.is_some(), "kept");
    }
    // A listed one is not reported.
    let known = deck(EVERY);
    assert!(
        !known
            .findings
            .has(K::Table(TableFinding::UnknownStyle), "tableStyleId")
    );
    assert_eq!(TableFinding::UnknownStyle.code(), "CS-TBF-0008");
}

#[test]
fn a_tablestyles_part_that_cannot_be_read_is_reported_and_the_deck_opens() {
    let path = "hostile/table-styles-wrong-root.bin";
    let d = deck(path);
    assert_eq!(d.table_styles, None, "{path}");
    assert!(
        d.findings.iter().any(|f| f.kind == K::UnreadablePart),
        "{path}"
    );
    assert!(
        d.findings
            .has(K::Table(TableFinding::UnknownStyle), "tableStyleId"),
        "{path}"
    );
}

#[test]
fn hostile_tables_refuse_with_their_codes_and_open_above_the_bound() {
    let code = |path: &str| deck_with(path, &SlideLimits::default()).unwrap_err().code();
    assert_eq!(code("hostile/table-limit-rows.bin"), "CS-TBL-0001");
    assert_eq!(code("hostile/table-limit-columns.bin"), "CS-TBL-0002");
    assert_eq!(
        code("hostile/table-limit-cells-in-table.bin"),
        "CS-TBL-0003"
    );
    assert_eq!(code("hostile/table-limit-styles.bin"), "CS-TBL-0005");
    assert_eq!(code("hostile/table-limit-headers.bin"), "CS-TBL-0006");

    // Each is the bound's doing: one more and the same file opens.
    let raised = |change: fn(&mut TableLimits)| {
        let mut limits = SlideLimits::default();
        change(&mut limits.table);
        limits
    };
    assert!(
        deck_with(
            "hostile/table-limit-rows.bin",
            &raised(|t| t.max_rows = 4_097)
        )
        .is_ok()
    );
    assert!(
        deck_with(
            "hostile/table-limit-columns.bin",
            &raised(|t| t.max_columns = 4_097)
        )
        .is_ok()
    );
    assert!(
        deck_with(
            "hostile/table-limit-cells-in-table.bin",
            &raised(|t| t.max_cells_per_table = 1_052_672)
        )
        .is_ok()
    );
    assert!(
        deck_with(
            "hostile/table-limit-styles.bin",
            &raised(|t| t.max_table_styles = 4_097)
        )
        .is_ok()
    );
    assert!(
        deck_with(
            "hostile/table-limit-headers.bin",
            &raised(|t| t.max_headers_per_cell = 1_025)
        )
        .is_ok()
    );

    // Too large a deck to commit at the default, so with the bound lowered.
    let tight = raised(|t| t.max_cells_per_part = 3);
    let err = deck_with(EVERY, &tight).unwrap_err();
    assert_eq!(err.code(), "CS-TBL-0004");
    assert_eq!(TableBound::CellsInPart.code(), "CS-TBL-0004");
    let roomy = raised(|t| t.max_cells_per_part = 5);
    assert!(deck_with(EVERY, &roomy).is_ok());
}

/// What stands where a document's text can stand.
const MARK: &str = "QzMarkerQz";

#[test]
fn a_hostile_table_opens_with_everything_reported_and_nothing_from_the_document_in_it() {
    let d = deck("hostile/table-bad-structure.bin");
    let f = &d.findings;
    for (finding, what) in [
        (TableFinding::CellCount, "rows of the wrong length"),
        (TableFinding::SpanOutOfRange, "spans past the table"),
        (TableFinding::MergeOverlap, "a covered cell with no flag"),
        (TableFinding::MergeNoOrigin, "a flag with no merge"),
        (TableFinding::Measure, "a negative width, a huge height"),
    ] {
        assert!(has_table(f, finding), "{what}: {f:?}");
    }
    for kind in [
        K::MalformedValue,
        K::OutOfRange,
        K::MissingRequired,
        K::DuplicateElement,
        K::DuplicateId,
        K::RetainedElement,
        K::RetainedAttribute,
    ] {
        assert!(f.iter().any(|x| x.kind == kind), "{kind:?}: {f:?}");
    }
    let t = table_of(first_slide(&d), "t");
    // A span that leaves the table is not one a consumer can be handed.
    let columns = t.column_count();
    for (r, row) in t.rows.iter().enumerate() {
        for (c, cell) in row.cells.iter().enumerate() {
            if let Some(span) = cell.grid_span {
                assert!(
                    span >= 1 && (span as usize) <= columns.saturating_sub(c),
                    "{r},{c}"
                );
            }
            if let Some(span) = cell.row_span {
                assert!(span >= 1 && (span as usize) <= t.rows.len() - r, "{r},{c}");
            }
        }
    }
    // Rendered every way a report is: not a character of the document.
    let everything = format!("{f:?}");
    assert!(!everything.contains(MARK), "{everything}");
}

#[test]
fn spans_are_checked_against_what_is_left_of_the_table_at_that_cell() {
    let table = |row: &str| {
        format!(
            r#"<a:tbl><a:tblGrid><a:gridCol w="1"/><a:gridCol w="1"/></a:tblGrid><a:tr h="1">{row}</a:tr><a:tr h="1"><a:tc/><a:tc/></a:tr></a:tbl>"#
        )
    };
    let read = |row: &str| read_table(&table(row));
    // Fills what is left, from each column: kept.
    let (t, f) = read(r#"<a:tc gridSpan="2" rowSpan="2"/><a:tc hMerge="1" rowSpan="2"/>"#);
    assert_eq!(t.rows[0].cells[0].grid_span, Some(2));
    assert_eq!(t.rows[0].cells[0].row_span, Some(2));
    assert!(!has_table(&f, TableFinding::SpanOutOfRange), "{f:?}");
    // One more than what is left, from the second column: left out.
    let (t, f) = read(r#"<a:tc/><a:tc gridSpan="2"/>"#);
    assert_eq!(t.rows[0].cells[1].grid_span, None);
    assert!(has_table(&f, TableFinding::SpanOutOfRange));
    // Past the rows.
    let (t, f) = read(r#"<a:tc rowSpan="3"/><a:tc/>"#);
    assert_eq!(t.rows[0].cells[0].row_span, None);
    assert!(has_table(&f, TableFinding::SpanOutOfRange));
    // Not a number at all: as any malformed value.
    let (_, f) = read(r#"<a:tc gridSpan="x"/><a:tc/>"#);
    assert!(f.iter().any(|x| x.kind == K::MalformedValue));
    // The largest int, on a machine where that is far past anything allocated.
    let (t, _) = read(r#"<a:tc gridSpan="2147483647" rowSpan="2147483647"/><a:tc/>"#);
    assert_eq!(t.rows[0].cells[0].grid_span, None);
    assert_eq!(t.rows[0].cells[0].row_span, None);
}

/// Each rule of `MS-OI29500` 2.1.1424 d, broken alone.
#[test]
fn each_merge_rule_broken_alone_is_reported_under_its_own_code() {
    let two_by_two = |rows: &str| {
        format!(
            r#"<a:tbl><a:tblGrid><a:gridCol w="1"/><a:gridCol w="1"/></a:tblGrid>{rows}</a:tbl>"#
        )
    };
    let only = |rows: &str, expected: Option<TableFinding>| {
        let (_, f) = read_table(&two_by_two(rows));
        for finding in [
            TableFinding::MergeOverlap,
            TableFinding::MergeNoOrigin,
            TableFinding::MergeSpans,
        ] {
            assert_eq!(
                has_table(&f, finding),
                expected == Some(finding),
                "{rows}: {f:?}"
            );
        }
    };
    // As it should be.
    only(
        r#"<a:tr h="1"><a:tc gridSpan="2"/><a:tc hMerge="1"/></a:tr>"#,
        None,
    );
    only(
        r#"<a:tr h="1"><a:tc rowSpan="2"/><a:tc/></a:tr><a:tr h="1"><a:tc vMerge="1"/><a:tc/></a:tr>"#,
        None,
    );
    // Covered from the left with no flag.
    only(
        r#"<a:tr h="1"><a:tc gridSpan="2"/><a:tc/></a:tr>"#,
        Some(TableFinding::MergeOverlap),
    );
    // Covered from above with no flag.
    only(
        r#"<a:tr h="1"><a:tc rowSpan="2"/><a:tc/></a:tr><a:tr h="1"><a:tc/><a:tc/></a:tr>"#,
        Some(TableFinding::MergeOverlap),
    );
    // A flag no merge explains.
    only(
        r#"<a:tr h="1"><a:tc hMerge="1"/><a:tc/></a:tr>"#,
        Some(TableFinding::MergeNoOrigin),
    );
    only(
        r#"<a:tr h="1"><a:tc vMerge="1"/><a:tc/></a:tr>"#,
        Some(TableFinding::MergeNoOrigin),
    );
    // A covered cell in the first row that does not repeat its merge's rowSpan.
    only(
        r#"<a:tr h="1"><a:tc gridSpan="2" rowSpan="2"/><a:tc hMerge="1"/></a:tr><a:tr h="1"><a:tc gridSpan="2" vMerge="1"/><a:tc hMerge="1" vMerge="1"/></a:tr>"#,
        Some(TableFinding::MergeSpans),
    );
}

/// A silent drop fails here. Each element this tier owns takes an attribute and a child
/// nothing knows, one element at a time, and the report must name each.
#[test]
fn nothing_this_tier_does_not_model_goes_unreported() {
    let bytes = std::fs::read(fixtures().join(EVERY)).unwrap();
    let mut pkg = PresentationPackage::open(bytes, OoxmlLimits::default()).unwrap();
    let slide = String::from_utf8(pkg.read_part("ppt/slides/slide1.xml").unwrap()).unwrap();
    let styles = String::from_utf8(pkg.read_part("ppt/tableStyles.xml").unwrap()).unwrap();

    let report_of_slide = |xml: &str| {
        let tree = XmlTree::parse(xml.as_bytes(), "s.xml", &OoxmlLimits::default()).unwrap();
        let limits = ImportLimits::default();
        let mut findings = Findings::new(limits.max_findings);
        let mut color = ColorFindings::default();
        import_slide_part(
            &tree,
            "s.xml",
            &[],
            SlideKind::Slide,
            0,
            &SlideLimits::default(),
            &limits,
            &mut findings,
            &mut color,
        )
        .unwrap();
        findings
    };
    let report_of_styles = |xml: &str| {
        let tree = XmlTree::parse(xml.as_bytes(), "t.xml", &OoxmlLimits::default()).unwrap();
        let limits = ImportLimits::default();
        let mut findings = Findings::new(limits.max_findings);
        let mut color = ColorFindings::default();
        import_table_styles(
            &tree,
            "t.xml",
            &[],
            &SlideLimits::default(),
            &limits,
            &mut findings,
            &mut color,
        )
        .unwrap();
        findings
    };
    let attribute = |xml: &str, name: &str| {
        let open = format!("<a:{name}");
        let mut out = String::new();
        let mut rest = xml;
        let mut hit = false;
        while let Some(at) = rest.find(&open) {
            let after = at + open.len();
            out.push_str(&rest[..after]);
            let next = rest[after..].chars().next().unwrap();
            if matches!(next, ' ' | '>' | '/') {
                out.push_str(r#" zzUnknown="1""#);
                hit = true;
            }
            rest = &rest[after..];
        }
        out.push_str(rest);
        assert!(hit, "{name} is not in the fixture");
        out
    };
    let child = |xml: &str, name: &str| {
        let open = format!("<a:{name}>");
        xml.contains(&open)
            .then(|| xml.replace(&open, &format!("{open}<a:zzChild/>")))
    };
    let names_attribute = |f: &Findings, name: &str| {
        f.iter().any(|x| {
            x.kind == K::RetainedAttribute
                && x.element == name
                && x.attribute.as_deref() == Some("zzUnknown")
        })
    };
    let names_child = |f: &Findings, name: &str| {
        f.has(K::RetainedElement, "zzChild") || f.has(K::MalformedValue, name)
    };

    for name in [
        "tbl",
        "tblPr",
        "tblGrid",
        "gridCol",
        "tr",
        "tc",
        "tcPr",
        "cell3D",
        "headers",
        "header",
        "tableStyleId",
        "tableStyle",
        "txBody",
    ] {
        let f = report_of_slide(&attribute(&slide, name));
        assert!(names_attribute(&f, name), "{name}: attribute: {f:?}");
        if let Some(xml) = child(&slide, name) {
            let f = report_of_slide(&xml);
            assert!(names_child(&f, name), "{name}: child: {f:?}");
        }
    }
    for name in [
        "tblStyle",
        "tblBg",
        "wholeTbl",
        "band1H",
        "nwCell",
        "tcTxStyle",
        "tcStyle",
        "tcBdr",
        "left",
        "right",
        "fill",
        "fillRef",
        "effect",
        "effectRef",
        "fontRef",
        "lnRef",
        "cell3D",
    ] {
        let f = report_of_styles(&attribute(&styles, name));
        assert!(names_attribute(&f, name), "{name}: attribute: {f:?}");
        if let Some(xml) = child(&styles, name) {
            let f = report_of_styles(&xml);
            assert!(names_child(&f, name), "{name}: child: {f:?}");
        }
    }
}

#[test]
fn merges_of_a_hostile_model_cost_one_pass_and_never_index_out() {
    // In memory, with no reader to have left bad spans out.
    let cell = TableCell {
        grid_span: Some(i32::MAX),
        row_span: Some(i32::MAX),
        ..TableCell::default()
    };
    let t = Table {
        grid: Some(vec![Default::default(); 3]),
        rows: vec![
            casual_slide_model::table::TableRow {
                height: None,
                cells: vec![cell.clone(); 40],
            };
            40
        ],
        properties: None,
    };
    let map = t.merges();
    assert_eq!(map.merges.len(), 1);
    assert_eq!(map.merges[0].rows, 40);
    assert_eq!(map.merges[0].columns, 3);
    assert!(
        map.faults
            .contains(&MergeFault::Overlap { row: 0, column: 1 })
    );
    // No grid: nothing is a merge, and nothing panics.
    let none = Table {
        grid: None,
        ..t.clone()
    };
    assert_eq!(none.merges().merges, []);
    assert_eq!(Table::default().merges().merges, []);
}

#[test]
fn every_deck_reads_its_tables_with_no_table_left_unread() {
    // Real decks, whatever else they hold: no `tbl` is only retained.
    let mut tables = 0;
    let mut decks_with_tables = Vec::new();
    for set in ["generated", "producers", "real"] {
        let mut names: Vec<_> = std::fs::read_dir(fixtures().join(set))
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| !p.extension().is_some_and(|e| e == "tsv" || e == "md"))
            .collect();
        names.sort();
        for p in names {
            let rel = format!("{set}/{}", p.file_name().unwrap().to_string_lossy());
            let d = deck(&rel);
            assert!(!d.findings.has(K::RetainedElement, "tbl"), "{rel}");
            let here = d
                .parts
                .parts
                .iter()
                .flatten()
                .filter_map(|part| part.shape_tree())
                .flat_map(|tree| tree.descendants())
                .filter(|n| n.table.is_some())
                .count();
            if here > 0 && set == "real" {
                decks_with_tables.push(rel.clone());
            }
            tables += here;
        }
    }
    assert!(tables >= 8, "{tables}");
    // Real tables name built-in styles, with nothing in `tableStyles.xml`: reported.
    for rel in &decks_with_tables {
        let d = deck(rel);
        assert!(
            d.table_styles.as_ref().is_some_and(|l| l.styles.is_empty()),
            "{rel}"
        );
        assert!(
            d.findings
                .has(K::Table(TableFinding::UnknownStyle), "tableStyleId"),
            "{rel}"
        );
    }
    assert_eq!(
        decks_with_tables,
        [
            "real/z17310490-vierfelder-de.pptx",
            "real/z19828779-education-ar.pptx"
        ]
    );
}
