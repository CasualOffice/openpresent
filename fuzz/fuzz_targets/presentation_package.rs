//! Opening any bytes as a presentation refuses cleanly or produces a graph, and an
//! unedited save is the input.
#![no_main]

use casual_slide_ooxml::{OoxmlLimits, PresentationPackage};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(package) = PresentationPackage::open(data.to_vec(), OoxmlLimits::default()) {
        assert_eq!(package.save_unedited(), data);
    }
});
