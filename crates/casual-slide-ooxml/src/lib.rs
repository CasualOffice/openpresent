//! `casual-slide-ooxml` — the PresentationML package: what is in it, what points
//! at what, and which of it deserves suspicion.
//!
//! No model mapping lives here. Every part is hostile input, so all XML goes
//! through one bounded walker, and external targets are classified and never
//! followed. See `docs/28-PACKAGE-AND-PART-GRAPH.md` and
//! `docs/07-SECURITY-THREAT-MODEL.md`.

mod content_types;
mod error;
mod graph;
mod limits;
mod rels;
mod xml;

pub use content_types::ContentTypes;
pub use error::OoxmlError;
pub use graph::{Conformance, Finding, FindingKind, PresentationKind, PresentationPackage};
pub use limits::OoxmlLimits;
pub use rels::{ExternalKind, Relationship, TargetMode, classify_external};
pub use xml::walk;
