//! `render-deck <in.pptx> <outdir> [dpi]`: one PNG per slide, `slide-001.png` and on,
//! and a `report.txt` saying what each slide did not draw.
//!
//! Run with `cargo run --release -p casual-slide-render --example render-deck`.
//! Exits non-zero, naming the problem, if the deck cannot be opened or a file
//! cannot be written; a slide that fails to render is reported and the rest go on.

use std::fmt::Write as _;
use std::path::PathBuf;
use std::process::ExitCode;

use casual_slide_render::Deck;
use casual_slide_text::cjk::{Region, SliceDirectory, Style, Variant};

/// Load the CJK slices (`webapp/fonts/cjk`, T2D5-02) holding the deck's characters,
/// in every region and style, as a host fetches them before drawing. Without the
/// directory, CJK text is reported as having no face, as before.
fn load_cjk(deck: &Deck) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../webapp/fonts/cjk");
    let Ok(dir) = SliceDirectory::open(root) else {
        return;
    };
    let chars = deck.characters();
    let mut wanted = Vec::new();
    for region in Region::ALL {
        for (style, bold) in [
            (Style::Sans, false),
            (Style::Sans, true),
            (Style::Serif, false),
        ] {
            let v = Variant::new(region, style, bold);
            wanted.extend(chars.iter().map(|c| (v, *c)));
        }
    }
    dir.load(wanted);
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (Some(input), Some(outdir)) = (args.first(), args.get(1)) else {
        eprintln!("usage: render-deck <in.pptx> <outdir> [dpi]");
        return ExitCode::from(2);
    };
    let dpi = match args.get(2).map(|d| d.parse::<f64>()) {
        None => 96.0,
        Some(Ok(d)) if d > 0.0 && d <= 600.0 => d,
        Some(_) => {
            eprintln!("render-deck: dpi must be a number from 1 to 600");
            return ExitCode::from(2);
        }
    };
    let bytes = match std::fs::read(input) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("render-deck: cannot read {input}: {e}");
            return ExitCode::FAILURE;
        }
    };
    let mut deck = match Deck::open(bytes) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("render-deck: {input}: {e}");
            return ExitCode::FAILURE;
        }
    };
    load_cjk(&deck);
    let outdir = PathBuf::from(outdir);
    if let Err(e) = std::fs::create_dir_all(&outdir) {
        eprintln!("render-deck: cannot create {}: {e}", outdir.display());
        return ExitCode::FAILURE;
    }
    let mut report = String::new();
    let mut failed = 0usize;
    for index in 0..deck.slide_count() {
        let name = format!("slide-{:03}.png", index + 1);
        let _ = writeln!(report, "{name}");
        match deck.render_slide_png(index, dpi, None) {
            Ok((png, r)) => {
                let path = outdir.join(&name);
                if let Err(e) = std::fs::write(&path, png) {
                    eprintln!("render-deck: cannot write {}: {e}", path.display());
                    return ExitCode::FAILURE;
                }
                for line in r.lines() {
                    let _ = writeln!(report, "  {line}");
                }
            }
            Err(e) => {
                failed += 1;
                let _ = writeln!(report, "  not rendered: {e}");
            }
        }
    }
    if deck.themes_unread > 0 {
        let _ = writeln!(report, "themes not read: {}", deck.themes_unread);
    }
    let path = outdir.join("report.txt");
    if let Err(e) = std::fs::write(&path, report) {
        eprintln!("render-deck: cannot write {}: {e}", path.display());
        return ExitCode::FAILURE;
    }
    println!(
        "render-deck: {} of {} slides rendered to {}",
        deck.slide_count().saturating_sub(failed),
        deck.slide_count(),
        outdir.display()
    );
    ExitCode::SUCCESS
}
