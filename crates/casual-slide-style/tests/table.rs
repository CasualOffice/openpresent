//! Tier 1G resolution: which style a table takes, and the order part styles apply in
//! (`MS-OI29500` 2.1.1265). Grid positions only: nothing here measures.

use casual_slide_model::fill::Fill;
use casual_slide_model::table::{
    Table, TableCell, TableCellProperties, TablePartKind as K, TablePartStyle, TableProperties,
    TableRow, TableStyle, TableStyleCellStyle, TableStyleChoice, TableStyleFill, TableStyleList,
    TableStyleTextStyle,
};
use casual_slide_model::theme::{OnOffStyle, StyleMatrixRef};
use casual_slide_style::table::{
    CellChain, CellFill, CellLevel, StyleSource, part_kinds, table_style,
};

const ID: &str = "{1C8F3A2E-5B6D-4E7F-8A9B-0C1D2E3F4A5B}";

fn grid(rows: usize, columns: usize, flags: TableProperties) -> Table {
    Table {
        tbl_pr: Some(Box::new(flags)),
        tbl_grid: Some(casual_slide_model::table::TableGrid {
            grid_col: vec![Default::default(); columns],
        }),
        rows: vec![
            TableRow {
                height: None,
                cells: vec![TableCell::default(); columns],
            };
            rows
        ],
    }
}

fn all_on() -> TableProperties {
    TableProperties {
        first_row: Some(true),
        first_col: Some(true),
        last_row: Some(true),
        last_col: Some(true),
        band_row: Some(true),
        band_col: Some(true),
        ..TableProperties::default()
    }
}

/// A style whose every part names its kind by a style matrix index.
fn style() -> TableStyle {
    let mut s = TableStyle {
        style_id: Some(ID.to_owned()),
        ..TableStyle::default()
    };
    for (i, kind) in K::ALL.iter().enumerate() {
        *s.slot_mut(*kind) = Some(Box::new(TablePartStyle {
            tc_tx_style: None,
            tc_style: Some(TableStyleCellStyle {
                fill: Some(TableStyleFill::Ref(StyleMatrixRef {
                    idx: Some(i as u32),
                    color: None,
                })),
                ..TableStyleCellStyle::default()
            }),
        }));
    }
    s
}

#[test]
fn the_part_order_is_the_one_mso_oi29500_lists() {
    let listed = [
        "wholeTbl", "band1H", "band2H", "band1V", "band2V", "lastCol", "firstCol", "lastRow",
        "seCell", "swCell", "firstRow", "neCell", "nwCell",
    ];
    let ours: Vec<&str> = K::ALL.iter().map(|k| k.as_xml()).collect();
    assert_eq!(ours, listed);
}

#[test]
fn corners_belong_to_their_row_and_column_and_apply_last() {
    let t = grid(4, 4, all_on());
    assert_eq!(
        part_kinds(&t, 0, 0),
        [K::WholeTable, K::FirstColumn, K::FirstRow, K::NorthWestCell]
    );
    assert_eq!(
        part_kinds(&t, 0, 3),
        [K::WholeTable, K::LastColumn, K::FirstRow, K::NorthEastCell]
    );
    assert_eq!(
        part_kinds(&t, 3, 0),
        [K::WholeTable, K::FirstColumn, K::LastRow, K::SouthWestCell]
    );
    assert_eq!(
        part_kinds(&t, 3, 3),
        [K::WholeTable, K::LastColumn, K::LastRow, K::SouthEastCell]
    );
}

#[test]
fn banding_counts_from_past_the_header_and_skips_a_total() {
    let t = grid(5, 1, all_on());
    let bands = |row| {
        part_kinds(&t, row, 0)
            .into_iter()
            .filter(|k| matches!(k, K::Band1Horizontal | K::Band2Horizontal))
            .collect::<Vec<_>>()
    };
    assert_eq!(bands(0), []);
    assert_eq!(bands(1), [K::Band1Horizontal]);
    assert_eq!(bands(2), [K::Band2Horizontal]);
    assert_eq!(bands(3), [K::Band1Horizontal]);
    assert_eq!(bands(4), []);
}

#[test]
fn with_no_flags_only_the_whole_table_part_applies() {
    let t = grid(3, 3, TableProperties::default());
    for r in 0..3 {
        for c in 0..3 {
            assert_eq!(part_kinds(&t, r, c), [K::WholeTable]);
        }
    }
    let bare = Table::default();
    assert_eq!(part_kinds(&bare, 0, 0), [K::WholeTable]);
}

#[test]
fn the_latest_part_answers_and_the_cells_own_wins_over_all() {
    let mut t = grid(2, 2, all_on());
    let s = style();
    let m = t.merges();
    let nw = CellChain::new(&t, &m, Some(&s), 0, 0).fill().unwrap();
    assert_eq!(nw.level, CellLevel::Part(K::NorthWestCell));

    t.rows[0].cells[0].tc_pr = Some(Box::new(TableCellProperties {
        fill: Some(Fill::None),
        ..TableCellProperties::default()
    }));
    let own = CellChain::new(&t, &m, Some(&s), 0, 0).fill().unwrap();
    assert_eq!(own.level, CellLevel::Cell);
    assert!(matches!(own.value, CellFill::Cell(Fill::None)));
}

#[test]
fn a_part_with_nothing_to_say_passes_to_the_next() {
    let t = grid(2, 2, all_on());
    let mut s = style();
    *s.slot_mut(K::NorthWestCell) = Some(Box::default());
    let m = t.merges();
    let fill = CellChain::new(&t, &m, Some(&s), 0, 0).fill().unwrap();
    assert_eq!(fill.level, CellLevel::Part(K::FirstRow));

    // `def` says nothing either.
    s.first_row = Some(Box::new(TablePartStyle {
        tc_tx_style: Some(TableStyleTextStyle {
            bold: Some(OnOffStyle::Def),
            ..TableStyleTextStyle::default()
        }),
        tc_style: None,
    }));
    s.whole_tbl = Some(Box::new(TablePartStyle {
        tc_tx_style: Some(TableStyleTextStyle {
            bold: Some(OnOffStyle::On),
            ..TableStyleTextStyle::default()
        }),
        tc_style: None,
    }));
    let bold = CellChain::new(&t, &m, Some(&s), 0, 0).bold().unwrap();
    assert_eq!(bold.level, CellLevel::Part(K::WholeTable));
    assert_eq!(bold.value, OnOffStyle::On);
}

#[test]
fn a_covered_cell_answers_as_its_origin_does() {
    let mut t = grid(2, 2, TableProperties::default());
    t.rows[0].cells[0].grid_span = Some(2);
    t.rows[0].cells[0].tc_pr = Some(Box::new(TableCellProperties {
        fill: Some(Fill::Group),
        ..TableCellProperties::default()
    }));
    t.rows[0].cells[1].h_merge = Some(true);
    let m = t.merges();
    let covered = CellChain::new(&t, &m, None, 0, 1).fill().unwrap();
    assert!(matches!(covered.value, CellFill::Cell(Fill::Group)));
    assert_eq!(CellChain::new(&t, &m, None, 1, 1).fill(), None);
}

#[test]
fn margins_and_the_rest_default_as_the_schema_says_and_say_so() {
    let t = grid(1, 1, TableProperties::default());
    let m = t.merges();
    let c = CellChain::new(&t, &m, None, 0, 0);
    assert_eq!(c.margin_left().value, 91_440);
    assert_eq!(c.margin_left().level, CellLevel::SchemaDefault);
    assert_eq!(c.margin_right().value, 91_440);
    assert_eq!(c.margin_top().value, 45_720);
    assert_eq!(c.margin_bottom().value, 45_720);
    assert!(!c.anchor_ctr().value);
}

#[test]
fn a_style_is_found_inline_listed_or_not_at_all_and_an_unknown_guid_is_an_answer() {
    let mut t = grid(1, 1, TableProperties::default());
    assert_eq!(table_style(&t, None), StyleSource::None);

    let list = TableStyleList {
        def: None,
        tbl_style: vec![style()],
    };
    let by_id = |id: &str| {
        Some(Box::new(TableProperties {
            style: Some(TableStyleChoice::Id(id.to_owned())),
            ..TableProperties::default()
        }))
    };
    t.tbl_pr = by_id(ID);
    assert!(matches!(
        table_style(&t, Some(&list)),
        StyleSource::Listed(_)
    ));
    // GUIDs compare by value.
    t.tbl_pr = by_id(&ID.to_ascii_lowercase());
    assert!(matches!(
        table_style(&t, Some(&list)),
        StyleSource::Listed(_)
    ));

    t.tbl_pr = by_id("{00000000-0000-0000-0000-000000000000}");
    let unknown = table_style(&t, Some(&list));
    assert!(matches!(unknown, StyleSource::NotInDeck(_)));
    assert_eq!(unknown.style(), None);
    assert!(matches!(table_style(&t, None), StyleSource::NotInDeck(_)));

    // And it resolves, with only the cell's own to go on.
    let m = t.merges();
    assert_eq!(CellChain::new(&t, &m, unknown.style(), 0, 0).fill(), None);

    t.tbl_pr = Some(Box::new(TableProperties {
        style: Some(TableStyleChoice::Inline(Box::new(style()))),
        ..TableProperties::default()
    }));
    assert!(matches!(table_style(&t, None), StyleSource::Inline(_)));
}

#[test]
fn a_position_with_no_cell_or_a_hostile_shape_never_panics() {
    let t = Table {
        tbl_pr: None,
        tbl_grid: None,
        rows: vec![TableRow {
            height: None,
            cells: vec![TableCell {
                grid_span: Some(-1),
                row_span: Some(i32::MIN),
                ..TableCell::default()
            }],
        }],
    };
    let m = t.merges();
    let s = style();
    for (r, c) in [(0, 0), (9, 9), (usize::MAX, usize::MAX)] {
        let chain = CellChain::new(&t, &m, Some(&s), r, c);
        assert!(chain.fill().is_some());
        let _ = (
            chain.bold(),
            chain.italic(),
            chain.text_color(),
            chain.font(),
        );
    }
}
