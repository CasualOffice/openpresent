//! A table's graphic frame to display items: the grid, merged cells, cell fills and
//! borders from the cell's own properties over its table style's parts, and each
//! cell's text laid out by `casual-slide-text` in the cell less its margins.
//!
//! The style's parts apply in the order `MS-OI29500` 2.1.1265 gives, later over
//! earlier, each only where the table's flags (`firstRow`, `bandRow` and so on) turn
//! it on. A row grows to fit its text, as PowerPoint draws it; a stated height is the
//! least it is.

use casual_slide_geometry::TextRect;
use casual_slide_model::color::{Color, ColorTransform, ColorValue, SchemeColor, TransformKind};
use casual_slide_model::fill::Fill;
use casual_slide_model::line::Line;
use casual_slide_model::presentation::Coordinate;
use casual_slide_model::shape::ShapeNode;
use casual_slide_model::table::{
    Table, TableBorder, TableCell, TableCellBorderStyle, TablePartKind, TablePartStyle, TableStyle,
    TableStyleCellStyle, TableStyleChoice, TableStyleFill, TableStyleTextStyle,
};
use casual_slide_model::text::{
    BodyProperties, CharacterProperties, TextAnchor, TextBody, TextItem, TextWrap, Written,
};
use casual_slide_model::theme::OnOffStyle;
use casual_slide_text::{Frame, TextEnv, layout_text_body};

use super::{Builder, EMU_PER_POINT, placement, rect_path};
use crate::display::{Affine, Layer, Paint, PathCmd};

/// The most grid columns or rows drawn of one table. Import bounds a table already;
/// this keeps the renderer's own work bounded on its own terms.
const MAX_TABLE_CELLS: usize = 100_000;

/// The cell margins the schema gives when `tcPr` states none: 0.1" left and right,
/// 0.05" top and bottom.
const DEFAULT_MARGIN_LR: i64 = 91_440;
const DEFAULT_MARGIN_TB: i64 = 45_720;

/// One cell's place in the grid, after merges.
#[derive(Debug, Clone, Copy)]
struct Placed {
    row: usize,
    col: usize,
    rows: usize,
    cols: usize,
}

/// Which side of a cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side {
    Left,
    Right,
    Top,
    Bottom,
}

impl<'a> Builder<'a> {
    /// Draw a table frame.
    pub(super) fn table(
        &mut self,
        node: &'a ShapeNode,
        table: &'a Table,
        layer: Layer,
        parent: &Affine,
    ) {
        let Some(t) = self.chain.transform(node).map(|t| t.value) else {
            self.report.note("table (no placement)");
            return;
        };
        let transform = parent.then_after(&placement(
            t.offset.x.0 as f64,
            t.offset.y.0 as f64,
            t.extent.cx as f64,
            t.extent.cy as f64,
            t.rotation,
            t.flip_h,
            t.flip_v,
        ));
        let cols: Vec<f64> = table
            .tbl_grid
            .iter()
            .flat_map(|g| g.grid_col.iter())
            .map(|c| c.width.as_ref().map_or(0.0, |w| w.value().0.max(0) as f64))
            .collect();
        let nrows = table.rows.len();
        let ncols = cols.len();
        if ncols == 0 || nrows == 0 {
            return;
        }
        if nrows.saturating_mul(ncols) > MAX_TABLE_CELLS {
            self.report.note("table (over the cell budget)");
            return;
        }
        let props = table.tbl_pr.as_deref();
        let flag = |f: fn(&casual_slide_model::table::TableProperties) -> Option<bool>| {
            props.and_then(f).unwrap_or(false)
        };
        let flags = Flags {
            first_row: flag(|p| p.first_row),
            last_row: flag(|p| p.last_row),
            first_col: flag(|p| p.first_col),
            last_col: flag(|p| p.last_col),
            band_row: flag(|p| p.band_row),
            band_col: flag(|p| p.band_col),
            rows: nrows,
            cols: ncols,
        };
        // A built-in style the deck names but does not carry, made here.
        let builtin: Option<TableStyle>;
        let style: Option<&TableStyle> = match props.and_then(|p| p.style.as_ref()) {
            Some(TableStyleChoice::Inline(s)) => Some(s),
            Some(TableStyleChoice::Id(id)) => {
                match self.scene.table_styles.and_then(|l| l.find(id)) {
                    Some(found) => Some(found),
                    None => {
                        builtin = builtin_style(id);
                        if builtin.is_none() {
                            self.report.note("table style (not in the deck)");
                        }
                        builtin.as_ref()
                    }
                }
            }
            None => None,
        };
        if props.is_some_and(|p| p.effects.is_some()) {
            self.report.note("effects");
        }

        // Where each cell lands: merges by their origin, covered cells skipped.
        let merges = table.merges();
        let mut cells: Vec<(Placed, &'a TableCell)> = Vec::new();
        for (r, row) in table.rows.iter().enumerate() {
            for (c, cell) in row.cells.iter().enumerate().take(ncols) {
                if merges.origins.contains_key(&(r, c)) {
                    continue;
                }
                let (rows, cols_spanned) = merges
                    .merges
                    .iter()
                    .find(|m| m.row == r && m.column == c)
                    .map_or((1, 1), |m| (m.rows, m.columns));
                cells.push((
                    Placed {
                        row: r,
                        col: c,
                        rows: rows.clamp(1, nrows - r),
                        cols: cols_spanned.clamp(1, ncols - c),
                    },
                    cell,
                ));
            }
        }

        // Column edges, then row heights: the stated height, grown to fit the text
        // of each cell that spans that row alone.
        let mut xs = Vec::with_capacity(ncols + 1);
        xs.push(0.0);
        for w in &cols {
            xs.push(xs.last().copied().unwrap_or(0.0) + w);
        }
        let mut heights: Vec<f64> = table
            .rows
            .iter()
            .map(|r| r.height.as_ref().map_or(0.0, |h| h.value().0.max(0) as f64))
            .collect();
        let bodies: Vec<Option<TextBody>> = cells
            .iter()
            .map(|(p, cell)| self.cell_body(cell, style, &flags, *p))
            .collect();
        for ((p, cell), body) in cells.iter().zip(&bodies) {
            if p.rows != 1 {
                continue;
            }
            let Some(body) = body else { continue };
            let width = xs[p.col + p.cols] - xs[p.col];
            let (ml, mr, mt, mb) = margins(cell);
            let need = self.measure(node, body, width - ml - mr) + mt + mb;
            if let Some(h) = heights.get_mut(p.row)
                && need > *h
            {
                *h = need;
            }
        }
        if layer == Layer::Slide {
            self.text.table_rows.insert(node.id, heights.clone());
        }
        let mut ys = Vec::with_capacity(nrows + 1);
        ys.push(0.0);
        for h in &heights {
            ys.push(ys.last().copied().unwrap_or(0.0) + h);
        }
        let (tw, th) = (xs[ncols], ys[nrows]);

        // The table background, under every cell.
        if let Some(fill) = style
            .and_then(|s| s.tbl_bg.as_ref())
            .and_then(|b| b.fill.as_ref())
            && let Some(paint) = self.style_fill_paint(fill, tw, th, node.id.part)
        {
            self.push(node, layer, transform, rect_path(tw, th), Some(paint), None);
        }
        if let Some(fill) = props.and_then(|p| p.fill.as_ref())
            && let Some(paint) = self.fill_paint(fill, None, tw, th, node.id.part, None)
        {
            self.push(node, layer, transform, rect_path(tw, th), Some(paint), None);
        }

        // Fills, then borders over every fill, then text over both.
        let boxes: Vec<(f64, f64, f64, f64)> = cells
            .iter()
            .map(|(p, _)| {
                let (x0, y0) = (xs[p.col], ys[p.row]);
                (x0, y0, xs[p.col + p.cols] - x0, ys[p.row + p.rows] - y0)
            })
            .collect();
        for ((p, cell), &(x, y, w, h)) in cells.iter().zip(&boxes) {
            let own = cell.tc_pr.as_deref().and_then(|t| t.fill.as_ref());
            let paint = match own {
                Some(f) => self.fill_paint(f, None, w, h, node.id.part, None),
                None => {
                    let fill = parts_for(style, &flags, *p)
                        .into_iter()
                        .rev()
                        .find_map(|(part, _)| part.tc_style.as_ref()?.fill.as_ref());
                    match fill {
                        Some(f) => self.style_fill_paint(f, w, h, node.id.part),
                        None => None,
                    }
                }
            };
            if let Some(paint) = paint {
                let at = transform.then_after(&Affine::translate(x, y));
                self.push(node, layer, at, rect_path(w, h), Some(paint), None);
            }
        }
        for ((p, cell), &(x, y, w, h)) in cells.iter().zip(&boxes) {
            let at = transform.then_after(&Affine::translate(x, y));
            let own = cell.tc_pr.as_deref();
            for (side, from, to) in [
                (Side::Top, (0.0, 0.0), (w, 0.0)),
                (Side::Bottom, (0.0, h), (w, h)),
                (Side::Left, (0.0, 0.0), (0.0, h)),
                (Side::Right, (w, 0.0), (w, h)),
            ] {
                let own_line = own.and_then(|t| match side {
                    Side::Top => t.ln_t.as_deref(),
                    Side::Bottom => t.ln_b.as_deref(),
                    Side::Left => t.ln_l.as_deref(),
                    Side::Right => t.ln_r.as_deref(),
                });
                let border = match own_line {
                    Some(l) => Some(Border::Line(l)),
                    None => style_border(style, &flags, *p, side),
                };
                self.border(node, layer, at, border, from, to);
            }
            let diagonals = [
                (
                    own.and_then(|t| t.ln_tl_to_br.as_deref()),
                    (0.0, 0.0),
                    (w, h),
                ),
                (
                    own.and_then(|t| t.ln_bl_to_tr.as_deref()),
                    (0.0, h),
                    (w, 0.0),
                ),
            ];
            for (line, from, to) in diagonals {
                if let Some(l) = line {
                    self.border(node, layer, at, Some(Border::Line(l)), from, to);
                }
            }
        }
        for (((_, cell), body), &(x, y, w, h)) in cells.iter().zip(&bodies).zip(&boxes) {
            let Some(body) = body else { continue };
            let at = transform.then_after(&Affine::translate(x, y));
            let rect = TextRect {
                l: 0.0,
                t: 0.0,
                r: w,
                b: h,
            };
            let _ = cell;
            self.text_in(node, body, layer, at, rect, None);
        }
    }

    /// A table style's fill, which may be a reference into the theme.
    fn style_fill_paint(
        &mut self,
        fill: &TableStyleFill,
        w: f64,
        h: f64,
        ordinal: u32,
    ) -> Option<Paint> {
        match fill {
            TableStyleFill::Inline(f) => self.fill_paint(f, None, w, h, ordinal, None),
            TableStyleFill::Ref(r) => self.style_fill(r, w, h, Some(ordinal)),
        }
    }

    /// One border segment, in the cell's space.
    fn border(
        &mut self,
        node: &ShapeNode,
        layer: Layer,
        at: Affine,
        border: Option<Border<'_>>,
        from: (f64, f64),
        to: (f64, f64),
    ) {
        let (line, ph): (Line, Option<Color>) = match border {
            Some(Border::Line(l)) => (l.clone(), None),
            Some(Border::Ref(idx, color)) => {
                let Some(l) = self.format().and_then(|f| f.line(idx)) else {
                    return;
                };
                (l.clone(), color.cloned())
            }
            None => return,
        };
        // No fill stated on a cell border is no line (`a:ln` in `tcPr` with nothing).
        if line.fill.is_none() || matches!(line.fill, Some(Fill::None)) {
            return;
        }
        let (w, h) = ((to.0 - from.0).abs(), (to.1 - from.1).abs());
        let Some((stroke, _)) = self.line_stroke(&line, ph.as_ref(), w, h, node.id.part, None)
        else {
            return;
        };
        let path = vec![PathCmd::MoveTo(from.0, from.1), PathCmd::LineTo(to.0, to.1)];
        self.push(node, layer, at, path, None, Some(stroke));
    }

    /// The text height of `body` laid out `width` wide, EMU, without insets.
    fn measure(&mut self, node: &'a ShapeNode, body: &TextBody, width: f64) -> f64 {
        if width <= 0.0 {
            return 0.0;
        }
        let object_defaults = self.scene.theme.and_then(|t| t.object_defaults.as_ref());
        let chain = self
            .chain
            .text_chain(node, self.scene.default_text_style, object_defaults);
        let env = TextEnv {
            colors: self.text_colors(None),
            fonts: self.font_scheme(),
        };
        // Insets are zero here: the caller adds the cell's margins.
        let mut body = body.clone();
        if let Some(p) = body.properties.as_mut() {
            for inset in [
                &mut p.inset_left,
                &mut p.inset_right,
                &mut p.inset_top,
                &mut p.inset_bottom,
            ] {
                *inset = Some(Written::new(Coordinate(0)));
            }
        }
        let frame = Frame {
            x: 0.0,
            y: 0.0,
            width: (width / EMU_PER_POINT) as f32,
            height: 1.0e6,
        };
        let layout = layout_text_body(&body, &chain, &env, frame);
        f64::from(layout.content_height) * EMU_PER_POINT
    }

    /// A cell's text body as it is laid out: its margins and anchor moved into the
    /// body's properties, and the table style's text colour and weight given to
    /// every run that states none. A copy for drawing; the model is not touched.
    fn cell_body(
        &self,
        cell: &TableCell,
        style: Option<&TableStyle>,
        flags: &Flags,
        at: Placed,
    ) -> Option<TextBody> {
        let mut body = cell.tx_body.as_deref()?.clone();
        if !body.paragraphs.iter().any(|p| !p.items.is_empty()) {
            return None;
        }
        let tc = cell.tc_pr.as_deref();
        let props = body.properties.get_or_insert_with(BodyProperties::default);
        let (ml, mr, mt, mb) = margins(cell);
        props.inset_left = Some(Written::new(Coordinate(ml as i64)));
        props.inset_right = Some(Written::new(Coordinate(mr as i64)));
        props.inset_top = Some(Written::new(Coordinate(mt as i64)));
        props.inset_bottom = Some(Written::new(Coordinate(mb as i64)));
        props.anchor = Some(tc.and_then(|t| t.anchor).unwrap_or(TextAnchor::Top));
        props.wrap = Some(TextWrap::Square);
        props.autofit = None;
        // The text style: later parts over earlier, field by field.
        let (mut color, mut bold, mut italic) = (None, None, None);
        for (part, _) in parts_for(style, flags, at) {
            let Some(tx) = part.tc_tx_style.as_ref() else {
                continue;
            };
            if let Some(c) = tx.color.as_ref() {
                color = Some(c.clone());
            } else if let Some(c) = match tx.font.as_ref() {
                Some(casual_slide_model::theme::ThemeableFont::Ref(r)) => r.color.as_ref(),
                _ => None,
            } {
                color = Some(c.clone());
            }
            if let Some(b) = tx.bold {
                bold = Some(b == OnOffStyle::On);
            }
            if let Some(i) = tx.italic {
                italic = Some(i == OnOffStyle::On);
            }
        }
        if color.is_some() || bold.is_some() || italic.is_some() {
            let apply = |props: &mut Option<Box<CharacterProperties>>| {
                let p = props.get_or_insert_with(Box::default);
                if p.fill.is_none()
                    && let Some(c) = &color
                {
                    p.fill = Some(Fill::Solid(Some(c.clone())));
                }
                if p.bold.is_none() {
                    p.bold = bold;
                }
                if p.italic.is_none() {
                    p.italic = italic;
                }
            };
            for paragraph in &mut body.paragraphs {
                for item in &mut paragraph.items {
                    match item {
                        TextItem::Run(r) => apply(&mut r.properties),
                        TextItem::Field(f) => apply(&mut f.properties),
                        TextItem::Break(b) => apply(&mut b.properties),
                        _ => {}
                    }
                }
                apply(&mut paragraph.end_properties);
            }
        }
        Some(body)
    }
}

/// PowerPoint's built-in "Medium Style 2" family by GUID, with the accent each
/// uses; `None` for the plain one, drawn in `dk1`. A deck names one by
/// `tableStyleId` and often does not carry it, since PowerPoint knows it. Only this
/// family is built in here, the default for a new table; the others are reported.
/// The GUIDs and the definition are PowerPoint's own, Inferred from decks it wrote.
const MEDIUM_STYLE_2: [(&str, Option<SchemeColor>); 7] = [
    ("{073A0DAA-6AF3-43AB-8588-CEC1D06C72B9}", None),
    (
        "{5C22544A-7EE6-4342-B048-85BDC9FD1C3A}",
        Some(SchemeColor::Accent1),
    ),
    (
        "{21E4AEA4-8DFA-4A89-87EB-49C32662AFE8}",
        Some(SchemeColor::Accent2),
    ),
    (
        "{F5AB1C69-6EDB-4FF4-983F-18BD219EF322}",
        Some(SchemeColor::Accent3),
    ),
    (
        "{00A15C55-8517-42AA-B614-E9B94910E393}",
        Some(SchemeColor::Accent4),
    ),
    (
        "{7DF18680-E054-41AD-8BC1-D1AEF772440D}",
        Some(SchemeColor::Accent5),
    ),
    (
        "{93296810-A885-4BE3-A3E7-6D5BEEA58F35}",
        Some(SchemeColor::Accent6),
    ),
];

/// A built-in table style by GUID, if it is one this knows.
fn builtin_style(id: &str) -> Option<TableStyle> {
    let (_, accent) = MEDIUM_STYLE_2
        .iter()
        .find(|(guid, _)| guid.eq_ignore_ascii_case(id))?;
    let base = accent.unwrap_or(SchemeColor::Dk1);
    let scheme = |c: SchemeColor, tint: Option<i32>| Color {
        value: ColorValue::Scheme(c),
        transforms: tint
            .map(|t| vec![ColorTransform::new(TransformKind::Tint, t)])
            .unwrap_or_default(),
    };
    let fill = |tint: Option<i32>| {
        Some(TableStyleFill::Inline(Fill::Solid(Some(scheme(
            base, tint,
        )))))
    };
    let line = |w: i32| {
        Some(TableBorder::Line(Box::new(Line {
            width: Some(w),
            fill: Some(Fill::Solid(Some(scheme(SchemeColor::Lt1, None)))),
            ..Line::default()
        })))
    };
    let part = |text: Option<SchemeColor>, bold: bool, fill, bdr| {
        Some(Box::new(TablePartStyle {
            tc_tx_style: Some(TableStyleTextStyle {
                bold: bold.then_some(OnOffStyle::On),
                color: text.map(|c| scheme(c, None)),
                ..TableStyleTextStyle::default()
            }),
            tc_style: Some(TableStyleCellStyle {
                tc_bdr: bdr,
                fill,
                cell3d: None,
            }),
        }))
    };
    let all = TableCellBorderStyle {
        left: line(12_700),
        right: line(12_700),
        top: line(12_700),
        bottom: line(12_700),
        inside_h: line(12_700),
        inside_v: line(12_700),
        ..TableCellBorderStyle::default()
    };
    Some(TableStyle {
        style_id: Some(id.to_owned()),
        whole_tbl: part(Some(SchemeColor::Dk1), false, fill(Some(20_000)), Some(all)),
        band1_h: part(None, false, fill(Some(40_000)), None),
        band1_v: part(None, false, fill(Some(40_000)), None),
        last_col: part(Some(SchemeColor::Lt1), true, fill(None), None),
        first_col: part(Some(SchemeColor::Lt1), true, fill(None), None),
        last_row: part(
            Some(SchemeColor::Lt1),
            true,
            fill(None),
            Some(TableCellBorderStyle {
                top: line(38_100),
                ..TableCellBorderStyle::default()
            }),
        ),
        first_row: part(
            Some(SchemeColor::Lt1),
            true,
            fill(None),
            Some(TableCellBorderStyle {
                bottom: line(38_100),
                ..TableCellBorderStyle::default()
            }),
        ),
        ..TableStyle::default()
    })
}

/// The table's flags and size, for picking style parts.
#[derive(Debug, Clone, Copy)]
struct Flags {
    first_row: bool,
    last_row: bool,
    first_col: bool,
    last_col: bool,
    band_row: bool,
    band_col: bool,
    rows: usize,
    cols: usize,
}

/// The region a style part covers for one cell: `(row0, col0, row1, col1)`, end
/// exclusive. Its outer borders are the part's `top`, `left` and so on, its inner
/// ones `insideH` and `insideV`.
type Region = (usize, usize, usize, usize);

/// The style parts that apply to a cell, in the order they apply, each with the
/// region it covers.
fn parts_for<'s>(
    style: Option<&'s TableStyle>,
    f: &Flags,
    at: Placed,
) -> Vec<(&'s TablePartStyle, Region)> {
    let Some(style) = style else {
        return Vec::new();
    };
    let (r, c) = (at.row, at.col);
    let (last_r, last_c) = (f.rows - 1, f.cols - 1);
    let whole = (0, 0, f.rows, f.cols);
    let row = (r, 0, r + 1, f.cols);
    let col = (0, c, f.rows, c + 1);
    let cell = (r, c, r + 1, c + 1);
    let in_first_row = f.first_row && r == 0;
    let in_last_row = f.last_row && r + at.rows - 1 == last_r;
    let in_first_col = f.first_col && c == 0;
    let in_last_col = f.last_col && c + at.cols - 1 == last_c;
    let mut kinds: Vec<(TablePartKind, Region)> = vec![(TablePartKind::WholeTable, whole)];
    if f.band_row && !in_first_row && !in_last_row {
        let k = r - usize::from(f.first_row);
        let kind = if k % 2 == 0 {
            TablePartKind::Band1Horizontal
        } else {
            TablePartKind::Band2Horizontal
        };
        kinds.push((kind, row));
    }
    if f.band_col && !in_first_col && !in_last_col {
        let k = c - usize::from(f.first_col);
        let kind = if k % 2 == 0 {
            TablePartKind::Band1Vertical
        } else {
            TablePartKind::Band2Vertical
        };
        kinds.push((kind, col));
    }
    if in_last_col {
        kinds.push((TablePartKind::LastColumn, col));
    }
    if in_first_col {
        kinds.push((TablePartKind::FirstColumn, col));
    }
    if in_last_row {
        kinds.push((TablePartKind::LastRow, row));
    }
    if in_last_row && in_last_col {
        kinds.push((TablePartKind::SouthEastCell, cell));
    }
    if in_last_row && in_first_col {
        kinds.push((TablePartKind::SouthWestCell, cell));
    }
    if in_first_row {
        kinds.push((TablePartKind::FirstRow, row));
    }
    if in_first_row && in_last_col {
        kinds.push((TablePartKind::NorthEastCell, cell));
    }
    if in_first_row && in_first_col {
        kinds.push((TablePartKind::NorthWestCell, cell));
    }
    kinds
        .into_iter()
        .filter_map(|(k, region)| Some((style.part(k)?, region)))
        .collect()
}

/// A border, from a cell's own line or from its style.
#[derive(Debug, Clone, Copy)]
enum Border<'s> {
    /// A line as stated.
    Line(&'s Line),
    /// A theme line by index, with the colour standing for `phClr`.
    Ref(u32, Option<&'s Color>),
}

/// The border a cell's style gives one of its sides: for each part that applies,
/// its outer border where the side lies on the part's edge, its inner one where
/// not; later parts over earlier.
fn style_border<'s>(
    style: Option<&'s TableStyle>,
    f: &Flags,
    at: Placed,
    side: Side,
) -> Option<Border<'s>> {
    let mut out = None;
    for (part, (r0, c0, r1, c1)) in parts_for(style, f, at) {
        let Some(bdr) = part.tc_style.as_ref().and_then(|s| s.tc_bdr.as_ref()) else {
            continue;
        };
        let border = match side {
            Side::Top if at.row == r0 => bdr.top.as_ref(),
            Side::Top => bdr.inside_h.as_ref(),
            Side::Bottom if at.row + at.rows >= r1 => bdr.bottom.as_ref(),
            Side::Bottom => bdr.inside_h.as_ref(),
            Side::Left if at.col == c0 => bdr.left.as_ref(),
            Side::Left => bdr.inside_v.as_ref(),
            Side::Right if at.col + at.cols >= c1 => bdr.right.as_ref(),
            Side::Right => bdr.inside_v.as_ref(),
        };
        match border {
            Some(TableBorder::Line(l)) => out = Some(Border::Line(l)),
            Some(TableBorder::Ref(r)) => {
                if let Some(idx) = r.idx {
                    out = Some(Border::Ref(idx, r.color.as_ref()));
                }
            }
            Some(TableBorder::Empty) | None => {}
        }
    }
    out
}

/// A cell's margins, EMU: left, right, top, bottom.
fn margins(cell: &TableCell) -> (f64, f64, f64, f64) {
    let tc = cell.tc_pr.as_deref();
    let get = |v: Option<&Written<Coordinate>>, default: i64| {
        v.map_or(default, |w| w.value().0).max(0) as f64
    };
    (
        get(tc.and_then(|t| t.mar_l.as_ref()), DEFAULT_MARGIN_LR),
        get(tc.and_then(|t| t.mar_r.as_ref()), DEFAULT_MARGIN_LR),
        get(tc.and_then(|t| t.mar_t.as_ref()), DEFAULT_MARGIN_TB),
        get(tc.and_then(|t| t.mar_b.as_ref()), DEFAULT_MARGIN_TB),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flags() -> Flags {
        Flags {
            first_row: true,
            last_row: false,
            first_col: false,
            last_col: false,
            band_row: true,
            band_col: false,
            rows: 4,
            cols: 3,
        }
    }

    fn style() -> TableStyle {
        let part = || Some(Box::new(TablePartStyle::default()));
        TableStyle {
            whole_tbl: part(),
            band1_h: part(),
            band2_h: part(),
            first_row: part(),
            ..TableStyle::default()
        }
    }

    fn at(row: usize, col: usize) -> Placed {
        Placed {
            row,
            col,
            rows: 1,
            cols: 1,
        }
    }

    #[test]
    fn the_default_built_in_style_is_known_by_guid_in_any_case() {
        let s = builtin_style("{5c22544a-7ee6-4342-b048-85bdc9fd1c3a}").unwrap();
        assert!(s.first_row.is_some() && s.band1_h.is_some() && s.band2_h.is_none());
        assert!(builtin_style("{00000000-0000-0000-0000-000000000000}").is_none());
    }

    #[test]
    fn the_header_row_takes_first_row_over_the_whole_table_and_no_band() {
        let s = style();
        let parts = parts_for(Some(&s), &flags(), at(0, 1));
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[1].1, (0, 0, 1, 3));
    }

    #[test]
    fn bands_alternate_from_the_first_row_under_the_header() {
        let s = style();
        let first = parts_for(Some(&s), &flags(), at(1, 0));
        let second = parts_for(Some(&s), &flags(), at(2, 0));
        assert!(std::ptr::eq(first[1].0, s.band1_h.as_deref().unwrap()));
        assert!(std::ptr::eq(second[1].0, s.band2_h.as_deref().unwrap()));
    }
}
