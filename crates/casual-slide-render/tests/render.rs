//! Real decks to display lists and PNGs. Runs on GitHub CI (owner instruction 38).

use std::path::{Path, PathBuf};

use casual_slide_render::display::{DisplayItem, Layer, Paint, Rgba};
use casual_slide_render::raster::pixel_size;
use casual_slide_render::{Deck, NoImages, render_pixmap};

fn generated(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/generated")
        .join(name)
}

fn open(name: &str) -> Deck {
    Deck::open(std::fs::read(generated(name)).unwrap()).unwrap()
}

fn pixel(png: &[u8], x: u32, y: u32) -> [u8; 4] {
    let p = tiny_skia::Pixmap::decode_png(png).unwrap();
    let c = p.pixel(x, y).unwrap().demultiply();
    [c.red(), c.green(), c.blue(), c.alpha()]
}

/// Every slide of every generated deck the readers open renders, at a low DPI so the
/// sweep stays quick. No panic, no error, the size the slide asks for.
#[test]
fn every_slide_of_every_generated_deck_renders() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/generated");
    let mut decks: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.starts_with("pp") || e.starts_with("po"))
        })
        .collect();
    decks.sort();
    let mut slides = 0usize;
    for path in &decks {
        let Ok(mut deck) = Deck::open(std::fs::read(path).unwrap()) else {
            continue;
        };
        for i in 0..deck.slide_count().min(20) {
            let (png, _) = deck
                .render_slide_png(i, 8.0, None)
                .unwrap_or_else(|e| panic!("{} slide {i}: {e}", path.display()));
            assert!(png.starts_with(&[0x89, b'P', b'N', b'G']));
            slides += 1;
        }
    }
    assert!(slides > 100, "only {slides} slides rendered");
}

#[test]
fn a_gradient_fill_reaches_the_display_list_and_the_pixels() {
    let mut deck = open("shape-fills.pptx");
    let (list, _) = deck.display_list(0).unwrap();
    let gradient = list.items.iter().any(|i| {
        matches!(
            i,
            DisplayItem::Path {
                fill: Some(Paint::Linear { .. }),
                layer: Layer::Slide,
                ..
            }
        )
    });
    assert!(gradient, "no linear gradient in {:?}", list.items.len());
    // "Linear down" sits at 150000,100000 EMU, 1400000 by 620000, red at its top.
    let (png, _) = deck.render_slide_png(0, 96.0, None).unwrap();
    let x = ((150_000.0 + 700_000.0) * 96.0 / 914_400.0) as u32;
    let y = ((100_000.0 + 20_000.0) * 96.0 / 914_400.0) as u32;
    let [r, g, b, _] = pixel(&png, x, y);
    assert!(r > 150 && g < 80 && b < 80, "{r},{g},{b}");
}

#[test]
fn text_is_laid_out_into_glyph_runs_and_drawn() {
    let mut deck = open("shape-fills.pptx");
    let (list, _) = deck.display_list(0).unwrap();
    let runs = list
        .items
        .iter()
        .filter(|i| matches!(i, DisplayItem::GlyphRun { glyphs, .. } if !glyphs.is_empty()))
        .count();
    assert!(runs > 0, "no glyph runs");
    let (_, report) = deck.render_slide_png(0, 24.0, None).unwrap();
    assert!(report.scene.text_bodies_drawn > 0);
    assert_eq!(report.glyph_runs_undrawn, 0);
}

/// A title on a white slide puts dark pixels inside its box once text is drawn.
#[test]
fn a_title_draws_dark_pixels_in_its_box() {
    let mut deck = open("image.pptx");
    let (png, report) = deck.render_slide_png(0, 96.0, None).unwrap();
    assert_eq!(report.glyphs_undrawn, 0, "{:?}", report.lines());
    let p = tiny_skia::Pixmap::decode_png(&png).unwrap();
    let px = |emu: f64| (emu * 96.0 / 914_400.0) as u32;
    // `Title 1`: 838200,365125, 10515600 by 1325563 EMU.
    let mut dark = 0;
    for y in px(365_125.0)..px(365_125.0 + 1_325_563.0) {
        for x in px(838_200.0)..px(838_200.0 + 10_515_600.0) {
            if let Some(c) = p.pixel(x, y) {
                let c = c.demultiply();
                if u32::from(c.red()) + u32::from(c.green()) + u32::from(c.blue()) < 300 {
                    dark += 1;
                }
            }
        }
    }
    assert!(dark > 50, "only {dark} dark pixels where the title is");
}

#[test]
fn a_picture_is_drawn_from_its_media_part() {
    let mut deck = open("area-pic.pptx");
    let (list, _) = deck.display_list(0).unwrap();
    let part = list.items.iter().find_map(|i| match i {
        DisplayItem::Path {
            fill: Some(Paint::Picture { part, .. }),
            ..
        } => Some(part.clone()),
        _ => None,
    });
    assert_eq!(part.as_deref(), Some("ppt/media/image1.png"));
    let (_, report) = deck.render_slide_png(0, 48.0, None).unwrap();
    assert!(report.images.drawn >= 1, "{:?}", report.images);
    assert!(report.images.is_complete(), "{:?}", report.images);
}

#[test]
fn a_picture_with_no_media_supplied_is_reported_by_part() {
    let deck = open("area-pic.pptx");
    let (list, _) = deck.display_list(0).unwrap();
    let (_, report) = render_pixmap(&list, 12.0, &NoImages, None).unwrap();
    assert_eq!(report.images.drawn, 0);
    assert_eq!(report.images.undrawn.len(), 1);
    assert_eq!(report.images.undrawn[0].part, "ppt/media/image1.png");
}

#[test]
fn the_master_background_reference_is_drawn_behind_everything() {
    let deck = open("slides-masters-layouts.pptx");
    let (list, _) = deck.display_list(0).unwrap();
    assert!(
        matches!(
            list.items.first(),
            Some(DisplayItem::Path {
                layer: Layer::Background,
                fill: Some(Paint::Solid(_)),
                ..
            })
        ),
        "{:?}",
        list.items.first()
    );
}

#[test]
fn a_layout_background_fill_is_read_and_drawn() {
    let deck = open("slides-masters-layouts.pptx");
    let layout = deck
        .parts()
        .structure
        .parts
        .iter()
        .find(|e| e.part == "ppt/slideLayouts/slideLayout3.xml")
        .unwrap()
        .id
        .part;
    let (list, _) = deck.display_list_of_part(layout);
    let Some(DisplayItem::Path {
        layer: Layer::Background,
        fill: Some(Paint::Solid(c)),
        ..
    }) = list.items.first()
    else {
        panic!("no background: {:?}", list.items.first());
    };
    assert_eq!(
        *c,
        Rgba {
            r: 0x1f,
            g: 0x1f,
            b: 0x1f,
            a: 255
        }
    );
}

#[test]
fn a_slide_background_of_no_fill_overrides_its_layout() {
    let deck = open("slides-masters-layouts.pptx");
    let (list, _) = deck.display_list(2).unwrap();
    assert!(!list.items.iter().any(|i| matches!(
        i,
        DisplayItem::Path {
            layer: Layer::Background,
            ..
        }
    )));
}

#[test]
fn preset_shapes_become_paths_at_the_slide_size() {
    let mut deck = open("preset-shapes.pptx");
    let (list, _) = deck.display_list(0).unwrap();
    assert!(list.items.len() > 1);
    let (png, _) = deck.render_slide_png(0, 24.0, None).unwrap();
    let p = tiny_skia::Pixmap::decode_png(&png).unwrap();
    assert_eq!((p.width(), p.height()), pixel_size(&list, 24.0));
}

#[test]
fn a_slide_index_past_the_end_is_an_error_with_a_code() {
    let mut deck = open("minimal.pptx");
    let err = deck.render_slide_png(99, 12.0, None).unwrap_err();
    assert_eq!(err.code(), "CS-RND-0003");
}

/// A table is drawn, not skipped: its cells fill, and the report no longer names it.
#[test]
fn a_table_draws_its_cells_and_is_not_reported_as_skipped() {
    let mut deck = open("table-merged-cells.pptx");
    let (list, scene) = deck.display_list(0).unwrap();
    assert!(
        !scene.features.contains_key("table"),
        "{:?}",
        scene.features
    );
    let filled = list
        .items
        .iter()
        .filter(|i| matches!(i, DisplayItem::Path { fill: Some(_), .. }))
        .count();
    assert!(filled >= 4, "only {filled} filled items");
    // The merged origin cell, 2000000 by 740000 EMU at the frame's corner, is
    // filled in the fixture style's near-black.
    let (png, _) = deck.render_slide_png(0, 96.0, None).unwrap();
    let px = |emu: f64| (emu * 96.0 / 914_400.0) as u32;
    let [r, g, b, _] = pixel(&png, px(500_000.0), px(150_000.0));
    assert!(r < 60 && g < 60 && b < 60, "{r},{g},{b}");
}

/// Classic charts are drawn from their cached values: bars, lines, areas, pies and
/// scatters reach the display list, and the report names only the families left.
#[test]
fn classic_charts_draw_from_their_cached_values() {
    let deck = open("chart-families.pptx");
    let (list, scene) = deck.display_list(0).unwrap();
    for key in scene.features.keys() {
        assert!(
            *key != "chart" && !key.starts_with("chart (part") && !key.starts_with("chart (rel"),
            "{:?}",
            scene.features
        );
    }
    let filled = list
        .items
        .iter()
        .filter(|i| matches!(i, DisplayItem::Path { fill: Some(_), .. }))
        .count();
    let stroked = list
        .items
        .iter()
        .filter(|i| {
            matches!(
                i,
                DisplayItem::Path {
                    fill: None,
                    stroke: Some(_),
                    ..
                }
            )
        })
        .count();
    assert!(
        filled > 30 && stroked > 10,
        "{filled} filled, {stroked} stroked"
    );
}

/// SmartArt is drawn from the drawing PowerPoint stored for it, not reported.
#[test]
fn a_diagram_is_drawn_from_its_stored_drawing() {
    let deck = open("graphics-diagram-consistent.pptx");
    let (list, scene) = deck.display_list(0).unwrap();
    assert!(
        !scene.features.keys().any(|k| k.starts_with("diagram")),
        "{:?}",
        scene.features
    );
    let drawn = list
        .items
        .iter()
        .filter(|i| {
            matches!(
                i,
                DisplayItem::Path {
                    layer: Layer::Slide,
                    ..
                }
            )
        })
        .count();
    assert!(drawn > 0, "nothing of the diagram drawn");
}

/// An OLE object is drawn from the picture stored in its place, never opened.
#[test]
fn an_ole_object_is_drawn_from_its_fallback_picture() {
    let deck = open("area-ole.pptx");
    let (list, scene) = deck.display_list(0).unwrap();
    assert!(
        !scene.features.contains_key("graphic frame"),
        "{:?}",
        scene.features
    );
    assert!(list.items.iter().any(|i| matches!(
        i,
        DisplayItem::Path {
            fill: Some(Paint::Picture { .. }),
            ..
        }
    )));
}

/// An outer shadow is drawn behind its shape as translucent paint, and the slide no
/// longer reports its effects as skipped.
#[test]
fn an_outer_shadow_is_drawn_behind_its_shape() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/real/z19828779-education-ar.pptx");
    let deck = Deck::open(std::fs::read(path).unwrap()).unwrap();
    let (list, scene) = deck.display_list(0).unwrap();
    assert!(
        !scene.features.contains_key("effects"),
        "{:?}",
        scene.features
    );
    let translucent = list.items.iter().any(|i| {
        matches!(
            i,
            DisplayItem::Blurred {
                color: Rgba { a, .. },
                filled: true,
                sigma,
                ..
            } if *a < 255 && *sigma > 0.0
        )
    });
    assert!(translucent, "no blurred shadow drawn");
}

/// `normAutofit` draws at the scale the file stores: a 10 pt run under
/// `fontScale="62500"` is drawn at 6.25 pt.
#[test]
fn norm_autofit_draws_text_at_the_stored_font_scale() {
    let deck = open("area-text.pptx");
    let (list, _) = deck.display_list(2).unwrap();
    // `Text 55` sits at 8656320,4361688 EMU.
    let sizes: Vec<f64> = list
        .items
        .iter()
        .filter_map(|i| match i {
            DisplayItem::GlyphRun {
                transform, size, ..
            } if (transform.e - 8_656_320.0).abs() < 1.0
                && (transform.f - 4_361_688.0).abs() < 1.0 =>
            {
                Some(*size)
            }
            _ => None,
        })
        .collect();
    assert!(!sizes.is_empty(), "no text in the autofit box");
    for s in sizes {
        assert!((s - 6.25 * 12_700.0).abs() < 1.0, "drawn at {s} EMU");
    }
}

/// Light and dark pixels along row `y` from `x0` to `x1`: how often it changes
/// between them.
fn transitions(png: &[u8], y: u32, x0: u32, x1: u32) -> usize {
    let dark = |x| {
        pixel(png, x, y)[..3]
            .iter()
            .map(|c| u32::from(*c))
            .sum::<u32>()
            < 3 * 128
    };
    (x0..x1).filter(|x| dark(*x) != dark(x + 1)).count()
}

/// A pattern fill draws its cells, foreground on background, not one blended colour
/// (FH-01: `area-fill` slide 1 drew every preset as a flat blend).
#[test]
fn a_pattern_fill_draws_foreground_and_background_cells() {
    let mut deck = open("area-fill.pptx");
    let (png, _) = deck.render_slide_png(0, 96.0, None).unwrap();
    // `vert` is the fifth preset in the first row: vertical lines across it.
    assert!(transitions(&png, 115, 590, 690) > 10, "no lines in `vert`");
    // `pct90` is nearly all foreground, `pct5` nearly all background.
    let [r, g, b, _] = pixel(&png, 92, 110);
    assert!(r > 200 && g > 200 && b > 200, "pct5 is {r},{g},{b}");
}

/// A tiled picture fill repeats the picture at its own size rather than stretching
/// it over the box (FH-01: `area-fill` slide 3 drew every tile stretched).
#[test]
fn a_tiled_picture_fill_repeats_the_picture() {
    let mut deck = open("area-fill.pptx");
    let (png, _) = deck.render_slide_png(2, 96.0, None).unwrap();
    // "stretch" at the top left: a few cells across. "tile flip none" beside it:
    // the picture at its own size, many times over.
    let stretched = transitions(&png, 165, 35, 260);
    let tiled = transitions(&png, 165, 282, 505);
    assert!(stretched < 10, "stretch changed {stretched} times");
    assert!(
        tiled > 3 * stretched.max(4),
        "tile changed only {tiled} times"
    );
}

/// `bodyPr/@vert="vert"` turns the text a quarter turn: its ink is taller than wide
/// (FH-03: `area-text` drew every vertical kind horizontal).
#[test]
fn vertical_text_runs_down_the_box() {
    let mut deck = open("area-text.pptx");
    let (png, _) = deck.render_slide_png(0, 96.0, None).unwrap();
    // "vert vert", third box of the third row.
    let (mut x0, mut x1, mut y0, mut y1) = (u32::MAX, 0, u32::MAX, 0);
    for y in 335..425 {
        for x in 385..543 {
            if pixel(&png, x, y)[0] < 128 {
                (x0, x1, y0, y1) = (x0.min(x), x1.max(x), y0.min(y), y1.max(y));
            }
        }
    }
    assert!(x1 > x0 && y1 > y0, "no text in the box");
    assert!(y1 - y0 > 3 * (x1 - x0), "ink {}x{}", x1 - x0, y1 - y0);
}

fn open_real(name: &str) -> Deck {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/real")
        .join(name);
    Deck::open(std::fs::read(path).unwrap()).unwrap()
}

/// A Japanese deck draws its text once the CJK slices holding the deck's characters
/// are loaded (T2D5-02; FH-01 found `z21464709-souzou-ja` drawn with no text).
#[test]
fn a_japanese_deck_draws_its_text_from_the_slices_its_characters_name() {
    use casual_slide_text::cjk::{Region, SliceDirectory, Style, Variant};
    let mut deck = open_real("z21464709-souzou-ja.pptx");
    let dir =
        SliceDirectory::open(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../webapp/fonts/cjk"))
            .unwrap();
    let jp = Variant::new(Region::Jp, Style::Sans, false);
    assert!(dir.load(deck.characters().into_iter().map(|c| (jp, c))) > 0);
    let (png, _) = deck.render_slide_png(0, 96.0, None).unwrap();
    // The title's first five characters, left of its colon, at 960 by 720.
    let mut ink = 0;
    for y in (70..120).step_by(2) {
        for x in (200..370).step_by(2) {
            if pixel(&png, x, y)[0] < 100 {
                ink += 1;
            }
        }
    }
    assert!(ink > 100, "{ink} dark pixels in the title");
}

/// A group's scale places its children but never thickens their lines. This deck's
/// groups map a child space of master units onto EMU, 1587.5 times; scaling a 4.5 pt
/// line by that painted the slide black (FH-01, gallery run 42492516147).
#[test]
fn a_group_scale_does_not_thicken_its_childrens_lines() {
    let mut deck = open_real("z21464709-souzou-ja.pptx");
    let (png, _) = deck.render_slide_png(0, 96.0, None).unwrap();
    // Above the title and left of the diagram, the slide's white background shows.
    for (x, y) in [(20, 20), (480, 10), (300, 680), (940, 300)] {
        let [r, g, b, _] = pixel(&png, x, y);
        assert!(r > 200 && g > 200 && b > 200, "({x},{y}) is {r},{g},{b}");
    }
}

/// A glow fades out of its shape as a Gaussian does: smoothly, a little every pixel,
/// with no flat bands. Stepped rings gave bands two or three pixels wide.
#[test]
fn a_glow_fades_smoothly_like_a_gaussian_blur() {
    let mut deck = open("area-effect.pptx");
    let (png, _) = deck.render_slide_png(0, 96.0, None).unwrap();
    // `glow 7`: its top edge at 685800 EMU (72 px), its middle at 4419600 EMU
    // (464 px); an orange glow 139700 EMU (14.7 px) wide, read upwards.
    let blue = |y: u32| 255 - u32::from(pixel(&png, 464, y)[2]);
    let profile: Vec<u32> = (59..71).rev().map(blue).collect();
    assert!(
        profile[0] > 40,
        "the glow is drawn beside the shape: {profile:?}"
    );
    assert!(
        profile.windows(2).all(|w| w[1] < w[0]),
        "it falls every pixel, with no flat band: {profile:?}"
    );
    let steps: Vec<u32> = profile.windows(2).map(|w| w[0] - w[1]).collect();
    let biggest = steps.iter().copied().max().unwrap_or(0);
    assert!(
        biggest * 3 < profile[0],
        "no one step takes a third of it: {profile:?}"
    );
}
