//! `casual-slide-import`. See docs/19 and docs/28 (ADR-013).

// rustdoc proves auto traits through the generated equation types, which nest deeper than the
// default limit of 128 allows (E0275 on `math::schema`). Compilation itself never needs it.
#![recursion_limit = "256"]

pub mod chart;
pub mod color;
pub mod comment;
pub mod custom_data;
pub mod d3;
pub mod diagram;
pub mod drawing;
pub mod effect;
pub mod fill;
pub mod geometry;
pub mod ink;
pub mod line;
pub mod math;
pub mod media;
pub mod presentation;
pub mod properties;
pub mod retained;
#[rustfmt::skip]
mod schema_names;
pub mod section;
pub mod shape;
pub mod slide;
pub mod table;
pub mod text;
pub mod theme;
pub mod timing;
pub mod transition;
