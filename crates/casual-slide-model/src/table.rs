//! Tables and table styles as plain typed values (`docs/spec/TABLE.md`): `a:tbl`, its
//! grid, rows and cells, and `a:tblStyleLst`.
//!
//! Each field is what the file said, or `None`. Nothing here lays a table out: a row's
//! height is what was stored, and how a row grows to fit its text is Tier 2E's. A cell's
//! fills and borders are Tier 1E's types and its text Tier 1F's.
//!
//! `a16:colId`, `a16:rowId`, and every other `extLst` stay in the tree.
//!
//! Evidence: `MS-OI29500` v20260818, section numbers on each use.

use std::collections::HashMap;

use crate::color::{Color, xml_enum};
use crate::d3::{Bevel, LightRig, PresetMaterial};
use crate::effect::EffectProperties;
use crate::fill::Fill;
use crate::line::Line;
use crate::presentation::Coordinate;
use crate::text::{HorizontalOverflow, TextAnchor, TextBody, TextVertical, Written};
use crate::theme::{OnOffStyle, StyleMatrixRef, ThemeableFont};

/// The most rows or columns Office accepts (`MS-OI29500` 2.1.1424 a and b, 2.1.1425).
/// Beyond it a table is kept and reported.
pub const OFFICE_TABLE_DIMENSION_LIMIT: usize = 1_000;
/// The tallest row Office accepts (`MS-OI29500` 2.1.1427 a).
pub const OFFICE_ROW_HEIGHT_LIMIT: i64 = 0x3FFF_FFFF;

/// The schema's name of each field of these types, where a field has a clearer name of
/// its own. For a reader of this page, and `tools/spec-audit`, which matches names.
pub const SCHEMA_NAMES: &[&str] = &[
    "lnL",
    "lnR",
    "lnT",
    "lnB",
    "lnTlToBr",
    "lnBlToTr",
    "marL",
    "marR",
    "marT",
    "marB",
    "anchorCtr",
    "horzOverflow",
    "tblGrid",
    "gridCol",
    "txBody",
    "tcPr",
    "tc",
    "tableStyleId",
    "bandCol",
    "tblPr",
    "prstMaterial",
    "ln",
    "lnRef",
    "insideH",
    "insideV",
    "tl2br",
    "tr2bl",
    "tcBdr",
    "tcTxStyle",
    "tcStyle",
    "tblBg",
    "styleId",
    "styleName",
    "tblStyle",
    "tblStyleLst",
    "effectLst",
    "effectDag",
    "CT_TableGrid",
    "CT_ThemeableLineStyle",
    "CT_TableBackgroundStyle",
    "CT_TablePartStyle",
];

/// `a:CT_Table`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Table {
    /// `tblPr`.
    pub properties: Option<Box<TableProperties>>,
    /// `tblGrid`. A grid with no column and no grid at all differ.
    pub grid: Option<Vec<GridColumn>>,
    /// `tr`.
    pub rows: Vec<TableRow>,
}

/// `a:CT_TableCol`. `extLst`, which holds `a16:colId`, stays in the tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GridColumn {
    /// `@w`.
    pub width: Option<Written<Coordinate>>,
}

/// `a:CT_TableRow`. `extLst`, which holds `a16:rowId`, stays in the tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TableRow {
    /// `@h`, as stored. The drawn height is Tier 2E's.
    pub height: Option<Written<Coordinate>>,
    /// `tc`. In Office, as many as the grid has columns, covered cells included, so a
    /// cell's place in its row is its column (`MS-OI29500` 2.1.1424 c).
    pub cells: Vec<TableCell>,
}

/// `a:CT_TableCell`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TableCell {
    /// `@rowSpan`. One that is under 1 or leaves the table is reported and left out.
    pub row_span: Option<i32>,
    /// `@gridSpan`. One that is under 1 or leaves the grid is reported and left out.
    pub grid_span: Option<i32>,
    /// `@hMerge`.
    pub h_merge: Option<bool>,
    /// `@vMerge`.
    pub v_merge: Option<bool>,
    /// `@id`.
    pub id: Option<String>,
    /// `txBody`.
    pub text: Option<Box<TextBody>>,
    /// `tcPr`.
    pub properties: Option<Box<TableCellProperties>>,
}

/// `a:CT_TableCellProperties`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TableCellProperties {
    /// `lnL`.
    pub line_left: Option<Box<Line>>,
    /// `lnR`.
    pub line_right: Option<Box<Line>>,
    /// `lnT`.
    pub line_top: Option<Box<Line>>,
    /// `lnB`.
    pub line_bottom: Option<Box<Line>>,
    /// `lnTlToBr`.
    pub line_top_left_to_bottom_right: Option<Box<Line>>,
    /// `lnBlToTr`.
    pub line_bottom_left_to_top_right: Option<Box<Line>>,
    /// `cell3D`.
    pub cell3d: Option<Cell3D>,
    /// `EG_FillProperties`.
    pub fill: Option<Fill>,
    /// `headers/header`, as text. A list present and empty is not an absent one.
    pub headers: Option<Vec<String>>,
    /// `@marL`.
    pub margin_left: Option<Written<Coordinate>>,
    /// `@marR`.
    pub margin_right: Option<Written<Coordinate>>,
    /// `@marT`.
    pub margin_top: Option<Written<Coordinate>>,
    /// `@marB`.
    pub margin_bottom: Option<Written<Coordinate>>,
    /// `@vert`.
    pub vertical: Option<TextVertical>,
    /// `@anchor`.
    pub anchor: Option<TextAnchor>,
    /// `@anchorCtr`.
    pub anchor_center: Option<bool>,
    /// `@horzOverflow`.
    pub horizontal_overflow: Option<HorizontalOverflow>,
}

/// `a:CT_Cell3D`. `extLst` stays in the tree.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Cell3D {
    /// `@prstMaterial`. Office's default is `dkEdge`, not the schema's `plastic`
    /// (`MS-OI29500` 2.1.1415 a).
    pub material: Option<PresetMaterial>,
    /// `bevel`.
    pub bevel: Option<Bevel>,
    /// `lightRig`. Office takes a `flood` rig when there is none (2.1.1415 b).
    pub light_rig: Option<LightRig>,
}

/// `a:CT_TableProperties`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TableProperties {
    /// `EG_FillProperties`.
    pub fill: Option<Fill>,
    /// `effectLst` or `effectDag`.
    pub effects: Option<EffectProperties>,
    /// `tableStyle` or `tableStyleId`.
    pub style: Option<TableStyleChoice>,
    /// `@rtl`.
    pub rtl: Option<bool>,
    /// `@firstRow`.
    pub first_row: Option<bool>,
    /// `@firstCol`.
    pub first_column: Option<bool>,
    /// `@lastRow`.
    pub last_row: Option<bool>,
    /// `@lastCol`.
    pub last_column: Option<bool>,
    /// `@bandRow`.
    pub band_row: Option<bool>,
    /// `@bandCol`.
    pub band_column: Option<bool>,
}

/// The choice in `a:CT_TableProperties`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TableStyleChoice {
    /// `tableStyle`: the style, in the table.
    Inline(Box<TableStyle>),
    /// `tableStyleId`: a GUID, as written. Only a reference: nothing says it names
    /// a style this deck holds (BEH-29).
    Id(String),
}

/// `a:CT_TableStyleList`, the root of the `tableStyles` part.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TableStyleList {
    /// `@def`, as written.
    pub default_style: Option<String>,
    /// `tblStyle`, in order.
    pub styles: Vec<TableStyle>,
}

impl TableStyleList {
    /// The first style with this id. GUIDs compare by value, so case does not count
    /// (Inferred), and a repeated id is reported on import and the first is used.
    pub fn find(&self, id: &str) -> Option<&TableStyle> {
        self.styles.iter().find(|s| {
            s.id.as_deref()
                .is_some_and(|mine| mine.eq_ignore_ascii_case(id))
        })
    }
}

/// `a:CT_TableStyle`. `extLst` stays in the tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TableStyle {
    /// `@styleId`.
    pub id: Option<String>,
    /// `@styleName`.
    pub name: Option<String>,
    /// `tblBg`.
    pub background: Option<TableBackgroundStyle>,
    /// `wholeTbl`.
    pub whole_table: Option<Box<TablePartStyle>>,
    /// `band1H`.
    pub band1_horizontal: Option<Box<TablePartStyle>>,
    /// `band2H`.
    pub band2_horizontal: Option<Box<TablePartStyle>>,
    /// `band1V`.
    pub band1_vertical: Option<Box<TablePartStyle>>,
    /// `band2V`.
    pub band2_vertical: Option<Box<TablePartStyle>>,
    /// `lastCol`.
    pub last_column: Option<Box<TablePartStyle>>,
    /// `firstCol`.
    pub first_column: Option<Box<TablePartStyle>>,
    /// `lastRow`.
    pub last_row: Option<Box<TablePartStyle>>,
    /// `seCell`.
    pub south_east_cell: Option<Box<TablePartStyle>>,
    /// `swCell`.
    pub south_west_cell: Option<Box<TablePartStyle>>,
    /// `firstRow`.
    pub first_row: Option<Box<TablePartStyle>>,
    /// `neCell`.
    pub north_east_cell: Option<Box<TablePartStyle>>,
    /// `nwCell`.
    pub north_west_cell: Option<Box<TablePartStyle>>,
}

xml_enum! {
    /// The part styles of a `CT_TableStyle`, in the order Office applies them, later
    /// over earlier (`MS-OI29500` 2.1.1265).
    TablePartKind {
        WholeTable = "wholeTbl",
        Band1Horizontal = "band1H",
        Band2Horizontal = "band2H",
        Band1Vertical = "band1V",
        Band2Vertical = "band2V",
        LastColumn = "lastCol",
        FirstColumn = "firstCol",
        LastRow = "lastRow",
        SouthEastCell = "seCell",
        SouthWestCell = "swCell",
        FirstRow = "firstRow",
        NorthEastCell = "neCell",
        NorthWestCell = "nwCell",
    }
}

impl TableStyle {
    /// A part style by kind.
    pub fn part(&self, kind: TablePartKind) -> Option<&TablePartStyle> {
        self.slot(kind).as_deref()
    }

    /// A part style's slot.
    pub fn slot(&self, kind: TablePartKind) -> &Option<Box<TablePartStyle>> {
        match kind {
            TablePartKind::WholeTable => &self.whole_table,
            TablePartKind::Band1Horizontal => &self.band1_horizontal,
            TablePartKind::Band2Horizontal => &self.band2_horizontal,
            TablePartKind::Band1Vertical => &self.band1_vertical,
            TablePartKind::Band2Vertical => &self.band2_vertical,
            TablePartKind::LastColumn => &self.last_column,
            TablePartKind::FirstColumn => &self.first_column,
            TablePartKind::LastRow => &self.last_row,
            TablePartKind::SouthEastCell => &self.south_east_cell,
            TablePartKind::SouthWestCell => &self.south_west_cell,
            TablePartKind::FirstRow => &self.first_row,
            TablePartKind::NorthEastCell => &self.north_east_cell,
            TablePartKind::NorthWestCell => &self.north_west_cell,
        }
    }

    /// A part style's slot, to fill.
    pub fn slot_mut(&mut self, kind: TablePartKind) -> &mut Option<Box<TablePartStyle>> {
        match kind {
            TablePartKind::WholeTable => &mut self.whole_table,
            TablePartKind::Band1Horizontal => &mut self.band1_horizontal,
            TablePartKind::Band2Horizontal => &mut self.band2_horizontal,
            TablePartKind::Band1Vertical => &mut self.band1_vertical,
            TablePartKind::Band2Vertical => &mut self.band2_vertical,
            TablePartKind::LastColumn => &mut self.last_column,
            TablePartKind::FirstColumn => &mut self.first_column,
            TablePartKind::LastRow => &mut self.last_row,
            TablePartKind::SouthEastCell => &mut self.south_east_cell,
            TablePartKind::SouthWestCell => &mut self.south_west_cell,
            TablePartKind::FirstRow => &mut self.first_row,
            TablePartKind::NorthEastCell => &mut self.north_east_cell,
            TablePartKind::NorthWestCell => &mut self.north_west_cell,
        }
    }
}

/// `a:CT_TableBackgroundStyle`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TableBackgroundStyle {
    /// `fill` or `fillRef`.
    pub fill: Option<TableStyleFill>,
    /// `effect` or `effectRef`.
    pub effect: Option<TableStyleEffect>,
}

/// `EG_ThemeableFillStyle`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TableStyleFill {
    /// `fill`, holding one of `EG_FillProperties`.
    Inline(Fill),
    /// `fillRef`.
    Ref(StyleMatrixRef),
}

/// `EG_ThemeableEffectStyle`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TableStyleEffect {
    /// `effect`, holding one of `EG_EffectProperties`.
    Inline(EffectProperties),
    /// `effectRef`.
    Ref(StyleMatrixRef),
}

/// `a:CT_TablePartStyle`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TablePartStyle {
    /// `tcTxStyle`.
    pub text: Option<TableTextStyle>,
    /// `tcStyle`.
    pub cell: Option<TableCellStyle>,
}

/// `a:CT_TableStyleTextStyle`. `extLst` stays in the tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TableTextStyle {
    /// `@b`.
    pub bold: Option<OnOffStyle>,
    /// `@i`.
    pub italic: Option<OnOffStyle>,
    /// `font` or `fontRef`.
    pub font: Option<ThemeableFont>,
    /// `EG_ColorChoice`.
    pub color: Option<Color>,
}

/// `a:CT_TableStyleCellStyle`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TableCellStyle {
    /// `tcBdr`.
    pub borders: Option<TableCellBorders>,
    /// `fill` or `fillRef`.
    pub fill: Option<TableStyleFill>,
    /// `cell3D`.
    pub cell3d: Option<Cell3D>,
}

/// `a:CT_TableCellBorderStyle`. `extLst` stays in the tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TableCellBorders {
    /// `left`.
    pub left: Option<TableBorder>,
    /// `right`.
    pub right: Option<TableBorder>,
    /// `top`.
    pub top: Option<TableBorder>,
    /// `bottom`.
    pub bottom: Option<TableBorder>,
    /// `insideH`.
    pub inside_horizontal: Option<TableBorder>,
    /// `insideV`.
    pub inside_vertical: Option<TableBorder>,
    /// `tl2br`.
    pub top_left_to_bottom_right: Option<TableBorder>,
    /// `tr2bl`.
    pub top_right_to_bottom_left: Option<TableBorder>,
}

/// `a:CT_ThemeableLineStyle`. An empty one is `Empty`, which no schema allows and which
/// is reported, so the element stays where it is.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum TableBorder {
    /// Neither `ln` nor `lnRef`.
    #[default]
    Empty,
    /// `ln`.
    Line(Box<Line>),
    /// `lnRef`.
    Ref(StyleMatrixRef),
}

/// One merge of adjoining cells, by its top-left cell (`MS-OI29500` 2.1.1424 d).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableMerge {
    /// The origin's row.
    pub row: usize,
    /// The origin's column.
    pub column: usize,
    /// Rows covered, at least 1.
    pub rows: usize,
    /// Columns covered, at least 1.
    pub columns: usize,
}

/// What does not hold of a table's merges by `MS-OI29500` 2.1.1424 d and 2.1.1426 c.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeFault {
    /// A cell inside a merge lacks the flag saying so, or starts a merge of its own: two
    /// merges overlap.
    Overlap {
        /// The cell's row.
        row: usize,
        /// The cell's column.
        column: usize,
    },
    /// `hMerge` or `vMerge` on a cell that no merge covers.
    NoOrigin {
        /// The cell's row.
        row: usize,
        /// The cell's column.
        column: usize,
    },
    /// A covered cell whose own spans disagree with its merge's.
    SpansDisagree {
        /// The cell's row.
        row: usize,
        /// The cell's column.
        column: usize,
    },
}

/// A table's merges, found in one pass.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MergeMap {
    /// Each merge, by its origin, in document order.
    pub merges: Vec<TableMerge>,
    /// What does not hold of them.
    pub faults: Vec<MergeFault>,
    /// Each covered cell's `(row, column)`, to its origin's.
    pub origins: HashMap<(usize, usize), (usize, usize)>,
}

/// What covers one column of the row being walked.
#[derive(Debug, Clone, Copy, Default)]
struct Cover {
    /// The first row below the merge.
    until_row: usize,
    /// The merge's first row.
    origin_row: usize,
    /// The merge's first column.
    origin_column: usize,
    /// The merge's rows.
    rows: usize,
    /// The merge's columns.
    columns: usize,
}

fn span(value: Option<i32>) -> usize {
    value
        .and_then(|v| usize::try_from(v).ok())
        .unwrap_or(1)
        .max(1)
}

impl Table {
    /// The number of columns the grid states.
    pub fn column_count(&self) -> usize {
        self.grid.as_ref().map_or(0, Vec::len)
    }

    /// The table's merges, and what does not hold of them. One pass over the cells,
    /// with one slot per grid column: no span from a file sizes anything or is walked
    /// over, so overlapping merges cost what the cells cost. A cell past the grid
    /// neither starts nor ends a merge.
    pub fn merges(&self) -> MergeMap {
        let columns = self.column_count();
        let rows = self.rows.len();
        let mut map = MergeMap::default();
        let mut covers = vec![Cover::default(); columns];
        for (row, tr) in self.rows.iter().enumerate() {
            // The merge this row started, from its origin rightwards.
            let mut across: Option<(usize, Cover)> = None;
            for (column, cell) in tr.cells.iter().enumerate() {
                let Some(slot) = covers.get_mut(column) else {
                    continue;
                };
                let horizontal = across.filter(|(end, _)| column < *end).map(|(_, c)| c);
                let vertical = (row < slot.until_row).then_some(*slot);
                let flagged_h = cell.h_merge == Some(true);
                let flagged_v = cell.v_merge == Some(true);
                if let Some(cover) = vertical {
                    let first_column = column == cover.origin_column;
                    if !flagged_v || flagged_h == first_column {
                        map.faults.push(MergeFault::Overlap { row, column });
                    }
                    let wide = if first_column { cover.columns } else { 1 };
                    if span(cell.grid_span) != wide || span(cell.row_span) != 1 {
                        map.faults.push(MergeFault::SpansDisagree { row, column });
                    }
                    map.origins
                        .insert((row, column), (cover.origin_row, cover.origin_column));
                    continue;
                }
                if let Some(cover) = horizontal {
                    if !flagged_h || flagged_v {
                        map.faults.push(MergeFault::Overlap { row, column });
                    }
                    if span(cell.row_span) != cover.rows || span(cell.grid_span) != 1 {
                        map.faults.push(MergeFault::SpansDisagree { row, column });
                    }
                    map.origins
                        .insert((row, column), (cover.origin_row, cover.origin_column));
                    *slot = cover;
                    continue;
                }
                if flagged_h || flagged_v {
                    map.faults.push(MergeFault::NoOrigin { row, column });
                    continue;
                }
                let (down, wide) = (span(cell.row_span), span(cell.grid_span));
                if down == 1 && wide == 1 {
                    continue;
                }
                // Clamped, for a model that was not read from a file.
                let down = down.min(rows.saturating_sub(row)).max(1);
                let wide = wide.min(columns.saturating_sub(column)).max(1);
                map.merges.push(TableMerge {
                    row,
                    column,
                    rows: down,
                    columns: wide,
                });
                let cover = Cover {
                    until_row: row.saturating_add(down),
                    origin_row: row,
                    origin_column: column,
                    rows: down,
                    columns: wide,
                };
                *slot = cover;
                across = Some((column.saturating_add(wide), cover));
            }
        }
        map
    }
}
