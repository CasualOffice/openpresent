#![no_main]
//! Any bytes as what a graphic frame's `a:tbl` holds, and as what `a:tblStyleLst` holds.
//! Import never panics; merges and resolving never panic; a patch of an unchanged model
//! changes nothing; an edit the exporter accepts reads back as it was asked.

use casual_slide_export::slide::patch_slide_part;
use casual_slide_export::table::patch_table_styles;
use casual_slide_import::color::Findings as ColorFindings;
use casual_slide_import::properties::{Findings, ImportLimits};
use casual_slide_import::slide::{import_slide_part, SlideLimits};
use casual_slide_import::table::import_table_styles;
use casual_slide_model::fill::Fill;
use casual_slide_model::slide::{SlideKind, SlidePart};
use casual_slide_model::table::{Table, TableStyleList};
use casual_slide_model::text::TextItem;
use casual_slide_ooxml::{OoxmlLimits, XmlTree};
use casual_slide_style::table::{table_style, CellChain};
use libfuzzer_sys::fuzz_target;

const PART: &str = "ppt/slides/slide1.xml";
const STYLES: &str = "ppt/tableStyles.xml";
const NS: &str = concat!(
    r#"xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" "#,
    r#"xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" "#,
    r#"xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main""#
);

fn read(tree: &XmlTree, limits: &SlideLimits, import: &ImportLimits) -> Option<SlidePart> {
    let mut findings = Findings::new(import.max_findings);
    let mut color = ColorFindings::default();
    import_slide_part(
        tree, PART, &[], SlideKind::Slide, 0, limits, import, &mut findings, &mut color,
    )
    .ok()
}

fn read_list(tree: &XmlTree, limits: &SlideLimits, import: &ImportLimits) -> Option<TableStyleList> {
    let mut findings = Findings::new(import.max_findings);
    let mut color = ColorFindings::default();
    import_table_styles(tree, STYLES, &[], limits, import, &mut findings, &mut color).ok()
}

fn table_of(part: &SlidePart) -> Option<&Table> {
    part.shape_tree()?.children.first()?.table.as_deref()
}

/// Every cell a grid position can name, and one past each edge.
fn resolve(table: &Table, list: Option<&TableStyleList>) {
    let merges = table.merges();
    let style = table_style(table, list).style();
    let rows = table.rows.len().min(16).saturating_add(1);
    let columns = table.column_count().min(16).saturating_add(1);
    for row in 0..rows {
        for column in 0..columns {
            let chain = CellChain::new(table, &merges, style, row, column);
            let _ = (chain.fill(), chain.bold(), chain.text_color(), chain.margin_left());
        }
    }
}

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    let xml = OoxmlLimits::default();
    // Low enough that the bounds are reached, as well as what lies under them.
    let mut limits = SlideLimits::default();
    limits.table.max_rows = 64;
    limits.table.max_columns = 64;
    limits.table.max_cells_per_table = 512;
    limits.table.max_cells_per_part = 1024;
    limits.table.max_table_styles = 16;
    limits.table.max_headers_per_cell = 16;
    let import = ImportLimits::default();

    let list = format!(r#"<a:tblStyleLst {NS}>{text}</a:tblStyleLst>"#);
    let mut styles = None;
    if let Ok(tree) = XmlTree::parse(list.as_bytes(), STYLES, &xml) {
        if let Some(before) = read_list(&tree, &limits, &import) {
            let mut same = tree.clone();
            let changes =
                patch_table_styles(&mut same, STYLES, false, &[], &before, &limits, &import, &xml)
                    .unwrap();
            assert_eq!(changes, 0, "an unchanged list changed something");
            assert_eq!(same.to_bytes(), tree.to_bytes());
            styles = Some(before);
        }
    }

    let slide = format!(
        r#"<p:sld {NS}><p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/><p:graphicFrame><p:nvGraphicFramePr><p:cNvPr id="2" name="t"/><p:cNvGraphicFramePr/><p:nvPr/></p:nvGraphicFramePr><p:xfrm/><a:graphic><a:graphicData uri="u"><a:tbl>{text}</a:tbl></a:graphicData></a:graphic></p:graphicFrame></p:spTree></p:cSld></p:sld>"#
    );
    let Ok(tree) = XmlTree::parse(slide.as_bytes(), PART, &xml) else {
        return;
    };
    let Some(before) = read(&tree, &limits, &import) else {
        return;
    };
    let mut same = tree.clone();
    let changes =
        patch_slide_part(&mut same, PART, false, &[], &before, &limits, &import, &xml).unwrap();
    assert_eq!(changes, 0, "an unchanged model changed something");
    assert_eq!(same.to_bytes(), tree.to_bytes());
    if let Some(table) = table_of(&before) {
        resolve(table, styles.as_ref());
    }

    // An edit: every cell's text and fill, and a row's height.
    let mut wanted = before.clone();
    let frame = wanted.shape_tree_mut().and_then(|t| t.children.first_mut());
    let Some(table) = frame.and_then(|n| n.table.as_mut()) else {
        return;
    };
    for row in &mut table.rows {
        for cell in &mut row.cells {
            cell.tc_pr.get_or_insert_with(Default::default).fill = Some(Fill::None);
            let Some(body) = cell.tx_body.as_mut() else {
                continue;
            };
            for paragraph in &mut body.paragraphs {
                for item in &mut paragraph.items {
                    if let TextItem::Run(run) = item {
                        run.text = Some("edited & <new>".to_owned());
                    }
                }
            }
        }
    }
    let mut edited = tree.clone();
    if patch_slide_part(&mut edited, PART, false, &[], &wanted, &limits, &import, &xml).is_ok() {
        let again = XmlTree::parse(&edited.to_bytes(), PART, &xml).unwrap();
        assert_eq!(read(&again, &limits, &import).as_ref(), Some(&wanted));
    }
});
