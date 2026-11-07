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
mod mce;
mod namespaces;
mod part_name;
mod rels;
mod xml;

pub use content_types::ContentTypes;
pub use error::OoxmlError;
pub use graph::{Conformance, Finding, FindingKind, PresentationKind, PresentationPackage};
pub use limits::OoxmlLimits;
pub use mce::{
    AlternateResolution, Branch, Compatibility, NonConformance, resolve_compatibility, resolve_with,
};
pub use namespaces::{
    MARKUP_COMPATIBILITY, STRICT_TO_TRANSITIONAL, is_understood, relationship_type_to_strict,
    relationship_type_to_transitional, to_strict, to_transitional, understood_namespaces,
};
pub use part_name::{part_name_key, part_names_equivalent};
pub use rels::{ExternalKind, Relationship, TargetMode, classify_external, resolve_internal};
pub use xml::{ElementId, XmlTree, walk};
