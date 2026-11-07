//! Tier 1H: pictures, SVG, audio and video, read from fixtures.
//!
//! The rows are read from `docs/spec/checklist.tsv` when the test runs, so a row added to
//! PIC or MEDIA with no answer here fails. Each row is proven by taking its text out of
//! `generated/media-every-reference.pptx` and seeing the model or the report change.

use std::path::{Path, PathBuf};

use casual_slide_import::color::Findings as ColorFindings;
use casual_slide_import::media::{
    CatalogFinding, MediaFinding, MediaLimits, catalog_media, sha256,
};
use casual_slide_import::presentation::import_deck;
use casual_slide_import::properties::{Findings, ImportFindingKind as K, ImportLimits};
use casual_slide_import::slide::{SlideLimits, import_slide_part, import_slide_parts};
use casual_slide_model::fill::{Blip, BlipFillMode, Fill};
use casual_slide_model::media::{
    AudioCd, AudioCdTime, DisplayLocation, EmbeddedWavAudioFile, ImageFormat, Media, MediaClass,
    MediaFile,
};
use casual_slide_model::properties::Lexical;
use casual_slide_model::shape::{NodeKind, ShapeNode};
use casual_slide_model::slide::{SlideKind, SlidePart};
use casual_slide_model::text::Bullet;
use casual_slide_ooxml::{ElementId, OoxmlLimits, PresentationPackage, Relationship, XmlTree};

/// How a name outside the schemas is reported, as it may be document text.
const UNKNOWN_NAME: &str = "(unknown)";
const SLIDE: &str = "ppt/slides/slide1.xml";
const EVERYTHING: &str = "generated/media-every-reference.pptx";
const EXT: &str = r#"<a:extLst><a:ext uri="{00000000-0000-0000-0000-000000000001}"/></a:extLst>"#;
const RGB: &str = r#"<a:srgbClr val="112233"><a:alpha val="50000"/></a:srgbClr>"#;

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn read_fixture(rel: &str) -> Vec<u8> {
    std::fs::read(fixtures().join(rel)).unwrap()
}

fn open(rel: &str) -> PresentationPackage {
    PresentationPackage::open(read_fixture(rel), OoxmlLimits::default()).unwrap()
}

struct Read {
    part: SlidePart,
    findings: Findings,
}

fn read_xml(xml: &str, rels: &[Relationship]) -> Read {
    let tree = XmlTree::parse(xml.as_bytes(), SLIDE, &OoxmlLimits::default()).unwrap();
    let mut findings = Findings::new(100_000);
    let mut color = ColorFindings::default();
    let part = import_slide_part(
        &tree,
        SLIDE,
        rels,
        SlideKind::Slide,
        0,
        &SlideLimits::default(),
        &ImportLimits::default(),
        &mut findings,
        &mut color,
    )
    .unwrap();
    Read { part, findings }
}

fn source(rel: &str) -> (String, Vec<Relationship>) {
    let mut pkg = open(rel);
    let xml = String::from_utf8(pkg.read_part(SLIDE).unwrap()).unwrap();
    let rels = pkg.relationships_of(SLIDE).to_vec();
    (xml, rels)
}

fn read_fixture_slide(rel: &str) -> Read {
    let (xml, rels) = source(rel);
    read_xml(&xml, &rels)
}

fn named<'a>(part: &'a SlidePart, name: &str) -> &'a ShapeNode {
    part.shape_tree()
        .unwrap()
        .descendants()
        .find(|n| {
            n.non_visual
                .as_ref()
                .is_some_and(|v| v.name.as_deref() == Some(name))
        })
        .unwrap_or_else(|| panic!("no node named {name}"))
}

/// How a row is shown to be handled.
enum Proof {
    /// Text that must be in the fixture, for a row that only groups other rows.
    Present(&'static str),
    /// Replacing this text changes the model.
    Modelled(String, String),
    /// Replacing this text leaves the model, and reports this element one fewer time.
    Reported(String, String, &'static str),
    /// Inside a retained element: this text is in the fixture, this container is reported,
    /// and taking the container away takes the report away.
    Inside(&'static str, &'static str, &'static str),
    /// An entry in an extension list: without it the element is not reported, with it, it is.
    Extension(&'static str, &'static str),
    /// A value of a simple type: this attribute holding it reads as itself.
    Value(&'static str, &'static str),
}

fn cut(text: &str) -> Proof {
    Proof::Modelled(text.to_owned(), String::new())
}

fn swap(from: &str, to: &str) -> Proof {
    Proof::Modelled(from.to_owned(), to.to_owned())
}

fn ext_of(head: &str, tail: &str, reported: &'static str) -> Proof {
    Proof::Reported(
        format!("{head}{EXT}{tail}"),
        format!("{head}{tail}"),
        reported,
    )
}

const WORD_PICTURE: &str =
    r#"<a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture">"#;
const CANVAS: &str =
    r#"<a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/lockedCanvas">"#;

/// The proof for a row, by its id. A new row in the checklist has none, and fails.
fn proof(id: &str) -> Option<Proof> {
    let video = r#"<a:videoFile r:link="rId4" contentType="video/mp4">"#;
    let audio = r#"<a:audioFile r:link="rId6" contentType="audio/mpeg">"#;
    Some(match id {
        // MEDIA: `a:`
        "a:CT_AudioFile" => Proof::Present("<a:audioFile"),
        "a:CT_AudioFile/extLst" => ext_of(audio, "</a:audioFile>", "extLst"),
        "a:CT_AudioFile/@r:link" => swap(audio, r#"<a:audioFile contentType="audio/mpeg">"#),
        "a:CT_AudioFile/@contentType" => swap(audio, r#"<a:audioFile r:link="rId6">"#),
        "a:CT_VideoFile" => Proof::Present("<a:videoFile"),
        "a:CT_VideoFile/extLst" => ext_of(video, "</a:videoFile>", "extLst"),
        "a:CT_VideoFile/@r:link" => swap(video, r#"<a:videoFile contentType="video/mp4">"#),
        "a:CT_VideoFile/@contentType" => swap(video, r#"<a:videoFile r:link="rId4">"#),
        "a:<videoFile>" => Proof::Extension(
            r#"<a:ext uri="{00000000-0000-0000-0000-000000000005}"><a:videoFile r:link="rId4"/></a:ext>"#,
            "videoFile",
        ),
        "a:CT_QuickTimeFile" => Proof::Present("<a:quickTimeFile"),
        "a:CT_QuickTimeFile/extLst" => ext_of(
            r#"<a:quickTimeFile r:link="rId13">"#,
            "</a:quickTimeFile>",
            "extLst",
        ),
        "a:CT_QuickTimeFile/@r:link" => {
            swap(r#"<a:quickTimeFile r:link="rId13">"#, "<a:quickTimeFile>")
        }
        "a:CT_AudioCDTime" => Proof::Present(r#"<a:st track="1""#),
        "a:CT_AudioCDTime/@track" => swap(r#"<a:st track="1" time="2"/>"#, r#"<a:st time="2"/>"#),
        "a:CT_AudioCDTime/@time" => swap(r#"<a:st track="1" time="2"/>"#, r#"<a:st track="1"/>"#),
        "a:CT_AudioCD" => Proof::Present("<a:audioCd>"),
        "a:CT_AudioCD/st" => cut(r#"<a:st track="1" time="2"/>"#),
        "a:CT_AudioCD/end" => cut(r#"<a:end track="3" time="4"/>"#),
        "a:CT_AudioCD/extLst" => ext_of(r#"<a:end track="3" time="4"/>"#, "</a:audioCd>", "extLst"),
        "a:EG_Media" => Proof::Present("<a:audioCd>"),
        "a:EG_Media/audioCd" => cut(&format!(
            r#"<a:audioCd><a:st track="1" time="2"/><a:end track="3" time="4"/>{EXT}</a:audioCd>"#
        )),
        "a:EG_Media/wavAudioFile" => cut(r#"<a:wavAudioFile r:embed="rId8" name="chime.wav"/>"#),
        "a:EG_Media/audioFile" => cut(r#"<a:audioFile r:link="rId12"/>"#),
        "a:EG_Media/videoFile" => cut(r#"<a:videoFile r:link="rId11"/>"#),
        "a:EG_Media/quickTimeFile" => swap(
            &format!(r#"<a:quickTimeFile r:link="rId13">{EXT}</a:quickTimeFile>"#),
            "",
        ),
        "a:CT_EmbeddedWAVAudioFile" => Proof::Present("<a:wavAudioFile"),
        "a:CT_EmbeddedWAVAudioFile/@r:embed" => swap(
            r#"<a:wavAudioFile r:embed="rId8" name"#,
            r#"<a:wavAudioFile name"#,
        ),
        "a:CT_EmbeddedWAVAudioFile/@name" => swap(r#" name="chime.wav""#, ""),
        // `p14:media`: `EXT` rows, read here (`MS-PPTX` "Media").
        "x:powerpoint/2010/main:CT_MediaTrim" => Proof::Present("<p14:trim"),
        "x:powerpoint/2010/main:CT_MediaTrim/@st" => swap(r#"st="1000.5" "#, ""),
        "x:powerpoint/2010/main:CT_MediaTrim/@end" => swap(r#" end="2000""#, ""),
        "x:powerpoint/2010/main:CT_MediaFade" => Proof::Present("<p14:fade"),
        "x:powerpoint/2010/main:CT_MediaFade/@in" => swap(r#"in="300" "#, ""),
        "x:powerpoint/2010/main:CT_MediaFade/@out" => swap(r#" out="400""#, ""),
        "x:powerpoint/2010/main:CT_MediaBookmark" => Proof::Present("<p14:bmk "),
        "x:powerpoint/2010/main:CT_MediaBookmark/@name" => swap(r#"name="One" "#, ""),
        "x:powerpoint/2010/main:CT_MediaBookmark/@time" => swap(r#" time="100""#, ""),
        "x:powerpoint/2010/main:CT_MediaBookmarkList" => Proof::Present("<p14:bmkLst>"),
        "x:powerpoint/2010/main:CT_MediaBookmarkList/bmk" => {
            cut(r#"<p14:bmk name="Two" time="200.25"/>"#)
        }
        "x:powerpoint/2010/main:CT_Media" => Proof::Present("<p14:media"),
        "x:powerpoint/2010/main:CT_Media/trim" => cut(r#"<p14:trim st="1000.5" end="2000"/>"#),
        "x:powerpoint/2010/main:CT_Media/fade" => cut(r#"<p14:fade in="300" out="400"/>"#),
        "x:powerpoint/2010/main:CT_Media/bmkLst" => cut(
            r#"<p14:bmkLst><p14:bmk name="One" time="100"/><p14:bmk name="Two" time="200.25"/></p14:bmkLst>"#,
        ),
        "x:powerpoint/2010/main:CT_Media/extLst" => {
            swap("<p173:tracksInfo", "<p173:wat").with_close("</p173:tracksInfo>", "</p173:wat>")
        }
        "x:powerpoint/2010/main:CT_Media/AG_Blob" => swap(r#"r:embed="rId5""#, ""),
        "x:powerpoint/2010/main:<media>" => Proof::Present("<p14:media"),
        // MEDIA: tracks
        "x:powerpoint/2017/3/main:ST_DisplayLocation" => Proof::Present(r#"displayLoc="media""#),
        "x:powerpoint/2017/3/main:ST_DisplayLocation/=media" => {
            Proof::Value(r#"displayLoc="media""#, "displayLoc")
        }
        "x:powerpoint/2017/3/main:ST_DisplayLocation/=slide" => {
            Proof::Value(r#"displayLoc="media""#, "displayLoc")
        }
        "x:powerpoint/2017/3/main:CT_Track" => Proof::Present("<p173:track "),
        "x:powerpoint/2017/3/main:CT_Track/@id" => {
            swap(r#"id="{1C8F3A2E-5B6D-4E7F-8A9B-0C1D2E3F4A5B}" "#, "")
        }
        "x:powerpoint/2017/3/main:CT_Track/@label" => swap(r#"label="Captions" "#, ""),
        "x:powerpoint/2017/3/main:CT_Track/@lang" => swap(r#"lang="en-US" "#, ""),
        "x:powerpoint/2017/3/main:CT_Track/AG_Blob" => swap(r#" r:embed="rId9""#, ""),
        "x:powerpoint/2017/3/main:CT_TrackList" => Proof::Present("<p173:trackLst>"),
        "x:powerpoint/2017/3/main:CT_TrackList/track" => cut(
            r#"<p173:track id="{2C8F3A2E-5B6D-4E7F-8A9B-0C1D2E3F4A5B}" label="Remote" r:link="rId10"/>"#,
        ),
        "x:powerpoint/2017/3/main:CT_TracksInfo" => Proof::Present("<p173:tracksInfo"),
        "x:powerpoint/2017/3/main:CT_TracksInfo/trackLst" => {
            swap("<p173:trackLst>", "<p173:wat>").with_close("</p173:trackLst>", "</p173:wat>")
        }
        "x:powerpoint/2017/3/main:CT_TracksInfo/@displayLoc" => swap(r#" displayLoc="media""#, ""),
        "x:powerpoint/2017/3/main:<tracksInfo>" => Proof::Present("<p173:tracksInfo"),
        // MEDIA: online content
        "x:word/2020/oembed:CT_OEmbed" => Proof::Present("<woe:oembed"),
        "x:word/2020/oembed:CT_OEmbed/@oEmbedUrl" => {
            swap(r#" oEmbedUrl="https://example.invalid/watch""#, "")
        }
        "x:word/2020/oembed:CT_OEmbed/@mediaType" => swap(r#" mediaType="video""#, ""),
        "x:word/2020/oembed:CT_OEmbed/@picLocksAutoForOEmbed" => {
            swap(r#" picLocksAutoForOEmbed="1""#, "")
        }
        "x:word/2020/oembed:<oembed>" => Proof::Present("<woe:oembed"),
        // PIC: rectangles and references
        "a:CT_RelativeRect" => Proof::Present("<a:srcRect"),
        "a:CT_RelativeRect/@l" => swap(r#"<a:srcRect l="1000" "#, "<a:srcRect "),
        "a:CT_RelativeRect/@t" => swap(r#" t="2000""#, ""),
        "a:CT_RelativeRect/@r" => swap(r#" r="3000""#, ""),
        "a:CT_RelativeRect/@b" => swap(r#" b="4000""#, ""),
        "a:AG_Blob" => Proof::Present("r:embed="),
        "a:AG_Blob/@r:embed" => swap(
            r#"<a:blip r:embed="rId2" cstate="print">"#,
            r#"<a:blip cstate="print">"#,
        ),
        "a:AG_Blob/@r:link" => swap(r#"<a:blip r:link="rId14"/>"#, "<a:blip/>"),
        // PIC: non-visual
        "a:CT_PictureLocking" => Proof::Present("<a:picLocks"),
        "a:CT_PictureLocking/extLst" => ext_of(r#"noCrop="1">"#, "</a:picLocks>", "extLst"),
        "a:CT_PictureLocking/AG_Locking" => swap(r#"noGrp="1" "#, ""),
        "a:CT_PictureLocking/@noCrop" => swap(r#" noCrop="1""#, ""),
        "a:CT_NonVisualPictureProperties" => Proof::Present("<p:cNvPicPr"),
        "a:CT_NonVisualPictureProperties/picLocks" => {
            swap("<a:picLocks ", "<a:picWat ").with_close("</a:picLocks>", "</a:picWat>")
        }
        "a:CT_NonVisualPictureProperties/extLst" => {
            ext_of("</a:picLocks>", "</p:cNvPicPr>", "extLst")
        }
        "a:CT_NonVisualPictureProperties/@preferRelativeResize" => {
            swap(r#" preferRelativeResize="0""#, "")
        }
        // PIC: pictures the model does not read. A `p:pic` is `SHAPE`'s; these are the
        // DrawingML forms, inside a graphic frame's data.
        "a:CT_GvmlPictureNonVisual" => Proof::Inside("<a:nvPicPr>", "lockedCanvas", CANVAS),
        "a:CT_GvmlPictureNonVisual/cNvPr" => {
            Proof::Inside(r#"<a:cNvPr id="2" name="c"/>"#, "lockedCanvas", CANVAS)
        }
        "a:CT_GvmlPictureNonVisual/cNvPicPr" => {
            Proof::Inside("<a:cNvPicPr/>", "lockedCanvas", CANVAS)
        }
        "a:CT_GvmlPicture" => Proof::Inside("<a:pic>", "lockedCanvas", CANVAS),
        "a:CT_GvmlPicture/nvPicPr" => Proof::Inside("<a:nvPicPr>", "lockedCanvas", CANVAS),
        "a:CT_GvmlPicture/blipFill" => Proof::Inside(
            "<a:blipFill><a:blip r:embed=\"rId2\"/></a:blipFill><a:spPr/>",
            "lockedCanvas",
            CANVAS,
        ),
        "a:CT_GvmlPicture/spPr" => Proof::Inside("<a:spPr/>", "lockedCanvas", CANVAS),
        "a:CT_GvmlPicture/style" => Proof::Inside("<a:style>", "lockedCanvas", CANVAS),
        "a:CT_GvmlPicture/extLst" => Proof::Inside("</a:style><a:extLst>", "lockedCanvas", CANVAS),
        "pic:CT_PictureNonVisual" => Proof::Inside("<pic:nvPicPr>", "pic", WORD_PICTURE),
        "pic:CT_PictureNonVisual/cNvPr" => Proof::Inside("<pic:cNvPr ", "pic", WORD_PICTURE),
        "pic:CT_PictureNonVisual/cNvPicPr" => Proof::Inside("<pic:cNvPicPr/>", "pic", WORD_PICTURE),
        "pic:CT_Picture" => Proof::Inside("<pic:pic ", "pic", WORD_PICTURE),
        "pic:CT_Picture/nvPicPr" => Proof::Inside("<pic:nvPicPr>", "pic", WORD_PICTURE),
        "pic:CT_Picture/blipFill" => Proof::Inside("<pic:blipFill>", "pic", WORD_PICTURE),
        "pic:CT_Picture/spPr" => Proof::Inside("<pic:spPr/>", "pic", WORD_PICTURE),
        "pic:<pic>" => Proof::Inside("<pic:pic ", "pic", WORD_PICTURE),
        "x:drawing/2010/picture:<style>" => Proof::Inside("<pic14:style/>", "pic", WORD_PICTURE),
        "x:drawing/2010/picture:<extLst>" => Proof::Inside("<pic14:extLst/>", "pic", WORD_PICTURE),
        // PIC: the blip
        "a:ST_BlipCompression" => Proof::Present(r#"cstate="print""#),
        "a:ST_BlipCompression/=email"
        | "a:ST_BlipCompression/=screen"
        | "a:ST_BlipCompression/=print"
        | "a:ST_BlipCompression/=hqprint"
        | "a:ST_BlipCompression/=none" => Proof::Value(r#"cstate="print""#, "cstate"),
        "a:CT_Blip" => Proof::Present("<a:blip "),
        "a:<blip>" => Proof::Extension(
            r#"<a:ext uri="{00000000-0000-0000-0000-000000000004}"><a:blip r:embed="rId2"/></a:ext>"#,
            "blip",
        ),
        "a:CT_Blip/AG_Blob" => swap(r#"<a:blip r:link="rId14"/>"#, "<a:blip/>"),
        "a:CT_Blip/@cstate" => swap(r#" cstate="print""#, ""),
        "a:CT_Blip/alphaBiLevel" => cut(r#"<a:alphaBiLevel thresh="50000"/>"#),
        "a:CT_Blip/alphaCeiling" => cut("<a:alphaCeiling/>"),
        "a:CT_Blip/alphaFloor" => swap("<a:alphaCeiling/><a:alphaFloor/>", "<a:alphaCeiling/>"),
        "a:CT_Blip/alphaInv" => cut(&format!("<a:alphaInv>{RGB}</a:alphaInv>")),
        "a:CT_Blip/alphaMod" => {
            cut(r#"<a:alphaMod><a:cont type="sib" name="m"><a:alphaFloor/></a:cont></a:alphaMod>"#)
        }
        "a:CT_Blip/alphaModFix" => cut(r#"<a:alphaModFix amt="90000"/>"#),
        "a:CT_Blip/alphaRepl" => cut(r#"<a:alphaRepl a="20000"/>"#),
        "a:CT_Blip/biLevel" => cut(r#"<a:biLevel thresh="30000"/>"#),
        "a:CT_Blip/blur" => cut(r#"<a:blur rad="9"/>"#),
        "a:CT_Blip/clrChange" => cut(&format!(
            r#"<a:clrChange useA="0"><a:clrFrom>{RGB}</a:clrFrom><a:clrTo>{RGB}</a:clrTo></a:clrChange>"#
        )),
        "a:CT_Blip/clrRepl" => cut(&format!("<a:clrRepl>{RGB}</a:clrRepl>")),
        "a:CT_Blip/duotone" => cut(&format!(
            r#"<a:duotone>{RGB}<a:schemeClr val="accent1"/></a:duotone>"#
        )),
        "a:CT_Blip/fillOverlay" => cut(&format!(
            r#"<a:fillOverlay blend="screen"><a:solidFill>{RGB}</a:solidFill></a:fillOverlay>"#
        )),
        "a:CT_Blip/grayscl" => swap("</a:fillOverlay><a:grayscl/>", "</a:fillOverlay>"),
        "a:CT_Blip/hsl" => cut(r#"<a:hsl hue="60000" sat="10000" lum="-10000"/>"#),
        "a:CT_Blip/lum" => cut(r#"<a:lum bright="10000" contrast="-20000"/>"#),
        "a:CT_Blip/tint" => cut(r#"<a:tint hue="1" amt="2"/>"#),
        "a:CT_Blip/extLst" => swap("<asvg:svgBlip ", "<asvg:wat "),
        // PIC: SVG and formula
        "x:drawing/2016/SVG/main:CT_SVGBlip" => Proof::Present("<asvg:svgBlip"),
        "x:drawing/2016/SVG/main:CT_SVGBlip/AG_Blob" => swap(r#" r:link="rId16""#, ""),
        "x:drawing/2016/SVG/main:<svgBlip>" => Proof::Present("<asvg:svgBlip"),
        "x:drawing/2022/imageformula:CT_ImageFormula" => Proof::Present("<aif:imageFormula"),
        "x:drawing/2022/imageformula:CT_ImageFormula/@formula" => swap(r#" formula="=A1""#, ""),
        "x:drawing/2022/imageformula:<imageFormula>" => Proof::Present("<aif:imageFormula"),
        _ => return None,
    })
}

impl Proof {
    /// A second replacement, for what opens and closes.
    fn with_close(self, from: &str, to: &str) -> Proof {
        match self {
            Proof::Modelled(a, b) if !from.is_empty() => {
                Proof::Modelled(format!("{a}\u{0}{from}"), format!("{b}\u{0}{to}"))
            }
            other => other,
        }
    }
}

fn apply(xml: &str, from: &str, to: &str) -> String {
    let mut out = xml.to_owned();
    for (f, t) in from.split('\u{0}').zip(to.split('\u{0}')) {
        assert!(out.contains(f), "not in the fixture: {f}");
        out = out.replacen(f, t, 1);
    }
    out
}

fn retained_count(f: &Findings, element: &str) -> usize {
    f.iter()
        .filter(|x| x.kind == K::RetainedElement && x.element == element)
        .count()
}

fn checklist_rows() -> Vec<(String, String)> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/spec/checklist.tsv");
    let text = std::fs::read_to_string(path).unwrap();
    text.lines()
        .filter_map(|line| {
            let cols: Vec<&str> = line.split('\t').collect();
            let area = *cols.get(1)?;
            let id = *cols.first()?;
            // `p14:media` is in `EXT`; these rows are read here. Its event records and
            // bookmark targets are timing's.
            let p14 = id == "x:powerpoint/2010/main:<media>"
                || (id.starts_with("x:powerpoint/2010/main:CT_Media")
                    && !["BookmarkTarget", "PlaybackEvent", "SeekEvent"]
                        .iter()
                        .any(|x| id.contains(x)));
            (matches!(area, "PIC" | "MEDIA") || p14).then(|| (id.to_owned(), area.to_owned()))
        })
        .collect()
}

#[test]
fn every_pic_and_media_row_is_handled_or_reported() {
    let (xml, rels) = source(EVERYTHING);
    let base = read_xml(&xml, &rels);
    let rows = checklist_rows();
    assert_eq!(
        rows.iter().filter(|(_, area)| area != "EXT").count(),
        116,
        "PIC and MEDIA have changed; so must this"
    );
    assert_eq!(rows.len(), 116 + 18, "p14:media has changed; so must this");
    let mut missing = Vec::new();
    let (mut modelled, mut reported, mut inside, mut values, mut groups) = (0, 0, 0, 0, 0);
    for (id, _) in &rows {
        let Some(proof) = proof(id) else {
            missing.push(id.clone());
            continue;
        };
        match proof {
            Proof::Present(text) => {
                assert!(xml.contains(text), "{id}: {text} is not in the fixture");
                groups += 1;
            }
            Proof::Modelled(from, to) => {
                let read = read_xml(&apply(&xml, &from, &to), &rels);
                assert_ne!(read.part, base.part, "{id}: the model does not hold it");
                modelled += 1;
            }
            Proof::Reported(with, without, element) => {
                // A report names an element once a part, so with every other `extLst`
                // gone, this one alone must make the difference.
                let none = xml.replace(EXT, "");
                let one = apply(&none, &without, &with);
                let (a, b) = (read_xml(&none, &rels), read_xml(&one, &rels));
                assert_eq!(a.part, b.part, "{id}: expected a report, not a value");
                assert_eq!(retained_count(&a.findings, element), 0, "{id}");
                assert!(
                    retained_count(&b.findings, element) > 0,
                    "{id}: {element} was not reported"
                );
                reported += 1;
            }
            Proof::Extension(entry, element) => {
                assert!(xml.contains(entry), "{id}");
                let without = read_xml(&xml.replacen(entry, "", 1), &rels);
                assert_eq!(retained_count(&without.findings, element), 0, "{id}");
                assert!(retained_count(&base.findings, element) > 0, "{id}");
                reported += 1;
            }
            Proof::Inside(text, container, parent) => {
                let at = xml
                    .find(parent)
                    .unwrap_or_else(|| panic!("{id}: no {parent}"));
                let end = at + xml[at..].find("</a:graphicData>").unwrap();
                assert!(
                    xml[at..end].contains(text),
                    "{id}: {text} is not inside {parent}"
                );
                assert!(
                    retained_count(&base.findings, container) > 0,
                    "{id}: not reported"
                );
                // What reports it is that element: without the element, no report.
                let body = &xml[at + parent.len()..end];
                let without = xml.replacen(body, "", 1);
                let read = read_xml(&without, &rels);
                assert!(
                    retained_count(&read.findings, container)
                        < retained_count(&base.findings, container),
                    "{id}: {container} is not what reports it"
                );
                inside += 1;
            }
            Proof::Value(attr_text, attr) => {
                let value = id.rsplit_once("/=").unwrap().1;
                let good = read_xml(
                    &apply(&xml, attr_text, &format!(r#"{attr}="{value}""#)),
                    &rels,
                );
                assert!(
                    !good.findings.iter().any(
                        |f| f.kind == K::MalformedValue && f.attribute.as_deref() == Some(attr)
                    ),
                    "{id}: refused"
                );
                let bad = read_xml(&apply(&xml, attr_text, &format!(r#"{attr}="zzz""#)), &rels);
                assert!(
                    bad.findings.iter().any(
                        |f| f.kind == K::MalformedValue && f.attribute.as_deref() == Some(attr)
                    ),
                    "{id}: anything is accepted"
                );
                values += 1;
            }
        }
    }
    assert!(missing.is_empty(), "rows with no proof: {missing:?}");
    eprintln!(
        "PIC and MEDIA: {modelled} modelled, {reported} reported, {inside} retained inside a \
         reported element, {values} values, {groups} that group others"
    );
}

/// Every value in the checklist reads as itself, compared with the model and not only
/// with the absence of a report.
#[test]
fn every_enumerated_value_reads_as_itself() {
    let (xml, rels) = source(EVERYTHING);
    let mut seen = 0;
    for (id, _) in checklist_rows() {
        let Some((st, value)) = id.rsplit_once("/=") else {
            continue;
        };
        seen += 1;
        if st.ends_with("ST_BlipCompression") {
            let read = read_xml(
                &xml.replacen(r#"cstate="print""#, &format!(r#"cstate="{value}""#), 1),
                &rels,
            );
            let node = named(&read.part, "Everything");
            let got = node
                .properties
                .picture
                .as_ref()
                .unwrap()
                .blip
                .as_ref()
                .unwrap()
                .cstate;
            assert_eq!(got.map(|c| c.to_lexical()).as_deref(), Some(value));
        } else {
            let read = read_xml(
                &xml.replacen(
                    r#"displayLoc="media""#,
                    &format!(r#"displayLoc="{value}""#),
                    1,
                ),
                &rels,
            );
            let node = named(&read.part, "Video embedded");
            let info = node.app.as_ref().unwrap().media_extension.as_ref().unwrap();
            let got = info.tracks.as_ref().unwrap().display_loc.unwrap();
            assert_eq!(got.to_lexical(), value);
        }
    }
    assert_eq!(seen, 7);
}

fn picture_blip(node: &ShapeNode) -> &Blip {
    node.properties
        .picture
        .as_ref()
        .unwrap()
        .blip
        .as_ref()
        .unwrap()
}

#[test]
fn a_picture_reads_its_references_crop_tile_stretch_and_recolour() {
    let read = read_fixture_slide(EVERYTHING);
    let node = named(&read.part, "Everything");
    assert_eq!(node.kind, NodeKind::Picture);
    assert_eq!(node.prefer_relative_resize, Some(false));
    assert_eq!(node.locks.unwrap().no_crop, Some(true));
    let fill = node.properties.picture.as_ref().unwrap();
    assert_eq!(fill.dpi, Some(96));
    assert_eq!(fill.rotate_with_shape, Some(true));
    let crop = fill.source_rect.unwrap();
    assert_eq!(
        [crop.l, crop.t, crop.r, crop.b].map(|p| p.unwrap().0),
        [1000, 2000, 3000, 4000]
    );
    let Some(BlipFillMode::Stretch(Some(rect))) = fill.mode else {
        panic!("stretch");
    };
    assert_eq!(rect.l.unwrap().0, 5000);
    let blip = fill.blip.as_ref().unwrap();
    assert_eq!(blip.embed.as_deref(), Some("rId2"));
    assert_eq!(blip.effects.len(), 17, "every recolour effect");
    let svg = blip.svg.as_ref().unwrap();
    assert_eq!(svg.embed.as_deref(), Some("rId3"));
    assert_eq!(svg.link.as_deref(), Some("rId16"));
    assert_eq!(
        blip.image_formula.as_ref().unwrap().formula.as_deref(),
        Some("=A1")
    );
    let oembed = blip.oembed.as_ref().unwrap();
    assert_eq!(oembed.media_type.as_deref(), Some("video"));
    assert_eq!(oembed.pic_locks_auto_for_o_embed, Some(true));

    let tiled = named(&read.part, "Tiled");
    assert_eq!(picture_blip(tiled).link.as_deref(), Some("rId14"));
    let Some(BlipFillMode::Tile(tile)) = tiled.properties.picture.as_ref().unwrap().mode else {
        panic!("tile");
    };
    assert_eq!(tile.sx.unwrap().0, 50000);
    assert_eq!(tile.flip.unwrap().to_lexical(), "xy");

    let filled = named(&read.part, "Filled");
    let Some(Fill::Blip(f)) = &filled.properties.fill else {
        panic!("a picture fill");
    };
    assert_eq!(f.source_rect.unwrap().l.unwrap().0, 9000);
}

#[test]
fn audio_and_video_read_embedded_and_linked_with_trim_bookmarks_tracks_and_poster() {
    let read = read_fixture_slide(EVERYTHING);
    let video = named(&read.part, "Video embedded");
    let app = video.app.as_ref().unwrap();
    assert_eq!(
        app.media,
        Some(Media::VideoFile(MediaFile {
            link: Some("rId4".to_owned()),
            content_type: Some("video/mp4".to_owned()),
        }))
    );
    let m = app.media_extension.as_ref().unwrap();
    assert_eq!(m.embed.as_deref(), Some("rId5"));
    let trim = m.trim.as_ref().unwrap();
    assert_eq!(trim.st.as_deref(), Some("1000.5"));
    assert_eq!(trim.end.as_deref(), Some("2000"));
    let fade = m.fade.as_ref().unwrap();
    assert_eq!(fade.fade_in.as_deref(), Some("300"));
    assert_eq!(fade.fade_out.as_deref(), Some("400"));
    let marks = m.bookmarks.as_ref().unwrap();
    assert_eq!(marks.len(), 2);
    assert_eq!(marks[1].name.as_deref(), Some("Two"));
    assert_eq!(marks[1].time.as_deref(), Some("200.25"));
    let tracks = m.tracks.as_ref().unwrap();
    assert_eq!(tracks.display_loc, Some(DisplayLocation::Media));
    let list = &tracks.track_lst.as_ref().unwrap().track;
    assert_eq!(list[0].embed.as_deref(), Some("rId9"));
    assert_eq!(list[0].lang.as_deref(), Some("en-US"));
    assert_eq!(list[1].link.as_deref(), Some("rId10"));
    assert_eq!(
        video.poster().and_then(|b| b.embed.as_deref()),
        Some("rId2")
    );

    let linked = named(&read.part, "Video linked");
    let app = linked.app.as_ref().unwrap();
    assert_eq!(
        app.media_extension.as_ref().unwrap().link.as_deref(),
        Some("rId15")
    );
    assert!(matches!(&app.media, Some(Media::VideoFile(f)) if f.link.as_deref() == Some("rId11")));

    assert!(matches!(
        &named(&read.part, "Audio linked").app.as_ref().unwrap().media,
        Some(Media::AudioFile(f)) if f.link.as_deref() == Some("rId12")
    ));
    assert_eq!(
        named(&read.part, "Wave").app.as_ref().unwrap().media,
        Some(Media::WavAudioFile(EmbeddedWavAudioFile {
            embed: Some("rId8".to_owned()),
            name: Some("chime.wav".to_owned()),
        }))
    );
    assert_eq!(
        named(&read.part, "CD").app.as_ref().unwrap().media,
        Some(Media::AudioCd(AudioCd {
            st: Some(AudioCdTime {
                track: Some(1),
                time: Some(2)
            }),
            end: Some(AudioCdTime {
                track: Some(3),
                time: Some(4)
            }),
        }))
    );
    assert!(matches!(
        &named(&read.part, "QuickTime").app.as_ref().unwrap().media,
        Some(Media::QuickTimeFile(f)) if f.link.as_deref() == Some("rId13")
    ));

    let sound = named(&read.part, "Sound");
    let link = sound
        .non_visual
        .as_ref()
        .unwrap()
        .hyperlink_click
        .as_ref()
        .unwrap();
    assert_eq!(link.sound.as_ref().unwrap().embed.as_deref(), Some("rId8"));

    // A plain picture has no poster: it is not media.
    assert!(named(&read.part, "Everything").poster().is_none());
    assert!(!read.findings.has(K::DanglingRelationship, "media"));
}

#[test]
fn a_picture_bullet_holds_the_whole_blip() {
    let read = read_fixture_slide(EVERYTHING);
    let node = named(&read.part, "Bullet");
    let props = node.text_body.as_ref().unwrap().paragraphs[0]
        .properties
        .as_ref()
        .unwrap();
    let Some(Bullet::Picture(Some(blip))) = &props.bullet else {
        panic!("a picture bullet");
    };
    assert_eq!(blip.embed.as_deref(), Some("rId2"));
    assert_eq!(blip.effects.len(), 1);
    assert_eq!(blip.cstate.unwrap().to_lexical(), "email");
    assert_eq!(blip.svg.as_ref().unwrap().embed.as_deref(), Some("rId3"));
}

/// Each external target, in the slide's report. Nothing here has the means to follow one.
#[test]
fn each_external_reference_is_reported_on_the_element_that_names_it() {
    let read = read_fixture_slide(EVERYTHING);
    let external: Vec<(String, Option<String>)> = read
        .findings
        .iter()
        .filter(|f| f.kind == K::Media(MediaFinding::ExternalTarget))
        .map(|f| (f.element.clone(), f.attribute.clone()))
        .collect();
    for (element, attribute) in [
        ("blip", "link"),
        ("svgBlip", "link"),
        ("oembed", "oEmbedUrl"),
        ("videoFile", "link"),
        ("audioFile", "link"),
        ("quickTimeFile", "link"),
        ("media", "link"),
        ("track", "link"),
    ] {
        assert!(
            external.contains(&(element.to_owned(), Some(attribute.to_owned()))),
            "{element}/@{attribute} was not reported: {external:?}"
        );
    }
}

/// `CS-MED-0002`: at the bound it reads; one past, the whole of `p14:media` is left out and
/// reported. The bound is not lowered: the fixture crosses the default.
#[test]
fn a_list_at_its_bound_reads_and_one_past_is_left_out_and_reported() {
    for (fixture, one, count) in [
        (
            "hostile/media-limit-bookmarks.bin",
            r#"<p14:bmk name="b" time="1"/>"#,
            4097,
        ),
        (
            "hostile/media-limit-tracks.bin",
            r#"<p173:track id="{1C8F3A2E-5B6D-4E7F-8A9B-0C1D2E3F4A5B}" label="t"/>"#,
            1025,
        ),
    ] {
        let read = read_fixture_slide(fixture);
        let node = named(&read.part, if count == 4097 { "b" } else { "t" });
        assert!(
            node.app.as_ref().unwrap().media_extension.is_none(),
            "{fixture}"
        );
        assert!(
            read.findings
                .has(K::Media(MediaFinding::OverLimit), "media"),
            "{fixture}"
        );

        let (xml, rels) = source(fixture);
        assert_eq!(xml.matches(one).count(), count);
        let at_bound = read_xml(&xml.replacen(one, "", 1), &rels);
        let node = named(&at_bound.part, if count == 4097 { "b" } else { "t" });
        let m = node
            .app
            .as_ref()
            .unwrap()
            .media_extension
            .as_ref()
            .expect("at the bound");
        assert_eq!(m.embed.as_deref(), Some("rId5"));
        assert!(
            !at_bound
                .findings
                .has(K::Media(MediaFinding::OverLimit), "media")
        );
    }
    assert_eq!(MediaLimits::default().max_bookmarks, 4096);
    assert_eq!(MediaLimits::default().max_tracks, 1024);
}

#[test]
fn every_wrong_reference_is_reported_and_none_refuses() {
    let read = read_fixture_slide("hostile/media-bad-references.bin");
    let f = &read.findings;
    // A relationship of the wrong type, kept.
    assert!(f.has(K::WrongRelationship, "videoFile"));
    // A second EG_Media.
    assert!(f.has(K::DuplicateElement, "audioFile"));
    assert!(f.has(K::DanglingRelationship, "audioFile"));
    assert!(f.has(K::DanglingRelationship, "media"));
    assert!(f.has(K::MalformedValue, "trim"));
    assert!(f.has(K::RetainedElement, UNKNOWN_NAME));
    // `r:embed` reaching outside the package is reported twice over.
    assert!(f.has(K::WrongRelationship, "wavAudioFile"));
    assert!(f.has(K::WrongRelationship, "media"));
    assert!(f.has(K::Media(MediaFinding::ExternalTarget), "wavAudioFile"));
    assert!(f.has(K::MissingRequired, "videoFile"));
    assert!(f.has(K::MissingRequired, "wavAudioFile"));
    assert!(f.has(K::OutOfRange, "st") || f.has(K::MalformedValue, "st"));
    assert!(f.has(K::DuplicateElement, "st"));
    assert!(f.has(K::MissingRequired, "end"));
    assert!(f.has(K::MalformedValue, "tracksInfo"));
    assert!(f.has(K::MalformedValue, "track"));
    assert!(f.has(K::MissingRequired, "track"));
    assert!(f.has(K::DuplicateElement, "trackLst"));
    assert!(f.has(K::DanglingRelationship, "svgBlip"));
    assert!(f.has(K::WrongRelationship, "blip"));
    assert!(f.has(K::MalformedValue, "blip"));
    assert!(f.has(K::MissingRequired, "oembed"));
    assert!(f.has(K::Media(MediaFinding::SvgWithoutFallback), "svgBlip"));
    // What a bad reference said stays in the model, for the tree says so.
    let node = named(&read.part, "dangling");
    assert!(matches!(
        &node.app.as_ref().unwrap().media,
        Some(Media::AudioFile(f)) if f.link.as_deref() == Some("rId404")
    ));
}

#[test]
fn nothing_a_media_element_does_not_model_goes_unreported() {
    let (xml, rels) = source(EVERYTHING);
    for (from, to, element) in [
        (
            r#"<a:videoFile r:link="rId4""#,
            r#"<a:videoFile wat="1" r:link="rId4""#,
            "videoFile",
        ),
        (
            r#"<a:quickTimeFile r:link="rId13""#,
            r#"<a:quickTimeFile contentType="x" r:link="rId13""#,
            "quickTimeFile",
        ),
        (
            r#"<a:wavAudioFile r:embed"#,
            r#"<a:wavAudioFile wat="1" r:embed"#,
            "wavAudioFile",
        ),
        (r#"<a:st track="1""#, r#"<a:st wat="1" track="1""#, "st"),
        (
            r#"<p14:trim st="1000.5""#,
            r#"<p14:trim wat="1" st="1000.5""#,
            "trim",
        ),
        (
            r#"<p14:fade in="300""#,
            r#"<p14:fade wat="1" in="300""#,
            "fade",
        ),
        (
            r#"<p14:bmk name="One""#,
            r#"<p14:bmk wat="1" name="One""#,
            "bmk",
        ),
        (
            r#"<p14:media xmlns"#,
            r#"<p14:media wat="1" xmlns"#,
            "media",
        ),
        (
            r#"<p173:tracksInfo xmlns"#,
            r#"<p173:tracksInfo wat="1" xmlns"#,
            "tracksInfo",
        ),
        (
            r#"<p173:track id="{1C"#,
            r#"<p173:track wat="1" id="{1C"#,
            "track",
        ),
        (
            r#"<asvg:svgBlip xmlns"#,
            r#"<asvg:svgBlip wat="1" xmlns"#,
            "svgBlip",
        ),
        (
            r#"<woe:oembed xmlns"#,
            r#"<woe:oembed wat="1" xmlns"#,
            "oembed",
        ),
        (
            r#"<aif:imageFormula xmlns"#,
            r#"<aif:imageFormula wat="1" xmlns"#,
            "imageFormula",
        ),
    ] {
        assert!(xml.contains(from), "{from}");
        let read = read_xml(&xml.replacen(from, to, 1), &rels);
        assert!(
            read.findings
                .iter()
                .any(|f| f.kind == K::RetainedAttribute && f.element == element),
            "{element}: an unknown attribute went unreported"
        );
    }
    // A foreign `ext` in `nvPr` and in `p14:media`, named and not swallowed.
    let foreign = r#"<foreign xmlns="urn:x"/>"#;
    let read = read_xml(
        &xml.replacen(
            r#"<p:extLst><p:ext uri="{DAA4B4D4-6D71-4841-9C94-3DE7FCFB9230}">"#,
            &format!(r#"<p:extLst><p:ext uri="y">{foreign}</p:ext><p:ext uri="{{DAA4B4D4-6D71-4841-9C94-3DE7FCFB9230}}">"#),
            1,
        ),
        &rels,
    );
    assert!(read.findings.has(K::RetainedElement, UNKNOWN_NAME));
    assert!(
        named(&read.part, "Video embedded")
            .app
            .as_ref()
            .unwrap()
            .media_extension
            .is_some(),
        "another's extension hid p14:media"
    );
}

/// The elements of Tier 1H, by local name: a picture, its fill, and every media element.
const MEDIA_ELEMENTS: [&str; 26] = [
    "pic",
    "nvPicPr",
    "cNvPicPr",
    "blipFill",
    "blip",
    "srcRect",
    "stretch",
    "tile",
    "videoFile",
    "audioFile",
    "wavAudioFile",
    "quickTimeFile",
    "audioCd",
    "st",
    "end",
    "media",
    "trim",
    "fade",
    "bmkLst",
    "bmk",
    "tracksInfo",
    "trackLst",
    "track",
    "svgBlip",
    "oembed",
    "imageFormula",
];

/// What a read gives, as something to compare: the model and every finding.
fn as_read(xml: &str, rels: &[Relationship]) -> (SlidePart, Vec<String>) {
    let read = read_xml(xml, rels);
    let report = read
        .findings
        .iter()
        .map(|f| format!("{} {} {:?}", f.kind.code(), f.element, f.attribute))
        .collect();
    (read.part, report)
}

/// A silent drop fails here (`docs/06`, per-tier gate 6), the other way round from the row
/// test above: an element and an attribute nothing knows, put in every picture and media
/// element in turn, must change the model or the report. A reader that skips a child it does
/// not know, and says nothing, fails.
#[test]
fn something_unknown_in_any_picture_or_media_element_is_reported() {
    let (xml, rels) = source(EVERYTHING);
    let whole = XmlTree::parse(xml.as_bytes(), SLIDE, &OoxmlLimits::default()).unwrap();
    let before = as_read(&xml, &rels);
    let mut held: Vec<ElementId> = Vec::new();
    let mut stack = vec![whole.root()];
    while let Some(id) = stack.pop() {
        let qualified = whole.qualified_name(id);
        let local = qualified.rsplit(':').next().unwrap_or(qualified);
        // An extension the fixture made up, and a graphic frame's own content, are kept whole
        // and named at their container: what is inside is that container's, asked in its own test.
        let unknown_ext = local == "ext"
            && whole
                .attributes(id, SLIDE)
                .unwrap()
                .iter()
                .any(|(k, v)| k == "uri" && v.starts_with("{00000000-"));
        if matches!(local, "graphicData" | "Fallback") || unknown_ext {
            continue;
        }
        stack.extend(whole.child_elements(id));
        if MEDIA_ELEMENTS.contains(&local) {
            held.push(id);
        }
    }
    let mut tried = std::collections::HashSet::new();
    let mut silent = Vec::new();
    for id in held {
        let name = whole.qualified_name(id).to_owned();
        if !tried.insert(name.clone()) {
            continue;
        }
        let mut with_child = whole.clone();
        let new = with_child.new_element("p:madeUp");
        with_child.insert_child(id, None, new);
        let text = String::from_utf8(with_child.to_bytes()).unwrap();
        if as_read(&text, &rels) == before {
            silent.push(format!("a child in <{name}>"));
        }
        let mut with_attr = whole.clone();
        assert!(with_attr.set_attribute(id, "madeUp", "1"));
        let text = String::from_utf8(with_attr.to_bytes()).unwrap();
        if as_read(&text, &rels) == before {
            silent.push(format!("an attribute on <{name}>"));
        }
    }
    assert!(tried.len() >= 25, "{}", tried.len());
    assert!(
        silent.is_empty(),
        "{} read past:\n{}",
        silent.len(),
        silent.join("\n")
    );
}

#[test]
fn sha256_matches_the_published_vectors_and_every_fixture_in_the_manifest() {
    let hex = |b: &[u8]| sha256(b).to_string();
    assert_eq!(
        hex(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
        "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
    );
    // Lengths around the block and padding boundaries, against Python's `hashlib`, by way
    // of the manifest: every fixture, whatever its size.
    let manifest = std::fs::read_to_string(fixtures().join("manifest.tsv")).unwrap();
    let mut seen = 0;
    for line in manifest.lines().skip(1) {
        let mut cols = line.split('\t');
        let (path, want) = (cols.next().unwrap(), cols.next().unwrap());
        let bytes = read_fixture(path);
        if bytes.len() > 4 << 20 {
            continue;
        }
        assert_eq!(hex(&bytes), want, "{path}");
        seen += 1;
    }
    assert!(seen > 100, "{seen}");
    for len in [55usize, 56, 63, 64, 65, 119, 120] {
        // The padding boundaries, each checked against the value it must not equal.
        assert_ne!(hex(&vec![0; len]), hex(&vec![0; len + 1]));
    }
}

fn catalog_of(
    rel: &str,
    limits: &MediaLimits,
) -> (casual_slide_model::media::MediaCatalog, Vec<CatalogFinding>) {
    catalog_media(&mut open(rel), limits)
}

fn codes(findings: &[CatalogFinding], kind: MediaFinding) -> Vec<&str> {
    findings
        .iter()
        .filter(|f| f.kind == kind)
        .map(|f| f.part.as_str())
        .collect()
}

#[test]
fn opening_reads_no_media_and_the_catalog_reads_it_when_asked() {
    let mut pkg = open(EVERYTHING);
    let at_open = pkg.consumed();
    let deck = import_deck(&mut pkg, &ImportLimits::default())
        .unwrap()
        .deck;
    import_slide_parts(
        &mut pkg,
        &deck.presentation,
        &SlideLimits::default(),
        &ImportLimits::default(),
    )
    .unwrap();
    let after_slides = pkg.consumed();
    // Only XML so far: the sum of every media part would be seen here.
    let (catalog, _) = catalog_media(&mut pkg, &MediaLimits::default());
    let media: u64 = catalog.parts.iter().map(|p| p.size.unwrap()).sum();
    assert!(media > 0);
    assert_eq!(
        pkg.consumed(),
        after_slides + media,
        "the catalog charges what it reads"
    );
    assert!(at_open <= after_slides);
}

#[test]
fn a_catalog_knows_a_part_by_its_hash_and_every_relationship_to_it() {
    let (catalog, findings) = catalog_of(EVERYTHING, &MediaLimits::default());
    let picture = catalog.part("ppt/media/image1.png").unwrap();
    assert_eq!(picture.class, MediaClass::Image);
    assert_eq!(picture.format, Some(ImageFormat::Png));
    let size = picture.dimensions.unwrap();
    assert_eq!((size.width, size.height), (1, 1));
    assert!(picture.is_shared());
    assert_eq!(picture.referrers.len(), 2, "both slides");
    assert!(picture.outlives("ppt/slides/slide1.xml", "rId2"));

    let png = read_fixture_part(EVERYTHING, "ppt/media/image1.png");
    assert_eq!(picture.hash, Some(sha256(&png)));
    assert_eq!(catalog.with_hash(sha256(&png)).unwrap().part, picture.part);

    let clip = catalog.part("ppt/media/media1.mp4").unwrap();
    assert_eq!(clip.class, MediaClass::Video);
    assert_eq!(clip.format, None, "nothing looks inside a clip");
    assert_eq!(clip.dimensions, None);
    assert_eq!(clip.referrers.len(), 4, "video and media, from both slides");
    assert_eq!(
        catalog
            .reached_by("ppt/slides/slide1.xml", "rId5")
            .unwrap()
            .part,
        clip.part
    );
    assert_eq!(
        catalog.part("ppt/media/media2.mp3").unwrap().class,
        MediaClass::Audio
    );
    assert_eq!(
        catalog.part("ppt/media/track1.vtt").unwrap().class,
        MediaClass::Track
    );

    // Every external target, and not one fetched: this crate cannot.
    assert_eq!(catalog.external.len(), 14, "{:?}", catalog.external);
    assert_eq!(
        codes(&findings, MediaFinding::ExternalTarget).len(),
        catalog.external.len()
    );
    assert!(
        catalog
            .external
            .iter()
            .any(|e| e.class == MediaClass::Track)
    );
    assert!(
        findings
            .iter()
            .all(|f| f.kind == MediaFinding::ExternalTarget),
        "{findings:?}"
    );
}

fn read_fixture_part(rel: &str, part: &str) -> Vec<u8> {
    open(rel).read_part(part).unwrap()
}

/// `SEC-C03`: the SVG is a file nothing reads. It holds a script, `foreignObject`, an
/// external reference, a DOCTYPE and bad XML, and any parser reaching it would refuse.
#[test]
fn a_scripted_svg_stays_opaque_and_its_content_reaches_nothing() {
    let bytes = read_fixture("generated/svg-with-script.pptx");
    let svg = read_fixture_part("generated/svg-with-script.pptx", "ppt/media/image2.svg");
    for needle in [
        "<script",
        "foreignObject",
        "onload",
        "attacker.example",
        "<!DOCTYPE",
    ] {
        assert!(String::from_utf8_lossy(&svg).contains(needle), "{needle}");
    }
    assert!(
        XmlTree::parse(&svg, "ppt/media/image2.svg", &OoxmlLimits::default()).is_err(),
        "the XML reader must refuse this, or this test shows nothing"
    );
    // Opening, reading the deck and listing its media all ignore what is in it.
    let mut pkg = PresentationPackage::open(bytes, OoxmlLimits::default()).unwrap();
    let deck = import_deck(&mut pkg, &ImportLimits::default()).unwrap();
    let parts = import_slide_parts(
        &mut pkg,
        &deck.deck.presentation,
        &SlideLimits::default(),
        &ImportLimits::default(),
    )
    .unwrap();
    let (catalog, findings) = catalog_media(&mut pkg, &MediaLimits::default());
    let entry = catalog.part("ppt/media/image2.svg").unwrap();
    assert_eq!(entry.class, MediaClass::Svg);
    assert_eq!(entry.hash, Some(sha256(&svg)));
    assert_eq!(entry.format, None, "SVG is not sniffed");
    assert_eq!(entry.dimensions, None, "SVG is not measured");
    assert!(findings.is_empty(), "{findings:?}");

    // Nothing the SVG says is anywhere in what was read.
    let everything = format!(
        "{:?}{:?}{catalog:?}{findings:?}",
        parts.parts,
        parts.findings.iter().collect::<Vec<_>>()
    );
    for needle in [
        "<script",
        "fetch(",
        "foreignObject",
        "onload",
        "attacker",
        "DOCTYPE",
        "iframe",
    ] {
        assert!(
            !everything.contains(needle),
            "{needle} reached the model or the report"
        );
    }
    let node = parts.parts.get(parts.parts.structure.slides[0]).unwrap();
    let node = named(node, "Scripted");
    let blip = picture_blip(node);
    assert_eq!(blip.embed.as_deref(), Some("rId2"), "the PNG fallback");
    assert_eq!(blip.svg.as_ref().unwrap().embed.as_deref(), Some("rId3"));
}

/// `SEC-C01`: small files declaring enormous images. Read from the header, not decoded,
/// and reported for a renderer to refuse. Every format here that gives a size.
#[test]
fn an_image_declaring_too_many_pixels_is_reported_from_its_header() {
    let (catalog, findings) = catalog_of("hostile/media-image-bomb.bin", &MediaLimits::default());
    let over = codes(&findings, MediaFinding::OverPixelCap);
    assert_eq!(over.len(), 4, "{findings:?}");
    for (name, format, width, height) in [
        (
            "ppt/media/b0.png",
            ImageFormat::Png,
            0x7FFF_FFFF,
            0x7FFF_FFFF,
        ),
        ("ppt/media/b1.gif", ImageFormat::Gif, 0xFFFF, 0xFFFF),
        ("ppt/media/b2.jpg", ImageFormat::Jpeg, 0xFFFF, 0xFFFF),
        (
            "ppt/media/b3.bmp",
            ImageFormat::Bmp,
            0x7FFF_FFFF,
            0x7FFF_FFFF,
        ),
    ] {
        let part = catalog.part(name).unwrap();
        assert_eq!(part.format, Some(format), "{name}");
        let size = part.dimensions.unwrap();
        assert_eq!((size.width, size.height), (width, height), "{name}");
        assert!(over.contains(&name), "{name}");
        assert!(
            part.size.unwrap() < 64,
            "{name} is not an image, only a header"
        );
    }
    // One under the bound is not reported, one over is.
    let exact = catalog
        .part("ppt/media/b1.gif")
        .unwrap()
        .dimensions
        .unwrap()
        .pixels();
    let (_, at) = catalog_of(
        "hostile/media-image-bomb.bin",
        &MediaLimits {
            max_image_pixels: exact,
            ..MediaLimits::default()
        },
    );
    assert!(!codes(&at, MediaFinding::OverPixelCap).contains(&"ppt/media/b1.gif"));
    let (_, past) = catalog_of(
        "hostile/media-image-bomb.bin",
        &MediaLimits {
            max_image_pixels: exact - 1,
            ..MediaLimits::default()
        },
    );
    assert!(codes(&past, MediaFinding::OverPixelCap).contains(&"ppt/media/b1.gif"));
}

/// A file that claims to be one thing and holds another, or stops inside its header.
#[test]
fn an_image_that_is_not_what_it_says_is_reported_and_not_believed() {
    let (catalog, findings) = catalog_of("hostile/media-lying-images.bin", &MediaLimits::default());
    let mismatched = codes(&findings, MediaFinding::FormatMismatch);
    assert!(mismatched.contains(&"ppt/media/fake.png"), "{findings:?}");
    assert!(mismatched.contains(&"ppt/media/fake.jpg"));
    assert!(mismatched.contains(&"ppt/media/empty.png"));
    assert_eq!(
        catalog.part("ppt/media/fake.jpg").unwrap().format,
        Some(ImageFormat::Png)
    );
    // Cut short: the format, and no size. Nothing was guessed.
    for name in [
        "cut.png", "cut.jpg", "cut2.jpg", "zero.jpg", "run.jpg", "cut.gif", "cut.bmp",
    ] {
        let part = catalog.part(&format!("ppt/media/{name}")).unwrap();
        assert!(part.format.is_some(), "{name}");
        assert_eq!(part.dimensions, None, "{name}");
    }
}

#[test]
fn a_target_that_is_not_there_is_reported() {
    let (catalog, findings) =
        catalog_of("hostile/media-missing-target.bin", &MediaLimits::default());
    assert!(catalog.parts.is_empty());
    assert_eq!(
        codes(&findings, MediaFinding::MissingPart),
        ["ppt/slides/slide1.xml"]
    );
    assert_eq!(findings[0].rel_id.as_deref(), Some("rId2"));
}

/// `CS-MED-0003` and `CS-MED-0008`. The first by the package's capped read, with nothing
/// inflated, and by the catalog's own byte budget.
#[test]
fn a_catalog_stops_at_its_bounds_and_says_so() {
    let mut pkg = PresentationPackage::open(
        read_fixture("real/z14230715-ddi-ko.pptx"),
        OoxmlLimits::default(),
    )
    .unwrap();
    let before = pkg.consumed();
    let (catalog, findings) = catalog_media(
        &mut pkg,
        &MediaLimits {
            max_part_bytes: 0,
            ..MediaLimits::default()
        },
    );
    assert_eq!(pkg.consumed(), before, "refused with nothing inflated");
    let over = codes(&findings, MediaFinding::PartOverCap);
    assert!(!over.is_empty(), "the deck has a picture larger than this");
    for name in &over {
        let part = catalog.part(name).unwrap();
        assert_eq!(part.hash, None, "{name}");
        assert_eq!(part.size, None, "{name}");
    }

    let (catalog, findings) = catalog_of(
        EVERYTHING,
        &MediaLimits {
            max_catalog_bytes: 1,
            ..MediaLimits::default()
        },
    );
    let over = codes(&findings, MediaFinding::PartOverCap);
    assert_eq!(
        over.len(),
        catalog.parts.len() - 1,
        "the first is read, then it is spent"
    );

    let (catalog, findings) = catalog_of(
        EVERYTHING,
        &MediaLimits {
            max_catalog_parts: 2,
            ..MediaLimits::default()
        },
    );
    assert_eq!(catalog.parts.len(), 2);
    assert!(
        findings
            .iter()
            .any(|f| f.kind == MediaFinding::CatalogOverBound)
    );
}

#[test]
fn catalog_codes_are_stable_and_carry_no_document_text() {
    for (kind, code) in [
        (MediaFinding::ExternalTarget, "CS-MED-0001"),
        (MediaFinding::OverLimit, "CS-MED-0002"),
        (MediaFinding::PartOverCap, "CS-MED-0003"),
        (MediaFinding::OverPixelCap, "CS-MED-0004"),
        (MediaFinding::FormatMismatch, "CS-MED-0005"),
        (MediaFinding::MissingPart, "CS-MED-0006"),
        (MediaFinding::SvgWithoutFallback, "CS-MED-0007"),
        (MediaFinding::CatalogOverBound, "CS-MED-0008"),
        (MediaFinding::UnreadablePart, "CS-MED-0009"),
    ] {
        assert_eq!(kind.code(), code);
        assert_eq!(K::Media(kind).code(), code);
    }
    let read = read_fixture_slide(EVERYTHING);
    let (_, catalog) = catalog_of(EVERYTHING, &MediaLimits::default());
    let reported = format!("{:?}{catalog:?}", read.findings.iter().collect::<Vec<_>>());
    for text in [
        "example.invalid",
        "chime.wav",
        "Captions",
        "=A1",
        "Bookmark",
        "file:///",
    ] {
        assert!(!reported.contains(text), "{text} reached a finding");
    }
}

/// Real decks and producers: every picture reads, each reference resolves as the deck
/// wrote it, and the SVG real deck keeps its fallback.
#[test]
fn real_decks_catalog_with_no_lies_and_keep_svg_with_its_fallback() {
    let mut pictures = 0;
    let mut svgs = 0;
    let mut hashed = 0;
    for folder in ["real", "producers"] {
        for entry in std::fs::read_dir(fixtures().join(folder)).unwrap() {
            let path = entry.unwrap().path();
            if !matches!(
                path.extension().and_then(|e| e.to_str()),
                Some("pptx" | "pptm" | "ppsx" | "potx")
            ) {
                continue;
            }
            let name = path.display().to_string();
            let mut pkg =
                PresentationPackage::open(std::fs::read(&path).unwrap(), OoxmlLimits::default())
                    .unwrap();
            let (catalog, findings) = catalog_media(&mut pkg, &MediaLimits::default());
            for f in &findings {
                eprintln!("{name}: {} {} {:?}", f.kind.code(), f.part, f.rel_id);
            }
            assert!(
                !findings.iter().any(|f| matches!(
                    f.kind,
                    MediaFinding::UnreadablePart | MediaFinding::OverPixelCap
                )),
                "{name}: {findings:?}"
            );
            hashed += catalog.parts.iter().filter(|p| p.hash.is_some()).count();
            let deck = import_deck(&mut pkg, &ImportLimits::default())
                .unwrap()
                .deck;
            let slides = import_slide_parts(
                &mut pkg,
                &deck.presentation,
                &SlideLimits::default(),
                &ImportLimits::default(),
            )
            .unwrap();
            for part in slides.parts.parts.iter().flatten() {
                let Some(tree) = part.shape_tree() else {
                    continue;
                };
                for node in tree.descendants() {
                    if let Some(blip) = node
                        .properties
                        .picture
                        .as_ref()
                        .and_then(|p| p.blip.as_ref())
                    {
                        pictures += 1;
                        if blip.svg.is_some() {
                            svgs += 1;
                            assert!(blip.embed.is_some(), "{name}: an SVG with no fallback");
                        }
                    }
                }
            }
            assert!(
                !slides
                    .findings
                    .iter()
                    .any(|f| f.kind == K::Media(MediaFinding::SvgWithoutFallback)),
                "{name}"
            );
        }
    }
    eprintln!("{pictures} pictures, {svgs} with SVG, {hashed} parts hashed");
    assert!(pictures >= 25, "{pictures}");
    assert!(svgs >= 3, "{svgs}");
    assert!(hashed >= 15, "{hashed}");
}
