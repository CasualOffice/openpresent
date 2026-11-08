use super::*;

fn style(size: f32) -> RunStyle {
    RunStyle {
        family: Some("Calibri".to_owned()),
        size,
        bold: false,
        italic: false,
        color: Rgba::BLACK,
        ..RunStyle::default()
    }
}

fn run(text: &str, size: f32) -> StyledText {
    StyledText {
        text: text.to_owned(),
        style: style(size),
    }
}

fn paragraph(runs: Vec<StyledText>) -> ParagraphLayout {
    ParagraphLayout {
        runs,
        end: style(18.0),
        ..ParagraphLayout::default()
    }
}

fn body(paragraphs: Vec<ParagraphLayout>) -> BodyLayout {
    BodyLayout {
        wrap: true,
        paragraphs,
        ..BodyLayout::default()
    }
}

const FRAME: Frame = Frame {
    x: 10.0,
    y: 20.0,
    width: 200.0,
    height: 300.0,
};

fn baselines(layout: &TextLayout) -> Vec<f32> {
    let mut ys: Vec<f32> = Vec::new();
    for g in layout.runs.iter().flat_map(|r| r.glyphs.iter()) {
        if !ys.iter().any(|y| (y - g.y).abs() < 0.01) {
            ys.push(g.y);
        }
    }
    ys
}

fn drawn_text(layout: &TextLayout) -> String {
    layout.runs.iter().map(|r| r.text.as_str()).collect()
}

#[test]
fn short_text_is_one_line_starting_at_the_left_inset() {
    let mut b = body(vec![paragraph(vec![run("Hello world", 18.0)])]);
    b.insets.left = 7.2;
    let layout = layout_body(&b, FRAME);
    assert_eq!(layout.lines, 1);
    assert_eq!(drawn_text(&layout), "Hello world");
    let first = layout.runs[0].glyphs[0];
    assert!((first.x - (FRAME.x + 7.2)).abs() < 0.01, "{first:?}");
    assert!(first.y > FRAME.y && first.y < FRAME.y + 30.0, "{first:?}");
}

#[test]
fn long_text_wraps_inside_the_frame_and_keeps_every_character() {
    let text = "The quick brown fox jumps over the lazy dog and keeps running";
    let layout = layout_body(&body(vec![paragraph(vec![run(text, 18.0)])]), FRAME);
    assert!(layout.lines >= 3, "{} lines", layout.lines);
    assert_eq!(baselines(&layout).len(), layout.lines);
    let right = FRAME.x + FRAME.width;
    for g in layout.runs.iter().flat_map(|r| r.glyphs.iter()) {
        assert!(g.x >= FRAME.x - 0.01 && g.x < right, "{g:?}");
    }
    let words: Vec<&str> = text.split(' ').collect();
    let drawn = drawn_text(&layout);
    let drawn_words: Vec<&str> = drawn.split_whitespace().collect();
    assert_eq!(
        drawn_words.concat(),
        words.concat(),
        "nothing lost at a break"
    );
}

#[test]
fn without_wrap_only_a_line_break_ends_a_line() {
    let text = "The quick brown fox jumps over the lazy dog and keeps running";
    let mut b = body(vec![paragraph(vec![run(text, 18.0)])]);
    b.wrap = false;
    assert_eq!(layout_body(&b, FRAME).lines, 1);
    b.paragraphs[0].runs.push(run("\n", 18.0));
    b.paragraphs[0].runs.push(run("second", 18.0));
    assert_eq!(layout_body(&b, FRAME).lines, 2);
}

#[test]
fn a_line_break_starts_a_new_line_even_when_there_is_room() {
    let layout = layout_body(
        &body(vec![paragraph(vec![
            run("a", 18.0),
            run("\n", 18.0),
            run("b", 18.0),
        ])]),
        FRAME,
    );
    assert_eq!(layout.lines, 2);
    assert_eq!(baselines(&layout).len(), 2);
    assert!(!drawn_text(&layout).contains('\n'), "a break draws nothing");
}

#[test]
fn a_word_wider_than_the_line_breaks_between_characters() {
    let narrow = Frame {
        width: 40.0,
        ..FRAME
    };
    let layout = layout_body(
        &body(vec![paragraph(vec![run("Supercalifragilistic", 18.0)])]),
        narrow,
    );
    assert!(layout.lines > 1);
    assert_eq!(drawn_text(&layout), "Supercalifragilistic");
}

fn line_extent(layout: &TextLayout) -> (f32, f32) {
    let xs: Vec<f32> = layout
        .runs
        .iter()
        .flat_map(|r| r.glyphs.iter())
        .map(|g| g.x)
        .collect();
    (
        xs.iter().copied().fold(f32::INFINITY, f32::min),
        xs.iter().copied().fold(f32::NEG_INFINITY, f32::max),
    )
}

#[test]
fn alignment_moves_a_line_by_its_slack() {
    let laid = |align| {
        let mut p = paragraph(vec![run("Centre", 18.0)]);
        p.align = align;
        line_extent(&layout_body(&body(vec![p]), FRAME))
    };
    let (left, _) = laid(Align::Left);
    let (centre, centre_end) = laid(Align::Center);
    let (right, _) = laid(Align::Right);
    assert!((left - FRAME.x).abs() < 0.01);
    assert!(left < centre && centre < right);
    let middle = FRAME.x + FRAME.width / 2.0;
    assert!(
        centre < middle && centre_end > middle - 15.0,
        "centred about the middle: {centre}..{centre_end}"
    );
}

#[test]
fn justified_lines_fill_the_width_except_the_last() {
    let text = "aa bb cc dd ee ff gg hh ii jj kk ll mm nn oo pp qq rr ss tt";
    let mut p = paragraph(vec![run(text, 18.0)]);
    p.align = Align::Justified;
    let layout = layout_body(&body(vec![p]), FRAME);
    assert!(layout.lines >= 2);
    let ys = baselines(&layout);
    let last_x_on = |y: f32| {
        layout
            .runs
            .iter()
            .flat_map(|r| r.glyphs.iter())
            .filter(|g| (g.y - y).abs() < 0.01)
            .map(|g| g.x)
            .fold(f32::NEG_INFINITY, f32::max)
    };
    let right = FRAME.x + FRAME.width;
    assert!(
        right - last_x_on(ys[0]) < 15.0,
        "the first line reaches the right edge"
    );
    assert!(
        right - last_x_on(*ys.last().unwrap()) > 15.0,
        "the last line does not"
    );
}

#[test]
fn anchoring_moves_the_block_down_by_the_free_height() {
    let laid = |anchor| {
        let mut b = body(vec![paragraph(vec![run("Anchored", 18.0)])]);
        b.anchor = anchor;
        layout_body(&b, FRAME)
    };
    let top = laid(Anchor::Top);
    let middle = laid(Anchor::Middle);
    let bottom = laid(Anchor::Bottom);
    let free = FRAME.height - top.content_height;
    let y = |l: &TextLayout| l.runs[0].glyphs[0].y;
    assert!((y(&bottom) - y(&top) - free).abs() < 0.01);
    assert!((y(&middle) - y(&top) - free / 2.0).abs() < 0.01);
}

#[test]
fn double_line_spacing_doubles_the_distance_between_baselines() {
    let laid = |spacing| {
        let mut p = paragraph(vec![run("a", 18.0), run("\n", 18.0), run("b", 18.0)]);
        p.line_spacing = spacing;
        let ys = baselines(&layout_body(&body(vec![p]), FRAME));
        ys[1] - ys[0]
    };
    let single = laid(Spacing::Percent(1.0));
    let double = laid(Spacing::Percent(2.0));
    assert!((double - 2.0 * single).abs() < 0.01, "{single} {double}");
    assert!(
        single > 18.0 && single < 18.0 * 1.4,
        "single spacing is about 1.2 em: {single}"
    );
    assert!((laid(Spacing::Points(30.0)) - 30.0).abs() < 0.01);
}

#[test]
fn a_taller_east_asian_font_opens_the_lines_of_latin_text() {
    // Times New Roman text with `ea` Calibri, as `z18866178-doklad-ru` writes every
    // run: the lines are spaced by Calibri's height (Carlito, 1.22 em), not Times
    // New Roman's (Liberation Serif, 1.11 em).
    let pitch = |ea: Option<&str>| {
        let mut s = style(32.0);
        s.family = Some("Times New Roman".to_owned());
        s.east_asian_family = ea.map(str::to_owned);
        let t = |text: &str| StyledText {
            text: text.to_owned(),
            style: s.clone(),
        };
        let ys = baselines(&layout_body(
            &body(vec![paragraph(vec![t("a"), t("\n"), t("b")])]),
            FRAME,
        ));
        ys[1] - ys[0]
    };
    let own = pitch(None);
    let opened = pitch(Some("Calibri"));
    assert!((own / 32.0 - 1.107).abs() < 0.01, "{own}");
    assert!((opened / 32.0 - 1.221).abs() < 0.01, "{opened}");
    // One no taller changes nothing.
    assert!((pitch(Some("Times New Roman")) - own).abs() < 0.01);
}

#[test]
fn space_before_and_after_separate_paragraphs_but_not_the_ends() {
    let gap = |before: f32, after: f32| {
        let mut a = paragraph(vec![run("a", 18.0)]);
        a.space_after = Spacing::Points(after);
        a.space_before = Spacing::Points(100.0);
        let mut b = paragraph(vec![run("b", 18.0)]);
        b.space_before = Spacing::Points(before);
        let layout = layout_body(&body(vec![a, b]), FRAME);
        let ys = baselines(&layout);
        (ys[0] - FRAME.y, ys[1] - ys[0])
    };
    let (first_plain, plain) = gap(0.0, 0.0);
    let (first, spaced) = gap(6.0, 4.0);
    assert!((spaced - plain - 10.0).abs() < 0.01);
    assert!(
        (first - first_plain).abs() < 0.01 && first < 30.0,
        "the first paragraph's space before is not applied"
    );
}

#[test]
fn a_hanging_bullet_sits_at_the_indent_and_the_text_at_the_margin() {
    let mut p = paragraph(vec![run("Item", 18.0)]);
    p.margin_left = 30.0;
    p.indent = -30.0;
    p.bullet = Some(run("\u{2022}", 18.0));
    let layout = layout_body(&body(vec![p]), FRAME);
    assert_eq!(layout.runs.len(), 2, "bullet and text");
    assert_eq!(layout.runs[0].text, "\u{2022}");
    assert!((layout.runs[0].glyphs[0].x - FRAME.x).abs() < 0.01);
    assert!((layout.runs[1].glyphs[0].x - (FRAME.x + 30.0)).abs() < 0.01);
    assert_eq!(layout.runs[0].glyphs[0].y, layout.runs[1].glyphs[0].y);
}

#[test]
fn an_empty_paragraph_takes_height_from_its_mark_and_draws_no_bullet() {
    let mut empty = paragraph(Vec::new());
    empty.bullet = Some(run("\u{2022}", 18.0));
    empty.end = style(40.0);
    let layout = layout_body(&body(vec![empty, paragraph(vec![run("x", 18.0)])]), FRAME);
    assert_eq!(layout.lines, 2);
    assert_eq!(drawn_text(&layout), "x");
    assert!(
        layout.runs[0].glyphs[0].y - FRAME.y > 40.0,
        "below a 40 pt line"
    );
}

#[test]
fn runs_keep_their_size_colour_and_face() {
    let mut bold = run("bold", 24.0);
    bold.style.bold = true;
    bold.style.color = Rgba {
        r: 200,
        g: 10,
        b: 20,
        a: 255,
    };
    let layout = layout_body(
        &body(vec![paragraph(vec![run("plain ", 12.0), bold])]),
        FRAME,
    );
    assert_eq!(layout.runs.len(), 2);
    let (plain, bold) = (&layout.runs[0], &layout.runs[1]);
    assert_eq!(plain.size, 12.0);
    assert_eq!(bold.size, 24.0);
    assert_eq!(bold.color.r, 200);
    assert!(
        !core::ptr::eq(plain.face, bold.face),
        "bold has its own face"
    );
    assert_eq!(plain.glyphs[0].y, bold.glyphs[0].y, "one baseline");
    assert!(bold.glyphs[0].x > plain.glyphs[0].x);
}

#[test]
fn layout_is_deterministic() {
    let b = body(vec![paragraph(vec![run(
        "Same input, same glyphs, every time and on every host.",
        20.0,
    )])]);
    assert_eq!(layout_body(&b, FRAME), layout_body(&b, FRAME));
}

#[test]
fn hostile_sizes_and_frames_do_not_panic() {
    for (size, frame) in [
        (0.0, FRAME),
        (-5.0, FRAME),
        (f32::NAN, FRAME),
        (4000.0, FRAME),
        (
            18.0,
            Frame {
                width: -10.0,
                height: f32::INFINITY,
                ..FRAME
            },
        ),
    ] {
        let _ = layout_body(&body(vec![paragraph(vec![run("text here", size)])]), frame);
    }
}

#[test]
fn caret_stops_cover_every_character_boundary_of_each_line() {
    let b = body(vec![
        paragraph(vec![run("Hello ", 18.0), run("world", 18.0)]),
        paragraph(vec![]),
        paragraph(vec![run("a\nbc", 18.0)]),
    ]);
    let layout = layout_body(&b, FRAME);
    let lines: Vec<(usize, Vec<usize>)> = layout
        .carets
        .iter()
        .map(|l| (l.paragraph, l.stops.iter().map(|s| s.0).collect()))
        .collect();
    assert_eq!(
        lines,
        [
            (0, (0..=11).collect::<Vec<_>>()),
            (1, vec![0]),
            (2, vec![0, 1]),
            (2, vec![2, 3, 4]),
        ]
    );
    let first = &layout.carets[0];
    assert!(first.stops.windows(2).all(|w| w[1].1 > w[0].1));
    assert!((first.stops[0].1 - FRAME.x).abs() < 0.01);
    // The glyphs sit between each line's top and bottom.
    let glyph_y = layout.runs[0].glyphs[0].y;
    assert!(first.top < glyph_y && glyph_y < first.bottom);
    assert!(layout.carets[1].top >= first.bottom - 0.01);
}

#[test]
fn an_underlined_run_carries_a_line_under_its_glyphs_at_the_faces_underline_metrics() {
    let mut underlined = run("Under ", 20.0);
    underlined.style.underline = Some(UnderlineType::Sng);
    // Trailing spaces of the line are not underlined.
    let plain = run("plain ", 20.0);
    let layout = layout_body(&body(vec![paragraph(vec![underlined, plain])]), FRAME);
    let first = &layout.runs[0];
    let line = first.underline.expect("the underlined run has a line");
    assert_eq!(line.kind, UnderlineType::Sng);
    assert!((line.x0 - first.glyphs[0].x).abs() < 0.01, "{line:?}");
    assert!(line.x1 > first.glyphs.last().unwrap().x, "{line:?}");
    // Below the baseline, by a few percent of the size, and a sensible thickness.
    let baseline = first.glyphs[0].y;
    assert!(
        line.y > baseline && line.y < baseline + 20.0 * 0.3,
        "{line:?}"
    );
    assert!(
        line.thickness > 0.2 && line.thickness < 20.0 * 0.15,
        "{line:?}"
    );
    assert!(layout.runs[1].underline.is_none());
}

/// The x after the last glyph of `run`: its line's end, from the caret stops.
fn advance_of(text: &str, style: RunStyle) -> f32 {
    let layout = layout_body(
        &body(vec![paragraph(vec![StyledText {
            text: text.to_owned(),
            style,
        }])]),
        FRAME,
    );
    let stops = &layout.carets[0].stops;
    stops.last().unwrap().1 - stops[0].1
}

#[test]
fn a_struck_run_carries_a_line_through_its_glyphs_and_a_double_one_says_so() {
    let mut struck = run("Struck", 20.0);
    struck.style.strike = Some(StrikeKind::Single);
    let mut double = run(" twice", 20.0);
    double.style.strike = Some(StrikeKind::Double);
    let plain = run(" plain", 20.0);
    let layout = layout_body(&body(vec![paragraph(vec![struck, double, plain])]), FRAME);
    let first = &layout.runs[0];
    let line = first.strike.expect("the struck run has a line");
    assert_eq!(line.kind, StrikeKind::Single);
    assert!((line.x0 - first.glyphs[0].x).abs() < 0.01, "{line:?}");
    assert!(line.x1 > first.glyphs.last().unwrap().x, "{line:?}");
    // Above the baseline, through the lower case: between a tenth and a half of the size.
    let baseline = first.glyphs[0].y;
    assert!(
        line.y < baseline - 20.0 * 0.1 && line.y > baseline - 20.0 * 0.5,
        "{line:?}"
    );
    assert!(line.thickness > 0.2 && line.thickness < 20.0 * 0.15);
    assert_eq!(
        layout.runs[1].strike.map(|s| s.kind),
        Some(StrikeKind::Double)
    );
    assert!(layout.runs[2].strike.is_none());
}

#[test]
fn superscript_and_subscript_are_raised_and_lowered_and_drawn_at_two_thirds() {
    let plain = run("x", 30.0);
    let mut sup = run("2", 30.0);
    sup.style.baseline = 0.3;
    let mut sub = run("i", 30.0);
    sub.style.baseline = -0.25;
    let layout = layout_body(&body(vec![paragraph(vec![plain, sup, sub])]), FRAME);
    let line = layout.runs[0].glyphs[0].y;
    assert_eq!(layout.runs[0].size, 30.0);
    assert!(
        (layout.runs[1].size - 20.0).abs() < 0.01,
        "{:?}",
        layout.runs[1].size
    );
    assert!(
        (layout.runs[1].glyphs[0].y - (line - 9.0)).abs() < 0.01,
        "raised 30% of 30 pt"
    );
    assert!((layout.runs[2].size - 20.0).abs() < 0.01);
    assert!(
        (layout.runs[2].glyphs[0].y - (line + 7.5)).abs() < 0.01,
        "lowered 25% of 30 pt"
    );
    // Advanced at the smaller size too.
    let mut small = style(30.0);
    small.baseline = 0.3;
    assert!((advance_of("2222", small) - advance_of("2222", style(20.0))).abs() < 0.01);
}

#[test]
fn character_spacing_adds_its_points_after_every_character() {
    let tight = advance_of("Spaced", style(20.0));
    let mut loose = style(20.0);
    loose.spacing = 3.0;
    assert!((advance_of("Spaced", loose) - (tight + 6.0 * 3.0)).abs() < 0.01);
    let mut condensed = style(20.0);
    condensed.spacing = -1.0;
    assert!((advance_of("Spaced", condensed) - (tight - 6.0)).abs() < 0.01);
}
