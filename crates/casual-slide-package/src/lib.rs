//! `casual-slide-package` — format-neutral, security-bounded ZIP/OPC package
//! admission and part reads.
//!
//! This is the substrate every OPC-based format sits on: it admits an archive
//! under explicit, non-bypassable [`PackageLimits`] (size, entry count,
//! expansion ratio, path safety), then exposes on-demand, size-capped part
//! reads. It backs `.pptx` through `casual-slide-ooxml`.
//!
//! The design is OpenCalc's `docs/28-XLSX-PACKAGE-READER.md` and
//! `docs/21-PARSER-LIMITS.md`; see `PROVENANCE.md`.

mod directory;
mod error;
mod limits;
mod package;
mod path;

pub use error::PackageError;
pub use limits::PackageLimits;
pub use package::{EntryInfo, Package, PartEdits};
pub use path::is_safe_part_path;

#[cfg(test)]
mod tests;
