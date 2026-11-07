//! Equations as plain typed values (`docs/spec/MATH.md`): OMML as PowerPoint holds it,
//! in `a14:m` in a paragraph.
//!
//! An equation has a fallback for readers that do not know `a14`: a whole `p:sp` around
//! the paragraph, or `a:r` runs where the `mc:AlternateContent` stands in the paragraph.
//! Both are kept beside the equation and neither is drawn or regenerated here.
//! Laying out an equation is Tier 2K's.

pub mod schema;

use crate::text::TextItem;

/// `a14:CT_TextMath`. The extension's schema says one `m:oMath`; Office writes
/// `m:oMathPara` here too, so both are read, in order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextMath {
    /// What the `a14:m` holds, in file order.
    pub items: Vec<MathItem>,
}

/// One child of `a14:m`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MathItem {
    /// `m:oMath`.
    OMath(Box<schema::OMath>),
    /// `m:oMathPara`.
    OMathPara(Box<schema::OMathPara>),
}

/// An equation among a paragraph's runs. A bare `a14:m` has no `alternate`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InlineMath {
    /// `a14:m`. `None` for a choice with no equation in it.
    pub math: Option<TextMath>,
    /// The `mc:AlternateContent` around it.
    pub alternate: Option<MathAlternate>,
}

/// `mc:AlternateContent` in a paragraph.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MathAlternate {
    /// `mc:Choice/@Requires`, as written.
    pub requires: Option<String>,
    /// `mc:Fallback`'s runs, in order. `None` for no `mc:Fallback`.
    pub fallback: Option<Vec<TextItem>>,
}
