//! The recovering reader, on archives written to order. Hostile inputs never panic,
//! and the image is always what the strict reader admits.

use std::io::{Cursor, Write};

use zip::CompressionMethod;
use zip::write::{SimpleFileOptions, ZipWriter};

use super::*;
use crate::Package;

fn zip_of(entries: &[(&str, &[u8])], method: CompressionMethod) -> Vec<u8> {
    let mut w = ZipWriter::new(Cursor::new(Vec::new()));
    let opts = SimpleFileOptions::default().compression_method(method);
    for (name, data) in entries {
        w.start_file(*name, opts).unwrap();
        w.write_all(data).unwrap();
    }
    w.finish().unwrap().into_inner()
}

fn deflated(entries: &[(&str, &[u8])]) -> Vec<u8> {
    zip_of(entries, CompressionMethod::Deflated)
}

fn stored(entries: &[(&str, &[u8])]) -> Vec<u8> {
    zip_of(entries, CompressionMethod::Stored)
}

fn salvaged(bytes: &[u8]) -> Salvaged {
    salvage(bytes, PackageLimits::default(), |_, _| true).unwrap()
}

/// The image the strict reader admits.
fn image_of(s: &Salvaged) -> Package {
    Package::open(s.image.clone(), PackageLimits::default()).unwrap()
}

fn codes(s: &Salvaged) -> Vec<&'static str> {
    let mut codes: Vec<_> = s.faults.iter().map(|f| f.kind.code()).collect();
    codes.sort_unstable();
    codes.dedup();
    codes
}

const BODY: &[u8] = b"<a>some ordinary text, long enough to want compressing compressing</a>";

#[test]
fn a_deck_with_no_directory_is_rebuilt_from_local_headers() {
    let whole = deflated(&[("a.xml", BODY), ("b/c.xml", BODY)]);
    let directory = find_sig(&whole, 0, CENTRAL_SIG).unwrap();
    let s = salvaged(&whole[..directory]);
    assert_eq!(codes(&s), ["CS-REC-0101"]);
    let mut image = image_of(&s);
    assert_eq!(image.read_part("a.xml").unwrap(), BODY);
    assert_eq!(image.read_part("b/c.xml").unwrap(), BODY);
}

#[test]
fn prepended_bytes_are_stepped_over() {
    let mut junk = b"#!/bin/sh\nexit 0\n".to_vec();
    junk.extend(deflated(&[("a.xml", BODY)]));
    let s = salvaged(&junk);
    assert_eq!(codes(&s), ["CS-REC-0102"]);
    assert_eq!(image_of(&s).read_part("a.xml").unwrap(), BODY);
}

#[test]
fn trailing_bytes_are_ignored() {
    let mut whole = deflated(&[("a.xml", BODY)]);
    whole.extend_from_slice(&[7; 100]);
    let s = salvaged(&whole);
    assert!(codes(&s).contains(&"CS-REC-0102"), "{:?}", s.faults);
    assert_eq!(image_of(&s).read_part("a.xml").unwrap(), BODY);
}

#[test]
fn a_truncated_entry_keeps_what_could_be_read() {
    let long: Vec<u8> = (0..200_000u32).flat_map(|i| i.to_le_bytes()).collect();
    let whole = deflated(&[("a.xml", BODY), ("big.bin", &long)]);
    let s = salvaged(&whole[..whole.len() / 2]);
    assert!(codes(&s).contains(&"CS-REC-0103"), "{:?}", s.faults);
    assert!(s.suspect.contains("big.bin"));
    let mut image = image_of(&s);
    assert_eq!(image.read_part("a.xml").unwrap(), BODY);
    let partial = image.read_part("big.bin").unwrap();
    assert!(!partial.is_empty() && long.starts_with(&partial));
}

#[test]
fn a_wrong_checksum_keeps_the_bytes_and_says_so() {
    let mut whole = stored(&[("a.xml", BODY)]);
    // The local header's and the directory's checksum.
    whole[14] ^= 0xFF;
    let central = find_sig(&whole, 0, CENTRAL_SIG).unwrap();
    whole[central + 16] ^= 0xFF;
    let s = salvaged(&whole);
    assert_eq!(codes(&s), ["CS-REC-0104"]);
    assert!(s.suspect.contains("a.xml"));
    assert_eq!(image_of(&s).read_part("a.xml").unwrap(), BODY);
}

/// A ZIP holding two entries of one name, which `zip`'s writer will not make.
fn twin(first: &[u8], second: &[u8]) -> Vec<u8> {
    let mut w = ImageWriter::default();
    for data in [first, second] {
        let mut crc = Crc::new();
        crc.update(data);
        assert!(w.add(ImageEntry {
            name: "a.xml",
            method: STORED,
            crc: crc.sum(),
            data,
            uncompressed: data.len() as u64,
        }));
    }
    w.finish()
}

#[test]
fn of_two_entries_the_last_that_parses_is_read_and_the_other_kept() {
    let bytes = twin(b"<good/>", b"<broken");
    let s = salvage(&bytes, PackageLimits::default(), |_, data| {
        data.ends_with(b"/>")
    })
    .unwrap();
    assert_eq!(codes(&s), ["CS-REC-0105"]);
    let mut image = image_of(&s);
    assert_eq!(image.read_part("a.xml").unwrap(), b"<good/>");
    assert_eq!(s.retained.len(), 1);
    assert_eq!(
        image.read_part(&s.retained[0].stored_as).unwrap(),
        b"<broken"
    );
    // Both parse: the last wins.
    let s = salvage(
        &twin(b"<one/>", b"<two/>"),
        PackageLimits::default(),
        |_, _| true,
    )
    .unwrap();
    assert_eq!(image_of(&s).read_part("a.xml").unwrap(), b"<two/>");
}

#[test]
fn a_case_colliding_entry_is_kept_and_only_one_is_read() {
    let bytes = stored(&[("ppt/a.xml", b"<one/>"), ("PPT/A.XML", b"<two/>")]);
    let s = salvaged(&bytes);
    assert_eq!(codes(&s), ["CS-REC-0105"]);
    let image = image_of(&s);
    assert_eq!(image.len(), 2);
    assert!(image.find_part("ppt/a.xml").is_some());
    assert!(
        image
            .entry_names()
            .iter()
            .any(|n| n.starts_with(RETAINED_DIR))
    );
}

#[test]
fn an_unsafe_name_never_becomes_a_part() {
    for name in [
        "../../etc/passwd",
        "/etc/passwd",
        "C:\\win.ini",
        "a\u{1}b.xml",
    ] {
        let mut w = ImageWriter::default();
        assert!(w.add(ImageEntry {
            name,
            method: STORED,
            crc: 0,
            data: b"x",
            uncompressed: 1,
        }));
        let bytes = w.finish();
        let s = salvaged(&bytes);
        assert!(codes(&s).contains(&"CS-REC-0106"), "{name}");
        let image = image_of(&s);
        assert_eq!(image.len(), 1, "{name}");
        let only = &image.entry_names()[0];
        assert!(only.starts_with(RETAINED_DIR), "{only}");
        assert!(!only[RETAINED_DIR.len()..].contains('/'), "{only}");
        assert!(is_safe_part_path(only), "{only}");
    }
}

#[test]
fn a_bomb_is_retained_by_raw_copy_and_never_inflated() {
    let bomb = vec![0u8; 8 << 20];
    let bytes = deflated(&[("a.xml", BODY), ("bomb.bin", &bomb)]);
    let s = salvaged(&bytes);
    assert_eq!(codes(&s), ["CS-REC-0107"]);
    let mut image = image_of(&s);
    assert!(image.find_part("bomb.bin").is_none());
    assert_eq!(image.read_part("a.xml").unwrap(), BODY);
    let kept = s.retained[0].stored_as.clone();
    assert_eq!(s.retained[0].original, "bomb.bin");
    let raw = image.read_part(&kept).unwrap();
    // It is the compressed stream, and it stays that: nothing grew.
    assert!(raw.len() < bomb.len() / 100, "{}", raw.len());
    assert!(bytes.windows(raw.len()).any(|w| w == raw));
}

/// `SEC-P02` for this reader: a part that lies about its size is stopped at what its input
/// could honestly produce, and the file's number sizes nothing.
#[test]
fn a_part_that_understates_its_size_is_stopped_by_the_ratio() {
    let bomb = vec![0u8; 8 << 20];
    let mut bytes = deflated(&[("bomb.bin", &bomb)]);
    // Declares one byte, in both headers.
    bytes[22..26].copy_from_slice(&1u32.to_le_bytes());
    let central = find_sig(&bytes, 0, CENTRAL_SIG).unwrap();
    bytes[central + 24..central + 28].copy_from_slice(&1u32.to_le_bytes());
    let limits = PackageLimits {
        max_expansion_ratio: 10,
        ..PackageLimits::default()
    };
    let s = salvage(&bytes, limits, |_, _| true).unwrap();
    assert_eq!(codes(&s), ["CS-REC-0107"]);
}

#[test]
fn a_declared_size_allocates_nothing() {
    // A header claiming 4 GiB of output over a few bytes, with no directory: the reader
    // would abort a WebAssembly instance if it believed the claim.
    let mut bytes = stored(&[("a.bin", b"xyz")]);
    bytes[22..26].copy_from_slice(&(u32::MAX - 2).to_le_bytes());
    let cut = find_sig(&bytes, 0, CENTRAL_SIG).unwrap();
    bytes.truncate(cut);
    let s = salvaged(&bytes);
    assert_eq!(image_of(&s).read_part("a.bin").unwrap(), b"xyz");
}

/// A ZIP64 record can claim anything in 64 bits. Sized from that, this would abort.
#[test]
fn a_claim_of_exabytes_allocates_nothing() {
    let name = b"a.bin";
    let data = [0x03, 0x00]; // an empty deflate stream
    let mut bytes = LOCAL_SIG.to_vec();
    bytes.extend_from_slice(&[20, 0, 0, 0, 8, 0, 0, 0, 0x21, 0]);
    bytes.extend_from_slice(&0u32.to_le_bytes()); // crc
    bytes.extend_from_slice(&2u32.to_le_bytes()); // compressed
    bytes.extend_from_slice(&u32::MAX.to_le_bytes()); // uncompressed: see the extra
    bytes.extend_from_slice(&(name.len() as u16).to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.extend_from_slice(name);
    bytes.extend_from_slice(&data);
    let central = bytes.len();
    bytes.extend_from_slice(&CENTRAL_SIG);
    bytes.extend_from_slice(&[20, 0, 20, 0, 0, 0, 8, 0, 0, 0, 0x21, 0]);
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&2u32.to_le_bytes());
    bytes.extend_from_slice(&u32::MAX.to_le_bytes());
    bytes.extend_from_slice(&(name.len() as u16).to_le_bytes());
    bytes.extend_from_slice(&12u16.to_le_bytes()); // extra
    bytes.extend_from_slice(&[0; 10]); // comment, disk, attributes
    bytes.extend_from_slice(&0u32.to_le_bytes()); // offset
    bytes.extend_from_slice(name);
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&8u16.to_le_bytes());
    bytes.extend_from_slice(&(1u64 << 62).to_le_bytes());
    let size = bytes.len() - central;
    bytes.extend_from_slice(&EOCD_SIG);
    bytes.extend_from_slice(&[0; 4]);
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&(size as u32).to_le_bytes());
    bytes.extend_from_slice(&(central as u32).to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    let s = salvaged(&bytes);
    assert_eq!(codes(&s), ["CS-REC-0107"]);
    assert_eq!(s.retained.len(), 1);
    assert_eq!(s.retained[0].original, "a.bin");
    image_of(&s);
}

#[test]
fn overlapping_entries_share_one_budget() {
    // Fifield's construction: many directory records naming one stream.
    let stream = vec![0u8; 1 << 20];
    let mut w = ZipWriter::new(Cursor::new(Vec::new()));
    let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    w.start_file("ss.bin", opts).unwrap();
    w.write_all(&stream).unwrap();
    let base = w.finish().unwrap().into_inner();
    let central = find_sig(&base, 0, CENTRAL_SIG).unwrap();
    let eocd = find_sig(&base, 0, EOCD_SIG).unwrap();
    let record = base[central..eocd].to_vec();
    let mut bytes = base[..central].to_vec();
    let count = 40u16;
    for i in 0..count {
        let mut r = record.clone();
        // A distinct name of the same length.
        let n = r.len() - 1;
        r[n - 4] = b'a' + (i % 26) as u8;
        r[n - 5] = b'a' + (i / 26) as u8;
        bytes.extend_from_slice(&r);
    }
    let size = record.len() * usize::from(count);
    bytes.extend_from_slice(&EOCD_SIG);
    bytes.extend_from_slice(&[0; 4]);
    bytes.extend_from_slice(&count.to_le_bytes());
    bytes.extend_from_slice(&count.to_le_bytes());
    bytes.extend_from_slice(&(size as u32).to_le_bytes());
    bytes.extend_from_slice(&(central as u32).to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    let limits = PackageLimits {
        max_total_uncompressed: 5 << 20,
        ..PackageLimits::default()
    };
    let s = salvage(&bytes, limits, |_, _| true).unwrap();
    assert!(codes(&s).contains(&"CS-REC-0107"), "{:?}", s.faults);
    // Five of them, at most, were read as parts; the rest stayed opaque.
    assert!(s.retained.len() >= 34, "{}", s.retained.len());
}

#[test]
fn encrypted_and_unknown_methods_are_kept_unopened() {
    let mut bytes = stored(&[("enc.xml", b"secret"), ("odd.xml", b"bytes")]);
    let second = find_sig(&bytes, 4, LOCAL_SIG).unwrap();
    let central = find_sig(&bytes, 0, CENTRAL_SIG).unwrap();
    let second_central = find_sig(&bytes, central + 4, CENTRAL_SIG).unwrap();
    bytes[6] |= 1;
    bytes[central + 8] |= 1;
    bytes[second + 8] = 93;
    bytes[second_central + 10] = 93;
    let s = salvaged(&bytes);
    assert_eq!(codes(&s), ["CS-REC-0108"]);
    assert_eq!(s.retained.len(), 2);
}

#[test]
fn the_entry_limit_stops_the_scan_and_says_so() {
    let many: Vec<(String, &[u8])> = (0..10).map(|i| (format!("{i}.xml"), BODY)).collect();
    let refs: Vec<(&str, &[u8])> = many.iter().map(|(n, b)| (n.as_str(), *b)).collect();
    let whole = stored(&refs);
    let cut = find_sig(&whole, 0, CENTRAL_SIG).unwrap();
    let limits = PackageLimits {
        max_entries: 3,
        ..PackageLimits::default()
    };
    let s = salvage(&whole[..cut], limits, |_, _| true).unwrap();
    assert!(codes(&s).contains(&"CS-REC-0109"), "{:?}", s.faults);
    assert_eq!(image_of(&s).len(), 3);
}

#[test]
fn what_holds_nothing_is_still_refused() {
    for bytes in [
        &b""[..],
        b"this is not a package",
        &[0x50, 0x4b, 3, 4],
        &[0xAB; 4096],
    ] {
        let err = salvage(bytes, PackageLimits::default(), |_, _| true).unwrap_err();
        assert_eq!(err.code(), "CS-PKG-0005");
    }
}

#[test]
fn the_whole_file_limit_still_refuses() {
    let limits = PackageLimits {
        max_input_bytes: 3,
        ..PackageLimits::default()
    };
    let err = salvage(b"abcd", limits, |_, _| true).unwrap_err();
    assert_eq!(err.code(), "CS-PKG-0001");
}

/// A stored entry's bytes can hold a whole ZIP, which is content and not more entries.
#[test]
fn a_zip_inside_a_stored_entry_is_content() {
    let inner = stored(&[("inner.xml", BODY)]);
    let whole = stored(&[("embed.bin", &inner), ("a.xml", BODY)]);
    let cut = find_sig(&whole, inner.len() + 40, CENTRAL_SIG).unwrap();
    let s = salvaged(&whole[..cut]);
    let names = image_of(&s).entry_names();
    assert_eq!(names, ["embed.bin", "a.xml"], "{:?}", s.faults);
}

#[test]
fn salvaging_is_deterministic() {
    let whole = deflated(&[("a.xml", BODY), ("b.xml", BODY)]);
    let cut = whole.len() - 30;
    assert_eq!(salvaged(&whole[..cut]), salvaged(&whole[..cut]));
}

#[test]
fn no_damage_at_any_offset_panics_or_breaks_the_image() {
    let whole = deflated(&[
        ("[Content_Types].xml", BODY),
        ("a/b.xml", BODY),
        ("c.bin", &[1, 2, 3, 4, 5, 6, 7, 8]),
    ]);
    let limits = PackageLimits::default();
    for at in 0..whole.len() {
        for byte in [0x00u8, 0xFF, whole[at] ^ 0x10] {
            let mut bad = whole.clone();
            bad[at] = byte;
            if let Ok(s) = salvage(&bad, limits, |_, _| true) {
                Package::open(s.image, limits).unwrap();
            }
        }
        if let Ok(s) = salvage(&whole[..at], limits, |_, _| true) {
            Package::open(s.image, limits).unwrap();
        }
    }
}
