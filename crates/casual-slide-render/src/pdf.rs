//! The PDF backend: a second executor of the slide display list, one page per
//! slide, with real text (`VS-04`, `T2G-01`).
//!
//! Copied from OpenCalc's `casual-calc-render/src/pdf.rs` at `b23c67c`
//! (PROVENANCE.md, owner instruction 42): the object store with reserved numbers
//! and a cross-reference table, the number writer that never emits an exponent or
//! `NaN`, the Type0 / Identity-H fonts over embedded, subset TrueType faces with
//! a `/W` array and a `ToUnicode` map, `/ActualText` for a run whose glyphs do not
//! map one to one onto its characters, and `TJ` adjustments taken from the same
//! rounded widths `/W` is written with. What is new is the item set: OpenCalc
//! paints cells, a slide paints paths with transforms, gradients and pictures.
//!
//! # Why the bytes are written here, by hand
//!
//! OpenCalc's reason holds: a page is a few dozen operators, and every PDF crate
//! carries a large surface reached by a document's own bytes. This adds no PDF
//! dependency; `skrifa` reads the faces, and `flate2`, already in the tree for
//! the package reader, compresses pictures and font programs.
//!
//! # Coordinates
//!
//! PDF space is points, y up. The display list is EMU, y down. Each page installs
//! one matrix, `1/12700 0 0 -1/12700 0 h cm`, so everything under it is written in
//! slide EMU, y down; each item then installs its own [`Affine`]. Text flips its
//! glyphs back upright with a `1 0 0 -1 x y Tm` text matrix, as in OpenCalc.
//!
//! # Determinism
//!
//! No creation date, no file id, no producer version: the same deck gives the
//! same bytes. Faces, pictures, shadings and transparency states are numbered in
//! first-encounter order over the pages.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::Write as _;

use skrifa::instance::{LocationRef, Size};
use skrifa::{FontRef, MetadataProvider};

use crate::display::{Affine, Cap, DisplayItem, DisplayList, Join, Paint, PathCmd, Rgba, Stop};
use crate::images::{self, ImageSource};
use crate::{RenderError, RenderReport};

/// EMU in one PDF point.
pub const EMU_PER_POINT: f64 = 12_700.0;

/// What the document says about itself.
#[derive(Debug, Clone, Default)]
pub struct PdfMetadata {
    /// The document title. Empty writes no `/Info` at all.
    pub title: String,
}

/// A face the document embeds, and the glyphs drawn from it.
struct Face {
    bytes: &'static [u8],
    /// Glyph id to the text it stands for, for `ToUnicode` and `/W`. Empty for a
    /// glyph whose text is carried by `/ActualText` instead.
    used: BTreeMap<u32, String>,
}

/// Everything the pages share, collected while their content is written.
#[derive(Default)]
struct Resources {
    /// Faces by key, in first-encounter order.
    faces: Vec<(String, Face)>,
    /// Transparency states by `(stroke, thousandths)`.
    alphas: BTreeMap<(bool, u16), usize>,
    /// Shading dictionaries, in order.
    shadings: Vec<String>,
    /// Pictures by part: their index, or `None` when they could not be decoded.
    pictures: BTreeMap<String, Option<usize>>,
    /// Decoded pictures, in index order.
    picture_data: Vec<Picture>,
}

/// A decoded picture: straight RGB, and its alpha when any pixel is not opaque.
struct Picture {
    width: u32,
    height: u32,
    rgb: Vec<u8>,
    alpha: Option<Vec<u8>>,
}

impl Resources {
    /// The resource index of the face a key names, adding it when new; `None`
    /// when no face has that key or its bytes do not parse.
    fn face(&mut self, key: &str) -> Option<usize> {
        if let Some(i) = self.faces.iter().position(|(k, _)| k == key) {
            return Some(i);
        }
        let bytes = casual_slide_text::face_by_key(key)?;
        FontRef::new(bytes).ok()?;
        self.faces.push((
            key.to_owned(),
            Face {
                bytes,
                used: BTreeMap::new(),
            },
        ));
        Some(self.faces.len() - 1)
    }

    /// A glyph's advance in thousandths of an em, rounded as `/W` writes it.
    fn advance(&self, face: usize, gid: u32) -> f64 {
        let Some((_, face)) = self.faces.get(face) else {
            return 0.0;
        };
        let Ok(font) = FontRef::new(face.bytes) else {
            return 0.0;
        };
        let upem = f64::from(
            font.metrics(Size::unscaled(), LocationRef::default())
                .units_per_em
                .max(1),
        );
        let advance = font
            .glyph_metrics(Size::unscaled(), LocationRef::default())
            .advance_width(skrifa::GlyphId::new(gid))
            .unwrap_or(0.0);
        (f64::from(advance) * 1000.0 / upem).round()
    }

    /// The text a glyph already stands for, if any.
    fn glyph_text(&self, face: usize, gid: u32) -> Option<&str> {
        self.faces
            .get(face)?
            .1
            .used
            .get(&gid)
            .map(String::as_str)
            .filter(|t| !t.is_empty())
    }

    fn note(&mut self, face: usize, gid: u32, text: &str) {
        if let Some((_, f)) = self.faces.get_mut(face) {
            let entry = f.used.entry(gid).or_default();
            if entry.is_empty() {
                entry.push_str(text);
            }
        }
    }

    /// The name of the transparency state for `alpha`, for fills or strokes.
    /// `None` when the colour is opaque.
    fn alpha(&mut self, stroke: bool, alpha: f64) -> Option<String> {
        let thousandths = (alpha.clamp(0.0, 1.0) * 1000.0).round() as u16;
        if thousandths >= 1000 {
            return None;
        }
        let next = self.alphas.len();
        let index = *self.alphas.entry((stroke, thousandths)).or_insert(next);
        Some(format!("/A{index}"))
    }

    /// The picture a part holds, decoded once.
    fn picture(
        &mut self,
        part: &str,
        source: &dyn ImageSource,
        report: &mut RenderReport,
    ) -> Option<usize> {
        if let Some(known) = self.pictures.get(part) {
            if known.is_some() {
                report.images.drew();
            }
            return *known;
        }
        let decoded = match images::decode(part, source) {
            Ok(pixmap) => {
                let pixels = pixmap.pixels();
                let mut rgb = Vec::with_capacity(pixels.len().saturating_mul(3));
                let mut alpha = Vec::with_capacity(pixels.len());
                for p in pixels {
                    let c = p.demultiply();
                    rgb.extend_from_slice(&[c.red(), c.green(), c.blue()]);
                    alpha.push(c.alpha());
                }
                let opaque = alpha.iter().all(|&a| a == 255);
                self.picture_data.push(Picture {
                    width: pixmap.width(),
                    height: pixmap.height(),
                    rgb,
                    alpha: (!opaque).then_some(alpha),
                });
                report.images.drew();
                Some(self.picture_data.len() - 1)
            }
            Err(reason) => {
                report.images.missed(part, reason);
                None
            }
        };
        self.pictures.insert(part.to_owned(), decoded);
        decoded
    }
}

/// Write a PDF of `pages`, one page per display list, drawing the pictures
/// `images` supplies. Returns the file and, per page, what it did not draw or
/// drew approximately; nothing is left out silently.
///
/// # Errors
///
/// [`RenderError::InvalidSize`] when a page has no area: a media box of zero is a
/// file no viewer opens.
pub fn write_pdf(
    pages: &[&DisplayList],
    images: &dyn ImageSource,
    meta: &PdfMetadata,
) -> Result<(Vec<u8>, Vec<RenderReport>), RenderError> {
    for page in pages {
        let ok = |v: f64| v.is_finite() && v >= 1.0;
        if !ok(page.width) || !ok(page.height) {
            return Err(RenderError::InvalidSize {
                width: page.width.max(0.0).min(f64::from(u32::MAX)) as u32,
                height: page.height.max(0.0).min(f64::from(u32::MAX)) as u32,
            });
        }
    }

    // Content first: it is what discovers the faces, pictures and states.
    let mut res = Resources::default();
    let mut streams = Vec::with_capacity(pages.len());
    let mut reports = Vec::with_capacity(pages.len());
    for page in pages {
        let mut report = RenderReport::default();
        streams.push(page_content(page, images, &mut res, &mut report));
        reports.push(report);
    }

    let mut objects = Objects::default();
    let catalog = objects.reserve();
    let pages_id = objects.reserve();

    let font_ids = write_faces(&mut objects, &res);
    let mut resources = String::from("<< /ExtGState << ");
    for (&(stroke, thousandths), index) in &res.alphas {
        let key = if stroke { "CA" } else { "ca" };
        let id = objects.set_now(
            format!(
                "<< /Type /ExtGState /{key} {} >>",
                num(f64::from(thousandths) / 1000.0)
            )
            .into_bytes(),
        );
        let _ = write!(resources, "/A{index} {id} 0 R ");
    }
    resources.push_str(">> /Font << ");
    for (i, id) in font_ids.iter().enumerate() {
        let _ = write!(resources, "/F{i} {id} 0 R ");
    }
    resources.push_str(">> /Shading << ");
    for (i, body) in res.shadings.iter().enumerate() {
        let id = objects.set_now(body.clone().into_bytes());
        let _ = write!(resources, "/S{i} {id} 0 R ");
    }
    resources.push_str(">> /XObject << ");
    for (i, picture) in res.picture_data.iter().enumerate() {
        let smask = picture.alpha.as_ref().map(|alpha| {
            let dict = format!(
                "/Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace /DeviceGray \
                 /BitsPerComponent 8 /Filter /FlateDecode",
                picture.width, picture.height
            );
            objects.stream(dict.as_bytes(), &deflate(alpha))
        });
        let mut dict = format!(
            "/Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace /DeviceRGB \
             /BitsPerComponent 8 /Filter /FlateDecode",
            picture.width, picture.height
        );
        if let Some(smask) = smask {
            let _ = write!(dict, " /SMask {smask} 0 R");
        }
        let id = objects.stream(dict.as_bytes(), &deflate(&picture.rgb));
        let _ = write!(resources, "/I{i} {id} 0 R ");
    }
    resources.push_str(">> >>");
    let resources_id = objects.set_now(resources.into_bytes());

    let mut kids = String::new();
    for (page, stream) in pages.iter().zip(&streams) {
        let content_id = objects.stream(b"", stream.as_bytes());
        let body = format!(
            "<< /Type /Page /Parent {pages_id} 0 R /MediaBox [0 0 {} {}] \
             /Resources {resources_id} 0 R /Contents {content_id} 0 R >>",
            num(page.width / EMU_PER_POINT),
            num(page.height / EMU_PER_POINT),
        );
        let id = objects.set_now(body.into_bytes());
        let _ = write!(kids, "{id} 0 R ");
    }
    objects.set(
        pages_id,
        format!(
            "<< /Type /Pages /Kids [{}] /Count {} >>",
            kids.trim_end(),
            pages.len()
        )
        .into_bytes(),
    );
    let info = (!meta.title.is_empty()).then(|| {
        objects.set_now(format!("<< /Title {} >>", utf16_string(&meta.title)).into_bytes())
    });
    objects.set(
        catalog,
        format!("<< /Type /Catalog /Pages {pages_id} 0 R >>").into_bytes(),
    );
    Ok((objects.finish(catalog, info), reports))
}

/// zlib-wrapped deflate, which is what `/FlateDecode` reads.
fn deflate(data: &[u8]) -> Vec<u8> {
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    // Writing to a `Vec` cannot fail; an error would leave an empty stream,
    // which a viewer draws as nothing rather than refusing the file.
    if encoder.write_all(data).is_err() {
        return Vec::new();
    }
    encoder.finish().unwrap_or_default()
}

// ---------------------------------------------------------------------------
// The object store (OpenCalc's, unchanged but for a checked `set`)
// ---------------------------------------------------------------------------

/// Indirect objects by number, written out with a cross-reference table.
#[derive(Default)]
struct Objects {
    bodies: Vec<Option<Vec<u8>>>,
}

impl Objects {
    fn reserve(&mut self) -> usize {
        self.bodies.push(None);
        self.bodies.len()
    }

    fn set(&mut self, id: usize, body: Vec<u8>) {
        if let Some(slot) = id.checked_sub(1).and_then(|i| self.bodies.get_mut(i)) {
            *slot = Some(body);
        }
    }

    fn set_now(&mut self, body: Vec<u8>) -> usize {
        self.bodies.push(Some(body));
        self.bodies.len()
    }

    /// A stream object: `dict_extra` is spliced into its dictionary alongside the
    /// `/Length` this computes.
    fn stream(&mut self, dict_extra: &[u8], data: &[u8]) -> usize {
        let mut body = Vec::with_capacity(data.len() + 64);
        body.extend_from_slice(b"<< /Length ");
        body.extend_from_slice(data.len().to_string().as_bytes());
        if !dict_extra.is_empty() {
            body.push(b' ');
            body.extend_from_slice(dict_extra);
        }
        body.extend_from_slice(b" >>\nstream\n");
        body.extend_from_slice(data);
        body.extend_from_slice(b"\nendstream");
        self.set_now(body)
    }

    fn finish(self, catalog: usize, info: Option<usize>) -> Vec<u8> {
        let mut out: Vec<u8> = Vec::new();
        out.extend_from_slice(b"%PDF-1.7\n%\xE2\xE3\xCF\xD3\n");
        let mut offsets = Vec::with_capacity(self.bodies.len());
        for (index, body) in self.bodies.iter().enumerate() {
            offsets.push(out.len());
            out.extend_from_slice(format!("{} 0 obj\n", index + 1).as_bytes());
            match body {
                Some(bytes) => out.extend_from_slice(bytes),
                None => out.extend_from_slice(b"null"),
            }
            out.extend_from_slice(b"\nendobj\n");
        }
        let startxref = out.len();
        out.extend_from_slice(format!("xref\n0 {}\n", self.bodies.len() + 1).as_bytes());
        out.extend_from_slice(b"0000000000 65535 f \n");
        for offset in &offsets {
            out.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
        }
        let mut trailer = format!(
            "trailer\n<< /Size {} /Root {catalog} 0 R",
            self.bodies.len() + 1
        );
        if let Some(info) = info {
            let _ = write!(trailer, " /Info {info} 0 R");
        }
        trailer.push_str(" >>\n");
        out.extend_from_slice(trailer.as_bytes());
        out.extend_from_slice(format!("startxref\n{startxref}\n%%EOF\n").as_bytes());
        out
    }
}

/// A text string as UTF-16BE hex with its byte-order mark: any title, any script.
fn utf16_string(text: &str) -> String {
    let mut out = String::from("<FEFF");
    for unit in text.encode_utf16() {
        let _ = write!(out, "{unit:04X}");
    }
    out.push('>');
    out
}

/// The `/ActualText` dictionary for a run, as UTF-16BE.
fn actual_text_dict(text: &str) -> String {
    format!("<< /ActualText {} >>", utf16_string(text))
}

/// A number a PDF parser accepts: fixed point, never an exponent, `NaN` or `inf`.
fn num(value: f64) -> String {
    if !value.is_finite() {
        return "0".to_owned();
    }
    let mut s = format!("{value:.3}");
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
    if s == "-0" { "0".to_owned() } else { s }
}

// ---------------------------------------------------------------------------
// Fonts (OpenCalc's `write_faces` and `to_unicode_cmap`)
// ---------------------------------------------------------------------------

/// Write every face as a Type0 font over an embedded subset, returning the object
/// id of each. Identity-H addresses glyphs by id, and `ToUnicode` gives the text
/// back to a reader, a search box, or `pdftotext`.
fn write_faces(objects: &mut Objects, res: &Resources) -> Vec<usize> {
    let mut ids = Vec::with_capacity(res.faces.len());
    for (index, (_, face)) in res.faces.iter().enumerate() {
        let Ok(font) = FontRef::new(face.bytes) else {
            // `Resources::face` parsed it before any glyph was recorded, so this
            // is unreachable; a placeholder keeps the numbering aligned.
            ids.push(
                objects.set_now(b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_vec()),
            );
            continue;
        };
        let metrics = font.metrics(Size::unscaled(), LocationRef::default());
        let upem = f64::from(metrics.units_per_em.max(1));
        let to_thousandths = |v: f32| (f64::from(v) * 1000.0 / upem).round();
        let glyph_metrics = font.glyph_metrics(Size::unscaled(), LocationRef::default());

        // Only the glyphs the pages draw, at their own ids. A face the subsetter
        // cannot read (CFF) is embedded whole. The `XXXXXX+` prefix marks a
        // subset (ISO 32000-1 §9.6.4), derived from the index for determinism.
        let subset = crate::subset::subset(face.bytes, face.used.keys().copied());
        let program: &[u8] = subset.as_deref().unwrap_or(face.bytes);
        let base = if subset.is_some() {
            let mut tag = String::new();
            let mut n = index;
            for _ in 0..6 {
                tag.push(char::from(b'A' + (n % 26) as u8));
                n /= 26;
            }
            format!("{tag}+OPF{index}")
        } else {
            format!("OPF{index}")
        };
        let file_id = objects.stream(
            format!("/Length1 {} /Filter /FlateDecode", program.len()).as_bytes(),
            &deflate(program),
        );

        let bounds = metrics.bounds.unwrap_or(skrifa::metrics::BoundingBox {
            x_min: 0.0,
            y_min: -200.0,
            x_max: 1000.0,
            y_max: 900.0,
        });
        let mut flags = 4u32;
        if metrics.is_monospace {
            flags |= 1;
        }
        if metrics.italic_angle != 0.0 {
            flags |= 64;
        }
        let descriptor = objects.set_now(
            format!(
                "<< /Type /FontDescriptor /FontName /{base} /Flags {flags} \
                 /FontBBox [{} {} {} {}] /ItalicAngle {} /Ascent {} /Descent {} \
                 /CapHeight {} /StemV 80 /FontFile2 {file_id} 0 R >>",
                num(to_thousandths(bounds.x_min)),
                num(to_thousandths(bounds.y_min)),
                num(to_thousandths(bounds.x_max)),
                num(to_thousandths(bounds.y_max)),
                num(f64::from(metrics.italic_angle)),
                num(to_thousandths(metrics.ascent)),
                num(to_thousandths(metrics.descent)),
                num(to_thousandths(metrics.cap_height.unwrap_or(metrics.ascent))),
            )
            .into_bytes(),
        );

        let mut widths = String::new();
        for &gid in face.used.keys() {
            let advance = glyph_metrics
                .advance_width(skrifa::GlyphId::new(gid))
                .unwrap_or(0.0);
            let _ = write!(widths, "{gid} [{}] ", num(to_thousandths(advance)));
        }
        let cid_font = objects.set_now(
            format!(
                "<< /Type /Font /Subtype /CIDFontType2 /BaseFont /{base} \
                 /CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> \
                 /FontDescriptor {descriptor} 0 R /CIDToGIDMap /Identity /W [{}] >>",
                widths.trim_end()
            )
            .into_bytes(),
        );
        let to_unicode = objects.stream(b"", to_unicode_cmap(&face.used).as_bytes());
        ids.push(
            objects.set_now(
                format!(
                    "<< /Type /Font /Subtype /Type0 /BaseFont /{base} /Encoding /Identity-H \
                     /DescendantFonts [{cid_font} 0 R] /ToUnicode {to_unicode} 0 R >>"
                )
                .into_bytes(),
            ),
        );
    }
    ids
}

/// The `ToUnicode` CMap: what each glyph id means.
fn to_unicode_cmap(used: &BTreeMap<u32, String>) -> String {
    let mut out = String::from(
        "/CIDInit /ProcSet findresource begin\n12 dict begin\nbegincmap\n\
         /CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def\n\
         /CMapName /Adobe-Identity-UCS def\n/CMapType 2 def\n\
         1 begincodespacerange\n<0000> <FFFF>\nendcodespacerange\n",
    );
    let entries: Vec<(u32, &str)> = used
        .iter()
        .filter(|(_, text)| !text.is_empty())
        .map(|(&g, t)| (g, t.as_str()))
        .collect();
    // `bfchar` blocks hold at most 100 entries.
    for chunk in entries.chunks(100) {
        let _ = writeln!(out, "{} beginbfchar", chunk.len());
        for (gid, text) in chunk {
            let mut utf16 = String::new();
            for unit in text.encode_utf16() {
                let _ = write!(utf16, "{unit:04X}");
            }
            let _ = writeln!(out, "<{gid:04X}> <{utf16}>");
        }
        out.push_str("endbfchar\n");
    }
    out.push_str("endcmap\nCMapName currentdict /CMap defineresource pop\nend\nend\n");
    out
}

// ---------------------------------------------------------------------------
// Content
// ---------------------------------------------------------------------------

fn note(report: &mut RenderReport, feature: &'static str) {
    let n = report.scene.features.entry(feature).or_insert(0);
    *n = n.saturating_add(1);
}

/// The content stream for one slide.
fn page_content(
    list: &DisplayList,
    images: &dyn ImageSource,
    res: &mut Resources,
    report: &mut RenderReport,
) -> String {
    let mut out = String::new();
    let s = 1.0 / EMU_PER_POINT;
    let _ = writeln!(
        out,
        "q {} 0 0 {} 0 {} cm",
        num_fine(s),
        num_fine(-s),
        num(list.height / EMU_PER_POINT)
    );
    // A white ground, as the PNG path paints.
    let _ = writeln!(
        out,
        "1 1 1 rg 0 0 {} {} re f",
        num(list.width),
        num(list.height)
    );
    // PDF has no blur here: a blurred shadow or glow is stepped into rings.
    let mut items: Vec<std::borrow::Cow<'_, DisplayItem>> = Vec::with_capacity(list.items.len());
    for item in &list.items {
        if matches!(item, DisplayItem::Blurred { .. }) {
            note(report, "shadow or glow blur (stepped)");
            items.extend(item.stepped().into_iter().map(std::borrow::Cow::Owned));
        } else {
            items.push(std::borrow::Cow::Borrowed(item));
        }
    }
    for item in &items {
        let item: &DisplayItem = item;
        match item {
            DisplayItem::Path {
                transform,
                path,
                fill,
                stroke,
                ..
            } => {
                let mut outline = String::new();
                if !write_path(&mut outline, path) {
                    continue;
                }
                if let Some(fill) = fill {
                    let _ = writeln!(out, "q {}", cm(transform));
                    write_fill(&mut out, &outline, fill, images, res, report);
                    out.push_str("Q\n");
                }
                if let Some(stroke) = stroke {
                    let _ = writeln!(out, "q {}", cm(transform));
                    let color = match &stroke.paint {
                        Paint::Solid(c) => *c,
                        Paint::Linear { stops, .. } | Paint::Radial { stops, .. } => {
                            note(report, "pdf: gradient line (drawn in its first colour)");
                            stops.first().map_or(BLACK, |s| s.color)
                        }
                        Paint::Picture { .. } => {
                            note(report, "pdf: picture line (not drawn)");
                            out.push_str("Q\n");
                            continue;
                        }
                        Paint::Pattern { fg, .. } => {
                            note(report, "pdf: pattern line (drawn in its foreground)");
                            *fg
                        }
                    };
                    if let Some(gs) = res.alpha(true, f64::from(color.a) / 255.0) {
                        let _ = write!(out, "{gs} gs ");
                    }
                    let (cap, join, miter) = match (stroke.cap, stroke.join) {
                        (c, Join::Round) => (c, 1, 10.0),
                        (c, Join::Bevel) => (c, 2, 10.0),
                        (c, Join::Miter(limit)) => (c, 0, limit.clamp(1.0, 100.0)),
                    };
                    let cap = match cap {
                        Cap::Butt => 0,
                        Cap::Round => 1,
                        Cap::Square => 2,
                    };
                    let _ = write!(
                        out,
                        "{} RG {} w {cap} J {join} j {} M ",
                        rgb(color),
                        num(stroke.width.max(0.0)),
                        num(miter)
                    );
                    let dashes: Vec<String> = stroke
                        .dash
                        .iter()
                        .filter(|d| d.is_finite() && **d >= 0.0)
                        .map(|d| num(*d))
                        .collect();
                    if !dashes.is_empty() && dashes.iter().any(|d| d != "0") {
                        let _ = write!(out, "[{}] 0 d ", dashes.join(" "));
                    }
                    out.push('\n');
                    out.push_str(&outline);
                    out.push_str("S\nQ\n");
                }
            }
            // Stepped into paths above.
            DisplayItem::Blurred { .. } => {}
            DisplayItem::GlyphRun {
                transform,
                face,
                size,
                glyphs,
                fill,
                text,
                ..
            } => {
                write_run(
                    &mut out, transform, face, *size, glyphs, fill, text, res, report,
                );
            }
        }
    }
    out.push_str("Q\n");
    out
}

const BLACK: Rgba = Rgba {
    r: 0,
    g: 0,
    b: 0,
    a: 255,
};

/// The scale factor needs more places than a coordinate.
fn num_fine(value: f64) -> String {
    if !value.is_finite() {
        return "0".to_owned();
    }
    let s = format!("{value:.10}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" || s.is_empty() {
        "0".to_owned()
    } else {
        s.to_owned()
    }
}

fn cm(t: &Affine) -> String {
    format!(
        "{} {} {} {} {} {} cm",
        num_fine(t.a),
        num_fine(t.b),
        num_fine(t.c),
        num_fine(t.d),
        num(t.e),
        num(t.f)
    )
}

fn rgb(c: Rgba) -> String {
    let f = |v: u8| num(f64::from(v) / 255.0);
    format!("{} {} {}", f(c.r), f(c.g), f(c.b))
}

/// Write `cmds` as path construction operators. Returns false when there is no
/// path to draw. Quadratics become cubics, which is all a PDF path has.
fn write_path(out: &mut String, cmds: &[PathCmd]) -> bool {
    let mut current = (0.0, 0.0);
    let mut start = (0.0, 0.0);
    let mut open = false;
    let mut any = false;
    for cmd in cmds {
        match *cmd {
            PathCmd::MoveTo(x, y) => {
                let _ = writeln!(out, "{} {} m", num(x), num(y));
                current = (x, y);
                start = current;
                open = true;
            }
            PathCmd::LineTo(x, y) => {
                if !open {
                    let _ = writeln!(out, "{} {} m", num(current.0), num(current.1));
                    open = true;
                }
                let _ = writeln!(out, "{} {} l", num(x), num(y));
                current = (x, y);
                any = true;
            }
            PathCmd::QuadTo(qx, qy, x, y) => {
                if !open {
                    let _ = writeln!(out, "{} {} m", num(current.0), num(current.1));
                    open = true;
                }
                let c1 = (
                    current.0 + 2.0 / 3.0 * (qx - current.0),
                    current.1 + 2.0 / 3.0 * (qy - current.1),
                );
                let c2 = (x + 2.0 / 3.0 * (qx - x), y + 2.0 / 3.0 * (qy - y));
                let _ = writeln!(
                    out,
                    "{} {} {} {} {} {} c",
                    num(c1.0),
                    num(c1.1),
                    num(c2.0),
                    num(c2.1),
                    num(x),
                    num(y)
                );
                current = (x, y);
                any = true;
            }
            PathCmd::CubicTo(ax, ay, bx, by, x, y) => {
                if !open {
                    let _ = writeln!(out, "{} {} m", num(current.0), num(current.1));
                    open = true;
                }
                let _ = writeln!(
                    out,
                    "{} {} {} {} {} {} c",
                    num(ax),
                    num(ay),
                    num(bx),
                    num(by),
                    num(x),
                    num(y)
                );
                current = (x, y);
                any = true;
            }
            PathCmd::Close => {
                if open {
                    out.push_str("h\n");
                    current = start;
                }
            }
        }
    }
    any
}

/// Fill the path in `outline`, already under the item's transform.
fn write_fill(
    out: &mut String,
    outline: &str,
    paint: &Paint,
    images: &dyn ImageSource,
    res: &mut Resources,
    report: &mut RenderReport,
) {
    match paint {
        Paint::Solid(c) => {
            if c.a == 0 {
                return;
            }
            if let Some(gs) = res.alpha(false, f64::from(c.a) / 255.0) {
                let _ = write!(out, "{gs} gs ");
            }
            let _ = writeln!(out, "{} rg", rgb(*c));
            out.push_str(outline);
            out.push_str("f\n");
        }
        Paint::Linear { start, end, stops } => {
            let coords = format!(
                "{} {} {} {}",
                num(start.0),
                num(start.1),
                num(end.0),
                num(end.1)
            );
            shade(out, outline, 2, &coords, stops, res, report);
        }
        Paint::Radial {
            center,
            radius,
            stops,
        } => {
            let coords = format!(
                "{x} {y} 0 {x} {y} {}",
                num(radius.max(0.0)),
                x = num(center.0),
                y = num(center.1)
            );
            shade(out, outline, 3, &coords, stops, res, report);
        }
        Paint::Pattern { .. } => {
            note(report, "pdf: pattern fill (drawn as a blend)");
            if let Some(c) = paint.representative() {
                write_fill(out, outline, &Paint::Solid(c), images, res, report);
            }
        }
        Paint::Picture {
            part,
            frame,
            alpha,
            tile,
        } => {
            if tile.is_some() {
                note(report, "pdf: tiled picture fill (drawn stretched)");
            }
            let Some(index) = res.picture(part, images, report) else {
                return;
            };
            if let Some(gs) = res.alpha(false, *alpha) {
                let _ = write!(out, "{gs} gs ");
            }
            out.push_str(outline);
            // The image's unit square, y up, onto the frame, y down.
            let (x, y, w, h) = *frame;
            let _ = writeln!(
                out,
                "W n {} 0 0 {} {} {} cm /I{index} Do",
                num(w),
                num(-h),
                num(x),
                num(y + h)
            );
        }
    }
}

/// Fill the clip of `outline` with an axial (2) or radial (3) shading.
fn shade(
    out: &mut String,
    outline: &str,
    kind: u8,
    coords: &str,
    stops: &[Stop],
    res: &mut Resources,
    report: &mut RenderReport,
) {
    let Some(function) = gradient_function(stops) else {
        return;
    };
    if stops.iter().any(|s| s.color.a < 255) {
        note(report, "pdf: gradient transparency (drawn opaque)");
    }
    res.shadings.push(format!(
        "<< /ShadingType {kind} /ColorSpace /DeviceRGB /Coords [{coords}] \
         /Function {function} /Extend [true true] >>"
    ));
    let index = res.shadings.len() - 1;
    out.push_str(outline);
    let _ = writeln!(out, "W n /S{index} sh");
}

/// The colour function across `stops`: one exponential segment between each
/// pair, stitched. `None` when there are no stops.
fn gradient_function(stops: &[Stop]) -> Option<String> {
    let mut points: Vec<(f64, Rgba)> = stops
        .iter()
        .map(|s| {
            let o = if s.offset.is_finite() {
                s.offset.clamp(0.0, 1.0)
            } else {
                0.0
            };
            (o, s.color)
        })
        .collect();
    points.sort_by(|a, b| a.0.total_cmp(&b.0));
    let first = *points.first()?;
    let last = *points.last()?;
    if first.0 > 0.0 {
        points.insert(0, (0.0, first.1));
    }
    if last.0 < 1.0 {
        points.push((1.0, last.1));
    }
    let segment = |a: Rgba, b: Rgba| {
        format!(
            "<< /FunctionType 2 /Domain [0 1] /C0 [{}] /C1 [{}] /N 1 >>",
            rgb(a),
            rgb(b)
        )
    };
    if points.len() == 1 {
        return Some(segment(first.1, first.1));
    }
    if points.len() == 2 {
        return Some(segment(points[0].1, points[1].1));
    }
    let mut functions = String::new();
    let mut bounds = String::new();
    let mut encode = String::new();
    for pair in points.windows(2) {
        if let [a, b] = pair {
            let _ = write!(functions, "{} ", segment(a.1, b.1));
            let _ = write!(encode, "0 1 ");
        }
    }
    for p in points.iter().skip(1).take(points.len().saturating_sub(2)) {
        let _ = write!(bounds, "{} ", num(p.0));
    }
    Some(format!(
        "<< /FunctionType 3 /Domain [0 1] /Functions [{}] /Bounds [{}] /Encode [{}] >>",
        functions.trim_end(),
        bounds.trim_end(),
        encode.trim_end()
    ))
}

/// One glyph run as real text: the face embedded, each glyph by id, the text
/// given back through `ToUnicode` or, where glyphs and characters do not pair
/// one to one, `/ActualText`.
#[allow(clippy::too_many_arguments)] // One display item's fields, and the shared state.
fn write_run(
    out: &mut String,
    transform: &Affine,
    face: &str,
    size: f64,
    glyphs: &[crate::display::Glyph],
    fill: &Paint,
    text: &str,
    res: &mut Resources,
    report: &mut RenderReport,
) {
    // Identity-H addresses glyphs with two bytes; no TrueType face has more.
    let glyphs: Vec<&crate::display::Glyph> = glyphs
        .iter()
        .filter(|g| g.id <= 0xFFFF && g.x.is_finite() && g.y.is_finite())
        .collect();
    if glyphs.is_empty() || !size.is_finite() || size <= 0.0 {
        return;
    }
    let Some(index) = res.face(face) else {
        report.glyph_runs_undrawn = report.glyph_runs_undrawn.saturating_add(1);
        return;
    };
    let color = match fill {
        Paint::Solid(c) => *c,
        Paint::Linear { stops, .. } | Paint::Radial { stops, .. } => {
            note(report, "pdf: gradient text (drawn in its first colour)");
            stops.first().map_or(BLACK, |s| s.color)
        }
        Paint::Picture { .. } => {
            note(report, "pdf: picture-filled text (drawn black)");
            BLACK
        }
        Paint::Pattern { fg, .. } => {
            note(report, "pdf: pattern-filled text (drawn in its foreground)");
            *fg
        }
    };

    // One to one when there are as many characters as glyphs and no glyph has
    // been given another meaning already; otherwise the run says what it means.
    let chars: Vec<char> = text.chars().collect();
    let mut seen: BTreeMap<u32, char> = BTreeMap::new();
    let paired = chars.len() == glyphs.len()
        && glyphs.iter().zip(&chars).all(|(g, c)| {
            let earlier = res
                .glyph_text(index, g.id)
                .is_none_or(|t| t.chars().eq(std::iter::once(*c)));
            earlier && *seen.entry(g.id).or_insert(*c) == *c
        });
    for (i, g) in glyphs.iter().enumerate() {
        let meaning = if paired {
            chars.get(i).map(|c| c.to_string()).unwrap_or_default()
        } else {
            String::new()
        };
        res.note(index, g.id, &meaning);
    }

    let Some(first) = glyphs.first() else {
        return;
    };
    let _ = writeln!(out, "q {}", cm(transform));
    let actual = !paired && !text.is_empty();
    if actual {
        let _ = writeln!(out, "/Span {} BDC", actual_text_dict(text));
    }
    if let Some(gs) = res.alpha(false, f64::from(color.a) / 255.0) {
        let _ = write!(out, "{gs} gs ");
    }
    let _ = writeln!(
        out,
        "BT /F{index} {} Tf {} rg 1 0 0 -1 {} {} Tm",
        num(size),
        rgb(color),
        num(first.x),
        num(first.y)
    );
    let mut array = String::new();
    for (i, g) in glyphs.iter().enumerate() {
        let _ = write!(array, "<{:04X}> ", g.id);
        let Some(next) = glyphs.get(i + 1) else {
            break;
        };
        if (next.y - g.y).abs() > 0.5 {
            // Off this baseline: start again where the next glyph sits.
            flush_tj(out, &mut array);
            let _ = writeln!(out, "1 0 0 -1 {} {} Tm", num(next.x), num(next.y));
            continue;
        }
        let natural = res.advance(index, g.id) / 1000.0 * size;
        let drift = next.x - g.x - natural;
        let adjust = -drift * 1000.0 / size;
        if adjust.abs() >= 0.5 {
            let _ = write!(array, "{} ", num(adjust));
        }
    }
    flush_tj(out, &mut array);
    out.push_str("ET\n");
    if actual {
        out.push_str("EMC\n");
    }
    out.push_str("Q\n");
}

/// Close an accumulated `TJ` array, if it has anything in it.
fn flush_tj(out: &mut String, array: &mut String) {
    if !array.is_empty() {
        let _ = writeln!(out, "[{}] TJ", array.trim_end());
        array.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::{Glyph, Layer};
    use crate::images::NoImages;

    fn page(items: Vec<DisplayItem>) -> DisplayList {
        DisplayList {
            width: 9_144_000.0,
            height: 6_858_000.0,
            items,
        }
    }

    #[test]
    fn numbers_never_carry_an_exponent_or_nan() {
        assert_eq!(num(f64::NAN), "0");
        assert_eq!(num(f64::INFINITY), "0");
        assert_eq!(num(-0.0001), "0");
        assert_eq!(num(1e20), "100000000000000000000");
        assert_eq!(num(1.5), "1.5");
    }

    #[test]
    fn one_page_per_list_and_a_page_of_no_size_is_an_error() {
        let a = page(Vec::new());
        let b = page(Vec::new());
        let (pdf, reports) = write_pdf(&[&a, &b], &NoImages, &PdfMetadata::default()).unwrap();
        assert_eq!(reports.len(), 2);
        let text = String::from_utf8_lossy(&pdf);
        assert!(text.contains("/Count 2"));
        assert_eq!(text.matches("/Type /Page ").count(), 2);
        let empty = DisplayList::default();
        assert_eq!(
            write_pdf(&[&empty], &NoImages, &PdfMetadata::default())
                .unwrap_err()
                .code(),
            "CS-RND-0001"
        );
    }

    #[test]
    fn a_run_in_an_unknown_face_is_counted_not_dropped() {
        let run = DisplayItem::GlyphRun {
            object: None,
            layer: Layer::Slide,
            transform: Affine::IDENTITY,
            face: "no-such-face".to_owned(),
            size: 228_600.0,
            glyphs: vec![Glyph {
                id: 3,
                x: 0.0,
                y: 0.0,
            }],
            fill: Paint::Solid(BLACK),
            text: "x".to_owned(),
        };
        let (_, reports) =
            write_pdf(&[&page(vec![run])], &NoImages, &PdfMetadata::default()).unwrap();
        assert_eq!(reports[0].glyph_runs_undrawn, 1);
    }

    #[test]
    fn the_same_lists_give_the_same_bytes() {
        let make = || {
            let item = DisplayItem::Path {
                object: None,
                layer: Layer::Slide,
                transform: Affine::translate(100.0, 200.0),
                path: vec![
                    PathCmd::MoveTo(0.0, 0.0),
                    PathCmd::QuadTo(10.0, 0.0, 10.0, 10.0),
                    PathCmd::Close,
                ],
                fill: Some(Paint::Linear {
                    start: (0.0, 0.0),
                    end: (10.0, 0.0),
                    stops: vec![
                        Stop {
                            offset: 0.0,
                            color: BLACK,
                        },
                        Stop {
                            offset: 0.5,
                            color: Rgba {
                                r: 255,
                                g: 0,
                                b: 0,
                                a: 128,
                            },
                        },
                        Stop {
                            offset: 1.0,
                            color: BLACK,
                        },
                    ],
                }),
                stroke: None,
            };
            let list = page(vec![item]);
            write_pdf(&[&list], &NoImages, &PdfMetadata::default())
                .unwrap()
                .0
        };
        let one = make();
        assert_eq!(one, make());
        let text = String::from_utf8_lossy(&one);
        assert!(text.contains("/FunctionType 3"));
        assert!(text.contains("/Bounds [0.5]"));
    }
}
