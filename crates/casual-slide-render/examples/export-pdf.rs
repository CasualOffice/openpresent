//! `export-pdf <in.pptx> <out.pdf>`: every slide as one PDF page, with real text,
//! and on standard error what each slide did not draw or drew approximately.
//!
//! Run with `cargo run --release -p casual-slide-render --example export-pdf`.
//! Exits non-zero, naming the problem, if the deck cannot be opened or the file
//! cannot be written.

use std::process::ExitCode;

use casual_slide_render::{Deck, PdfMetadata};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (Some(input), Some(output)) = (args.first(), args.get(1)) else {
        eprintln!("usage: export-pdf <in.pptx> <out.pdf>");
        return ExitCode::from(2);
    };
    let bytes = match std::fs::read(input) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("export-pdf: cannot read {input}: {e}");
            return ExitCode::FAILURE;
        }
    };
    let mut deck = match Deck::open(bytes) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("export-pdf: {input}: {e}");
            return ExitCode::FAILURE;
        }
    };
    let (pdf, reports) = match deck.render_pdf(&PdfMetadata::default()) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("export-pdf: {input}: {e}");
            return ExitCode::FAILURE;
        }
    };
    if let Err(e) = std::fs::write(output, pdf) {
        eprintln!("export-pdf: cannot write {output}: {e}");
        return ExitCode::FAILURE;
    }
    for (index, report) in reports.iter().enumerate() {
        for line in report.lines() {
            eprintln!("slide {}: {line}", index + 1);
        }
    }
    println!("export-pdf: {} pages to {output}", reports.len());
    ExitCode::SUCCESS
}
