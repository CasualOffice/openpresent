//! Design edits (`VS-12`): a theme's colour and font schemes brought to a model, and the
//! presentation's slide size. Each patches the named nodes only (ADR-013) and returns
//! whether anything changed, so a no-op leaves the part out of the save.

use casual_slide_import::properties::ns;
use casual_slide_model::theme::{BaseStyles, ColorScheme, ColorSchemeIndex, FontScheme};
use casual_slide_ooxml::XmlTree;

use crate::color::ExportError;
use crate::theme::{
    FontRole, FontSlot, set_color_scheme_name, set_font_scheme_name, set_scheme_color, set_typeface,
};

/// The twelve colour-scheme slots, in [`ColorScheme::slots`]' order.
const SLOTS: [ColorSchemeIndex; 12] = [
    ColorSchemeIndex::Dk1,
    ColorSchemeIndex::Lt1,
    ColorSchemeIndex::Dk2,
    ColorSchemeIndex::Lt2,
    ColorSchemeIndex::Accent1,
    ColorSchemeIndex::Accent2,
    ColorSchemeIndex::Accent3,
    ColorSchemeIndex::Accent4,
    ColorSchemeIndex::Accent5,
    ColorSchemeIndex::Accent6,
    ColorSchemeIndex::Hlink,
    ColorSchemeIndex::FolHlink,
];

fn patch_colors(
    tree: &mut XmlTree,
    before: &ColorScheme,
    after: &ColorScheme,
) -> Result<bool, ExportError> {
    let mut changed = false;
    if before.name != after.name {
        let name = after
            .name
            .as_deref()
            .ok_or(ExportError::InvalidValue("a colour scheme with no name"))?;
        changed |= set_color_scheme_name(tree, name)?;
    }
    for (slot, (b, a)) in SLOTS.iter().zip(before.slots.iter().zip(&after.slots)) {
        if b == a {
            continue;
        }
        let color = a
            .as_ref()
            .ok_or(ExportError::InvalidValue("a colour scheme slot taken away"))?;
        changed |= set_scheme_color(tree, *slot, color)?;
    }
    Ok(changed)
}

fn patch_fonts(
    tree: &mut XmlTree,
    before: &FontScheme,
    after: &FontScheme,
) -> Result<bool, ExportError> {
    let mut changed = false;
    if before.name != after.name {
        let name = after
            .name
            .as_deref()
            .ok_or(ExportError::InvalidValue("a font scheme with no name"))?;
        changed |= set_font_scheme_name(tree, name)?;
    }
    for (role, b, a) in [
        (FontRole::Major, &before.major, &after.major),
        (FontRole::Minor, &before.minor, &after.minor),
    ] {
        let (Some(b), Some(a)) = (b, a) else {
            if b != a {
                return Err(ExportError::NodeMissing("a font collection"));
            }
            continue;
        };
        if b.fonts != a.fonts || b.extensions != a.extensions {
            return Err(ExportError::InvalidValue("a supplemental font"));
        }
        for (slot, bf, af) in [
            (FontSlot::Latin, &b.latin, &a.latin),
            (FontSlot::Ea, &b.ea, &a.ea),
            (FontSlot::Cs, &b.cs, &a.cs),
        ] {
            if bf == af {
                continue;
            }
            let (Some(bf), Some(af)) = (bf, af) else {
                return Err(ExportError::NodeMissing("a theme font"));
            };
            if (&bf.panose, bf.pitch_family, bf.charset)
                != (&af.panose, af.pitch_family, af.charset)
            {
                return Err(ExportError::InvalidValue("a theme font's metrics"));
            }
            let typeface = af
                .typeface
                .as_deref()
                .ok_or(ExportError::InvalidValue("a theme font with no typeface"))?;
            changed |= set_typeface(tree, role, slot, typeface)?;
        }
    }
    Ok(changed)
}

/// Bring a theme part's `clrScheme` and `fontScheme` from `before`, what the part holds,
/// to `after`: names, colour slots and typefaces, node by node. A scheme added or taken
/// away, or a change past those, is refused before anything is written, since this
/// writes only what Design's theme and variant commands change.
pub fn patch_theme_schemes(
    tree: &mut XmlTree,
    before: &BaseStyles,
    after: &BaseStyles,
) -> Result<bool, ExportError> {
    let mut changed = false;
    match (&before.color_scheme, &after.color_scheme) {
        (Some(b), Some(a)) if b != a => {
            if b.extensions != a.extensions {
                return Err(ExportError::InvalidValue("colour scheme extensions"));
            }
            changed |= patch_colors(tree, b, a)?;
        }
        (b, a) if b != a => return Err(ExportError::NodeMissing("clrScheme")),
        _ => {}
    }
    match (&before.font_scheme, &after.font_scheme) {
        (Some(b), Some(a)) if b != a => {
            if b.extensions != a.extensions {
                return Err(ExportError::InvalidValue("font scheme extensions"));
            }
            changed |= patch_fonts(tree, b, a)?;
        }
        (b, a) if b != a => return Err(ExportError::NodeMissing("fontScheme")),
        _ => {}
    }
    Ok(changed)
}

/// `sldSz/@type` for a size PowerPoint names: 4:3 on screen and 16:10. Widescreen 16:9
/// at 13.333 by 7.5 inches is written with no `@type`, as PowerPoint writes it.
fn size_type(cx: i64, cy: i64) -> Option<&'static str> {
    match (cx, cy) {
        (9_144_000, 6_858_000) => Some("screen4x3"),
        (9_144_000, 5_715_000) => Some("screen16x10"),
        (9_144_000, 5_143_500) => Some("screen16x9"),
        _ => None,
    }
}

/// Set the presentation part's `sldSz` to `cx` by `cy` EMU, its `@type` to the size's
/// name, or taken away when the size has none ("custom"). The element must be there:
/// the schema requires it, and a deck without one is refused rather than given one.
pub fn set_slide_size(tree: &mut XmlTree, cx: i64, cy: i64) -> Result<bool, ExportError> {
    let root = tree.root();
    let el = tree
        .child_elements(root)
        .into_iter()
        .find(|&c| {
            tree.local_name(c) == "sldSz"
                && tree
                    .namespace_uri(c)
                    .is_some_and(|u| ns::PML.contains(&u.as_str()))
        })
        .ok_or(ExportError::NodeMissing("sldSz"))?;
    if !tree.attributes_valid(el) {
        return Err(ExportError::MalformedNode("sldSz"));
    }
    let fits = |v: i64| (914_400..=51_206_400).contains(&v);
    if !fits(cx) || !fits(cy) {
        return Err(ExportError::InvalidValue("a slide size"));
    }
    let mut changed = false;
    for (name, value) in [("cx", cx), ("cy", cy)] {
        let value = value.to_string();
        if tree.plain_attribute(el, name).as_deref() != Some(value.as_str()) {
            if !tree.set_attribute(el, name, &value) {
                return Err(ExportError::MalformedNode("sldSz"));
            }
            changed = true;
        }
    }
    let wanted = size_type(cx, cy);
    let had = tree.plain_attribute(el, "type");
    // Unchanged in size, the type stays as read: a size can carry another name.
    if changed && had.as_deref() != wanted {
        match wanted {
            Some(t) => {
                if !tree.set_attribute(el, "type", t) {
                    return Err(ExportError::MalformedNode("sldSz"));
                }
            }
            None => {
                tree.remove_attribute(el, "type");
            }
        }
    }
    Ok(changed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use casual_slide_ooxml::OoxmlLimits;

    const PRES: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"><p:sldSz cx="9144000" cy="6858000" type="screen4x3"/><p:notesSz cx="6858000" cy="9144000"/></p:presentation>"#;

    fn tree() -> XmlTree {
        XmlTree::parse(
            PRES.as_bytes(),
            "/ppt/presentation.xml",
            &OoxmlLimits::default(),
        )
        .unwrap()
    }

    #[test]
    fn slide_size_patches_sldsz_alone_and_names_its_type() {
        let mut t = tree();
        assert!(set_slide_size(&mut t, 12_192_000, 6_858_000).unwrap());
        let out = String::from_utf8(t.to_bytes()).unwrap();
        assert!(
            out.contains(r#"<p:sldSz cx="12192000" cy="6858000"/>"#),
            "{out}"
        );
        assert!(out.contains(r#"<p:notesSz cx="6858000" cy="9144000"/>"#));
        assert!(set_slide_size(&mut t, 9_144_000, 6_858_000).unwrap());
        let out = String::from_utf8(t.to_bytes()).unwrap();
        assert!(out.contains(r#"type="screen4x3""#), "{out}");
        assert!(!set_slide_size(&mut t, 9_144_000, 6_858_000).unwrap());
    }

    #[test]
    fn a_slide_size_outside_the_schema_is_refused() {
        let mut t = tree();
        let err = set_slide_size(&mut t, 100, 6_858_000).unwrap_err();
        assert_eq!(err.code(), "CS-CLE-0001");
    }
}
