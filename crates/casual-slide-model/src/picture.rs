//! What a picture holds beyond its `a:CT_Blip` (`docs/spec/PIC.md`).
//!
//! A reference here is a relationship id and nothing else. Nothing is decoded,
//! fetched, parsed or placed: an SVG is the opaque target of [`SvgBlip`], and a URL in
//! [`OEmbed`] is text that nothing follows (`SEC-C03`, `SEC-N01`).
//!
//! `MS-OI29500` v20260818, for what is drawn in Phase 2 and changes no stored value:
//! 21.3.2.2 Office does not support `blipFill/@dpi`; 20.1.2.2.31 Office ignores
//! `noChangeArrowheads`, `noChangeShapeType`, `noEditPoints` and `noRot` on a picture,
//! and PowerPoint `noAdjustHandles`. `MS-ODRAWXML` "Pictures": the main blip holds a PNG
//! rasterized from the SVG, for backward compatibility.

/// `asvg:CT_SVGBlip`, from `a:blip/a:extLst`. Its fallback is the `a:blip` it sits in.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SvgBlip {
    /// `@r:embed`.
    pub embed: Option<String>,
    /// `@r:link`. Never followed.
    pub link: Option<String>,
}

/// `woe:CT_OEmbed`, online content a picture stands for. Its placement under
/// `a:blip/a:extLst` is Inferred: `MS-ODRAWXML` names the extension and not its parent,
/// and no deck here has one.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OEmbed {
    /// `@oEmbedUrl`. Never fetched, and reported.
    pub url: Option<String>,
    /// `@mediaType`.
    pub media_type: Option<String>,
    /// `@picLocksAutoForOEmbed`.
    pub pic_locks_auto: Option<bool>,
    /// The `uri` of the `a:ext` holding it. No document gives the one to write, so it is
    /// kept with the value, and only a file can supply one.
    pub ext_uri: Option<String>,
}

/// `aif:CT_ImageFormula`. Its placement under `a:blip/a:extLst` is Inferred, as for
/// [`OEmbed`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImageFormula {
    /// `@formula`.
    pub formula: Option<String>,
    /// The `uri` of the `a:ext` holding it, as for [`OEmbed::ext_uri`].
    pub ext_uri: Option<String>,
}
