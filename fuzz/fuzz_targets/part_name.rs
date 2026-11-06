//! Part-name equivalence never panics, and a name it accepts has no segment that
//! means traversal.
#![no_main]

use casual_slide_ooxml::{part_name_key, part_names_equivalent};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(name) = std::str::from_utf8(data) else {
        return;
    };
    if let Some(key) = part_name_key(name) {
        assert!(key
            .split('/')
            .all(|s| !s.is_empty() && !s.ends_with('.') && !s.contains('\\')));
        assert!(part_names_equivalent(name, name));
    }
});
