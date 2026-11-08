//! The backend-neutral display list: the one contract between building a slide's
//! scene and any backend that paints it (`docs/42` §Display list).
//!
//! Shaped after OpenCalc's `casual-calc-layout::display` (its ADR-008): painter's
//! order, deterministic, nothing a backend has to decide, and pictures carried by
//! package path rather than by bytes. Its items differ, because a slide is general
//! vector art rather than grid decoration: a cell rectangle becomes a path with a
//! transform, a fill and a stroke.
//!
//! Units are EMU throughout. Each item's coordinates are in its shape's own space,
//! `0..w` by `0..h`, and its [`Affine`] maps that space onto the slide. A backend
//! scales slide EMU to pixels once, at the end.

use casual_slide_model::shape::ObjectId;

/// A 2D affine map `(x, y) -> (a·x + c·y + e, b·x + d·y + f)`, in EMU.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Affine {
    /// x scale.
    pub a: f64,
    /// y skew.
    pub b: f64,
    /// x skew.
    pub c: f64,
    /// y scale.
    pub d: f64,
    /// x translation.
    pub e: f64,
    /// y translation.
    pub f: f64,
}

impl Affine {
    /// The identity.
    pub const IDENTITY: Affine = Affine {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: 0.0,
        f: 0.0,
    };

    /// A translation.
    pub fn translate(x: f64, y: f64) -> Affine {
        Affine {
            e: x,
            f: y,
            ..Self::IDENTITY
        }
    }

    /// A scale about the origin.
    pub fn scale(x: f64, y: f64) -> Affine {
        Affine {
            a: x,
            d: y,
            ..Self::IDENTITY
        }
    }

    /// A clockwise rotation, in degrees, about the origin, in a y-down space.
    pub fn rotate_degrees(degrees: f64) -> Affine {
        let (s, c) = degrees.to_radians().sin_cos();
        Affine {
            a: c,
            b: s,
            c: -s,
            d: c,
            e: 0.0,
            f: 0.0,
        }
    }

    /// `self` applied after `inner`: `self ∘ inner`.
    #[must_use]
    pub fn then_after(&self, inner: &Affine) -> Affine {
        Affine {
            a: self.a * inner.a + self.c * inner.b,
            b: self.b * inner.a + self.d * inner.b,
            c: self.a * inner.c + self.c * inner.d,
            d: self.b * inner.c + self.d * inner.d,
            e: self.a * inner.e + self.c * inner.f + self.e,
            f: self.b * inner.e + self.d * inner.f + self.f,
        }
    }

    /// Map a point.
    pub fn apply(&self, x: f64, y: f64) -> (f64, f64) {
        (
            self.a * x + self.c * y + self.e,
            self.b * x + self.d * y + self.f,
        )
    }
}

/// One path command, in the item's own space. Arcs are already cubics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathCmd {
    /// Start a subpath.
    MoveTo(f64, f64),
    /// Straight line.
    LineTo(f64, f64),
    /// Quadratic Bezier: control, end.
    QuadTo(f64, f64, f64, f64),
    /// Cubic Bezier: two controls, end.
    CubicTo(f64, f64, f64, f64, f64, f64),
    /// Close the subpath.
    Close,
}

/// An 8-bit sRGB colour with straight alpha.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgba {
    /// Red.
    pub r: u8,
    /// Green.
    pub g: u8,
    /// Blue.
    pub b: u8,
    /// Alpha, 255 opaque.
    pub a: u8,
}

/// A gradient stop: an offset from 0 to 1, and its colour.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stop {
    /// Position along the gradient, 0 to 1.
    pub offset: f64,
    /// Colour.
    pub color: Rgba,
}

/// What fills a path.
#[derive(Debug, Clone, PartialEq)]
pub enum Paint {
    /// One colour.
    Solid(Rgba),
    /// A linear gradient from `start` to `end`, in the item's space.
    Linear {
        /// Where offset 0 lies.
        start: (f64, f64),
        /// Where offset 1 lies.
        end: (f64, f64),
        /// At least one stop, in offset order.
        stops: Vec<Stop>,
    },
    /// A radial gradient: offset 0 at `center`, 1 at `radius` from it.
    Radial {
        /// Centre.
        center: (f64, f64),
        /// Radius, EMU.
        radius: f64,
        /// At least one stop, in offset order.
        stops: Vec<Stop>,
    },
    /// A picture, stretched over `frame` in the item's space, which a crop may make
    /// larger than the path. `part` is the media part's package path: the bytes stay
    /// with the host, as in OpenCalc, and a backend that cannot get them says so.
    Picture {
        /// The media part, e.g. `ppt/media/image1.png`.
        part: String,
        /// Where the whole picture lands: `(x, y, w, h)`.
        frame: (f64, f64, f64, f64),
        /// Opacity, 0 to 1.
        alpha: f64,
    },
}

/// How a stroke's segments end.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cap {
    /// Flat at the end point.
    Butt,
    /// Rounded.
    Round,
    /// Squared past the end point.
    Square,
}

/// How a stroke's segments meet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Join {
    /// Rounded.
    Round,
    /// Bevelled.
    Bevel,
    /// Mitred, with this limit as a ratio of the width.
    Miter(f64),
}

/// A stroke.
#[derive(Debug, Clone, PartialEq)]
pub struct Stroke {
    /// What fills the stroke. A line's fill in DrawingML is a full fill; the
    /// backend draws any [`Paint`].
    pub paint: Paint,
    /// Width, EMU; 0 is the thinnest the device draws.
    pub width: f64,
    /// Dash and gap lengths, EMU, alternating. Empty is solid.
    pub dash: Vec<f64>,
    /// Ends.
    pub cap: Cap,
    /// Corners.
    pub join: Join,
}

/// One positioned glyph in a [`DisplayItem::GlyphRun`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Glyph {
    /// The glyph id in its face.
    pub id: u32,
    /// Pen position of its origin, in the item's space, EMU.
    pub x: f64,
    /// Baseline, in the item's space, EMU.
    pub y: f64,
}

/// Which painter's-order layer an item belongs to, for a report or a viewer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Layer {
    /// The slide background.
    Background,
    /// A shape from the master.
    Master,
    /// A shape from the layout.
    Layout,
    /// A shape on the slide.
    Slide,
}

/// One paint instruction.
#[derive(Debug, Clone, PartialEq)]
pub enum DisplayItem {
    /// A filled and, or, stroked path: every shape, connector and picture.
    Path {
        /// The shape it draws, so an editor can invalidate per object.
        object: Option<ObjectId>,
        /// Where it came from.
        layer: Layer,
        /// The item's space onto the slide.
        transform: Affine,
        /// The outline.
        path: Vec<PathCmd>,
        /// The fill, if any, by the non-zero rule.
        fill: Option<Paint>,
        /// The stroke, if any.
        stroke: Option<Stroke>,
    },
    /// Shaped text. Filled by the text crate (lane B, `casual-slide-text`); until
    /// it lands, no item of this kind is built, and the render report counts the
    /// text bodies left undrawn.
    GlyphRun {
        /// The shape whose text this is.
        object: Option<ObjectId>,
        /// Where it came from.
        layer: Layer,
        /// The item's space onto the slide.
        transform: Affine,
        /// The face, as the text crate names it.
        face: String,
        /// Em size, EMU.
        size: f64,
        /// The glyphs.
        glyphs: Vec<Glyph>,
        /// Their fill.
        fill: Paint,
        /// The characters these glyphs draw, so a backend that keeps text (PDF)
        /// can give it back to a reader. Document data: never logged or reported.
        text: String,
    },
}

/// A slide's paint instructions, back to front, and its size.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DisplayList {
    /// Slide width, EMU.
    pub width: f64,
    /// Slide height, EMU.
    pub height: f64,
    /// The items, back to front.
    pub items: Vec<DisplayItem>,
}
