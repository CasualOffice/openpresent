//! Rich objects (`VS-11`): a frame's table and a chart's part. Each operation sets a whole
//! value and returns the one it replaced, so undo gives back the very model that was read
//! and the writer, which patches what differs (ADR-013), then patches nothing.
//!
//! Table edits (a cell's text, a row or column added or deleted, a style) are built as a
//! new [`Table`] by the pure functions here and applied as one [`Operation::SetTable`].
//! A table with merged cells is refused for row and column edits: what a merge covers
//! moves with it, which this thin pass does not do.

use casual_slide_model::table::{Table, TableCell, TableProperties, TableStyleChoice};
use casual_slide_model::text::{Paragraph, TextItem, TextRun};

use crate::{Applied, Document, Operation, TxnError};

/// The most rows or columns an edit makes: PowerPoint's own table bound.
pub const MAX_TABLE_SIDE: usize = 75;

/// Apply a rich-object operation; `None` for any other, which the caller applies.
pub(crate) fn apply_rich<D: Document + ?Sized>(
    doc: &mut D,
    op: &Operation,
) -> Result<Option<Applied>, TxnError> {
    let applied = match op {
        Operation::SetTable { node, table } => {
            let node = *node;
            let shape = doc
                .parts_mut()
                .get_mut(node.part)
                .and_then(|p| p.node_mut(node))
                .ok_or(TxnError::NoSuchObject(node))?;
            if shape.table.is_none() {
                return Err(TxnError::NoTable(node));
            }
            let changed = shape.table != *table;
            let before = core::mem::replace(&mut shape.table, table.clone());
            (
                Operation::SetTable {
                    node,
                    table: before,
                },
                changed,
            )
        }
        Operation::SetChart { part, chart } => {
            let slot = doc
                .chart_mut(part)
                .ok_or_else(|| TxnError::NoChart(part.clone()))?;
            let changed = *slot != **chart;
            let before = core::mem::replace(slot, (**chart).clone());
            (
                Operation::SetChart {
                    part: part.clone(),
                    chart: Box::new(before),
                },
                changed,
            )
        }
        _ => return Ok(None),
    };
    Ok(Some(applied))
}

fn has_merges(t: &Table) -> bool {
    t.rows.iter().flat_map(|r| &r.cells).any(|c| {
        c.grid_span.is_some_and(|s| s > 1)
            || c.row_span.is_some_and(|s| s > 1)
            || c.h_merge == Some(true)
            || c.v_merge == Some(true)
    })
}

/// A cell like `like`, its text gone: its paragraph and run properties kept, so what is
/// typed into it looks as its neighbour's would.
fn emptied(like: &TableCell) -> TableCell {
    let mut cell = like.clone();
    cell.id = None;
    cell.row_span = None;
    cell.grid_span = None;
    cell.h_merge = None;
    cell.v_merge = None;
    if let Some(body) = cell.tx_body.as_mut() {
        let first = body.paragraphs.first().cloned().unwrap_or_default();
        body.paragraphs = vec![Paragraph {
            items: Vec::new(),
            end_properties: first
                .end_properties
                .clone()
                .or_else(|| run_properties(&first)),
            ..first
        }];
    }
    cell
}

fn run_properties(p: &Paragraph) -> Option<Box<casual_slide_model::text::CharacterProperties>> {
    p.items.iter().find_map(|i| match i {
        TextItem::Run(r) => r.properties.clone(),
        _ => None,
    })
}

/// Insert an empty row at `at`, copied in look from the row beside it.
pub fn insert_row(t: &Table, at: usize) -> Result<Table, TxnError> {
    if has_merges(t) || at > t.rows.len() || t.rows.len() >= MAX_TABLE_SIDE {
        return Err(TxnError::TableShape);
    }
    let like = t
        .rows
        .get(at)
        .or_else(|| t.rows.get(at.wrapping_sub(1)))
        .ok_or(TxnError::TableShape)?;
    let mut row = like.clone();
    row.cells = like.cells.iter().map(emptied).collect();
    let mut out = t.clone();
    out.rows.insert(at, row);
    Ok(out)
}

/// Delete row `at`. The last row is not deleted: the table would be left with none.
pub fn delete_row(t: &Table, at: usize) -> Result<Table, TxnError> {
    if has_merges(t) || at >= t.rows.len() || t.rows.len() < 2 {
        return Err(TxnError::TableShape);
    }
    let mut out = t.clone();
    out.rows.remove(at);
    Ok(out)
}

/// Insert an empty column at `at`. The table keeps its width, as PowerPoint's Insert Left
/// and Right do not: the new column takes its neighbour's width, so the table grows.
pub fn insert_column(t: &Table, at: usize) -> Result<Table, TxnError> {
    let grid = t.tbl_grid.as_ref().ok_or(TxnError::TableShape)?;
    let columns = grid.grid_col.len();
    if has_merges(t) || at > columns || columns >= MAX_TABLE_SIDE {
        return Err(TxnError::TableShape);
    }
    if t.rows.iter().any(|r| r.cells.len() != columns) {
        return Err(TxnError::TableShape);
    }
    let near = |len: usize| if at < len { at } else { len.saturating_sub(1) };
    let mut out = t.clone();
    let col = grid
        .grid_col
        .get(near(columns))
        .cloned()
        .ok_or(TxnError::TableShape)?;
    if let Some(g) = out.tbl_grid.as_mut() {
        g.grid_col.insert(at, col);
    }
    for row in &mut out.rows {
        let cell = row
            .cells
            .get(near(columns))
            .map(emptied)
            .ok_or(TxnError::TableShape)?;
        row.cells.insert(at, cell);
    }
    Ok(out)
}

/// Delete column `at`. The last column is not deleted.
pub fn delete_column(t: &Table, at: usize) -> Result<Table, TxnError> {
    let grid = t.tbl_grid.as_ref().ok_or(TxnError::TableShape)?;
    let columns = grid.grid_col.len();
    if has_merges(t) || at >= columns || columns < 2 {
        return Err(TxnError::TableShape);
    }
    if t.rows.iter().any(|r| r.cells.len() != columns) {
        return Err(TxnError::TableShape);
    }
    let mut out = t.clone();
    if let Some(g) = out.tbl_grid.as_mut() {
        g.grid_col.remove(at);
    }
    for row in &mut out.rows {
        row.cells.remove(at);
    }
    Ok(out)
}

/// The plain text of a cell: paragraphs joined by line feeds.
pub fn cell_text(t: &Table, row: usize, column: usize) -> Option<String> {
    let cell = t.rows.get(row)?.cells.get(column)?;
    let Some(body) = cell.tx_body.as_ref() else {
        return Some(String::new());
    };
    Some(
        body.paragraphs
            .iter()
            .map(|p| {
                p.items
                    .iter()
                    .map(|i| match i {
                        TextItem::Run(r) => r.text.clone().unwrap_or_default(),
                        TextItem::Break(_) => "\u{B}".to_owned(),
                        TextItem::Field(f) => f.text.clone().unwrap_or_default(),
                        TextItem::Math(_) | TextItem::Retained(_) => String::new(),
                    })
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

/// Set a cell's text: one paragraph a line, each one run in the look of the cell's first
/// run. Unchanged text leaves the cell as it was, run for run.
pub fn set_cell_text(t: &Table, row: usize, column: usize, text: &str) -> Result<Table, TxnError> {
    if cell_text(t, row, column).ok_or(TxnError::TableShape)? == text {
        return Ok(t.clone());
    }
    let mut out = t.clone();
    let cell = out
        .rows
        .get_mut(row)
        .and_then(|r| r.cells.get_mut(column))
        .ok_or(TxnError::TableShape)?;
    let body = cell.tx_body.get_or_insert_with(|| {
        Box::new(casual_slide_model::text::TextBody {
            properties: Some(Default::default()),
            list_style: Some(Default::default()),
            paragraphs: Vec::new(),
        })
    });
    let first = body.paragraphs.first().cloned().unwrap_or_default();
    let look = run_properties(&first).or_else(|| first.end_properties.clone());
    body.paragraphs = text
        .split('\n')
        .map(|line| {
            let clean: String = line
                .chars()
                .filter(|c| !c.is_control() || *c == '\t')
                .collect();
            Paragraph {
                properties: first.properties.clone(),
                items: if clean.is_empty() {
                    Vec::new()
                } else {
                    vec![TextItem::Run(TextRun {
                        properties: look.clone(),
                        text: Some(clean),
                    })]
                },
                end_properties: first.end_properties.clone().or_else(|| look.clone()),
            }
        })
        .collect();
    Ok(out)
}

/// Give the table a style by id (Table Design's gallery), or none.
pub fn set_style(t: &Table, style_id: Option<&str>) -> Table {
    let mut out = t.clone();
    let props = out
        .tbl_pr
        .get_or_insert_with(|| Box::new(TableProperties::default()));
    props.style = style_id.map(|id| TableStyleChoice::Id(id.to_owned()));
    out
}
