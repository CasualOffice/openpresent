//! The recovering reader, beside [`walk`](crate::walk) (`REC-03`, `docs/29-RECOVERY.md`).
//!
//! Strict first: a part `walk` admits never comes here. For one it refuses, this makes
//! the best reading it can and writes it out as XML that `walk` admits, so nothing above
//! has a second reader to disagree with. That output is the invariant, and fuzz target
//! `xml_recover` holds it.
//!
//! **Nothing is obeyed.** A document type is skipped and never interpreted. No entity is
//! expanded: the five predefined and character references stand, and any other reference
//! is literal text. A file's number sizes nothing. The reader is one loop over the input
//! with no recursion, bounded in size, elements, depth and attributes as `walk` is, and
//! a bound stops the reading rather than refusing it.
//!
//! **Damage is contained.** A repair marks the elements it is inside. Under
//! [`Prune::SlideLike`], one shape, or one optional child of a slide, holding a mark is
//! left out whole, since what remains of it cannot be trusted to mean what it meant; the
//! others are byte for byte what arrived. The bytes as they arrived are not lost: the
//! caller keeps them.
//!
//! **No document text in a repair** (`SEC-015`): a repair is a value of a fixed enum.

use std::borrow::Cow;
use std::collections::{BTreeSet, HashSet};

use crate::limits::OoxmlLimits;
use crate::xml::{
    Scope, XML_NS, XMLNS_NS, attribute_value, char_reference, declaration_refused, is_name_char,
    is_name_start, is_xml_char, split_qname,
};

/// Stands for a character XML does not allow, or a byte that was not text, until the
/// reader is at it and knows which element to mark. XML cannot hold it, so it is ours.
const MARK: char = '\u{FFFF}';
const MARK_LEN: usize = MARK.len_utf8();
/// How far a reference's `;` may be. The longest legitimate one is far shorter.
const MAX_REFERENCE: usize = 32;

const TRANSITIONAL_PREFIXES: [(&str, &str); 4] = [
    ("a", "http://schemas.openxmlformats.org/drawingml/2006/main"),
    (
        "r",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
    ),
    (
        "p",
        "http://schemas.openxmlformats.org/presentationml/2006/main",
    ),
    (
        "mc",
        "http://schemas.openxmlformats.org/markup-compatibility/2006",
    ),
];
const STRICT_PREFIXES: [(&str, &str); 4] = [
    ("a", "http://purl.oclc.org/ooxml/drawingml/main"),
    (
        "r",
        "http://purl.oclc.org/ooxml/officeDocument/relationships",
    ),
    ("p", "http://purl.oclc.org/ooxml/presentationml/main"),
    (
        "mc",
        "http://schemas.openxmlformats.org/markup-compatibility/2006",
    ),
];
/// Bound to a prefix nothing declared, so the part still reads. Nothing schema-bound uses it.
const UNDECLARED_NAMESPACE: &str = "urn:openpresent:undeclared-prefix";

/// What the reader had to do. Fixed values: nothing from the part.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum XmlRepair {
    /// A document type was skipped, and never interpreted.
    DoctypeSkipped,
    /// Unclosed or mismatched tags were closed, or stray ones dropped.
    NestingClosed,
    /// A bad, repeated or undeclared attribute or prefix was mended or left out.
    AttributeRepaired,
    /// The wrong encoding was declared, or bytes or characters XML does not allow.
    TextRepaired,
    /// A reference to an unknown entity was kept as literal text, and not expanded.
    ReferenceLiteral,
    /// Content outside the one root element was left out.
    OutsideRootDropped,
    /// A bound on the part stopped the reading, and what was read was kept.
    LimitStopped,
    /// A shape or optional child holding damage was left out whole.
    SubtreeLeftOut,
}

/// How much of what is damaged to keep.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prune {
    /// Keep everything that could be read.
    Nothing,
    /// A slide, layout, master, notes slide, or notes or handout master: leave out any
    /// shape, or optional child of the root or of `cSld`, that holds a repair. What is
    /// left is what arrived, so it is as valid as it was.
    SlideLike,
}

/// The best reading of a part.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveredXml {
    /// UTF-8 that [`walk`](crate::walk) admits, or empty if no root element could be found.
    pub bytes: Vec<u8>,
    /// What was done.
    pub repairs: BTreeSet<XmlRepair>,
    /// The root element's local name.
    pub root_local: Option<String>,
    /// How many subtrees [`Prune`] left out.
    pub left_out: usize,
}

impl RecoveredXml {
    /// Whether a root element was found at all.
    pub fn has_root(&self) -> bool {
        self.root_local.is_some()
    }
}

/// Windows-1252's C1 range, which is what a stray high byte in a real deck usually is.
const CP1252_C1: [char; 32] = [
    '\u{20AC}', '\u{81}', '\u{201A}', '\u{192}', '\u{201E}', '\u{2026}', '\u{2020}', '\u{2021}',
    '\u{2C6}', '\u{2030}', '\u{160}', '\u{2039}', '\u{152}', '\u{8D}', '\u{17D}', '\u{8F}',
    '\u{90}', '\u{2018}', '\u{2019}', '\u{201C}', '\u{201D}', '\u{2022}', '\u{2013}', '\u{2014}',
    '\u{2DC}', '\u{2122}', '\u{161}', '\u{203A}', '\u{153}', '\u{9D}', '\u{17E}', '\u{178}',
];

fn cp1252(byte: u8) -> char {
    match byte {
        0x80..=0x9F => CP1252_C1
            .get(usize::from(byte - 0x80))
            .copied()
            .unwrap_or(MARK),
        _ => char::from(byte),
    }
}

/// The encoding a declaration names, lowered, if the bytes open with one.
fn declared_encoding(bytes: &[u8]) -> Option<String> {
    let head = bytes.get(..bytes.len().min(256))?;
    if !head.starts_with(b"<?xml") {
        return None;
    }
    let end = head.windows(2).position(|w| w == b"?>")?;
    let decl = head.get(..end)?;
    let at = decl.windows(8).position(|w| w == b"encoding")?;
    let rest = decl.get(at + 8..)?;
    let open = rest.iter().position(|b| matches!(b, b'"' | b'\''))?;
    let quote = *rest.get(open)?;
    let rest = rest.get(open + 1..)?;
    let close = rest.iter().position(|b| *b == quote)?;
    Some(
        String::from_utf8_lossy(rest.get(..close)?)
            .trim()
            .to_ascii_lowercase(),
    )
}

/// Whether a declaration claims an encoding the bytes are not in: UTF-8 bytes declared as
/// anything else, which readers will disagree about. `walk` looks only at the mark.
pub fn declaration_disagrees(bytes: &[u8]) -> bool {
    if bytes.starts_with(&[0xFF, 0xFE]) || bytes.starts_with(&[0xFE, 0xFF]) {
        return false;
    }
    declared_encoding(bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes))
        .is_some_and(|e| e != "utf-8" && e != "utf8")
}

fn decode_units(rest: &[u8], word: fn([u8; 2]) -> u16) -> String {
    let units = rest.chunks_exact(2).filter_map(|c| match c {
        [a, b] => Some(word([*a, *b])),
        _ => None,
    });
    char::decode_utf16(units)
        .map(|c| c.unwrap_or(MARK))
        .collect()
}

fn decode_utf32(rest: &[u8], word: fn([u8; 4]) -> u32) -> String {
    rest.chunks_exact(4)
        .filter_map(|c| match c {
            [a, b, d, e] => Some(char::from_u32(word([*a, *b, *d, *e])).unwrap_or(MARK)),
            _ => None,
        })
        .collect()
}

/// Bytes to text by whatever evidence there is: a mark, a null pattern, a declaration, or
/// UTF-8 as OPC has it. Every byte stands for something; nothing fails.
fn decode_any(bytes: &[u8], repairs: &mut BTreeSet<XmlRepair>) -> String {
    match bytes {
        // Longest marks first: the UTF-32 little-endian mark begins with the UTF-16 one.
        [0xFF, 0xFE, 0, 0, rest @ ..] => {
            repairs.insert(XmlRepair::TextRepaired);
            return decode_utf32(rest, u32::from_le_bytes);
        }
        [0, 0, 0xFE, 0xFF, rest @ ..] => {
            repairs.insert(XmlRepair::TextRepaired);
            return decode_utf32(rest, u32::from_be_bytes);
        }
        [0xFF, 0xFE, rest @ ..] => return decode_units(rest, u16::from_le_bytes),
        [0xFE, 0xFF, rest @ ..] => return decode_units(rest, u16::from_be_bytes),
        // No mark, and the nulls of UTF-16 around the first `<`.
        [b'<', 0, ..] => {
            repairs.insert(XmlRepair::TextRepaired);
            return decode_units(bytes, u16::from_le_bytes);
        }
        [0, b'<', ..] => {
            repairs.insert(XmlRepair::TextRepaired);
            return decode_units(bytes, u16::from_be_bytes);
        }
        _ => {}
    }
    let body = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
    if declaration_disagrees(body) {
        repairs.insert(XmlRepair::TextRepaired);
    }
    if let Ok(text) = std::str::from_utf8(body) {
        return text.to_owned();
    }
    repairs.insert(XmlRepair::TextRepaired);
    // A declared single-byte encoding is believed, since the bytes are not UTF-8. Otherwise
    // what is UTF-8 stays so, and each stray byte is marked where it stands.
    let single_byte = declared_encoding(body).is_some_and(|e| {
        matches!(
            e.as_str(),
            "iso-8859-1" | "latin1" | "latin-1" | "windows-1252" | "cp1252" | "us-ascii" | "ascii"
        )
    });
    if single_byte {
        return body.iter().copied().map(cp1252).collect();
    }
    let mut out = String::with_capacity(body.len().min(1 << 20));
    for chunk in body.utf8_chunks() {
        out.push_str(chunk.valid());
        for byte in chunk.invalid() {
            out.push(MARK);
            out.push(cp1252(*byte));
        }
    }
    out
}

/// Characters XML does not allow become marks, and leading byte-order marks go.
fn clean_characters(text: String, repairs: &mut BTreeSet<XmlRepair>) -> String {
    let trimmed = text.trim_start_matches('\u{FEFF}');
    let had_bom = trimmed.len() != text.len();
    let clean = trimmed
        .chars()
        .all(|c| is_xml_char(u32::from(c)) || c == MARK);
    if !clean || had_bom {
        repairs.insert(XmlRepair::TextRepaired);
    }
    if clean && !had_bom {
        return text;
    }
    trimmed
        .chars()
        .map(|c| if is_xml_char(u32::from(c)) { c } else { MARK })
        .collect()
}

/// An element being read.
#[derive(Debug)]
struct Frame {
    name: String,
    local: String,
    /// Where its start tag begins in the output, so it can be taken back.
    start: usize,
    dirty: bool,
}

fn local_of(name: &str) -> &str {
    name.rsplit(':').next().unwrap_or(name)
}

fn is_xml_space(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\r' | b'\n')
}

/// Make any name into a qualified name: anything else in it goes, and a colon after the
/// first becomes `_`. `None` if nothing is left to name it.
fn repaired_name(raw: &str) -> Option<String> {
    let mut out = String::with_capacity(raw.len());
    let mut colons = 0;
    for c in raw.chars() {
        match c {
            ':' => {
                colons += 1;
                if out.is_empty() || out.ends_with(':') {
                    continue;
                }
                out.push(if colons == 1 { ':' } else { '_' });
            }
            c if is_name_char(c) => {
                let at_start = out.is_empty() || out.ends_with(':');
                if !at_start || is_name_start(c) {
                    out.push(c);
                }
            }
            _ => {}
        }
    }
    let out = out.trim_end_matches(':').to_owned();
    (!out.is_empty() && split_qname(out.as_bytes()).is_some()).then_some(out)
}

/// Text escaped for content or an attribute: every `&` that begins a reference XML allows
/// stands, any other is literal, and what the context cannot hold is escaped.
fn escape_loose(
    raw: &str,
    in_attribute: bool,
    out: &mut String,
    literal: &mut bool,
    mark: &mut bool,
) {
    let mut rest = raw;
    while let Some(c) = rest.chars().next() {
        match c {
            MARK => {
                *mark = true;
                rest = rest.get(MARK_LEN..).unwrap_or_default();
            }
            '&' => {
                let after = rest.get(1..).unwrap_or_default();
                let window = after.get(..after.len().min(MAX_REFERENCE)).unwrap_or(after);
                let body = window
                    .find(';')
                    .and_then(|end| after.get(..end))
                    .filter(|b| {
                        b.as_bytes() != b"lt"
                            && !b.is_empty()
                            && (matches!(*b, "gt" | "amp" | "apos" | "quot")
                                || char_reference(b.as_bytes()).is_some())
                    });
                let lt = window.strip_prefix("lt;").is_some();
                if lt {
                    out.push_str("&lt;");
                    rest = after.get(3..).unwrap_or_default();
                } else if let Some(body) = body {
                    out.push('&');
                    out.push_str(body);
                    out.push(';');
                    rest = after.get(body.len() + 1..).unwrap_or_default();
                } else {
                    *literal = true;
                    out.push_str("&amp;");
                    rest = after;
                }
            }
            '<' => {
                out.push_str("&lt;");
                rest = rest.get(1..).unwrap_or_default();
            }
            '>' => {
                out.push_str("&gt;");
                rest = rest.get(1..).unwrap_or_default();
            }
            '"' if in_attribute => {
                out.push_str("&quot;");
                rest = rest.get(1..).unwrap_or_default();
            }
            c => {
                out.push(c);
                rest = rest.get(c.len_utf8()..).unwrap_or_default();
            }
        }
    }
}

/// One attribute as the reader found it.
struct LooseAttribute {
    name: String,
    /// Escaped, ready to write between double quotes.
    value: String,
}

struct Reader<'t, 'l> {
    text: &'t str,
    at: usize,
    out: String,
    frames: Vec<Frame>,
    scope: Scope,
    repairs: BTreeSet<XmlRepair>,
    limits: &'l OoxmlLimits,
    prune: Prune,
    elements: usize,
    root_local: Option<String>,
    root_closed: bool,
    strict_namespaces: bool,
    left_out: usize,
    stopped: bool,
    /// A search for a closer found none, so none is ahead of any later opener either.
    /// Without these, a part of nothing but `<!--` costs its length squared.
    no_comment_end: bool,
    no_instruction_end: bool,
    /// As those, for a document type that ran to the end of the part.
    doctype_unclosed: bool,
}

impl Reader<'_, '_> {
    fn bytes(&self) -> &[u8] {
        self.text.as_bytes()
    }

    fn peek(&self, offset: usize) -> Option<u8> {
        self.bytes().get(self.at.saturating_add(offset)).copied()
    }

    fn rest(&self) -> &str {
        self.text.get(self.at..).unwrap_or_default()
    }

    /// Note a repair, and that the element it is inside is damaged.
    fn repair(&mut self, what: XmlRepair) {
        self.repairs.insert(what);
        if let Some(top) = self.frames.last_mut() {
            top.dirty = true;
        }
    }

    /// Whether this depth holds only elements, under the rule for slide-like parts: the
    /// root, `cSld` and `spTree`, where text is never content.
    fn in_element_only(&self) -> bool {
        self.prune == Prune::SlideLike && self.frames.len() <= 3
    }

    fn run(&mut self) {
        while self.at < self.text.len() && !self.stopped {
            if self.peek(0) == Some(b'<') {
                self.markup();
            } else {
                self.text_run();
            }
        }
        while !self.frames.is_empty() {
            self.repair(XmlRepair::NestingClosed);
            self.close(false);
        }
    }

    fn markup(&mut self) {
        let rest = self.rest();
        if rest.starts_with("<!--") {
            self.comment();
        } else if rest.starts_with("<![CDATA[") {
            self.cdata();
        } else if rest.starts_with("<!") {
            self.doctype();
        } else if rest.starts_with("<?") {
            self.instruction();
        } else if rest.starts_with("</") {
            self.end_tag();
        } else if rest
            .get(1..)
            .and_then(|r| r.chars().find(|c| *c != MARK))
            .is_some_and(|c| is_name_start(c) || c == ':')
        {
            self.start_tag();
        } else {
            // A `<` that opens nothing is a character someone meant, or damage.
            self.repair(XmlRepair::NestingClosed);
            if !self.frames.is_empty() {
                self.out.push_str("&lt;");
            }
            self.at += 1;
        }
    }

    fn text_run(&mut self) {
        let end = self
            .rest()
            .find('<')
            .map_or(self.text.len(), |i| self.at + i);
        let run = self.text.get(self.at..end).unwrap_or_default();
        self.at = end;
        let blank = run.bytes().all(is_xml_space);
        if self.frames.is_empty() {
            if blank {
                self.out.push_str(run);
            } else if self.root_closed {
                self.repairs.insert(XmlRepair::OutsideRootDropped);
                self.stopped = true;
            } else if run.chars().any(|c| c != MARK) {
                self.repairs.insert(XmlRepair::OutsideRootDropped);
            }
            return;
        }
        if blank {
            self.out.push_str(run);
            return;
        }
        if self.in_element_only() {
            self.repair(XmlRepair::OutsideRootDropped);
            return;
        }
        let (mut literal, mut mark) = (false, false);
        let mut buffer = String::with_capacity(run.len());
        escape_loose(run, false, &mut buffer, &mut literal, &mut mark);
        self.out.push_str(&buffer);
        if literal {
            self.repairs.insert(XmlRepair::ReferenceLiteral);
        }
        if mark {
            self.repair(XmlRepair::TextRepaired);
        }
    }

    fn comment(&mut self) {
        let body_at = self.at + 4;
        let found = if self.no_comment_end {
            None
        } else {
            self.text.get(body_at..).and_then(|r| r.find("-->"))
        };
        let Some(end) = found else {
            self.no_comment_end = true;
            self.repair(XmlRepair::NestingClosed);
            let next = self.text.get(body_at..).and_then(|r| r.find('<'));
            self.at = next.map_or(self.text.len(), |i| body_at + i);
            return;
        };
        let body = self.text.get(body_at..body_at + end).unwrap_or_default();
        self.at = body_at + end + 3;
        let bad = body.contains("--") || body.ends_with('-') || body.contains(MARK);
        if bad {
            self.repairs.insert(XmlRepair::TextRepaired);
        } else {
            self.out.push_str("<!--");
            self.out.push_str(body);
            self.out.push_str("-->");
        }
    }

    fn cdata(&mut self) {
        let body_at = self.at + 9;
        let found = self.text.get(body_at..).and_then(|r| r.find("]]>"));
        let body = match found {
            Some(end) => self.text.get(body_at..body_at + end).unwrap_or_default(),
            None => self.text.get(body_at..).unwrap_or_default(),
        };
        self.at = found.map_or(self.text.len(), |end| body_at + end + 3);
        if found.is_none() {
            self.repair(XmlRepair::NestingClosed);
        }
        if self.frames.is_empty() {
            self.repairs.insert(XmlRepair::OutsideRootDropped);
            return;
        }
        if self.in_element_only() {
            self.repair(XmlRepair::OutsideRootDropped);
            return;
        }
        if body.contains(MARK) {
            self.repair(XmlRepair::TextRepaired);
        }
        self.out.push_str("<![CDATA[");
        for c in body.chars().filter(|c| *c != MARK) {
            self.out.push(c);
        }
        self.out.push_str("]]>");
    }

    /// Skip a document type, whatever it says, and never look at what it says. Only the
    /// brackets and quotes that show where it ends are read.
    fn doctype(&mut self) {
        self.repairs.insert(XmlRepair::DoctypeSkipped);
        if self.doctype_unclosed {
            let rest = self.text.get(self.at + 2..).unwrap_or_default();
            self.at = rest.find('<').map_or(self.text.len(), |i| self.at + 2 + i);
            return;
        }
        let mut depth = 0usize;
        let mut quote = None;
        let bytes = self.bytes();
        let mut i = self.at + 2;
        let mut end = None;
        while let Some(&b) = bytes.get(i) {
            match (quote, b) {
                (Some(q), _) if b == q => quote = None,
                (Some(_), _) => {}
                (None, b'"' | b'\'') => quote = Some(b),
                (None, b'[') => depth = depth.saturating_add(1),
                (None, b']') => depth = depth.saturating_sub(1),
                (None, b'>') if depth == 0 => {
                    end = Some(i + 1);
                    break;
                }
                _ => {}
            }
            i += 1;
        }
        // Never closed: it took the rest of the part with it. Try again from the root.
        self.doctype_unclosed = end.is_none();
        self.at = end.unwrap_or_else(|| {
            let rest = self.text.get(self.at + 2..).unwrap_or_default();
            rest.find('<').map_or(self.text.len(), |i| self.at + 2 + i)
        });
    }

    fn instruction(&mut self) {
        let body_at = self.at + 2;
        let found = if self.no_instruction_end {
            None
        } else {
            self.text.get(body_at..).and_then(|r| r.find("?>"))
        };
        let Some(end) = found else {
            self.no_instruction_end = true;
            self.repair(XmlRepair::NestingClosed);
            let next = self.text.get(body_at..).and_then(|r| r.find('<'));
            self.at = next.map_or(self.text.len(), |i| body_at + i);
            return;
        };
        let body = self.text.get(body_at..body_at + end).unwrap_or_default();
        self.at = body_at + end + 2;
        let target = body
            .split(|c: char| c.is_ascii_whitespace())
            .next()
            .unwrap_or_default();
        if target.eq_ignore_ascii_case("xml") {
            // Text is UTF-8 here whatever the part said, so the declaration says so. Only
            // the first thing in the part can be one.
            if self.out.is_empty() {
                self.out
                    .push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"");
                if let Some(v) = standalone_of(body) {
                    self.out.push_str(" standalone=\"");
                    self.out.push_str(v);
                    self.out.push('"');
                }
                self.out.push_str("?>");
            } else {
                self.repairs.insert(XmlRepair::OutsideRootDropped);
            }
            return;
        }
        let good = target.chars().next().is_some_and(is_name_start)
            && target.chars().all(is_name_char)
            && !target.contains(':')
            && !body.contains(MARK);
        if good && !self.in_element_only() {
            self.out.push_str("<?");
            self.out.push_str(body);
            self.out.push_str("?>");
        } else {
            self.repairs.insert(XmlRepair::TextRepaired);
        }
    }

    fn end_tag(&mut self) {
        let start = self.at + 2;
        let rest = self.text.get(start..).unwrap_or_default();
        let stop = rest.find(['>', '<']).unwrap_or(rest.len());
        let inner = rest.get(..stop).unwrap_or_default();
        let closed_properly = rest.as_bytes().get(stop) == Some(&b'>');
        self.at = start + stop + usize::from(closed_properly);
        let name: String = inner
            .trim_matches(|c: char| c.is_ascii_whitespace() || c == MARK)
            .chars()
            .filter(|c| *c != MARK)
            .collect();
        if !closed_properly || inner.contains(MARK) {
            self.repair(XmlRepair::NestingClosed);
        }
        let Some(depth) = self.frames.iter().rposition(|f| f.name == name) else {
            // Closes nothing that is open.
            if self.frames.is_empty() && self.root_closed {
                self.repairs.insert(XmlRepair::OutsideRootDropped);
            } else {
                self.repair(XmlRepair::NestingClosed);
            }
            return;
        };
        while self.frames.len() > depth + 1 {
            self.repair(XmlRepair::NestingClosed);
            self.close(false);
        }
        self.close(false);
    }

    /// Close the top frame, and take it back if it is damaged and [`Prune`] says so.
    fn close(&mut self, self_closed: bool) {
        let Some(frame) = self.frames.pop() else {
            return;
        };
        self.scope.leave();
        if !self_closed {
            self.out.push_str("</");
            self.out.push_str(&frame.name);
            self.out.push('>');
        }
        if self.frames.is_empty() {
            self.root_closed = true;
        }
        if !frame.dirty {
            return;
        }
        if self.is_droppable(&frame) {
            self.out.truncate(frame.start);
            self.left_out = self.left_out.saturating_add(1);
            self.repairs.insert(XmlRepair::SubtreeLeftOut);
            return;
        }
        if let Some(parent) = self.frames.last_mut() {
            parent.dirty = true;
        }
    }

    fn is_droppable(&self, frame: &Frame) -> bool {
        if self.prune != Prune::SlideLike {
            return false;
        }
        let parent = self.frames.last().map(|f| f.local.as_str());
        match (self.frames.len(), parent) {
            (1, _) => frame.local != "cSld",
            (2, Some("cSld")) => frame.local != "spTree",
            (3, Some("spTree")) => true,
            _ => false,
        }
    }

    /// Read a value's source text starting at a quote, or a bare one.
    fn attribute_value_raw(&mut self) -> &str {
        let quote = self.peek(0);
        if let Some(q @ (b'"' | b'\'')) = quote {
            let start = self.at + 1;
            let rest = self.text.get(start..).unwrap_or_default();
            // Up to the first of them, so a value costs only what it consumes.
            let first = rest.as_bytes().iter().position(|b| *b == q || *b == b'<');
            if let Some(close) = first.filter(|at| rest.as_bytes().get(*at) == Some(&q)) {
                self.at = start + close + 1;
                return self.text.get(start..start + close).unwrap_or_default();
            }
            // The closing quote is gone, or is past the next tag. The value ends at the
            // first `>` or `<`, and the markup after it is still markup.
            let stop = rest.find(['>', '<']).unwrap_or(rest.len());
            let value = rest.get(..stop).unwrap_or_default();
            let value = value.strip_suffix('/').unwrap_or(value);
            self.at = start + value.len();
            self.repair(XmlRepair::AttributeRepaired);
            return self.text.get(start..self.at).unwrap_or_default();
        }
        let start = self.at;
        let rest = self.text.get(start..).unwrap_or_default();
        let stop = rest
            .find(|c: char| c.is_ascii_whitespace() || c == '>' || c == '<')
            .unwrap_or(rest.len());
        let value = rest.get(..stop).unwrap_or_default();
        let value = value.strip_suffix('/').unwrap_or(value);
        self.at = start + value.len();
        self.repair(XmlRepair::AttributeRepaired);
        self.text.get(start..self.at).unwrap_or_default()
    }

    fn start_tag(&mut self) {
        let begin = self.at;
        let name_at = self.at + 1;
        let rest = self.text.get(name_at..).unwrap_or_default();
        let stop = rest
            .find(|c: char| {
                c.is_ascii_whitespace() || matches!(c, '/' | '>' | '<' | '=' | '"' | '\'')
            })
            .unwrap_or(rest.len());
        let raw_name = rest.get(..stop).unwrap_or_default();
        self.at = name_at + stop;
        let Some(mut name) = repaired_name(raw_name) else {
            self.repair(XmlRepair::NestingClosed);
            if !self.frames.is_empty() {
                self.out.push_str("&lt;");
            }
            self.at = begin + 1;
            return;
        };
        // The second root of a part: nothing after the first is read.
        if self.root_closed {
            self.repairs.insert(XmlRepair::OutsideRootDropped);
            self.stopped = true;
            return;
        }
        if self.elements >= self.limits.max_xml_elements
            || self.frames.len() >= self.limits.max_xml_depth
        {
            self.repair(XmlRepair::LimitStopped);
            self.stopped = true;
            return;
        }
        self.elements += 1;
        // A frame first, so a repair in this tag marks this element.
        let start = self.out.len();
        if name.as_bytes().starts_with(b"xmlns:") {
            name = name.replacen("xmlns:", "xmlns_", 1);
            self.repairs.insert(XmlRepair::AttributeRepaired);
        }
        if raw_name != name {
            self.repairs.insert(XmlRepair::AttributeRepaired);
        }
        let local = local_of(&name).to_owned();
        self.frames.push(Frame {
            name: name.clone(),
            local: local.clone(),
            start,
            dirty: raw_name != name,
        });
        self.scope.enter(Vec::new());
        let (attributes, self_closed) = self.read_attributes();
        self.write_start(&name, attributes, self_closed);
        if self.root_local.is_none() {
            self.root_local = Some(local);
        }
        if self_closed {
            self.close(true);
        }
    }

    /// Read to the end of a start tag. `true` if it closes itself.
    fn read_attributes(&mut self) -> (Vec<LooseAttribute>, bool) {
        let mut found: Vec<LooseAttribute> = Vec::new();
        let mut names: HashSet<String> = HashSet::new();
        loop {
            let before = self.at;
            while self.peek(0).is_some_and(is_xml_space) || self.rest().starts_with(MARK) {
                if self.rest().starts_with(MARK) {
                    self.repair(XmlRepair::TextRepaired);
                    self.at += MARK_LEN;
                } else {
                    self.at += 1;
                }
            }
            match self.peek(0) {
                None => {
                    self.repair(XmlRepair::NestingClosed);
                    return (found, false);
                }
                Some(b'>') => {
                    self.at += 1;
                    return (found, false);
                }
                Some(b'/') => {
                    if self.peek(1) == Some(b'>') {
                        self.at += 2;
                        return (found, true);
                    }
                    self.repair(XmlRepair::AttributeRepaired);
                    self.at += 1;
                    continue;
                }
                // The tag never closed, and the next one begins.
                Some(b'<') => {
                    self.repair(XmlRepair::NestingClosed);
                    return (found, false);
                }
                _ => {}
            }
            // An attribute with no space before it, run on from the last one.
            if self.at == before {
                self.repair(XmlRepair::AttributeRepaired);
            }
            let rest = self.rest();
            let stop = rest
                .find(|c: char| {
                    c.is_ascii_whitespace() || matches!(c, '=' | '>' | '/' | '<' | '"' | '\'')
                })
                .unwrap_or(rest.len());
            if stop == 0 {
                // A stray `=` or quote.
                self.repair(XmlRepair::AttributeRepaired);
                self.at += 1;
                continue;
            }
            let raw_name = rest.get(..stop).unwrap_or_default().to_owned();
            self.at += stop;
            while self.peek(0).is_some_and(is_xml_space) {
                self.at += 1;
            }
            let raw_value = if self.peek(0) == Some(b'=') {
                self.at += 1;
                while self.peek(0).is_some_and(is_xml_space) {
                    self.at += 1;
                }
                self.attribute_value_raw().to_owned()
            } else {
                // A name with no value keeps its name.
                self.repair(XmlRepair::AttributeRepaired);
                String::new()
            };
            let name = repaired_name(&raw_name);
            let Some(name) = name else {
                self.repair(XmlRepair::AttributeRepaired);
                continue;
            };
            if name != raw_name {
                self.repair(XmlRepair::AttributeRepaired);
            }
            // The first value wins.
            if !names.insert(name.clone()) {
                self.repair(XmlRepair::AttributeRepaired);
                continue;
            }
            if found.len() >= self.limits.max_xml_attributes {
                self.repair(XmlRepair::LimitStopped);
                continue;
            }
            let (mut literal, mut mark) = (false, false);
            let mut value = String::with_capacity(raw_value.len());
            escape_loose(&raw_value, true, &mut value, &mut literal, &mut mark);
            if literal {
                self.repairs.insert(XmlRepair::ReferenceLiteral);
            }
            if mark {
                self.repair(XmlRepair::TextRepaired);
            }
            found.push(LooseAttribute { name, value });
        }
    }

    fn standard_namespace(&self, prefix: &str) -> Option<&'static str> {
        let table = if self.strict_namespaces {
            &STRICT_PREFIXES
        } else {
            &TRANSITIONAL_PREFIXES
        };
        table
            .iter()
            .find(|(p, _)| *p == prefix)
            .map(|(_, uri)| *uri)
    }

    /// Open the namespace scope this tag declares, and write it, keeping `walk`'s rules:
    /// a prefix nothing binds is bound, and a repeat by expanded name is left out.
    fn write_start(&mut self, name: &str, attributes: Vec<LooseAttribute>, self_closed: bool) {
        // The root's namespace says which form of the standard the part is in.
        if self.frames.len() == 1
            && let Some(a) = attributes.iter().find(|a| {
                a.name == format!("xmlns:{}", name.split(':').next().unwrap_or_default())
                    || a.name == "xmlns"
            })
        {
            self.strict_namespaces = a.value.contains("purl.oclc.org");
        }
        let mut declared: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
        let mut kept: Vec<LooseAttribute> = Vec::with_capacity(attributes.len());
        for a in attributes {
            let declares = match a.name.strip_prefix("xmlns") {
                Some("") => Some(""),
                Some(rest) => rest.strip_prefix(':'),
                None => None,
            };
            if let Some(prefix) = declares {
                let uri = attribute_value(a.value.as_bytes())
                    .map(Cow::into_owned)
                    .unwrap_or_default();
                if declaration_refused(prefix.as_bytes(), &uri) {
                    self.repair(XmlRepair::AttributeRepaired);
                    continue;
                }
                declared.push((prefix.as_bytes().to_vec(), uri));
            }
            kept.push(a);
        }
        self.scope.bind(declared);

        // What this tag needs bound: its own prefix, and each attribute's.
        let mut extra: Vec<(String, &'static str)> = Vec::new();
        let mut need = |this: &mut Self, prefix: &str, is_damage: bool| {
            if prefix.is_empty()
                || prefix == "xml"
                || this.scope.resolve(prefix.as_bytes()).is_some()
                || extra.iter().any(|(p, _)| p == prefix)
            {
                return;
            }
            let (uri, damage) = match this.standard_namespace(prefix) {
                Some(uri) => (uri, false),
                None => (UNDECLARED_NAMESPACE, true),
            };
            if damage || is_damage {
                this.repair(XmlRepair::AttributeRepaired);
            } else {
                this.repairs.insert(XmlRepair::AttributeRepaired);
            }
            extra.push((prefix.to_owned(), uri));
        };
        let element_prefix = name.split_once(':').map_or("", |(p, _)| p).to_owned();
        need(self, &element_prefix, false);
        for a in &kept {
            if let Some((prefix, _)) = a.name.split_once(':')
                && prefix != "xmlns"
            {
                need(self, prefix, false);
            }
        }
        self.scope.bind(
            extra
                .iter()
                .map(|(p, u)| (p.as_bytes().to_vec(), u.as_bytes().to_vec()))
                .collect(),
        );

        // Unique by expanded name.
        let mut seen: HashSet<(Vec<u8>, String)> = HashSet::new();
        let mut unique: Vec<LooseAttribute> = Vec::with_capacity(kept.len());
        for a in kept {
            let (uri, local): (Vec<u8>, String) = match a.name.split_once(':') {
                Some(("xmlns", l)) => (XMLNS_NS.to_vec(), l.to_owned()),
                None if a.name == "xmlns" => (XMLNS_NS.to_vec(), String::new()),
                None => (Vec::new(), a.name.clone()),
                Some(("xml", l)) => (XML_NS.to_vec(), l.to_owned()),
                Some((p, l)) => (
                    self.scope
                        .resolve(p.as_bytes())
                        .map(<[u8]>::to_vec)
                        .unwrap_or_default(),
                    l.to_owned(),
                ),
            };
            if seen.insert((uri, local)) {
                unique.push(a);
            } else {
                self.repair(XmlRepair::AttributeRepaired);
            }
        }

        // What was mended on the root, `cSld` or `spTree` cannot be left out with them,
        // and cannot be trusted either: only the declarations stay.
        let dirty_here = self.frames.last().is_some_and(|f| f.dirty);
        let keep_only_declarations =
            self.prune == Prune::SlideLike && self.frames.len() <= 3 && dirty_here;

        let max = self.limits.max_xml_attributes;
        let mut written = 0usize;
        self.out.push('<');
        self.out.push_str(name);
        for (prefix, uri) in &extra {
            if written >= max {
                break;
            }
            self.out.push_str(" xmlns:");
            self.out.push_str(prefix);
            self.out.push_str("=\"");
            self.out.push_str(uri);
            self.out.push('"');
            written += 1;
        }
        for a in unique {
            if keep_only_declarations && !a.name.starts_with("xmlns") {
                continue;
            }
            if written >= max {
                self.repair(XmlRepair::LimitStopped);
                break;
            }
            self.out.push(' ');
            self.out.push_str(&a.name);
            self.out.push_str("=\"");
            self.out.push_str(&a.value);
            self.out.push('"');
            written += 1;
        }
        self.out.push_str(if self_closed { "/>" } else { ">" });
    }
}

fn standalone_of(declaration: &str) -> Option<&'static str> {
    let at = declaration.find("standalone")?;
    let rest = declaration.get(at + "standalone".len()..)?;
    if rest.contains("yes") {
        Some("yes")
    } else if rest.contains("no") {
        Some("no")
    } else {
        None
    }
}

/// The best reading of `bytes`. Never fails: a part with no root element comes back empty.
///
/// The output is what [`walk`](crate::walk) admits under the same `limits`, in UTF-8. A
/// reading that would not fit the part size is given up, since nothing could then read it.
pub fn recover_xml(bytes: &[u8], limits: &OoxmlLimits, prune: Prune) -> RecoveredXml {
    let mut repairs = BTreeSet::new();
    if bytes.len() > limits.max_xml_part_bytes {
        repairs.insert(XmlRepair::LimitStopped);
        return RecoveredXml {
            bytes: Vec::new(),
            repairs,
            root_local: None,
            left_out: 0,
        };
    }
    let text = decode_any(bytes, &mut repairs);
    let text = clean_characters(text, &mut repairs);
    let mut reader = Reader {
        text: &text,
        at: 0,
        out: String::with_capacity(text.len().min(1 << 20)),
        frames: Vec::new(),
        scope: Scope::default(),
        repairs,
        limits,
        prune,
        elements: 0,
        root_local: None,
        root_closed: false,
        strict_namespaces: false,
        left_out: 0,
        stopped: false,
        no_comment_end: false,
        no_instruction_end: false,
        doctype_unclosed: false,
    };
    reader.run();
    let Reader {
        out,
        repairs,
        root_local,
        left_out,
        ..
    } = reader;
    let mut repairs = repairs;
    if root_local.is_none() || out.len() > limits.max_xml_part_bytes {
        repairs.insert(XmlRepair::LimitStopped);
        return RecoveredXml {
            bytes: Vec::new(),
            repairs,
            root_local: None,
            left_out,
        };
    }
    RecoveredXml {
        bytes: out.into_bytes(),
        repairs,
        root_local,
        left_out,
    }
}

#[cfg(test)]
mod tests;
