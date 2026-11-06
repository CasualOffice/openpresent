//! Native cross-check of the pipeline's speed, and tests of what it produces. The
//! same core runs in the browser; the gap against WASM is the WASM tax.
use spk_text_core::{Engine, corpus};
use std::time::Instant;

fn main() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../fonts/");
    let read = |n: &str| std::fs::read(format!("{dir}{n}")).expect("run fetch_fonts.sh");
    let mut e = Engine::new();
    let t = Instant::now();
    let carlito = e.register(read("Carlito-Regular.ttf")).unwrap();
    let liberation = e.register(read("LiberationSans-Regular.ttf")).unwrap();
    let arabic = e.register(read("NotoSansArabic.ttf")).unwrap();
    let cjk = e.register(read("NotoSansSC.otf")).unwrap();
    println!(
        "register 4 faces: {:.2} ms",
        t.elapsed().as_secs_f64() * 1e3
    );
    let cases: Vec<(&str, String, Vec<usize>)> = vec![
        ("latin-carlito", corpus::latin(2000, 1), vec![carlito]),
        ("latin-liberation", corpus::latin(2000, 1), vec![liberation]),
        ("arabic", corpus::arabic(2000, 3), vec![arabic, carlito]),
        ("cjk", corpus::cjk(2000, 5), vec![cjk, carlito]),
        ("mixed", corpus::mixed(2000, 7), vec![carlito, arabic, cjk]),
    ];
    for (name, text, stack) in &cases {
        let t = Instant::now();
        let first = e.layout(text, stack, 16.0, 600.0);
        let cold = t.elapsed().as_secs_f64() * 1e3;
        let t = Instant::now();
        let n = 200;
        for _ in 0..n {
            std::hint::black_box(e.layout(text, stack, 16.0, 600.0));
        }
        let warm = t.elapsed().as_secs_f64() * 1e3 / f64::from(n);
        println!(
            "{name}: chars {} bytes {} glyphs {} lines {} first {cold:.3} ms warm {warm:.3} ms",
            text.chars().count(),
            text.len(),
            first.gids.len(),
            first.line_y.len()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> (Engine, usize, usize, usize) {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../fonts/");
        let read = |n: &str| std::fs::read(format!("{dir}{n}")).expect("run fetch_fonts.sh");
        let mut e = Engine::new();
        let a = e.register(read("Carlito-Regular.ttf")).unwrap();
        let b = e.register(read("NotoSansArabic.ttf")).unwrap();
        let c = e.register(read("NotoSansSC.otf")).unwrap();
        (e, a, b, c)
    }

    #[test]
    fn latin_ligature_is_one_cluster_and_wraps() {
        let (mut e, a, _, _) = engine();
        let text = corpus::latin(2000, 1);
        let l = e.layout(&text, &[a], 16.0, 300.0);
        assert!(l.line_y.len() > 5, "wrapped");
        assert!(
            l.line_w.iter().all(|w| *w < 330.0),
            "lines fit: {:?}",
            l.line_w
        );
        let single = e.layout("office", &[a], 16.0, 1000.0);
        assert!(
            single.gids.len() < 6,
            "ffi ligature formed: {:?}",
            single.gids
        );
        // Every byte of text is on exactly one line.
        assert_eq!(l.line_start[0], 0);
        assert_eq!(*l.line_end.last().unwrap() as usize, text.len());
        for (end, next) in l.line_end.iter().zip(l.line_start.iter().skip(1)) {
            assert_eq!(end, next);
        }
    }

    #[test]
    fn arabic_is_rtl_and_joins() {
        let (mut e, a, b, _) = engine();
        let text = "\u{0633}\u{0644}\u{0627}\u{0645}";
        let l = e.layout(text, &[b, a], 20.0, 400.0);
        assert_eq!(l.line_rtl, vec![1]);
        // First logical cluster is drawn right of the last.
        let first = l.cl_start.iter().position(|c| *c == 0).unwrap();
        let last = l.cl_start.iter().position(|c| *c == 6).unwrap();
        assert!(l.cl_x[first] > l.cl_x[last]);
        // Joining: the initial form differs from the isolated letter.
        let iso = e.layout("\u{0633}", &[b, a], 20.0, 400.0);
        assert!(!l.gids.contains(&iso.gids[0]), "seen joined");
    }

    #[test]
    fn mixed_uses_each_face_and_loses_no_text() {
        let (mut e, a, b, c) = engine();
        let text = corpus::mixed(2000, 7);
        let l = e.layout(&text, &[a, b, c], 16.0, 500.0);
        for f in [a, b, c] {
            assert!(l.faces.contains(&(f as u32)), "face {f} used");
        }
        let mut covered: Vec<(u32, u32)> = l
            .cl_start
            .iter()
            .copied()
            .zip(l.cl_end.iter().copied())
            .collect();
        covered.sort_unstable();
        assert_eq!(covered[0].0, 0);
        assert_eq!(covered.last().unwrap().1 as usize, text.len());
        for w in covered.windows(2) {
            assert_eq!(w[0].1, w[1].0, "clusters tile the text");
        }
    }

    #[test]
    fn outlines_exist_for_truetype_and_cff() {
        let (mut e, a, _, c) = engine();
        let l = e.layout("A", &[a], 16.0, 100.0);
        assert!(
            e.faces[a].glyph_svg(l.gids[0]).contains('M'),
            "glyf outline"
        );
        let l = e.layout("\u{6C49}", &[c], 16.0, 100.0);
        assert!(
            e.faces[c].glyph_svg(l.gids[0]).contains('C'),
            "CFF cubic outline"
        );
    }
}
