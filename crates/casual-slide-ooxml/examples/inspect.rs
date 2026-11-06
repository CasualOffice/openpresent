//! Print what a package holds: `cargo run --example inspect -- deck.pptx`.

use casual_slide_ooxml::{OoxmlLimits, PresentationPackage};

fn main() {
    for path in std::env::args().skip(1) {
        let bytes = std::fs::read(&path).expect("readable file");
        match PresentationPackage::open(bytes, OoxmlLimits::default()) {
            Err(err) => println!("{path}: {err}"),
            Ok(pkg) => {
                println!(
                    "{path}: {:?} {:?} main={} parts={}",
                    pkg.kind(),
                    pkg.conformance(),
                    pkg.main_part(),
                    pkg.part_names().len()
                );
                for f in pkg.findings() {
                    println!("  {} {:?} {} {:?}", f.kind.code(), f.kind, f.part, f.rel_id);
                }
            }
        }
    }
}
