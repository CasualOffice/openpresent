//! Admission tests, including hostile inputs that must be rejected within
//! limits (`docs/21-PARSER-LIMITS.md`, `docs/15-CI-AND-RELEASE-GATES.md`).

use std::io::{Cursor, Write};

use zip::CompressionMethod;
use zip::write::{SimpleFileOptions, ZipWriter};

use crate::{Package, PackageError, PackageLimits};

/// Build an in-memory ZIP from `(name, bytes)` entries, deflate-compressed.
fn make_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    for (name, data) in entries {
        writer.start_file(*name, opts).unwrap();
        writer.write_all(data).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

#[test]
fn admits_a_valid_package_and_reads_parts() {
    let bytes = make_zip(&[
        ("[Content_Types].xml", b"<Types/>"),
        ("xl/workbook.xml", b"<workbook/>"),
    ]);
    let mut pkg = Package::open(bytes, PackageLimits::default()).unwrap();

    assert_eq!(pkg.len(), 2);
    assert!(!pkg.is_empty());
    assert!(pkg.contains("xl/workbook.xml"));
    assert!(!pkg.contains("xl/missing.xml"));

    let names = pkg.entry_names();
    assert!(names.iter().any(|n| n == "xl/workbook.xml"));

    let part = pkg.read_part("xl/workbook.xml").unwrap();
    assert_eq!(part, b"<workbook/>");

    let entries = pkg.entries().unwrap();
    assert_eq!(entries.len(), 2);
}

#[test]
fn rejects_non_package() {
    let err = Package::open(b"not a zip at all".to_vec(), PackageLimits::default()).unwrap_err();
    assert_eq!(err, PackageError::NotAPackage);
    assert_eq!(err.code(), "CS-PKG-0005");
}

#[test]
fn rejects_oversized_input() {
    let bytes = make_zip(&[("a.xml", b"hello")]);
    let limits = PackageLimits {
        max_input_bytes: 8, // smaller than any real zip
        ..PackageLimits::default()
    };
    let err = Package::open(bytes, limits).unwrap_err();
    assert!(matches!(err, PackageError::InputTooLarge { .. }));
    assert_eq!(err.code(), "CS-PKG-0001");
}

#[test]
fn rejects_too_many_entries() {
    let bytes = make_zip(&[("a.xml", b"a"), ("b.xml", b"b"), ("c.xml", b"c")]);
    let limits = PackageLimits {
        max_entries: 2,
        ..PackageLimits::default()
    };
    let err = Package::open(bytes, limits).unwrap_err();
    assert!(matches!(
        err,
        PackageError::TooManyEntries { count: 3, limit: 2 }
    ));
    assert_eq!(err.code(), "CS-PKG-0002");
}

#[test]
fn rejects_zip_bomb_by_expansion_ratio() {
    // 1 MiB of zeros compresses to a tiny deflate stream → very high ratio.
    let payload = vec![0u8; 1 << 20];
    let bytes = make_zip(&[("bomb.bin", &payload)]);
    let limits = PackageLimits {
        max_expansion_ratio: 100,
        ..PackageLimits::default()
    };
    let err = Package::open(bytes, limits).unwrap_err();
    assert!(matches!(err, PackageError::ExpansionRatioExceeded { .. }));
    assert_eq!(err.code(), "CS-PKG-0003");
}

#[test]
fn rejects_expansion_over_total_cap() {
    let payload = vec![0u8; 1 << 20]; // 1 MiB uncompressed
    let bytes = make_zip(&[("big.bin", &payload)]);
    let limits = PackageLimits {
        max_total_uncompressed: 4096,  // 4 KiB cap
        max_expansion_ratio: u64::MAX, // isolate the total-size check
        ..PackageLimits::default()
    };
    let err = Package::open(bytes, limits).unwrap_err();
    assert!(matches!(err, PackageError::ExpansionTooLarge { .. }));
    assert_eq!(err.code(), "CS-PKG-0003");
}

#[test]
fn rejects_path_traversal() {
    let bytes = make_zip(&[("../../etc/passwd", b"secret")]);
    let err = Package::open(bytes, PackageLimits::default()).unwrap_err();
    assert!(matches!(err, PackageError::UnsafePath { .. }));
    assert_eq!(err.code(), "CS-PKG-0004");
}

#[test]
fn missing_part_is_reported() {
    let bytes = make_zip(&[("a.xml", b"a")]);
    let mut pkg = Package::open(bytes, PackageLimits::default()).unwrap();
    let err = pkg.read_part("nope.xml").unwrap_err();
    assert!(matches!(err, PackageError::PartNotFound { .. }));
    assert_eq!(err.code(), "CS-PKG-0006");
}

/// Rewrite every declared uncompressed size in `zip` to `claim`.
///
/// Four bytes in the central directory and four in each local header — the
/// whole of the "defence" that stood between a 1 MB file and a gigabyte of
/// memory. Patched here rather than described, because a security bound that
/// has only been reasoned about is a security bound nobody has tested.
fn lie_about_sizes(mut zip: Vec<u8>, claim: u32) -> Vec<u8> {
    let mut i = 0;
    while i + 30 <= zip.len() {
        // Central directory header: uncompressed size at offset 24.
        if zip[i..i + 4] == [b'P', b'K', 1, 2] && i + 28 <= zip.len() {
            zip[i + 24..i + 28].copy_from_slice(&claim.to_le_bytes());
            i += 4;
            continue;
        }
        // Local file header: uncompressed size at offset 22.
        if zip[i..i + 4] == [b'P', b'K', 3, 4] && i + 26 <= zip.len() {
            zip[i + 22..i + 26].copy_from_slice(&claim.to_le_bytes());
            i += 4;
            continue;
        }
        i += 1;
    }
    zip
}

/// **A part may not produce more than it declared.**
///
/// Admission adds up `entry.size()` across the archive to refuse an oversized
/// package or a suspicious expansion ratio — and every one of those numbers is
/// the attacker's. With honest headers this payload is rejected as a bomb.
/// Claiming 100 bytes instead got it admitted, and `read_part` then expanded
/// the real stream in full, because the only ceiling it applied was the
/// *whole package's*. Since unmodelled parts are retained in the model, those
/// bytes then stayed resident.
#[test]
fn a_part_that_produces_more_than_it_declared_is_refused() {
    let payload = vec![0u8; 1 << 20]; // 1 MiB of zeros, deflates to almost nothing
    let honest = make_zip(&[("[Content_Types].xml", b"<Types/>"), ("big.bin", &payload)]);

    // With honest headers the ratio check catches it, which is the behaviour
    // the patched version is escaping.
    let strict = PackageLimits {
        max_expansion_ratio: 100,
        ..PackageLimits::default()
    };
    assert!(
        matches!(
            Package::open(honest.clone(), strict),
            Err(PackageError::ExpansionRatioExceeded { .. })
        ),
        "the honest package is refused, so the lie is what buys entry"
    );

    // Now claim 100 bytes. Admission sees a tiny, unremarkable archive.
    let patched = lie_about_sizes(honest, 100);
    let mut pkg =
        Package::open(patched, strict).expect("the lie gets it past admission, as it must");

    // Mapped to its length before asserting: when this fails it fails by
    // *returning the megabyte*, and a megabyte in the assertion message buries
    // the result it is trying to report.
    let outcome = pkg.read_part("big.bin").map(|part| part.len());
    let err = outcome
        .expect_err("a part that produces ten thousand times what it declared must be refused");
    assert!(
        matches!(err, PackageError::ExpansionTooLarge { .. }),
        "got {err:?}"
    );
    assert_eq!(err.code(), "CS-PKG-0003");
}

// ---- Threat model probes (docs/07-SECURITY-THREAT-MODEL.md) -------------------

/// Rewrite one part name everywhere it occurs (local header and central
/// directory), which is the only way to make a ZIP a well-behaved writer refuses
/// to produce.
fn rename(zip: &[u8], from: &[u8], to: &[u8]) -> Vec<u8> {
    assert_eq!(from.len(), to.len());
    let mut out = zip.to_vec();
    let mut i = 0;
    while i + from.len() <= out.len() {
        if &out[i..i + from.len()] == from {
            out[i..i + from.len()].copy_from_slice(to);
        }
        i += 1;
    }
    out
}

fn is_refused(bytes: Vec<u8>) -> bool {
    Package::open(bytes, PackageLimits::default()).is_err()
}

/// Two entries with one name: one reader sees the first and another the last, so
/// what a scanner approved need not be what the editor opens. SEC-P05.
#[test]
fn a_duplicate_part_name_is_refused() {
    let zip = make_zip(&[("ppt/one.xml", b"<safe/>"), ("ppt/two.xml", b"<evil/>")]);
    assert!(is_refused(rename(&zip, b"ppt/two.xml", b"ppt/one.xml")));
}

/// OPC treats part names as case-insensitively equivalent. SEC-P06.
#[test]
fn parts_differing_only_in_case_are_refused() {
    let zip = make_zip(&[("ppt/slide.xml", b"<a/>"), ("PPT/Slide.xml", b"<b/>")]);
    assert!(is_refused(zip));
}

/// OPC part-name grammar, enforced so a retained part cannot smuggle a name that
/// means something else to the next reader. SEC-P07.
#[test]
fn malformed_part_names_are_refused() {
    for name in [
        "ppt/./slide.xml",
        "ppt//slide.xml",
        "ppt/slide.xml.",
        "ppt/sl\u{0}ide.xml",
        "ppt/%2e%2e/slide.xml",
        "ppt/%2F/slide.xml",
        "ppt/sl\u{1}ide.xml",
    ] {
        assert!(
            is_refused(make_zip(&[(name, b"<a/>")])),
            "admitted {name:?}"
        );
    }
}

/// An ordinary percent-encoded name is valid OPC and must stay admitted.
#[test]
fn an_ordinary_percent_encoded_name_is_admitted() {
    assert!(!is_refused(make_zip(&[("ppt/media/my%20image.png", b"x")])));
}

/// Allocation up front follows the declared size, which the attacker wrote. In
/// WebAssembly a failed allocation aborts the whole instance. SEC-P04.
#[test]
fn up_front_allocation_does_not_trust_the_declared_size() {
    assert_eq!(crate::package::initial_capacity(100), 100);
    assert!(crate::package::initial_capacity(3 << 30) <= 1 << 20);
}

/// A forged directory: records the count does not admit to, hidden from a reader
/// that trusts the count but visible to one that walks the bytes. SEC-P05.
#[test]
fn a_directory_count_that_disagrees_with_its_records_is_refused() {
    let mut zip = make_zip(&[("ppt/one.xml", b"<a/>"), ("ppt/two.xml", b"<b/>")]);
    let eocd = zip.len() - 22;
    zip[eocd + 10] = 1; // entries on this disk
    zip[eocd + 8] = 1; // entries in total, as the crate reads either
    assert!(is_refused(zip));
}

/// A ZIP64 end record must be followed the same way. Built by hand, since the
/// writer only emits one for archives too large for a test.
#[test]
fn record_count_follows_a_zip64_end_record() {
    let zip = make_zip(&[("ppt/one.xml", b"<a/>"), ("ppt/two.xml", b"<b/>")]);
    let eocd = zip.len() - 22;
    let cd_size = u32::from_le_bytes(zip[eocd + 12..eocd + 16].try_into().unwrap()) as u64;
    let mut out = zip[..eocd].to_vec();
    let record = out.len() as u64;
    out.extend_from_slice(&[0x50, 0x4b, 0x06, 0x06]);
    out.extend_from_slice(&44u64.to_le_bytes());
    out.extend_from_slice(&[0; 12]);
    out.extend_from_slice(&2u64.to_le_bytes());
    out.extend_from_slice(&2u64.to_le_bytes());
    out.extend_from_slice(&cd_size.to_le_bytes());
    out.extend_from_slice(&0u64.to_le_bytes());
    out.extend_from_slice(&[0x50, 0x4b, 0x06, 0x07, 0, 0, 0, 0]);
    out.extend_from_slice(&record.to_le_bytes());
    out.extend_from_slice(&1u32.to_le_bytes());
    let mut end = zip[eocd..].to_vec();
    end[8..12].copy_from_slice(&[0xFF; 4]);
    end[12..16].copy_from_slice(&[0xFF; 4]);
    out.extend_from_slice(&end);
    assert_eq!(crate::directory::record_count(&out), Some(2));
}

// ---- Rewriting (docs/28-PACKAGE-AND-PART-GRAPH.md) ------------------------------

fn raw_stream(zip: &[u8], name: &str) -> Vec<u8> {
    use std::io::Read;
    let mut archive = zip::ZipArchive::new(Cursor::new(zip)).unwrap();
    let mut entry = archive.by_name_raw(name.as_bytes()).unwrap();
    let mut out = Vec::new();
    entry.read_to_end(&mut out).unwrap();
    out
}

#[test]
fn a_rewrite_with_no_edits_returns_the_source_bytes() {
    let zip = make_zip(&[("a.xml", b"<a/>"), ("b.xml", b"<b/>")]);
    let mut pkg = Package::open(zip.clone(), PackageLimits::default()).unwrap();
    assert_eq!(pkg.source_bytes(), zip.as_slice());
    assert_eq!(pkg.rewrite(&crate::PartEdits::default()).unwrap(), zip);
}

#[test]
fn a_rewrite_keeps_untouched_parts_compressed_bytes_and_applies_edits() {
    // Stored rather than deflated, so a recompressing rewrite would change the stream.
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    writer.start_file("keep.xml", stored).unwrap();
    writer.write_all(b"<keep>same</keep>").unwrap();
    writer.start_file("edit.xml", stored).unwrap();
    writer.write_all(b"<old/>").unwrap();
    writer.start_file("gone.xml", stored).unwrap();
    writer.write_all(b"<gone/>").unwrap();
    let zip = writer.finish().unwrap().into_inner();

    let mut edits = crate::PartEdits::default();
    edits.replace("EDIT.xml", b"<new/>".to_vec());
    edits.replace("added.xml", b"<added/>".to_vec());
    edits.remove("GONE.xml");

    let mut pkg = Package::open(zip.clone(), PackageLimits::default()).unwrap();
    let out = pkg.rewrite(&edits).unwrap();
    assert_eq!(raw_stream(&zip, "keep.xml"), raw_stream(&out, "keep.xml"));
    assert_eq!(raw_stream(&out, "keep.xml"), b"<keep>same</keep>");

    let mut re = Package::open(out.clone(), PackageLimits::default()).unwrap();
    assert_eq!(re.read_part("edit.xml").unwrap(), b"<new/>");
    assert_eq!(re.read_part("added.xml").unwrap(), b"<added/>");
    assert!(!re.contains("gone.xml"));
    assert_eq!(re.entry_names(), ["keep.xml", "edit.xml", "added.xml"]);

    // The same edits give the same bytes.
    let mut again = Package::open(zip, PackageLimits::default()).unwrap();
    assert_eq!(again.rewrite(&edits).unwrap(), out);
}

#[test]
fn a_rewrite_refuses_an_unsafe_new_part_name() {
    let mut edits = crate::PartEdits::default();
    edits.replace("../evil.xml", b"x".to_vec());
    let zip = make_zip(&[("a.xml", b"<a/>")]);
    let mut pkg = Package::open(zip, PackageLimits::default()).unwrap();
    assert!(pkg.rewrite(&edits).is_err());
}

#[test]
fn find_part_ignores_case() {
    let zip = make_zip(&[("ppt/Slides/Slide1.xml", b"<a/>")]);
    let pkg = Package::open(zip, PackageLimits::default()).unwrap();
    assert_eq!(
        pkg.find_part("PPT/slides/slide1.XML"),
        Some("ppt/Slides/Slide1.xml")
    );
    assert_eq!(pkg.find_part("ppt/missing.xml"), None);
}

/// The overall ratio alone is diluted by ordinary parts. SEC-P12.
#[test]
fn a_bomb_diluted_by_ordinary_parts_is_refused() {
    // Incompressible filler, so the package as a whole stays under the ratio.
    let filler: Vec<u8> = (0..200_000u32)
        .flat_map(|i| i.wrapping_mul(2_654_435_761).to_le_bytes())
        .collect();
    let bytes = make_zip(&[("zeros.bin", &vec![0u8; 64 << 20]), ("filler.bin", &filler)]);
    let err = Package::open(bytes, PackageLimits::default()).unwrap_err();
    assert!(
        matches!(err, PackageError::ExpansionRatioExceeded { .. }),
        "{err}"
    );
}

/// Refused from the declaration, with nothing decompressed. SEC-P13.
#[test]
fn a_capped_read_refuses_before_decompressing() {
    let zip = make_zip(&[("big.xml", &[b'x'; 5000]), ("small.xml", b"<a/>")]);
    let mut pkg = Package::open(zip, PackageLimits::default()).unwrap();
    let err = pkg.read_part_capped("big.xml", 1000).unwrap_err();
    assert!(
        matches!(err, PackageError::PartTooLarge { size: 5000, .. }),
        "{err}"
    );
    assert_eq!(pkg.consumed(), 0);
    assert_eq!(pkg.read_part_capped("small.xml", 1000).unwrap(), b"<a/>");
    assert_eq!(pkg.consumed(), 4);
}

// ---- PKG-01, PKG-02, PKG-03, SEC-017 -------------------------------------------

fn stored_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    for (name, data) in entries {
        writer.start_file(*name, opts).unwrap();
        writer.write_all(data).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

/// `SEC-017`, `SEC-P04`: the allocation `read_part` makes before any byte arrives
/// is the bounded one, whatever the part declares. Observed on the real path.
#[test]
fn read_part_allocates_up_front_no_more_than_the_bound() {
    let big: Vec<u8> = (0..3u32 << 20)
        .map(|i| (i.wrapping_mul(2_654_435_761) >> 24) as u8)
        .collect();
    let mut pkg =
        Package::open(stored_zip(&[("big.bin", &big)]), PackageLimits::default()).unwrap();
    let part = pkg.read_part("big.bin").unwrap();
    assert_eq!(part.len(), big.len());
    let up_front = crate::package::LAST_UP_FRONT.with(std::cell::Cell::get);
    assert!(
        up_front <= 1 << 20,
        "allocated {up_front} bytes before reading a part that declared {}",
        big.len()
    );
}

/// `PKG-01`: a long session reads parts again and again. Each part's bytes are
/// charged once, so a legitimate deck is never refused for being reread, while
/// what is charged still adds up across parts.
#[test]
fn rereading_a_part_does_not_spend_the_budget_again() {
    let chunk = vec![b'x'; 4096];
    let bytes = make_zip(&[
        ("[Content_Types].xml", b"<Types/>"),
        ("a.bin", &chunk),
        ("b.bin", &chunk),
        ("c.bin", &chunk),
    ]);
    let limits = PackageLimits {
        max_total_uncompressed: 12_288 + 64,
        max_expansion_ratio: u64::MAX,
        ..PackageLimits::default()
    };
    let mut pkg = Package::open(bytes, limits).expect("admitted");
    for name in ["a.bin", "b.bin", "c.bin"] {
        assert_eq!(pkg.read_part(name).unwrap().len(), 4096);
    }
    assert_eq!(pkg.consumed(), 12_288);
    for _ in 0..1000 {
        for name in ["a.bin", "b.bin", "c.bin", "[Content_Types].xml"] {
            pkg.read_part(name)
                .unwrap_or_else(|e| panic!("{name}: {e}"));
        }
    }
    assert_eq!(pkg.consumed(), 12_288 + 8);
}

/// `PKG-02`: package names compare as `part_name_key` does: percent-decoded once,
/// then ASCII case-insensitively. `É` and `é` are different parts.
#[test]
fn letters_outside_ascii_are_not_folded() {
    let zip = make_zip(&[
        ("ppt/media/\u{c9}.png", b"upper"),
        ("ppt/media/\u{e9}.png", b"lower"),
    ]);
    let mut pkg =
        Package::open(zip, PackageLimits::default()).expect("two parts, by ECMA-376 Part 2");
    assert_eq!(
        pkg.find_part("PPT/MEDIA/\u{c9}.PNG"),
        Some("ppt/media/\u{c9}.png")
    );
    assert_eq!(
        pkg.find_part("ppt/media/\u{e9}.png"),
        Some("ppt/media/\u{e9}.png")
    );
    let mut edits = crate::PartEdits::default();
    edits.remove("ppt/media/\u{c9}.png");
    let out = pkg.rewrite(&edits).unwrap();
    let re = Package::open(out, PackageLimits::default()).unwrap();
    assert_eq!(re.entry_names(), ["ppt/media/\u{e9}.png"]);
}

#[test]
fn percent_encoded_spellings_are_one_part() {
    let err = Package::open(
        make_zip(&[("ppt/media/a b.png", b"x"), ("ppt/media/a%20b.png", b"y")]),
        PackageLimits::default(),
    )
    .unwrap_err();
    assert_eq!(err.code(), "CS-PKG-0007");

    let zip = make_zip(&[("ppt/my slide.xml", b"<old/>")]);
    let mut pkg = Package::open(zip, PackageLimits::default()).unwrap();
    assert_eq!(
        pkg.find_part("PPT/my%20slide.xml"),
        Some("ppt/my slide.xml")
    );
    let mut edits = crate::PartEdits::default();
    edits.replace("ppt/my%20slide.xml", b"<new/>".to_vec());
    let out = pkg.rewrite(&edits).unwrap();
    let mut re = Package::open(out, PackageLimits::default()).expect("one part, not two");
    assert_eq!(re.entry_names(), ["ppt/my slide.xml"]);
    assert_eq!(re.read_part("ppt/my slide.xml").unwrap(), b"<new/>");
}

/// `PKG-03`: what a rewrite writes carries the fixed time, set explicitly, so a
/// `zip` feature turned on by some other crate cannot put the clock in the bytes.
#[test]
fn a_rewrite_writes_a_fixed_timestamp() {
    let zip = make_zip(&[("keep.xml", b"<a/>"), ("edit.xml", b"<b/>")]);
    let mut pkg = Package::open(zip, PackageLimits::default()).unwrap();
    let mut edits = crate::PartEdits::default();
    edits.replace("edit.xml", b"<c/>".to_vec());
    edits.replace("new.xml", b"<d/>".to_vec());
    let out = pkg.rewrite(&edits).unwrap();
    let mut archive = zip::ZipArchive::new(Cursor::new(out)).unwrap();
    for name in ["edit.xml", "new.xml"] {
        let entry = archive.by_name(name).unwrap();
        let t = entry.last_modified().unwrap();
        assert_eq!(
            (
                t.year(),
                t.month(),
                t.day(),
                t.hour(),
                t.minute(),
                t.second()
            ),
            (1980, 1, 1, 0, 0, 0),
            "{name}"
        );
    }
}

/// `PKG-03`: a rewritten entry records one host system on every build target. `zip`
/// otherwise records DOS on Windows and Unix elsewhere, and the bytes differ.
#[test]
fn a_rewrite_records_the_same_host_system_on_every_target() {
    let zip = make_zip(&[("keep.xml", b"<a/>"), ("edit.xml", b"<b/>")]);
    let mut pkg = Package::open(zip, PackageLimits::default()).unwrap();
    let mut edits = crate::PartEdits::default();
    edits.replace("edit.xml", b"<c/>".to_vec());
    edits.replace("new.xml", b"<d/>".to_vec());
    let out = pkg.rewrite(&edits).unwrap();
    let mut archive = zip::ZipArchive::new(Cursor::new(out)).unwrap();
    for name in ["edit.xml", "new.xml"] {
        let entry = archive.by_name(name).unwrap();
        assert_eq!(entry.system(), zip::System::Unix, "{name}");
    }
}

/// Set the sizes in the first local header only, leaving the central directory as written.
fn lie_in_local_header(
    mut zip: Vec<u8>,
    compressed: Option<u32>,
    uncompressed: Option<u32>,
) -> Vec<u8> {
    let at = zip.windows(4).position(|w| w == b"PK\x03\x04").unwrap();
    if let Some(c) = compressed {
        zip[at + 18..at + 22].copy_from_slice(&c.to_le_bytes());
    }
    if let Some(u) = uncompressed {
        zip[at + 22..at + 26].copy_from_slice(&u.to_le_bytes());
    }
    zip
}

/// A local header whose sizes disagree with the central directory: a reader trusting one
/// stops at a different byte from a reader trusting the other. SEC-P11.
#[test]
fn a_local_header_whose_sizes_disagree_with_the_directory_is_refused() {
    let zip = make_zip(&[("[Content_Types].xml", b"<Types/>"), ("a.xml", b"<a/>")]);
    assert!(!is_refused(zip.clone()), "the honest package opens");
    let err = Package::open(
        lie_in_local_header(zip.clone(), Some(1), None),
        PackageLimits::default(),
    )
    .unwrap_err();
    assert_eq!(err.code(), "CS-PKG-0005");
    assert!(is_refused(lie_in_local_header(zip, None, Some(7))));
}

/// The host's time budget stops a slow decompression inside every size cap, and a budget
/// that never runs out changes nothing. The predicate stands in for a clock. SEC-P10.
#[test]
fn the_hosts_time_budget_stops_a_part_read() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let payload: Vec<u8> = (0..1u32 << 20).map(|i| (i % 251) as u8).collect();
    let zip = make_zip(&[("[Content_Types].xml", b"<Types/>"), ("big.bin", &payload)]);

    let mut pkg = Package::open(zip.clone(), PackageLimits::default()).unwrap();
    pkg.set_cancel(Some(Arc::new(|| false)));
    assert_eq!(pkg.read_part("big.bin").unwrap(), payload);

    let asked = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&asked);
    let mut pkg = Package::open(zip, PackageLimits::default()).unwrap();
    pkg.set_cancel(Some(Arc::new(move || {
        counter.fetch_add(1, Ordering::Relaxed) >= 2
    })));
    let err = pkg.read_part("big.bin").map(|b| b.len()).unwrap_err();
    assert!(matches!(err, PackageError::Cancelled { .. }), "got {err:?}");
    assert_eq!(err.code(), "CS-PKG-0010");
    assert!(asked.load(Ordering::Relaxed) >= 3);
}
