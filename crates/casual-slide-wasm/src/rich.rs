//! Rich objects (`VS-11`): Insert > Table and Insert > Chart, a table's cells, rows,
//! columns and style, and a chart's data. Each edit is one operation of
//! `casual-slide-transaction` with its inverse, on the one undo history.
//!
//! A new table or chart frame is a small XML template read back by the reader, as a new
//! shape is (`VS-08`), so the model holds only what the reader would give. A new chart's
//! part is drawn from the model this session holds; at save the writer adds the chart
//! part and a minimal workbook built from its data (`casual_slide_export::new_chart`).
//! An edited chart of the file has its caches patched, and its workbook kept.

use casual_slide_export::new_chart::{
    ChartGrid, NewChartKind, chart_xml, read_grid, workbook, write_grid,
};
use casual_slide_export::writer::NewShape;
use casual_slide_import::chart::{ChartLimits, import_chart_part};
use casual_slide_import::drawing::DrawingLimits;
use casual_slide_import::properties::{Findings, ImportLimits};
use casual_slide_model::chart::ChartPart;
use casual_slide_model::shape::{ObjectId, ShapeNode};
use casual_slide_model::table::{Table, TableStyleChoice};
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage, XmlTree};
use casual_slide_transaction::Operation;
use casual_slide_transaction::rich;

use crate::build::{NAMESPACES, NEW_REL, TREE_HEAD, highest, read_slide};
use crate::design::DeckDoc;
use crate::{BridgeError, Viewer};

/// PowerPoint's default table style, Medium Style 2 - Accent 1, by its built-in id.
pub const DEFAULT_TABLE_STYLE: &str = "{5C22544A-7EE6-4342-B048-85BDC9FD1C3A}";

/// A new table row's height, EMU: PowerPoint's 0.4 inch.
const ROW_HEIGHT: i64 = 370_840;

/// Where an inserted chart is named within this session; never reaches the file.
const NEW_CHART: &str = "openpresent-new-chart/";

/// A row or column edit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableEdit {
    /// Insert a row at the place.
    InsertRow,
    /// Delete the row at the place.
    DeleteRow,
    /// Insert a column at the place.
    InsertColumn,
    /// Delete the column at the place.
    DeleteColumn,
}

impl TableEdit {
    /// From a bridge number: 0 to 3, in the order above.
    pub fn from_u32(n: u32) -> Option<Self> {
        Some(match n {
            0 => Self::InsertRow,
            1 => Self::DeleteRow,
            2 => Self::InsertColumn,
            3 => Self::DeleteColumn,
            _ => return None,
        })
    }
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn frame_head(id: u32, name: &str, b: [i64; 4]) -> String {
    let [x, y, cx, cy] = b;
    format!(
        "<p:nvGraphicFramePr><p:cNvPr id=\"{id}\" name=\"{}\"/><p:cNvGraphicFramePr>\
         <a:graphicFrameLocks noGrp=\"1\"/></p:cNvGraphicFramePr><p:nvPr/></p:nvGraphicFramePr>\
         <p:xfrm><a:off x=\"{x}\" y=\"{y}\"/><a:ext cx=\"{cx}\" cy=\"{cy}\"/></p:xfrm>",
        esc(name)
    )
}

fn chart_part_of(xml: &str, key: &str) -> Result<ChartPart, BridgeError> {
    let tree = XmlTree::parse(xml.as_bytes(), key, &OoxmlLimits::default())
        .map_err(|e| BridgeError::Build(e.code()))?;
    let import = ImportLimits::default();
    let mut findings = Findings::new(import.max_findings);
    let mut color = casual_slide_import::color::Findings::default();
    import_chart_part(
        &tree,
        key,
        &[],
        &ChartLimits::default(),
        DrawingLimits::default(),
        &import,
        &mut findings,
        &mut color,
    )
    .map_err(|e| BridgeError::Build(e.code()))
}

impl Viewer {
    fn frame(&self, index: usize, node: u32) -> Result<&ShapeNode, BridgeError> {
        let ordinal = self.slide_ordinal(index)?;
        self.deck
            .parts()
            .get(ordinal)
            .and_then(|p| {
                p.node(ObjectId {
                    part: ordinal,
                    node,
                })
            })
            .ok_or(BridgeError::BadArgument)
    }

    fn table(&self, index: usize, node: u32) -> Result<(ObjectId, &Table), BridgeError> {
        let frame = self.frame(index, node)?;
        let table = frame.table.as_deref().ok_or(BridgeError::BadArgument)?;
        Ok((frame.id, table))
    }

    fn set_table(&mut self, id: ObjectId, table: Table) -> Result<bool, BridgeError> {
        self.typing = None;
        let changed = self.history.apply(
            &mut DeckDoc(&mut self.deck),
            Operation::SetTable {
                node: id,
                table: Some(Box::new(table)),
            },
        )?;
        self.reports.clear();
        Ok(changed)
    }

    /// Put a new frame read from `inner` on slide `index`, on top: one undo step.
    fn insert_frame(
        &mut self,
        index: usize,
        base: &str,
        b: [i64; 4],
        inner: impl FnOnce(u32, &str, [i64; 4]) -> String,
    ) -> Result<u32, BridgeError> {
        let ordinal = self.slide_ordinal(index)?;
        let parts = self.deck.parts();
        let part = parts.get(ordinal).ok_or(BridgeError::BadArgument)?;
        let root = part.shape_tree().ok_or(BridgeError::BadArgument)?;
        let (top_spid, top_node) = highest(part);
        let id = top_spid.checked_add(1).ok_or(BridgeError::BadArgument)?;
        let name = format!("{base} {}", id.saturating_sub(1));
        let clamp = |v: i64| v.clamp(0, i64::from(i32::MAX));
        let b = [clamp(b[0]), clamp(b[1]), clamp(b[2]), clamp(b[3])];
        let xml = format!(
            "<p:sld {NAMESPACES}><p:cSld><p:spTree>{TREE_HEAD}{}</p:spTree></p:cSld></p:sld>",
            inner(id, &name, b)
        );
        let read = read_slide(xml.as_bytes(), "ppt/slides/openpresent-insert.xml", ordinal)?;
        let mut node = read
            .shape_tree()
            .and_then(|t| t.children.first())
            .cloned()
            .ok_or(BridgeError::Build("CS-WSM-0007"))?;
        let node_number = top_node.checked_add(1).ok_or(BridgeError::BadArgument)?;
        node.id = ObjectId {
            part: ordinal,
            node: node_number,
        };
        node.children.clear();
        let parent = root.id;
        let at = root.children.len();
        self.typing = None;
        self.history.apply(
            &mut DeckDoc(&mut self.deck),
            Operation::InsertNode {
                parent,
                index: at,
                node: Box::new(node),
            },
        )?;
        self.reports.remove(&index);
        Ok(node_number)
    }

    /// The deck's default table style: its `tableStyles/@def`, else PowerPoint's.
    fn default_table_style(&self) -> String {
        self.deck
            .table_styles()
            .and_then(|l| l.def.clone())
            .unwrap_or_else(|| DEFAULT_TABLE_STYLE.to_owned())
    }

    /// Insert > Table: `rows` by `columns` in this box (EMU), in the deck's default
    /// style with a header row and banded rows, each cell empty. Returns its node.
    pub fn insert_table(
        &mut self,
        index: usize,
        rows: usize,
        columns: usize,
        b: [i64; 4],
    ) -> Result<u32, BridgeError> {
        if rows == 0
            || columns == 0
            || rows > rich::MAX_TABLE_SIDE
            || columns > rich::MAX_TABLE_SIDE
        {
            return Err(BridgeError::BadArgument);
        }
        let style = self.default_table_style();
        let width = (b[2].max(0) / columns as i64).max(1);
        let height = b[3].max(0).max(ROW_HEIGHT.saturating_mul(rows as i64));
        let b = [b[0], b[1], width.saturating_mul(columns as i64), height];
        self.insert_frame(index, "Table", b, |id, name, b| {
            let grid: String = (0..columns)
                .map(|_| format!("<a:gridCol w=\"{width}\"/>"))
                .collect();
            let cell = "<a:tc><a:txBody><a:bodyPr/><a:lstStyle/><a:p>\
                        <a:endParaRPr lang=\"en-US\" dirty=\"0\"/></a:p></a:txBody><a:tcPr/></a:tc>";
            let row = format!("<a:tr h=\"{ROW_HEIGHT}\">{}</a:tr>", cell.repeat(columns));
            format!(
                "<p:graphicFrame>{}<a:graphic><a:graphicData \
                 uri=\"http://schemas.openxmlformats.org/drawingml/2006/table\"><a:tbl>\
                 <a:tblPr firstRow=\"1\" bandRow=\"1\"><a:tableStyleId>{}</a:tableStyleId></a:tblPr>\
                 <a:tblGrid>{grid}</a:tblGrid>{}</a:tbl></a:graphicData></a:graphic></p:graphicFrame>",
                frame_head(id, name, b),
                esc(&style),
                row.repeat(rows)
            )
        })
    }

    /// A table's shape for the editor: rows, columns, then each column's width and each
    /// row's height, EMU (0 where the file states none).
    pub fn table_shape(&self, index: usize, node: u32) -> Result<Vec<i64>, BridgeError> {
        let (_, t) = self.table(index, node)?;
        let grid = t.tbl_grid.as_ref().map_or(&[][..], |g| &g.grid_col[..]);
        let mut out = vec![t.rows.len() as i64, grid.len() as i64];
        out.extend(
            grid.iter()
                .map(|c| c.width.as_ref().map_or(0, |w| w.value().0)),
        );
        out.extend(
            t.rows
                .iter()
                .map(|r| r.height.as_ref().map_or(0, |h| h.value().0)),
        );
        Ok(out)
    }

    /// The plain text of a cell.
    pub fn table_cell_text(
        &self,
        index: usize,
        node: u32,
        row: usize,
        column: usize,
    ) -> Result<String, BridgeError> {
        let (_, t) = self.table(index, node)?;
        rich::cell_text(t, row, column).ok_or(BridgeError::BadArgument)
    }

    /// Type a cell's text: one undo step. Returns whether anything changed.
    pub fn set_table_cell_text(
        &mut self,
        index: usize,
        node: u32,
        row: usize,
        column: usize,
        text: &str,
    ) -> Result<bool, BridgeError> {
        let (id, t) = self.table(index, node)?;
        let edited = rich::set_cell_text(t, row, column, text)?;
        self.set_table(id, edited)
    }

    /// Insert or delete a row or column at `at`: one undo step.
    pub fn edit_table(
        &mut self,
        index: usize,
        node: u32,
        edit: TableEdit,
        at: usize,
    ) -> Result<bool, BridgeError> {
        let (id, t) = self.table(index, node)?;
        let edited = match edit {
            TableEdit::InsertRow => rich::insert_row(t, at)?,
            TableEdit::DeleteRow => rich::delete_row(t, at)?,
            TableEdit::InsertColumn => rich::insert_column(t, at)?,
            TableEdit::DeleteColumn => rich::delete_column(t, at)?,
        };
        self.set_table(id, edited)
    }

    /// Table Design's gallery: `(id, name)` for each style the deck's `tableStyles`
    /// holds, then PowerPoint's default when the deck does not hold it.
    pub fn table_styles(&self) -> Vec<(String, String)> {
        let mut out: Vec<(String, String)> = self
            .deck
            .table_styles()
            .map(|l| {
                l.tbl_style
                    .iter()
                    .filter_map(|s| {
                        let id = s.style_id.clone()?;
                        let name = s.style_name.clone().unwrap_or_else(|| id.clone());
                        Some((id, name))
                    })
                    .collect()
            })
            .unwrap_or_default();
        if !out.iter().any(|(id, _)| id == DEFAULT_TABLE_STYLE) {
            out.push((
                DEFAULT_TABLE_STYLE.to_owned(),
                "Medium Style 2 - Accent 1".to_owned(),
            ));
        }
        out
    }

    /// The style id a table names, or empty.
    pub fn table_style(&self, index: usize, node: u32) -> Result<String, BridgeError> {
        let (_, t) = self.table(index, node)?;
        Ok(match t.tbl_pr.as_ref().and_then(|p| p.style.as_ref()) {
            Some(TableStyleChoice::Id(id)) => id.clone(),
            _ => String::new(),
        })
    }

    /// Table Design: give a table a style by id: one undo step.
    pub fn set_table_style(
        &mut self,
        index: usize,
        node: u32,
        style: &str,
    ) -> Result<bool, BridgeError> {
        let valid = style.len() == 38
            && style.starts_with('{')
            && style.ends_with('}')
            && style
                .chars()
                .all(|c| c.is_ascii_hexdigit() || matches!(c, '{' | '}' | '-'));
        if !valid {
            return Err(BridgeError::BadArgument);
        }
        let (id, t) = self.table(index, node)?;
        let edited = rich::set_style(t, Some(style));
        self.set_table(id, edited)
    }

    /// Insert > Chart: a chart of this kind with PowerPoint's sample data, in this box.
    /// Returns its node.
    pub fn insert_chart(
        &mut self,
        index: usize,
        kind: NewChartKind,
        b: [i64; 4],
    ) -> Result<u32, BridgeError> {
        let ordinal = self.slide_ordinal(index)?;
        let grid = ChartGrid::sample(kind);
        self.minted = self.minted.saturating_add(1);
        let rel = format!("{NEW_REL}{}", self.minted);
        let key = format!("{NEW_CHART}{}", self.minted);
        let part = chart_part_of(&chart_xml(&grid, None), &key)?;
        self.deck.add_chart(ordinal, &rel, &key, part);
        self.insert_frame(index, "Chart", b, |id, name, b| {
            format!(
                "<p:graphicFrame>{}<a:graphic><a:graphicData \
                 uri=\"http://schemas.openxmlformats.org/drawingml/2006/chart\">\
                 <c:chart xmlns:c=\"http://schemas.openxmlformats.org/drawingml/2006/chart\" \
                 r:id=\"{rel}\"/></a:graphicData></a:graphic></p:graphicFrame>",
                frame_head(id, name, b)
            )
        })
    }

    /// The chart part a frame names.
    fn chart_key(&self, index: usize, node: u32) -> Result<String, BridgeError> {
        let frame = self.frame(index, node)?;
        let rel = frame
            .chart
            .as_ref()
            .and_then(|c| c.rel_id.as_deref())
            .ok_or(BridgeError::BadArgument)?;
        self.deck
            .related_part(frame.id.part, rel)
            .ok_or(BridgeError::BadArgument)
    }

    /// A chart's data, for the data sheet. Refused for a chart other than column, bar,
    /// line or pie.
    pub fn chart_grid(&self, index: usize, node: u32) -> Result<ChartGrid, BridgeError> {
        let key = self.chart_key(index, node)?;
        let part = self.deck.chart(&key).ok_or(BridgeError::BadArgument)?;
        read_grid(part).ok_or(BridgeError::BadArgument)
    }

    /// Change a chart's data: one undo step. A chart inserted in this session takes any
    /// shape of grid; a chart of the file keeps its number of series and categories.
    pub fn set_chart_grid(
        &mut self,
        index: usize,
        node: u32,
        grid: ChartGrid,
    ) -> Result<bool, BridgeError> {
        let key = self.chart_key(index, node)?;
        let part = self.deck.chart(&key).ok_or(BridgeError::BadArgument)?;
        let current = read_grid(part).ok_or(BridgeError::BadArgument)?;
        let grid = ChartGrid {
            kind: current.kind,
            ..grid
        }
        .normalized();
        let new = if key.starts_with(NEW_CHART) {
            chart_part_of(&chart_xml(&grid, None), &key)?
        } else {
            write_grid(part, &grid).ok_or(BridgeError::BadArgument)?
        };
        self.typing = None;
        let changed = self.history.apply(
            &mut DeckDoc(&mut self.deck),
            Operation::SetChart {
                part: key.clone(),
                chart: Box::new(new),
            },
        )?;
        if !key.starts_with(NEW_CHART) {
            self.edited_charts.insert(key);
        }
        self.reports.clear();
        Ok(changed)
    }

    /// What the writer inserts for a frame new in this session.
    pub(crate) fn new_frame(&self, node: &ShapeNode) -> Result<NewShape, BridgeError> {
        if let Some(t) = node.table.as_deref() {
            let columns = t
                .tbl_grid
                .as_ref()
                .map(|g| {
                    g.grid_col
                        .iter()
                        .map(|c| c.width.as_ref().map_or(0, |w| w.value().0))
                        .collect()
                })
                .unwrap_or_default();
            let rows = t
                .rows
                .iter()
                .map(|r| r.height.as_ref().map_or(0, |h| h.value().0))
                .collect();
            return Ok(NewShape::Table { columns, rows });
        }
        let rel = node
            .chart
            .as_ref()
            .and_then(|c| c.rel_id.as_deref())
            .filter(|r| r.starts_with(NEW_REL))
            .ok_or(BridgeError::Save("CS-WSM-0009"))?;
        let key = self
            .deck
            .related_part(node.id.part, rel)
            .ok_or(BridgeError::Save("CS-WSM-0009"))?;
        let grid = self
            .deck
            .chart(&key)
            .and_then(read_grid)
            .ok_or(BridgeError::Save("CS-WSM-0009"))?;
        let book = workbook(&grid).map_err(|e| BridgeError::Save(e.code()))?;
        Ok(NewShape::Chart {
            chart: chart_xml(&grid, Some(NewShape::WORKBOOK_REL)),
            workbook: book,
        })
    }

    /// The file's charts whose data was edited, patched in place; their workbooks kept.
    pub(crate) fn chart_pass(&self, bytes: Vec<u8>) -> Result<Vec<u8>, BridgeError> {
        let edited: Vec<(String, ChartPart)> = self
            .edited_charts
            .iter()
            .filter_map(|k| Some((k.clone(), self.deck.chart(k)?.clone())))
            .collect();
        if edited.is_empty() {
            return Ok(bytes);
        }
        let mut package = PresentationPackage::open(bytes.clone(), OoxmlLimits::default())
            .map_err(|e| BridgeError::Save(e.code()))?;
        let outcome = casual_slide_export::chart::save_chart_parts(
            &mut package,
            &edited,
            &ChartLimits::default(),
            &ImportLimits::default(),
        )
        .map_err(|e| BridgeError::Save(e.code()))?;
        Ok(outcome.bytes)
    }
}
