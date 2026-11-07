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

/// `a:CT_Table`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Table {
    /// `tblPr`.
    pub tbl_pr: Option<Box<TableProperties>>,
    /// `tblGrid`. A grid with no column and no grid at all differ.
    pub tbl_grid: Option<TableGrid>,
    /// `tr`.
    pub rows: Vec<TableRow>,
}

/// `a:CT_TableGrid`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TableGrid {
    /// `gridCol`.
    pub grid_col: Vec<TableCol>,
}

/// `a:CT_Headers`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Headers {
    /// `header`, as text.
    pub header: Vec<String>,
}

/// `a:CT_TableCol`. `extLst`, which holds `a16:colId`, stays in the tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TableCol {
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
    pub tx_body: Option<Box<TextBody>>,
    /// `tcPr`.
    pub tc_pr: Option<Box<TableCellProperties>>,
}

/// `a:CT_TableCellProperties`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TableCellProperties {
    /// `lnL`.
    pub ln_l: Option<Box<Line>>,
    /// `lnR`.
    pub ln_r: Option<Box<Line>>,
    /// `lnT`.
    pub ln_t: Option<Box<Line>>,
    /// `lnB`.
    pub ln_b: Option<Box<Line>>,
    /// `lnTlToBr`.
    pub ln_tl_to_br: Option<Box<Line>>,
    /// `lnBlToTr`.
    pub ln_bl_to_tr: Option<Box<Line>>,
    /// `cell3D`.
    pub cell3d: Option<Cell3D>,
    /// `EG_FillProperties`.
    pub fill: Option<Fill>,
    /// `headers`. A list present and empty is not an absent one.
    pub headers: Option<Headers>,
    /// `@marL`.
    pub mar_l: Option<Written<Coordinate>>,
    /// `@marR`.
    pub mar_r: Option<Written<Coordinate>>,
    /// `@marT`.
    pub mar_t: Option<Written<Coordinate>>,
    /// `@marB`.
    pub mar_b: Option<Written<Coordinate>>,
    /// `@vert`.
    pub vert: Option<TextVertical>,
    /// `@anchor`.
    pub anchor: Option<TextAnchor>,
    /// `@anchorCtr`.
    pub anchor_ctr: Option<bool>,
    /// `@horzOverflow`.
    pub horz_overflow: Option<HorizontalOverflow>,
}

/// `a:CT_Cell3D`. `extLst` stays in the tree.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Cell3D {
    /// `@prstMaterial`. Office's default is `dkEdge`, not the schema's `plastic`
    /// (`MS-OI29500` 2.1.1415 a).
    pub prst_material: Option<PresetMaterial>,
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
    pub first_col: Option<bool>,
    /// `@lastRow`.
    pub last_row: Option<bool>,
    /// `@lastCol`.
    pub last_col: Option<bool>,
    /// `@bandRow`.
    pub band_row: Option<bool>,
    /// `@bandCol`.
    pub band_col: Option<bool>,
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
    pub def: Option<String>,
    /// `tblStyle`, in order.
    pub tbl_style: Vec<TableStyle>,
}

impl TableStyleList {
    /// The first style with this id. GUIDs compare by value, so case does not count
    /// (Inferred), and a repeated id is reported on import and the first is used.
    pub fn find(&self, id: &str) -> Option<&TableStyle> {
        self.tbl_style.iter().find(|s| {
            s.style_id
                .as_deref()
                .is_some_and(|mine| mine.eq_ignore_ascii_case(id))
        })
    }
}

/// `a:CT_TableStyle`. `extLst` stays in the tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TableStyle {
    /// `@styleId`.
    pub style_id: Option<String>,
    /// `@styleName`.
    pub style_name: Option<String>,
    /// `tblBg`.
    pub tbl_bg: Option<TableBackgroundStyle>,
    /// `wholeTbl`.
    pub whole_tbl: Option<Box<TablePartStyle>>,
    /// `band1H`.
    pub band1_h: Option<Box<TablePartStyle>>,
    /// `band2H`.
    pub band2_h: Option<Box<TablePartStyle>>,
    /// `band1V`.
    pub band1_v: Option<Box<TablePartStyle>>,
    /// `band2V`.
    pub band2_v: Option<Box<TablePartStyle>>,
    /// `lastCol`.
    pub last_col: Option<Box<TablePartStyle>>,
    /// `firstCol`.
    pub first_col: Option<Box<TablePartStyle>>,
    /// `lastRow`.
    pub last_row: Option<Box<TablePartStyle>>,
    /// `seCell`.
    pub se_cell: Option<Box<TablePartStyle>>,
    /// `swCell`.
    pub sw_cell: Option<Box<TablePartStyle>>,
    /// `firstRow`.
    pub first_row: Option<Box<TablePartStyle>>,
    /// `neCell`.
    pub ne_cell: Option<Box<TablePartStyle>>,
    /// `nwCell`.
    pub nw_cell: Option<Box<TablePartStyle>>,
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
            TablePartKind::WholeTable => &self.whole_tbl,
            TablePartKind::Band1Horizontal => &self.band1_h,
            TablePartKind::Band2Horizontal => &self.band2_h,
            TablePartKind::Band1Vertical => &self.band1_v,
            TablePartKind::Band2Vertical => &self.band2_v,
            TablePartKind::LastColumn => &self.last_col,
            TablePartKind::FirstColumn => &self.first_col,
            TablePartKind::LastRow => &self.last_row,
            TablePartKind::SouthEastCell => &self.se_cell,
            TablePartKind::SouthWestCell => &self.sw_cell,
            TablePartKind::FirstRow => &self.first_row,
            TablePartKind::NorthEastCell => &self.ne_cell,
            TablePartKind::NorthWestCell => &self.nw_cell,
        }
    }

    /// A part style's slot, to fill.
    pub fn slot_mut(&mut self, kind: TablePartKind) -> &mut Option<Box<TablePartStyle>> {
        match kind {
            TablePartKind::WholeTable => &mut self.whole_tbl,
            TablePartKind::Band1Horizontal => &mut self.band1_h,
            TablePartKind::Band2Horizontal => &mut self.band2_h,
            TablePartKind::Band1Vertical => &mut self.band1_v,
            TablePartKind::Band2Vertical => &mut self.band2_v,
            TablePartKind::LastColumn => &mut self.last_col,
            TablePartKind::FirstColumn => &mut self.first_col,
            TablePartKind::LastRow => &mut self.last_row,
            TablePartKind::SouthEastCell => &mut self.se_cell,
            TablePartKind::SouthWestCell => &mut self.sw_cell,
            TablePartKind::FirstRow => &mut self.first_row,
            TablePartKind::NorthEastCell => &mut self.ne_cell,
            TablePartKind::NorthWestCell => &mut self.nw_cell,
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
    pub tc_tx_style: Option<TableStyleTextStyle>,
    /// `tcStyle`.
    pub tc_style: Option<TableStyleCellStyle>,
}

/// `a:CT_TableStyleTextStyle`. `extLst` stays in the tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TableStyleTextStyle {
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
pub struct TableStyleCellStyle {
    /// `tcBdr`.
    pub tc_bdr: Option<TableCellBorderStyle>,
    /// `fill` or `fillRef`.
    pub fill: Option<TableStyleFill>,
    /// `cell3D`.
    pub cell3d: Option<Cell3D>,
}

/// `a:CT_TableCellBorderStyle`. `extLst` stays in the tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TableCellBorderStyle {
    /// `left`.
    pub left: Option<TableBorder>,
    /// `right`.
    pub right: Option<TableBorder>,
    /// `top`.
    pub top: Option<TableBorder>,
    /// `bottom`.
    pub bottom: Option<TableBorder>,
    /// `insideH`.
    pub inside_h: Option<TableBorder>,
    /// `insideV`.
    pub inside_v: Option<TableBorder>,
    /// `tl2br`.
    pub tl2br: Option<TableBorder>,
    /// `tr2bl`.
    pub tr2bl: Option<TableBorder>,
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
        self.tbl_grid.as_ref().map_or(0, |g| g.grid_col.len())
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
                    let first_col = column == cover.origin_column;
                    if !flagged_v || flagged_h == first_col {
                        map.faults.push(MergeFault::Overlap { row, column });
                    }
                    let wide = if first_col { cover.columns } else { 1 };
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
