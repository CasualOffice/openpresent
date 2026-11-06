//! Counting the central directory's records directly.
//!
//! The `zip` crate keys entries by name, so two entries with one name collapse
//! before anything above it can see them, and its entry count is the collapsed
//! one. Comparing against the records actually present is the only place the
//! duplicate is visible. The same walk covers a record count that disagrees with
//! the directory's size, which readers would resolve differently.

const EOCD_SIG: [u8; 4] = [0x50, 0x4b, 0x05, 0x06];
const EOCD64_SIG: [u8; 4] = [0x50, 0x4b, 0x06, 0x06];
const LOCATOR_SIG: [u8; 4] = [0x50, 0x4b, 0x06, 0x07];
const RECORD_SIG: [u8; 4] = [0x50, 0x4b, 0x01, 0x02];
const EOCD_LEN: usize = 22;
const RECORD_LEN: usize = 46;

fn u16_at(b: &[u8], at: usize) -> Option<u64> {
    let s = b.get(at..at.checked_add(2)?)?;
    Some(u64::from(u16::from_le_bytes([s[0], s[1]])))
}

fn u32_at(b: &[u8], at: usize) -> Option<u64> {
    let s = b.get(at..at.checked_add(4)?)?;
    Some(u64::from(u32::from_le_bytes([s[0], s[1], s[2], s[3]])))
}

fn u64_at(b: &[u8], at: usize) -> Option<u64> {
    let s = b.get(at..at.checked_add(8)?)?;
    Some(u64::from_le_bytes(s.try_into().ok()?))
}

/// The end-of-central-directory record: the last signature whose comment length
/// reaches exactly to the end of the input.
fn find_eocd(b: &[u8]) -> Option<usize> {
    let last = b.len().checked_sub(EOCD_LEN)?;
    let first = last.saturating_sub(usize::from(u16::MAX));
    (first..=last).rev().find(|&at| {
        b[at..at + 4] == EOCD_SIG
            && u16_at(b, at + 20).is_some_and(|c| at as u64 + 22 + c == b.len() as u64)
    })
}

/// The number of records in the central directory, or `None` if the directory
/// is not self-consistent: a record is malformed, or the walk does not land
/// exactly on the declared size and count.
pub(crate) fn record_count(b: &[u8]) -> Option<u64> {
    let eocd = find_eocd(b)?;
    let mut declared = u16_at(b, eocd + 10)?;
    let mut size = u32_at(b, eocd + 12)?;
    let mut end = eocd;

    if declared == 0xFFFF || size == 0xFFFF_FFFF {
        let locator = eocd.checked_sub(20)?;
        if b.get(locator..locator + 4)? != LOCATOR_SIG {
            return None;
        }
        let record = usize::try_from(u64_at(b, locator + 8)?).ok()?;
        if b.get(record..record.checked_add(4)?)? != EOCD64_SIG {
            return None;
        }
        declared = u64_at(b, record + 32)?;
        size = u64_at(b, record + 40)?;
        end = record;
    }

    let start = end.checked_sub(usize::try_from(size).ok()?)?;
    let mut at = start;
    let mut count = 0u64;
    while at < end {
        if b.get(at..at.checked_add(4)?)? != RECORD_SIG {
            return None;
        }
        let name = u16_at(b, at + 28)?;
        let extra = u16_at(b, at + 30)?;
        let comment = u16_at(b, at + 32)?;
        at = at
            .checked_add(RECORD_LEN)?
            .checked_add(usize::try_from(name + extra + comment).ok()?)?;
        count += 1;
    }
    (at == end && count == declared).then_some(count)
}
