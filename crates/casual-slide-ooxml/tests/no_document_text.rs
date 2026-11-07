//! `SEC-015`: no document text in any error message or finding.
//!
//! A marker is planted wherever a part can hold text of its own: element and
//! attribute names, values, text, comments, entity names, content types, targets,
//! namespaces. Then every code in `docs/20` is produced, and neither the message
//! nor the debug form of what comes back may hold the marker. Part names and
//! relationship ids are how a report says where, and may appear; the marker is
//! never put in one.

use std::collections::BTreeSet;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};

use casual_slide_ooxml::{OoxmlError, OoxmlLimits, PresentationPackage, XmlTree, walk};
use casual_slide_package::{PackageError, PackageLimits, PartEdits};
use zip::CompressionMethod;
use zip::write::{SimpleFileOptions, ZipWriter};

const MARK: &str = "QzMarkerQz";
const PML: &str = "http://schemas.openxmlformats.org/presentationml/2006/main";
const RELS: &str = "http://schemas.openxmlformats.org/package/2006/relationships";
const T: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn minimal() -> Vec<u8> {
    std::fs::read(fixtures().join("generated/minimal.pptx")).unwrap()
}

/// The minimal deck with parts replaced or added.
fn with(parts: &[(&str, String)]) -> Vec<u8> {
    let mut pkg = PresentationPackage::open(minimal(), OoxmlLimits::default()).unwrap();
    let mut edits = PartEdits::default();
    for (name, xml) in parts {
        edits.replace(*name, xml.clone().into_bytes());
    }
    pkg.save_with(&edits).unwrap()
}

/// A ZIP written as given, for what a well-behaved editor would not produce.
fn raw_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    for (name, data) in entries {
        writer.start_file(*name, opts).unwrap();
        writer.write_all(data).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

/// The main part, with `body` inside its root.
fn main_with(body: &str) -> Vec<u8> {
    with(&[(
        "ppt/presentation.xml",
        format!("<p:presentation xmlns:p=\"{PML}\">{body}</p:presentation>"),
    )])
}

fn main_is(xml: &str) -> Vec<u8> {
    with(&[("ppt/presentation.xml", xml.to_owned())])
}

fn rels(items: &[(&str, &str, Option<&str>)]) -> String {
    let mut out = format!("<Relationships xmlns=\"{RELS}\">");
    for (i, (typ, target, mode)) in items.iter().enumerate() {
        let typ = if typ.starts_with("http") {
            (*typ).to_owned()
        } else {
            format!("{T}/{typ}")
        };
        let mode = mode
            .map(|m| format!(" TargetMode=\"{m}\""))
            .unwrap_or_default();
        out.push_str(&format!(
            "<Relationship Id=\"rId{}\" Type=\"{typ}\" Target=\"{target}\"{mode}/>",
            i + 1
        ));
    }
    out + "</Relationships>"
}

fn slide_rels(items: &[(&str, &str, Option<&str>)]) -> Vec<u8> {
    let mut all = vec![("slideLayout", "../slideLayouts/slideLayout1.xml", None)];
    all.extend_from_slice(items);
    with(&[("ppt/slides/_rels/slide1.xml.rels", rels(&all))])
}

fn content_types(extra: &str) -> String {
    let mut out = String::from(
        "<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
         <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
         <Default Extension=\"xml\" ContentType=\"application/xml\"/>",
    );
    out.push_str(&format!(
        "<Override PartName=\"/ppt/presentation.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml\"/>{extra}</Types>"
    ));
    out
}

/// Case folded, since a reader may have lowered what it quotes.
fn leaks(text: &str) -> bool {
    text.to_ascii_lowercase()
        .contains(&MARK.to_ascii_lowercase())
}

struct Checked {
    codes: BTreeSet<String>,
    leaks: Vec<String>,
}

impl Checked {
    fn error(&mut self, label: &str, err: &OoxmlError) {
        self.codes.insert(err.code().to_owned());
        let text = format!("{err} {err:?}");
        if leaks(&text) {
            self.leaks.push(format!("{label}: {} {text}", err.code()));
        }
    }

    fn open(
        &mut self,
        label: &str,
        bytes: Vec<u8>,
        limits: OoxmlLimits,
    ) -> Option<PresentationPackage> {
        match PresentationPackage::open(bytes, limits) {
            Ok(pkg) => {
                for f in pkg.findings() {
                    self.codes.insert(f.kind.code().to_owned());
                    if leaks(&format!("{f:?}")) {
                        self.leaks.push(format!("{label}: {f:?}"));
                    }
                }
                Some(pkg)
            }
            Err(err) => {
                self.error(label, &err);
                None
            }
        }
    }

    fn refused(&mut self, label: &str, bytes: Vec<u8>) {
        assert!(
            self.open(label, bytes, OoxmlLimits::default()).is_none(),
            "{label}: opened"
        );
    }
}

/// Every code the registry lists for this layer.
fn registered() -> BTreeSet<String> {
    let registry =
        std::fs::read_to_string(fixtures().join("../docs/20-ERROR-CODE-REGISTRY.md")).unwrap();
    let mut codes = BTreeSet::new();
    for word in registry.split(|c: char| !(c.is_ascii_alphanumeric() || c == '-')) {
        if ["CS-PKG-", "CS-XML-", "CS-OPC-", "CS-MCE-"]
            .iter()
            .any(|p| word.starts_with(p))
        {
            codes.insert(word.to_owned());
        }
    }
    let findings = registry
        .lines()
        .find(|l| l.starts_with("Findings"))
        .unwrap();
    for word in findings.split(|c: char| !c.is_ascii_digit()) {
        if word.len() == 4 {
            codes.insert(format!("CS-FND-{word}"));
        }
    }
    codes
}

#[test]
fn no_message_or_finding_carries_document_text() {
    let mut c = Checked {
        codes: BTreeSet::new(),
        leaks: Vec::new(),
    };
    let lim = OoxmlLimits::default;
    let marked = || {
        main_with(&format!(
            "<p:{MARK} {MARK}=\"{MARK}\">{MARK}<!--{MARK}--></p:{MARK}>"
        ))
    };

    // --- The package -----------------------------------------------------------------
    let tiny = |f: fn(&mut PackageLimits)| {
        let mut l = OoxmlLimits::default();
        f(&mut l.package);
        l
    };
    c.open("input size", marked(), tiny(|p| p.max_input_bytes = 100));
    c.open("entries", marked(), tiny(|p| p.max_entries = 3));
    c.open(
        "expansion",
        marked(),
        tiny(|p| p.max_total_uncompressed = 1000),
    );
    c.refused("unsafe path", raw_zip(&[("../evil.xml", MARK.as_bytes())]));
    c.refused("not a package", MARK.repeat(40).into_bytes());
    c.refused(
        "duplicate",
        raw_zip(&[("a b.xml", MARK.as_bytes()), ("a%20b.xml", MARK.as_bytes())]),
    );
    if let Some(mut pkg) = c.open("missing part", marked(), lim()) {
        c.error(
            "missing part",
            &pkg.read_part("ppt/missing.xml").unwrap_err(),
        );
    }
    let mut raw = casual_slide_package::Package::open(marked(), PackageLimits::default()).unwrap();
    c.error(
        "part not found",
        &raw.read_part("ppt/missing.xml").unwrap_err().into(),
    );
    c.error("rewrite", &OoxmlError::from(PackageError::RewriteFailed));
    let small = OoxmlLimits {
        max_xml_part_bytes: 600,
        ..OoxmlLimits::default()
    };
    c.open(
        "declared size",
        main_with(&format!("<!--{}-->", MARK.repeat(100))),
        small,
    );

    // --- XML -------------------------------------------------------------------------
    let elements = OoxmlLimits {
        max_xml_elements: 20,
        ..OoxmlLimits::default()
    };
    c.open(
        "elements",
        main_with(&format!("<{MARK}/>").repeat(30)),
        elements,
    );
    c.refused(
        "depth",
        main_with(&format!(
            "{}{}",
            format!("<{MARK}>").repeat(300),
            format!("</{MARK}>").repeat(300)
        )),
    );
    let attributes: String = (0..1100)
        .map(|i| format!(" {MARK}{i}=\"{MARK}\""))
        .collect();
    c.refused(
        "attributes",
        main_is(&format!("<p:presentation xmlns:p=\"{PML}\"{attributes}/>")),
    );
    c.refused(
        "doctype",
        main_is(&format!(
            "<!DOCTYPE {MARK} [<!ENTITY {MARK} \"{MARK}\">]><p:presentation xmlns:p=\"{PML}\"/>"
        )),
    );
    let over = OoxmlLimits {
        max_xml_part_bytes: 10,
        ..OoxmlLimits::default()
    };
    let part = format!("<{MARK}>{MARK}</{MARK}>");
    c.error(
        "walker size",
        &walk(part.as_bytes(), "t", &over, |_, _| Ok(())).unwrap_err(),
    );
    c.error(
        "tree size",
        &XmlTree::parse(part.as_bytes(), "t", &over).unwrap_err(),
    );

    // Malformed, every way the walker or a reader on it can say so. Several are
    // quick-xml's own errors, whose messages quote the document.
    for (label, body) in [
        ("mismatched end", format!("<{MARK}></{MARK}x>")),
        ("unmatched end", format!("</{MARK}>")),
        ("unclosed", format!("<{MARK}>")),
        ("unclosed tag", format!("<{MARK} a=\"{MARK}\"")),
        ("bad syntax", format!("<!{MARK}>")),
        ("unclosed comment", format!("<!--{MARK}")),
        ("comment dashes", format!("<!-- {MARK} -- {MARK} -->")),
        ("undeclared entity", format!("&{MARK};")),
        ("unclosed reference", format!("&{MARK}")),
        ("bad char reference", format!("&#x{MARK};")),
        ("undeclared prefix", format!("<{MARK}:x/>")),
        (
            "undeclared attribute prefix",
            format!("<x {MARK}:a=\"{MARK}\"/>"),
        ),
        ("no value", format!("<x {MARK}/>")),
        ("unquoted", format!("<x a={MARK}/>")),
        (
            "repeated",
            format!("<x {MARK}=\"{MARK}\" {MARK}=\"{MARK}\"/>"),
        ),
        ("run together", format!("<x a=\"{MARK}\"{MARK}=\"1\"/>")),
        ("lt in value", format!("<x a=\"<{MARK}\"/>")),
        ("bad name", format!("<x {MARK}:{MARK}:{MARK}=\"1\"/>")),
        ("bad declaration", format!("<x xmlns:{MARK}=\"\"/>")),
        ("cdata end", format!("<x>]]>{MARK}</x>")),
        ("nul", format!("<x>{MARK}\u{0}</x>")),
    ] {
        c.refused(label, main_with(&body));
    }
    c.refused(
        "text outside the root",
        main_is(&format!("{MARK}<p:presentation xmlns:p=\"{PML}\"/>")),
    );
    c.refused(
        "wrong root",
        main_is(&format!("<{MARK} {MARK}=\"{MARK}\"/>")),
    );
    let mut latin1 = format!("<p:presentation xmlns:p=\"{PML}\">{MARK}").into_bytes();
    latin1.extend_from_slice(b"\xe9</p:presentation>");
    let mut pkg = PresentationPackage::open(minimal(), OoxmlLimits::default()).unwrap();
    let mut edits = PartEdits::default();
    edits.replace("ppt/presentation.xml", latin1);
    c.refused("not UTF-8", pkg.save_with(&edits).unwrap());
    c.refused("target mode", slide_rels(&[("image", MARK, Some(MARK))]));
    c.refused(
        "repeated id",
        with(&[("ppt/slides/_rels/slide1.xml.rels", format!(
            "<Relationships xmlns=\"{RELS}\"><Relationship Id=\"rId1\" Type=\"{MARK}\" Target=\"{MARK}\"/><Relationship Id=\"rId1\" Type=\"{MARK}\" Target=\"{MARK}\"/></Relationships>"
        ))]),
    );
    c.refused(
        "missing attribute",
        with(&[("ppt/slides/_rels/slide1.xml.rels", format!(
            "<Relationships xmlns=\"{RELS}\"><Relationship Id=\"rId1\" Type=\"{MARK}\"/></Relationships>"
        ))]),
    );
    c.refused(
        "two types",
        with(&[("[Content_Types].xml", content_types(&format!(
            "<Default Extension=\"{MARK}\" ContentType=\"{MARK}/a\"/><Default Extension=\"{MARK}\" ContentType=\"{MARK}/b\"/>"
        )))]),
    );

    // --- OPC -------------------------------------------------------------------------
    c.refused(
        "no content types",
        raw_zip(&[("_rels/.rels", MARK.as_bytes())]),
    );
    let per_part = OoxmlLimits {
        max_part_relationships: 3,
        ..OoxmlLimits::default()
    };
    let flood: Vec<(&str, &str, Option<&str>)> = vec![("hyperlink", MARK, Some("External")); 5];
    c.open("relationships", slide_rels(&flood), per_part);
    let types = OoxmlLimits {
        max_content_types: 3,
        ..OoxmlLimits::default()
    };
    let many: String = (0..5)
        .map(|i| format!("<Default Extension=\"{MARK}{i}\" ContentType=\"{MARK}\"/>"))
        .collect();
    c.open(
        "content types",
        with(&[("[Content_Types].xml", content_types(&many))]),
        types,
    );
    c.refused(
        "not a presentation",
        with(&[("[Content_Types].xml", content_types("").replace(
            "application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml",
            &format!("application/{MARK}"),
        ))]),
    );
    let mut compound = vec![0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
    compound.extend(MARK.repeat(20).bytes());
    c.refused("compound file", compound);

    // --- Markup compatibility ----------------------------------------------------------
    let must = format!(
        "<p:sld xmlns:p=\"{PML}\" xmlns:mc=\"http://schemas.openxmlformats.org/markup-compatibility/2006\" \
         xmlns:{MARK}=\"urn:{MARK}\" mc:MustUnderstand=\"{MARK}\"/>"
    );
    if let Some(mut pkg) = c.open(
        "must understand",
        with(&[("ppt/slides/slide1.xml", must)]),
        lim(),
    ) {
        c.error(
            "must understand",
            &pkg.read_resolved("ppt/slides/slide1.xml").unwrap_err(),
        );
    }

    // --- Findings ----------------------------------------------------------------------
    let deck = with(&[
        (
            "ppt/slides/_rels/slide1.xml.rels",
            rels(&[
                ("slideLayout", "../slideLayouts/slideLayout1.xml", None),
                (
                    "image",
                    &format!("https://{MARK}.example/{MARK}.png"),
                    Some("External"),
                ),
                ("hyperlink", &format!("javascript:{MARK}"), Some("External")),
                ("image", &format!("../../../../{MARK}.png"), None),
                ("image", &format!("../media/{MARK}.png"), None),
                ("image", "../media/x.undeclared", None),
                (
                    &format!("http://{MARK}.example/{MARK}"),
                    &format!("https://{MARK}"),
                    Some("External"),
                ),
                (
                    "http://schemas.microsoft.com/office/2006/relationships/vbaProject",
                    "../v.bin",
                    None,
                ),
                ("oleObject", "../e.bin", None),
                ("image", &format!("%2e%2e/{MARK}.png"), None),
            ]),
        ),
        ("ppt/media/x.undeclared", MARK.to_owned()),
        ("ppt/v.bin", MARK.to_owned()),
        ("ppt/e.bin", MARK.to_owned()),
        ("ppt/orphan.xml", MARK.to_owned()),
        (
            "_rels/.rels",
            rels(&[
                ("officeDocument", "ppt/presentation.xml", None),
                (
                    "http://schemas.openxmlformats.org/package/2006/relationships/digital-signature/origin",
                    "_xmlsignatures/origin.sigs",
                    None,
                ),
            ]),
        ),
        ("_xmlsignatures/origin.sigs", MARK.to_owned()),
        (
            "ppt/presentation.xml",
            format!(
                "<p:presentation xmlns:p=\"{PML}\" xmlns:mc=\"http://schemas.openxmlformats.org/markup-compatibility/2006\" mc:Ignorable=\"{MARK}\"/>"
            ),
        ),
    ]);
    let pkg = c
        .open("findings", deck, lim())
        .expect("findings do not stop a deck");
    assert!(pkg.findings().len() >= 11, "{:?}", pkg.findings());

    let missing: Vec<String> = registered()
        .into_iter()
        .filter(|code| !c.codes.contains(code))
        .collect();
    assert!(missing.is_empty(), "codes never produced: {missing:?}");
    assert!(
        c.leaks.is_empty(),
        "{} leaks:\n{}",
        c.leaks.len(),
        c.leaks.join("\n")
    );
}
