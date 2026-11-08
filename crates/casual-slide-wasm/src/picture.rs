//! The Picture Format tab (`UX-003`): Crop, Reset Picture, and the basic Picture
//! Styles, each one undo step through [`Operation::SetLook`], and Crop with the
//! picture's new box in the same batch, as PowerPoint moves the frame with the crop
//! handle so what stays visible does not move.
//!
//! A crop is `p:blipFill/a:srcRect`, in thousandths of a percent of the image cut
//! from each side; a negative value pads instead. The writer patches it as any
//! other `blipFill` change (ADR-013), and the renderer already draws it.

use casual_slide_model::color::{Angle, Color, Percentage};
use casual_slide_model::effect::{Effect, EffectProperties, OuterShadow};
use casual_slide_model::fill::{Fill, RectAlignment, RelativeRect};
use casual_slide_model::line::Line;
use casual_slide_model::presentation::Coordinate;
use casual_slide_model::shape::{NodeKind, ObjectId, ShapeNode};
use casual_slide_transaction::Operation;

use crate::{BridgeError, Viewer};

/// The most a crop may cut from one side, or from two opposite sides together,
/// thousandths of a percent: something of the picture always stays.
pub const MAX_CROP: i32 = 99_000;

/// The most a crop may pad one side (a negative crop), thousandths of a percent.
pub const MAX_PAD: i32 = 1_000_000;

/// The Picture Styles this build offers, by PowerPoint's names: our own simple
/// versions, not Microsoft's artwork (owner instruction 51).
pub const PICTURE_STYLES: [&str; 4] = [
    "Simple Frame, White",
    "Simple Frame, Black",
    "Drop Shadow Rectangle",
    "Thick Matte, Black",
];

fn percent(v: i32) -> Option<Percentage> {
    (v != 0).then_some(Percentage(v))
}

fn frame_line(width: i32, color: Color) -> Box<Line> {
    Box::new(Line {
        width: Some(width),
        fill: Some(Fill::Solid(Some(color))),
        ..Line::default()
    })
}

impl Viewer {
    fn picture_at(&self, index: usize, node: u32) -> Result<(ObjectId, &ShapeNode), BridgeError> {
        let id = ObjectId {
            part: self.slide_ordinal(index)?,
            node,
        };
        let shape = self
            .deck
            .parts()
            .get(id.part)
            .and_then(|p| p.node(id))
            .filter(|n| n.kind == NodeKind::Picture && n.properties.picture.is_some())
            .ok_or(BridgeError::BadArgument)?;
        Ok((id, shape))
    }

    /// Picture `node`'s crop: `[left, top, right, bottom]`, thousandths of a percent,
    /// 0 for a side not cropped.
    pub fn picture_crop(&self, index: usize, node: u32) -> Result<[i32; 4], BridgeError> {
        let (_, shape) = self.picture_at(index, node)?;
        let r = shape
            .properties
            .picture
            .as_ref()
            .and_then(|p| p.source_rect)
            .unwrap_or_default();
        let v = |p: Option<Percentage>| p.map_or(0, |p| p.0);
        Ok([v(r.l), v(r.t), v(r.r), v(r.b)])
    }

    /// Crop picture `node` to `crop` (as [`Self::picture_crop`] gives it) and place it
    /// at `frame` (`x, y, cx, cy`, EMU), as one undo step. A crop past [`MAX_CROP`] or
    /// [`MAX_PAD`] is refused before anything changes. No crop at all writes no
    /// `srcRect`.
    pub fn set_picture_crop(
        &mut self,
        index: usize,
        node: u32,
        crop: [i32; 4],
        frame: (i64, i64, i64, i64),
    ) -> Result<(), BridgeError> {
        let [l, t, r, b] = crop;
        let side_ok = |v: i32| (-MAX_PAD..=MAX_CROP).contains(&v);
        let pair_ok = |a: i32, c: i32| a.saturating_add(c) <= MAX_CROP;
        if !crop.iter().all(|&v| side_ok(v)) || !pair_ok(l, r) || !pair_ok(t, b) {
            return Err(BridgeError::BadArgument);
        }
        let (id, shape) = self.picture_at(index, node)?;
        let rotation = shape.transform.and_then(|t| t.rotation).unwrap_or(0);
        let mut properties = shape.properties.clone();
        let style = shape.style.clone();
        if let Some(fill) = properties.picture.as_mut() {
            fill.source_rect = (crop != [0; 4]).then_some(RelativeRect {
                l: percent(l),
                t: percent(t),
                r: percent(r),
                b: percent(b),
            });
        }
        let place = self.transform_op(index, node, frame, rotation)?;
        self.typing = None;
        self.history.apply(
            self.deck.parts_mut(),
            Operation::Batch(vec![
                Operation::SetLook {
                    node: id,
                    properties: Box::new(properties),
                    style,
                },
                place,
            ]),
        )?;
        self.reports.remove(&index);
        Ok(())
    }

    /// Reset Picture: the picture's styles, outline, effects and colour adjustments
    /// taken off, its crop kept. With `and_size`, Reset Picture & Size: the crop
    /// taken off too, the frame grown back by what it cut, at the same scale.
    pub fn reset_picture(
        &mut self,
        index: usize,
        node: u32,
        and_size: bool,
    ) -> Result<bool, BridgeError> {
        let (id, shape) = self.picture_at(index, node)?;
        let mut properties = shape.properties.clone();
        properties.line = None;
        properties.effects = None;
        properties.scene3d = None;
        properties.sp3d = None;
        if let Some(blip) = properties.picture.as_mut().and_then(|p| p.blip.as_mut()) {
            blip.effects.clear();
        }
        let mut ops = Vec::with_capacity(2);
        if and_size {
            let [l, t, r, b] = self.picture_crop(index, node)?;
            if let Some(p) = properties.picture.as_mut() {
                p.source_rect = None;
            }
            let chain = casual_slide_style::Chain::of(self.deck.parts(), id.part);
            if let Some(e) = chain.transform(shape).map(|t| t.value) {
                // The visible part is `1 - l - r` of the whole: grow back to the whole.
                let grow = |at: i64, size: i64, lead: i32, trail: i32| {
                    let kept = 100_000_i64 - i64::from(lead) - i64::from(trail);
                    let whole = size.saturating_mul(100_000) / kept.max(1);
                    (at - whole.saturating_mul(i64::from(lead)) / 100_000, whole)
                };
                let (x, cx) = grow(e.offset.x.0, e.extent.cx, l, r);
                let (y, cy) = grow(e.offset.y.0, e.extent.cy, t, b);
                ops.push(self.transform_op(index, node, (x, y, cx, cy), e.rotation)?);
            }
        }
        ops.insert(
            0,
            Operation::SetLook {
                node: id,
                properties: Box::new(properties),
                style: None,
            },
        );
        self.typing = None;
        let changed = self
            .history
            .apply(self.deck.parts_mut(), Operation::Batch(ops))?;
        self.reports.remove(&index);
        Ok(changed)
    }

    /// Apply one of [`PICTURE_STYLES`] by name: an outline, a shadow, or both, in
    /// place of what the picture had.
    pub fn set_picture_style(
        &mut self,
        index: usize,
        node: u32,
        name: &str,
    ) -> Result<bool, BridgeError> {
        let (id, shape) = self.picture_at(index, node)?;
        let mut properties = shape.properties.clone();
        let white = Color::srgb(255, 255, 255);
        let black = Color::srgb(0, 0, 0);
        let shadow = || {
            let mut color = Color::srgb(0, 0, 0);
            color
                .transforms
                .push(casual_slide_model::color::ColorTransform::new(
                    casual_slide_model::color::TransformKind::Alpha,
                    40_000,
                ));
            EffectProperties::List(vec![Effect::OuterShadow(Box::new(OuterShadow {
                blur_radius: Some(Coordinate(76_200)),
                distance: Some(Coordinate(38_100)),
                direction: Some(Angle(2_700_000)),
                align: Some(RectAlignment::Tl),
                rotate_with_shape: Some(false),
                color: Some(color),
                ..OuterShadow::default()
            }))])
        };
        let (line, effects) = match name {
            "Simple Frame, White" => (Some(frame_line(88_900, white)), None),
            "Simple Frame, Black" => (Some(frame_line(88_900, black)), None),
            "Drop Shadow Rectangle" => (None, Some(shadow())),
            "Thick Matte, Black" => (Some(frame_line(190_500, black)), Some(shadow())),
            _ => return Err(BridgeError::BadArgument),
        };
        properties.line = line;
        properties.effects = effects;
        self.edit(
            Operation::SetLook {
                node: id,
                properties: Box::new(properties),
                style: None,
            },
            false,
        )
    }
}

#[cfg(test)]
mod tests {
    use casual_slide_ooxml::{OoxmlLimits, PresentationPackage};

    use super::*;

    fn fixture(name: &str) -> Vec<u8> {
        let path = format!(
            "{}/../../fixtures/generated/{name}",
            env!("CARGO_MANIFEST_DIR")
        );
        std::fs::read(path).unwrap()
    }

    fn picture(v: &Viewer) -> crate::ShapeBox {
        let part = v.slide_part(0).unwrap();
        *v.shapes(0)
            .unwrap()
            .iter()
            .find(|s| {
                v.deck
                    .parts()
                    .get(part)
                    .and_then(|p| p.node(ObjectId { part, node: s.node }))
                    .is_some_and(|n| n.kind == NodeKind::Picture)
            })
            .expect("a picture on slide 1")
    }

    fn slide_xml(bytes: &[u8]) -> String {
        let mut p = PresentationPackage::open(bytes.to_vec(), OoxmlLimits::default()).unwrap();
        String::from_utf8(p.read_part("ppt/slides/slide1.xml").unwrap()).unwrap()
    }

    #[test]
    fn a_crop_and_its_new_box_save_read_back_and_undo_in_one_step() {
        let mut v = Viewer::open(fixture("area-pic.pptx")).unwrap();
        let p = picture(&v);
        let before = v.picture_crop(0, p.node).unwrap();
        let cut = p.cx / 4;
        v.set_picture_crop(
            0,
            p.node,
            [25_000, 0, 0, 10_000],
            (p.x + cut, p.y, p.cx - cut, p.cy),
        )
        .unwrap();
        assert_eq!(v.picture_crop(0, p.node).unwrap(), [25_000, 0, 0, 10_000]);
        let saved = v.save().unwrap();
        assert!(slide_xml(&saved).contains("<a:srcRect l=\"25000\" b=\"10000\"/>"));
        let again = Viewer::open(saved).unwrap();
        assert_eq!(
            again.picture_crop(0, p.node).unwrap(),
            [25_000, 0, 0, 10_000]
        );
        let q = picture(&again);
        assert_eq!((q.x, q.cx), (p.x + cut, p.cx - cut));
        v.undo().unwrap();
        assert_eq!(v.picture_crop(0, p.node).unwrap(), before);
        assert_eq!(picture(&v).x, p.x);
    }

    #[test]
    fn a_crop_cutting_everything_or_a_shape_not_a_picture_is_refused() {
        let mut v = Viewer::open(fixture("area-pic.pptx")).unwrap();
        let p = picture(&v);
        let frame = (p.x, p.y, p.cx, p.cy);
        assert!(
            v.set_picture_crop(0, p.node, [60_000, 0, 50_000, 0], frame)
                .is_err()
        );
        assert!(
            v.set_picture_crop(0, p.node, [0, 100_000, 0, 0], frame)
                .is_err()
        );
        assert_eq!(v.picture_crop(0, p.node).unwrap(), [0; 4]);
        let mut shapes = Viewer::open(fixture("preset-shapes.pptx")).unwrap();
        let s = shapes.shapes(0).unwrap()[0];
        assert!(
            shapes
                .set_picture_crop(0, s.node, [1, 0, 0, 0], frame)
                .is_err()
        );
    }

    #[test]
    fn a_picture_style_saves_and_reset_takes_it_off_and_reset_and_size_the_crop() {
        let mut v = Viewer::open(fixture("area-pic.pptx")).unwrap();
        let p = picture(&v);
        assert!(
            v.set_picture_style(0, p.node, "Thick Matte, Black")
                .unwrap()
        );
        assert!(v.set_picture_style(0, p.node, "Nonesuch").is_err());
        let xml = slide_xml(&v.save().unwrap());
        assert!(xml.contains("<a:ln w=\"190500\">"), "outline written");
        assert!(xml.contains("<a:outerShdw"), "shadow written");
        v.set_picture_crop(
            0,
            p.node,
            [50_000, 0, 0, 0],
            (p.x + p.cx / 2, p.y, p.cx / 2, p.cy),
        )
        .unwrap();
        assert!(v.reset_picture(0, p.node, false).unwrap());
        let xml = slide_xml(&v.save().unwrap());
        assert!(!xml.contains("<a:outerShdw"), "shadow taken off");
        assert_eq!(
            v.picture_crop(0, p.node).unwrap(),
            [50_000, 0, 0, 0],
            "crop kept"
        );
        v.reset_picture(0, p.node, true).unwrap();
        assert_eq!(v.picture_crop(0, p.node).unwrap(), [0; 4]);
        let q = picture(&v);
        assert_eq!((q.x, q.cx), (p.x, p.cx / 2 * 2));
    }
}
