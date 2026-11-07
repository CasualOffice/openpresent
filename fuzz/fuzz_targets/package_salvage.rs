//! `REC-01`: the recovering ZIP reader never panics, refuses only what holds nothing, and
//! writes an image the strict reader admits and can read every entry of.
#![no_main]

use casual_slide_package::{salvage, Package, PackageLimits};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let limits = PackageLimits {
        // Small enough that a run spends its time on structure, and not on inflating.
        max_total_uncompressed: 16 << 20,
        max_entries: 4_096,
        ..PackageLimits::default()
    };
    match salvage(data, limits, |_, bytes| !bytes.is_empty()) {
        Err(err) => assert_eq!(err.code(), "CS-PKG-0005", "{err}"),
        Ok(salvaged) => {
            let mut image = Package::open(salvaged.image, limits).expect("the image is admitted");
            assert!(image.verify_entries(), "every entry in the image reads");
            for name in image.entry_names() {
                image.read_part(&name).expect("every part in the image reads");
            }
        }
    }
});
