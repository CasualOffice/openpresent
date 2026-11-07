//! Package admission errors, each carrying a stable diagnostic code.
//! See `docs/20-ERROR-CODE-REGISTRY.md`.

use core::fmt;

/// A reason a package was refused, or a part could not be read. Every variant
/// maps to a stable `CS-PKG-*` code via [`PackageError::code`]. All refusals are
/// *clean*: on any error the package is not partially admitted.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PackageError {
    /// Input exceeds `max_input_bytes`.
    InputTooLarge { size: u64, limit: u64 },
    /// Entry count exceeds `max_entries`.
    TooManyEntries { count: usize, limit: usize },
    /// Total uncompressed size exceeds `max_total_uncompressed`.
    ExpansionTooLarge { total: u64, limit: u64 },
    /// Uncompressed:compressed ratio exceeds `max_expansion_ratio`.
    ExpansionRatioExceeded { ratio: u64, limit: u64 },
    /// An entry path is longer than `max_path_bytes`.
    PathTooLong { len: usize, limit: usize },
    /// An entry path is unsafe (traversal, absolute, or drive-qualified).
    UnsafePath { path: String },
    /// Two entries name the same part, exactly or by OPC's case-insensitive
    /// equivalence.
    DuplicatePart { path: String },
    /// A part declares more bytes than its reader accepts.
    PartTooLarge { name: String, size: u64, limit: u64 },
    /// A new package could not be written.
    RewriteFailed,
    /// The input is not a valid ZIP/OPC package.
    NotAPackage,
    /// A requested part does not exist in the package.
    PartNotFound { name: String },
    /// The host's time budget stopped a part's decompression (`SEC-P10`).
    Cancelled { name: String },
}

impl PackageError {
    /// The stable diagnostic code for this error (`docs/20`).
    pub fn code(&self) -> &'static str {
        match self {
            PackageError::InputTooLarge { .. } => "CS-PKG-0001",
            PackageError::TooManyEntries { .. } => "CS-PKG-0002",
            PackageError::ExpansionTooLarge { .. }
            | PackageError::ExpansionRatioExceeded { .. } => "CS-PKG-0003",
            PackageError::PathTooLong { .. } | PackageError::UnsafePath { .. } => "CS-PKG-0004",
            PackageError::NotAPackage => "CS-PKG-0005",
            PackageError::PartNotFound { .. } => "CS-PKG-0006",
            PackageError::DuplicatePart { .. } => "CS-PKG-0007",
            PackageError::RewriteFailed => "CS-PKG-0008",
            PackageError::PartTooLarge { .. } => "CS-PKG-0009",
            PackageError::Cancelled { .. } => "CS-PKG-0010",
        }
    }
}

impl fmt::Display for PackageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] ", self.code())?;
        match self {
            PackageError::InputTooLarge { size, limit } => {
                write!(f, "input too large: {size} bytes exceeds limit {limit}")
            }
            PackageError::TooManyEntries { count, limit } => {
                write!(f, "too many entries: {count} exceeds limit {limit}")
            }
            PackageError::ExpansionTooLarge { total, limit } => {
                write!(
                    f,
                    "expanded size too large: {total} bytes exceeds limit {limit}"
                )
            }
            PackageError::ExpansionRatioExceeded { ratio, limit } => {
                write!(
                    f,
                    "expansion ratio too high: {ratio}:1 exceeds limit {limit}:1"
                )
            }
            PackageError::PathTooLong { len, limit } => {
                write!(f, "entry path too long: {len} bytes exceeds limit {limit}")
            }
            PackageError::UnsafePath { path } => write!(f, "unsafe entry path: {path:?}"),
            PackageError::DuplicatePart { path } => write!(f, "duplicate part name: {path:?}"),
            PackageError::PartTooLarge { name, size, limit } => {
                write!(
                    f,
                    "part {name:?} declares {size} bytes, over the limit {limit}"
                )
            }
            PackageError::RewriteFailed => write!(f, "could not write the package"),
            PackageError::NotAPackage => write!(f, "not a valid ZIP/OPC package"),
            PackageError::PartNotFound { name } => write!(f, "part not found: {name:?}"),
            PackageError::Cancelled { name } => {
                write!(f, "reading part {name:?} stopped by the time budget")
            }
        }
    }
}

impl std::error::Error for PackageError {}
