//! What a table's style says of a cell, at read time: which style applies, which of its
//! part styles a cell belongs to, and which of them answers (Tier 1G).
//!
//! Positions here are grid positions, never sizes. Nothing here lays out, measures or
//! draws: row growth, and which of two neighbouring cells' borders is drawn, are Tier
//! 2E's (BEH-14), so a style's borders are given as the part styles that hold them and
//! are not resolved to a cell's edges.
//!
//! | Rule | Evidence |
//! | --- | --- |
//! | Part styles apply in the order `TablePartKind` lists, later over earlier | `MS-OI29500` 2.1.1265 |
//! | A `tableStyleId` naming nothing gives no style | `MS-OI29500` 2.1.1423, for what is neither built in nor in the file. **We hold no built-in definitions**, so [`StyleSource::NotInDeck`] does not say Office applies none: real decks refer to built-ins with an empty list (`T1G` report) |
//! | A cell's `tcPr` wins over the style, whole property by whole property | **Inferred** |
//! | `firstRow`, `lastRow`, `firstCol`, `lastCol`, `bandRow`, `bandCol` turn on the parts of the same name, and an absent flag is the schema's `false` | Part names: ECMA-376 21.1.3; **Inferred** for the mapping |
//! | A corner part applies only with both its flags on | **Inferred** |
//! | Banding counts from the first row or column after a header turned on, and a header or a total is not banded | **Inferred** |
//! | A covered cell answers with its merge's origin | `MS-OI29500` 2.1.1424 d says the origin's spans decide the merge; that its properties speak for it is **Inferred** |
//! | Margins, anchor, overflow and `vert` default as the schema does | `docs/spec/TABLE.md` |
//! | `prstMaterial` defaults to `dkEdge`, and a missing `lightRig` is `flood` | `MS-OI29500` 2.1.1415; those are constants in the model's documentation and are not resolved here |

use casual_slide_model::color::Color;
use casual_slide_model::presentation::Coordinate;
use casual_slide_model::table::{
    MergeMap, Table, TableCell, TableCellProperties, TablePartKind, TablePartStyle,
    TableProperties, TableStyle, TableStyleChoice, TableStyleFill, TableStyleList,
};
use casual_slide_model::text::{HorizontalOverflow, TextAnchor, TextVertical};
use casual_slide_model::theme::{OnOffStyle, ThemeableFont};

/// `tcPr/@marL` and `@marR` when absent: the schema's 91440 EMU.
pub const CELL_MARGIN_HORIZONTAL_DEFAULT: i64 = 91_440;
/// `tcPr/@marT` and `@marB` when absent: the schema's 45720 EMU.
pub const CELL_MARGIN_VERTICAL_DEFAULT: i64 = 45_720;

/// Where a table's style comes from. Never an error: an unknown GUID is an answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleSource<'a> {
    /// The table names no style.
    None,
    /// `tableStyle`, in the table.
    Inline(&'a TableStyle),
    /// `tableStyleId`, found in the `tableStyles` part.
    Listed(&'a TableStyle),
    /// `tableStyleId`, naming nothing in the part (BEH-29): a built-in style, or nothing.
    NotInDeck(&'a str),
}

impl<'a> StyleSource<'a> {
    /// The style, if the deck holds it.
    pub fn style(self) -> Option<&'a TableStyle> {
        match self {
            Self::Inline(s) | Self::Listed(s) => Some(s),
            Self::None | Self::NotInDeck(_) => None,
        }
    }
}

/// Which style a table takes.
pub fn table_style<'a>(table: &'a Table, list: Option<&'a TableStyleList>) -> StyleSource<'a> {
    match table.properties.as_ref().and_then(|p| p.style.as_ref()) {
        None => StyleSource::None,
        Some(TableStyleChoice::Inline(style)) => StyleSource::Inline(style),
        Some(TableStyleChoice::Id(id)) => match list.and_then(|l| l.find(id)) {
            Some(style) => StyleSource::Listed(style),
            None => StyleSource::NotInDeck(id),
        },
    }
}

/// Where a cell's value came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellLevel {
    /// The cell's own `tcPr`, or its merge's origin's.
    Cell,
    /// A part style of the table's style.
    Part(TablePartKind),
    /// No level said, so the schema's default applies.
    SchemaDefault,
}

/// An effective value and the level it came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellResolved<T> {
    /// The value.
    pub value: T,
    /// Where it came from.
    pub level: CellLevel,
}

fn on(flag: Option<bool>) -> bool {
    flag == Some(true)
}

/// The part kinds a cell at this grid position belongs to, in the order they apply.
pub fn part_kinds(table: &Table, row: usize, column: usize) -> Vec<TablePartKind> {
    let default = TableProperties::default();
    let p = table.properties.as_deref().unwrap_or(&default);
    let rows = table.rows.len();
    let columns = table.column_count();
    let top = on(p.first_row) && row == 0;
    let bottom = on(p.last_row) && rows > 0 && row == rows - 1;
    let left = on(p.first_column) && column == 0;
    let right = on(p.last_column) && columns > 0 && column == columns - 1;
    let row_band = row.saturating_sub(usize::from(on(p.first_row)));
    let column_band = column.saturating_sub(usize::from(on(p.first_column)));
    let band_row = on(p.band_row) && !top && !bottom;
    let band_column = on(p.band_column) && !left && !right;
    TablePartKind::ALL
        .iter()
        .copied()
        .filter(|kind| match kind {
            TablePartKind::WholeTable => true,
            TablePartKind::Band1Horizontal => band_row && row_band % 2 == 0,
            TablePartKind::Band2Horizontal => band_row && row_band % 2 == 1,
            TablePartKind::Band1Vertical => band_column && column_band % 2 == 0,
            TablePartKind::Band2Vertical => band_column && column_band % 2 == 1,
            TablePartKind::LastColumn => right,
            TablePartKind::FirstColumn => left,
            TablePartKind::LastRow => bottom,
            TablePartKind::SouthEastCell => bottom && right,
            TablePartKind::SouthWestCell => bottom && left,
            TablePartKind::FirstRow => top,
            TablePartKind::NorthEastCell => top && right,
            TablePartKind::NorthWestCell => top && left,
        })
        .collect()
}

/// One cell's chain: its own properties, then the part styles it belongs to, latest first.
#[derive(Debug, Clone)]
pub struct CellChain<'a> {
    cell: Option<&'a TableCellProperties>,
    parts: Vec<(TablePartKind, &'a TablePartStyle)>,
}

impl<'a> CellChain<'a> {
    /// The chain for the cell at this grid position, in a table of this style, with the
    /// map [`Table::merges`] gave, so a whole table costs one pass and not one a cell. A
    /// covered cell answers as its merge's origin does (Inferred). A position with no
    /// cell has only the style to answer.
    pub fn new(
        table: &'a Table,
        merges: &MergeMap,
        style: Option<&'a TableStyle>,
        row: usize,
        column: usize,
    ) -> Self {
        let (at_row, at_column) = merges
            .origins
            .get(&(row, column))
            .copied()
            .unwrap_or((row, column));
        let cell = table
            .rows
            .get(at_row)
            .and_then(|r| r.cells.get(at_column))
            .and_then(|c: &TableCell| c.properties.as_deref());
        let mut parts: Vec<_> = part_kinds(table, row, column)
            .into_iter()
            .filter_map(|kind| style.and_then(|s| s.part(kind)).map(|p| (kind, p)))
            .collect();
        parts.reverse();
        Self { cell, parts }
    }

    fn part<T>(&self, get: impl Fn(&'a TablePartStyle) -> Option<T>) -> Option<CellResolved<T>> {
        self.parts.iter().find_map(|(kind, part)| {
            get(part).map(|value| CellResolved {
                value,
                level: CellLevel::Part(*kind),
            })
        })
    }

    fn own<T>(&self, value: Option<T>, default: T) -> CellResolved<T> {
        match value {
            Some(value) => CellResolved {
                value,
                level: CellLevel::Cell,
            },
            None => CellResolved {
                value: default,
                level: CellLevel::SchemaDefault,
            },
        }
    }

    /// The cell's fill: its own, else the latest part style's that has one. `None` if
    /// nothing says, and then the table's own fill and `tblBg` are what lies behind it.
    pub fn fill(&self) -> Option<CellResolved<CellFill<'a>>> {
        if let Some(fill) = self.cell.and_then(|c| c.fill.as_ref()) {
            return Some(CellResolved {
                value: CellFill::Cell(fill),
                level: CellLevel::Cell,
            });
        }
        self.part(|p| p.cell.as_ref()?.fill.as_ref().map(CellFill::Style))
    }

    /// `tcTxStyle/@b`. `def` says nothing, so the next part answers.
    pub fn bold(&self) -> Option<CellResolved<OnOffStyle>> {
        self.part(|p| p.text.as_ref()?.bold.filter(|v| *v != OnOffStyle::Def))
    }

    /// `tcTxStyle/@i`. `def` says nothing, so the next part answers.
    pub fn italic(&self) -> Option<CellResolved<OnOffStyle>> {
        self.part(|p| p.text.as_ref()?.italic.filter(|v| *v != OnOffStyle::Def))
    }

    /// `tcTxStyle`'s colour.
    pub fn text_color(&self) -> Option<CellResolved<&'a Color>> {
        self.part(|p| p.text.as_ref()?.color.as_ref())
    }

    /// `tcTxStyle`'s font.
    pub fn font(&self) -> Option<CellResolved<&'a ThemeableFont>> {
        self.part(|p| p.text.as_ref()?.font.as_ref())
    }

    /// `@marL`, in EMU.
    pub fn margin_left(&self) -> CellResolved<i64> {
        let value = self.cell.and_then(|c| c.margin_left.as_ref());
        self.own(value.map(|m| m.value().0), CELL_MARGIN_HORIZONTAL_DEFAULT)
    }

    /// `@marR`, in EMU.
    pub fn margin_right(&self) -> CellResolved<i64> {
        let value = self.cell.and_then(|c| c.margin_right.as_ref());
        self.own(value.map(|m| m.value().0), CELL_MARGIN_HORIZONTAL_DEFAULT)
    }

    /// `@marT`, in EMU.
    pub fn margin_top(&self) -> CellResolved<i64> {
        let value = self.cell.and_then(|c| c.margin_top.as_ref());
        self.own(value.map(|m| m.value().0), CELL_MARGIN_VERTICAL_DEFAULT)
    }

    /// `@marB`, in EMU.
    pub fn margin_bottom(&self) -> CellResolved<i64> {
        let value = self.cell.and_then(|c| c.margin_bottom.as_ref());
        self.own(
            value.map(|m: &casual_slide_model::text::Written<Coordinate>| m.value().0),
            CELL_MARGIN_VERTICAL_DEFAULT,
        )
    }

    /// `@anchor`.
    pub fn anchor(&self) -> CellResolved<TextAnchor> {
        self.own(self.cell.and_then(|c| c.anchor), TextAnchor::Top)
    }

    /// `@vert`.
    pub fn vertical(&self) -> CellResolved<TextVertical> {
        self.own(self.cell.and_then(|c| c.vertical), TextVertical::Horz)
    }

    /// `@anchorCtr`.
    pub fn anchor_center(&self) -> CellResolved<bool> {
        self.own(self.cell.and_then(|c| c.anchor_center), false)
    }

    /// `@horzOverflow`.
    pub fn horizontal_overflow(&self) -> CellResolved<HorizontalOverflow> {
        self.own(
            self.cell.and_then(|c| c.horizontal_overflow),
            HorizontalOverflow::Clip,
        )
    }
}

/// A cell's effective fill.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellFill<'a> {
    /// `tcPr`'s own.
    Cell(&'a casual_slide_model::fill::Fill),
    /// A part style's.
    Style(&'a TableStyleFill),
}
