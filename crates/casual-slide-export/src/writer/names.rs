//! Part names and relationship ids: what the writer mints (`INV-PN-*`, `INV-RI-03`).

use std::collections::BTreeMap;

use casual_slide_package::part_entry_key;

use super::error::WriterError;

/// The name every comparison of part names goes through (`INV-PE-04`).
pub(crate) fn key(name: &str) -> String {
    part_entry_key(name.strip_prefix('/').unwrap_or(name))
}

/// The `.rels` part of `part`; `""` is the package.
pub(crate) fn rels_part_for(part: &str) -> String {
    match part.rsplit_once('/') {
        Some((dir, file)) => format!("{dir}/_rels/{file}.rels"),
        None if part.is_empty() => "_rels/.rels".to_owned(),
        None => format!("_rels/{part}.rels"),
    }
}

/// The source a `.rels` entry describes, or `None` if `name` is not one.
pub(crate) fn source_of_rels(name: &str) -> Option<String> {
    let stem = name.strip_suffix(".rels")?;
    let (dir, file) = stem.rsplit_once("_rels/")?;
    if !dir.is_empty() && !dir.ends_with('/') {
        return None;
    }
    Some(format!("{dir}{file}"))
}

/// Whether `name` is a relationships part.
pub(crate) fn is_rels_part(name: &str) -> bool {
    source_of_rels(&key(name)).is_some()
}

/// `INV-PN-02` (narrower set Inferred): what a minted segment may hold.
pub(crate) fn is_mintable_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-'))
}

/// A file name's family: stem without its trailing number, the number, and extension.
pub(crate) fn family_of(file: &str) -> (&str, Option<u64>, &str) {
    let (stem, ext) = file.rsplit_once('.').unwrap_or((file, ""));
    let base = stem.trim_end_matches(|c: char| c.is_ascii_digit());
    let digits = stem.get(base.len()..).unwrap_or("");
    (base, digits.parse().ok(), ext)
}

/// The next free number per family, one above the highest ever seen in this save, so
/// numbers of removed parts are not reused (`INV-PN-05`, Inferred).
#[derive(Debug, Default, Clone)]
pub(crate) struct Families {
    highest: BTreeMap<(String, String), u64>,
}

impl Families {
    pub(crate) fn note(&mut self, name: &str) {
        let k = key(name);
        let (dir, file) = k.rsplit_once('/').unwrap_or(("", &k));
        let (base, number, _) = family_of(file);
        let slot = self
            .highest
            .entry((dir.to_owned(), base.to_owned()))
            .or_insert(0);
        *slot = (*slot).max(number.unwrap_or(0));
    }

    /// The highest number in the family of `name`.
    pub(crate) fn highest_of(&self, name: &str) -> u64 {
        let k = key(name);
        let (dir, file) = k.rsplit_once('/').unwrap_or(("", &k));
        let (base, _, _) = family_of(file);
        self.highest
            .get(&(dir.to_owned(), base.to_owned()))
            .copied()
            .unwrap_or(0)
    }

    /// Mint a name in `dir`, whose file is `stem<N>.<ext>`. Never reuses a number.
    pub(crate) fn mint(&mut self, dir: &str, stem: &str, ext: &str) -> Result<String, WriterError> {
        // A non-ASCII name that arrived is never copied into a new one (`INV-PN-02`).
        let dir = if dir.split('/').all(is_mintable_segment) {
            dir
        } else {
            "ppt/retained"
        };
        let base = stem.trim_end_matches(|c: char| c.is_ascii_digit());
        let base: String = base
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'))
            .collect();
        let base = if base.is_empty() { "part" } else { &base };
        let family = (key(dir), base.to_ascii_lowercase());
        let slot = self.highest.entry(family).or_insert(0);
        let next = slot
            .checked_add(1)
            .ok_or(WriterError::Exhausted { what: "part name" })?;
        *slot = next;
        let ext: String = ext.chars().filter(char::is_ascii_alphanumeric).collect();
        Ok(if ext.is_empty() {
            format!("{dir}/{base}{next}")
        } else {
            format!("{dir}/{base}{next}.{ext}")
        })
    }
}

/// `INV-RI-03` (Inferred): `rId` plus one above the highest numeric `rIdN`, checked
/// against every other id. Falls back to the lowest free number at the top of the space.
pub(crate) fn next_rel_id<'a>(existing: impl Iterator<Item = &'a str> + Clone) -> String {
    let top = existing
        .clone()
        .filter_map(|id| id.strip_prefix("rId")?.parse::<u64>().ok())
        .max()
        .unwrap_or(0);
    let start = top.checked_add(1).unwrap_or(1);
    (start..)
        .map(|n| format!("rId{n}"))
        .find(|candidate| !existing.clone().any(|id| id == candidate))
        .unwrap_or_default()
}

/// A target for the part `target`, relative to `source`, or absolute when its siblings
/// are (`INV-RI-07`). Names are as stored, so nothing is encoded (`INV-PE-02`, `-03`).
pub(crate) fn target_for(source: &str, target: &str, absolute: bool) -> String {
    if absolute {
        return format!("/{target}");
    }
    if source.is_empty() {
        return target.to_owned();
    }
    let from: Vec<&str> = source.split('/').collect();
    let from = from.get(..from.len().saturating_sub(1)).unwrap_or(&[]);
    let to: Vec<&str> = target.split('/').collect();
    let shared = from.iter().zip(&to).take_while(|(a, b)| a == b).count();
    let mut out: Vec<&str> = vec![".."; from.len().saturating_sub(shared)];
    out.extend(to.get(shared..).unwrap_or(&[]));
    out.join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_id_is_one_above_the_highest_numeric_one() {
        let ids = ["rId7", "foo", "rIdX", "rId2"];
        assert_eq!(next_rel_id(ids.iter().copied()), "rId8");
        assert_eq!(next_rel_id(["foo"].iter().copied()), "rId1");
        let top = format!("rId{}", u64::MAX);
        assert_eq!(
            next_rel_id([top.as_str(), "rId1"].iter().copied()),
            "rId2",
            "the top of the number space falls back to the lowest free"
        );
    }

    #[test]
    fn a_family_never_reuses_a_number_and_ignores_case() {
        let mut f = Families::default();
        f.note("ppt/slides/Slide3.xml");
        f.note("ppt/slides/slide1.xml");
        assert_eq!(
            f.mint("ppt/slides", "slide", "xml").unwrap(),
            "ppt/slides/slide4.xml"
        );
        assert_eq!(
            f.mint("ppt/slides", "slide", "xml").unwrap(),
            "ppt/slides/slide5.xml"
        );
    }

    #[test]
    fn a_minted_name_never_carries_a_non_ascii_source() {
        let mut f = Families::default();
        let name = f.mint("ppt/médias", "résumé", "pñg").unwrap();
        assert_eq!(name, "ppt/retained/rsum1.pg");
        assert!(name.is_ascii());
    }

    #[test]
    fn a_target_is_relative_to_its_source() {
        assert_eq!(
            target_for("ppt/slides/slide1.xml", "ppt/media/a.png", false),
            "../media/a.png"
        );
        assert_eq!(
            target_for("ppt/presentation.xml", "ppt/slides/s.xml", false),
            "slides/s.xml"
        );
        assert_eq!(
            target_for("", "ppt/presentation.xml", false),
            "ppt/presentation.xml"
        );
        assert_eq!(target_for("ppt/a.xml", "ppt/b.xml", true), "/ppt/b.xml");
    }
}
