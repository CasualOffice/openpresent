//! Bounded ZIP/OPC package admission and on-demand part reads.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt;
use std::io::{Cursor, Read, Write};
use std::sync::Arc;

use zip::write::{SimpleFileOptions, ZipWriter};
use zip::{CompressionMethod, ZipArchive};

use crate::directory::record_count;
use crate::error::PackageError;
use crate::limits::PackageLimits;
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
    /// Lower-cased part name to the name the archive actually uses. OPC compares
    /// part names case-insensitively, so a relationship may spell one differently.
    names: HashMap<String, String>,
    limits: PackageLimits,
    /// Bytes actually handed out by [`Package::read_part`] so far.
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

        let source: Arc<[u8]> = Arc::from(bytes);
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
            // opens here. OPC compares part names case-insensitively.
            if !entry.is_dir() && names.insert(name.to_lowercase(), name.clone()).is_some() {
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

        Ok(Package {
            archive,
            source,
            names,
            limits,
            consumed: 0,
        })
    }

    /// The package's bytes exactly as they arrived.
    pub fn source_bytes(&self) -> &[u8] {
        &self.source
    }

    /// The name a part actually has in the archive, matched case-insensitively.
    pub fn find_part(&self, name: &str) -> Option<&str> {
        self.names.get(&name.to_lowercase()).map(String::as_str)
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
        for name in edits.replace.keys() {
            if !is_safe_part_path(name) {
                return Err(PackageError::UnsafePath { path: name.clone() });
            }
        }
        // A fixed timestamp and method: determinism is a contract.
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        let failed = |_| PackageError::RewriteFailed;
        let mut out = ZipWriter::new(Cursor::new(Vec::new()));
        let mut written = BTreeSet::new();
        for i in 0..self.archive.len() {
            let raw = self.archive.by_index_raw(i).map_err(failed)?;
            let name = raw.name().map_err(failed)?.into_owned();
            let key = name.to_lowercase();
            if edits.remove.contains(&key) {
                continue;
            }
            if let Some((_, bytes)) = edits.replace.iter().find(|(n, _)| n.to_lowercase() == key) {
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
            if !written.contains(&name.to_lowercase()) {
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

    /// Bytes handed out by reads so far.
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
    pub fn read_part(&mut self, name: &str) -> Result<Vec<u8>, PackageError> {
        let total_cap = self.limits.max_total_uncompressed;
        let remaining = total_cap.saturating_sub(self.consumed);
        let mut entry = self
            .archive
            .by_name(name)
            .map_err(|_| PackageError::PartNotFound {
                name: name.to_owned(),
            })?;

        // Whichever is tighter: what this entry claims it will produce, or what
        // the package has left in its budget.
        let declared = entry.size();
        let cap = declared.min(remaining);
        let mut buf = Vec::with_capacity(initial_capacity(cap));
        // One byte past the cap, so exceeding it is detectable rather than
        // silently truncated — a truncated part is a corrupt document that
        // reports success.
        entry
            .by_ref()
            .take(cap.saturating_add(1))
            .read_to_end(&mut buf)
            .map_err(|e| {
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
        self.consumed = self.consumed.saturating_add(produced);
        Ok(buf)
    }
}

/// Changes to apply when writing a package. Everything not named here is carried
/// over untouched.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PartEdits {
    replace: BTreeMap<String, Vec<u8>>,
    remove: BTreeSet<String>,
}

impl PartEdits {
    /// Set a part's bytes, adding it if it does not exist.
    pub fn replace(&mut self, name: impl Into<String>, bytes: Vec<u8>) {
        let name = name.into();
        self.remove.remove(&name.to_lowercase());
        self.replace.insert(name, bytes);
    }

    /// Remove a part. OPC names compare case-insensitively.
    pub fn remove(&mut self, name: &str) {
        self.replace.retain(|n, _| !n.eq_ignore_ascii_case(name));
        self.remove.insert(name.to_lowercase());
    }

    /// Whether nothing would change.
    pub fn is_empty(&self) -> bool {
        self.replace.is_empty() && self.remove.is_empty()
    }
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
