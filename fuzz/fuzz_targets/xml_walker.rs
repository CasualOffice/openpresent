//! The one XML entry point never panics, and every element it reports is within
//! the depth bound.
#![no_main]

use casual_slide_ooxml::{walk, OoxmlLimits};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let limits = OoxmlLimits {
        max_xml_depth: 32,
        ..OoxmlLimits::default()
    };
    let _ = walk(data, "fuzz", &limits, |_, depth| {
        assert!(depth <= 32);
        Ok(())
    });
});
