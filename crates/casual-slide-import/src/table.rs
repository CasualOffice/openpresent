//! Read tables and table styles from an [`XmlTree`] (Tier 1G): `a:tbl` in a graphic
//! frame, and the `tableStyles` part.
//!
//! By the same rule as the rest of this crate. A usable value goes in the model. An
//! unusable one is reported and left out, and stays in the tree. An element nothing here
//! knows, `extLst` among them, is reported and stays in the tree. A count over a bound in
//! [`TableLimits`] refuses, with the bound's own code.
//!
//! What `MS-OI29500` says Office requires, and the schema does not, is reported and
//! never repaired. A table may hold a row of the wrong length, merges that overlap, or a
//! style nobody defines (BEH-29), and it opens, saves and loses nothing. A span is the
//! exception to keeping what the schema allows: under 1 or leaving the table, it would
//! be a bound on every consumer's indexing, so it is left out as an unusable value.
//!
//! A table has a fixed depth. Fills, lines and effects have their own bounds, under
//! [`DrawingLimits`], and text its own under `TextLimits`. No count from the file sizes
//! anything: spans are checked and merges found in one pass over the cells that exist.
//!
//! Evidence: `MS-OI29500` v20260818, sections 2.1.1265 and 2.1.1415 to 2.1.1427, by
//! section at each use. A GUID comparing without regard to case is Inferred.

use std::collections::HashSet;

use casual_slide_model::color::Color;
use casual_slide_model::presentation::{COORDINATE_MIN, Coordinate, POSITIVE_COORDINATE_MAX};
use casual_slide_model::table::{
    Cell3D, GridColumn, MergeFault, OFFICE_ROW_HEIGHT_LIMIT, OFFICE_TABLE_DIMENSION_LIMIT, Table,
    TableBackgroundStyle, TableBorder, TableCell, TableCellBorders, TableCellProperties,
    TableCellStyle, TablePartKind, TablePartStyle, TableProperties, TableRow, TableStyle,
    TableStyleChoice, TableStyleEffect, TableStyleFill, TableStyleList, TableTextStyle,
};
use casual_slide_model::text::Written;
use casual_slide_model::theme::OnOffStyle;
use casual_slide_ooxml::{PresentationPackage, Relationship, XmlTree};

use crate::color::{Findings as ColorFindings, is_color_element, read_color};
use crate::drawing::{Ctx, DrawingLimits, is_fill_element, read_effects, read_fill, read_line};
use crate::presentation::{PartLocation, locate_part};
use crate::properties::{
    Elem, Findings, ImportFindingKind as K, ImportLimits, Reader, is_guid, ns,
};
use crate::slide::{SlideError, SlideLimits, is_part_fault};
use crate::text::{TextCounts, TextReader};
use crate::theme::{read_style_matrix_ref, read_themeable_font};

/// The `tableStyles` relationship, Transitional and Strict.
pub const TABLE_STYLES_RELATIONSHIP: &[&str] = &[
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/tableStyles",
    "http://purl.oclc.org/ooxml/officeDocument/relationships/tableStyles",
];

/// Children an element may hold beside a counted list, before the exact count.
const SLACK: usize = 8;

/// Bounds on what tables can make this reader hold. Over a bound, the part is refused.
/// Each is beyond what Office accepts, so a table Office would open is never refused by
/// one of these, though `TextLimits` can refuse the text of one that large.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableLimits {
    /// `tr` in one table. Office stops at 1000 (`MS-OI29500` 2.1.1424 a). `CS-TBL-0001`.
    pub max_rows: usize,
    /// `gridCol` in one grid. Office stops at 1000 (2.1.1424 b, 2.1.1425). `CS-TBL-0002`.
    pub max_columns: usize,
    /// `tc` in one table. `CS-TBL-0003`.
    pub max_cells_per_table: usize,
    /// `tc` across one part. `CS-TBL-0004`.
    pub max_cells_per_part: usize,
    /// `tblStyle` in the list. `CS-TBL-0005`.
    pub max_table_styles: usize,
    /// `header` in one cell's `headers`. `CS-TBL-0006`.
    pub max_headers_per_cell: usize,
}

impl Default for TableLimits {
    fn default() -> Self {
        Self {
            max_rows: 4_096,
            max_columns: 4_096,
            max_cells_per_table: 1_048_576,
            max_cells_per_part: 2_097_152,
            max_table_styles: 4_096,
            max_headers_per_cell: 1_024,
        }
    }
}

/// Which table bound was crossed. Each has its own stable code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableBound {
    /// Rows in one table.
    Rows,
    /// Columns in one grid.
    Columns,
    /// Cells in one table.
    CellsInTable,
    /// Cells across one part.
    CellsInPart,
    /// Styles in the list.
    Styles,
    /// Headers in one cell.
    Headers,
}

impl TableBound {
    /// The stable diagnostic code.
    pub fn code(self) -> &'static str {
        match self {
            Self::Rows => "CS-TBL-0001",
            Self::Columns => "CS-TBL-0002",
            Self::CellsInTable => "CS-TBL-0003",
            Self::CellsInPart => "CS-TBL-0004",
            Self::Styles => "CS-TBL-0005",
            Self::Headers => "CS-TBL-0006",
        }
    }

    pub(crate) fn what(self) -> &'static str {
        match self {
            Self::Rows => "rows in a table",
            Self::Columns => "columns in a grid",
            Self::CellsInTable => "cells in a table",
            Self::CellsInPart => "table cells in the part",
            Self::Styles => "table styles",
            Self::Headers => "headers in a cell",
        }
    }
}

/// What a table's reader noticed beyond the crate's common findings. Nothing here
/// stopped the import, and the tree keeps all of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TableFinding {
    /// A row whose cells are not one per grid column (`MS-OI29500` 2.1.1424 c). Kept.
    CellCount,
    /// A `gridSpan` or `rowSpan` under 1 or leaving the table (2.1.1426 a, b). Left out.
    SpanOutOfRange,
    /// A cell in a merge without the flags saying so, or two merges overlapping
    /// (2.1.1424 d, 2.1.1426 c). Kept.
    MergeOverlap,
    /// `hMerge` or `vMerge` on a cell no merge covers (2.1.1426 c). Kept.
    MergeNoOrigin,
    /// A covered cell whose spans disagree with its merge's (2.1.1424 d). Kept.
    MergeSpans,
    /// No rows or columns, or more than Office's 1000 (2.1.1424 a, b; 2.1.1425). Kept.
    Dimension,
    /// A row height or a column width Office would not take: negative, or a row over
    /// `0x3FFFFFFF` (2.1.1427 a). Kept.
    Measure,
    /// A `tableStyleId` or `@def` naming no style in the `tableStyles` part (BEH-29). It
    /// may be one of Office's built-in styles, which have no definition in the file:
    /// `MS-OI29500` 2.1.1423 says Office applies no style only if it is neither. Kept.
    UnknownStyle,
}

impl TableFinding {
    /// The stable diagnostic code.
    pub fn code(self) -> &'static str {
        match self {
            Self::CellCount => "CS-TBF-0001",
            Self::SpanOutOfRange => "CS-TBF-0002",
            Self::MergeOverlap => "CS-TBF-0003",
            Self::MergeNoOrigin => "CS-TBF-0004",
            Self::MergeSpans => "CS-TBF-0005",
            Self::Dimension => "CS-TBF-0006",
            Self::Measure => "CS-TBF-0007",
            Self::UnknownStyle => "CS-TBF-0008",
        }
    }
}

/// What a part has spent of the part-wide bounds so far.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct TableCounts {
    pub cells: usize,
}

fn coordinate(c: &Written<Coordinate>) -> bool {
    (COORDINATE_MIN..=POSITIVE_COORDINATE_MAX).contains(&c.value().0)
}

fn coordinate32(c: &Written<Coordinate>) -> bool {
    (i64::from(i32::MIN)..=i64::from(i32::MAX)).contains(&c.value().0)
}

/// One table, or the style list, being read.
pub(crate) struct TableReader<'a, 'r> {
    pub r: &'a mut Reader<'r>,
    pub rels: &'a [Relationship],
    pub limits: &'a TableLimits,
    pub color: &'a mut ColorFindings,
    pub text_counts: &'a mut TextCounts,
    pub counts: &'a mut TableCounts,
}

impl TableReader<'_, '_> {
    fn over(&self, bound: TableBound, limit: usize) -> SlideError {
        SlideError::Table {
            bound,
            part: self.r.part.to_owned(),
            limit,
        }
    }

    fn note(&mut self, finding: TableFinding, el: &Elem, attribute: Option<&str>) {
        self.r.note(K::Table(finding), el, attribute);
    }

    /// Refuse before the children are read at all, so a hostile list costs nothing.
    fn check_children(&self, el: &Elem, bound: TableBound, limit: usize) -> Result<(), SlideError> {
        if self.r.tree.child_elements(el.id).len() > limit.saturating_add(SLACK) {
            return Err(self.over(bound, limit));
        }
        Ok(())
    }

    /// `a:CT_Table`.
    pub fn tbl(&mut self, el: &Elem) -> Result<Table, SlideError> {
        self.r.finish(el);
        let limits = *self.limits;
        // Rows, or anything else: a table holds these and a handful of others.
        self.check_children(el, TableBound::Rows, limits.max_rows)?;
        let mut table = Table::default();
        let mut rows = Vec::new();
        for child in self.r.children(el)? {
            if !child.in_namespace(ns::DML) {
                self.r.retained(&child);
                continue;
            }
            match child.local.as_str() {
                "tr" => {
                    if rows.len() >= limits.max_rows {
                        return Err(self.over(TableBound::Rows, limits.max_rows));
                    }
                    rows.push(child);
                }
                "tblPr" if table.properties.is_some() => {
                    self.r.note(K::DuplicateElement, &child, None);
                }
                "tblPr" => table.properties = Some(Box::new(self.tbl_pr(&child)?)),
                "tblGrid" if table.grid.is_some() => {
                    self.r.note(K::DuplicateElement, &child, None);
                }
                "tblGrid" => table.grid = Some(self.tbl_grid(&child)?),
                _ => self.r.retained(&child),
            }
        }
        if table.grid.is_none() {
            self.r.note(K::MissingRequired, el, Some("tblGrid"));
        }
        let office = 1..=OFFICE_TABLE_DIMENSION_LIMIT;
        if !office.contains(&rows.len()) {
            self.note(TableFinding::Dimension, el, Some("tr"));
        }
        // The grid is read first, wherever a damaged file put it: spans check against it.
        let columns = table.column_count();
        let total = rows.len();
        let mut cells_here = 0usize;
        table.rows.reserve(total);
        for (at, tr) in rows.iter().enumerate() {
            let row = self.tr(tr, at, total, columns, &mut cells_here)?;
            table.rows.push(row);
        }
        for fault in table.merges().faults {
            let finding = match fault {
                MergeFault::Overlap { .. } => TableFinding::MergeOverlap,
                MergeFault::NoOrigin { .. } => TableFinding::MergeNoOrigin,
                MergeFault::SpansDisagree { .. } => TableFinding::MergeSpans,
            };
            self.r
                .findings
                .note(K::Table(finding), self.r.part, "tc", None);
        }
        Ok(table)
    }

    fn tbl_grid(&mut self, el: &Elem) -> Result<Vec<GridColumn>, SlideError> {
        self.r.finish(el);
        let limit = self.limits.max_columns;
        self.check_children(el, TableBound::Columns, limit)?;
        let mut grid = Vec::new();
        for child in self.r.children(el)? {
            if !child.is(ns::DML, "gridCol") {
                self.r.retained(&child);
                continue;
            }
            if grid.len() >= limit {
                return Err(self.over(TableBound::Columns, limit));
            }
            let mut e = child.clone();
            let width = self
                .r
                .required::<Written<Coordinate>>(&mut e, "w", coordinate)?;
            if width.as_ref().is_some_and(|w| w.value().0 < 0) {
                self.note(TableFinding::Measure, &child, Some("w"));
            }
            self.r.finish(&e);
            // `extLst`, which holds `a16:colId`, stays.
            for kid in self.r.children(&child)? {
                self.r.retained(&kid);
            }
            grid.push(GridColumn { width });
        }
        if !(1..=OFFICE_TABLE_DIMENSION_LIMIT).contains(&grid.len()) {
            self.note(TableFinding::Dimension, el, Some("gridCol"));
        }
        Ok(grid)
    }

    fn tr(
        &mut self,
        el: &Elem,
        at: usize,
        rows: usize,
        columns: usize,
        cells_here: &mut usize,
    ) -> Result<TableRow, SlideError> {
        let limits = *self.limits;
        self.check_children(el, TableBound::CellsInTable, limits.max_cells_per_table)?;
        let mut e = el.clone();
        let height = self
            .r
            .required::<Written<Coordinate>>(&mut e, "h", coordinate)?;
        if height
            .as_ref()
            .is_some_and(|h| !(0..=OFFICE_ROW_HEIGHT_LIMIT).contains(&h.value().0))
        {
            self.note(TableFinding::Measure, el, Some("h"));
        }
        self.r.finish(&e);
        let mut row = TableRow {
            height,
            cells: Vec::new(),
        };
        for child in self.r.children(el)? {
            if !child.is(ns::DML, "tc") {
                // `extLst`, which holds `a16:rowId`, stays.
                self.r.retained(&child);
                continue;
            }
            *cells_here = cells_here.saturating_add(1);
            if *cells_here > limits.max_cells_per_table {
                return Err(self.over(TableBound::CellsInTable, limits.max_cells_per_table));
            }
            self.counts.cells = self.counts.cells.saturating_add(1);
            if self.counts.cells > limits.max_cells_per_part {
                return Err(self.over(TableBound::CellsInPart, limits.max_cells_per_part));
            }
            let column = row.cells.len();
            row.cells.push(self.tc(
                &child,
                rows.saturating_sub(at),
                columns.saturating_sub(column),
            )?);
        }
        if row.cells.len() != columns {
            self.note(TableFinding::CellCount, el, None);
        }
        Ok(row)
    }

    /// A span, which must stay in what is left of the table from here.
    fn span(&mut self, e: &mut Elem, name: &str, room: usize) -> Result<Option<i32>, SlideError> {
        let Some(value) = self.r.attr::<i32>(e, name)? else {
            return Ok(None);
        };
        let fits = usize::try_from(value).is_ok_and(|v| (1..=room).contains(&v));
        if !fits {
            self.note(TableFinding::SpanOutOfRange, e, Some(name));
            return Ok(None);
        }
        Ok(Some(value))
    }

    /// `a:CT_TableCell`. `rows` and `columns` are what is left of the table, this cell in.
    fn tc(&mut self, el: &Elem, rows: usize, columns: usize) -> Result<TableCell, SlideError> {
        let mut e = el.clone();
        let mut cell = TableCell {
            row_span: self.span(&mut e, "rowSpan", rows)?,
            grid_span: self.span(&mut e, "gridSpan", columns)?,
            h_merge: self.r.attr(&mut e, "hMerge")?,
            v_merge: self.r.attr(&mut e, "vMerge")?,
            id: self.r.attr(&mut e, "id")?,
            text: None,
            properties: None,
        };
        self.r.finish(&e);
        let mut seen = HashSet::new();
        for child in self.r.children(el)? {
            let name = child.local.as_str();
            let known = child.in_namespace(ns::DML) && matches!(name, "txBody" | "tcPr");
            if !known {
                self.r.retained(&child);
                continue;
            }
            if !seen.insert(name.to_owned()) {
                self.r.note(K::DuplicateElement, &child, None);
            } else if name == "txBody" {
                let mut text = TextReader {
                    r: &mut *self.r,
                    rels: self.rels,
                    color: &mut *self.color,
                    counts: &mut *self.text_counts,
                };
                cell.text = Some(Box::new(text.body(&child)?));
            } else {
                cell.properties = Some(Box::new(self.tc_pr(&child)?));
            }
        }
        Ok(cell)
    }

    fn tc_pr(&mut self, el: &Elem) -> Result<TableCellProperties, SlideError> {
        let mut e = el.clone();
        let r = &mut *self.r;
        let mut p = TableCellProperties {
            margin_left: r.attr_in(&mut e, "marL", coordinate32)?,
            margin_right: r.attr_in(&mut e, "marR", coordinate32)?,
            margin_top: r.attr_in(&mut e, "marT", coordinate32)?,
            margin_bottom: r.attr_in(&mut e, "marB", coordinate32)?,
            vertical: r.attr(&mut e, "vert")?,
            anchor: r.attr(&mut e, "anchor")?,
            anchor_center: r.attr(&mut e, "anchorCtr")?,
            horizontal_overflow: r.attr(&mut e, "horzOverflow")?,
            ..TableCellProperties::default()
        };
        r.finish(&e);
        let mut seen = HashSet::new();
        for child in self.r.children(el)? {
            if !child.in_namespace(ns::DML) {
                self.r.retained(&child);
                continue;
            }
            let fill = is_fill_element(&child);
            let name = if fill { "fill" } else { child.local.as_str() };
            let known = fill
                || matches!(
                    name,
                    "lnL" | "lnR" | "lnT" | "lnB" | "lnTlToBr" | "lnBlToTr" | "cell3D" | "headers"
                );
            if !known {
                // `extLst` and the unknown.
                self.r.retained(&child);
                continue;
            }
            if !seen.insert(name.to_owned()) {
                self.r.note(K::DuplicateElement, &child, None);
                continue;
            }
            let limits = DrawingLimits::default();
            match name {
                "fill" => p.fill = read_fill(self.r, self.rels, limits, &child)?,
                "cell3D" => p.cell3d = Some(self.cell3d(&child)?),
                "headers" => p.headers = Some(self.headers(&child)?),
                _ => {
                    let line = read_line(self.r, self.rels, limits, &child)?.map(Box::new);
                    match name {
                        "lnL" => p.line_left = line,
                        "lnR" => p.line_right = line,
                        "lnT" => p.line_top = line,
                        "lnB" => p.line_bottom = line,
                        "lnTlToBr" => p.line_top_left_to_bottom_right = line,
                        _ => p.line_bottom_left_to_top_right = line,
                    }
                }
            }
        }
        Ok(p)
    }

    fn headers(&mut self, el: &Elem) -> Result<Vec<String>, SlideError> {
        self.r.finish(el);
        let limit = self.limits.max_headers_per_cell;
        self.check_children(el, TableBound::Headers, limit)?;
        let mut out = Vec::new();
        for child in self.r.children(el)? {
            if !child.is(ns::DML, "header") {
                self.r.retained(&child);
                continue;
            }
            if out.len() >= limit {
                return Err(self.over(TableBound::Headers, limit));
            }
            self.r.finish(&child);
            if let Some(text) = self.r.text(&child)? {
                out.push(text);
            }
        }
        Ok(out)
    }

    /// `a:CT_Cell3D`, in a cell or a style.
    fn cell3d(&mut self, el: &Elem) -> Result<Cell3D, SlideError> {
        let mut e = el.clone();
        let mut out = Cell3D {
            material: self.r.attr(&mut e, "prstMaterial")?,
            ..Cell3D::default()
        };
        self.r.finish(&e);
        let mut seen = HashSet::new();
        for child in self.r.children(el)? {
            let name = child.local.as_str();
            if !child.in_namespace(ns::DML) || !matches!(name, "bevel" | "lightRig") {
                self.r.retained(&child);
                continue;
            }
            if !seen.insert(name.to_owned()) {
                self.r.note(K::DuplicateElement, &child, None);
                continue;
            }
            let mut ctx = Ctx::new(&mut *self.r, self.rels, DrawingLimits::default());
            if name == "bevel" {
                out.bevel = ctx.guard(&child, |c| c.bevel(&child))?;
            } else {
                out.light_rig = ctx.guard(&child, |c| c.light_rig(&child))?;
            }
        }
        if !seen.contains("bevel") {
            self.r.note(K::MissingRequired, el, Some("bevel"));
        }
        Ok(out)
    }

    fn tbl_pr(&mut self, el: &Elem) -> Result<TableProperties, SlideError> {
        let mut e = el.clone();
        let r = &mut *self.r;
        let mut p = TableProperties {
            rtl: r.attr(&mut e, "rtl")?,
            first_row: r.attr(&mut e, "firstRow")?,
            first_column: r.attr(&mut e, "firstCol")?,
            last_row: r.attr(&mut e, "lastRow")?,
            last_column: r.attr(&mut e, "lastCol")?,
            band_row: r.attr(&mut e, "bandRow")?,
            band_column: r.attr(&mut e, "bandCol")?,
            ..TableProperties::default()
        };
        r.finish(&e);
        let mut seen = HashSet::new();
        for child in self.r.children(el)? {
            if !child.in_namespace(ns::DML) {
                self.r.retained(&child);
                continue;
            }
            let slot = match child.local.as_str() {
                "effectLst" | "effectDag" => "effects",
                "tableStyle" | "tableStyleId" => "style",
                _ if is_fill_element(&child) => "fill",
                _ => {
                    self.r.retained(&child);
                    continue;
                }
            };
            if !seen.insert(slot) {
                self.r.note(K::DuplicateElement, &child, None);
                continue;
            }
            let limits = DrawingLimits::default();
            match slot {
                "fill" => p.fill = read_fill(self.r, self.rels, limits, &child)?,
                "effects" => p.effects = read_effects(self.r, self.rels, limits, &child)?,
                _ if child.local == "tableStyle" => {
                    p.style = Some(TableStyleChoice::Inline(Box::new(self.tbl_style(&child)?)));
                }
                _ => {
                    self.r.finish(&child);
                    match self.r.text(&child)? {
                        Some(text) if is_guid(&text) => p.style = Some(TableStyleChoice::Id(text)),
                        Some(_) => self.r.note(K::MalformedValue, &child, None),
                        None => {}
                    }
                }
            }
        }
        Ok(p)
    }

    /// `a:CT_TableStyle`, in the list or in a table.
    pub fn tbl_style(&mut self, el: &Elem) -> Result<TableStyle, SlideError> {
        let mut e = el.clone();
        let mut style = TableStyle {
            id: self
                .r
                .required(&mut e, "styleId", |s: &String| is_guid(s))?,
            name: self.r.required(&mut e, "styleName", |_| true)?,
            ..TableStyle::default()
        };
        self.r.finish(&e);
        let mut seen = HashSet::new();
        for child in self.r.children(el)? {
            if !child.in_namespace(ns::DML) {
                self.r.retained(&child);
                continue;
            }
            let name = child.local.clone();
            let first = seen.insert(name.clone());
            match name.as_str() {
                "tblBg" => {
                    if first {
                        style.background = Some(self.tbl_bg(&child)?);
                    } else {
                        self.r.note(K::DuplicateElement, &child, None);
                    }
                }
                "wholeTbl" | "band1H" | "band2H" | "band1V" | "band2V" | "lastCol" | "firstCol"
                | "lastRow" | "seCell" | "swCell" | "firstRow" | "neCell" | "nwCell" => {
                    if !first {
                        self.r.note(K::DuplicateElement, &child, None);
                        continue;
                    }
                    let value = Some(Box::new(self.part_style(&child)?));
                    if let Some(kind) = TablePartKind::from_xml(&name) {
                        *style.slot_mut(kind) = value;
                    }
                }
                // `extLst` and the unknown.
                _ => self.r.retained(&child),
            }
        }
        Ok(style)
    }

    fn tbl_bg(&mut self, el: &Elem) -> Result<TableBackgroundStyle, SlideError> {
        self.r.finish(el);
        let mut out = TableBackgroundStyle::default();
        let mut seen = HashSet::new();
        for child in self.r.children(el)? {
            if !child.in_namespace(ns::DML) {
                self.r.retained(&child);
                continue;
            }
            let slot = match child.local.as_str() {
                "fill" | "fillRef" => "fill",
                "effect" | "effectRef" => "effect",
                _ => {
                    self.r.retained(&child);
                    continue;
                }
            };
            if !seen.insert(slot) {
                self.r.note(K::DuplicateElement, &child, None);
            } else if slot == "fill" {
                out.fill = self.style_fill(&child)?;
            } else if child.local == "effectRef" {
                out.effect = Some(TableStyleEffect::Ref(read_style_matrix_ref(
                    self.r.tree,
                    child.id,
                    self.color,
                )));
                self.finish_reference(&child)?;
            } else {
                out.effect = self.style_effect(&child)?;
            }
        }
        Ok(out)
    }

    /// `fill` or `fillRef`.
    fn style_fill(&mut self, el: &Elem) -> Result<Option<TableStyleFill>, SlideError> {
        if el.local == "fillRef" {
            let found = read_style_matrix_ref(self.r.tree, el.id, self.color);
            self.finish_reference(el)?;
            return Ok(Some(TableStyleFill::Ref(found)));
        }
        self.r.finish(el);
        let mut out = None;
        let mut any = false;
        for child in self.r.children(el)? {
            if !is_fill_element(&child) {
                self.r.retained(&child);
            } else if any {
                self.r.note(K::DuplicateElement, &child, None);
            } else {
                any = true;
                out = read_fill(self.r, self.rels, DrawingLimits::default(), &child)?
                    .map(TableStyleFill::Inline);
            }
        }
        if !any {
            self.r.note(K::MissingRequired, el, Some("fill"));
        }
        Ok(out)
    }

    /// `effect`.
    fn style_effect(&mut self, el: &Elem) -> Result<Option<TableStyleEffect>, SlideError> {
        self.r.finish(el);
        let mut out = None;
        let mut any = false;
        for child in self.r.children(el)? {
            if !child.is(ns::DML, "effectLst") && !child.is(ns::DML, "effectDag") {
                self.r.retained(&child);
            } else if any {
                self.r.note(K::DuplicateElement, &child, None);
            } else {
                any = true;
                out = read_effects(self.r, self.rels, DrawingLimits::default(), &child)?
                    .map(TableStyleEffect::Inline);
            }
        }
        if !any {
            self.r.note(K::MissingRequired, el, Some("effect"));
        }
        Ok(out)
    }

    fn part_style(&mut self, el: &Elem) -> Result<TablePartStyle, SlideError> {
        self.r.finish(el);
        let mut out = TablePartStyle::default();
        let mut seen = HashSet::new();
        for child in self.r.children(el)? {
            let name = child.local.as_str();
            if !child.in_namespace(ns::DML) || !matches!(name, "tcTxStyle" | "tcStyle") {
                self.r.retained(&child);
                continue;
            }
            if !seen.insert(name.to_owned()) {
                self.r.note(K::DuplicateElement, &child, None);
            } else if name == "tcTxStyle" {
                out.text = Some(self.tc_tx_style(&child)?);
            } else {
                out.cell = Some(self.tc_style(&child)?);
            }
        }
        Ok(out)
    }

    /// What a style matrix or font reference holds beyond `@idx` and its colour, which
    /// the theme reader took and does not report.
    fn finish_reference(&mut self, el: &Elem) -> Result<(), SlideError> {
        let mut e = el.clone();
        e.take("idx");
        self.r.finish(&e);
        let tree = self.r.tree;
        for child in self.r.children(el)? {
            if !is_color_element(tree, child.id) {
                self.r.retained(&child);
            }
        }
        Ok(())
    }

    fn on_off(&mut self, e: &mut Elem, name: &str) -> Option<OnOffStyle> {
        let raw = e.take(name)?;
        let value = OnOffStyle::from_xml(raw.trim());
        if value.is_none() {
            self.r.note(K::MalformedValue, e, Some(name));
        }
        value
    }

    fn tc_tx_style(&mut self, el: &Elem) -> Result<TableTextStyle, SlideError> {
        let mut e = el.clone();
        let mut out = TableTextStyle {
            bold: self.on_off(&mut e, "b"),
            italic: self.on_off(&mut e, "i"),
            ..TableTextStyle::default()
        };
        self.r.finish(&e);
        let tree = self.r.tree;
        let mut seen = HashSet::new();
        for child in self.r.children(el)? {
            let font = child.is(ns::DML, "font") || child.is(ns::DML, "fontRef");
            let color = is_color_element(tree, child.id);
            if !font && !color {
                // `extLst` and the unknown.
                self.r.retained(&child);
                continue;
            }
            if !seen.insert(font) {
                self.r.note(K::DuplicateElement, &child, None);
            } else if font {
                // The first of either, which is this one.
                out.font = read_themeable_font(tree, el.id, self.color);
                if child.local == "fontRef" {
                    self.finish_reference(&child)?;
                } else {
                    self.r.finish(&child);
                }
            } else {
                let read: Option<Color> = read_color(tree, child.id, self.color);
                out.color = read;
            }
        }
        Ok(out)
    }

    fn tc_style(&mut self, el: &Elem) -> Result<TableCellStyle, SlideError> {
        self.r.finish(el);
        let mut out = TableCellStyle::default();
        let mut seen = HashSet::new();
        for child in self.r.children(el)? {
            if !child.in_namespace(ns::DML) {
                self.r.retained(&child);
                continue;
            }
            let slot = match child.local.as_str() {
                "tcBdr" => "tcBdr",
                "fill" | "fillRef" => "fill",
                "cell3D" => "cell3D",
                _ => {
                    self.r.retained(&child);
                    continue;
                }
            };
            if !seen.insert(slot) {
                self.r.note(K::DuplicateElement, &child, None);
                continue;
            }
            match slot {
                "tcBdr" => out.borders = Some(self.tc_bdr(&child)?),
                "fill" => out.fill = self.style_fill(&child)?,
                _ => out.cell3d = Some(self.cell3d(&child)?),
            }
        }
        Ok(out)
    }

    fn tc_bdr(&mut self, el: &Elem) -> Result<TableCellBorders, SlideError> {
        self.r.finish(el);
        let mut out = TableCellBorders::default();
        let mut seen = HashSet::new();
        for child in self.r.children(el)? {
            let name = child.local.clone();
            let known = child.in_namespace(ns::DML)
                && matches!(
                    name.as_str(),
                    "left" | "right" | "top" | "bottom" | "insideH" | "insideV" | "tl2br" | "tr2bl"
                );
            if !known {
                // `extLst` and the unknown.
                self.r.retained(&child);
                continue;
            }
            if !seen.insert(name.clone()) {
                self.r.note(K::DuplicateElement, &child, None);
                continue;
            }
            let border = Some(self.border(&child)?);
            match name.as_str() {
                "left" => out.left = border,
                "right" => out.right = border,
                "top" => out.top = border,
                "bottom" => out.bottom = border,
                "insideH" => out.inside_horizontal = border,
                "insideV" => out.inside_vertical = border,
                "tl2br" => out.top_left_to_bottom_right = border,
                _ => out.top_right_to_bottom_left = border,
            }
        }
        Ok(out)
    }

    /// `a:CT_ThemeableLineStyle`.
    fn border(&mut self, el: &Elem) -> Result<TableBorder, SlideError> {
        self.r.finish(el);
        let mut out = TableBorder::Empty;
        let mut any = false;
        for child in self.r.children(el)? {
            let known = child.is(ns::DML, "ln") || child.is(ns::DML, "lnRef");
            if !known {
                self.r.retained(&child);
            } else if any {
                self.r.note(K::DuplicateElement, &child, None);
            } else {
                any = true;
                if child.local == "ln" {
                    let line = read_line(self.r, self.rels, DrawingLimits::default(), &child)?;
                    if let Some(line) = line {
                        out = TableBorder::Line(Box::new(line));
                    }
                } else {
                    out =
                        TableBorder::Ref(read_style_matrix_ref(self.r.tree, child.id, self.color));
                    self.finish_reference(&child)?;
                }
            }
        }
        if !any {
            self.r.note(K::MissingRequired, el, Some("ln"));
        }
        Ok(out)
    }
}

/// Read a `tableStyles` part.
#[allow(clippy::too_many_arguments)]
pub fn import_table_styles(
    tree: &XmlTree,
    part: &str,
    rels: &[Relationship],
    limits: &SlideLimits,
    import_limits: &ImportLimits,
    findings: &mut Findings,
    color: &mut ColorFindings,
) -> Result<TableStyleList, SlideError> {
    let mut reader = Reader::new(tree, part, import_limits, findings);
    let root = reader.root(ns::DML, "tblStyleLst")?;
    let mut e = root.clone();
    let default_style = reader.required(&mut e, "def", |s: &String| is_guid(s))?;
    reader.finish(&e);
    let mut text_counts = TextCounts::default();
    let mut counts = TableCounts::default();
    let mut styles = TableReader {
        r: &mut reader,
        rels,
        limits: &limits.table,
        color,
        text_counts: &mut text_counts,
        counts: &mut counts,
    };
    let limit = limits.table.max_table_styles;
    styles.check_children(&root, TableBound::Styles, limit)?;
    let mut out = TableStyleList {
        default_style,
        styles: Vec::new(),
    };
    let mut ids = HashSet::new();
    for child in styles.r.children(&root)? {
        if !child.is(ns::DML, "tblStyle") {
            styles.r.retained(&child);
            continue;
        }
        if out.styles.len() >= limit {
            return Err(styles.over(TableBound::Styles, limit));
        }
        let style = styles.tbl_style(&child)?;
        if let Some(id) = &style.id
            && !ids.insert(id.to_ascii_uppercase())
        {
            styles.r.note(K::DuplicateId, &child, Some("styleId"));
        }
        out.styles.push(style);
    }
    if let Some(id) = &out.default_style
        && out.find(id).is_none()
    {
        styles
            .r
            .note(K::Table(TableFinding::UnknownStyle), &root, Some("def"));
    }
    Ok(out)
}

/// The deck's `tableStyles` part, followed from the main part. A part that is malformed
/// or wrongly rooted is reported as `CS-IMF-0012` and left out; a bound refuses.
pub fn read_deck_table_styles(
    package: &mut PresentationPackage,
    limits: &SlideLimits,
    import_limits: &ImportLimits,
    findings: &mut Findings,
    color: &mut ColorFindings,
) -> Result<Option<TableStyleList>, SlideError> {
    let main = package.main_part().to_owned();
    let PartLocation::At(part) = locate_part(package, &main, TABLE_STYLES_RELATIONSHIP, findings)
    else {
        return Ok(None);
    };
    let bytes = package
        .read_xml_part(&part)
        .map_err(crate::properties::ImportError::from)?;
    let xml = *package.limits();
    let rels = package.relationships_of(&part).to_vec();
    let result = XmlTree::parse(&bytes, &part, &xml)
        .map_err(SlideError::from)
        .and_then(|tree| {
            import_table_styles(&tree, &part, &rels, limits, import_limits, findings, color)
        });
    match result {
        Ok(list) => Ok(Some(list)),
        Err(err) if is_part_fault(&err) => {
            findings.note(K::UnreadablePart, &part, "", None);
            Ok(None)
        }
        Err(err) => Err(err),
    }
}

/// Report each `tableStyleId` that names nothing in the list, part by part. Never an error: an unknown GUID is a finding (BEH-29), and is what a
/// built-in style looks like too, as those have no definition in the file.
pub fn check_style_references(
    parts: &casual_slide_model::slide::SlideParts,
    list: Option<&TableStyleList>,
    findings: &mut Findings,
) {
    let unknown = K::Table(TableFinding::UnknownStyle);
    for (entry, part) in parts.structure.parts.iter().zip(&parts.parts) {
        let Some(tree) = part.as_ref().and_then(|p| p.shape_tree()) else {
            continue;
        };
        for node in tree.descendants() {
            let style = node
                .table
                .as_ref()
                .and_then(|t| t.properties.as_ref())
                .and_then(|p| p.style.as_ref());
            if let Some(TableStyleChoice::Id(id)) = style
                && list.is_none_or(|l| l.find(id).is_none())
            {
                findings.note(unknown, &entry.part, "tableStyleId", None);
            }
        }
    }
}
