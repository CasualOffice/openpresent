//! Objects kept as bytes and never opened (`SEC-C07`, Tier 1L): an OLE object, an ActiveX
//! control, a 3D model. Only who an object is and which image stands in for it are
//! modelled. Everything else stays in the file, and nothing here is ever followed.

/// What kind of retained object a node stands for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetainedKind {
    /// `p:oleObj` with `p:embed`, or with neither stated.
    OleEmbedded,
    /// `p:oleObj` with `p:link`: the source is outside the file, and is never fetched.
    OleLinked,
    /// `p:control`, an ActiveX control. Never instantiated.
    Control,
    /// `am3d:model3d`, a 3D model. Never decoded.
    Model3d,
}

/// Identity and fallback of a retained object: `CT_OleObject` and `CT_Control`, through
/// `AG_Ole`, and `CT_Model3D`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetainedObject {
    /// Which kind of object.
    pub kind: RetainedKind,
    /// `@spid`: the shape in the legacy VML drawing part that draws it.
    pub spid: Option<String>,
    /// `@name`.
    pub display_name: Option<String>,
    /// `@progId`: names the handler. A label only: nothing is ever started from it.
    pub prog_id: Option<String>,
    /// `@showAsIcon`.
    pub show_as_icon: Option<bool>,
    /// `@imgW`.
    pub img_w: Option<i64>,
    /// `@imgH`.
    pub img_h: Option<i64>,
    /// `@r:id` of an object or control, or `@r:embed` of a model: the retained bytes.
    pub rel_id: Option<String>,
    /// The relationship of the image drawn in its place: `pic`'s `a:blip/@r:embed`, a
    /// model's `am3d:raster/am3d:blip/@r:embed`, or the picture in the `mc:Fallback` beside it.
    pub fallback_rel_id: Option<String>,
}

impl RetainedObject {
    /// An object of a kind, with nothing stated.
    pub fn new(kind: RetainedKind) -> Self {
        Self {
            kind,
            spid: None,
            display_name: None,
            prog_id: None,
            show_as_icon: None,
            img_w: None,
            img_h: None,
            rel_id: None,
            fallback_rel_id: None,
        }
    }
}
