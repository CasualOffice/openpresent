//! Ink as plain typed values (`docs/spec/INK.md`, W3C InkML): the part a `p:contentPart`
//! names, `inkml:ink`, with `msink:context` inside `inkml:annotationXML`, and the
//! recorded `iact:actions`.
//!
//! A stroke's `inkml:trace` holds its points as the file wrote them: nothing here decodes
//! them, since drawing is Tier 2I's. `mc:Fallback`'s picture is a shape on the slide, so it
//! keeps its own bytes and is never regenerated here.

pub mod schema;

use crate::diagram::SetAside;
use crate::properties::Lexical;
use schema::{BrushProperty, Ink2016ExtendedBrushPropertyName, Ink2016InkEffectsType};

/// The 2016 ink schema declares its names and values and no element that carries them:
/// they arrive as InkML `brushProperty` text, which stays as written (`T1K-04`). These
/// read that text as the types, without changing it.
impl BrushProperty {
    /// `@name` as one of 2016's extended names, if it is one.
    pub fn extended_name(&self) -> Option<Ink2016ExtendedBrushPropertyName> {
        self.name
            .as_deref()
            .and_then(Ink2016ExtendedBrushPropertyName::parse_lexical)
    }

    /// `@value` as an ink effect, when `@name` is `inkEffects` and the value is one.
    pub fn ink_effect(&self) -> Option<Ink2016InkEffectsType> {
        match self.extended_name()? {
            Ink2016ExtendedBrushPropertyName::InkEffects => self
                .value
                .as_deref()
                .and_then(Ink2016InkEffectsType::parse_lexical),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn property(name: &str, value: &str) -> BrushProperty {
        BrushProperty {
            name: Some(name.to_owned()),
            value: Some(value.to_owned()),
            ..BrushProperty::default()
        }
    }

    #[test]
    fn an_extended_brush_property_is_read_as_its_type() {
        let p = property("inkEffects", "galaxy");
        assert_eq!(
            p.extended_name(),
            Some(Ink2016ExtendedBrushPropertyName::InkEffects)
        );
        assert_eq!(p.ink_effect(), Some(Ink2016InkEffectsType::Galaxy));
        let p = property("scaleFactor", "galaxy");
        assert_eq!(
            p.extended_name(),
            Some(Ink2016ExtendedBrushPropertyName::ScaleFactor)
        );
        assert_eq!(p.ink_effect(), None, "only inkEffects names an effect");
    }

    #[test]
    fn a_name_or_value_outside_2016_s_lists_is_none() {
        assert_eq!(property("width", "1").extended_name(), None);
        assert_eq!(property("inkEffects", "glitter").ink_effect(), None);
        assert_eq!(BrushProperty::default().ink_effect(), None);
    }
}

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
