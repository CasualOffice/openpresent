//! The recovering reader: read what can be read from a damaged archive (`REC-01`,
//! `docs/29-RECOVERY.md`).
//!
//! [`Package::open`](crate::Package::open) stays strict. When it refuses, [`salvage`]
//! rebuilds the entry list from the central directory where one survives and from local
//! headers where it does not, and writes a clean archive, **the image**, that the strict
//! reader then admits. Everything above reads the image; the bytes as they arrived are
//! kept by the caller for an unedited save.
//!
//! Nothing here obeys the file:
//!
//! - no allocation is sized from a number in the file: buffers grow as bytes arrive;
//! - a scan only ever moves forward, so forged headers cost linear time;
//! - inflating is charged to one budget, `max_total_uncompressed`, so overlapping entries
//!   cannot multiply it;
//! - names are never used as paths, and an unsafe, duplicate or colliding entry is kept as
//!   an opaque blob under a name minted here, which no relationship can reach and whose
//!   compressed bytes are never inflated;
//! - a message holds a scrubbed entry name and nothing read from inside an entry.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;

use flate2::write::DeflateEncoder;
use flate2::{Compression, Crc, Decompress, FlushDecompress, Status};

use crate::error::PackageError;
use crate::limits::PackageLimits;
use crate::part_name::part_entry_key;
use crate::path::is_safe_part_path;

const LOCAL_SIG: [u8; 4] = [0x50, 0x4b, 0x03, 0x04];
const CENTRAL_SIG: [u8; 4] = [0x50, 0x4b, 0x01, 0x02];
const EOCD_SIG: [u8; 4] = [0x50, 0x4b, 0x05, 0x06];
const EOCD64_SIG: [u8; 4] = [0x50, 0x4b, 0x06, 0x06];
const LOCATOR_SIG: [u8; 4] = [0x50, 0x4b, 0x06, 0x07];
const DESCRIPTOR_SIG: [u8; 4] = [0x50, 0x4b, 0x07, 0x08];
const LOCAL_LEN: usize = 30;
const CENTRAL_LEN: usize = 46;
const EOCD_LEN: usize = 22;
const STORED: u16 = 0;
const DEFLATED: u16 = 8;
const FLAG_ENCRYPTED: u16 = 0x1;
const FLAG_DESCRIPTOR: u16 = 0x8;
const FLAG_UTF8: u16 = 0x800;
const INFLATE_CHUNK: usize = 64 * 1024;
/// A part's expansion is judged against this much output before the ratio applies, so a
/// tiny part that compresses well is not called a bomb.
const RATIO_FLOOR: u64 = 1 << 20;
/// The folder retained entries are written under.
pub const RETAINED_DIR: &str = "recovered/";
/// The extension of a retained entry, so one `Default` declares them all.
pub const RETAINED_EXTENSION: &str = "retained";
/// The longest part of a name repeated in a retained entry's name or in a report.
const MAX_SCRUBBED_NAME: usize = 200;

/// What was wrong with the container, in the order it was noticed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ContainerFaultKind {
    /// No usable central directory: the entry list was rebuilt from local headers.
    DirectoryRebuilt,
    /// Bytes before or after the ZIP. The ZIP inside was read.
    BytesAroundArchive,
    /// An entry is cut short or its stream is broken. What could be read is kept.
    Truncated,
    /// An entry's bytes do not match its checksum, or its declared size. Kept.
    ChecksumMismatch,
    /// Another entry names the same part. One is read; this one is retained, unread.
    DuplicateEntry,
    /// The entry's name is unsafe or too long. It is retained, unread, and never a path.
    UnsafeName,
    /// The entry is over a ratio or size limit. Retained by raw copy, not decompressed.
    OverLimit,
    /// The entry is encrypted or uses a method this engine cannot read. Retained.
    Unreadable,
    /// More entries than `max_entries`. Reading stopped there.
    TooManyEntries,
}

impl ContainerFaultKind {
    /// The stable diagnostic code (`docs/20`).
    pub fn code(self) -> &'static str {
        match self {
            Self::DirectoryRebuilt => "CS-REC-0101",
            Self::BytesAroundArchive => "CS-REC-0102",
            Self::Truncated => "CS-REC-0103",
            Self::ChecksumMismatch => "CS-REC-0104",
            Self::DuplicateEntry => "CS-REC-0105",
            Self::UnsafeName => "CS-REC-0106",
            Self::OverLimit => "CS-REC-0107",
            Self::Unreadable => "CS-REC-0108",
            Self::TooManyEntries => "CS-REC-0109",
        }
    }

    /// What to tell the user, in plain language. Fixed text.
    pub fn plain(self) -> &'static str {
        match self {
            Self::DirectoryRebuilt => "the file's table of contents was unusable and was rebuilt",
            Self::BytesAroundArchive => "extra data around the file's contents was ignored",
            Self::Truncated => "this item was cut short, and only part of it could be read",
            Self::ChecksumMismatch => {
                "this item failed its integrity check, and may hold damaged content"
            }
            Self::DuplicateEntry => "this item appears more than once; one copy is used",
            Self::UnsafeName => "this item has an unsafe name; it was kept but is not used",
            Self::OverLimit => {
                "this item is too large to open safely; it was kept unopened and left out"
            }
            Self::Unreadable => "this item could not be read; it was kept unopened and left out",
            Self::TooManyEntries => "the file holds too many items; the rest were not read",
        }
    }
}

/// One thing wrong with the container.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ContainerFault {
    /// What is wrong.
    pub kind: ContainerFaultKind,
    /// The entry, scrubbed of anything but printable characters. Empty for the archive.
    pub entry: String,
}

/// An entry kept as an opaque blob.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetainedEntry {
    /// Its name in the image, minted here.
    pub stored_as: String,
    /// Its name in the input, scrubbed.
    pub original: String,
    /// Why it is not a part.
    pub why: ContainerFaultKind,
}

/// A damaged archive, read as far as it can be.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Salvaged {
    /// A clean archive holding what was read, in input order. The strict reader admits it.
    pub image: Vec<u8>,
    /// Everything noticed, in a stable order.
    pub faults: Vec<ContainerFault>,
    /// Entries set aside whole.
    pub retained: Vec<RetainedEntry>,
    /// Entries whose content arrived damaged or cut short: their bytes are what could be
    /// read, under their own names, and are not to be trusted as the writer wrote them.
    pub suspect: BTreeSet<String>,
}

/// A name fit to repeat: printable, and bounded. Never a path, and never all of a long one.
pub fn scrub_name(name: &str) -> String {
    name.chars()
        .filter(|c| !c.is_control())
        .take(MAX_SCRUBBED_NAME)
        .collect()
}

fn u16_at(b: &[u8], at: usize) -> Option<u16> {
    let s = b.get(at..at.checked_add(2)?)?;
    Some(u16::from_le_bytes(s.try_into().ok()?))
}

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    let s = b.get(at..at.checked_add(4)?)?;
    Some(u32::from_le_bytes(s.try_into().ok()?))
}

fn u64_at(b: &[u8], at: usize) -> Option<u64> {
    let s = b.get(at..at.checked_add(8)?)?;
    Some(u64::from_le_bytes(s.try_into().ok()?))
}

/// The next occurrence of `sig` at or after `from`.
fn find_sig(b: &[u8], from: usize, sig: [u8; 4]) -> Option<usize> {
    let rest = b.get(from..)?;
    rest.windows(4)
        .position(|w| w == sig)
        .and_then(|i| from.checked_add(i))
}

/// The last end-of-central-directory signature whose fixed part is wholly present.
fn find_eocd_tolerant(b: &[u8]) -> Option<usize> {
    let last = b.len().checked_sub(EOCD_LEN)?;
    (0..=last)
        .rev()
        .find(|&at| b.get(at..at.saturating_add(4)) == Some(&EOCD_SIG[..]))
}

/// One entry as the directory or a local header describes it.
#[derive(Debug, Clone)]
struct Candidate {
    /// The name's raw bytes. Sixteen bits of length, so bounded.
    name: Vec<u8>,
    flags: u16,
    method: u16,
    /// Where the local header begins.
    header: usize,
    /// Where the data begins.
    data: usize,
    /// How many compressed bytes the header claims, if it claims any.
    compressed: Option<u64>,
    /// How many bytes the header claims it expands to, if it claims any.
    uncompressed: Option<u64>,
    /// The checksum the header claims, if it claims one.
    crc: Option<u32>,
}

/// A local header at `at`, if it is plausible: lengths in bounds and a name present.
fn local_header(b: &[u8], at: usize) -> Option<Candidate> {
    if b.get(at..at.checked_add(4)?)? != LOCAL_SIG {
        return None;
    }
    let flags = u16_at(b, at.checked_add(6)?)?;
    let method = u16_at(b, at.checked_add(8)?)?;
    let crc = u32_at(b, at.checked_add(14)?)?;
    let compressed = u32_at(b, at.checked_add(18)?)?;
    let uncompressed = u32_at(b, at.checked_add(22)?)?;
    let name_len = usize::from(u16_at(b, at.checked_add(26)?)?);
    let extra_len = usize::from(u16_at(b, at.checked_add(28)?)?);
    if name_len == 0 {
        return None;
    }
    let name_at = at.checked_add(LOCAL_LEN)?;
    let name = b.get(name_at..name_at.checked_add(name_len)?)?.to_vec();
    let data = name_at.checked_add(name_len)?.checked_add(extra_len)?;
    if data > b.len() {
        return None;
    }
    // Sizes in a descriptor, or in a ZIP64 extra, are not here.
    let sized = flags & FLAG_DESCRIPTOR == 0 && compressed != u32::MAX && uncompressed != u32::MAX;
    Some(Candidate {
        name,
        flags,
        method,
        header: at,
        data,
        compressed: sized.then_some(u64::from(compressed)),
        uncompressed: sized.then_some(u64::from(uncompressed)),
        crc: (flags & FLAG_DESCRIPTOR == 0).then_some(crc),
    })
}

/// What a walk of the central directory found.
struct Directory {
    entries: Vec<Candidate>,
    /// Every record was read, and landed exactly on what the end record declares.
    complete: bool,
    /// Bytes precede the archive, or trail it.
    around: bool,
    /// Records past `max_entries` were not read.
    too_many: bool,
}

/// ZIP64 sizes and offset from a directory record's extra field, each where the 32-bit
/// field says to look.
fn zip64_extra(
    extra: &[u8],
    sizes: (u32, u32),
    offset: u32,
) -> (Option<u64>, Option<u64>, Option<u64>) {
    let mut rest = extra;
    while let (Some(id), Some(len)) = (u16_at(rest, 0), u16_at(rest, 2)) {
        let end = 4usize.saturating_add(usize::from(len));
        let body = rest.get(4..end).unwrap_or_default();
        if id == 1 {
            let mut at = 0usize;
            let mut next = |wanted: bool| {
                if !wanted {
                    return None;
                }
                let value = u64_at(body, at);
                at = at.saturating_add(8);
                value
            };
            let uncompressed = next(sizes.0 == u32::MAX);
            let compressed = next(sizes.1 == u32::MAX);
            let local = next(offset == u32::MAX);
            return (uncompressed, compressed, local);
        }
        rest = rest.get(end..).unwrap_or_default();
    }
    (None, None, None)
}

fn read_directory(b: &[u8], limits: &PackageLimits) -> Option<Directory> {
    let eocd = find_eocd_tolerant(b)?;
    let mut declared = u64::from(u16_at(b, eocd.checked_add(10)?)?);
    let mut size = u64::from(u32_at(b, eocd.checked_add(12)?)?);
    let mut offset = u64::from(u32_at(b, eocd.checked_add(16)?)?);
    let comment = u64::from(u16_at(b, eocd.checked_add(20)?)?);
    let mut end = eocd;
    if (declared == 0xFFFF || size == u64::from(u32::MAX) || offset == u64::from(u32::MAX))
        && let Some(locator) = eocd.checked_sub(20)
        && b.get(locator..locator.saturating_add(4)) == Some(&LOCATOR_SIG[..])
        && let Some(record) =
            u64_at(b, locator.saturating_add(8)).and_then(|r| usize::try_from(r).ok())
        && b.get(record..record.saturating_add(4)) == Some(&EOCD64_SIG[..])
    {
        declared = u64_at(b, record.checked_add(32)?)?;
        size = u64_at(b, record.checked_add(40)?)?;
        offset = u64_at(b, record.checked_add(48)?)?;
        end = record;
    }
    let trailing = (eocd as u64)
        .saturating_add(EOCD_LEN as u64)
        .saturating_add(comment)
        != b.len() as u64;

    // Where the directory really is, against where it says it is: the difference is how
    // much was put in front of the archive.
    let start = end.checked_sub(usize::try_from(size).ok()?)?;
    if b.get(start..start.checked_add(4)?) != Some(&CENTRAL_SIG[..]) {
        return None;
    }
    let shift = (start as u64).checked_sub(offset);
    let mut entries = Vec::new();
    let mut at = start;
    let mut seen = 0u64;
    let mut clean = true;
    let mut too_many = false;
    while at < end {
        if b.get(at..at.saturating_add(4)) != Some(&CENTRAL_SIG[..]) {
            clean = false;
            break;
        }
        if entries.len() >= limits.max_entries {
            too_many = true;
            break;
        }
        seen = seen.saturating_add(1);
        let Some(next) = record_end(b, at) else {
            clean = false;
            break;
        };
        match central_record(b, at, shift) {
            Some(candidate) => entries.push(candidate),
            // A record that points nowhere. The ones after it are still worth reading.
            None => clean = false,
        }
        at = next;
    }
    Some(Directory {
        entries,
        complete: clean && at == end && seen == declared,
        around: trailing || shift != Some(0),
        too_many,
    })
}

/// Where the directory record at `at` ends, by its own lengths.
fn record_end(b: &[u8], at: usize) -> Option<usize> {
    let name = usize::from(u16_at(b, at.checked_add(28)?)?);
    let extra = usize::from(u16_at(b, at.checked_add(30)?)?);
    let comment = usize::from(u16_at(b, at.checked_add(32)?)?);
    at.checked_add(CENTRAL_LEN)?
        .checked_add(name)?
        .checked_add(extra)?
        .checked_add(comment)
}

fn central_record(b: &[u8], at: usize, shift: Option<u64>) -> Option<Candidate> {
    let flags = u16_at(b, at.checked_add(8)?)?;
    let method = u16_at(b, at.checked_add(10)?)?;
    let crc = u32_at(b, at.checked_add(16)?)?;
    let compressed = u32_at(b, at.checked_add(20)?)?;
    let uncompressed = u32_at(b, at.checked_add(24)?)?;
    let name_len = usize::from(u16_at(b, at.checked_add(28)?)?);
    let extra_len = usize::from(u16_at(b, at.checked_add(30)?)?);
    let local = u32_at(b, at.checked_add(42)?)?;
    let name_at = at.checked_add(CENTRAL_LEN)?;
    let name = b.get(name_at..name_at.checked_add(name_len)?)?.to_vec();
    let extra_at = name_at.checked_add(name_len)?;
    let extra = b.get(extra_at..extra_at.checked_add(extra_len)?)?;
    let (big_uncompressed, big_compressed, big_local) =
        zip64_extra(extra, (uncompressed, compressed), local);
    let local = big_local.unwrap_or(u64::from(local));
    let local = usize::try_from(local.checked_add(shift?)?).ok()?;
    if b.get(local..local.checked_add(4)?) != Some(&LOCAL_SIG[..]) {
        return None;
    }
    let name_in_local = usize::from(u16_at(b, local.checked_add(26)?)?);
    let extra_in_local = usize::from(u16_at(b, local.checked_add(28)?)?);
    let data = local
        .checked_add(LOCAL_LEN)?
        .checked_add(name_in_local)?
        .checked_add(extra_in_local)?;
    if data > b.len() {
        return None;
    }
    Some(Candidate {
        name,
        flags,
        method,
        header: local,
        data,
        compressed: Some(big_compressed.unwrap_or(u64::from(compressed))),
        uncompressed: Some(big_uncompressed.unwrap_or(u64::from(uncompressed))),
        crc: Some(crc),
    })
}

/// How an entry's stream read.
#[derive(Debug, Clone, Copy, Default)]
struct Inflated {
    /// Compressed bytes consumed.
    consumed: u64,
    /// Bytes produced.
    produced: u64,
    /// The checksum of what was produced.
    crc: u32,
    /// The stream reached its end marker.
    complete: bool,
    /// Stopped at `cap`, with more to come.
    over_cap: bool,
}

/// Inflate `data`, handing each chunk to `sink`, and stop at `cap` bytes out.
///
/// The buffer is fixed: nothing here is sized from the file.
fn inflate(data: &[u8], cap: u64, mut sink: impl FnMut(&[u8])) -> Inflated {
    let mut state = Decompress::new(false);
    let mut crc = Crc::new();
    let mut buffer = vec![0u8; INFLATE_CHUNK];
    let mut out = Inflated::default();
    loop {
        let input = usize::try_from(state.total_in())
            .ok()
            .and_then(|at| data.get(at..))
            .unwrap_or_default();
        let before_out = state.total_out();
        let before_in = state.total_in();
        let status = state.decompress(input, &mut buffer, FlushDecompress::None);
        let produced = usize::try_from(state.total_out().saturating_sub(before_out)).unwrap_or(0);
        let chunk = buffer.get(..produced).unwrap_or_default();
        let room = usize::try_from(cap.saturating_sub(out.produced)).unwrap_or(usize::MAX);
        if chunk.len() > room {
            let kept = chunk.get(..room).unwrap_or_default();
            crc.update(kept);
            sink(kept);
            out.produced = out.produced.saturating_add(kept.len() as u64);
            out.over_cap = true;
            break;
        }
        crc.update(chunk);
        sink(chunk);
        out.produced = out.produced.saturating_add(chunk.len() as u64);
        match status {
            Ok(Status::StreamEnd) => {
                out.complete = true;
                break;
            }
            Ok(_) if produced == 0 && state.total_in() == before_in => break,
            Ok(_) => {}
            Err(_) => break,
        }
    }
    out.consumed = state.total_in();
    out.crc = crc.sum();
    out
}

/// Where a stored entry whose size cannot be trusted ends: the next header's signature.
fn next_record_after(b: &[u8], from: usize) -> usize {
    let rest = b.get(from..).unwrap_or_default();
    rest.windows(4)
        .position(|w| w == LOCAL_SIG || w == CENTRAL_SIG || w == DESCRIPTOR_SIG || w == EOCD_SIG)
        .map_or(b.len(), |i| from.saturating_add(i))
}

/// The checksum in a data descriptor, if one sits at `at`.
fn descriptor_crc(b: &[u8], at: usize) -> Option<u32> {
    let at = if b.get(at..at.checked_add(4)?) == Some(&DESCRIPTOR_SIG[..]) {
        at.checked_add(4)?
    } else {
        at
    };
    u32_at(b, at)
}

/// What was learned about one entry.
#[derive(Debug, Clone)]
struct Read {
    candidate: Candidate,
    /// Found by scanning for local headers, not through a directory.
    scanned: bool,
    /// The compressed bytes.
    span: std::ops::Range<usize>,
    /// Actual bytes out, when they were all produced.
    produced: u64,
    crc: u32,
    /// The stream is whole: a raw copy would inflate.
    whole: bool,
    /// The checksum or size disagrees with the header.
    mismatch: bool,
    /// Past a limit, or beyond the budget: not inflated, or abandoned.
    over_limit: bool,
    /// Encrypted, or a method this engine cannot read.
    unreadable: bool,
}

/// Measure one entry, charging what it produces to `budget`.
fn measure(b: &[u8], candidate: Candidate, limits: &PackageLimits, budget: &mut u64) -> Read {
    let start = candidate.data;
    let available = b.get(start..).unwrap_or_default();
    let claimed = candidate
        .compressed
        .and_then(|c| usize::try_from(c).ok())
        .filter(|c| *c <= available.len());
    let mut read = Read {
        scanned: false,
        span: start..start,
        produced: 0,
        crc: 0,
        whole: false,
        mismatch: false,
        over_limit: false,
        unreadable: false,
        candidate,
    };
    let flags = read.candidate.flags;
    let method = read.candidate.method;
    if flags & FLAG_ENCRYPTED != 0 || (method != STORED && method != DEFLATED) {
        read.unreadable = true;
        read.span = start..claimed.map_or_else(|| next_record_after(b, start), |c| start + c);
        return read;
    }
    // Refused on the declaration, with nothing inflated. A stored entry cannot expand, so
    // its declaration is only a number: it costs what it holds.
    if method == DEFLATED
        && let (Some(claim), Some(out)) = (claimed, read.candidate.uncompressed)
    {
        let ratio_over = out
            .checked_div(claim as u64)
            .is_some_and(|r| r > limits.max_expansion_ratio)
            && out > RATIO_FLOOR;
        if ratio_over || out > *budget {
            read.span = start..start + claim;
            read.over_limit = true;
            return read;
        }
    }
    if method == STORED {
        let end = claimed.map_or_else(|| next_record_after(b, start), |c| start + c);
        let data = b.get(start..end).unwrap_or_default();
        read.span = start..end;
        if data.len() as u64 > *budget {
            read.over_limit = true;
            return read;
        }
        let mut crc = Crc::new();
        crc.update(data);
        read.crc = crc.sum();
        read.produced = data.len() as u64;
        // Cut short, when the header promised more than the file holds.
        read.whole = claimed.is_some() || flags & FLAG_DESCRIPTOR != 0;
        *budget = budget.saturating_sub(read.produced);
        let wanted = read.candidate.crc.or_else(|| descriptor_crc(b, end));
        read.mismatch = wanted.is_some_and(|w| w != read.crc)
            || read
                .candidate
                .uncompressed
                .is_some_and(|u| u != read.produced);
        return read;
    }
    // What the input could honestly produce at the ratio, or the budget if that is less.
    let input_len = claimed.unwrap_or(available.len());
    let ratio_cap = (input_len as u64)
        .saturating_mul(limits.max_expansion_ratio)
        .max(RATIO_FLOOR);
    let cap = ratio_cap.min(*budget);
    let input = available.get(..input_len).unwrap_or(available);
    let mut seen = inflate(input, cap, |_| {});
    // A declared compressed size that was damaged can cut the stream short, when the rest
    // of the file carries on: try without the claim.
    if !seen.complete && !seen.over_cap && claimed.is_some() {
        let again = inflate(available, cap, |_| {});
        if again.complete {
            seen = again;
        }
    }
    read.crc = seen.crc;
    read.produced = seen.produced;
    let used = usize::try_from(seen.consumed).unwrap_or(usize::MAX);
    *budget = budget.saturating_sub(seen.produced);
    if seen.over_cap {
        read.span = start..start.saturating_add(input_len);
        read.over_limit = true;
        return read;
    }
    read.span = start..start.saturating_add(used.min(available.len()));
    read.whole = seen.complete;
    if seen.complete {
        let wanted = read
            .candidate
            .crc
            .or_else(|| descriptor_crc(b, read.span.end));
        let wrong_crc = wanted.is_some_and(|w| w != seen.crc);
        let wrong_size = read
            .candidate
            .uncompressed
            .is_some_and(|u| u != seen.produced);
        read.mismatch = wrong_crc || wrong_size;
    }
    read
}

/// The whole of an entry's bytes, as far as they can be read, under a cap.
fn inflate_all(b: &[u8], read: &Read, cap: u64) -> Vec<u8> {
    let data = b.get(read.span.clone()).unwrap_or_default();
    if read.candidate.method == STORED {
        return data.to_vec();
    }
    let mut out = Vec::new();
    inflate(data, cap, |chunk| out.extend_from_slice(chunk));
    out
}

/// Entries by local-header scan, skipping what `covered` already holds.
fn scan_local_headers(
    b: &[u8],
    limits: &PackageLimits,
    budget: &mut u64,
    covered: &mut [std::ops::Range<usize>],
    out: &mut Vec<Read>,
    too_many: &mut bool,
) {
    covered.sort_by_key(|r| r.start);
    let mut at = 0usize;
    let mut next_covered = 0usize;
    while let Some(found) = find_sig(b, at, LOCAL_SIG) {
        while covered.get(next_covered).is_some_and(|r| r.end <= found) {
            next_covered += 1;
        }
        if let Some(span) = covered.get(next_covered)
            && span.start <= found
        {
            at = span.end.max(found.saturating_add(4));
            continue;
        }
        let Some(candidate) = local_header(b, found) else {
            at = found.saturating_add(4);
            continue;
        };
        if out.len() >= limits.max_entries {
            *too_many = true;
            return;
        }
        let mut read = measure(b, candidate, limits, budget);
        read.scanned = true;
        at = read.span.end.max(found.saturating_add(4));
        out.push(read);
    }
}

/// A clean archive being written by hand: this reader's output has to be exact, and a
/// raw copy of a damaged entry is not something `zip`'s writer can be asked for.
#[derive(Default)]
struct ImageWriter {
    bytes: Vec<u8>,
    central: Vec<u8>,
    entries: u32,
}

#[derive(Clone, Copy)]
struct ImageEntry<'a> {
    name: &'a str,
    method: u16,
    crc: u32,
    data: &'a [u8],
    uncompressed: u64,
}

impl ImageWriter {
    /// Add an entry. `false` if it would not fit in a plain ZIP, which leaves it out.
    fn add(&mut self, e: ImageEntry<'_>) -> bool {
        let offset = self.bytes.len() as u64;
        let name = e.name.as_bytes();
        let (Ok(name_len), Ok(compressed), Ok(uncompressed), Ok(local)) = (
            u16::try_from(name.len()),
            u32::try_from(e.data.len()),
            u32::try_from(e.uncompressed),
            u32::try_from(offset),
        ) else {
            return false;
        };
        // Half the 32-bit range, and short of the 16-bit count: the image stays a plain
        // ZIP, whatever overlapping entries would make it add up to.
        if offset.saturating_add(e.data.len() as u64) > u64::from(u32::MAX >> 1)
            || self.entries >= u32::from(u16::MAX) - 1
            || compressed == u32::MAX
            || uncompressed == u32::MAX
        {
            return false;
        }
        let flags: u16 = if e.name.is_ascii() { 0 } else { FLAG_UTF8 };
        let common = |out: &mut Vec<u8>| {
            out.extend_from_slice(&20u16.to_le_bytes()); // version needed
            out.extend_from_slice(&flags.to_le_bytes());
            out.extend_from_slice(&e.method.to_le_bytes());
            out.extend_from_slice(&0u16.to_le_bytes()); // time
            out.extend_from_slice(&0x0021u16.to_le_bytes()); // 1980-01-01
            out.extend_from_slice(&e.crc.to_le_bytes());
            out.extend_from_slice(&compressed.to_le_bytes());
            out.extend_from_slice(&uncompressed.to_le_bytes());
            out.extend_from_slice(&name_len.to_le_bytes());
            out.extend_from_slice(&0u16.to_le_bytes()); // extra
        };
        self.bytes.extend_from_slice(&LOCAL_SIG);
        common(&mut self.bytes);
        self.bytes.extend_from_slice(name);
        self.bytes.extend_from_slice(e.data);

        self.central.extend_from_slice(&CENTRAL_SIG);
        self.central.extend_from_slice(&20u16.to_le_bytes()); // version made by
        common(&mut self.central);
        self.central.extend_from_slice(&0u16.to_le_bytes()); // comment
        self.central.extend_from_slice(&0u16.to_le_bytes()); // disk
        self.central.extend_from_slice(&0u16.to_le_bytes()); // internal attributes
        self.central.extend_from_slice(&0u32.to_le_bytes()); // external attributes
        self.central.extend_from_slice(&local.to_le_bytes());
        self.central.extend_from_slice(name);
        self.entries += 1;
        true
    }

    fn finish(mut self) -> Vec<u8> {
        let offset = u32::try_from(self.bytes.len()).unwrap_or(u32::MAX);
        let size = u32::try_from(self.central.len()).unwrap_or(u32::MAX);
        let count = u16::try_from(self.entries).unwrap_or(u16::MAX);
        self.bytes.append(&mut self.central);
        self.bytes.extend_from_slice(&EOCD_SIG);
        self.bytes.extend_from_slice(&[0u8; 4]); // disks
        self.bytes.extend_from_slice(&count.to_le_bytes());
        self.bytes.extend_from_slice(&count.to_le_bytes());
        self.bytes.extend_from_slice(&size.to_le_bytes());
        self.bytes.extend_from_slice(&offset.to_le_bytes());
        self.bytes.extend_from_slice(&0u16.to_le_bytes());
        self.bytes
    }
}

/// The name a retained entry is stored under: minted here, ASCII, one segment, so it can
/// never be read as a path or reach a part another entry holds.
pub fn retained_name(ordinal: usize, original: &str) -> String {
    const TAIL: usize = 48;
    let clean: String = original
        .chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '_' | '-' | '.' => c,
            _ => '_',
        })
        .collect();
    let tail = clean.get(clean.len().saturating_sub(TAIL)..).unwrap_or("");
    format!("{RETAINED_DIR}{ordinal:05}-{tail}.{RETAINED_EXTENSION}")
}

/// Read what can be read of `bytes`.
///
/// Refuses only `CS-PKG-0001`, the whole-file limit, and `CS-PKG-0005`, when no entry can
/// be found at all. `accepts` says whether an entry's bytes are good, for choosing between
/// entries that name one part: the last it accepts wins, then the last that reads whole.
pub fn salvage(
    bytes: &[u8],
    limits: PackageLimits,
    accepts: impl Fn(&str, &[u8]) -> bool,
) -> Result<Salvaged, PackageError> {
    let input_len = bytes.len() as u64;
    if input_len > limits.max_input_bytes {
        return Err(PackageError::InputTooLarge {
            size: input_len,
            limit: limits.max_input_bytes,
        });
    }
    let mut faults = BTreeSet::new();
    let mut note = |kind: ContainerFaultKind, entry: &str| {
        faults.insert(ContainerFault {
            kind,
            entry: scrub_name(entry),
        });
    };

    let mut budget = limits.max_total_uncompressed;
    let mut reads: Vec<Read> = Vec::new();
    let mut covered = Vec::new();
    let mut too_many = false;
    let directory = read_directory(bytes, &limits);
    let complete = directory.as_ref().is_some_and(|d| d.complete);
    let around = directory.as_ref().is_some_and(|d| d.around);
    match directory {
        None => note(ContainerFaultKind::DirectoryRebuilt, ""),
        Some(directory) => {
            if directory.around {
                note(ContainerFaultKind::BytesAroundArchive, "");
            }
            too_many |= directory.too_many;
            if !directory.complete {
                note(ContainerFaultKind::DirectoryRebuilt, "");
            }
            for candidate in directory.entries {
                let read = measure(bytes, candidate, &limits, &mut budget);
                covered.push(read.candidate.header..read.span.end);
                reads.push(read);
            }
        }
    }
    // A directory that is whole and at the end of the file is believed, and anything not
    // in it is not an entry. Any other may be one from inside a stored entry, an embedded
    // workbook's, with the real directory cut off: so the scan looks at what it left out.
    if !complete || around {
        let found_by_directory = reads.len();
        scan_local_headers(
            bytes,
            &limits,
            &mut budget,
            &mut covered,
            &mut reads,
            &mut too_many,
        );
        if reads.len() > found_by_directory {
            note(ContainerFaultKind::DirectoryRebuilt, "");
        }
    }
    if too_many {
        note(ContainerFaultKind::TooManyEntries, "");
    }
    // Input order, whichever route found an entry. Directories hold nothing to keep.
    reads.sort_by_key(|r| r.span.start);
    reads.retain(|r| !r.candidate.name.ends_with(b"/"));
    // What sits inside an entry the scan found is that entry's content.
    let mut reach = 0usize;
    reads.retain(|r| {
        let inside = !r.scanned && r.span.end <= reach && r.span.start < reach;
        if r.scanned {
            reach = reach.max(r.span.end);
        }
        !inside
    });
    if reads.is_empty() {
        return Err(PackageError::NotAPackage);
    }

    let mut groups: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    let mut names = Vec::with_capacity(reads.len());
    for (i, read) in reads.iter().enumerate() {
        let name = std::str::from_utf8(&read.candidate.name).ok();
        let usable = name.filter(|n| n.len() <= limits.max_path_bytes && is_safe_part_path(n));
        names.push(usable.map(str::to_owned));
        if let Some(name) = usable {
            groups.entry(part_entry_key(name)).or_default().push(i);
        }
    }
    let mut chosen = BTreeSet::new();
    for group in groups.values() {
        let best = group.iter().rev().copied().find(|&i| {
            reads.get(i).is_some_and(|r| {
                if r.unreadable || r.over_limit || !r.whole {
                    return false;
                }
                // One entry needs no choosing, and is not inflated a second time for it.
                if group.len() == 1 {
                    return true;
                }
                let name = names.get(i).and_then(Option::as_deref).unwrap_or_default();
                accepts(name, &inflate_all(bytes, r, r.produced))
            })
        });
        let fallback = || {
            group
                .iter()
                .rev()
                .copied()
                .find(|&i| reads.get(i).is_some_and(|r| !r.unreadable && !r.over_limit))
        };
        if let Some(pick) = best.or_else(fallback) {
            chosen.insert(pick);
        }
    }

    let mut image = ImageWriter::default();
    let mut retained = Vec::new();
    let mut suspect = BTreeSet::new();
    for (i, read) in reads.iter().enumerate() {
        let raw_name = String::from_utf8_lossy(&read.candidate.name);
        let shown = scrub_name(&raw_name);
        let name = names.get(i).and_then(Option::as_deref);
        let data = bytes.get(read.span.clone()).unwrap_or_default();
        if let (Some(name), true) = (name, chosen.contains(&i)) {
            if read.whole && read.mismatch {
                note(ContainerFaultKind::ChecksumMismatch, name);
                suspect.insert(name.to_owned());
            }
            let added = if read.whole {
                image.add(ImageEntry {
                    name,
                    method: read.candidate.method,
                    crc: read.crc,
                    data,
                    uncompressed: read.produced,
                })
            } else {
                // A broken stream cannot be copied raw: what was read goes back deflated.
                note(ContainerFaultKind::Truncated, name);
                suspect.insert(name.to_owned());
                let content = inflate_all(bytes, read, read.produced);
                let mut crc = Crc::new();
                crc.update(&content);
                let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
                let written = encoder
                    .write_all(&content)
                    .ok()
                    .and_then(|()| encoder.finish().ok());
                written.is_some_and(|deflated| {
                    image.add(ImageEntry {
                        name,
                        method: DEFLATED,
                        crc: crc.sum(),
                        data: &deflated,
                        uncompressed: content.len() as u64,
                    })
                })
            };
            if !added {
                note(ContainerFaultKind::OverLimit, name);
            }
            continue;
        }
        let why = if name.is_none() {
            ContainerFaultKind::UnsafeName
        } else if read.over_limit {
            ContainerFaultKind::OverLimit
        } else if read.unreadable {
            ContainerFaultKind::Unreadable
        } else {
            ContainerFaultKind::DuplicateEntry
        };
        note(why, &shown);
        let stored_as = retained_name(i, &shown);
        let mut crc = Crc::new();
        crc.update(data);
        let added = image.add(ImageEntry {
            name: &stored_as,
            method: STORED,
            crc: crc.sum(),
            data,
            uncompressed: data.len() as u64,
        });
        if added {
            retained.push(RetainedEntry {
                stored_as,
                original: shown,
                why,
            });
        }
    }
    Ok(Salvaged {
        image: image.finish(),
        faults: faults.into_iter().collect(),
        retained,
        suspect,
    })
}

#[cfg(test)]
mod tests;
