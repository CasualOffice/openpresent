//! Prints evaluated presets as JSON lines, for tools/geometry-presets/crosscheck.py.
//!
//!     cargo run -q -p casual-slide-geometry --example dump_geometry -- 1200000 800000
//!
//! A third argument, `cubics`, gives shape EMU with arcs as cubic Beziers, for
//! tools/geometry-presets/render_crosscheck.py.

use casual_slide_geometry::{AdjustHandle, Extents, Point, Segment, preset_shapes, text_warps};
use std::fmt::Write;

fn pair(p: Point) -> String {
    format!("[{:?},{:?}]", p.x, p.y)
}

fn main() {
    let mut args = std::env::args().skip(1);
    let w = args
        .next()
        .and_then(|a| a.parse().ok())
        .unwrap_or(1_200_000);
    let h = args.next().and_then(|a| a.parse().ok()).unwrap_or(800_000);
    let cubics = args.next().as_deref() == Some("cubics");
    for def in preset_shapes().iter().chain(text_warps()) {
        let g = def.evaluate(Extents::new(w, h), &[]);
        let mut paths = Vec::new();
        for path in &g.paths {
            let converted = path.scaled_to(g.extents).with_arcs_as_cubics();
            let path = if cubics { &converted } else { path };
            let mut items = Vec::new();
            for segment in &path.segments {
                items.push(match segment {
                    Segment::MoveTo(p) => format!("[\"M\",{}]", pair(*p)),
                    Segment::LineTo(p) => format!("[\"L\",{}]", pair(*p)),
                    Segment::QuadTo { ctrl, end } => {
                        format!("[\"Q\",{},{}]", pair(*ctrl), pair(*end))
                    }
                    Segment::CubicTo { c1, c2, end } => {
                        format!("[\"C\",{},{},{}]", pair(*c1), pair(*c2), pair(*end))
                    }
                    Segment::ArcTo(a) => format!(
                        "[\"A\",[{:?},{:?}],[{:?},{:?}],{},{}]",
                        a.wr,
                        a.hr,
                        a.st_ang,
                        a.sw_ang,
                        pair(a.start),
                        pair(a.end)
                    ),
                    Segment::Close => "[\"Z\"]".to_owned(),
                });
            }
            paths.push(format!(
                "{{\"w\":{},\"h\":{},\"fill\":\"{:?}\",\"stroke\":{},\"segments\":[{}]}}",
                path.w,
                path.h,
                path.fill,
                path.stroke,
                items.join(",")
            ));
        }
        let sites = g
            .connection_sites
            .iter()
            .map(|s| pair(s.pos))
            .collect::<Vec<_>>()
            .join(",");
        let handles = g
            .adjust_handles
            .iter()
            .map(|h| match h {
                AdjustHandle::Xy { pos, .. } | AdjustHandle::Polar { pos, .. } => pair(*pos),
            })
            .collect::<Vec<_>>()
            .join(",");
        let t = g.text_rect;
        let mut line = String::new();
        let _ = write!(
            line,
            "{{\"name\":\"{}\",\"paths\":[{}],\"sites\":[{}],\"handles\":[{}],\"rect\":[{:?},{:?},{:?},{:?}]}}",
            def.name,
            paths.join(","),
            sites,
            handles,
            t.l,
            t.t,
            t.r,
            t.b
        );
        println!("{line}");
    }
}
