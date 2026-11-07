//! Reads theme parts, and colour maps wherever they live, into the model.
//!
//! What a theme holds beyond the model stays in the tree. A format scheme's fills, lines
//! and effects, and an object default's `spPr`, `bodyPr` and `lstStyle`, go through the
//! readers a shape's do, and report what they notice here. Elements unknown to the schema
//! are reported as `CS-CLR-0009`.

use std::collections::HashSet;

use casual_slide_model::drawing::ShapeProperties;
use casual_slide_model::fill::Fill;
use casual_slide_model::line::Line;
use casual_slide_model::theme::{
    BaseStyles, ClipboardStyleSheet, ColorMap, ColorMapOverride, ColorMapSlot, ColorScheme,
    ColorSchemeIndex, CustomColor, EffectReference, ExtraColorScheme, FontCollection,
    FontCollectionIndex, FontRef, FontScheme, FormatScheme, ObjectDefaults, ShapeDefinition,
    ShapeStyle, StyleEffect, StyleMatrixRef, SupplementalFont, TextFont, Theme, ThemeFamily,
    ThemeManager, ThemeOverride, ThemeableFont, ThemeableStyle,
};
use casual_slide_ooxml::{
    ElementId, OoxmlError, OoxmlLimits, PresentationPackage, Relationship, XmlTree,
};

use crate::color::{
    FindingKind, Findings, bounded_children, dml_name, is_color_element, read_color_choice,
    read_optional_color, report_unread_attributes, report_unread_children,
};
use crate::drawing::{
    Ctx, DrawingFinding, DrawingLimits, is_fill_element, note_dag_siblings, read_fill, read_line,
};
use crate::properties::{
    Elem, Findings as PropertyFindings, ImportError, ImportFindingKind as K, ImportLimits,
    Reader as PropertyReader, ns,
};
use crate::text::{TextCounts, TextReader};

/// The Office 2013 theme extension's namespace.
pub const THEME_ML_2012: &str = "http://schemas.microsoft.com/office/thememl/2012/main";
/// A theme part's content type.
pub const THEME_CONTENT_TYPE: &str = "application/vnd.openxmlformats-officedocument.theme+xml";
/// A theme override part's content type.
pub const THEME_OVERRIDE_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.themeOverride+xml";

/// A part read, with what was noticed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Imported<T> {
    /// The value.
    pub value: T,
    /// What was noticed.
    pub findings: Findings,
}

/// Why a part is not the thing asked for. Anything else is a finding.
#[derive(Debug)]
pub enum ThemeImportError {
    /// The XML was refused, or the part could not be read.
    Xml(OoxmlError),
    /// The root is not the expected element in DrawingML.
    WrongRoot {
        /// What was expected.
        expected: &'static str,
    },
}

impl core::fmt::Display for ThemeImportError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Xml(err) => write!(f, "{err}"),
            Self::WrongRoot { expected } => write!(f, "root element is not {expected}"),
        }
    }
}

impl std::error::Error for ThemeImportError {}

impl From<OoxmlError> for ThemeImportError {
    fn from(err: OoxmlError) -> Self {
        Self::Xml(err)
    }
}

fn expect_root(tree: &XmlTree, expected: &'static str) -> Result<ElementId, ThemeImportError> {
    let root = tree.root();
    if dml_name(tree, root) == Some(expected) {
        Ok(root)
    } else {
        Err(ThemeImportError::WrongRoot { expected })
    }
}

/// Every theme and theme override part in a package by declared content type,
/// sorted by name, with each one's tree parsed under the package's limits.
pub fn read_theme_parts(
    pkg: &mut PresentationPackage,
) -> Result<Vec<(String, XmlTree)>, ThemeImportError> {
    let mut names: Vec<String> = pkg
        .part_names()
        .into_iter()
        .filter(|name| {
            matches!(
                pkg.content_types().resolve(name),
                Some(THEME_CONTENT_TYPE | THEME_OVERRIDE_CONTENT_TYPE)
            )
        })
        .collect();
    names.sort_unstable();
    let limits: OoxmlLimits = *pkg.limits();
    let mut out = Vec::with_capacity(names.len());
    for name in names {
        let bytes = pkg.read_xml_part(&name)?;
        let tree = XmlTree::parse(&bytes, &name, &limits)?;
        out.push((name, tree));
    }
    Ok(out)
}

/// Read a `theme` part under the default limits.
pub fn import_theme(tree: &XmlTree) -> Result<Imported<Theme>, ThemeImportError> {
    import_theme_limited(tree, &OoxmlLimits::default())
}

/// Read a `theme` part under these limits, with its relationships not to hand: a picture
/// in a format scheme is read, and what it names is not checked.
pub fn import_theme_limited(
    tree: &XmlTree,
    limits: &OoxmlLimits,
) -> Result<Imported<Theme>, ThemeImportError> {
    read_theme(tree, None, limits)
}

/// Read a `theme` part with its relationships, so a picture in a format scheme that names
/// none of them is reported.
pub fn import_theme_with_rels(
    tree: &XmlTree,
    rels: &[Relationship],
    limits: &OoxmlLimits,
) -> Result<Imported<Theme>, ThemeImportError> {
    read_theme(tree, Some(rels), limits)
}

fn read_theme(
    tree: &XmlTree,
    rels: Option<&[Relationship]>,
    limits: &OoxmlLimits,
) -> Result<Imported<Theme>, ThemeImportError> {
    // spec-audit: a:<theme>
    let root = expect_root(tree, "theme")?;
    let mut findings = Findings::with_limits(limits);
    let mut value = Reader {
        tree,
        findings: &mut findings,
    }
    .theme(root);
    let scheme = value
        .elements
        .as_mut()
        .and_then(|e| e.format_scheme.as_mut());
    read_by_other_tiers(
        tree,
        rels,
        scheme,
        value.object_defaults.as_mut(),
        &mut findings,
    );
    Ok(Imported { value, findings })
}

/// Read a `themeOverride` part.
pub fn import_theme_override(tree: &XmlTree) -> Result<Imported<ThemeOverride>, ThemeImportError> {
    read_theme_override(tree, None)
}

/// Read a `themeOverride` part with its relationships.
pub fn import_theme_override_with_rels(
    tree: &XmlTree,
    rels: &[Relationship],
) -> Result<Imported<ThemeOverride>, ThemeImportError> {
    read_theme_override(tree, Some(rels))
}

// spec-audit: owner=themeOverride
fn read_theme_override(
    tree: &XmlTree,
    rels: Option<&[Relationship]>,
) -> Result<Imported<ThemeOverride>, ThemeImportError> {
    // spec-audit: a:<themeOverride>
    let root = expect_root(tree, "themeOverride")?;
    let mut findings = Findings::default();
    let mut r = Reader {
        tree,
        findings: &mut findings,
    };
    let mut value = ThemeOverride::default();
    r.strangers(root, &[]);
    for child in r.children(root) {
        match dml_name(tree, child) {
            Some("clrScheme") => {
                r.once(&mut value.color_scheme, child, Reader::color_scheme);
            }
            Some("fontScheme") => {
                r.once(&mut value.font_scheme, child, Reader::font_scheme);
            }
            Some("fmtScheme") => {
                r.once(&mut value.format_scheme, child, Reader::format_scheme);
            }
            _ => r.unknown(child),
        }
    }
    read_by_other_tiers(
        tree,
        rels,
        value.format_scheme.as_mut(),
        None,
        &mut findings,
    );
    Ok(Imported { value, findings })
}

/// Read a `themeManager` part, which by its schema holds nothing. Whatever it does hold
/// stays in the part, and is reported.
// spec-audit: owner=themeManager
pub fn import_theme_manager(tree: &XmlTree) -> Result<Imported<ThemeManager>, ThemeImportError> {
    // spec-audit: a:<themeManager>
    let root = expect_root(tree, "themeManager")?;
    let mut findings = Findings::default();
    let mut r = Reader {
        tree,
        findings: &mut findings,
    };
    r.strangers(root, &[]);
    for child in r.children(root) {
        r.unknown(child);
    }
    Ok(Imported {
        value: ThemeManager,
        findings,
    })
}

/// Read `CT_EffectReference`, an `a:effect` that names an effect. A missing `@ref` is
/// reported and left absent.
pub fn read_effect_reference(
    tree: &XmlTree,
    el: ElementId,
    findings: &mut Findings,
) -> EffectReference {
    let mut r = Reader { tree, findings };
    let reference = if r.readable(el) {
        r.required(el, "ref")
    } else {
        None
    };
    EffectReference { reference }
}

/// Read a `CT_ClipboardStyleSheet` root, by its schema shape.
// spec-audit: owner=objectDefaults
pub fn import_clipboard_style_sheet(tree: &XmlTree) -> Imported<ClipboardStyleSheet> {
    let mut findings = Findings::default();
    let mut r = Reader {
        tree,
        findings: &mut findings,
    };
    let mut value = ClipboardStyleSheet::default();
    for child in r.children(tree.root()) {
        match dml_name(tree, child) {
            Some("themeElements") => r.once(&mut value.elements, child, Reader::base_styles),
            Some("clrMap") => r.once(&mut value.map, child, Reader::color_map),
            _ => r.unknown(child),
        }
    }
    let scheme = value
        .elements
        .as_mut()
        .and_then(|e| e.format_scheme.as_mut());
    read_by_other_tiers(tree, None, scheme, None, &mut findings);
    Imported { value, findings }
}

/// Read `CT_ColorMapping` from any element with that shape: `p:clrMap` on a
/// master, `a:overrideClrMapping`, `a:clrMap`.
pub fn read_color_map(tree: &XmlTree, el: ElementId, findings: &mut Findings) -> ColorMap {
    Reader { tree, findings }.color_map(el)
}

/// Read `CT_ColorMappingOverride`. `None`, reported, if it holds neither choice.
pub fn read_color_map_override(
    tree: &XmlTree,
    el: ElementId,
    findings: &mut Findings,
) -> Option<ColorMapOverride> {
    let mut r = Reader { tree, findings };
    let mut value = None;
    r.strangers(el, &[]);
    for child in r.children(el) {
        let this = match dml_name(tree, child) {
            Some("masterClrMapping") => {
                r.strangers(child, &[]);
                report_unread_children(tree, child, r.findings);
                ColorMapOverride::Master
            }
            Some("overrideClrMapping") => ColorMapOverride::Override(r.color_map(child)),
            _ => {
                r.unknown(child);
                continue;
            }
        };
        if value.is_some() {
            r.note(FindingKind::Duplicate, child, "");
        } else {
            value = Some(this);
        }
    }
    if value.is_none() {
        r.note(
            FindingKind::MissingRequired,
            el,
            "masterClrMapping or overrideClrMapping",
        );
    }
    value
}

/// Read `CT_ShapeStyle`.
pub fn read_shape_style(tree: &XmlTree, el: ElementId, findings: &mut Findings) -> ShapeStyle {
    Reader { tree, findings }.shape_style(el)
}

/// Read `CT_StyleMatrixReference`.
pub fn read_style_matrix_ref(
    tree: &XmlTree,
    el: ElementId,
    findings: &mut Findings,
) -> StyleMatrixRef {
    Reader { tree, findings }.matrix_ref(el)
}

/// Read `CT_FontReference`.
pub fn read_font_ref(tree: &XmlTree, el: ElementId, findings: &mut Findings) -> FontRef {
    Reader { tree, findings }.font_ref(el)
}

/// Read `EG_ThemeableFillStyle` or `EG_ThemeableEffectStyle` from their holder.
// spec-audit: owner=tblBg
pub fn read_themeable_style(
    tree: &XmlTree,
    holder: ElementId,
    findings: &mut Findings,
) -> Option<ThemeableStyle> {
    let mut r = Reader { tree, findings };
    for child in r.children(holder) {
        match dml_name(tree, child) {
            Some("fill" | "effect") => return Some(ThemeableStyle::Inline),
            Some("fillRef" | "effectRef") => {
                return Some(ThemeableStyle::Ref(r.matrix_ref(child)));
            }
            _ => {}
        }
    }
    None
}

/// Read `EG_ThemeableFontStyles` from its holder.
// spec-audit: owner=tcTxStyle
pub fn read_themeable_font(
    tree: &XmlTree,
    holder: ElementId,
    findings: &mut Findings,
) -> Option<ThemeableFont> {
    let mut r = Reader { tree, findings };
    for child in r.children(holder) {
        match dml_name(tree, child) {
            Some("font") => return Some(ThemeableFont::Inline(r.font_collection(child))),
            Some("fontRef") => return Some(ThemeableFont::Ref(r.font_ref(child))),
            _ => {}
        }
    }
    None
}

struct Reader<'a> {
    tree: &'a XmlTree,
    findings: &'a mut Findings,
}

impl Reader<'_> {
    fn note(&mut self, kind: FindingKind, at: ElementId, detail: &'static str) {
        self.findings.push(kind, self.tree, at, detail);
    }

    fn unknown(&mut self, at: ElementId) {
        self.note(FindingKind::Unmodelled, at, "");
    }

    /// Report attributes of `el` other than `known`. They stay in the tree.
    fn strangers(&mut self, el: ElementId, known: &[&str]) {
        report_unread_attributes(self.tree, el, known, self.findings);
    }

    /// Report anything but a colour under `el`, which may hold one and nothing else.
    fn only_colors(&mut self, el: ElementId) {
        for child in self.children(el) {
            if !is_color_element(self.tree, child) {
                self.unknown(child);
            }
        }
    }

    /// An element's children, capped, with the cap reported.
    fn children(&mut self, el: ElementId) -> Vec<ElementId> {
        bounded_children(self.tree, el, self.findings)
    }

    fn once<T>(
        &mut self,
        slot: &mut Option<T>,
        child: ElementId,
        read: fn(&mut Self, ElementId) -> T,
    ) {
        if slot.is_some() {
            self.note(FindingKind::Duplicate, child, "");
        } else {
            *slot = Some(read(self, child));
        }
    }

    /// The attributes of `el` can be read; if not, report it.
    fn readable(&mut self, el: ElementId) -> bool {
        let ok = self.tree.attributes_valid(el);
        if !ok {
            self.note(FindingKind::MalformedAttributes, el, "");
        }
        ok
    }

    fn attr(&self, el: ElementId, name: &str) -> Option<String> {
        self.tree.plain_attribute(el, name)
    }

    fn required(&mut self, el: ElementId, name: &'static str) -> Option<String> {
        let value = self.attr(el, name);
        if value.is_none() {
            self.note(FindingKind::MissingRequired, el, name);
        }
        value
    }

    fn extensions(&mut self, el: ElementId) -> Vec<String> {
        let mut uris = Vec::new();
        for ext in self.children(el) {
            if self.tree.attributes_valid(ext) {
                uris.extend(self.attr(ext, "uri"));
            }
        }
        uris.truncate(self.findings.bounds.list_entries);
        uris
    }

    // spec-audit: owner=theme
    fn theme(&mut self, root: ElementId) -> Theme {
        let mut theme = Theme::default();
        self.strangers(root, &["name"]);
        if self.readable(root) {
            theme.name = self.attr(root, "name");
        }
        for child in self.children(root) {
            match dml_name(self.tree, child) {
                Some("themeElements") => {
                    self.once(&mut theme.elements, child, Self::base_styles);
                }
                Some("objectDefaults") => {
                    self.once(&mut theme.object_defaults, child, Self::object_defaults);
                }
                Some("extraClrSchemeLst") => {
                    self.once(&mut theme.extra_color_schemes, child, Self::extra_schemes);
                }
                Some("custClrLst") => {
                    self.once(&mut theme.custom_colors, child, Self::custom_colors);
                }
                Some("extLst") => {
                    theme.extensions = self.extensions(child);
                    theme.family = self.theme_family(child);
                }
                _ => self.unknown(child),
            }
        }
        if theme.elements.is_none() {
            self.note(FindingKind::MissingRequired, root, "themeElements");
        }
        theme
    }

    // spec-audit: owner=themeFamily
    fn theme_family(&mut self, ext_list: ElementId) -> Option<ThemeFamily> {
        for ext in self.children(ext_list) {
            for inner in self.children(ext) {
                let is_family = self.tree.local_name(inner) == "themeFamily"
                    && self.tree.namespace_uri(inner).as_deref() == Some(THEME_ML_2012);
                if is_family && self.readable(inner) {
                    return Some(ThemeFamily {
                        name: self.attr(inner, "name"),
                        id: self.attr(inner, "id"),
                        vid: self.attr(inner, "vid"),
                    });
                }
            }
        }
        None
    }

    // spec-audit: owner=themeElements
    fn base_styles(&mut self, el: ElementId) -> BaseStyles {
        let mut out = BaseStyles::default();
        self.strangers(el, &[]);
        for child in self.children(el) {
            match dml_name(self.tree, child) {
                Some("clrScheme") => self.once(&mut out.color_scheme, child, Self::color_scheme),
                Some("fontScheme") => self.once(&mut out.font_scheme, child, Self::font_scheme),
                Some("fmtScheme") => self.once(&mut out.format_scheme, child, Self::format_scheme),
                Some("extLst") => out.extensions = self.extensions(child),
                _ => self.unknown(child),
            }
        }
        for (present, name) in [
            (out.color_scheme.is_some(), "clrScheme"),
            (out.font_scheme.is_some(), "fontScheme"),
            (out.format_scheme.is_some(), "fmtScheme"),
        ] {
            if !present {
                self.note(FindingKind::MissingRequired, el, name);
            }
        }
        out
    }

    // spec-audit: owner=clrScheme
    fn color_scheme(&mut self, el: ElementId) -> ColorScheme {
        let mut out = ColorScheme::default();
        self.strangers(el, &["name"]);
        if self.readable(el) {
            out.name = self.required(el, "name");
        }
        let mut seen = [false; 12];
        for child in self.children(el) {
            let Some(name) = dml_name(self.tree, child) else {
                self.unknown(child);
                continue;
            };
            if name == "extLst" {
                out.extensions = self.extensions(child);
            } else if let Some(index) = ColorSchemeIndex::from_xml(name) {
                let at = index as usize;
                self.strangers(child, &[]);
                if core::mem::replace(&mut seen[at], true) {
                    self.note(FindingKind::Duplicate, child, "");
                } else {
                    out.slots[at] = read_color_choice(self.tree, child, self.findings);
                }
            } else {
                self.unknown(child);
            }
        }
        for index in ColorSchemeIndex::ALL {
            if !seen[*index as usize] {
                self.note(FindingKind::MissingRequired, el, index.as_xml());
            }
        }
        out
    }

    // spec-audit: owner=clrMap,overrideClrMapping
    fn color_map(&mut self, el: ElementId) -> ColorMap {
        // The slots come from the model's names.
        // spec-audit: a:CT_ColorMapping/@bg1
        // spec-audit: a:CT_ColorMapping/@tx1
        // spec-audit: a:CT_ColorMapping/@bg2
        // spec-audit: a:CT_ColorMapping/@tx2
        // spec-audit: a:CT_ColorMapping/@accent1
        // spec-audit: a:CT_ColorMapping/@accent2
        // spec-audit: a:CT_ColorMapping/@accent3
        // spec-audit: a:CT_ColorMapping/@accent4
        // spec-audit: a:CT_ColorMapping/@accent5
        // spec-audit: a:CT_ColorMapping/@accent6
        // spec-audit: a:CT_ColorMapping/@hlink
        // spec-audit: a:CT_ColorMapping/@folHlink
        let mut out = ColorMap::default();
        if !self.readable(el) {
            return out;
        }
        let known: Vec<&str> = ColorMapSlot::ALL.iter().map(|s| s.as_xml()).collect();
        self.strangers(el, &known);
        for slot in ColorMapSlot::ALL {
            let at = *slot as usize;
            let name = slot.as_xml();
            let Some(text) = self.required(el, name) else {
                continue;
            };
            out.slots[at] = ColorSchemeIndex::from_xml(&text);
            if out.slots[at].is_none() {
                self.note(FindingKind::BadValue, el, name);
            }
        }
        for child in self.children(el) {
            match dml_name(self.tree, child) {
                Some("extLst") => out.extensions = self.extensions(child),
                _ => self.unknown(child),
            }
        }
        out
    }

    // spec-audit: owner=fontScheme
    fn font_scheme(&mut self, el: ElementId) -> FontScheme {
        let mut out = FontScheme::default();
        self.strangers(el, &["name"]);
        if self.readable(el) {
            out.name = self.required(el, "name");
        }
        for child in self.children(el) {
            match dml_name(self.tree, child) {
                Some("majorFont") => self.once(&mut out.major, child, Self::font_collection),
                Some("minorFont") => self.once(&mut out.minor, child, Self::font_collection),
                Some("extLst") => out.extensions = self.extensions(child),
                _ => self.unknown(child),
            }
        }
        out
    }

    // spec-audit: owner=majorFont,minorFont,font
    fn font_collection(&mut self, el: ElementId) -> FontCollection {
        let mut out = FontCollection::default();
        self.strangers(el, &[]);
        for child in self.children(el) {
            match dml_name(self.tree, child) {
                Some("latin") => self.once(&mut out.latin, child, Self::text_font),
                Some("ea") => self.once(&mut out.ea, child, Self::text_font),
                Some("cs") => self.once(&mut out.cs, child, Self::text_font),
                Some("font") => {
                    if out.fonts.len() >= self.findings.bounds.list_entries {
                        self.note(FindingKind::ListCapped, el, "font");
                        break;
                    }
                    let mut font = SupplementalFont::default();
                    self.strangers(child, &["script", "typeface"]);
                    report_unread_children(self.tree, child, self.findings);
                    if self.readable(child) {
                        font.script = self.required(child, "script");
                        font.typeface = self.required(child, "typeface");
                    }
                    out.fonts.push(font);
                }
                Some("extLst") => out.extensions = self.extensions(child),
                _ => self.unknown(child),
            }
        }
        out
    }

    // spec-audit: owner=latin,ea,cs
    fn text_font(&mut self, el: ElementId) -> TextFont {
        let mut out = TextFont::default();
        self.strangers(el, &["typeface", "panose", "pitchFamily", "charset"]);
        report_unread_children(self.tree, el, self.findings);
        if !self.readable(el) {
            return out;
        }
        out.typeface = self.required(el, "typeface");
        if let Some(text) = self.attr(el, "panose") {
            let good = text.len() == 20 && text.bytes().all(|b| b.is_ascii_hexdigit());
            if good {
                out.panose = Some(text);
            } else {
                self.note(FindingKind::BadValue, el, "panose");
            }
        }
        out.pitch_family = self.small_int(el, "pitchFamily");
        out.charset = self.small_int(el, "charset");
        out
    }

    fn small_int(&mut self, el: ElementId, name: &'static str) -> Option<i8> {
        let text = self.attr(el, name)?;
        let parsed = text.trim().parse().ok();
        if parsed.is_none() {
            self.note(FindingKind::BadValue, el, name);
        }
        parsed
    }

    // spec-audit: owner=fmtScheme
    fn format_scheme(&mut self, el: ElementId) -> FormatScheme {
        let mut out = FormatScheme::default();
        self.strangers(el, &["name"]);
        if self.readable(el) {
            out.name = self.attr(el, "name");
        }
        for child in self.children(el) {
            match dml_name(self.tree, child) {
                // Read by `read_by_other_tiers`, which has what their readers need.
                Some("fillStyleLst" | "lnStyleLst" | "effectStyleLst" | "bgFillStyleLst") => {}
                _ => self.unknown(child),
            }
        }
        out
    }

    // spec-audit: owner=objectDefaults
    fn object_defaults(&mut self, el: ElementId) -> ObjectDefaults {
        let mut out = ObjectDefaults::default();
        self.strangers(el, &[]);
        for child in self.children(el) {
            match dml_name(self.tree, child) {
                Some("spDef") => self.once(&mut out.shape, child, Self::shape_definition),
                Some("lnDef") => self.once(&mut out.line, child, Self::shape_definition),
                Some("txDef") => self.once(&mut out.text, child, Self::shape_definition),
                Some("extLst") => out.extensions = self.extensions(child),
                _ => self.unknown(child),
            }
        }
        out
    }

    // spec-audit: owner=spDef,lnDef,txDef
    fn shape_definition(&mut self, el: ElementId) -> ShapeDefinition {
        let mut out = ShapeDefinition::default();
        self.strangers(el, &[]);
        for child in self.children(el) {
            match dml_name(self.tree, child) {
                // Read by `read_by_other_tiers`, which has what their readers need.
                Some("spPr" | "bodyPr" | "lstStyle") => {}
                Some("style") => self.once(&mut out.style, child, Self::shape_style),
                Some("extLst") => out.extensions = self.extensions(child),
                _ => self.unknown(child),
            }
        }
        out
    }

    // spec-audit: owner=style
    fn shape_style(&mut self, el: ElementId) -> ShapeStyle {
        let mut out = ShapeStyle::default();
        self.strangers(el, &[]);
        for child in self.children(el) {
            match dml_name(self.tree, child) {
                Some("lnRef") => self.once(&mut out.line, child, Self::matrix_ref),
                Some("fillRef") => self.once(&mut out.fill, child, Self::matrix_ref),
                Some("effectRef") => self.once(&mut out.effect, child, Self::matrix_ref),
                Some("fontRef") => self.once(&mut out.font, child, Self::font_ref),
                _ => self.unknown(child),
            }
        }
        out
    }

    // spec-audit: owner=lnRef,fillRef,effectRef
    fn matrix_ref(&mut self, el: ElementId) -> StyleMatrixRef {
        let mut out = StyleMatrixRef::default();
        self.strangers(el, &["idx"]);
        self.only_colors(el);
        if self.readable(el)
            && let Some(text) = self.required(el, "idx")
        {
            out.idx = text.trim().parse().ok();
            if out.idx.is_none() {
                self.note(FindingKind::BadValue, el, "idx");
            }
        }
        out.color = read_optional_color(self.tree, el, self.findings);
        out
    }

    // spec-audit: owner=fontRef
    fn font_ref(&mut self, el: ElementId) -> FontRef {
        let mut out = FontRef::default();
        self.strangers(el, &["idx"]);
        self.only_colors(el);
        if self.readable(el)
            && let Some(text) = self.required(el, "idx")
        {
            out.idx = FontCollectionIndex::from_xml(&text);
            if out.idx.is_none() {
                self.note(FindingKind::BadValue, el, "idx");
            }
        }
        out.color = read_optional_color(self.tree, el, self.findings);
        out
    }

    // spec-audit: owner=extraClrSchemeLst
    fn extra_schemes(&mut self, el: ElementId) -> Vec<ExtraColorScheme> {
        let mut out = Vec::new();
        self.strangers(el, &[]);
        for child in self.children(el) {
            if out.len() >= self.findings.bounds.list_entries {
                self.note(FindingKind::ListCapped, el, "");
                break;
            }
            if dml_name(self.tree, child) != Some("extraClrScheme") {
                self.unknown(child);
                continue;
            }
            let mut extra = ExtraColorScheme::default();
            self.strangers(child, &[]);
            for part in self.children(child) {
                match dml_name(self.tree, part) {
                    Some("clrScheme") => {
                        self.once(&mut extra.color_scheme, part, Self::color_scheme);
                    }
                    Some("clrMap") => self.once(&mut extra.map, part, Self::color_map),
                    _ => self.unknown(part),
                }
            }
            out.push(extra);
        }
        out
    }

    // spec-audit: owner=custClrLst
    fn custom_colors(&mut self, el: ElementId) -> Vec<CustomColor> {
        let mut out = Vec::new();
        self.strangers(el, &[]);
        for child in self.children(el) {
            if out.len() >= self.findings.bounds.list_entries {
                self.note(FindingKind::ListCapped, el, "");
                break;
            }
            if dml_name(self.tree, child) != Some("custClr") {
                self.unknown(child);
                continue;
            }
            self.strangers(child, &["name"]);
            let name = if self.readable(child) {
                self.attr(child, "name")
            } else {
                None
            };
            let color = read_color_choice(self.tree, child, self.findings);
            out.push(CustomColor { name, color });
        }
        out
    }
}

/// Fill in a format scheme's lists, and each object default's `spPr`, `bodyPr` and `lstStyle`,
/// with the readers a shape's go through. A theme reports in its own list, so what they notice
/// is carried over; a theme that crosses a bound keeps what was read before it, reported, and
/// the deck opens. With `rels` not to hand, what a picture names is not judged.
fn read_by_other_tiers(
    tree: &XmlTree,
    rels: Option<&[Relationship]>,
    scheme: Option<&mut FormatScheme>,
    defaults: Option<&mut ObjectDefaults>,
    findings: &mut Findings,
) {
    const PART: &str = "theme";
    let limits = ImportLimits::default();
    let mut noticed = PropertyFindings::new(limits.max_findings);
    let capped = {
        let mut r = PropertyReader::new(tree, PART, &limits, &mut noticed);
        other_tiers(&mut r, rels.unwrap_or(&[]), scheme, defaults, findings).is_err()
    };
    for f in noticed.iter() {
        let kind = match f.kind {
            K::DanglingRelationship if rels.is_none() => continue,
            K::MissingRequired => FindingKind::MissingRequired,
            K::DuplicateElement => FindingKind::Duplicate,
            K::RetainedElement | K::RetainedAttribute => FindingKind::Unmodelled,
            K::Color(kind) => kind,
            K::Drawing(DrawingFinding::OverLimit) => FindingKind::ListCapped,
            K::Drawing(DrawingFinding::PercentRounded) => FindingKind::PercentRounded,
            _ => FindingKind::BadValue,
        };
        findings.carry(kind, &f.element);
    }
    if capped {
        findings.carry(FindingKind::ListCapped, "theme");
    }
}

fn other_tiers(
    r: &mut PropertyReader<'_>,
    rels: &[Relationship],
    mut scheme: Option<&mut FormatScheme>,
    mut defaults: Option<&mut ObjectDefaults>,
    findings: &mut Findings,
) -> Result<(), ImportError> {
    let root = Elem::root(r.tree, r.part)?;
    for holder in r.children(&root)? {
        if holder.is(ns::DML, "objectDefaults") {
            // A second is reported already, and stays in the part.
            if let Some(defaults) = defaults.take() {
                object_defaults(r, rels, &holder, defaults, findings)?;
            }
        } else if holder.is(ns::DML, "fmtScheme") {
            if let Some(scheme) = scheme.take() {
                format_lists(r, rels, &holder, scheme, findings)?;
            }
        } else if holder.is(ns::DML, "themeElements") {
            for kid in r.children(&holder)? {
                if kid.is(ns::DML, "fmtScheme")
                    && let Some(scheme) = scheme.take()
                {
                    format_lists(r, rels, &kid, scheme, findings)?;
                }
            }
        }
    }
    Ok(())
}

fn format_lists(
    r: &mut PropertyReader<'_>,
    rels: &[Relationship],
    el: &Elem,
    scheme: &mut FormatScheme,
    findings: &Findings,
) -> Result<(), ImportError> {
    let cap = findings.bounds.list_entries;
    for list in r.children(el)? {
        if !list.in_namespace(ns::DML) {
            continue;
        }
        let taken = match list.local.as_str() {
            "fillStyleLst" => scheme.fills.is_some(),
            "lnStyleLst" => scheme.lines.is_some(),
            "effectStyleLst" => scheme.effects.is_some(),
            "bgFillStyleLst" => scheme.background_fills.is_some(),
            _ => continue,
        };
        if taken {
            r.note(K::DuplicateElement, &list, None);
            continue;
        }
        r.finish(&list);
        match list.local.as_str() {
            "fillStyleLst" => scheme.fills = Some(fill_list(r, rels, &list, cap)?),
            "lnStyleLst" => scheme.lines = Some(line_list(r, rels, &list, cap)?),
            "effectStyleLst" => scheme.effects = Some(effect_list(r, rels, &list, cap)?),
            _ => scheme.background_fills = Some(fill_list(r, rels, &list, cap)?),
        }
    }
    Ok(())
}

/// Each style list has at least three entries by its schema, which a style reference counts on.
fn at_least_three(r: &mut PropertyReader<'_>, list: &Elem, len: usize) {
    if len < 3 {
        r.note(K::MissingRequired, list, None);
    }
}

// spec-audit: owner=fillStyleLst,bgFillStyleLst
fn fill_list(
    r: &mut PropertyReader<'_>,
    rels: &[Relationship],
    list: &Elem,
    cap: usize,
) -> Result<Vec<Fill>, ImportError> {
    let mut out = Vec::new();
    for child in r.children(list)? {
        if out.len() >= cap {
            r.note(K::Drawing(DrawingFinding::OverLimit), list, None);
            break;
        }
        if !is_fill_element(&child) {
            r.retained(&child);
            continue;
        }
        out.extend(read_fill(r, rels, DrawingLimits::default(), &child)?);
    }
    at_least_three(r, list, out.len());
    Ok(out)
}

// spec-audit: owner=lnStyleLst
fn line_list(
    r: &mut PropertyReader<'_>,
    rels: &[Relationship],
    list: &Elem,
    cap: usize,
) -> Result<Vec<Line>, ImportError> {
    let mut out = Vec::new();
    for child in r.children(list)? {
        if out.len() >= cap {
            r.note(K::Drawing(DrawingFinding::OverLimit), list, None);
            break;
        }
        if !child.is(ns::DML, "ln") {
            r.retained(&child);
            continue;
        }
        out.extend(read_line(r, rels, DrawingLimits::default(), &child)?);
    }
    at_least_three(r, list, out.len());
    Ok(out)
}

// spec-audit: owner=effectStyleLst
fn effect_list(
    r: &mut PropertyReader<'_>,
    rels: &[Relationship],
    list: &Elem,
    cap: usize,
) -> Result<Vec<StyleEffect>, ImportError> {
    let mut out = Vec::new();
    for child in r.children(list)? {
        if out.len() >= cap {
            r.note(K::Drawing(DrawingFinding::OverLimit), list, None);
            break;
        }
        if !child.is(ns::DML, "effectStyle") {
            r.retained(&child);
            continue;
        }
        r.finish(&child);
        let mut props = ShapeProperties::default();
        let mut seen = HashSet::new();
        let mut ctx = Ctx::new(&mut *r, rels, DrawingLimits::default());
        for part in ctx.r.children(&child)? {
            let fits = part.in_namespace(ns::DML)
                && matches!(
                    part.local.as_str(),
                    "effectLst" | "effectDag" | "scene3d" | "sp3d"
                );
            if !fits || !ctx.property(&part, &mut props, false, &mut seen)? {
                ctx.r.retained(&part);
            }
        }
        out.push(StyleEffect {
            effects: props.effects,
            scene3d: props.scene3d,
            sp3d: props.sp3d,
        });
    }
    at_least_three(r, list, out.len());
    Ok(out)
}

fn object_defaults(
    r: &mut PropertyReader<'_>,
    rels: &[Relationship],
    holder: &Elem,
    defaults: &mut ObjectDefaults,
    findings: &mut Findings,
) -> Result<(), ImportError> {
    let mut seen = HashSet::new();
    for definition in r.children(holder)? {
        if !definition.in_namespace(ns::DML) {
            continue;
        }
        let slot = match definition.local.as_str() {
            "spDef" => defaults.shape.as_mut(),
            "lnDef" => defaults.line.as_mut(),
            "txDef" => defaults.text.as_mut(),
            _ => None,
        };
        // A second of a name is reported already, and stays in the part.
        let Some(slot) = slot.filter(|_| seen.insert(definition.local.clone())) else {
            continue;
        };
        for child in r.children(&definition)? {
            let taken = (child.is(ns::DML, "spPr") && slot.shape_properties.is_some())
                || (child.is(ns::DML, "bodyPr") && slot.body_properties.is_some())
                || (child.is(ns::DML, "lstStyle") && slot.list_style.is_some());
            if taken {
                r.note(K::DuplicateElement, &child, None);
            } else if child.is(ns::DML, "spPr") {
                slot.shape_properties = Some(definition_properties(r, rels, &child)?);
            } else if child.is(ns::DML, "bodyPr") || child.is(ns::DML, "lstStyle") {
                let mut text = TextReader {
                    r: &mut *r,
                    rels,
                    color: &mut *findings,
                    counts: &mut TextCounts::default(),
                    drawing: DrawingLimits::default(),
                };
                if child.is(ns::DML, "bodyPr") {
                    slot.body_properties = Some(text.body_properties(&child)?);
                } else {
                    slot.list_style = Some(text.list_style(&child)?);
                }
            }
        }
    }
    Ok(())
}

/// An object default's `a:spPr`, by Tier 1E's reader.
// spec-audit: owner=spPr
fn definition_properties(
    r: &mut PropertyReader<'_>,
    rels: &[Relationship],
    el: &Elem,
) -> Result<ShapeProperties, ImportError> {
    r.finish(el);
    let mut props = ShapeProperties::default();
    let mut seen = HashSet::new();
    let mut ctx = Ctx::new(&mut *r, rels, DrawingLimits::default());
    for child in ctx.r.children(el)? {
        if !ctx.property(&child, &mut props, false, &mut seen)? {
            ctx.r.retained(&child);
        }
    }
    note_dag_siblings(r, el, &props);
    Ok(props)
}
