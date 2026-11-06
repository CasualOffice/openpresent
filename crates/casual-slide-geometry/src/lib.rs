//! Preset and custom geometry to resolved paths. See docs/19, GEO-01, SEC-C08,
//! BEH-07.
//!
//! Input is a [`GeometryDef`], from the generated [`preset_shape`] and [`text_warp`]
//! tables or from [`RawGeometry::compile`], plus shape extents in EMU and adjust
//! values. [`GeometryDef::evaluate`] returns paths, the text rectangle, connection
//! sites and adjust handles.
//!
//! Custom geometry comes from hostile files, so:
//!
//! * Guides evaluate in one forward pass. Operands resolve at compile time to earlier
//!   slots, so cycles cannot be written down and nothing recurses.
//! * Arithmetic saturates at [`math::VALUE_LIMIT`]; a result with no defined value,
//!   such as a division by zero, is 0; NaN cannot survive an operation.
//! * [`limits`] caps guides, paths, points, handles and sites at compile time.
//! * Results repeat bit for bit across targets; [`math`] says how.
//!
//! Positions come out in `f64` EMU, unrounded: rounding to device or file units
//! belongs to the caller (BEH-22). The crate depends on nothing, parses no XML and
//! compiles to `wasm32-unknown-unknown`.

mod compile;
mod def;
mod diagnostic;
mod eval;
#[rustfmt::skip]
mod generated;
pub mod math;

pub use compile::{
    CompiledGeometry, GeometryError, RawAxis, RawCommand, RawGeometry, RawGuide, RawHandle,
    RawPath, RawPoint, RawRect, RawSite,
};
pub use def::{
    Arg, AxisDef, Cmd, FillMode, GeometryDef, GuideDef, HandleDef, Op, PathDef, PtDef, RectDef,
    SiteDef,
};
pub use diagnostic::{Diagnostic, Diagnostics};
pub use eval::{
    Adjust, AdjustHandle, AdjustValue, Arc, ConnectionSite, Extents, HandleAxis, Point,
    ResolvedGeometry, ResolvedPath, Segment, TextRect, apply,
};

/// Size limits on custom geometry, enforced by [`RawGeometry::compile`]. The largest
/// preset has 244 guides; real freeforms have many more points than presets do,
/// which is why that cap is the generous one.
pub mod limits {
    /// `av` plus `gd` guides.
    pub const MAX_GUIDES: usize = 4096;
    /// Paths.
    pub const MAX_PATHS: usize = 1024;
    /// Points over all paths.
    pub const MAX_PATH_POINTS: usize = 262_144;
    /// Adjust handles.
    pub const MAX_HANDLES: usize = 256;
    /// Connection sites.
    pub const MAX_SITES: usize = 4096;
    /// Bytes in one formula.
    pub const MAX_FORMULA_LEN: usize = 1024;
    /// Bytes in one guide name.
    pub const MAX_NAME_LEN: usize = 256;
}

fn find(table: &'static [GeometryDef], name: &str) -> Option<&'static GeometryDef> {
    let index = table
        .binary_search_by(|def| def.name.as_ref().cmp(name))
        .ok()?;
    table.get(index)
}

/// A preset shape by its `ST_ShapeType` token, case-sensitive as in the schema.
pub fn preset_shape(name: &str) -> Option<&'static GeometryDef> {
    find(&generated::PRESET_SHAPES, name)
}

/// A text warp by its `ST_TextShapeType` token.
pub fn text_warp(name: &str) -> Option<&'static GeometryDef> {
    find(&generated::TEXT_WARPS, name)
}

/// All preset shapes, sorted by name.
pub fn preset_shapes() -> &'static [GeometryDef] {
    &generated::PRESET_SHAPES
}

/// All text warps, sorted by name.
pub fn text_warps() -> &'static [GeometryDef] {
    &generated::TEXT_WARPS
}

/// Names of the built-in guides, in slot order: the slot is the index here.
pub fn builtin_guide_names() -> &'static [&'static str] {
    &generated::BUILTIN_ORDER
}

/// One number for the whole table: every preset and warp at a fixed set of sizes,
/// through [`ResolvedGeometry::fingerprint`]. A test pins it, and the same value on
/// another target is the evidence that results reproduce there.
pub fn table_fingerprint() -> u64 {
    const SIZES: [(i64, i64); 4] = [
        (914_400, 914_400),
        (1_200_000, 800_000),
        (333, 7_777_777),
        (12_192_000, 6_858_000),
    ];
    let mut combined = 0xcbf2_9ce4_8422_2325u64;
    for def in preset_shapes().iter().chain(text_warps()) {
        for (w, h) in SIZES {
            let value = def.evaluate(Extents::new(w, h), &[]).fingerprint();
            combined = (combined ^ value).wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    combined
}
