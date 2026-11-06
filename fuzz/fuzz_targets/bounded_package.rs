//! Package admission never panics; reads stay bounded; a rewrite of what was
//! admitted is admitted again.
#![no_main]

use casual_slide_package::{Package, PackageLimits, PartEdits};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(mut package) = Package::open(data.to_vec(), PackageLimits::default()) {
        for name in package.entry_names() {
            let _ = package.read_part(&name);
        }
        assert_eq!(package.rewrite(&PartEdits::default()).unwrap(), data);
    }
});
