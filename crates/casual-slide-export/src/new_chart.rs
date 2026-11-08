//! A new chart and its workbook (`VS-11`): Insert > Chart's column, bar, line and pie,
//! written from a small grid of data, and the grid read back from any chart's caches so
//! the editor's data sheet can show and change it.
//!
//! The chart part is the plain shape PowerPoint writes for these four: one plot family,
//! each series' name, categories and values cached and referenced into `Sheet1` of the
//! embedded workbook, and a category and value axis except on a pie. The workbook is a
//! minimal valid `.xlsx` holding the same grid, so PowerPoint's Edit Data opens what the
//! chart shows (docs/36: written for a new chart; an existing chart's is kept).

use casual_slide_model::chart::{ChartPart, c};
use casual_slide_package::build_package;

/// The four kinds Insert > Chart offers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewChartKind {
    /// Clustered column.
    Column,
    /// Clustered bar.
    Bar,
    /// Line with markers.
    Line,
    /// Pie: its first series.
    Pie,
}

impl NewChartKind {
    /// From a bridge number: 0 column, 1 bar, 2 line, 3 pie.
    pub fn from_u32(n: u32) -> Option<Self> {
        Some(match n {
            0 => Self::Column,
            1 => Self::Bar,
            2 => Self::Line,
            3 => Self::Pie,
            _ => return None,
        })
    }

    /// The bridge number.
    pub fn code(self) -> u32 {
        match self {
            Self::Column => 0,
            Self::Bar => 1,
            Self::Line => 2,
            Self::Pie => 3,
        }
    }
}

/// The most categories or series a grid holds: the data sheet's bound.
pub const MAX_GRID_SIDE: usize = 64;

/// A chart's data as a sheet shows it: categories down, series across.
#[derive(Debug, Clone, PartialEq)]
pub struct ChartGrid {
    /// What draws it.
    pub kind: NewChartKind,
    /// Each category's label.
    pub categories: Vec<String>,
    /// Each series' name and its value at each category; `None` is a blank cell.
    pub series: Vec<(String, Vec<Option<f64>>)>,
}

impl ChartGrid {
    /// PowerPoint's own starting data for a new chart of this kind.
    pub fn sample(kind: NewChartKind) -> Self {
        let cats = |names: &[&str]| names.iter().map(|s| (*s).to_owned()).collect();
        if kind == NewChartKind::Pie {
            return Self {
                kind,
                categories: cats(&["1st Qtr", "2nd Qtr", "3rd Qtr", "4th Qtr"]),
                series: vec![(
                    "Sales".to_owned(),
                    vec![Some(8.2), Some(3.2), Some(1.4), Some(1.2)],
                )],
            };
        }
        let values = [
            [4.3, 2.5, 3.5, 4.5],
            [2.4, 4.4, 1.8, 2.8],
            [2.0, 2.0, 3.0, 5.0],
        ];
        Self {
            kind,
            categories: cats(&["Category 1", "Category 2", "Category 3", "Category 4"]),
            series: values
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    (
                        format!("Series {}", i + 1),
                        v.iter().map(|x| Some(*x)).collect(),
                    )
                })
                .collect(),
        }
    }

    /// Bounded to [`MAX_GRID_SIDE`] each way, every series as long as the categories.
    pub fn normalized(mut self) -> Self {
        self.categories.truncate(MAX_GRID_SIDE);
        self.series.truncate(MAX_GRID_SIDE);
        let n = self.categories.len();
        for (_, values) in &mut self.series {
            values.resize(n, None);
        }
        if self.kind == NewChartKind::Pie {
            self.series.truncate(1);
        }
        self
    }
}

fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            c if c.is_control() && c != '\t' => {}
            c => out.push(c),
        }
    }
    out
}

/// A spreadsheet column's letters, from 0.
fn column(mut n: usize) -> String {
    let mut out = Vec::new();
    loop {
        out.push(b'A' + (n % 26) as u8);
        if n < 26 {
            break;
        }
        n = n / 26 - 1;
    }
    out.reverse();
    String::from_utf8(out).unwrap_or_default()
}

fn number(v: f64) -> String {
    if v.is_finite() {
        format!("{v}")
    } else {
        "0".to_owned()
    }
}

/// The chart part's XML. With `workbook_rel`, it names that relationship as its data.
pub fn chart_xml(grid: &ChartGrid, workbook_rel: Option<&str>) -> String {
    let grid = grid.clone().normalized();
    let n = grid.categories.len();
    let last = n.saturating_add(1);
    let mut series = String::new();
    for (i, (name, values)) in grid.series.iter().enumerate() {
        let col = column(i + 1);
        let mut cats = String::new();
        for (k, c) in grid.categories.iter().enumerate() {
            cats.push_str(&format!("<c:pt idx=\"{k}\"><c:v>{}</c:v></c:pt>", esc(c)));
        }
        let mut vals = String::new();
        for (k, v) in values.iter().enumerate() {
            if let Some(v) = v {
                vals.push_str(&format!(
                    "<c:pt idx=\"{k}\"><c:v>{}</c:v></c:pt>",
                    number(*v)
                ));
            }
        }
        let marker = match grid.kind {
            NewChartKind::Line => {
                "<c:marker><c:symbol val=\"circle\"/><c:size val=\"5\"/></c:marker>"
            }
            _ => "",
        };
        let invert = match grid.kind {
            NewChartKind::Column | NewChartKind::Bar => "<c:invertIfNegative val=\"0\"/>",
            _ => "",
        };
        let smooth = match grid.kind {
            NewChartKind::Line => "<c:smooth val=\"0\"/>",
            _ => "",
        };
        series.push_str(&format!(
            "<c:ser><c:idx val=\"{i}\"/><c:order val=\"{i}\"/>\
             <c:tx><c:strRef><c:f>Sheet1!${col}$1</c:f><c:strCache><c:ptCount val=\"1\"/>\
             <c:pt idx=\"0\"><c:v>{}</c:v></c:pt></c:strCache></c:strRef></c:tx>{invert}{marker}\
             <c:cat><c:strRef><c:f>Sheet1!$A$2:$A${last}</c:f><c:strCache><c:ptCount val=\"{n}\"/>{cats}\
             </c:strCache></c:strRef></c:cat>\
             <c:val><c:numRef><c:f>Sheet1!${col}$2:${col}${last}</c:f><c:numCache>\
             <c:formatCode>General</c:formatCode><c:ptCount val=\"{n}\"/>{vals}</c:numCache></c:numRef></c:val>\
             {smooth}</c:ser>",
            esc(name)
        ));
    }
    let axes_ids = "<c:axId val=\"500000001\"/><c:axId val=\"500000002\"/>";
    let family = match grid.kind {
        NewChartKind::Column | NewChartKind::Bar => format!(
            "<c:barChart><c:barDir val=\"{}\"/><c:grouping val=\"clustered\"/><c:varyColors val=\"0\"/>\
             {series}<c:gapWidth val=\"219\"/><c:overlap val=\"-27\"/>{axes_ids}</c:barChart>",
            if grid.kind == NewChartKind::Bar {
                "bar"
            } else {
                "col"
            }
        ),
        NewChartKind::Line => format!(
            "<c:lineChart><c:grouping val=\"standard\"/><c:varyColors val=\"0\"/>{series}\
             <c:marker val=\"1\"/>{axes_ids}</c:lineChart>"
        ),
        NewChartKind::Pie => format!(
            "<c:pieChart><c:varyColors val=\"1\"/>{series}<c:firstSliceAng val=\"0\"/></c:pieChart>"
        ),
    };
    let (cat_pos, val_pos) = if grid.kind == NewChartKind::Bar {
        ("l", "b")
    } else {
        ("b", "l")
    };
    let axes = if grid.kind == NewChartKind::Pie {
        String::new()
    } else {
        format!(
            "<c:catAx><c:axId val=\"500000001\"/><c:scaling><c:orientation val=\"minMax\"/></c:scaling>\
             <c:delete val=\"0\"/><c:axPos val=\"{cat_pos}\"/><c:numFmt formatCode=\"General\" sourceLinked=\"1\"/>\
             <c:majorTickMark val=\"none\"/><c:minorTickMark val=\"none\"/><c:tickLblPos val=\"nextTo\"/>\
             <c:crossAx val=\"500000002\"/><c:crosses val=\"autoZero\"/><c:auto val=\"1\"/>\
             <c:lblAlgn val=\"ctr\"/><c:lblOffset val=\"100\"/><c:noMultiLvlLbl val=\"0\"/></c:catAx>\
             <c:valAx><c:axId val=\"500000002\"/><c:scaling><c:orientation val=\"minMax\"/></c:scaling>\
             <c:delete val=\"0\"/><c:axPos val=\"{val_pos}\"/><c:majorGridlines/>\
             <c:numFmt formatCode=\"General\" sourceLinked=\"1\"/><c:majorTickMark val=\"none\"/>\
             <c:minorTickMark val=\"none\"/><c:tickLblPos val=\"nextTo\"/><c:crossAx val=\"500000001\"/>\
             <c:crosses val=\"autoZero\"/><c:crossBetween val=\"between\"/></c:valAx>"
        )
    };
    let external = workbook_rel.map_or(String::new(), |r| {
        format!(
            "<c:externalData r:id=\"{}\"><c:autoUpdate val=\"0\"/></c:externalData>",
            esc(r)
        )
    });
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
         <c:chartSpace xmlns:c=\"http://schemas.openxmlformats.org/drawingml/2006/chart\" \
         xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" \
         xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\">\
         <c:date1904 val=\"0\"/><c:roundedCorners val=\"0\"/>\
         <c:chart><c:autoTitleDeleted val=\"{}\"/><c:plotArea><c:layout/>{family}{axes}</c:plotArea>\
         <c:legend><c:legendPos val=\"b\"/><c:overlay val=\"0\"/></c:legend>\
         <c:plotVisOnly val=\"1\"/><c:dispBlanksAs val=\"gap\"/></c:chart>{external}</c:chartSpace>",
        if grid.series.len() == 1 { 0 } else { 1 }
    )
}

/// The embedded workbook: `Sheet1` holding the grid, categories in column A from row 2
/// and each series in its own column with its name in row 1. Strings are inline.
pub fn workbook(grid: &ChartGrid) -> Result<Vec<u8>, casual_slide_package::PackageError> {
    let grid = grid.clone().normalized();
    let text = |r: &str, s: &str| {
        format!(
            "<c r=\"{r}\" t=\"inlineStr\"><is><t xml:space=\"preserve\">{}</t></is></c>",
            esc(s)
        )
    };
    let mut rows = String::from("<row r=\"1\">");
    for (i, (name, _)) in grid.series.iter().enumerate() {
        rows.push_str(&text(&format!("{}1", column(i + 1)), name));
    }
    rows.push_str("</row>");
    for (k, cat) in grid.categories.iter().enumerate() {
        let r = k + 2;
        rows.push_str(&format!("<row r=\"{r}\">"));
        rows.push_str(&text(&format!("A{r}"), cat));
        for (i, (_, values)) in grid.series.iter().enumerate() {
            if let Some(Some(v)) = values.get(k) {
                rows.push_str(&format!(
                    "<c r=\"{}{r}\"><v>{}</v></c>",
                    column(i + 1),
                    number(*v)
                ));
            }
        }
        rows.push_str("</row>");
    }
    const DECL: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n";
    const MAIN: &str = "http://schemas.openxmlformats.org/spreadsheetml/2006/main";
    const RELS: &str = "http://schemas.openxmlformats.org/package/2006/relationships";
    const DOC: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
    let types = format!(
        "{DECL}<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
         <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
         <Default Extension=\"xml\" ContentType=\"application/xml\"/>\
         <Override PartName=\"/xl/workbook.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml\"/>\
         <Override PartName=\"/xl/worksheets/sheet1.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml\"/>\
         </Types>"
    );
    let root_rels = format!(
        "{DECL}<Relationships xmlns=\"{RELS}\"><Relationship Id=\"rId1\" \
         Type=\"{DOC}/officeDocument\" Target=\"xl/workbook.xml\"/></Relationships>"
    );
    let book = format!(
        "{DECL}<workbook xmlns=\"{MAIN}\" xmlns:r=\"{DOC}\"><sheets>\
         <sheet name=\"Sheet1\" sheetId=\"1\" r:id=\"rId1\"/></sheets></workbook>"
    );
    let book_rels = format!(
        "{DECL}<Relationships xmlns=\"{RELS}\"><Relationship Id=\"rId1\" \
         Type=\"{DOC}/worksheet\" Target=\"worksheets/sheet1.xml\"/></Relationships>"
    );
    let sheet =
        format!("{DECL}<worksheet xmlns=\"{MAIN}\"><sheetData>{rows}</sheetData></worksheet>");
    build_package(&[
        ("[Content_Types].xml", types.as_bytes()),
        ("_rels/.rels", root_rels.as_bytes()),
        ("xl/workbook.xml", book.as_bytes()),
        ("xl/_rels/workbook.xml.rels", book_rels.as_bytes()),
        ("xl/worksheets/sheet1.xml", sheet.as_bytes()),
    ])
}

/// One series' parts, whichever family it is in.
struct SerRef<'a> {
    tx: &'a mut Option<Box<c::SerTx>>,
    cat: &'a mut Option<Box<c::AxDataSource>>,
    val: &'a mut Option<Box<c::NumDataSource>>,
}

/// The first bar, line or pie family's series, and its kind.
fn family_mut(space: &mut c::ChartSpace) -> Option<(NewChartKind, Vec<SerRef<'_>>)> {
    let plot = space.chart.as_mut()?.plot_area.as_mut()?;
    for family in &mut plot.charts {
        match family {
            c::PlotAreaCharts::BarChart(b) => {
                let horizontal = b
                    .bar_dir
                    .as_deref()
                    .and_then(|d| d.val)
                    .is_some_and(|v| v == c::BarDirValue::Bar);
                let kind = if horizontal {
                    NewChartKind::Bar
                } else {
                    NewChartKind::Column
                };
                let sers = b
                    .ser
                    .iter_mut()
                    .map(|s| SerRef {
                        tx: &mut s.tx,
                        cat: &mut s.cat,
                        val: &mut s.val,
                    })
                    .collect();
                return Some((kind, sers));
            }
            c::PlotAreaCharts::LineChart(l) => {
                let sers = l
                    .ser
                    .iter_mut()
                    .map(|s| SerRef {
                        tx: &mut s.tx,
                        cat: &mut s.cat,
                        val: &mut s.val,
                    })
                    .collect();
                return Some((NewChartKind::Line, sers));
            }
            c::PlotAreaCharts::PieChart(p) => {
                let sers = p
                    .ser
                    .iter_mut()
                    .map(|s| SerRef {
                        tx: &mut s.tx,
                        cat: &mut s.cat,
                        val: &mut s.val,
                    })
                    .collect();
                return Some((NewChartKind::Pie, sers));
            }
            _ => {}
        }
    }
    None
}

fn str_cache(cat: &mut c::AxDataSource) -> Option<&mut c::StrData> {
    cat.str_ref.as_deref_mut()?.str_cache.as_deref_mut()
}

fn num_cache(val: &mut c::NumDataSource) -> Option<&mut c::NumData> {
    if let Some(r) = val.num_ref.as_deref_mut() {
        return r.num_cache.as_deref_mut();
    }
    val.num_lit.as_deref_mut()
}

fn count(pt_count: Option<&c::UnsignedInt>, idx: impl Iterator<Item = Option<u32>>) -> usize {
    let most = idx.flatten().map(|i| i as usize + 1).max().unwrap_or(0);
    pt_count
        .and_then(|c| c.val)
        .map_or(most, |c| c as usize)
        .min(MAX_GRID_SIDE)
}

/// The grid a chart's caches hold, for its first bar, line or pie family. `None` for any
/// other chart, which the data sheet does not edit yet.
pub fn read_grid(part: &ChartPart) -> Option<ChartGrid> {
    let ChartPart::Chart(space) = part else {
        return None;
    };
    let mut space = (**space).clone();
    let (kind, sers) = family_mut(&mut space)?;
    let mut categories: Vec<String> = Vec::new();
    let mut series = Vec::new();
    for s in sers.into_iter().take(MAX_GRID_SIDE) {
        let name =
            s.tx.as_deref()
                .and_then(|t| {
                    t.v.clone().or_else(|| {
                        t.str_ref
                            .as_deref()?
                            .str_cache
                            .as_deref()?
                            .pt
                            .first()?
                            .v
                            .clone()
                    })
                })
                .unwrap_or_default();
        if categories.is_empty()
            && let Some(cache) = s.cat.as_deref_mut().and_then(str_cache)
        {
            let n = count(cache.pt_count.as_deref(), cache.pt.iter().map(|p| p.idx));
            categories = vec![String::new(); n];
            for (k, p) in cache.pt.iter().enumerate() {
                if let Some(slot) = categories.get_mut(p.idx.map_or(k, |i| i as usize)) {
                    *slot = p.v.clone().unwrap_or_default();
                }
            }
        }
        let mut values = Vec::new();
        if let Some(cache) = s.val.as_deref_mut().and_then(num_cache) {
            let n = count(cache.pt_count.as_deref(), cache.pt.iter().map(|p| p.idx));
            values = vec![None; n];
            for (k, p) in cache.pt.iter().enumerate() {
                if let Some(slot) = values.get_mut(p.idx.map_or(k, |i| i as usize)) {
                    *slot = p.v.as_deref().and_then(|v| v.trim().parse::<f64>().ok());
                }
            }
        }
        series.push((name, values));
    }
    let n = series
        .iter()
        .map(|(_, v)| v.len())
        .max()
        .unwrap_or(0)
        .max(categories.len());
    categories.resize(n, String::new());
    Some(
        ChartGrid {
            kind,
            categories,
            series,
        }
        .normalized(),
    )
}

/// A chart with the grid's names, categories and values in its caches, everything else
/// as it was. `None` when the grid's shape differs from what the chart holds: adding a
/// series or category to a chart from the file is not done yet.
pub fn write_grid(part: &ChartPart, grid: &ChartGrid) -> Option<ChartPart> {
    let current = read_grid(part)?;
    if current.categories.len() != grid.categories.len()
        || current.series.len() != grid.series.len()
    {
        return None;
    }
    let ChartPart::Chart(space) = part else {
        return None;
    };
    let mut space = (**space).clone();
    let (_, sers) = family_mut(&mut space)?;
    for (s, (name, values)) in sers.into_iter().zip(&grid.series) {
        if let Some(tx) = s.tx.as_deref_mut() {
            if tx.v.is_some() {
                tx.v = Some(name.clone());
            } else if let Some(pt) = tx
                .str_ref
                .as_deref_mut()
                .and_then(|r| r.str_cache.as_deref_mut())
                .and_then(|c| c.pt.first_mut())
            {
                pt.v = Some(name.clone());
            }
        }
        if let Some(cache) = s.cat.as_deref_mut().and_then(str_cache) {
            for (k, label) in grid.categories.iter().enumerate() {
                let idx = u32::try_from(k).ok()?;
                match cache.pt.iter_mut().find(|p| p.idx == Some(idx)) {
                    Some(p) => p.v = Some(label.clone()),
                    None => cache.pt.push(c::StrVal {
                        idx: Some(idx),
                        v: Some(label.clone()),
                    }),
                }
            }
        }
        if let Some(cache) = s.val.as_deref_mut().and_then(num_cache) {
            for (k, v) in values.iter().enumerate() {
                let idx = u32::try_from(k).ok()?;
                let at = cache.pt.iter().position(|p| p.idx == Some(idx));
                match (at, v) {
                    (Some(i), Some(v)) => {
                        if let Some(p) = cache.pt.get_mut(i)
                            && p.v.as_deref().and_then(|s| s.trim().parse::<f64>().ok()) != Some(*v)
                        {
                            p.v = Some(number(*v));
                        }
                    }
                    (Some(i), None) => {
                        cache.pt.remove(i);
                    }
                    (None, Some(v)) => cache.pt.push(c::NumVal {
                        idx: Some(idx),
                        v: Some(number(*v)),
                        ..Default::default()
                    }),
                    (None, None) => {}
                }
            }
            cache.pt.sort_by_key(|p| p.idx);
        }
    }
    Some(ChartPart::Chart(Box::new(space)))
}
