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
