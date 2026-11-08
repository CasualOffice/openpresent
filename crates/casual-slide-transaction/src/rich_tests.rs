// Slice 11's tests (`VS-11`): each rich-object operation, applied then inverted, gives
// the model back; the table edits keep the table's shape rules.

use super::rich::{
    cell_text, delete_column, delete_row, insert_column, insert_row, set_cell_text, set_style,
};
use super::*;
use casual_slide_model::chart::ChartPart;
use casual_slide_model::shape::NodeKind;
use casual_slide_model::slide::{SlideKind, SlidePart};
use casual_slide_model::table::{Table, TableCell, TableCol, TableGrid, TableRow};
use casual_slide_model::text::TextBody;

fn id(node: u32) -> ObjectId {
    ObjectId { part: 0, node }
}

fn cell(text: &str) -> TableCell {
    TableCell {
        tx_body: Some(Box::new(TextBody {
            properties: Some(Default::default()),
            list_style: Some(Default::default()),
            paragraphs: vec![Paragraph {
                items: vec![TextItem::Run(TextRun {
                    properties: None,
                    text: Some(text.to_owned()),
                })],
                ..Paragraph::default()
            }],
        })),
        ..TableCell::default()
    }
}

fn table() -> Table {
    Table {
        tbl_pr: None,
        tbl_grid: Some(TableGrid {
            grid_col: vec![TableCol::default(), TableCol::default()],
        }),
        rows: (0..2)
            .map(|r| TableRow {
                height: None,
                cells: (0..2).map(|c| cell(&format!("r{r}c{c}"))).collect(),
            })
            .collect(),
    }
}

struct Doc {
    parts: SlideParts,
    chart: ChartPart,
}

impl Document for Doc {
    fn parts_mut(&mut self) -> &mut SlideParts {
        &mut self.parts
    }
    fn chart_mut(&mut self, part: &str) -> Option<&mut ChartPart> {
        (part == "ppt/charts/chart1.xml").then_some(&mut self.chart)
    }
}

fn doc() -> Doc {
    let mut root = ShapeNode::new(id(1), NodeKind::Group);
    let mut frame = ShapeNode::new(id(2), NodeKind::GraphicFrame);
    frame.table = Some(Box::new(table()));
    root.children.push(frame);
    root.children.push(ShapeNode::new(id(3), NodeKind::Shape));
    let mut part = SlidePart::new(ObjectId::part(0), SlideKind::Slide);
    part.common = Some(casual_slide_model::slide::CommonSlideData {
        name: None,
        background: None,
        shape_tree: Some(root),
        controls: Vec::new(),
    });
    Doc {
        parts: SlideParts {
            parts: vec![Some(part)],
            ..SlideParts::default()
        },
        chart: ChartPart::Chart(Box::default()),
    }
}

fn table_of(d: &Doc) -> Table {
    let part = d.parts.get(0).unwrap();
    let node = part.shape_tree().unwrap().children[0].clone();
    (**node.table.as_ref().unwrap()).clone()
}

#[test]
fn each_table_edit_applied_then_undone_gives_the_model_back() {
    let t = table();
    let edits = [
        insert_row(&t, 1).unwrap(),
        insert_row(&t, 2).unwrap(),
        delete_row(&t, 0).unwrap(),
        insert_column(&t, 0).unwrap(),
        insert_column(&t, 2).unwrap(),
        delete_column(&t, 1).unwrap(),
        set_cell_text(&t, 1, 1, "new\nlines").unwrap(),
        set_style(&t, Some("{5C22544A-7EE6-4342-B048-85BDC9FD1C3A}")),
    ];
    for edited in edits {
        let mut d = doc();
        let before = d.parts.clone();
        let mut history = History::new();
        let op = Operation::SetTable {
            node: id(2),
            table: Some(Box::new(edited.clone())),
        };
        assert!(history.apply(&mut d, op).unwrap());
        assert_eq!(table_of(&d), edited);
        history.undo(&mut d).unwrap();
        assert_eq!(d.parts, before);
        history.redo(&mut d).unwrap();
        assert_eq!(table_of(&d), edited);
    }
}

#[test]
fn row_and_column_edits_have_the_shape_asked() {
    let t = table();
    let r = insert_row(&t, 1).unwrap();
    assert_eq!(r.rows.len(), 3);
    assert_eq!(cell_text(&r, 1, 0).unwrap(), "");
    assert_eq!(cell_text(&r, 2, 1).unwrap(), "r1c1");
    let c = insert_column(&t, 1).unwrap();
    assert_eq!(c.tbl_grid.as_ref().unwrap().grid_col.len(), 3);
    assert!(c.rows.iter().all(|row| row.cells.len() == 3));
    assert_eq!(cell_text(&c, 0, 2).unwrap(), "r0c1");
    let d = delete_column(&t, 0).unwrap();
    assert_eq!(cell_text(&d, 1, 0).unwrap(), "r1c1");
    let s = set_cell_text(&t, 0, 0, "a\nb").unwrap();
    assert_eq!(cell_text(&s, 0, 0).unwrap(), "a\nb");
    assert_eq!(set_cell_text(&t, 0, 0, "r0c0").unwrap(), t);
}

#[test]
fn edits_the_table_cannot_take_are_refused() {
    let t = table();
    assert_eq!(insert_row(&t, 3).unwrap_err(), TxnError::TableShape);
    let one = delete_row(&t, 0).unwrap();
    assert_eq!(delete_row(&one, 0).unwrap_err(), TxnError::TableShape);
    let narrow = delete_column(&t, 0).unwrap();
    assert_eq!(delete_column(&narrow, 0).unwrap_err(), TxnError::TableShape);
    let mut merged = t.clone();
    merged.rows[0].cells[0].grid_span = Some(2);
    merged.rows[0].cells[1].h_merge = Some(true);
    assert_eq!(insert_row(&merged, 0).unwrap_err(), TxnError::TableShape);
    assert_eq!(insert_column(&merged, 0).unwrap_err(), TxnError::TableShape);
    // A shape with no table, and a chart part that is not there.
    let mut d = doc();
    let no_table = Operation::SetTable {
        node: id(3),
        table: Some(Box::new(t)),
    };
    assert_eq!(apply(&mut d, no_table).unwrap_err().code(), "CS-TXN-0012");
    let no_chart = Operation::SetChart {
        part: "ppt/charts/chart9.xml".to_owned(),
        chart: Box::new(ChartPart::Chart(Box::default())),
    };
    assert_eq!(apply(&mut d, no_chart).unwrap_err().code(), "CS-TXN-0014");
}

#[test]
fn a_chart_set_then_undone_gives_the_part_back() {
    let mut d = doc();
    let before = d.chart.clone();
    let changed = casual_slide_model::chart::c::ChartSpace {
        rounded_corners: Some(Box::default()),
        ..Default::default()
    };
    let op = Operation::SetChart {
        part: "ppt/charts/chart1.xml".to_owned(),
        chart: Box::new(ChartPart::Chart(Box::new(changed))),
    };
    let mut history = History::new();
    assert!(history.apply(&mut d, op).unwrap());
    assert_ne!(d.chart, before);
    history.undo(&mut d).unwrap();
    assert_eq!(d.chart, before);
}
