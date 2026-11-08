//! The Design tab (`VS-12`): themes, their colour variants, Slide Size and Format
//! Background, each one operation through `casual-slide-transaction`, so each is one
//! undo step and saves through the one writer.
//!
//! The themes are a small set defined here: palettes and font pairs of our own, not
//! Microsoft's themes, named so they cannot be mistaken for them. A theme changes each
//! master's colour scheme and the Latin typeface of its heading and body fonts; every
//! other part of the theme stays as the file had it.

use casual_slide_model::color::{Angle, Color, Percentage};
use casual_slide_model::fill::{Fill, GradientFill, GradientShade, GradientStop};
use casual_slide_model::presentation::{Coordinate, POSITIVE_COORDINATE_MAX, Point, PositiveSize};
use casual_slide_model::slide::{
    Background, BackgroundFill, BackgroundProperties, SlideKind, SlideParts,
};
use casual_slide_model::theme::{ColorScheme, FontScheme, Theme};
use casual_slide_render::Deck;
use casual_slide_transaction::{Document, Operation};

use wasm_bindgen::prelude::*;

use crate::{BridgeError, SESSION, Viewer, with_viewer};

/// One built-in theme: a name, its fonts, and its colour variants, the first its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuiltinTheme {
    /// What the gallery shows, and the scheme names written to the file.
    pub name: &'static str,
    /// The heading font, `majorFont/latin`.
    pub major: &'static str,
    /// The body font, `minorFont/latin`.
    pub minor: &'static str,
    /// Colour sets, each the twelve slots `dk1, lt1, dk2, lt2, accent1..6, hlink,
    /// folHlink` as `0xRRGGBB`. The first is the theme's own; the rest are Variants.
    pub variants: &'static [[u32; 12]],
}

/// The built-in themes. Fonts are ones `casual-slide-text` draws with a
/// metric-compatible face, so a deck looks the same here as where they are installed.
pub const THEMES: &[BuiltinTheme] = &[
    BuiltinTheme {
        name: "Plain",
        major: "Calibri",
        minor: "Calibri",
        variants: &[
            [
                0x000000, 0xFFFFFF, 0x2B3A4A, 0xE8ECF0, 0x3A6EA5, 0xD9822B, 0x8A8F96, 0xE0B83A,
                0x4F8FC0, 0x6AA84F, 0x2F6FB0, 0x7A4F9A,
            ],
            [
                0x000000, 0xFFFFFF, 0x3B2F2F, 0xF3EEE8, 0xB5523B, 0xD9A441, 0x7D8C5A, 0x5B7DA8,
                0x9C6B98, 0x5FA39B, 0xB5523B, 0x7D5BA8,
            ],
            [
                0x000000, 0xFFFFFF, 0x1F3B35, 0xEAF2EF, 0x2F8F6F, 0x7DB547, 0x2F7FA8, 0xE0A23A,
                0xC0504D, 0x8064A2, 0x2F7FA8, 0x8064A2,
            ],
        ],
    },
    BuiltinTheme {
        name: "Harbour",
        major: "Georgia",
        minor: "Calibri",
        variants: &[
            [
                0x0B1F33, 0xFFFFFF, 0x16324F, 0xDDE7F0, 0x1F6FB2, 0x2BA6A0, 0xF2A541, 0xE4572E,
                0x76B041, 0x9B5DE5, 0x1F6FB2, 0x6B4E9B,
            ],
            [
                0x0B1F33, 0xFFFFFF, 0x24304A, 0xE6E9F2, 0x4058A8, 0x7F8FD0, 0xC9A227, 0x9E3D64,
                0x3E8E7E, 0xD8743C, 0x4058A8, 0x9E3D64,
            ],
            [
                0x0B1F33, 0xFFFFFF, 0x0F3D3E, 0xE0F0EE, 0x16807A, 0x5DB7AE, 0xF0C75E, 0xE07A5F,
                0x3D5A80, 0x98C1D9, 0x16807A, 0x3D5A80,
            ],
        ],
    },
    BuiltinTheme {
        name: "Meadow",
        major: "Cambria",
        minor: "Calibri",
        variants: &[
            [
                0x1E2B1A, 0xFFFFFF, 0x34502B, 0xEEF3E6, 0x5B8C3A, 0xA3C35A, 0xE3B23C, 0xC8553D,
                0x3F7F9F, 0x8C6D46, 0x3F7F9F, 0x8C6D46,
            ],
            [
                0x1E2B1A, 0xFFFFFF, 0x45412A, 0xF4F1E4, 0x8A7D2F, 0xC2A83E, 0x6F9A5B, 0xB8683A,
                0x5A7FA0, 0x9A5B7F, 0x5A7FA0, 0x9A5B7F,
            ],
        ],
    },
    BuiltinTheme {
        name: "Ember",
        major: "Arial",
        minor: "Arial",
        variants: &[
            [
                0x1A1110, 0xFFFFFF, 0x4A1F1A, 0xF7ECE8, 0xC0392B, 0xE67E22, 0xF1C40F, 0x8E5A3C,
                0x7F8C8D, 0x2C3E50, 0xC0392B, 0x8E44AD,
            ],
            [
                0x1A1110, 0xFFFFFF, 0x3D2333, 0xF5EAF0, 0xA8325E, 0xE0607E, 0xF0A35E, 0x6C4A8A,
                0x7A8A99, 0x2E4057, 0xA8325E, 0x6C4A8A,
            ],
        ],
    },
    BuiltinTheme {
        name: "Slate",
        major: "Calibri",
        minor: "Calibri",
        variants: &[
            [
                0x111418, 0xFFFFFF, 0x2E3440, 0xE5E9F0, 0x5E81AC, 0x88C0D0, 0xA3BE8C, 0xEBCB8B,
                0xD08770, 0xB48EAD, 0x5E81AC, 0xB48EAD,
            ],
            [
                0x111418, 0xFFFFFF, 0x333333, 0xEDEDED, 0x555555, 0x777777, 0x999999, 0x3D7EBA,
                0xC0504D, 0x4F9A6E, 0x3D7EBA, 0x7F5FA0,
            ],
        ],
    },
    BuiltinTheme {
        name: "Orchid",
        major: "Times New Roman",
        minor: "Georgia",
        variants: &[
            [
                0x24122E, 0xFFFFFF, 0x4B2A5E, 0xF2EAF6, 0x8E44AD, 0xC27BA0, 0x5DADE2, 0xF5B041,
                0x48C9B0, 0xEC7063, 0x5D6DBE, 0x8E44AD,
            ],
            [
                0x24122E, 0xFFFFFF, 0x2F2350, 0xEDEAF7, 0x5B4FC2, 0x9A8CE6, 0xE88A9A, 0xF2C46D,
                0x4FB3A9, 0x7A6F9B, 0x5B4FC2, 0x9A8CE6,
            ],
        ],
    },
];

/// How Slide Size fits the content to the new size, as PowerPoint asks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fit {
    /// Ensure Fit: scale down to the smaller ratio, so everything stays on the slide.
    EnsureFit,
    /// Maximize: scale to the larger ratio, so content fills the slide.
    Maximize,
}

impl Fit {
    /// From the bridge's number: 0 ensure fit, 1 maximize.
    pub fn from_code(code: u32) -> Option<Self> {
        match code {
            0 => Some(Self::EnsureFit),
            1 => Some(Self::Maximize),
            _ => None,
        }
    }
}

/// A background Format Background can set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackgroundChoice {
    /// Solid fill, `0xRRGGBB`.
    Solid(u32),
    /// A linear gradient from one colour to another at an angle, degrees.
    Gradient(u32, u32, u32),
    /// No background of the slide's own: it shows its layout's again.
    Reset,
}

/// The deck as `casual-slide-transaction` edits it: parts, each master's theme, the size.
pub(crate) struct DeckDoc<'a>(pub(crate) &'a mut Deck);

impl Document for DeckDoc<'_> {
    fn parts_mut(&mut self) -> &mut SlideParts {
        self.0.parts_mut()
    }
    fn theme_mut(&mut self, master: u32) -> Option<&mut Theme> {
        self.0.theme_mut(master)
    }
    fn slide_size_mut(&mut self) -> Option<&mut (i64, i64)> {
        Some(self.0.size_mut())
    }
    fn chart_mut(&mut self, part: &str) -> Option<&mut casual_slide_model::chart::ChartPart> {
        self.0.chart_mut(part)
    }
    fn properties_mut(
        &mut self,
    ) -> Option<&mut casual_slide_model::properties::DocumentProperties> {
        Some(self.0.properties_mut())
    }
}

/// Whether an operation changes more than one slide's picture: a theme or the slide
/// size, a layout or master, or several slides.
pub(crate) fn redraws_all(
    op: &Operation,
    structure: &casual_slide_model::slide::DeckStructure,
) -> bool {
    fn parts(op: &Operation, out: &mut std::collections::BTreeSet<u32>) {
        match op {
            Operation::Batch(ops) => ops.iter().for_each(|o| parts(o, out)),
            other => {
                if let Some(p) = other.part() {
                    out.insert(p);
                }
            }
        }
    }
    if op.is_deck_wide() {
        return true;
    }
    let mut touched = std::collections::BTreeSet::new();
    parts(op, &mut touched);
    touched.len() > 1 || touched.iter().any(|p| !structure.slides.contains(p))
}

fn rgb(v: u32) -> Color {
    let [_, r, g, b] = v.to_be_bytes();
    Color::srgb(r, g, b)
}

/// A theme's scheme with these colours: what was read, its slots replaced, so whatever
/// else it holds stays. `None` when the master's theme has no colour scheme to replace.
fn colors_from(read: Option<&ColorScheme>, name: &str, colors: &[u32; 12]) -> Option<ColorScheme> {
    let mut scheme = read?.clone();
    scheme.name = Some(name.to_owned());
    for (slot, v) in scheme.slots.iter_mut().zip(colors) {
        *slot = Some(rgb(*v));
    }
    Some(scheme)
}

/// A font scheme with these Latin typefaces, where the read scheme has a Latin font.
fn fonts_from(read: Option<&FontScheme>, theme: &BuiltinTheme) -> Option<FontScheme> {
    let mut scheme = read?.clone();
    scheme.name = Some(theme.name.to_owned());
    for (collection, face) in [
        (&mut scheme.major, theme.major),
        (&mut scheme.minor, theme.minor),
    ] {
        if let Some(latin) = collection.as_mut().and_then(|c| c.latin.as_mut()) {
            latin.typeface = Some(face.to_owned());
        }
    }
    Some(scheme)
}

fn clamp_coordinate(v: f64) -> Coordinate {
    let max = POSITIVE_COORDINATE_MAX as f64;
    Coordinate(v.round().clamp(-max, max) as i64)
}

fn clamp_size(v: f64) -> i64 {
    v.round().clamp(0.0, POSITIVE_COORDINATE_MAX as f64) as i64
}

impl Viewer {
    fn masters(&self) -> Vec<u32> {
        let themes = self.deck.themes();
        self.deck
            .parts()
            .structure
            .masters
            .iter()
            .copied()
            .filter(|m| themes.contains_key(m))
            .collect()
    }

    fn apply_design(&mut self, op: Operation) -> Result<bool, BridgeError> {
        self.typing = None;
        let changed = self
            .history
            .apply(&mut DeckDoc(&mut self.deck), op)
            .map_err(BridgeError::Edit)?;
        if changed {
            self.reports.clear();
        }
        Ok(changed)
    }

    /// Apply built-in theme `theme` with colour variant `variant` to every master: its
    /// colours and its fonts, one undo step. With `colors_only`, the Variants gallery,
    /// the fonts stay. Returns whether anything changed.
    pub fn apply_theme(
        &mut self,
        theme: usize,
        variant: usize,
        colors_only: bool,
    ) -> Result<bool, BridgeError> {
        let t = THEMES.get(theme).ok_or(BridgeError::BadArgument)?;
        let colors = t.variants.get(variant).ok_or(BridgeError::BadArgument)?;
        let mut ops = Vec::new();
        for master in self.masters() {
            let Some(styles) = self
                .deck
                .themes()
                .get(&master)
                .and_then(|t| t.elements.as_ref())
            else {
                continue;
            };
            if let Some(scheme) = colors_from(styles.color_scheme.as_ref(), t.name, colors) {
                ops.push(Operation::SetColorScheme {
                    master,
                    scheme: Some(Box::new(scheme)),
                });
            }
            if !colors_only && let Some(scheme) = fonts_from(styles.font_scheme.as_ref(), t) {
                ops.push(Operation::SetFontScheme {
                    master,
                    scheme: Some(Box::new(scheme)),
                });
            }
        }
        if ops.is_empty() {
            return Err(BridgeError::Edit(
                casual_slide_transaction::TxnError::NoDesign,
            ));
        }
        self.apply_design(Operation::Batch(ops))
    }

    /// Change the slide size to `cx` by `cy` EMU, scaling what every slide, layout and
    /// master places to fit, centred, as PowerPoint's Ensure Fit or Maximize. One undo
    /// step. Text sizes are not scaled.
    pub fn set_slide_size(&mut self, cx: i64, cy: i64, fit: Fit) -> Result<bool, BridgeError> {
        let (ow, oh) = self.deck.size();
        let mut ops = vec![Operation::SetSlideSize { cx, cy }];
        let (sx, sy) = (cx as f64 / ow.max(1) as f64, cy as f64 / oh.max(1) as f64);
        let s = match fit {
            Fit::EnsureFit => sx.min(sy),
            Fit::Maximize => sx.max(sy),
        };
        let dx = (cx as f64 - ow as f64 * s) / 2.0;
        let dy = (cy as f64 - oh as f64 * s) / 2.0;
        if (ow, oh) != (cx, cy) {
            let parts = self.deck.parts();
            for entry in &parts.structure.parts {
                if !matches!(
                    entry.kind,
                    SlideKind::Slide | SlideKind::Layout | SlideKind::Master
                ) {
                    continue;
                }
                let Some(root) = parts.get(entry.id.part).and_then(|p| p.shape_tree()) else {
                    continue;
                };
                for node in &root.children {
                    let Some(mut t) = node.transform else {
                        continue;
                    };
                    if let Some(o) = t.offset {
                        t.offset = Some(Point {
                            x: clamp_coordinate(o.x.0 as f64 * s + dx),
                            y: clamp_coordinate(o.y.0 as f64 * s + dy),
                        });
                    }
                    if let Some(e) = t.extent {
                        t.extent = Some(PositiveSize {
                            cx: clamp_size(e.cx as f64 * s),
                            cy: clamp_size(e.cy as f64 * s),
                        });
                    }
                    if Some(t) != node.transform {
                        ops.push(Operation::SetTransform {
                            node: node.id,
                            transform: Some(t),
                        });
                    }
                }
            }
        }
        self.apply_design(Operation::Batch(ops))
    }

    /// Give slide `index` a background, or every slide with `None`: Format Background's
    /// Apply to All. One undo step.
    pub fn set_background(
        &mut self,
        index: Option<usize>,
        choice: BackgroundChoice,
    ) -> Result<bool, BridgeError> {
        let slides: Vec<u32> = match index {
            Some(i) => vec![self.slide_ordinal(i)?],
            None => self.deck.parts().structure.slides.clone(),
        };
        let fill = match choice {
            BackgroundChoice::Solid(c) => Some(Fill::Solid(Some(rgb(c)))),
            BackgroundChoice::Gradient(a, b, angle) => {
                Some(Fill::Gradient(Box::new(GradientFill {
                    flip: None,
                    rotate_with_shape: Some(true),
                    stops: Some(vec![
                        GradientStop {
                            position: Some(Percentage(0)),
                            color: Some(rgb(a)),
                        },
                        GradientStop {
                            position: Some(Percentage(100_000)),
                            color: Some(rgb(b)),
                        },
                    ]),
                    shade: Some(GradientShade::Linear {
                        // Under 360 degrees, so under 21,600,000 and in an `i32`.
                        angle: Some(Angle(i32::try_from(angle % 360).unwrap_or(0) * 60_000)),
                        scaled: Some(false),
                    }),
                    tile_rect: None,
                })))
            }
            BackgroundChoice::Reset => None,
        };
        let mut ops = Vec::new();
        let parts = self.deck.parts();
        for ordinal in slides {
            let Some(common) = parts.get(ordinal).and_then(|p| p.common.as_ref()) else {
                continue;
            };
            let background = fill.clone().map(|fill| {
                let had = common.background.as_ref();
                Box::new(Background {
                    bw_mode: had.and_then(|b| b.bw_mode),
                    fill: Some(BackgroundFill::Properties(BackgroundProperties {
                        shade_to_title: None,
                        has_fill: true,
                        fill: Some(fill),
                        // The schema requires effects: those read are kept, and a new
                        // background has an empty list.
                        has_effects: true,
                    })),
                })
            });
            ops.push(Operation::SetBackground {
                part: ordinal,
                background,
            });
        }
        self.apply_design(Operation::Batch(ops))
    }

    /// The colour scheme and fonts the first master's theme has now: twelve colours as
    /// `0xRRGGBB` (0 where a slot is not plain RGB), for the gallery's current mark.
    pub fn theme_colors(&self) -> Vec<u32> {
        let Some(scheme) = self
            .masters()
            .first()
            .and_then(|m| self.deck.themes().get(m))
            .and_then(|t| t.elements.as_ref())
            .and_then(|e| e.color_scheme.as_ref())
        else {
            return Vec::new();
        };
        scheme
            .slots
            .iter()
            .map(|c| match c.as_ref().map(|c| &c.value) {
                Some(casual_slide_model::color::ColorValue::Srgb(v)) => {
                    u32::from_be_bytes([0, v.r, v.g, v.b])
                }
                _ => 0,
            })
            .collect()
    }
}

/// The built-in themes, one line each: `name|major|minor|variants`, the variants
/// separated by `;`, each twelve `RRGGBB` separated by `,`. Static data of our own;
/// nothing from the document.
#[wasm_bindgen]
#[must_use]
pub fn design_themes() -> Vec<String> {
    THEMES
        .iter()
        .map(|t| {
            let variants: Vec<String> = t
                .variants
                .iter()
                .map(|v| {
                    v.iter()
                        .map(|c| format!("{c:06X}"))
                        .collect::<Vec<_>>()
                        .join(",")
                })
                .collect();
            format!("{}|{}|{}|{}", t.name, t.major, t.minor, variants.join(";"))
        })
        .collect()
}

/// Apply built-in theme `theme`, colour variant `variant`; with `colors_only` the fonts
/// stay (the Variants gallery). Returns whether anything changed.
///
/// # Errors
///
/// `CS-WSM-0006` for a theme or variant that is not there; `CS-TXN-0010` when no
/// master has a theme to change.
#[wasm_bindgen]
pub fn design_apply_theme(theme: u32, variant: u32, colors_only: bool) -> Result<bool, JsError> {
    with_viewer(|v| v.apply_theme(theme as usize, variant as usize, colors_only))
}

/// Change the slide size to `cx` by `cy` EMU; `fit` 0 ensures the content fits, 1
/// maximizes it.
///
/// # Errors
///
/// `CS-TXN-0011` for a size outside 914400 to 51206400 EMU; `CS-WSM-0006` for a bad `fit`.
#[wasm_bindgen]
pub fn design_slide_size(cx: f64, cy: f64, fit: u32) -> Result<bool, JsError> {
    with_viewer(|v| {
        let fit = Fit::from_code(fit).ok_or(BridgeError::BadArgument)?;
        if !cx.is_finite() || !cy.is_finite() {
            return Err(BridgeError::BadArgument);
        }
        // Out of range stays out of range, for the transaction to refuse with its code.
        let side = |x: f64| x.round().clamp(-1.0, 1e15) as i64;
        v.set_slide_size(side(cx), side(cy), fit)
    })
}

/// Format Background for slide `index`, or every slide with -1: `kind` 0 solid in
/// colour `a`, 1 a linear gradient from `a` to `b` at `angle` degrees, 2 reset to the
/// layout's. Colours are `0xRRGGBB`.
///
/// # Errors
///
/// `CS-WSM-0006` for a bad `kind`; the slide's own error past the last slide.
#[wasm_bindgen]
pub fn design_background(
    index: i32,
    kind: u32,
    a: u32,
    b: u32,
    angle: u32,
) -> Result<bool, JsError> {
    with_viewer(|v| {
        let choice = match kind {
            0 => BackgroundChoice::Solid(a & 0xFF_FFFF),
            1 => BackgroundChoice::Gradient(a & 0xFF_FFFF, b & 0xFF_FFFF, angle % 360),
            2 => BackgroundChoice::Reset,
            _ => return Err(BridgeError::BadArgument),
        };
        let index = usize::try_from(index).ok();
        v.set_background(index, choice)
    })
}

/// The first master's twelve theme colours now, `0xRRGGBB`; empty with no theme.
#[wasm_bindgen]
#[must_use]
pub fn design_theme_colors() -> Vec<u32> {
    SESSION.with(|s| {
        s.borrow()
            .as_ref()
            .map(Viewer::theme_colors)
            .unwrap_or_default()
    })
}

#[cfg(test)]
mod tests;
