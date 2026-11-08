//! Rich objects like PowerPoint (`VS-11`): Insert > Table, typing in its cells, rows and
//! columns added and deleted, a Table Design style; Insert > Chart, and its data edited.
//! Each op undoes and redoes, a save writes a deck that opens again as the model said
//! with a chart part and a workbook, undoing every edit saves the opened bytes, and the
//! saved deck passes F4 and Microsoft's validator.

use std::path::Path;
use std::process::Command;

use casual_slide_export::new_chart::NewChartKind;
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage};
use casual_slide_wasm::{DEFAULT_TABLE_STYLE, ShapeBox, TableEdit, Viewer};

const BOX: [i64; 4] = [914_400, 914_400, 6_096_000, 1_483_360];
const OTHER_STYLE: &str = "{073A0DAA-6AF3-43AB-8588-CEC1D06C72B9}";

/// Insert a 3 by 4 table and a column chart, type, add and drop rows and columns,
/// restyle, and change the chart's data.
fn edit_a_deck(v: &mut Viewer) {
    let table = v.insert_table(0, 3, 4, BOX).unwrap();
    assert_eq!(&v.table_shape(0, table).unwrap()[..2], &[3, 4]);
    assert!(v.set_table_cell_text(0, table, 0, 0, "Region").unwrap());
    assert!(
        v.set_table_cell_text(0, table, 1, 2, "North & <South>")
            .unwrap()
    );
    let tall = |v: &Viewer| {
        v.shapes(0)
            .unwrap()
            .into_iter()
            .find(|b| b.node == table)
            .unwrap()
            .cy
    };
    let before = tall(v);
    v.edit_table(0, table, TableEdit::InsertRow, 1).unwrap();
    // The frame grows with its rows, as PowerPoint's.
    let row = v.table_shape(0, table).unwrap()[2 + 4];
    assert_eq!(before, row * 3);
    assert_eq!(tall(v), before + row);
    v.edit_table(0, table, TableEdit::DeleteColumn, 3).unwrap();
    v.edit_table(0, table, TableEdit::InsertColumn, 0).unwrap();
    assert_eq!(&v.table_shape(0, table).unwrap()[..2], &[4, 4]);
    assert_eq!(
        v.table_cell_text(0, table, 2, 3).unwrap(),
        "North & <South>"
    );
    let styles = v.table_styles();
    assert!(styles.iter().any(|(id, _)| id == DEFAULT_TABLE_STYLE));
    assert_eq!(v.table_style(0, table).unwrap(), DEFAULT_TABLE_STYLE);
    assert!(v.set_table_style(0, table, OTHER_STYLE).unwrap());
    assert!(v.set_table_style(0, table, "not a guid").is_err());

    let chart = v
        .insert_chart(
            0,
            NewChartKind::Column,
            [914_400, 3_000_000, 4_000_000, 3_000_000],
        )
        .unwrap();
    let mut grid = v.chart_grid(0, chart).unwrap();
    assert_eq!(grid.categories.len(), 4);
    assert_eq!(grid.series.len(), 3);
    grid.categories.push("Category 5".to_owned());
    grid.series[0].0 = "Revenue".to_owned();
    grid.series[0].1 = vec![Some(1.0), Some(2.0), None, Some(4.0), Some(5.5)];
    assert!(v.set_chart_grid(0, chart, grid).unwrap());
    let read = v.chart_grid(0, chart).unwrap();
    assert_eq!(read.categories.len(), 5);
    assert_eq!(read.series[0].1[4], Some(5.5));
}

fn part(bytes: &[u8], name: &str) -> String {
    let mut p = PresentationPackage::open(bytes.to_vec(), OoxmlLimits::default()).unwrap();
    String::from_utf8(p.read_part(name).unwrap()).unwrap()
}

fn find(v: &Viewer, flag: u32) -> u32 {
    v.shapes(0)
        .unwrap()
        .into_iter()
        .find(|b| b.flags & flag != 0)
        .unwrap()
        .node
}

#[test]
fn a_table_and_a_chart_save_and_open_as_edited() {
    let mut v = Viewer::blank().unwrap();
    edit_a_deck(&mut v);
    let saved = v.save().unwrap();
    let mut again = Viewer::open(saved.clone()).unwrap();
    let t = find(&again, ShapeBox::TABLE);
    let c = find(&again, ShapeBox::CHART);
    assert_eq!(&again.table_shape(0, t).unwrap()[..2], &[4, 4]);
    assert_eq!(again.table_cell_text(0, t, 0, 1).unwrap(), "Region");
    assert_eq!(
        again.table_cell_text(0, t, 2, 3).unwrap(),
        "North & <South>"
    );
    assert_eq!(again.table_style(0, t).unwrap(), OTHER_STYLE);
    let grid = again.chart_grid(0, c).unwrap();
    assert_eq!(grid.kind, NewChartKind::Column);
    assert_eq!(grid.series[0].0, "Revenue");
    assert_eq!(
        grid.series[0].1,
        vec![Some(1.0), Some(2.0), None, Some(4.0), Some(5.5)]
    );
    // The chart part and its workbook are in the file, and the workbook holds the data.
    let chart_xml = part(&saved, "ppt/charts/chart1.xml");
    assert!(
        chart_xml.contains("<c:externalData r:id=\"rId1\">"),
        "{chart_xml}"
    );
    let rels = part(&saved, "ppt/charts/_rels/chart1.xml.rels");
    assert!(
        rels.contains("embeddings/Microsoft_Excel_Worksheet"),
        "{rels}"
    );
    let mut p = PresentationPackage::open(saved.clone(), OoxmlLimits::default()).unwrap();
    let book_name = p
        .part_names()
        .into_iter()
        .find(|n| n.ends_with(".xlsx"))
        .unwrap();
    let book = p.read_part(&book_name).unwrap();
    let mut inner = casual_slide_package::Package::open(book, Default::default()).unwrap();
    let sheet = String::from_utf8(inner.read_part("xl/worksheets/sheet1.xml").unwrap()).unwrap();
    assert!(
        sheet.contains("Revenue") && sheet.contains("<v>5.5</v>"),
        "{sheet}"
    );
    // A save of the reopened deck changes nothing, and the slide draws.
    assert_eq!(again.save().unwrap(), saved);
    again.render_png(0, 320).unwrap();
}

#[test]
fn undoing_every_edit_saves_the_opened_bytes_and_redo_saves_the_same_deck() {
    let mut v = Viewer::blank().unwrap();
    let source = v.save().unwrap();
    edit_a_deck(&mut v);
    let edited = v.save().unwrap();
    while v.can_undo_redo().0 {
        v.undo().unwrap();
    }
    assert_eq!(v.save().unwrap(), source);
    while v.can_undo_redo().1 {
        v.redo().unwrap();
    }
    assert_eq!(v.save().unwrap(), edited);
}

#[test]
fn a_chart_of_the_file_takes_new_values_and_only_its_part_changes() {
    let mut v = Viewer::blank().unwrap();
    v.insert_chart(0, NewChartKind::Line, [0, 0, 4_000_000, 3_000_000])
        .unwrap();
    let first = v.save().unwrap();
    let mut v = Viewer::open(first.clone()).unwrap();
    let c = find(&v, ShapeBox::CHART);
    let mut grid = v.chart_grid(0, c).unwrap();
    grid.series[1].1[2] = Some(9.25);
    assert!(v.set_chart_grid(0, c, grid.clone()).unwrap());
    // The file's chart keeps its shape: a new category is refused.
    let mut bigger = grid.clone();
    bigger.categories.push("x".to_owned());
    assert!(v.set_chart_grid(0, c, bigger).is_err());
    let saved = v.save().unwrap();
    let after = part(&saved, "ppt/charts/chart1.xml");
    assert!(after.contains("<c:v>9.25</c:v>"), "{after}");
    let mut a = PresentationPackage::open(first.clone(), OoxmlLimits::default()).unwrap();
    let mut b = PresentationPackage::open(saved.clone(), OoxmlLimits::default()).unwrap();
    for name in a.part_names() {
        if name != "ppt/charts/chart1.xml" {
            assert_eq!(
                a.read_part(&name).unwrap(),
                b.read_part(&name).unwrap(),
                "{name}"
            );
        }
    }
    let again = Viewer::open(saved).unwrap();
    assert_eq!(again.chart_grid(0, c).unwrap().series[1].1[2], Some(9.25));
}

/// F4 and Microsoft's Open XML SDK validator (`tools/ooxml-sdk`) on the saved deck.
/// Each skips where its tool cannot run, unless `F4_REQUIRED` or `OOXML_SDK_REQUIRED`.
#[test]
fn the_saved_deck_passes_f4_and_the_sdk_validator() {
    let mut v = Viewer::blank().unwrap();
    edit_a_deck(&mut v);
    let saved = v.save().unwrap();
    let python = ["python3", "python"]
        .into_iter()
        .find(|p| Command::new(p).arg("--version").output().is_ok());
    let Some(python) = python else {
        assert!(std::env::var_os("OOXML_SDK_REQUIRED").is_none());
        assert!(std::env::var_os("F4_REQUIRED").is_none());
        return;
    };
    let dir = std::env::temp_dir().join("openpresent-vs11");
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join(format!("rich-{}.pptx", std::process::id()));
    std::fs::write(&file, &saved).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools");
    for (tool, flag) in [
        ("f4/validate.py", "F4_REQUIRED"),
        ("ooxml-sdk/validate.py", "OOXML_SDK_REQUIRED"),
    ] {
        let required = std::env::var_os(flag).is_some();
        let out = Command::new(python)
            .arg("-I")
            .arg(root.join(tool))
            .arg(&file)
            .output()
            .unwrap();
        let report = String::from_utf8_lossy(&out.stdout).into_owned()
            + &String::from_utf8_lossy(&out.stderr);
        match out.status.code() {
            Some(0) => {}
            Some(2) if !required => eprintln!("{tool} could not run: skipped\n{report}"),
            _ => panic!("{tool} rejected the saved deck:\n{report}"),
        }
    }
    std::fs::remove_file(&file).unwrap();
}

/// `UX-015`: a table's rows as drawn, grown to fit their text, follow the stated ones,
/// so a click is placed in the cell under it. On `z17310490-vierfelder-de` the rows
/// state less height than their two-line cells take, and a click in "falsch-positiv"
/// landed in the row below.
#[test]
fn a_tables_drawn_rows_follow_its_stated_ones() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/real/z17310490-vierfelder-de.pptx");
    let v = Viewer::open(std::fs::read(path).unwrap()).unwrap();
    let table = v
        .shapes(0)
        .unwrap()
        .into_iter()
        .find(|b| b.flags & ShapeBox::TABLE != 0)
        .unwrap()
        .node;
    let shape = v.table_shape(0, table).unwrap();
    let (rows, cols) = (shape[0] as usize, shape[1] as usize);
    assert_eq!(
        shape.len(),
        2 + cols + 2 * rows,
        "drawn heights follow: {shape:?}"
    );
    let stated = &shape[2 + cols..2 + cols + rows];
    let drawn = &shape[2 + cols + rows..];
    assert!(drawn.iter().zip(stated).all(|(d, s)| d >= s));
    assert!(
        drawn.iter().zip(stated).any(|(d, s)| d > s),
        "{stated:?} {drawn:?}"
    );
}
