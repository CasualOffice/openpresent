//! The one way XML enters this engine.
//!
//! Every part goes through [`walk`]. Keeping a single path is the control: a
//! parser written beside it would not inherit the document-type refusal or the
//! bounds, and nothing would show that it had not.

use std::borrow::Cow;

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::error::OoxmlError;
use crate::limits::OoxmlLimits;

const PREDEFINED: [&[u8]; 5] = [b"lt", b"gt", b"amp", b"apos", b"quot"];

/// UTF-8 bytes for a part, which OPC allows in UTF-8 or UTF-16, marked by a BOM.
fn decode<'a>(bytes: &'a [u8], part: &str) -> Result<Cow<'a, [u8]>, OoxmlError> {
    let utf16 = |rest: &[u8], word: fn([u8; 2]) -> u16| -> Result<Cow<'a, [u8]>, OoxmlError> {
        if rest.len() % 2 != 0 {
            return Err(malformed(part, "truncated UTF-16"));
        }
        let units = rest.chunks_exact(2).map(|c| word([c[0], c[1]]));
        char::decode_utf16(units)
            .collect::<Result<String, _>>()
            .map(|s| Cow::Owned(s.into_bytes()))
            .map_err(|_| malformed(part, "invalid UTF-16"))
    };
    match bytes {
        [0xEF, 0xBB, 0xBF, rest @ ..] => Ok(Cow::Borrowed(rest)),
        [0xFF, 0xFE, rest @ ..] => utf16(rest, u16::from_le_bytes),
        [0xFE, 0xFF, rest @ ..] => utf16(rest, u16::from_be_bytes),
        _ => Ok(Cow::Borrowed(bytes)),
    }
}

fn malformed(part: &str, reason: impl Into<String>) -> OoxmlError {
    OoxmlError::MalformedXml {
        part: part.to_owned(),
        reason: reason.into(),
    }
}

/// Walk one XML part, calling `on_element` for each start or empty element with
/// its depth, the root being 1.
///
/// Refuses: any document type; a reference to an entity other than the five
/// predefined; anything over the part's size, element, depth or attribute
/// bounds; and a part that is not exactly one well-nested root element.
pub fn walk(
    bytes: &[u8],
    part: &str,
    limits: &OoxmlLimits,
    mut on_element: impl FnMut(&BytesStart<'_>, usize) -> Result<(), OoxmlError>,
) -> Result<(), OoxmlError> {
    if bytes.len() > limits.max_xml_part_bytes {
        return Err(OoxmlError::XmlTooLarge {
            part: part.to_owned(),
            limit: limits.max_xml_part_bytes,
        });
    }
    let xml = decode(bytes, part)?;
    let mut reader = Reader::from_reader(xml.as_ref());
    let mut buf = Vec::new();
    let mut elements = 0usize;
    let mut depth = 0usize;
    let mut roots = 0usize;

    loop {
        let event = reader
            .read_event_into(&mut buf)
            .map_err(|err| malformed(part, err.to_string()))?;
        let (start, is_empty) = match event {
            Event::Start(e) => (e, false),
            Event::Empty(e) => (e, true),
            Event::End(_) => {
                depth = depth.saturating_sub(1);
                buf.clear();
                continue;
            }
            Event::DocType(_) => {
                return Err(OoxmlError::DoctypeRefused {
                    part: part.to_owned(),
                });
            }
            Event::GeneralRef(r) => {
                if !r.is_char_ref() && !PREDEFINED.contains(&r.as_ref()) {
                    return Err(malformed(part, "reference to an undeclared entity"));
                }
                buf.clear();
                continue;
            }
            Event::Eof => break,
            _ => {
                buf.clear();
                continue;
            }
        };

        if depth == 0 {
            roots += 1;
            if roots > 1 {
                return Err(malformed(part, "more than one root element"));
            }
        }
        elements += 1;
        if elements > limits.max_xml_elements {
            return Err(OoxmlError::TooManyElements {
                part: part.to_owned(),
                limit: limits.max_xml_elements,
            });
        }
        let this_depth = depth + 1;
        if this_depth > limits.max_xml_depth {
            return Err(OoxmlError::TooDeep {
                part: part.to_owned(),
                limit: limits.max_xml_depth,
            });
        }
        if start.attributes().count() > limits.max_xml_attributes {
            return Err(OoxmlError::TooManyAttributes {
                part: part.to_owned(),
                limit: limits.max_xml_attributes,
            });
        }
        on_element(&start, this_depth)?;
        if !is_empty {
            depth = this_depth;
        }
        buf.clear();
    }
    if depth != 0 {
        return Err(malformed(part, "an element is not closed"));
    }
    if roots == 0 {
        return Err(malformed(part, "no root element"));
    }
    Ok(())
}

/// An attribute's value by local name, normalized as XML requires. Duplicate and
/// malformed attributes are refused.
pub(crate) fn attr(
    e: &BytesStart<'_>,
    local: &[u8],
    part: &str,
) -> Result<Option<String>, OoxmlError> {
    let mut found = None;
    for a in e.attributes() {
        let a = a.map_err(|err| malformed(part, err.to_string()))?;
        if a.key.local_name().as_ref() == local {
            let value = a
                .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                .map_err(|err| malformed(part, err.to_string()))?;
            found = Some(value.into_owned());
        }
    }
    Ok(found)
}

pub(crate) fn required(e: &BytesStart<'_>, local: &[u8], part: &str) -> Result<String, OoxmlError> {
    attr(e, local, part)?.ok_or_else(|| {
        malformed(
            part,
            format!("missing attribute {}", String::from_utf8_lossy(local)),
        )
    })
}

pub(crate) fn wrong_root(part: &str, expected: &str) -> OoxmlError {
    malformed(part, format!("root element is not {expected}"))
}
