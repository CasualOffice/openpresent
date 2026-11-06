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
        if rest.len() & 1 != 0 {
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

/// One node of a [`XmlTree`], holding its source text exactly.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Node {
    Element {
        /// Everything between `<` and `>` or `/>`.
        start: String,
        /// `None` for a self-closed element; else what is between `</` and `>`.
        end: Option<String>,
        children: Vec<usize>,
    },
    /// Text, comments, declarations and the like, with their delimiters.
    Verbatim(String),
    /// Character data, still escaped, split from references as the source has it.
    Text(String),
}

/// A part as a tree that writes back the bytes it read.
///
/// This is how the fidelity contract holds inside an edited part (ADR-013): an
/// edit changes the nodes it names, so attribute order, prefixes, namespace
/// declarations, whitespace and everything unmodelled stay as they were.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlTree {
    nodes: Vec<Node>,
    /// Each node's parent, by the same index as `nodes`.
    parents: Vec<Option<usize>>,
    top: Vec<usize>,
    root: usize,
    bom: Bom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Bom {
    None,
    Utf8,
    Utf16Le,
    Utf16Be,
}

/// A handle on an element of an [`XmlTree`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ElementId(usize);

impl XmlTree {
    /// Parse under the same refusals and bounds as [`walk`], which runs first.
    pub fn parse(bytes: &[u8], part: &str, limits: &OoxmlLimits) -> Result<Self, OoxmlError> {
        walk(bytes, part, limits, |_, _| Ok(()))?;
        let bom = match bytes {
            [0xEF, 0xBB, 0xBF, ..] => Bom::Utf8,
            [0xFF, 0xFE, ..] => Bom::Utf16Le,
            [0xFE, 0xFF, ..] => Bom::Utf16Be,
            _ => Bom::None,
        };
        let xml = decode(bytes, part)?;
        let mut reader = Reader::from_reader(xml.as_ref());
        // `walk` has checked nesting. Keep `</a >` as written, which needs both.
        let config = reader.config_mut();
        config.trim_markup_names_in_closing_tags = false;
        config.check_end_names = false;
        let mut buf = Vec::new();
        let mut tree = Self {
            nodes: Vec::new(),
            parents: Vec::new(),
            top: Vec::new(),
            root: 0,
            bom,
        };
        let mut stack: Vec<usize> = Vec::new();
        let text = |b: &[u8]| {
            std::str::from_utf8(b)
                .map(str::to_owned)
                .map_err(|_| malformed(part, "not valid UTF-8"))
        };
        loop {
            let event = reader
                .read_event_into(&mut buf)
                .map_err(|err| malformed(part, err.to_string()))?;
            let node = match &event {
                Event::Eof => break,
                Event::Start(e) => Node::Element {
                    start: text(e)?,
                    end: Some(String::new()),
                    children: Vec::new(),
                },
                Event::Empty(e) => Node::Element {
                    start: text(e)?,
                    end: None,
                    children: Vec::new(),
                },
                Event::End(e) => {
                    let open = stack
                        .pop()
                        .ok_or_else(|| malformed(part, "stray end tag"))?;
                    if let Node::Element { end, .. } = &mut tree.nodes[open] {
                        *end = Some(text(e)?);
                    }
                    buf.clear();
                    continue;
                }
                Event::Text(e) => Node::Text(text(e)?),
                Event::GeneralRef(e) => Node::Text(format!("&{};", text(e)?)),
                Event::CData(e) => Node::Verbatim(format!("<![CDATA[{}]]>", text(e)?)),
                Event::Comment(e) => Node::Verbatim(format!("<!--{}-->", text(e)?)),
                Event::Decl(e) => Node::Verbatim(format!("<?{}?>", text(e)?)),
                Event::PI(e) => Node::Verbatim(format!("<?{}?>", text(e)?)),
                Event::DocType(_) => {
                    return Err(OoxmlError::DoctypeRefused {
                        part: part.to_owned(),
                    });
                }
            };
            let id = tree.nodes.len();
            let is_open = matches!(event, Event::Start(_));
            let is_element = matches!(node, Node::Element { .. });
            tree.nodes.push(node);
            tree.parents.push(stack.last().copied());
            match stack.last() {
                Some(&parent) => {
                    if let Node::Element { children, .. } = &mut tree.nodes[parent] {
                        children.push(id);
                    }
                }
                None => {
                    tree.top.push(id);
                    if is_element {
                        tree.root = id;
                    }
                }
            }
            if is_open {
                stack.push(id);
            }
            buf.clear();
        }
        Ok(tree)
    }

    /// The bytes, identical to the input for an untouched tree.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = String::new();
        for &id in &self.top {
            self.write(id, &mut out);
        }
        let utf16 = |word: fn(u16) -> [u8; 2], mark: [u8; 2]| {
            let mut b = mark.to_vec();
            out.encode_utf16().for_each(|u| b.extend(word(u)));
            b
        };
        match self.bom {
            Bom::None => out.clone().into_bytes(),
            Bom::Utf8 => [&[0xEF, 0xBB, 0xBF], out.as_bytes()].concat(),
            Bom::Utf16Le => utf16(u16::to_le_bytes, [0xFF, 0xFE]),
            Bom::Utf16Be => utf16(u16::to_be_bytes, [0xFE, 0xFF]),
        }
    }

    fn write(&self, id: usize, out: &mut String) {
        match &self.nodes[id] {
            Node::Verbatim(s) | Node::Text(s) => out.push_str(s),
            Node::Element {
                start,
                end,
                children,
            } => {
                out.push('<');
                out.push_str(start);
                match end {
                    None => out.push_str("/>"),
                    Some(end) => {
                        out.push('>');
                        for &c in children {
                            self.write(c, out);
                        }
                        out.push_str("</");
                        out.push_str(end);
                        out.push('>');
                    }
                }
            }
        }
    }

    /// The root element.
    pub fn root(&self) -> ElementId {
        ElementId(self.root)
    }

    fn element(&self, id: ElementId) -> (&str, &[usize]) {
        match &self.nodes[id.0] {
            Node::Element {
                start, children, ..
            } => (start, children),
            _ => ("", &[]),
        }
    }

    /// An element's local name.
    pub fn local_name(&self, id: ElementId) -> &str {
        let name = self.qualified_name(id);
        name.rsplit(':').next().unwrap_or(name)
    }

    /// An element's name with its prefix, as written.
    pub fn qualified_name(&self, id: ElementId) -> &str {
        let (start, _) = self.element(id);
        start
            .split(|c: char| c.is_ascii_whitespace())
            .next()
            .unwrap_or("")
    }

    /// An element's child elements, in order.
    pub fn child_elements(&self, id: ElementId) -> Vec<ElementId> {
        self.element(id)
            .1
            .iter()
            .filter(|&&c| matches!(self.nodes[c], Node::Element { .. }))
            .map(|&c| ElementId(c))
            .collect()
    }

    /// The first child with this local name.
    pub fn child(&self, id: ElementId, local: &str) -> Option<ElementId> {
        self.child_elements(id)
            .into_iter()
            .find(|&c| self.local_name(c) == local)
    }

    /// An attribute's value by local name, unescaped.
    pub fn attribute(&self, id: ElementId, local: &str) -> Option<String> {
        let (start, _) = self.element(id);
        let name_len = self.qualified_name(id).len();
        let tag = BytesStart::from_content(start, name_len);
        tag.attributes().flatten().find_map(|a| {
            (a.key.local_name().as_ref() == local.as_bytes())
                .then(|| a.normalized_value(quick_xml::XmlVersion::Implicit1_0).ok())
                .flatten()
                .map(std::borrow::Cow::into_owned)
        })
    }

    /// An element's direct text, unescaped. `None` if it has child elements.
    pub fn text(&self, id: ElementId) -> Option<String> {
        let (_, children) = self.element(id);
        let mut raw = String::new();
        for &c in children {
            match &self.nodes[c] {
                Node::Text(s) => raw.push_str(s),
                Node::Verbatim(s) if s.starts_with("<![CDATA[") => {
                    let inner = &s[9..s.len() - 3];
                    raw.push_str(&escape(inner));
                }
                Node::Verbatim(_) => {}
                Node::Element { .. } => return None,
            }
        }
        quick_xml::escape::unescape(&raw)
            .ok()
            .map(std::borrow::Cow::into_owned)
    }

    /// Replace an element's content with `value`, escaped. Nothing else moves.
    pub fn set_text(&mut self, id: ElementId, value: &str) {
        let new = self.nodes.len();
        self.nodes.push(Node::Text(escape(value)));
        self.parents.push(Some(id.0));
        if let Node::Element {
            start,
            end,
            children,
        } = &mut self.nodes[id.0]
        {
            *children = vec![new];
            if end.is_none() {
                let name = start
                    .split(|c: char| c.is_ascii_whitespace())
                    .next()
                    .unwrap_or("");
                *end = Some(name.to_owned());
                let trimmed = start.trim_end().len();
                start.truncate(trimmed);
            }
        }
    }

    /// Append a new child element with text, after the last child, using `name`
    /// as written, prefix included.
    pub fn append_text_child(&mut self, parent: ElementId, name: &str, value: &str) -> ElementId {
        let text = self.nodes.len();
        self.nodes.push(Node::Text(escape(value)));
        let id = self.nodes.len();
        self.parents.push(Some(id));
        self.nodes.push(Node::Element {
            start: name.to_owned(),
            end: Some(name.to_owned()),
            children: vec![text],
        });
        self.parents.push(Some(parent.0));
        if let Node::Element {
            start,
            end,
            children,
        } = &mut self.nodes[parent.0]
        {
            if end.is_none() {
                let tag = start
                    .split(|c: char| c.is_ascii_whitespace())
                    .next()
                    .unwrap_or("");
                *end = Some(tag.to_owned());
            }
            children.push(id);
        }
        ElementId(id)
    }

    /// An element's parent element.
    pub fn parent(&self, id: ElementId) -> Option<ElementId> {
        self.parents.get(id.0).copied().flatten().map(ElementId)
    }

    /// The namespace URI an element's prefix is bound to, looking outward through
    /// its ancestors. `None` if nothing binds it.
    pub fn namespace_uri(&self, id: ElementId) -> Option<String> {
        let name = self.qualified_name(id);
        let key = match name.split_once(':') {
            Some((prefix, _)) => format!("xmlns:{prefix}"),
            None => "xmlns".to_owned(),
        };
        let mut at = Some(id);
        while let Some(el) = at {
            if let Some(value) = self.exact_attribute(el, &key) {
                return Some(value);
            }
            at = self.parent(el);
        }
        None
    }

    /// The prefix bound to `uri` at this element, unshadowed: `Some("")` for the
    /// default namespace, `None` if nothing here can name that namespace.
    pub fn prefix_for_namespace(&self, id: ElementId, uri: &str) -> Option<String> {
        let mut at = Some(id);
        let mut seen: Vec<String> = Vec::new();
        while let Some(el) = at {
            let (start, _) = self.element(el);
            for (key, _) in attribute_ranges(start, self.qualified_name(el).len())? {
                let Some(key) = start.get(key) else { continue };
                let prefix = match key.strip_prefix("xmlns") {
                    Some("") => "",
                    Some(rest) => match rest.strip_prefix(':') {
                        Some(p) => p,
                        None => continue,
                    },
                    None => continue,
                };
                if !seen.iter().any(|s| s == prefix) {
                    seen.push(prefix.to_owned());
                    if self.exact_attribute(el, key).as_deref() == Some(uri) {
                        return Some(prefix.to_owned());
                    }
                }
            }
            at = self.parent(el);
        }
        None
    }

    fn exact_attribute(&self, id: ElementId, qualified: &str) -> Option<String> {
        let (start, _) = self.element(id);
        let name_len = self.qualified_name(id).len();
        BytesStart::from_content(start, name_len)
            .attributes()
            .flatten()
            .find(|a| a.key.as_ref() == qualified.as_bytes())
            .and_then(|a| {
                a.normalized_value(quick_xml::XmlVersion::Implicit1_0)
                    .ok()
                    .map(std::borrow::Cow::into_owned)
            })
    }

    /// An attribute with no prefix, unescaped. Use this for the schema's own
    /// attributes: [`XmlTree::attribute`] matches by local name in any namespace,
    /// so a foreign `x:val` written first would be taken for `val`.
    pub fn plain_attribute(&self, id: ElementId, name: &str) -> Option<String> {
        self.exact_attribute(id, name)
    }

    /// Whether the start tag's attributes are well formed and unrepeated.
    /// [`walk`] counts attributes without reading them, so a part can hold a
    /// malformed one and still parse. Read nothing from an element that fails this.
    pub fn attributes_valid(&self, id: ElementId) -> bool {
        let (start, _) = self.element(id);
        let name_len = self.qualified_name(id).len();
        let tag = BytesStart::from_content(start, name_len);
        tag.attributes().all(|a| {
            a.is_ok_and(|a| {
                a.normalized_value(quick_xml::XmlVersion::Implicit1_0)
                    .is_ok()
            })
        }) && attribute_ranges(start, name_len).is_some()
    }

    /// Turn an element into a new one in the same place with the same handle,
    /// dropping its attributes and content.
    pub fn replace_element(&mut self, id: ElementId, name: &str, attributes: &[(&str, &str)]) {
        if let Node::Element {
            start,
            end,
            children,
        } = &mut self.nodes[id.0]
        {
            *start = new_start_tag(name, attributes);
            *end = None;
            children.clear();
        }
    }

    /// Add a new empty element under `parent`, before the child `before`, or last
    /// if that is `None` or not a child of `parent`.
    pub fn insert_element(
        &mut self,
        parent: ElementId,
        before: Option<ElementId>,
        name: &str,
        attributes: &[(&str, &str)],
    ) -> ElementId {
        let id = self.nodes.len();
        self.nodes.push(Node::Element {
            start: new_start_tag(name, attributes),
            end: None,
            children: Vec::new(),
        });
        self.parents.push(Some(parent.0));
        let parent_name = self.qualified_name(parent).to_owned();
        if let Node::Element {
            start,
            end,
            children,
        } = &mut self.nodes[parent.0]
        {
            if end.is_none() {
                *end = Some(parent_name);
                let trimmed = start.trim_end().len();
                start.truncate(trimmed);
            }
            let at = before.and_then(|b| children.iter().position(|&c| c == b.0));
            children.insert(at.unwrap_or(children.len()), id);
        }
        ElementId(id)
    }

    /// Remove a child element, leaving the text around it. Returns whether it was
    /// a child.
    pub fn remove_element(&mut self, parent: ElementId, child: ElementId) -> bool {
        if let Node::Element { children, .. } = &mut self.nodes[parent.0]
            && let Some(at) = children.iter().position(|&c| c == child.0)
        {
            children.remove(at);
            return true;
        }
        false
    }
}

/// The byte ranges of each attribute's name and of its value without quotes.
/// `None` if the start tag does not scan as attributes.
fn attribute_ranges(
    start: &str,
    name_len: usize,
) -> Option<Vec<(core::ops::Range<usize>, core::ops::Range<usize>)>> {
    let bytes = start.as_bytes();
    let mut at = name_len;
    let mut spans = Vec::new();
    let skip_space = |mut i: usize| {
        while bytes.get(i).is_some_and(u8::is_ascii_whitespace) {
            i += 1;
        }
        i
    };
    loop {
        at = skip_space(at);
        if at >= bytes.len() {
            return Some(spans);
        }
        let key_start = at;
        while bytes
            .get(at)
            .is_some_and(|b| *b != b'=' && !b.is_ascii_whitespace())
        {
            at += 1;
        }
        let key_end = at;
        at = skip_space(at);
        if bytes.get(at) != Some(&b'=') || key_end == key_start {
            return None;
        }
        at = skip_space(at + 1);
        let quote = *bytes.get(at)?;
        if quote != b'"' && quote != b'\'' {
            return None;
        }
        let value_start = at + 1;
        let len = bytes.get(value_start..)?.iter().position(|b| *b == quote)?;
        spans.push((key_start..key_end, value_start..value_start + len));
        at = value_start + len + 1;
    }
}

fn new_start_tag(name: &str, attributes: &[(&str, &str)]) -> String {
    let mut start = name.to_owned();
    for (key, value) in attributes {
        start.push(' ');
        start.push_str(key);
        start.push_str("=\"");
        start.push_str(&escape_attribute(value));
        start.push('"');
    }
    start
}

/// Where one attribute sits in an element's source text.
struct AttributeSpan {
    /// From the whitespace before the name to the closing quote.
    whole: std::ops::Range<usize>,
    name: std::ops::Range<usize>,
    /// Between the quotes.
    value: std::ops::Range<usize>,
}

/// Locate an element's attributes. `None` if its text is not well formed.
fn attribute_spans(start: &str, name_len: usize) -> Option<Vec<AttributeSpan>> {
    let b = start.as_bytes();
    let mut spans = Vec::new();
    let mut i = name_len;
    loop {
        let before = i;
        while b.get(i).is_some_and(u8::is_ascii_whitespace) {
            i += 1;
        }
        if i >= b.len() {
            return Some(spans);
        }
        let name_start = i;
        while b
            .get(i)
            .is_some_and(|c| *c != b'=' && !c.is_ascii_whitespace())
        {
            i += 1;
        }
        let name = name_start..i;
        while b.get(i).is_some_and(u8::is_ascii_whitespace) {
            i += 1;
        }
        if b.get(i) != Some(&b'=') {
            return None;
        }
        i += 1;
        while b.get(i).is_some_and(u8::is_ascii_whitespace) {
            i += 1;
        }
        let quote = *b.get(i).filter(|c| matches!(**c, b'"' | b'\''))?;
        i += 1;
        let value_start = i;
        i += b[i..].iter().position(|c| *c == quote)?;
        spans.push(AttributeSpan {
            whole: before..i + 1,
            name,
            value: value_start..i,
        });
        i += 1;
    }
}

/// Reading attributes and mixed content, and the edits the model's export needs.
/// Each changes only the nodes it names.
impl XmlTree {
    /// Every attribute as `(name as written, value)`, in order and unescaped.
    /// Unlike [`XmlTree::attribute`] this tells `r:id` from `id`, and refuses a
    /// malformed or repeated attribute.
    pub fn attributes(
        &self,
        id: ElementId,
        part: &str,
    ) -> Result<Vec<(String, String)>, OoxmlError> {
        let (start, _) = self.element(id);
        let name_len = self.qualified_name(id).len();
        // The reader below takes `a="1"b="2"`, which XML does not, and which an
        // edit that removed `a` would fuse into the element's name.
        let spaced = attribute_spans(start, name_len)
            .is_some_and(|spans| spans.iter().all(|s| s.whole.start < s.name.start));
        if !spaced && !start[name_len..].trim().is_empty() {
            return Err(malformed(part, "attributes not separated by whitespace"));
        }
        let tag = BytesStart::from_content(start, name_len);
        let mut out = Vec::new();
        for a in tag.attributes() {
            let a = a.map_err(|err| malformed(part, err.to_string()))?;
            let key = std::str::from_utf8(a.key.as_ref())
                .map_err(|_| malformed(part, "not valid UTF-8"))?
                .to_owned();
            // A prefix holding a colon or a control character cannot be resolved,
            // and a new element named with it would not read back.
            let pieces = key.split(':').count();
            if pieces > 2
                || key
                    .split(':')
                    .any(|p| p.is_empty() || p.contains(char::is_control))
            {
                return Err(malformed(part, "attribute name is not a qualified name"));
            }
            let value = a
                .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                .map_err(|err| malformed(part, err.to_string()))?;
            out.push((key, value.into_owned()));
        }
        Ok(out)
    }

    /// An element's own text, unescaped and with line ends normalized as XML
    /// requires, skipping any child elements. `None` if a reference is invalid.
    pub fn own_text(&self, id: ElementId) -> Option<String> {
        let (_, children) = self.element(id);
        let mut raw = String::new();
        for &c in children {
            match &self.nodes[c] {
                Node::Text(s) => raw.push_str(&s.replace("\r\n", "\n").replace('\r', "\n")),
                Node::Verbatim(s) if s.starts_with("<![CDATA[") => {
                    let inner = s[9..s.len() - 3].replace("\r\n", "\n").replace('\r', "\n");
                    raw.push_str(&escape(&inner));
                }
                _ => {}
            }
        }
        quick_xml::escape::unescape(&raw)
            .ok()
            .map(std::borrow::Cow::into_owned)
    }

    /// Set an attribute, keeping its place and quote style, or add it after the
    /// last one. `name` is as written, prefix included. `false`, with nothing
    /// changed, if the element's text is not well formed.
    #[must_use]
    pub fn set_attribute(&mut self, id: ElementId, name: &str, value: &str) -> bool {
        let name_len = self.qualified_name(id).len();
        let Node::Element { start, .. } = &mut self.nodes[id.0] else {
            return false;
        };
        let Some(spans) = attribute_spans(start, name_len) else {
            return false;
        };
        let escaped = escape_attribute(value);
        match spans.iter().find(|s| &start[s.name.clone()] == name) {
            Some(span) => start.replace_range(span.value.clone(), &escaped),
            None => {
                let at = spans.last().map_or(name_len, |s| s.whole.end);
                start.insert_str(at, &format!(" {name}=\"{escaped}\""));
            }
        }
        true
    }

    /// Remove an attribute, by its name as written, and the whitespace before it.
    /// `false` if it was not there.
    pub fn remove_attribute(&mut self, id: ElementId, name: &str) -> bool {
        let name_len = self.qualified_name(id).len();
        let Node::Element { start, .. } = &mut self.nodes[id.0] else {
            return false;
        };
        let Some(spans) = attribute_spans(start, name_len) else {
            return false;
        };
        match spans.iter().find(|s| &start[s.name.clone()] == name) {
            Some(span) => {
                start.replace_range(span.whole.clone(), "");
                true
            }
            None => false,
        }
    }

    /// A new self-closed element, not yet in the tree. `name` is as written.
    pub fn new_element(&mut self, name: &str) -> ElementId {
        let id = self.nodes.len();
        self.nodes.push(Node::Element {
            start: name.to_owned(),
            end: None,
            children: Vec::new(),
        });
        ElementId(id)
    }

    /// Put `child` before `before`, or if `None`, just after the last child
    /// element, so a closing tag's indentation stays last.
    pub fn insert_child(&mut self, parent: ElementId, before: Option<ElementId>, child: ElementId) {
        self.open(parent);
        let at = self.insertion_point(parent, before);
        if let Node::Element { children, .. } = &mut self.nodes[parent.0] {
            children.insert(at, child.0);
        }
    }

    fn insertion_point(&self, parent: ElementId, before: Option<ElementId>) -> usize {
        let (_, children) = self.element(parent);
        if let Some(i) = before.and_then(|b| children.iter().position(|&c| c == b.0)) {
            return i;
        }
        children
            .iter()
            .rposition(|&c| matches!(self.nodes[c], Node::Element { .. }))
            .map_or(children.len(), |i| i + 1)
    }

    /// Turn a self-closed element into one that can hold content.
    fn open(&mut self, id: ElementId) {
        let name = self.qualified_name(id).to_owned();
        if let Node::Element { start, end, .. } = &mut self.nodes[id.0]
            && end.is_none()
        {
            let trimmed = start.trim_end().len();
            start.truncate(trimmed);
            *end = Some(name);
        }
    }

    /// Take `child` out of `parent`. `false` if it was not a child.
    pub fn remove_child(&mut self, parent: ElementId, child: ElementId) -> bool {
        if let Node::Element { children, .. } = &mut self.nodes[parent.0] {
            let before = children.len();
            children.retain(|&c| c != child.0);
            return children.len() != before;
        }
        false
    }

    /// Replace the run of elements `old` with `new`, slot by slot, so anything
    /// between them stays where it is. Spare slots close up; extra elements follow
    /// the last slot, or the last child element when there were no slots.
    pub fn replace_children(&mut self, parent: ElementId, old: &[ElementId], new: &[ElementId]) {
        self.open(parent);
        let fallback = self.insertion_point(parent, None);
        let Node::Element { children, .. } = &mut self.nodes[parent.0] else {
            return;
        };
        let slots: std::collections::HashSet<usize> = old.iter().map(|e| e.0).collect();
        let mut incoming = new.iter().map(|e| e.0);
        let mut out = Vec::with_capacity(children.len() + new.len());
        let mut at = None;
        for (i, &c) in children.iter().enumerate() {
            if i == fallback && slots.is_empty() {
                at = Some(out.len());
            }
            if slots.contains(&c) {
                if let Some(n) = incoming.next() {
                    out.push(n);
                }
                at = Some(out.len());
            } else {
                out.push(c);
            }
        }
        let at = at.unwrap_or(out.len());
        let rest: Vec<usize> = incoming.collect();
        out.splice(at..at, rest);
        *children = out;
    }

    /// Remove everything inside an element, keeping its tags.
    pub fn clear_content(&mut self, id: ElementId) {
        self.open(id);
        if let Node::Element { children, .. } = &mut self.nodes[id.0] {
            children.clear();
        }
    }
}

fn escape(s: &str) -> String {
    // A literal carriage return would come back as a line feed.
    quick_xml::escape::escape(s).replace('\r', "&#13;")
}

/// Escape for an attribute. Tab and line breaks go as references, since a literal
/// one would come back as a space after attribute normalization.
fn escape_attribute(s: &str) -> String {
    escape(s)
        .replace('\t', "&#9;")
        .replace('\n', "&#10;")
        .replace('\r', "&#13;")
}

#[cfg(test)]
mod tree_tests {
    use super::*;

    fn parse(s: &str) -> XmlTree {
        XmlTree::parse(s.as_bytes(), "t", &OoxmlLimits::default()).unwrap()
    }

    #[test]
    fn writes_back_exactly_what_it_read() {
        for s in [
            "<?xml version=\"1.0\"  encoding='UTF-8'?>\n<a:x  b = \"1\"\tc='2' xmlns:a=\"u\" >\n <!-- k --><y /><z>t&amp;u&#65;<![CDATA[<r>]]></z><?pi d?>\n</a:x >\n",
            "<a/>",
            "<a b=\"&lt;\"></a>",
        ] {
            assert_eq!(parse(s).to_bytes(), s.as_bytes(), "{s}");
        }
    }

    #[test]
    fn utf16_writes_back_as_utf16() {
        let mut bytes = vec![0xFF, 0xFE];
        "<a>é</a>"
            .encode_utf16()
            .for_each(|u| bytes.extend(u.to_le_bytes()));
        let tree = XmlTree::parse(&bytes, "t", &OoxmlLimits::default()).unwrap();
        assert_eq!(tree.to_bytes(), bytes);
        assert_eq!(tree.text(tree.root()).unwrap(), "é");
    }

    #[test]
    fn an_edit_changes_only_what_it_names() {
        let mut t = parse("<r  z=\"1\"><!-- c --><p:a q='2'>old</p:a> <b/></r>");
        let a = t.child(t.root(), "a").unwrap();
        t.set_text(a, "new & <x>");
        assert_eq!(
            t.to_bytes(),
            b"<r  z=\"1\"><!-- c --><p:a q='2'>new &amp; &lt;x&gt;</p:a> <b/></r>"
        );
        let b = t.child(t.root(), "b").unwrap();
        t.set_text(b, "v");
        assert_eq!(t.text(b).unwrap(), "v");
        assert!(t.to_bytes().ends_with(b"<b>v</b></r>"));
    }

    #[test]
    fn reads_names_attributes_and_text() {
        let t = parse("<p:r x:k=\"a&amp;b\"><c>1&lt;2<![CDATA[&]]></c><d><e/></d></p:r>");
        assert_eq!(t.local_name(t.root()), "r");
        assert_eq!(t.qualified_name(t.root()), "p:r");
        assert_eq!(t.attribute(t.root(), "k").unwrap(), "a&b");
        assert_eq!(t.text(t.child(t.root(), "c").unwrap()).unwrap(), "1<2&");
        assert_eq!(t.text(t.child(t.root(), "d").unwrap()), None);
    }

    #[test]
    fn set_attribute_patches_only_the_value() {
        let mut t = parse("<r  a = 'x'\tb=\"y\" ><k/></r >");
        assert!(t.set_attribute(t.root(), "b", "1<2\t"));
        assert_eq!(t.to_bytes(), b"<r  a = 'x'\tb=\"1&lt;2&#9;\" ><k/></r >");
        assert_eq!(t.plain_attribute(t.root(), "b").unwrap(), "1<2\t");
        assert!(t.set_attribute(t.root(), "a", "it's"));
        assert_eq!(t.plain_attribute(t.root(), "a").unwrap(), "it's");
        assert!(t.set_attribute(t.root(), "c", "z"));
        assert!(t.to_bytes().starts_with(b"<r  a = 'it&apos;s'\tb="));
        assert_eq!(t.plain_attribute(t.root(), "c").unwrap(), "z");
    }

    #[test]
    fn set_attribute_on_a_self_closed_tag_keeps_its_space() {
        let mut t = parse("<r><k a=\"1\" /></r>");
        let k = t.child(t.root(), "k").unwrap();
        assert!(t.set_attribute(k, "b", "2"));
        assert_eq!(t.to_bytes(), b"<r><k a=\"1\" b=\"2\" /></r>");
    }

    #[test]
    fn remove_attribute_leaves_the_rest() {
        let mut t = parse("<r a=\"1\"  b='2' c=\"3\"/>");
        assert!(t.remove_attribute(t.root(), "b"));
        assert_eq!(t.to_bytes(), b"<r a=\"1\" c=\"3\"/>");
        assert!(!t.remove_attribute(t.root(), "b"));
    }

    /// A value holding the other quote must not move the span.
    #[test]
    fn the_scanner_is_not_fooled_by_values() {
        let mut t = parse("<r a=\"it's b='9'\" b='two \"q\"'/>");
        assert!(t.set_attribute(t.root(), "b", "n"));
        assert_eq!(t.to_bytes(), b"<r a=\"it's b='9'\" b='n'/>");
    }

    #[test]
    fn a_foreign_attribute_does_not_shadow_the_plain_one() {
        let t = parse("<r xmlns:x=\"u\" x:val=\"bad\" val=\"good\"/>");
        assert_eq!(t.attribute(t.root(), "val").unwrap(), "bad");
        assert_eq!(t.plain_attribute(t.root(), "val").unwrap(), "good");
    }

    #[test]
    fn bad_attributes_are_seen_and_block_edits() {
        for s in [
            "<r a/>",
            "<r a=1/>",
            "<r a=\"1\" a=\"2\"/>",
            "<r a=\"&nope;\"/>",
        ] {
            let t = parse(s);
            assert!(!t.attributes_valid(t.root()), "{s}");
        }
        let mut t = parse("<r a/>");
        assert!(!t.set_attribute(t.root(), "a", "1"));
        assert_eq!(t.to_bytes(), b"<r a/>");
        let good = parse("<r a=\"1\" b='2'/>");
        assert!(good.attributes_valid(good.root()));
    }

    #[test]
    fn namespaces_resolve_outward_and_shadow() {
        let t = parse("<a:r xmlns:a=\"one\" xmlns=\"dflt\"><b/><a:c xmlns:a=\"two\"/><z:d/></a:r>");
        let kids = t.child_elements(t.root());
        assert_eq!(t.namespace_uri(t.root()).unwrap(), "one");
        assert_eq!(t.namespace_uri(kids[0]).unwrap(), "dflt");
        assert_eq!(t.namespace_uri(kids[1]).unwrap(), "two");
        assert_eq!(t.namespace_uri(kids[2]), None);
        assert_eq!(t.parent(kids[0]), Some(t.root()));
        assert_eq!(t.parent(t.root()), None);
        assert_eq!(t.prefix_for_namespace(kids[0], "one").unwrap(), "a");
        assert_eq!(t.prefix_for_namespace(kids[1], "one"), None, "shadowed");
        assert_eq!(t.prefix_for_namespace(kids[1], "two").unwrap(), "a");
        assert_eq!(t.prefix_for_namespace(kids[0], "dflt").unwrap(), "");
    }

    #[test]
    fn structural_edits() {
        let mut t = parse("<r><a x=\"1\"><i/></a> <b/></r>");
        let a = t.child(t.root(), "a").unwrap();
        let b = t.child(t.root(), "b").unwrap();
        t.replace_element(a, "n:z", &[("v", "<")]);
        assert_eq!(t.to_bytes(), b"<r><n:z v=\"&lt;\"/> <b/></r>");
        let c = t.insert_element(a, None, "c", &[]);
        assert_eq!(t.parent(c), Some(a));
        assert_eq!(t.to_bytes(), b"<r><n:z v=\"&lt;\"><c/></n:z> <b/></r>");
        t.insert_element(t.root(), Some(b), "m", &[]);
        assert_eq!(t.to_bytes(), b"<r><n:z v=\"&lt;\"><c/></n:z> <m/><b/></r>");
        assert!(t.remove_element(t.root(), b));
        assert!(!t.remove_element(t.root(), b));
        assert_eq!(t.to_bytes(), b"<r><n:z v=\"&lt;\"><c/></n:z> <m/></r>");
    }

    #[test]
    fn attributes_tell_prefixed_from_plain_and_refuse_repeats() {
        let t = parse("<a r:id=\"rel\" id=\"7\" k='&lt;'/>");
        let attrs = t.attributes(t.root(), "t").unwrap();
        assert_eq!(attrs[0], ("r:id".to_owned(), "rel".to_owned()));
        assert_eq!(attrs[1], ("id".to_owned(), "7".to_owned()));
        assert_eq!(attrs[2].1, "<");
        for bad in [
            "<a b=\"1\" b=\"2\"/>",
            "<a b=1/>",
            "<a b/>",
            "<a b=\"&nope;\"/>",
            "<a b=\"1\"c=\"2\"/>",
            "<a x:y:z=\"1\"/>",
            "<a :y=\"1\"/>",
            "<a xmlns:=\"u\"/>",
            "<a b\u{0}=\"1\"/>",
        ] {
            let t = parse(bad);
            let err = t.attributes(t.root(), "t").unwrap_err();
            assert_eq!(err.code(), "CS-XML-0004", "{bad}");
        }
    }

    #[test]
    fn attribute_edits_touch_only_that_attribute() {
        let mut t = parse("<a  x='1'\n\ty = \"2\" z=\"3\" />");
        let r = t.root();
        assert!(t.set_attribute(r, "y", "a'\"<\t\n\r"));
        assert_eq!(
            t.to_bytes(),
            b"<a  x='1'\n\ty = \"a&apos;&quot;&lt;&#9;&#10;&#13;\" z=\"3\" />"
        );
        assert_eq!(t.attributes(r, "t").unwrap()[1].1, "a'\"<\t\n\r");
        assert!(t.set_attribute(r, "n", "v"));
        assert!(t.remove_attribute(r, "x"));
        assert!(!t.remove_attribute(r, "x"));
        assert!(t.to_bytes().ends_with(b"z=\"3\" n=\"v\" />"));
        let mut bad = parse("<a b=1/>");
        let r = bad.root();
        assert!(!bad.set_attribute(r, "c", "v"));
        assert_eq!(bad.to_bytes(), b"<a b=1/>");
    }

    #[test]
    fn inserting_opens_a_self_closed_parent_and_keeps_order() {
        let mut t = parse("<r><a/>\n <c />\n</r>");
        let r = t.root();
        let c = t.child(r, "c").unwrap();
        let b = t.new_element("p:b");
        t.insert_child(r, Some(c), b);
        let d = t.new_element("d");
        t.insert_child(r, None, d);
        let e = t.new_element("e");
        t.insert_child(c, None, e);
        assert_eq!(t.to_bytes(), b"<r><a/>\n <p:b/><c><e/></c><d/>\n</r>");
        assert!(t.remove_child(r, b));
        assert!(!t.remove_child(r, b));
        t.clear_content(c);
        assert_eq!(t.to_bytes(), b"<r><a/>\n <c></c><d/>\n</r>");
    }

    #[test]
    fn replacing_children_keeps_what_sits_between_them() {
        let mut t = parse("<r> <a/> <!--k--> <b/> <x/> <c/> </r>");
        let r = t.root();
        let (a, b, c) = (
            t.child(r, "a").unwrap(),
            t.child(r, "b").unwrap(),
            t.child(r, "c").unwrap(),
        );
        t.replace_children(r, &[a, b, c], &[c, a]);
        assert_eq!(t.to_bytes(), b"<r> <c/> <!--k--> <a/> <x/>  </r>");
        let n = t.new_element("n");
        t.replace_children(r, &[c, a], &[c, a, n]);
        assert_eq!(t.to_bytes(), b"<r> <c/> <!--k--> <a/><n/> <x/>  </r>");
        let mut empty = parse("<r>\n</r>");
        let r = empty.root();
        let n = empty.new_element("n");
        empty.replace_children(r, &[], &[n]);
        assert_eq!(empty.to_bytes(), b"<r>\n<n/></r>");
    }

    #[test]
    fn own_text_skips_children_and_normalizes_line_ends() {
        let t = parse("<a>x\r\ny<b>no</b>&#13;<![CDATA[<z>]]></a>");
        assert_eq!(t.own_text(t.root()).unwrap(), "x\ny\r<z>");
        let bad = parse("<a>&#xD800;</a>");
        assert_eq!(bad.own_text(bad.root()), None);
    }

    #[test]
    fn a_carriage_return_set_as_text_comes_back() {
        let mut t = parse("<a/>");
        let r = t.root();
        t.set_text(r, "x\ry");
        assert_eq!(t.to_bytes(), b"<a>x&#13;y</a>");
        let again = XmlTree::parse(&t.to_bytes(), "t", &OoxmlLimits::default()).unwrap();
        assert_eq!(again.own_text(again.root()).unwrap(), "x\ry");
    }

    #[test]
    fn the_tree_inherits_the_walkers_refusals() {
        let err = XmlTree::parse(b"<!DOCTYPE a><a/>", "t", &OoxmlLimits::default()).unwrap_err();
        assert_eq!(err.code(), "CS-XML-0005");
    }
}
