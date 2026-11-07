//! Reads theme parts, and colour maps wherever they live, into the model.
//!
//! What a theme holds beyond the model (the rest of a fill, an effect, a shape
//! default, an unknown extension) stays in the tree. Elements unknown to the
//! schema are reported as `CS-CLR-0009`. Those the schema has but another area
//! owns are expected to be retained, and are not reported.

use casual_slide_model::theme::{
    BaseStyles, ClipboardStyleSheet, ColorMap, ColorMapOverride, ColorMapSlot, ColorScheme,
    ColorSchemeIndex, CustomColor, ExtraColorScheme, FontCollection, FontCollectionIndex, FontRef,
    FontScheme, FormatScheme, GradientStop, ObjectDefaults, ShapeDefinition, ShapeStyle,
    StyleEffect, StyleFill, StyleLine, StyleMatrixRef, SupplementalFont, TextFont, Theme,
    ThemeFamily, ThemeOverride, ThemeableFont, ThemeableStyle,
};
use casual_slide_ooxml::{ElementId, OoxmlError, OoxmlLimits, PresentationPackage, XmlTree};

use crate::color::{
    FindingKind, Findings, bounded_children, dml_name, read_color_choice, read_optional_color,
};

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

/// Read a `theme` part under these limits.
pub fn import_theme_limited(
    tree: &XmlTree,
    limits: &OoxmlLimits,
) -> Result<Imported<Theme>, ThemeImportError> {
    let root = expect_root(tree, "theme")?;
    let mut findings = Findings::with_limits(limits);
    let value = Reader {
        tree,
        findings: &mut findings,
    }
    .theme(root);
    Ok(Imported { value, findings })
}

/// Read a `themeOverride` part.
pub fn import_theme_override(tree: &XmlTree) -> Result<Imported<ThemeOverride>, ThemeImportError> {
    let root = expect_root(tree, "themeOverride")?;
    let mut findings = Findings::default();
    let mut r = Reader {
        tree,
        findings: &mut findings,
    };
    let mut value = ThemeOverride::default();
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
    Ok(Imported { value, findings })
}

/// Read a `CT_ClipboardStyleSheet` root, by its schema shape.
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
    for child in r.children(el) {
        let this = match dml_name(tree, child) {
            Some("masterClrMapping") => ColorMapOverride::Master,
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

    fn theme(&mut self, root: ElementId) -> Theme {
        let mut theme = Theme::default();
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

    fn base_styles(&mut self, el: ElementId) -> BaseStyles {
        let mut out = BaseStyles::default();
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

    fn color_scheme(&mut self, el: ElementId) -> ColorScheme {
        let mut out = ColorScheme::default();
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

    fn color_map(&mut self, el: ElementId) -> ColorMap {
        let mut out = ColorMap::default();
        if !self.readable(el) {
            return out;
        }
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

    fn font_scheme(&mut self, el: ElementId) -> FontScheme {
        let mut out = FontScheme::default();
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

    fn font_collection(&mut self, el: ElementId) -> FontCollection {
        let mut out = FontCollection::default();
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

    fn text_font(&mut self, el: ElementId) -> TextFont {
        let mut out = TextFont::default();
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

    fn format_scheme(&mut self, el: ElementId) -> FormatScheme {
        let mut out = FormatScheme::default();
        if self.readable(el) {
            out.name = self.attr(el, "name");
        }
        for child in self.children(el) {
            match dml_name(self.tree, child) {
                Some("fillStyleLst") => self.once(&mut out.fills, child, Self::fill_list),
                Some("lnStyleLst") => self.once(&mut out.lines, child, Self::line_list),
                Some("effectStyleLst") => self.once(&mut out.effects, child, Self::effect_list),
                Some("bgFillStyleLst") => {
                    self.once(&mut out.background_fills, child, Self::fill_list);
                }
                _ => self.unknown(child),
            }
        }
        out
    }

    fn fill_list(&mut self, el: ElementId) -> Vec<StyleFill> {
        let mut out = Vec::new();
        for child in self.children(el) {
            if out.len() >= self.findings.bounds.list_entries {
                self.note(FindingKind::ListCapped, el, "");
                break;
            }
            match self.fill(child) {
                Some(fill) => out.push(fill),
                None => self.unknown(child),
            }
        }
        self.at_least_three(el, out.len());
        out
    }

    fn at_least_three(&mut self, el: ElementId, len: usize) {
        if len < 3 {
            self.note(FindingKind::MissingRequired, el, "three entries");
        }
    }

    fn fill(&mut self, el: ElementId) -> Option<StyleFill> {
        Some(match dml_name(self.tree, el)? {
            "noFill" => StyleFill::None,
            "solidFill" => StyleFill::Solid(read_optional_color(self.tree, el, self.findings)),
            "gradFill" => StyleFill::Gradient(self.gradient_stops(el)),
            "blipFill" => StyleFill::Picture,
            "pattFill" => {
                let (mut foreground, mut background) = (None, None);
                for child in self.children(el) {
                    match dml_name(self.tree, child) {
                        Some("fgClr") => {
                            foreground = read_color_choice(self.tree, child, self.findings);
                        }
                        Some("bgClr") => {
                            background = read_color_choice(self.tree, child, self.findings);
                        }
                        _ => {}
                    }
                }
                StyleFill::Pattern {
                    foreground,
                    background,
                }
            }
            "grpFill" => StyleFill::Group,
            _ => return None,
        })
    }

    fn gradient_stops(&mut self, el: ElementId) -> Vec<GradientStop> {
        let mut out = Vec::new();
        for child in self.children(el) {
            if dml_name(self.tree, child) != Some("gsLst") {
                continue;
            }
            for stop in self.children(child) {
                if dml_name(self.tree, stop) != Some("gs") {
                    self.unknown(stop);
                    continue;
                }
                if out.len() >= self.findings.bounds.list_entries {
                    self.note(FindingKind::ListCapped, child, "gs");
                    break;
                }
                let mut position = None;
                if self.readable(stop) {
                    let text = self.required(stop, "pos");
                    if let Some(text) = text {
                        let read = casual_slide_model::color::parse_percentage(&text);
                        if read.is_none() {
                            self.note(FindingKind::BadValue, stop, "pos");
                        }
                        position = read.map(|r| r.value);
                    }
                }
                let color = read_color_choice(self.tree, stop, self.findings);
                out.push(GradientStop { position, color });
            }
        }
        out
    }

    fn line_list(&mut self, el: ElementId) -> Vec<StyleLine> {
        let mut out = Vec::new();
        for child in self.children(el) {
            if out.len() >= self.findings.bounds.list_entries {
                self.note(FindingKind::ListCapped, el, "");
                break;
            }
            if dml_name(self.tree, child) != Some("ln") {
                self.unknown(child);
                continue;
            }
            let mut line = StyleLine::default();
            if self.readable(child)
                && let Some(text) = self.attr(child, "w")
            {
                line.width = casual_slide_model::color::parse_int(&text);
                if line.width.is_none() {
                    self.note(FindingKind::BadValue, child, "w");
                }
            }
            for part in self.children(child) {
                // The other children of a line belong to LINE, and stay.
                if let Some(fill) = self.fill(part) {
                    line.fill = Some(fill);
                    break;
                }
            }
            out.push(line);
        }
        self.at_least_three(el, out.len());
        out
    }

    fn effect_list(&mut self, el: ElementId) -> Vec<StyleEffect> {
        let mut out = Vec::new();
        for child in self.children(el) {
            if out.len() >= self.findings.bounds.list_entries {
                self.note(FindingKind::ListCapped, el, "");
                break;
            }
            if dml_name(self.tree, child) != Some("effectStyle") {
                self.unknown(child);
                continue;
            }
            let mut effect = StyleEffect::default();
            for part in self.children(child) {
                match dml_name(self.tree, part) {
                    Some("effectLst" | "effectDag") => effect.has_effects = true,
                    Some("scene3d") => effect.has_scene3d = true,
                    Some("sp3d") => effect.has_sp3d = true,
                    _ => self.unknown(part),
                }
            }
            out.push(effect);
        }
        self.at_least_three(el, out.len());
        out
    }

    fn object_defaults(&mut self, el: ElementId) -> ObjectDefaults {
        let mut out = ObjectDefaults::default();
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

    fn shape_definition(&mut self, el: ElementId) -> ShapeDefinition {
        let mut out = ShapeDefinition::default();
        for child in self.children(el) {
            match dml_name(self.tree, child) {
                Some("spPr") => out.has_shape_properties = true,
                Some("bodyPr") => out.has_body_properties = true,
                Some("lstStyle") => out.has_list_style = true,
                Some("style") => self.once(&mut out.style, child, Self::shape_style),
                Some("extLst") => out.extensions = self.extensions(child),
                _ => self.unknown(child),
            }
        }
        out
    }

    fn shape_style(&mut self, el: ElementId) -> ShapeStyle {
        let mut out = ShapeStyle::default();
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

    fn matrix_ref(&mut self, el: ElementId) -> StyleMatrixRef {
        let mut out = StyleMatrixRef::default();
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

    fn font_ref(&mut self, el: ElementId) -> FontRef {
        let mut out = FontRef::default();
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

    fn extra_schemes(&mut self, el: ElementId) -> Vec<ExtraColorScheme> {
        let mut out = Vec::new();
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

    fn custom_colors(&mut self, el: ElementId) -> Vec<CustomColor> {
        let mut out = Vec::new();
        for child in self.children(el) {
            if out.len() >= self.findings.bounds.list_entries {
                self.note(FindingKind::ListCapped, el, "");
                break;
            }
            if dml_name(self.tree, child) != Some("custClr") {
                self.unknown(child);
                continue;
            }
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
