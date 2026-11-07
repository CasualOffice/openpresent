#![no_main]
//! GEO-04: any text as any field of custom geometry. Compiling and evaluating never
//! panics, always ends, and gives finite numbers inside the limit.
//!
//! The first bytes choose the extents and how many of each thing to make; the rest is cut
//! into text fields, as a part would supply them.

use casual_slide_geometry::math::COORD_LIMIT;
use casual_slide_geometry::{
    Adjust, Extents, RawAxis, RawCommand, RawGeometry, RawGuide, RawHandle, RawPath, RawPoint,
    RawRect, RawSite, Segment,
};
use libfuzzer_sys::fuzz_target;

/// Text fields from the input, forever, so a short input can still fill a long geometry.
struct Fields<'a> {
    words: Vec<&'a str>,
    at: usize,
}

impl<'a> Fields<'a> {
    fn next(&mut self) -> String {
        if self.words.is_empty() {
            return String::new();
        }
        let word = self.words[self.at % self.words.len()];
        self.at = self.at.wrapping_add(1);
        word.to_owned()
    }

    fn point(&mut self) -> RawPoint {
        RawPoint {
            x: self.next(),
            y: self.next(),
        }
    }

    fn axis(&mut self) -> RawAxis {
        RawAxis {
            guide: Some(self.next()),
            min: Some(self.next()),
            max: Some(self.next()),
        }
    }
}

fn finite(value: f64) {
    assert!(value.is_finite(), "{value}");
    assert!(value.abs() <= COORD_LIMIT, "{value}");
}

fuzz_target!(|data: &[u8]| {
    let Some((head, rest)) = data.split_first_chunk::<12>() else {
        return;
    };
    let text = String::from_utf8_lossy(rest);
    let mut f = Fields {
        words: text.split('\n').collect(),
        at: 0,
    };
    let count = |b: u8| usize::from(b) % 32;
    let raw = RawGeometry {
        av: (0..count(head[0]))
            .map(|_| RawGuide {
                name: f.next(),
                fmla: f.next(),
            })
            .collect(),
        gd: (0..count(head[1]))
            .map(|_| RawGuide {
                name: f.next(),
                fmla: f.next(),
            })
            .collect(),
        handles: (0..count(head[2]))
            .map(|i| {
                if i % 2 == 0 {
                    RawHandle::Xy {
                        x: f.axis(),
                        y: f.axis(),
                        pos: f.point(),
                    }
                } else {
                    RawHandle::Polar {
                        r: f.axis(),
                        ang: f.axis(),
                        pos: f.point(),
                    }
                }
            })
            .collect(),
        sites: (0..count(head[3]))
            .map(|_| RawSite {
                ang: f.next(),
                pos: f.point(),
            })
            .collect(),
        rect: (head[4] % 2 == 0).then(|| RawRect {
            l: f.next(),
            t: f.next(),
            r: f.next(),
            b: f.next(),
        }),
        paths: (0..count(head[5]) % 4)
            .map(|p| RawPath {
                w: i64::from(head[6]) * 1 << (p * 8),
                h: i64::from(head[7]) * 1 << (p * 8),
                commands: (0..count(head[8]))
                    .map(|i| match i % 6 {
                        0 => RawCommand::MoveTo(f.point()),
                        1 => RawCommand::LineTo(f.point()),
                        2 => RawCommand::ArcTo {
                            wr: f.next(),
                            hr: f.next(),
                            st_ang: f.next(),
                            sw_ang: f.next(),
                        },
                        3 => RawCommand::QuadBezTo(f.point(), f.point()),
                        4 => RawCommand::CubicBezTo(f.point(), f.point(), f.point()),
                        _ => RawCommand::Close,
                    })
                    .collect(),
                ..RawPath::default()
            })
            .collect(),
    };
    let Ok(compiled) = raw.compile() else {
        return;
    };
    let w = i64::from(i32::from_le_bytes([head[9], head[10], head[11], head[0]]));
    let h = w.rotate_left(13);
    let names: Vec<String> = (0..count(head[1])).map(|_| f.next()).collect();
    let adjusts: Vec<Adjust<'_>> = names
        .iter()
        .enumerate()
        .map(|(i, n)| Adjust::val(n, (i as i64).wrapping_mul(w)))
        .collect();
    let out = compiled.def.evaluate(Extents::new(w, h), &adjusts);
    for path in &out.paths {
        for candidate in [
            path.clone(),
            path.scaled_to(out.extents),
            path.with_arcs_as_cubics(),
        ] {
            for segment in &candidate.segments {
                match *segment {
                    Segment::MoveTo(p) | Segment::LineTo(p) => {
                        finite(p.x);
                        finite(p.y);
                    }
                    Segment::QuadTo { ctrl, end } => {
                        finite(ctrl.x);
                        finite(ctrl.y);
                        finite(end.x);
                        finite(end.y);
                    }
                    Segment::CubicTo { c1, c2, end } => {
                        finite(c1.x);
                        finite(c1.y);
                        finite(c2.x);
                        finite(c2.y);
                        finite(end.x);
                        finite(end.y);
                    }
                    _ => {}
                }
            }
        }
    }
    let _ = out.fingerprint();
});
