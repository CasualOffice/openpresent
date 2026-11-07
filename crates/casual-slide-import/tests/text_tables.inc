// The tables Tier 1F's import and export tests both run over. `include!`d, so each test
// crate reads the checklist itself, at test time, and a row or enum value added there
// with no line here fails rather than passing unseen.

use std::collections::BTreeMap;

use casual_slide_model::presentation::Percentage;
use casual_slide_model::properties::Lexical;
use casual_slide_model::text::{Bullet, ParagraphProperties, TabStop, TextBody, TextItem};

const NS_A: &str = "http://schemas.openxmlformats.org/drawingml/2006/main";
const NS_P: &str = "http://schemas.openxmlformats.org/presentationml/2006/main";
const NS_R: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

/// A slide holding one shape with this inside its `p:txBody`.
fn slide_with_body(body: &str) -> String {
    format!(
        r#"<p:sld xmlns:a="{NS_A}" xmlns:r="{NS_R}" xmlns:p="{NS_P}"><p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/><p:sp><p:nvSpPr><p:cNvPr id="2" name="s"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr/><p:txBody>{body}</p:txBody></p:sp></p:spTree></p:cSld></p:sld>"#
    )
}

fn checklist() -> Vec<BTreeMap<&'static str, String>> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/spec/checklist.tsv");
    let text = std::fs::read_to_string(path).unwrap();
    let mut lines = text.lines();
    let names: Vec<String> = lines.next().unwrap().split('\t').map(str::to_owned).collect();
    lines
        .map(|line| {
            names
                .iter()
                .zip(line.split('\t'))
                .map(|(n, v)| (Box::leak(n.clone().into_boxed_str()) as &'static str, v.to_owned()))
                .collect()
        })
        .collect()
}

/// `(simple type, value)` for every enum value in the checklist's TEXT rows.
fn enum_rows() -> Vec<(String, String)> {
    checklist()
        .into_iter()
        .filter(|r| r["area"] == "TEXT" && r["kind"] == "value")
        .map(|r| (r["owner"].clone(), r["name"].trim_start_matches('=').to_owned()))
        .collect()
}

/// `owner/name` for every attribute and element in the checklist's TEXT rows.
fn property_rows() -> Vec<String> {
    checklist()
        .into_iter()
        .filter(|r| r["area"] == "TEXT" && matches!(r["kind"].as_str(), "attribute" | "element"))
        .map(|r| format!("{}/{}", r["owner"], r["name"]))
        .collect()
}

/// How one simple type's values are written, and read back out of a model.
struct EnumCase {
    xml: fn(&str) -> String,
    get: fn(&TextBody) -> Option<String>,
    // Export's test uses it, and import's does not.
    #[allow(dead_code)]
    set: fn(&mut TextBody, &str),
}

fn first_paragraph(b: &TextBody) -> &ParagraphProperties {
    b.paragraphs[0].properties.as_ref().unwrap()
}

fn first_paragraph_mut(b: &mut TextBody) -> &mut ParagraphProperties {
    b.paragraphs[0].properties.as_mut().unwrap()
}

fn first_run(b: &TextBody) -> &casual_slide_model::text::CharacterProperties {
    let TextItem::Run(run) = &b.paragraphs[0].items[0] else {
        panic!("not a run");
    };
    run.properties.as_ref().unwrap()
}

fn first_run_mut(b: &mut TextBody) -> &mut casual_slide_model::text::CharacterProperties {
    let TextItem::Run(run) = &mut b.paragraphs[0].items[0] else {
        panic!("not a run");
    };
    run.properties.as_mut().unwrap()
}

fn lexeme<T: Lexical>(v: Option<&T>) -> Option<String> {
    v.map(Lexical::to_lexical)
}

fn parsed<T: Lexical>(v: &str) -> Option<T> {
    Some(T::parse_lexical(v).unwrap())
}

/// One case per simple type with enum values in TEXT. A new one is a panic, not a pass.
fn enum_case(simple_type: &str) -> EnumCase {
    macro_rules! on_body {
        ($attr:literal, $field:ident) => {
            EnumCase {
                xml: |v| format!(r#"<a:bodyPr {}="{v}"/><a:p/>"#, $attr),
                get: |b| lexeme(b.properties.as_ref().unwrap().$field.as_ref()),
                set: |b, v| b.properties.as_mut().unwrap().$field = parsed(v),
            }
        };
    }
    macro_rules! on_paragraph {
        ($attr:literal, $field:ident) => {
            EnumCase {
                xml: |v| format!(r#"<a:bodyPr/><a:p><a:pPr {}="{v}"/></a:p>"#, $attr),
                get: |b| lexeme(first_paragraph(b).$field.as_ref()),
                set: |b, v| first_paragraph_mut(b).$field = parsed(v),
            }
        };
    }
    macro_rules! on_run {
        ($attr:literal, $field:ident) => {
            EnumCase {
                xml: |v| format!(r#"<a:bodyPr/><a:p><a:r><a:rPr {}="{v}"/><a:t>x</a:t></a:r></a:p>"#, $attr),
                get: |b| lexeme(first_run(b).$field.as_ref()),
                set: |b, v| first_run_mut(b).$field = parsed(v),
            }
        };
    }
    match simple_type {
        "ST_TextAnchoringType" => on_body!("anchor", anchor),
        "ST_TextVertOverflowType" => on_body!("vertOverflow", vertical_overflow),
        "ST_TextHorzOverflowType" => on_body!("horzOverflow", horizontal_overflow),
        "ST_TextVerticalType" => on_body!("vert", vertical),
        "ST_TextWrappingType" => on_body!("wrap", wrap),
        "ST_TextAlignType" => on_paragraph!("algn", align),
        "ST_TextFontAlignType" => on_paragraph!("fontAlgn", font_align),
        "ST_TextUnderlineType" => on_run!("u", underline),
        "ST_TextStrikeType" => on_run!("strike", strike),
        "ST_TextCapsType" => on_run!("cap", caps),
        "ST_TextTabAlignType" => EnumCase {
            xml: |v| format!(r#"<a:bodyPr/><a:p><a:pPr><a:tabLst><a:tab pos="0" algn="{v}"/></a:tabLst></a:pPr></a:p>"#),
            get: |b| {
                let tabs: &[TabStop] = first_paragraph(b).tabs.as_ref().unwrap();
                lexeme(tabs[0].align.as_ref())
            },
            set: |b, v| first_paragraph_mut(b).tabs.as_mut().unwrap()[0].align = parsed(v),
        },
        "ST_TextAutonumberScheme" => EnumCase {
            xml: |v| format!(r#"<a:bodyPr/><a:p><a:pPr><a:buAutoNum type="{v}"/></a:pPr></a:p>"#),
            get: |b| match first_paragraph(b).bullet.as_ref() {
                Some(Bullet::AutoNumber { scheme, .. }) => lexeme(scheme.as_ref()),
                _ => None,
            },
            set: |b, v| {
                first_paragraph_mut(b).bullet = Some(Bullet::AutoNumber {
                    scheme: parsed(v),
                    start_at: None,
                });
            },
        },
        other => panic!("no case for {other}: a new enum in TEXT has no handling test"),
    }
}

/// What a row should come to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outcome {
    /// In the model, and not reported as retained.
    Modelled,
    /// That it is stated is in the model; what it holds is another tier's, is reported,
    /// and export cannot add one.
    Stated,
    /// Reported as retained, with no model field. Patching leaves it alone.
    Reported,
}

struct Row {
    base: String,
    with: String,
    outcome: Outcome,
    /// Local name of the element a retained finding names.
    element: &'static str,
}

fn body_row(attrs: &str, children: &str) -> [String; 2] {
    [
        "<a:bodyPr/><a:p/>".to_owned(),
        format!("<a:bodyPr {attrs}>{children}</a:bodyPr><a:p/>"),
    ]
}

fn paragraph_row(attrs: &str, children: &str) -> [String; 2] {
    [
        "<a:bodyPr/><a:p><a:pPr/></a:p>".to_owned(),
        format!("<a:bodyPr/><a:p><a:pPr {attrs}>{children}</a:pPr></a:p>"),
    ]
}

fn run_row(attrs: &str, children: &str) -> [String; 2] {
    [
        "<a:bodyPr/><a:p><a:r><a:rPr/><a:t>x</a:t></a:r></a:p>".to_owned(),
        format!("<a:bodyPr/><a:p><a:r><a:rPr {attrs}>{children}</a:rPr><a:t>x</a:t></a:r></a:p>"),
    ]
}

fn list_row(children: &str) -> [String; 2] {
    [
        "<a:bodyPr/><a:lstStyle/><a:p/>".to_owned(),
        format!("<a:bodyPr/><a:lstStyle>{children}</a:lstStyle><a:p/>"),
    ]
}

fn wrapped(before: &str, after: &str, base: &str, with: &str) -> [String; 2] {
    [format!("{before}{base}{after}"), format!("{before}{with}{after}")]
}

const GUID_FOR_TESTS: &str = "{1C8F3A2E-5B6D-4E7F-8A9B-0C1D2E3F4A5B}";

/// The row table: every attribute and element in TEXT that a text body can reach.
/// `None` for `CT_GvmlTextShape`: only `lockedCanvas` holds one, in `graphicData`,
/// which stays in the tree and is reported as `CS-IMF-0010`.
fn text_row(id: &str) -> Option<Row> {
    use Outcome::{Modelled, Reported, Stated};
    let attr = |name: &str, value: &str| format!(r#"{}="{value}""#, name.trim_start_matches('@'));
    let (owner, name) = id.split_once('/').unwrap();
    let mut outcome = Modelled;
    let mut element = "";
    let pair: [String; 2] = match owner {
        "CT_GvmlTextShape" => return None,
        "CT_TextBodyProperties" => match name {
            "prstTxWarp" => body_row("", r#"<a:prstTxWarp prst="textPlain"/>"#),
            "scene3d" => {
                outcome = Reported;
                element = "scene3d";
                body_row("", r#"<a:scene3d><a:camera prst="orthographicFront"/><a:lightRig rig="threePt" dir="t"/></a:scene3d>"#)
            }
            "extLst" => {
                outcome = Reported;
                element = "extLst";
                body_row("", "<a:extLst/>")
            }
            _ => {
                let value = match name {
                    "@rot" => "60000",
                    "@vertOverflow" | "@horzOverflow" => "clip",
                    "@vert" => "vert",
                    "@wrap" => "none",
                    "@numCol" => "2",
                    "@anchor" => "ctr",
                    "@lIns" | "@tIns" | "@rIns" | "@bIns" | "@spcCol" => "1",
                    _ => "1",
                };
                body_row(&attr(name, value), "")
            }
        },
        "EG_TextAutofit" => body_row("", &format!("<a:{name}/>")),
        "CT_TextNormalAutofit" => body_row("", &format!("<a:normAutofit {}/>", attr(name, "50000"))),
        "CT_TextBody" => match name {
            "bodyPr" => ["<a:p/>".to_owned(), "<a:bodyPr/><a:p/>".to_owned()],
            "lstStyle" => ["<a:bodyPr/><a:p/>".to_owned(), "<a:bodyPr/><a:lstStyle/><a:p/>".to_owned()],
            _ => ["<a:bodyPr/>".to_owned(), "<a:bodyPr/><a:p/>".to_owned()],
        },
        "CT_TextListStyle" => match name {
            "extLst" => {
                outcome = Reported;
                element = "extLst";
                list_row("<a:extLst/>")
            }
            _ => list_row(&format!(r#"<a:{name} algn="r"/>"#)),
        },
        "CT_TextParagraphProperties" => match name {
            "extLst" => {
                outcome = Reported;
                element = "extLst";
                paragraph_row("", "<a:extLst/>")
            }
            "lnSpc" | "spcBef" | "spcAft" => {
                paragraph_row("", &format!(r#"<a:{name}><a:spcPct val="50000"/></a:{name}>"#))
            }
            "tabLst" => paragraph_row("", r#"<a:tabLst><a:tab pos="1"/></a:tabLst>"#),
            "defRPr" => paragraph_row("", r#"<a:defRPr sz="1000"/>"#),
            _ => {
                let value = match name {
                    "@marL" | "@marR" | "@indent" | "@defTabSz" | "@lvl" => "1",
                    "@algn" => "r",
                    "@fontAlgn" => "t",
                    _ => "1",
                };
                paragraph_row(&attr(name, value), "")
            }
        },
        "CT_TextSpacing" => {
            let holder = |inner: &str| format!("<a:bodyPr/><a:p><a:pPr><a:lnSpc>{inner}</a:lnSpc></a:pPr></a:p>");
            let value = if name == "spcPct" { "50000" } else { "100" };
            [holder(r#"<a:spcPts val="7"/>"#), holder(&format!(r#"<a:{name} val="{value}"/>"#))]
        }
        "CT_TextSpacingPercent" => paragraph_row("", r#"<a:lnSpc><a:spcPct val="50000"/></a:lnSpc>"#),
        "CT_TextSpacingPoint" => paragraph_row("", r#"<a:lnSpc><a:spcPts val="100"/></a:lnSpc>"#),
        "CT_TextTabStopList" => paragraph_row("", r#"<a:tabLst><a:tab/></a:tabLst>"#),
        "CT_TextTabStop" => {
            let value = if name == "@pos" { "1" } else { "ctr" };
            let holder = |a: &str| format!("<a:bodyPr/><a:p><a:pPr><a:tabLst><a:tab {a}/></a:tabLst></a:pPr></a:p>");
            [holder(""), holder(&attr(name, value))]
        }
        "EG_TextBulletColor" if name == "buClr" => paragraph_row("", r#"<a:buClr><a:srgbClr val="FF0000"/></a:buClr>"#),
        "EG_TextBulletSize" if name == "buSzPct" => paragraph_row("", r#"<a:buSzPct val="150%"/>"#),
        "EG_TextBulletSize" if name == "buSzPts" => paragraph_row("", r#"<a:buSzPts val="1000"/>"#),
        "EG_TextBulletTypeface" if name == "buFont" => paragraph_row("", r#"<a:buFont typeface="Arial"/>"#),
        "EG_TextBullet" if name == "buAutoNum" => paragraph_row("", r#"<a:buAutoNum type="arabicPlain"/>"#),
        "EG_TextBullet" if name == "buChar" => paragraph_row("", r#"<a:buChar char="-"/>"#),
        "EG_TextBullet" if name == "buBlip" => {
            outcome = Stated;
            element = "buBlip";
            paragraph_row("", r#"<a:buBlip><a:blip r:embed="rId3"/></a:buBlip>"#)
        }
        "EG_TextBulletColor" | "EG_TextBulletSize" | "EG_TextBulletTypeface" | "EG_TextBullet" => {
            paragraph_row("", &format!("<a:{name}/>"))
        }
        "CT_TextBulletSizePercent" => paragraph_row("", r#"<a:buSzPct val="150%"/>"#),
        "CT_TextBulletSizePoint" => paragraph_row("", r#"<a:buSzPts val="1000"/>"#),
        "CT_TextAutonumberBullet" => match name {
            "@type" => paragraph_row("", r#"<a:buAutoNum type="arabicPlain"/>"#),
            _ => paragraph_row("", r#"<a:buAutoNum type="arabicPlain" startAt="3"/>"#),
        },
        "CT_TextCharBullet" => paragraph_row("", r#"<a:buChar char="-"/>"#),
        "CT_TextBlipBullet" => {
            outcome = Stated;
            element = "buBlip";
            paragraph_row("", r#"<a:buBlip><a:blip r:embed="rId3"/></a:buBlip>"#)
        }
        "CT_TextFont" => {
            let font = |a: &str| format!(r#"<a:bodyPr/><a:p><a:r><a:rPr><a:latin {a}/></a:rPr><a:t>x</a:t></a:r></a:p>"#);
            match name {
                "@typeface" => run_row("", r#"<a:latin typeface="Arial"/>"#),
                "@panose" => [font(r#"typeface="A""#), font(r#"typeface="A" panose="020F0502020204030204""#)],
                "@pitchFamily" => [font(r#"typeface="A""#), font(r#"typeface="A" pitchFamily="34""#)],
                _ => [font(r#"typeface="A""#), font(r#"typeface="A" charset="0""#)],
            }
        }
        "CT_TextCharacterProperties" => match name {
            "extLst" => {
                outcome = Reported;
                element = "extLst";
                run_row("", "<a:extLst/>")
            }
            "ln" => {
                outcome = Stated;
                element = "ln";
                run_row("", r#"<a:ln w="1"/>"#)
            }
            "highlight" => run_row("", r#"<a:highlight><a:srgbClr val="FF0000"/></a:highlight>"#),
            "latin" | "ea" | "cs" | "sym" => run_row("", &format!(r#"<a:{name} typeface="Arial"/>"#)),
            "hlinkClick" | "hlinkMouseOver" => run_row("", &format!(r#"<a:{name} r:id="rId2"/>"#)),
            "rtl" => run_row("", r#"<a:rtl val="1"/>"#),
            _ => {
                let value = match name {
                    "@lang" | "@altLang" => "de-DE",
                    "@sz" => "2400",
                    "@u" => "sng",
                    "@strike" => "sngStrike",
                    "@kern" => "1200",
                    "@cap" => "all",
                    "@spc" => "100",
                    "@baseline" => "30000",
                    "@smtId" => "7",
                    "@bmk" => "m",
                    _ => "1",
                };
                run_row(&attr(name, value), "")
            }
        },
        "EG_TextUnderlineLine" if name == "uLn" => {
            outcome = Stated;
            element = "uLn";
            run_row("", r#"<a:uLn w="1"/>"#)
        }
        "EG_TextUnderlineFill" if name == "uFill" => {
            run_row("", r#"<a:uFill><a:solidFill><a:srgbClr val="FF0000"/></a:solidFill></a:uFill>"#)
        }
        "EG_TextUnderlineLine" | "EG_TextUnderlineFill" => run_row("", &format!("<a:{name}/>")),
        "CT_RegularTextRun" => {
            let (before, after) = ("<a:bodyPr/><a:p><a:r>", "</a:r></a:p>");
            match name {
                "rPr" => wrapped(before, after, "<a:t>x</a:t>", "<a:rPr/><a:t>x</a:t>"),
                _ => wrapped(before, after, "", "<a:t>x</a:t>"),
            }
        }
        "CT_TextLineBreak" => wrapped("<a:bodyPr/><a:p><a:br>", "</a:br></a:p>", "", "<a:rPr/>"),
        "CT_TextField" => {
            let field = |attrs: &str, inner: &str| format!(r#"<a:bodyPr/><a:p><a:fld id="{GUID_FOR_TESTS}" {attrs}>{inner}</a:fld></a:p>"#);
            match name {
                "rPr" => [field("", ""), field("", "<a:rPr/>")],
                "pPr" => [field("", ""), field("", "<a:pPr/>")],
                "t" => [field("", ""), field("", "<a:t>1</a:t>")],
                "@type" => [field("", ""), field(r#"type="slidenum""#, "")],
                _ => [
                    "<a:bodyPr/><a:p><a:fld/></a:p>".to_owned(),
                    format!(r#"<a:bodyPr/><a:p><a:fld id="{GUID_FOR_TESTS}"/></a:p>"#),
                ],
            }
        }
        "CT_TextParagraph" => wrapped("<a:bodyPr/><a:p>", "</a:p>", "", &format!("<a:{name}/>")),
        "EG_TextRun" => {
            let inner = match name {
                "r" => "<a:r><a:t>x</a:t></a:r>".to_owned(),
                "br" => "<a:br/>".to_owned(),
                _ => format!(r#"<a:fld id="{GUID_FOR_TESTS}"/>"#),
            };
            wrapped("<a:bodyPr/><a:p>", "</a:p>", "", &inner)
        }
        other => panic!("no row for {other}/{name}: a new row in TEXT has no handling test"),
    };
    let [base, with] = pair;
    Some(Row {
        base,
        with,
        outcome,
        element,
    })
}

fn percent_value(v: &casual_slide_model::text::Written<Percentage>) -> i32 {
    v.value().0
}
