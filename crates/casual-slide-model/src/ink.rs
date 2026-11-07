//! Ink as plain typed values (`docs/spec/INK.md`, W3C InkML): the part a `p:contentPart`
//! names, `inkml:ink`, with `msink:context` inside `inkml:annotationXML`, and the
//! recorded `iact:actions`.
//!
//! A stroke's `inkml:trace` holds its points as the file wrote them: nothing here decodes
//! them, since drawing is Tier 2I's. `mc:Fallback`'s picture is a shape on the slide, so it
//! keeps its own bytes and is never regenerated here.

pub mod schema;

use crate::diagram::SetAside;

/// One ink part as read: its value, or why it was set aside.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InkPart {
    /// The part, as the archive spells it.
    pub part: String,
    /// What it holds. `None` if it was set aside.
    pub root: Option<schema::Root>,
    /// Set aside, with why. The part stays in the package untouched.
    pub set_aside: Option<SetAside>,
}
