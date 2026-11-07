//! Bounded ZIP/OPC package admission and on-demand part reads.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt;
use std::io::{Cursor, Read, Write};
use std::sync::Arc;

use zip::write::{SimpleFileOptions, ZipWriter};
use zip::{CompressionMethod, DateTime, ZipArchive};

use crate::directory::record_count;
use crate::error::PackageError;
use crate::limits::PackageLimits;
use crate::part_name::part_entry_key;
use crate::path::is_safe_part_path;

/// Metadata for one admitted entry (part).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryInfo {
    /// The part's path within the package.
    pub name: String,
    /// Declared uncompressed size in bytes.
    pub uncompressed: u64,
    /// Compressed size in bytes.
    pub compressed: u64,
}

/// A security-bounded, admitted ZIP/OPC package.
///
/// [`Package::open`] validates the whole archive against [`PackageLimits`]
/// *before* any part is decompressed; parts are then read on demand and under
/// the same size ceiling via [`Package::read_part`]. This substrate is
/// format-neutral: it backs `.pptx` and any later OPC format, as it backed `.xlsx` and `.ods` admission.
pub struct Package {
    archive: ZipArchive<Cursor<Arc<[u8]>>>,
    /// The bytes as they arrived, kept so an unedited save can return them.
    source: Arc<[u8]>,
    /// Each part's equivalence key ([`part_entry_key`]) to the name the archive
    /// actually uses. OPC compares part names after percent-decoding, and ASCII
    /// case-insensitively, so a relationship may spell one differently (`PKG-02`).
    names: HashMap<String, String>,
    limits: PackageLimits,
    /// Bytes charged to the whole-package budget by [`Package::read_part`]: each
    /// part's, once, however often it is read (`PKG-01`).
    ///
    /// The whole-package ceiling is checked at admission against the sizes the
    /// central directory *declares*, which is the only information available
    /// before decompressing anything. That check is worth nothing on its own:
    /// the declaration is attacker-controlled, and a part that says 1,000 and
    /// produces 600 MiB was admitted and then expanded. So the same ceiling is
    /// charged again here, against bytes that actually exist — five parts of
    /// 200 MiB each cannot each pass a 4 GiB test individually and add up to a
    /// gigabyte of retained memory.
    consumed: u64,
    /// The entries already charged, by index.
    charged: HashSet<usize>,
    /// The host's time budget (`SEC-P10`): asked between reads while a part decompresses,
    /// and a part read stops when it answers `true`. The package never samples a clock.
    cancel: Option<Cancel>,
}

/// A host-supplied predicate that says when to stop: a deadline passed, or the user gave up.
pub type Cancel = Arc<dyn Fn() -> bool + Send + Sync>;

/// A reader that asks [`Cancel`] before every read and stops with `TimedOut` when told to.
struct Cancellable<'a, R> {
    inner: R,
    cancel: Option<&'a Cancel>,
}

impl<R: Read> Read for Cancellable<'_, R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.cancel.is_some_and(|cancel| cancel()) {
            return Err(std::io::ErrorKind::TimedOut.into());
        }
        self.inner.read(buf)
    }
}

impl fmt::Debug for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Package")
            .field("entries", &self.archive.len())
            .field("limits", &self.limits)
            .finish()
    }
}

impl Package {
    /// Admit `bytes` as a package under `limits`, or fail cleanly.
    ///
    /// Enforces, in order: input size, entry count, per-entry path safety and
    /// length, cumulative uncompressed size, and overall expansion ratio. On any
    /// breach the package is rejected — never partially admitted.
    pub fn open(bytes: Vec<u8>, limits: PackageLimits) -> Result<Package, PackageError> {
        Self::open_shared(Arc::from(bytes), limits)
    }

    /// As [`Package::open`], over bytes someone else also keeps.
    pub fn open_shared(bytes: Arc<[u8]>, limits: PackageLimits) -> Result<Package, PackageError> {
        let input_len = bytes.len() as u64;
        if input_len > limits.max_input_bytes {
            return Err(PackageError::InputTooLarge {
                size: input_len,
                limit: limits.max_input_bytes,
            });
        }

        // Counted from the bytes, before `zip` collapses same-named entries.
        let records = record_count(&bytes).ok_or(PackageError::NotAPackage)?;
        let records = usize::try_from(records).unwrap_or(usize::MAX);
        if records > limits.max_entries {
            return Err(PackageError::TooManyEntries {
                count: records,
                limit: limits.max_entries,
            });
        }

        let source = bytes;
        let mut archive = ZipArchive::new(Cursor::new(Arc::clone(&source)))
            .map_err(|_| PackageError::NotAPackage)?;

        let count = archive.len();
        if count != records {
            return Err(PackageError::DuplicatePart {
                path: format!(
                    "{} of {records} directory records",
                    records - count.min(records)
                ),
            });
        }

        let mut total_uncompressed: u64 = 0;
        let mut total_compressed: u64 = 0;
        // Bounded by `max_entries`, checked above.
        let mut names = HashMap::with_capacity(count);
        // Where each entry's header starts and its stored bytes end, and where the
        // directory starts: an entry may not run into the next one (`FUZZ-02`).
        let mut extents = Vec::with_capacity(count);
        let mut directory = input_len;

        for i in 0..count {
            // Reading metadata does not decompress the entry.
            let entry = archive.by_index(i).map_err(|_| PackageError::NotAPackage)?;
            // A name that does not decode is a damaged directory, refused the
            // way an unreadable entry is. Checking every name here is what lets
            // the listings below treat names as always decodable.
            let name = entry
                .name()
                .map_err(|_| PackageError::NotAPackage)?
                .into_owned();

            if name.len() > limits.max_path_bytes {
                return Err(PackageError::PathTooLong {
                    len: name.len(),
                    limit: limits.max_path_bytes,
                });
            }
            if !entry.is_dir() && !is_safe_part_path(&name) {
                return Err(PackageError::UnsafePath { path: name });
            }
            // Which of two same-named entries a reader returns differs between
            // ZIP implementations, so what a scanner approved need not be what
            // opens here. OPC compares part names after percent-decoding, and
            // ASCII case-insensitively: `a%20b` is `A B`, and `é` is not `É`.
            if !entry.is_dir() && names.insert(part_entry_key(&name), name.clone()).is_some() {
                return Err(PackageError::DuplicatePart { path: name });
            }

            // Per entry as well as overall, because ordinary parts dilute the
            // overall ratio: a run of zeros at 1030:1 passes in a package with a
            // few percent of anything else.
            if let Some(ratio) = entry.size().checked_div(entry.compressed_size())
                && ratio > limits.max_expansion_ratio
            {
                return Err(PackageError::ExpansionRatioExceeded {
                    ratio,
                    limit: limits.max_expansion_ratio,
                });
            }

            directory = directory.min(entry.central_header_start());
            // The local header names the entry the directory names: a raw copy writes one
            // name from the other, and a file whose two disagree cannot be read back.
            if local_name(&source, entry.header_start()) != Some(entry.name_raw()) {
                return Err(PackageError::NotAPackage);
            }
            // And the sizes it declares, unless they are deferred to a data descriptor or to
            // a ZIP64 field: a reader trusting the local header would otherwise stop at a
            // different byte from one trusting the directory (`SEC-P11`).
            if !local_sizes_agree(
                &source,
                entry.header_start(),
                entry.compressed_size(),
                entry.size(),
            ) {
                return Err(PackageError::NotAPackage);
            }
            // Opening the entry read its local header, so where its data starts is known.
            if let Some(start) = entry.data_start() {
                let data_end = start
                    .checked_add(entry.compressed_size())
                    .ok_or(PackageError::NotAPackage)?;
                extents.push((entry.header_start(), data_end));
            }

            total_uncompressed = total_uncompressed.saturating_add(entry.size());
            total_compressed = total_compressed.saturating_add(entry.compressed_size());

            if total_uncompressed > limits.max_total_uncompressed {
                return Err(PackageError::ExpansionTooLarge {
                    total: total_uncompressed,
                    limit: limits.max_total_uncompressed,
                });
            }
        }

        if let Some(ratio) = total_uncompressed.checked_div(total_compressed)
            && ratio > limits.max_expansion_ratio
        {
            return Err(PackageError::ExpansionRatioExceeded {
                ratio,
                limit: limits.max_expansion_ratio,
            });
        }

        // A stored size reaching past the next entry, or past the directory, is a damaged
        // directory: a raw copy would carry that many bytes, or stop short of them, into a
        // file that then contradicts itself. Refused here, so recovery reads past it.
        extents.sort_unstable();
        let mut next_starts = extents.iter().skip(1).map(|&(start, _)| start);
        for &(_, end) in &extents {
            if end > next_starts.next().unwrap_or(directory) {
                return Err(PackageError::NotAPackage);
            }
        }

        Ok(Package {
            archive,
            source,
            names,
            limits,
            consumed: 0,
            charged: HashSet::new(),
            cancel: None,
        })
    }

    /// Set the host's time budget (`SEC-P10`). Decompression is bounded in bytes, not in
    /// time, so a deck inside every size cap can still be slow to inflate; `cancel` is asked
    /// between reads, and a part read it stops fails with [`PackageError::Cancelled`]. Time is
    /// supplied, never sampled: the predicate is the host's, whatever clock it keeps.
    pub fn set_cancel(&mut self, cancel: Option<Cancel>) {
        self.cancel = cancel;
    }

    /// The package's bytes exactly as they arrived.
    pub fn source_bytes(&self) -> &[u8] {
        &self.source
    }

    /// Whether every entry's stream reads to its end, and matches its checksum. Admission
    /// looks only at the directory, so this is how a flipped byte is found before a raw copy
    /// would carry it into a new file. Each stream goes to a sink, held to what its entry
    /// declared; nothing is retained or charged.
    pub fn verify_entries(&mut self) -> bool {
        for i in 0..self.archive.len() {
            let Ok(mut entry) = self.archive.by_index(i) else {
                return false;
            };
            if entry.is_dir() {
                continue;
            }
            if std::io::copy(&mut entry, &mut std::io::sink()).is_err() {
                return false;
            }
        }
        true
    }

    /// A part's bytes as they are stored, not decompressed. For keeping a part too large
    /// to open.
    pub fn compressed_bytes(&mut self, name: &str) -> Result<Vec<u8>, PackageError> {
        let not_found = || PackageError::PartNotFound {
            name: name.to_owned(),
        };
        let index = self.archive.index_for_name(name).ok_or_else(not_found)?;
        let mut raw = self.archive.by_index_raw(index).map_err(|_| not_found())?;
        let mut out = Vec::with_capacity(initial_capacity(raw.compressed_size()));
        raw.read_to_end(&mut out).map_err(|_| not_found())?;
        Ok(out)
    }

    /// The name a part actually has in the archive, matched by OPC's equivalence:
    /// percent-encoding decoded once, and ASCII letters in either case.
    pub fn find_part(&self, name: &str) -> Option<&str> {
        self.names.get(&part_entry_key(name)).map(String::as_str)
    }

    /// Write a new package with `edits` applied.
    ///
    /// **An untouched part keeps its compressed bytes:** it is copied raw, never
    /// recompressed. With no edits the source comes back byte for byte. Entries
    /// keep their order, and new parts follow in name order, so the same edits
    /// always produce the same bytes.
    pub fn rewrite(&mut self, edits: &PartEdits) -> Result<Vec<u8>, PackageError> {
        if edits.is_empty() {
            return Ok(self.source.to_vec());
        }
        // By key, so that many edits over many entries cost their sum and not their product.
        let mut replacing = HashMap::with_capacity(edits.replace.len());
        for (name, bytes) in &edits.replace {
            if !is_safe_part_path(name) {
                return Err(PackageError::UnsafePath { path: name.clone() });
            }
            // Two spellings of one part would write a package this crate refuses.
            if replacing
                .insert(part_entry_key(name), bytes.as_slice())
                .is_some()
            {
                return Err(PackageError::DuplicatePart { path: name.clone() });
            }
        }
        // A fixed timestamp and method: determinism is a contract. The time is set
        // here, not left to `zip`'s default, which is the clock when its `time`
        // feature is on, as any crate in the build could make it (`PKG-03`).
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .last_modified_time(DateTime::default());
        let failed = |_| PackageError::RewriteFailed;
        let mut out = ZipWriter::new(Cursor::new(Vec::new()));
        let mut written = BTreeSet::new();
        for i in 0..self.archive.len() {
            let raw = self.archive.by_index_raw(i).map_err(failed)?;
            let name = raw.name().map_err(failed)?.into_owned();
            let key = part_entry_key(&name);
            if edits.remove.contains(&key) {
                continue;
            }
            if let Some(bytes) = replacing.get(&key) {
                drop(raw);
                out.start_file(name.as_str(), options).map_err(failed)?;
                out.write_all(bytes)
                    .map_err(|_| PackageError::RewriteFailed)?;
                written.insert(key);
            } else {
                out.raw_copy_file(raw).map_err(failed)?;
            }
        }
        for (name, bytes) in &edits.replace {
            if !written.contains(&part_entry_key(name)) {
                out.start_file(name.as_str(), options).map_err(failed)?;
                out.write_all(bytes)
                    .map_err(|_| PackageError::RewriteFailed)?;
            }
        }
        Ok(out.finish().map_err(failed)?.into_inner())
    }

    /// The number of entries in the package.
    pub fn len(&self) -> usize {
        self.archive.len()
    }

    /// Whether the package has no entries.
    pub fn is_empty(&self) -> bool {
        self.archive.is_empty()
    }

    /// The paths of all entries, in archive order.
    pub fn entry_names(&self) -> Vec<String> {
        // Every name decoded when the package was opened (`open`), so none is
        // dropped here.
        self.archive
            .file_names()
            .filter_map(Result::ok)
            .map(std::borrow::Cow::into_owned)
            .collect()
    }

    /// Metadata for every entry, in archive order.
    pub fn entries(&mut self) -> Result<Vec<EntryInfo>, PackageError> {
        let count = self.archive.len();
        let mut out = Vec::with_capacity(count);
        for i in 0..count {
            let entry = self
                .archive
                .by_index(i)
                .map_err(|_| PackageError::NotAPackage)?;
            out.push(EntryInfo {
                name: entry
                    .name()
                    .map_err(|_| PackageError::NotAPackage)?
                    .into_owned(),
                uncompressed: entry.size(),
                compressed: entry.compressed_size(),
            });
        }
        Ok(out)
    }

    /// Whether a part with the given path exists.
    pub fn contains(&self, name: &str) -> bool {
        self.archive
            .file_names()
            .any(|n| n.is_ok_and(|n| n == name))
    }

    /// Bytes charged to the package's budget so far: each part read, once.
    pub fn consumed(&self) -> u64 {
        self.consumed
    }

    /// Read a part only if it declares at most `max` bytes, without decompressing
    /// anything otherwise. [`Package::read_part`] holds the bytes to the
    /// declaration, so the declaration bounds this too.
    pub fn read_part_capped(&mut self, name: &str, max: u64) -> Result<Vec<u8>, PackageError> {
        let declared = self
            .archive
            .by_name(name)
            .map_err(|_| PackageError::PartNotFound {
                name: name.to_owned(),
            })?
            .size();
        if declared > max {
            return Err(PackageError::PartTooLarge {
                name: name.to_owned(),
                size: declared,
                limit: max,
            });
        }
        self.read_part(name)
    }

    /// Read a part's decompressed bytes, bounded by what it declared and by
    /// what the package as a whole has already produced.
    ///
    /// **The declared size is the bound, not a hint.** [`Package::open`] adds
    /// up `entry.size()` across the archive and refuses a package whose total
    /// or expansion ratio is too high — but every one of those numbers comes
    /// from the central directory, which is part of the attacker's input.
    /// Patching one four-byte field turned a 1 MB file that was correctly
    /// rejected as a 1028:1 bomb into one that was admitted and then handed
    /// back 600 MiB. The admission check is only meaningful if the bytes are
    /// held to the declaration it was computed from, so a part that produces
    /// more than it said is refused here.
    ///
    /// The whole-package ceiling is charged a second time, cumulatively, for
    /// the same reason: each part staying under a 4 GiB cap says nothing about
    /// five of them together, and unmodelled parts are *retained* in the model,
    /// so the bytes stay resident.
    ///
    /// **A part is charged once** (`PKG-01`). Reading it again produces the same
    /// bytes, held to the same declaration, so it adds nothing the ledger does not
    /// already count; charging it again would refuse an editor's long session on a
    /// legitimate deck. Since every part is held to its declaration, and admission
    /// held the declarations' sum to this same ceiling, the ledger is a second
    /// line, reached only if the first fails.
    pub fn read_part(&mut self, name: &str) -> Result<Vec<u8>, PackageError> {
        let total_cap = self.limits.max_total_uncompressed;
        let index =
            self.archive
                .index_for_name(name)
                .ok_or_else(|| PackageError::PartNotFound {
                    name: name.to_owned(),
                })?;
        let charged = self.charged.contains(&index);
        let remaining = if charged {
            u64::MAX
        } else {
            total_cap.saturating_sub(self.consumed)
        };
        let mut entry = self
            .archive
            .by_index(index)
            .map_err(|_| PackageError::PartNotFound {
                name: name.to_owned(),
            })?;

        // Whichever is tighter: what this entry claims it will produce, or what
        // the package has left in its budget.
        let declared = entry.size();
        let cap = declared.min(remaining);
        let mut buf = Vec::with_capacity(initial_capacity(cap));
        #[cfg(test)]
        LAST_UP_FRONT.with(|c| c.set(buf.capacity()));
        // One byte past the cap, so exceeding it is detectable rather than
        // silently truncated — a truncated part is a corrupt document that
        // reports success.
        Cancellable {
            inner: entry.by_ref(),
            cancel: self.cancel.as_ref(),
        }
        .take(cap.saturating_add(1))
        .read_to_end(&mut buf)
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::TimedOut {
                return PackageError::Cancelled {
                    name: name.to_owned(),
                };
            }
            // zip 9 enforces the declared size itself and stops the read
            // with this error, before our own `take` would see the byte
            // past it. It is the same defect as `produced > cap` below — a
            // part producing more than it declared — so it is reported as
            // that, not as a damaged package. Matched on zip's message:
            // if a release rewords it,
            // `a_part_that_produces_more_than_it_declared_is_refused` turns
            // red rather than the code quietly changing.
            if e.kind() == std::io::ErrorKind::InvalidData
                && e.to_string()
                    .contains("larger than its declared uncompressed size")
            {
                PackageError::ExpansionTooLarge {
                    total: self.consumed.saturating_add(cap.saturating_add(1)),
                    limit: total_cap,
                }
            } else {
                PackageError::NotAPackage
            }
        })?;

        let produced = buf.len() as u64;
        if produced > cap {
            return Err(PackageError::ExpansionTooLarge {
                total: self.consumed.saturating_add(produced),
                limit: total_cap,
            });
        }
        if !charged {
            self.consumed = self.consumed.saturating_add(produced);
            self.charged.insert(index);
        }
        Ok(buf)
    }
}

/// Changes to apply when writing a package. Everything not named here is carried
/// over untouched.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PartEdits {
    replace: BTreeMap<String, Vec<u8>>,
    /// Which name in `replace` holds each key, so a later spelling of one part finds the
    /// earlier without a search: a repair may edit every part of a hostile deck.
    spelled: HashMap<String, String>,
    remove: BTreeSet<String>,
}

impl PartEdits {
    /// Set a part's bytes, adding it if it does not exist. Any equivalent spelling
    /// of an existing part's name replaces that part.
    pub fn replace(&mut self, name: impl Into<String>, bytes: Vec<u8>) {
        let name = name.into();
        let key = part_entry_key(&name);
        self.remove.remove(&key);
        if let Some(earlier) = self.spelled.insert(key, name.clone()) {
            self.replace.remove(&earlier);
        }
        self.replace.insert(name, bytes);
    }

    /// Remove a part, named in any spelling equivalent under OPC's rules.
    pub fn remove(&mut self, name: &str) {
        let key = part_entry_key(name);
        if let Some(earlier) = self.spelled.remove(&key) {
            self.replace.remove(&earlier);
        }
        self.remove.insert(key);
    }

    /// Whether nothing would change.
    pub fn is_empty(&self) -> bool {
        self.replace.is_empty() && self.remove.is_empty()
    }
}

#[cfg(test)]
thread_local! {
    /// What the last [`Package::read_part`] allocated before any byte arrived.
    pub(crate) static LAST_UP_FRONT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Up-front capacity for a part that declares `declared` bytes.
///
/// The declaration is attacker-controlled and a stored padding entry can satisfy
/// the expansion ratio, so it cannot size an allocation: in WebAssembly a failed
/// allocation aborts the whole instance rather than returning an error. Larger
/// parts grow as bytes actually arrive.
pub(crate) fn initial_capacity(declared: u64) -> usize {
    const MAX_UP_FRONT: u64 = 1 << 20;
    declared.min(MAX_UP_FRONT) as usize
}

/// The name a local header at `at` gives its entry, if the header is there and whole.
fn local_name(source: &[u8], at: u64) -> Option<&[u8]> {
    let at = usize::try_from(at).ok()?;
    let fixed = source.get(at..at.checked_add(30)?)?;
    if fixed.get(..4)? != b"PK\x03\x04" {
        return None;
    }
    let len = usize::from(u16::from_le_bytes([*fixed.get(26)?, *fixed.get(27)?]));
    let start = at.checked_add(30)?;
    source.get(start..start.checked_add(len)?)
}

/// Whether the local header at `at` declares the sizes the central directory does. A header
/// whose general-purpose flag bit 3 is set defers its sizes to a data descriptor, and one
/// holding `0xFFFF_FFFF` defers them to a ZIP64 extra field; neither is compared here.
fn local_sizes_agree(source: &[u8], at: u64, compressed: u64, uncompressed: u64) -> bool {
    let Some(fixed) = usize::try_from(at)
        .ok()
        .and_then(|at| source.get(at..at.checked_add(30)?))
    else {
        return false;
    };
    let word = |i: usize| -> Option<u32> {
        let b = fixed.get(i..i.checked_add(4)?)?;
        Some(u32::from_le_bytes([
            *b.first()?,
            *b.get(1)?,
            *b.get(2)?,
            *b.get(3)?,
        ]))
    };
    let (Some(flags), Some(local_compressed), Some(local_uncompressed)) = (
        fixed
            .get(6..8)
            .and_then(|b| Some(u16::from_le_bytes([*b.first()?, *b.get(1)?]))),
        word(18),
        word(22),
    ) else {
        return false;
    };
    if flags & 0x0008 != 0 {
        return true;
    }
    let agrees = |local: u32, central: u64| local == u32::MAX || u64::from(local) == central;
    agrees(local_compressed, compressed) && agrees(local_uncompressed, uncompressed)
}
