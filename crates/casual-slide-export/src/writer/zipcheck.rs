//! What is written, read back at ZIP level, by hand and with bounds: the rules no part's
//! content can show (`INV-PN-08` to `INV-PN-11`), and F2 itself: every entry not named as
//! changed keeps its compressed bytes.

use std::collections::{BTreeMap, BTreeSet};

use super::draft::CONTENT_TYPES_PART;
use super::error::WriterError;
use super::names::key;

#[derive(Debug, Clone)]
struct Record {
    name: String,
    flags: u16,
    method: u16,
    crc: u32,
    compressed: u64,
    uncompressed: u64,
    offset: u64,
    /// Local header and its data agree with this record.
    data: Option<std::ops::Range<usize>>,
    headers_agree: bool,
}

fn u16_at(bytes: &[u8], at: usize) -> Option<u16> {
    let b = bytes.get(at..at.checked_add(2)?)?;
    Some(u16::from_le_bytes([*b.first()?, *b.get(1)?]))
}

fn u32_at(bytes: &[u8], at: usize) -> Option<u32> {
    let b = bytes.get(at..at.checked_add(4)?)?;
    Some(u32::from_le_bytes([
        *b.first()?,
        *b.get(1)?,
        *b.get(2)?,
        *b.get(3)?,
    ]))
}

/// The end of central directory record: entries, directory offset. `None` for an
/// archive this reader will not vouch for, which is reported as such.
fn end_record(bytes: &[u8]) -> Option<(usize, usize)> {
    let floor = bytes.len().saturating_sub(22usize.saturating_add(0xFFFF));
    let mut at = bytes.len().checked_sub(22)?;
    loop {
        if u32_at(bytes, at)? == 0x0605_4B50 {
            let count = usize::from(u16_at(bytes, at.checked_add(10)?)?);
            let offset = usize::try_from(u32_at(bytes, at.checked_add(16)?)?).ok()?;
            return Some((count, offset));
        }
        if at == floor {
            return None;
        }
        at = at.checked_sub(1)?;
    }
}

fn read_local(bytes: &[u8], record: &mut Record) -> Option<()> {
    let at = usize::try_from(record.offset).ok()?;
    if u32_at(bytes, at)? != 0x0403_4B50 {
        return None;
    }
    let name_len = usize::from(u16_at(bytes, at.checked_add(26)?)?);
    let extra_len = usize::from(u16_at(bytes, at.checked_add(28)?)?);
    let name_at = at.checked_add(30)?;
    let name = bytes.get(name_at..name_at.checked_add(name_len)?)?;
    let flags = u16_at(bytes, at.checked_add(6)?)?;
    let method = u16_at(bytes, at.checked_add(8)?)?;
    let crc = u32_at(bytes, at.checked_add(14)?)?;
    let compressed = u32_at(bytes, at.checked_add(18)?)?;
    let uncompressed = u32_at(bytes, at.checked_add(22)?)?;
    let start = name_at.checked_add(name_len)?.checked_add(extra_len)?;
    let end = start.checked_add(usize::try_from(record.compressed).ok()?)?;
    bytes.get(start..end)?;
    record.data = Some(start..end);
    // With a data descriptor the local fields are zero and only the central ones count.
    let sizes_agree = flags & 0x8 != 0
        || record.compressed == u64::from(u32::MAX)
        || (crc == record.crc
            && u64::from(compressed) == record.compressed
            && u64::from(uncompressed) == record.uncompressed);
    record.headers_agree = name == record.name.as_bytes() && method == record.method && sizes_agree;
    Some(())
}

fn read_directory(bytes: &[u8]) -> Option<Vec<Record>> {
    let (count, mut at) = end_record(bytes)?;
    // `count` is a 16-bit field here, so this cannot size anything large.
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        if u32_at(bytes, at)? != 0x0201_4B50 {
            return None;
        }
        let flags = u16_at(bytes, at.checked_add(8)?)?;
        let method = u16_at(bytes, at.checked_add(10)?)?;
        let crc = u32_at(bytes, at.checked_add(16)?)?;
        let compressed = u32_at(bytes, at.checked_add(20)?)?;
        let uncompressed = u32_at(bytes, at.checked_add(24)?)?;
        let name_len = usize::from(u16_at(bytes, at.checked_add(28)?)?);
        let extra_len = usize::from(u16_at(bytes, at.checked_add(30)?)?);
        let comment_len = usize::from(u16_at(bytes, at.checked_add(32)?)?);
        let offset = u32_at(bytes, at.checked_add(42)?)?;
        let name_at = at.checked_add(46)?;
        let name = bytes.get(name_at..name_at.checked_add(name_len)?)?;
        let mut record = Record {
            name: std::str::from_utf8(name).ok()?.to_owned(),
            flags,
            method,
            crc,
            compressed: u64::from(compressed),
            uncompressed: u64::from(uncompressed),
            offset: u64::from(offset),
            data: None,
            headers_agree: false,
        };
        if !record.name.ends_with('/') {
            read_local(bytes, &mut record)?;
        } else {
            record.headers_agree = true;
        }
        out.push(record);
        at = name_at
            .checked_add(name_len)?
            .checked_add(extra_len)?
            .checked_add(comment_len)?;
    }
    Some(out)
}

/// Judge `output` against `input`: `changed` names the entries meant to differ, and
/// every other one must carry the same compressed bytes.
///
/// Seen as a fault here, in a file just written, a broken rule refuses the save.
pub fn check_zip(
    input: &[u8],
    output: &[u8],
    changed: &BTreeSet<String>,
) -> Result<(), WriterError> {
    let fail = |rule: &'static str, part: &str| WriterError::Emission {
        rule,
        part: part.to_owned(),
    };
    let Some(out) = read_directory(output) else {
        return Err(fail("INV-PN-10", "central directory"));
    };
    // An input only ZIP64 can describe has no records to compare; its parts are still
    // read back by name in `emit`.
    let Some(old) = read_directory(input) else {
        return Ok(());
    };
    let old_by_name: BTreeMap<&str, &Record> = old.iter().map(|r| (r.name.as_str(), r)).collect();
    let changed_keys: BTreeSet<String> = changed.iter().map(|c| key(c)).collect();
    let mut seen = BTreeSet::new();
    let mut literal = BTreeSet::new();
    for record in &out {
        // INV-PN-08 (Documented): unique, and ASCII unless it arrived that way.
        if !seen.insert(key(&record.name)) {
            // INV-PN-03 (Documented) when the two differ, in case, say.
            let rule = if literal.contains(record.name.as_str()) {
                "INV-PN-08"
            } else {
                "INV-PN-03"
            };
            return Err(fail(rule, &record.name));
        }
        literal.insert(record.name.as_str());
        let previous = old_by_name.get(record.name.as_str());
        if !record.name.is_ascii() && previous.is_none() {
            return Err(fail("INV-PN-08", &record.name));
        }
        // INV-PN-10 (Documented).
        if !record.headers_agree {
            return Err(fail("INV-PN-10", &record.name));
        }
        if changed_keys.contains(&key(&record.name)) || previous.is_none() {
            // INV-PN-09 (Documented): new entries deflate, with no encryption. A
            // directory entry has no content to compress.
            if !record.name.ends_with('/') && (record.method != 8 || record.flags & 1 != 0) {
                return Err(fail("INV-PN-09", &record.name));
            }
            continue;
        }
        let Some(previous) = previous else { continue };
        // F2, and INV-PN-09 for what is retained.
        let same = record.method == previous.method
            && record.crc == previous.crc
            && record.compressed == previous.compressed
            && record.uncompressed == previous.uncompressed
            && match (&record.data, &previous.data) {
                (Some(a), Some(b)) => output.get(a.clone()) == input.get(b.clone()),
                (None, None) => true,
                _ => false,
            };
        if !same {
            return Err(fail("F2", &record.name));
        }
    }
    // Every entry not removed is still there.
    let written: BTreeSet<String> = out.iter().map(|r| key(&r.name)).collect();
    for record in &old {
        let k = key(&record.name);
        if !written.contains(&k) && !changed_keys.contains(&k) {
            return Err(fail("F2", &record.name));
        }
    }
    // INV-PN-11 (Inferred): `[Content_Types].xml` keeps its place among what was there,
    // and new entries follow.
    let old_keys: BTreeSet<String> = old.iter().map(|r| key(&r.name)).collect();
    let retained_out: Vec<String> = out
        .iter()
        .map(|r| key(&r.name))
        .filter(|k| old_keys.contains(k))
        .collect();
    let retained_old: Vec<String> = old
        .iter()
        .map(|r| key(&r.name))
        .filter(|k| written.contains(k))
        .collect();
    let types = key(CONTENT_TYPES_PART);
    let place = |list: &[String]| list.iter().position(|k| *k == types);
    if place(&retained_out) != place(&retained_old) {
        return Err(fail("INV-PN-11", CONTENT_TYPES_PART));
    }
    Ok(())
}
