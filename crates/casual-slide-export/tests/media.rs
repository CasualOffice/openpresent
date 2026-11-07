//! Tier 1H: pictures, SVG and media through the tree: real decks first.
//!
//! The deck-level edits run over `fixtures/real`, `fixtures/producers` and
//! `fixtures/generated`. No real deck here holds audio or video, so those come from
//! `generated/media-every-reference.pptx`, which F4 validates.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use casual_slide_export::media::{NewMedia, relationship_references, replace_media};
use casual_slide_export::slide::{patch_slide_part, save_slide_parts};
use casual_slide_import::color::Findings as ColorFindings;
use casual_slide_import::media::{MediaLimits, catalog_media};
use casual_slide_import::presentation::import_deck;
use casual_slide_import::properties::{Findings, ImportLimits};
use casual_slide_import::slide::{SlideLimits, import_slide_part, import_slide_parts};
use casual_slide_model::color::Percentage;
use casual_slide_model::fill::{Blip, BlipFill, BlipFillMode, Fill, RelativeRect};
use casual_slide_model::media::{
    AudioCd, AudioCdTime, DisplayLocation, EmbeddedWavAudioFile, Media, MediaBookmark,
    MediaExtension, MediaFade, MediaFile, MediaTrack, MediaTrim, TrackList, TracksInfo,
};
use casual_slide_model::picture::SvgBlip;
use casual_slide_model::shape::{NodeKind, ObjectId, ShapeNode};
use casual_slide_model::slide::{SlidePart, SlideParts};
use casual_slide_model::text::Bullet;
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage, Relationship, TargetMode, XmlTree};

const EVERYTHING: &str = "generated/media-every-reference.pptx";
const SVG: &str = "generated/svg-with-script.pptx";
const SLIDE1: &str = "ppt/slides/slide1.xml";
const SLIDE2: &str = "ppt/slides/slide2.xml";
const IMAGE_REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image";
const MEDIA_REL: &str = "http://schemas.microsoft.com/office/2007/relationships/media";

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture(rel: &str) -> Vec<u8> {
    std::fs::read(repo().join("fixtures").join(rel)).unwrap()
}

fn open(bytes: &[u8]) -> PresentationPackage {
    PresentationPackage::open(bytes.to_vec(), OoxmlLimits::default()).unwrap()
}

fn model(bytes: &[u8]) -> SlideParts {
    let mut pkg = open(bytes);
    let deck = import_deck(&mut pkg, &ImportLimits::default())
        .unwrap()
        .deck;
    import_slide_parts(
        &mut pkg,
        &deck.presentation,
        &SlideLimits::default(),
        &ImportLimits::default(),
    )
    .unwrap()
    .parts
}

fn parts(bytes: &[u8]) -> BTreeMap<String, Vec<u8>> {
    let mut pkg = open(bytes);
    pkg.part_names()
        .into_iter()
        .map(|n| {
            let b = pkg.read_part(&n).unwrap();
            (n, b)
        })
        .collect()
}

fn decks(folder: &str) -> Vec<(String, Vec<u8>)> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(repo().join("fixtures").join(folder)).unwrap() {
        let path = entry.unwrap().path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if matches!(ext, "pptx" | "potx" | "ppsx" | "pptm") {
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            if name != "five-hundred-slides.pptx" {
                out.push((format!("{folder}/{name}"), std::fs::read(&path).unwrap()));
            }
        }
    }
    out.sort();
    out
}

fn all_decks() -> Vec<(String, Vec<u8>)> {
    let mut out = decks("real");
    out.extend(decks("producers"));
    out.extend(decks("generated"));
    out
}

/// Each part's status and error count, from `tools/f4/validate.py --json`.
fn f4_parts(bytes: &[u8], label: &str) -> Option<BTreeMap<String, (String, u64)>> {
    let required = std::env::var("F4_REQUIRED").is_ok_and(|v| v == "1");
    let have = |tool: &str| Command::new(tool).arg("--version").output().is_ok();
    if !have("xmllint") || !have("python3") {
        assert!(!required, "F4_REQUIRED, and xmllint or python3 is missing");
        eprintln!("xmllint or python3 not installed: F4 skipped");
        return None;
    }
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("f4-media");
    std::fs::create_dir_all(&dir).unwrap();
    let n = NEXT.fetch_add(1, Ordering::Relaxed);
    let file = dir.join(format!("deck-{}-{n}.pptx", std::process::id()));
    std::fs::write(&file, bytes).unwrap();
    let out = Command::new("python3")
        .arg("-I")
        .arg(repo().join("tools/f4/validate.py"))
        .arg("--json")
        .arg(&file)
        .output()
        .unwrap();
    std::fs::remove_file(&file).unwrap();
    assert_ne!(out.status.code(), Some(2), "{label}: F4 could not run");
    let text = String::from_utf8(out.stdout).unwrap();
    let value = |line: &str| {
        line.split_once(": ")
            .map(|(_, v)| v.trim_end_matches(',').trim_matches('"').to_owned())
    };
    let mut parts = BTreeMap::new();
    let (mut errors, mut part) = (0, String::new());
    for line in text.lines().map(str::trim) {
        if line.starts_with("\"error_count\"") {
            errors = value(line).unwrap().parse().unwrap();
        } else if line.starts_with("\"part\"") {
            part = value(line).unwrap();
        } else if line.starts_with("\"status\"") {
            let key = if part.is_empty() {
                "<deck>".to_owned()
            } else {
                std::mem::take(&mut part)
            };
            parts.insert(key, (value(line).unwrap(), errors));
            errors = 0;
        }
    }
    assert!(!parts.is_empty(), "{label}: nothing from F4: {text}");
    Some(parts)
}

/// Nothing an edit touched may get worse, and a deck that passed still passes.
fn f4_validates(source: &[u8], edited: &[u8], label: &str) {
    let (Some(before), Some(after)) = (f4_parts(source, label), f4_parts(edited, label)) else {
        return;
    };
    assert_eq!(
        before.keys().collect::<Vec<_>>(),
        after.keys().collect::<Vec<_>>(),
        "{label}"
    );
    for (part, (status, errors)) in &after {
        let (old_status, old_errors) = &before[part];
        assert!(
            status == old_status && errors <= old_errors,
            "{label}: {part} went from {old_status} ({old_errors}) to {status} ({errors})"
        );
    }
}

fn f4_clean(bytes: &[u8], label: &str) {
    if let Some(parts) = f4_parts(bytes, label) {
        for (part, (status, errors)) in parts {
            assert!(
                status != "invalid" && errors == 0,
                "{label}: {part} is {status} with {errors} errors"
            );
        }
    }
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

fn named_id(part: &SlidePart, name: &str) -> ObjectId {
    named(part, name).id
}

fn named_mut<'a>(part: &'a mut SlidePart, name: &str) -> &'a mut ShapeNode {
    let id = named_id(part, name);
    part.node_mut(id).unwrap()
}

/// The part's first slide, with what an edit made of it, saved over the deck.
fn save(source: &[u8], wanted: &SlidePart) -> Result<(Vec<u8>, Vec<String>), String> {
    let mut pkg = open(source);
    let structure = model(source).structure;
    let outcome = save_slide_parts(
        &mut pkg,
        &structure,
        &[wanted],
        &SlideLimits::default(),
        &ImportLimits::default(),
    )
    .map_err(|e| e.code().to_owned())?;
    Ok((outcome.bytes, outcome.changed_parts))
}

/// Edit a slide: save, reopen, read as asked, save again to the same bytes, every other
/// part byte for byte. Returns the output.
fn check_edit(
    source: &[u8],
    ordinal: u32,
    change: impl FnOnce(&mut SlidePart),
    label: &str,
) -> Vec<u8> {
    let original = model(source);
    let mut wanted = original.get(ordinal).unwrap().clone();
    let name = original.structure.entry(ordinal).unwrap().part.clone();
    change(&mut wanted);
    assert_ne!(&wanted, original.get(ordinal).unwrap(), "{label}: no edit");
    let (bytes, changed) = save(source, &wanted).unwrap_or_else(|e| panic!("{label}: {e}"));
    assert_eq!(changed.as_slice(), std::slice::from_ref(&name), "{label}");
    let (old, new) = (parts(source), parts(&bytes));
    assert_eq!(
        old.keys().collect::<Vec<_>>(),
        new.keys().collect::<Vec<_>>(),
        "{label}"
    );
    for (part, b) in &old {
        if *part != name {
            assert_eq!(new.get(part), Some(b), "{label}: {part} changed");
        }
    }
    let reopened = model(&bytes);
    assert_eq!(reopened.get(ordinal), Some(&wanted), "{label}: read back");
    let (again, changed) = save(&bytes, &wanted).unwrap();
    assert_eq!(again, bytes, "{label}: fixed point");
    assert!(changed.is_empty(), "{label}");
    f4_validates(source, &bytes, label);
    bytes
}

/// Every text position where two byte strings differ lies inside one element: its st
/// tag, in both, starts with this name.
fn only_inside(old: &str, new: &str, element: &str, label: &str) {
    let prefix = old
        .bytes()
        .zip(new.bytes())
        .take_while(|(a, b)| a == b)
        .count();
    let suffix = old
        .bytes()
        .rev()
        .zip(new.bytes().rev())
        .take_while(|(a, b)| a == b)
        .count()
        .min(old.len().min(new.len()) - prefix);
    let window = |s: &str| s[prefix..s.len() - suffix].to_owned();
    let (a, b) = (window(old), window(new));
    for text in [&a, &b] {
        // Whole new elements, and nothing else.
        let starts = text.matches('<').count();
        assert!(
            starts == 0
                || (starts == 1 && text.starts_with(&format!("<{element}")))
                || (starts <= 1 && !text.contains("</")),
            "{label}: the edit reached past {element}: {text:?}"
        );
    }
    if !a.is_empty() || !b.contains('<') {
        let open = old[..prefix].rfind('<').unwrap();
        assert!(
            old[open..].starts_with(&format!("<{element}"))
                || b.starts_with(&format!("<{element}")),
            "{label}: the edit is not in {element}"
        );
    }
}

fn pictures(part: &SlidePart) -> Vec<(ObjectId, String)> {
    part.shape_tree()
        .into_iter()
        .flat_map(ShapeNode::descendants)
        .filter(|n| n.kind == NodeKind::Picture && n.properties.picture.is_some())
        .filter_map(|n| Some((n.id, n.non_visual.as_ref()?.name.clone()?)))
        .collect()
}

fn new_crop() -> Option<RelativeRect> {
    Some(RelativeRect {
        l: Some(Percentage(12_345)),
        ..RelativeRect::default()
    })
}

/// The acceptance edit: crop one picture on a slide, save, reopen. Every other part byte
/// for byte, only that `srcRect` in this one, and a fixed point.
#[test]
fn editing_a_crop_changes_only_that_node_in_every_deck_with_a_picture() {
    let mut edited = 0;
    let mut real = 0;
    for (name, source) in all_decks() {
        let original = model(&source);
        for &ordinal in &original.structure.slides {
            let Some(part) = original.get(ordinal) else {
                continue;
            };
            let Some((id, node_name)) = pictures(part).into_iter().next() else {
                continue;
            };
            let change = |p: &mut SlidePart| {
                let fill = p.node_mut(id).unwrap().properties.picture.as_mut().unwrap();
                fill.source_rect = new_crop();
            };
            // A picture inside alternate content is refused, and stays so.
            let mut wanted = part.clone();
            change(&mut wanted);
            match save(&source, &wanted) {
                Err(code) if code == "CS-SLE-0002" => continue,
                Err(code) => panic!("{name}: {code}"),
                Ok(_) => {}
            }
            let label = format!("{name} {node_name}");
            let bytes = check_edit(&source, ordinal, change, &label);
            let entry = original.structure.entry(ordinal).unwrap().part.clone();
            let old = String::from_utf8_lossy(&parts(&source)[&entry]).into_owned();
            let new = String::from_utf8_lossy(&parts(&bytes)[&entry]).into_owned();
            assert_ne!(old, new, "{label}");
            only_inside(&old, &new, "a:srcRect", &label);
            edited += 1;
            real += usize::from(name.starts_with("real/"));
            break;
        }
    }
    eprintln!("crop edits: {edited}, of which on real decks {real}");
    assert!(edited >= 10, "{edited}");
    assert!(real >= 4, "{real}");
}

#[test]
fn a_strict_deck_is_cropped_in_strict_spelling() {
    let source = fixture("generated/slides-strict.pptx");
    let original = model(&source);
    for &ordinal in &original.structure.slides {
        let part = original.get(ordinal).unwrap();
        if let Some((id, _)) = pictures(part).into_iter().next() {
            let bytes = check_edit(
                &source,
                ordinal,
                |p| {
                    let fill = p.node_mut(id).unwrap().properties.picture.as_mut().unwrap();
                    fill.source_rect = new_crop();
                },
                "strict",
            );
            let entry = original.structure.entry(ordinal).unwrap().part.clone();
            let text = String::from_utf8_lossy(&parts(&bytes)[&entry]).into_owned();
            assert!(text.contains(r#"l="12.345%""#), "{text}");
            return;
        }
    }
    panic!("no picture in the Strict deck");
}

/// `SEC-C03`: the acceptance for SVG. The script stays inert, as bytes in a part nothing
/// reads, and the fixture's SVG comes out byte for byte however the picture is edited.
#[test]
fn a_scripted_svg_stays_byte_identical_however_its_picture_is_edited() {
    let source = fixture(SVG);
    let svg = parts(&source)["ppt/media/image2.svg"].clone();
    assert!(String::from_utf8_lossy(&svg).contains("<script>"));

    // Unedited: the very bytes.
    let original = model(&source);
    let ordinal = original.structure.slides[0];
    let part = original.get(ordinal).unwrap();
    let (bytes, changed) = save(&source, part).unwrap();
    assert_eq!(bytes, source);
    assert!(changed.is_empty());

    let edited = check_edit(
        &source,
        ordinal,
        |p| {
            let node = named_mut(p, "Scripted");
            node.properties.picture.as_mut().unwrap().source_rect = new_crop();
            node.locks = None;
        },
        "svg",
    );
    assert_eq!(parts(&edited)["ppt/media/image2.svg"], svg);
    let blip = named(model(&edited).get(ordinal).unwrap(), "Scripted")
        .properties
        .picture
        .as_ref()
        .unwrap()
        .blip
        .clone()
        .unwrap();
    assert_eq!(blip.embed.as_deref(), Some("rId2"), "the fallback stays");
    assert_eq!(blip.svg.unwrap().embed.as_deref(), Some("rId3"));

    // And in the slide the `svgBlip` is the same bytes too.
    let text = String::from_utf8_lossy(&parts(&edited)[SLIDE1]).into_owned();
    assert!(
        text.contains(r#"r:embed="rId3"/></a:ext></a:extLst></a:blip>"#),
        "{text}"
    );
}

#[test]
fn an_svg_is_not_left_without_its_fallback() {
    let source = fixture(SVG);
    let original = model(&source);
    let mut wanted = original.get(original.structure.slides[0]).unwrap().clone();
    let node = named_mut(&mut wanted, "Scripted");
    node.properties
        .picture
        .as_mut()
        .unwrap()
        .blip
        .as_mut()
        .unwrap()
        .embed = None;
    assert_eq!(save(&source, &wanted).unwrap_err(), "CS-EXP-0002");
    // Asking for the SVG to go, with its fallback staying, is an edit like any other.
    let bytes = check_edit(
        &source,
        original.structure.slides[0],
        |p| {
            let blip = named_mut(p, "Scripted")
                .properties
                .picture
                .as_mut()
                .unwrap()
                .blip
                .as_mut()
                .unwrap();
            blip.svg = None;
        },
        "svg gone",
    );
    let text = String::from_utf8_lossy(&parts(&bytes)[SLIDE1]).into_owned();
    assert!(!text.contains("svgBlip"), "{text}");
    assert!(
        !text.contains("<a:extLst"),
        "the holders go with the last entry: {text}"
    );
    // The part stays: dropping it is the writer's (Tier 1N), by its last relationship.
    assert!(parts(&bytes).contains_key("ppt/media/image2.svg"));
}

fn slide_of(rel: &str, name: &str) -> (Vec<u8>, XmlTree, Vec<Relationship>, SlidePart) {
    let source = fixture(rel);
    let mut pkg = open(&source);
    let rels = pkg.relationships_of(name).to_vec();
    let bytes = pkg.read_part(name).unwrap();
    let tree = XmlTree::parse(&bytes, name, &OoxmlLimits::default()).unwrap();
    let parts = model(&source);
    let part = parts
        .structure
        .parts
        .iter()
        .find(|e| e.part == name)
        .and_then(|e| parts.get(e.id.part))
        .unwrap()
        .clone();
    (source, tree, rels, part)
}

fn replace(
    tree: &mut XmlTree,
    name: &str,
    rels: &[Relationship],
    model: &mut SlidePart,
    node: ObjectId,
    old: &str,
    new: &NewMedia,
) -> Result<casual_slide_export::media::MediaReplacement, String> {
    replace_media(
        tree,
        name,
        false,
        rels,
        model,
        node,
        old,
        new,
        &SlideLimits::default(),
        &ImportLimits::default(),
        &OoxmlLimits::default(),
    )
    .map_err(|e| e.code().to_owned())
}

fn image(target: &str) -> NewMedia {
    NewMedia {
        rel_type: IMAGE_REL.to_owned(),
        target: target.to_owned(),
        mode: TargetMode::Internal,
    }
}

fn read_with(tree: &XmlTree, name: &str, rels: &[Relationship], part: &SlidePart) -> SlidePart {
    let again = XmlTree::parse(&tree.to_bytes(), name, &OoxmlLimits::default()).unwrap();
    let mut findings = Findings::new(10_000);
    let read = import_slide_part(
        &again,
        name,
        rels,
        part.kind,
        part.id.part,
        &SlideLimits::default(),
        &ImportLimits::default(),
        &mut findings,
        &mut ColorFindings::default(),
    )
    .unwrap();
    for f in findings.iter() {
        assert_ne!(f.kind.code(), "CS-IMF-0006", "a dangling reference: {f:?}");
    }
    read
}

/// A picture sharing its part with another slide: the slide gets a new relationship, the
/// old one goes from this part's list, and the part stays, since slide 1 still reaches it.
#[test]
fn replacing_a_shared_picture_gives_the_slide_a_new_relationship_and_leaves_the_part() {
    let (source, mut tree, rels, mut part) = slide_of(EVERYTHING, SLIDE2);
    let node = named_id(&part, "Shared");
    let before: Vec<String> = rels.iter().map(|r| r.id.clone()).collect();
    let done = replace(
        &mut tree,
        SLIDE2,
        &rels,
        &mut part,
        node,
        "rId2",
        &image("../media/new.png"),
    )
    .unwrap();

    // `INV-RI-03`: one above the highest.
    assert_eq!(done.rel_id, "rId17");
    let added = done.added.as_ref().unwrap();
    assert_eq!(added.target, "../media/new.png");
    assert_eq!(added.mode, TargetMode::Internal);
    assert_eq!(done.released.as_deref(), Some("rId2"));
    // `INV-RI-02`: nothing else was renumbered, or moved.
    let after: Vec<String> = done.relationships.iter().map(|r| r.id.clone()).collect();
    let kept: Vec<String> = before
        .iter()
        .filter(|i| *i != "rId2")
        .cloned()
        .chain(["rId17".to_owned()])
        .collect();
    assert_eq!(after, kept);

    let blip = named(&part, "Shared")
        .properties
        .picture
        .as_ref()
        .unwrap()
        .blip
        .as_ref()
        .unwrap();
    assert_eq!(blip.embed.as_deref(), Some("rId17"));
    assert_eq!(read_with(&tree, SLIDE2, &done.relationships, &part), part);

    // The part is still reached from slide 1, so it is not for the writer to remove.
    let (catalog, _) = catalog_media(&mut open(&source), &MediaLimits::default());
    assert!(catalog.part("ppt/media/image1.png").unwrap().is_shared());
    assert!(done.parts_released(&catalog, SLIDE2).is_empty());

    // `INV-RI-04` over the XML: every name resolves.
    let names = relationship_references(&tree, SLIDE2).unwrap();
    assert!(!names.contains_key("rId2"));
    assert_eq!(names.get("rId17"), Some(&1));
}

/// `INV-RI-03`: past a gap and past an id of another form, and no clash with either.
#[test]
fn a_new_relationship_goes_one_above_the_highest_numbered_id() {
    let (_, mut tree, mut rels, mut part) = slide_of(EVERYTHING, SLIDE2);
    rels.retain(|r| r.id != "rId3" && r.id != "rId9");
    rels.push(Relationship {
        id: "custom".to_owned(),
        rel_type: IMAGE_REL.to_owned(),
        target: "../media/other.png".to_owned(),
        mode: TargetMode::Internal,
    });
    let node = named_id(&part, "Shared");
    let done = replace(
        &mut tree,
        SLIDE2,
        &rels,
        &mut part,
        node,
        "rId2",
        &image("../media/new.png"),
    )
    .unwrap();
    assert_eq!(done.rel_id, "rId17", "not the gap at rId3");
}

/// A part reached by one relationship: replacing the picture leaves it to go.
#[test]
fn replacing_the_only_reference_to_a_picture_releases_its_part() {
    let source = fixture("real/z23429129-chapter3-ja.pptx");
    let original = model(&source);
    let (catalog, _) = catalog_media(&mut open(&source), &MediaLimits::default());
    let mut pkg = open(&source);
    let mut tested = 0;
    for &ordinal in &original.structure.slides {
        let part = original.get(ordinal).unwrap();
        let entry = original.structure.entry(ordinal).unwrap().part.clone();
        let rels = pkg.relationships_of(&entry).to_vec();
        for (id, _) in pictures(part) {
            let node = part.node(id).unwrap();
            let Some(old) = node
                .properties
                .picture
                .as_ref()
                .and_then(|f| f.blip.as_ref())
                .and_then(|b| b.embed.clone())
            else {
                continue;
            };
            let bytes = pkg.read_part(&entry).unwrap();
            let mut tree = XmlTree::parse(&bytes, &entry, &OoxmlLimits::default()).unwrap();
            let mut wanted = part.clone();
            let done = replace(
                &mut tree,
                &entry,
                &rels,
                &mut wanted,
                id,
                &old,
                &image("../media/new.png"),
            )
            .unwrap();
            let target = catalog.reached_by(&entry, &old).unwrap();
            assert_eq!(
                target.referrers.len(),
                1,
                "this deck does not share pictures"
            );
            assert_eq!(
                done.parts_released(&catalog, &entry).as_slice(),
                std::slice::from_ref(&target.part)
            );
            tested += 1;
        }
    }
    assert!(tested >= 3, "{tested}");
}

/// The reference stays where the tree still names it: what the model cannot see counts.
#[test]
fn a_relationship_something_else_names_is_not_released() {
    let (_, mut tree, rels, mut part) = slide_of(EVERYTHING, SLIDE1);
    // "Filled" names rId2, as do "Video embedded" and the rest, and a word processing
    // picture in a graphic frame the model does not read.
    let node = named_id(&part, "Filled");
    let done = replace(
        &mut tree,
        SLIDE1,
        &rels,
        &mut part,
        node,
        "rId2",
        &image("../media/new.png"),
    )
    .unwrap();
    assert_eq!(done.released, None);
    assert!(done.relationships.iter().any(|r| r.id == "rId2"));
    assert!(relationship_references(&tree, SLIDE1).unwrap()["rId2"] > 0);
}

#[test]
fn replacing_reuses_a_relationship_already_reaching_the_target_and_saves() {
    // No new relationship, so the deck can be saved today, ahead of Tier 1N.
    let (source, mut tree, rels, mut part) = slide_of(EVERYTHING, SLIDE2);
    let node = named_id(&part, "Shared");
    let target = rels.iter().find(|r| r.id == "rId3").unwrap();
    let new = NewMedia {
        rel_type: target.rel_type.clone(),
        target: target.target.clone(),
        mode: target.mode,
    };
    let done = replace(&mut tree, SLIDE2, &rels, &mut part, node, "rId2", &new).unwrap();
    assert_eq!(done.rel_id, "rId3");
    assert_eq!(done.added, None, "INV-RI-10");
    assert_eq!(done.released.as_deref(), Some("rId2"));
    let ordinal = part.id.part;
    let bytes = check_edit(&source, ordinal, |p| *p = part.clone(), "reuse");
    assert_eq!(
        String::from_utf8_lossy(&parts(&bytes)[SLIDE2]),
        String::from_utf8_lossy(&tree.to_bytes()),
    );
}

#[test]
fn replacing_a_clip_moves_the_embedded_reference_and_leaves_the_video_file() {
    let (_, mut tree, rels, mut part) = slide_of(EVERYTHING, SLIDE1);
    let node = named_id(&part, "Video embedded");
    let new = NewMedia {
        rel_type: MEDIA_REL.to_owned(),
        target: "https://example.invalid/other.mp4".to_owned(),
        mode: TargetMode::External,
    };
    let done = replace(&mut tree, SLIDE1, &rels, &mut part, node, "rId5", &new).unwrap();
    assert_eq!(done.rel_id, "rId17");
    assert_eq!(done.released.as_deref(), Some("rId5"));
    let app = named(&part, "Video embedded").app.as_ref().unwrap();
    assert_eq!(
        app.media_extension.as_ref().unwrap().embed.as_deref(),
        Some("rId17")
    );
    assert!(matches!(&app.media, Some(Media::VideoFile(f)) if f.link.as_deref() == Some("rId4")));
    assert_eq!(read_with(&tree, SLIDE1, &done.relationships, &part), part);
}

#[test]
fn replacing_refuses_half_of_an_svg_pair_and_a_reference_the_node_does_not_hold() {
    let (_, mut tree, rels, mut part) = slide_of(EVERYTHING, SLIDE1);
    let before = tree.to_bytes();
    let snapshot = part.clone();
    let node = named_id(&part, "Everything");
    assert_eq!(
        replace(
            &mut tree,
            SLIDE1,
            &rels,
            &mut part,
            node,
            "rId2",
            &image("../media/x.png")
        )
        .unwrap_err(),
        "CS-MDE-0002"
    );
    assert_eq!(
        replace(
            &mut tree,
            SLIDE1,
            &rels,
            &mut part,
            node,
            "rId3",
            &image("../media/x.svg")
        )
        .unwrap_err(),
        "CS-MDE-0002"
    );
    assert_eq!(
        replace(
            &mut tree,
            SLIDE1,
            &rels,
            &mut part,
            node,
            "rId404",
            &image("../media/x.png")
        )
        .unwrap_err(),
        "CS-MDE-0001"
    );
    let gone = ObjectId {
        part: part.id.part,
        node: 99_999,
    };
    assert_eq!(
        replace(
            &mut tree,
            SLIDE1,
            &rels,
            &mut part,
            gone,
            "rId2",
            &image("../media/x.png")
        )
        .unwrap_err(),
        "CS-MDE-0001"
    );
    assert_eq!(tree.to_bytes(), before, "nothing changed");
    assert_eq!(part, snapshot, "nothing changed");
}

/// An edit may not leave out what the schema requires, or the deck stops validating.
#[test]
fn media_leaving_out_what_the_schema_requires_is_refused() {
    let source = fixture(EVERYTHING);
    let original = model(&source);
    let ordinal = original.structure.slides[0];
    let changes: &[Change] = &[
        |p| {
            named_mut(p, "Video linked").app.as_mut().unwrap().media =
                Some(Media::VideoFile(MediaFile::default()))
        },
        |p| {
            named_mut(p, "Wave").app.as_mut().unwrap().media =
                Some(Media::WavAudioFile(EmbeddedWavAudioFile::default()))
        },
        |p| {
            named_mut(p, "CD").app.as_mut().unwrap().media =
                Some(Media::AudioCd(AudioCd::default()))
        },
        |p| {
            named_mut(p, "CD").app.as_mut().unwrap().media = Some(Media::AudioCd(AudioCd {
                st: Some(AudioCdTime::default()),
                end: Some(AudioCdTime {
                    track: Some(1),
                    time: None,
                }),
            }))
        },
        |p| {
            media_of(p, "Video embedded")
                .tracks
                .as_mut()
                .unwrap()
                .display_loc = None
        },
        |p| {
            media_of(p, "Video embedded")
                .tracks
                .as_mut()
                .unwrap()
                .track_lst
                .as_mut()
                .unwrap()
                .track[0]
                .id = None
        },
        |p| {
            media_of(p, "Video embedded")
                .tracks
                .as_mut()
                .unwrap()
                .track_lst
                .as_mut()
                .unwrap()
                .track[0]
                .label = None
        },
    ];
    for change in changes {
        let mut wanted = original.get(ordinal).unwrap().clone();
        change(&mut wanted);
        assert_eq!(save(&source, &wanted).unwrap_err(), "CS-EXP-0009");
    }
}

#[test]
fn a_model_naming_a_relationship_the_part_lacks_is_refused() {
    let source = fixture(EVERYTHING);
    let original = model(&source);
    let ordinal = original.structure.slides[0];
    for change in [
        (|p| {
            named_mut(p, "Video embedded")
                .app
                .as_mut()
                .unwrap()
                .media_extension
                .as_mut()
                .unwrap()
                .embed = Some("rId404".to_owned());
        }) as fn(&mut SlidePart),
        |p| {
            named_mut(p, "Video linked").app.as_mut().unwrap().media =
                Some(Media::VideoFile(MediaFile {
                    link: Some("rId404".to_owned()),
                    content_type: None,
                }));
        },
        |p| {
            named_mut(p, "Tiled")
                .properties
                .picture
                .as_mut()
                .unwrap()
                .blip
                .as_mut()
                .unwrap()
                .link = Some("rId404".to_owned());
        },
    ] {
        let mut wanted = original.get(ordinal).unwrap().clone();
        change(&mut wanted);
        assert_eq!(save(&source, &wanted).unwrap_err(), "CS-EXP-0003");
    }
}

type Change = fn(&mut SlidePart);

fn media_of<'a>(p: &'a mut SlidePart, name: &str) -> &'a mut MediaExtension {
    named_mut(p, name)
        .app
        .as_mut()
        .unwrap()
        .media_extension
        .as_mut()
        .unwrap()
}

/// Every field the model holds, edited on its own, read back and valid.
#[test]
fn each_picture_and_media_field_is_written_and_reads_back() {
    let source = fixture(EVERYTHING);
    let ordinal = model(&source).structure.slides[0];
    let changes: &[(&str, Change)] = &[
        ("prefer resize", |p| {
            named_mut(p, "Everything").prefer_relative_resize = Some(true)
        }),
        ("prefer resize gone", |p| {
            named_mut(p, "Everything").prefer_relative_resize = None
        }),
        ("crop gone", |p| {
            named_mut(p, "Everything")
                .properties
                .picture
                .as_mut()
                .unwrap()
                .source_rect = None
        }),
        ("stretch to tile", |p| {
            named_mut(p, "Everything")
                .properties
                .picture
                .as_mut()
                .unwrap()
                .mode = Some(BlipFillMode::Tile(Default::default()))
        }),
        ("stretch rect", |p| {
            named_mut(p, "Everything")
                .properties
                .picture
                .as_mut()
                .unwrap()
                .mode = Some(BlipFillMode::Stretch(None))
        }),
        ("recolour gone", |p| {
            named_mut(p, "Everything")
                .properties
                .picture
                .as_mut()
                .unwrap()
                .blip
                .as_mut()
                .unwrap()
                .effects
                .clear()
        }),
        ("svg moved", |p| {
            named_mut(p, "Everything")
                .properties
                .picture
                .as_mut()
                .unwrap()
                .blip
                .as_mut()
                .unwrap()
                .svg = Some(SvgBlip {
                embed: Some("rId2".to_owned()),
                link: None,
            })
        }),
        ("svg added", |p| {
            named_mut(p, "Tiled")
                .properties
                .picture
                .as_mut()
                .unwrap()
                .blip
                .as_mut()
                .unwrap()
                .svg = Some(SvgBlip {
                embed: Some("rId3".to_owned()),
                link: None,
            })
        }),
        ("formula", |p| {
            named_mut(p, "Everything")
                .properties
                .picture
                .as_mut()
                .unwrap()
                .blip
                .as_mut()
                .unwrap()
                .image_formula
                .as_mut()
                .unwrap()
                .formula = Some("=B2".to_owned())
        }),
        ("oembed media type", |p| {
            named_mut(p, "Everything")
                .properties
                .picture
                .as_mut()
                .unwrap()
                .blip
                .as_mut()
                .unwrap()
                .oembed
                .as_mut()
                .unwrap()
                .media_type = Some("html".to_owned())
        }),
        ("oembed gone", |p| {
            named_mut(p, "Everything")
                .properties
                .picture
                .as_mut()
                .unwrap()
                .blip
                .as_mut()
                .unwrap()
                .oembed = None
        }),
        ("trim", |p| {
            media_of(p, "Video embedded").trim.as_mut().unwrap().st = Some("1".to_owned())
        }),
        ("trim gone", |p| media_of(p, "Video embedded").trim = None),
        ("trim made", |p| {
            media_of(p, "Video linked").trim = Some(MediaTrim {
                st: Some("2".to_owned()),
                end: None,
            })
        }),
        ("fade", |p| {
            media_of(p, "Video embedded").fade = Some(MediaFade {
                fade_in: None,
                fade_out: Some("9".to_owned()),
            })
        }),
        ("fade made between", |p| {
            media_of(p, "Audio embedded").fade = Some(MediaFade {
                fade_in: Some("9".to_owned()),
                fade_out: None,
            })
        }),
        ("bookmark changed", |p| {
            media_of(p, "Video embedded").bookmarks.as_mut().unwrap()[0].time = Some("7".to_owned())
        }),
        ("bookmark added", |p| {
            media_of(p, "Video embedded")
                .bookmarks
                .as_mut()
                .unwrap()
                .push(MediaBookmark {
                    name: Some("Three".to_owned()),
                    time: Some("3".to_owned()),
                })
        }),
        ("bookmark removed", |p| {
            media_of(p, "Video embedded")
                .bookmarks
                .as_mut()
                .unwrap()
                .remove(0);
        }),
        ("bookmarks emptied", |p| {
            media_of(p, "Video embedded").bookmarks = Some(Vec::new())
        }),
        ("bookmarks made", |p| {
            media_of(p, "Video linked").bookmarks = Some(vec![MediaBookmark {
                name: Some("m".to_owned()),
                time: None,
            }])
        }),
        ("track changed", |p| {
            media_of(p, "Video embedded")
                .tracks
                .as_mut()
                .unwrap()
                .track_lst
                .as_mut()
                .unwrap()
                .track[0]
                .label = Some("New".to_owned())
        }),
        ("track location", |p| {
            media_of(p, "Video embedded")
                .tracks
                .as_mut()
                .unwrap()
                .display_loc = Some(DisplayLocation::Slide)
        }),
        ("track removed", |p| {
            media_of(p, "Video embedded")
                .tracks
                .as_mut()
                .unwrap()
                .track_lst
                .as_mut()
                .unwrap()
                .track
                .remove(1);
        }),
        ("tracks gone", |p| {
            media_of(p, "Video embedded").tracks = None
        }),
        ("tracks made", |p| {
            media_of(p, "Audio embedded").tracks = Some(TracksInfo {
                display_loc: Some(DisplayLocation::Media),
                track_lst: Some(TrackList {
                    track: vec![MediaTrack {
                        id: Some("{3C8F3A2E-5B6D-4E7F-8A9B-0C1D2E3F4A5B}".to_owned()),
                        label: Some("L".to_owned()),
                        lang: None,
                        embed: Some("rId9".to_owned()),
                        link: None,
                    }],
                }),
            })
        }),
        ("p14 gone", |p| {
            named_mut(p, "Video linked")
                .app
                .as_mut()
                .unwrap()
                .media_extension = None
        }),
        ("p14 made", |p| {
            named_mut(p, "Wave").app.as_mut().unwrap().media_extension =
                Some(Box::new(MediaExtension {
                    embed: Some("rId5".to_owned()),
                    ..MediaExtension::default()
                }))
        }),
        ("video link", |p| {
            named_mut(p, "Video linked").app.as_mut().unwrap().media =
                Some(Media::VideoFile(MediaFile {
                    link: Some("rId4".to_owned()),
                    content_type: None,
                }))
        }),
        ("content type gone", |p| {
            named_mut(p, "Video embedded").app.as_mut().unwrap().media =
                Some(Media::VideoFile(MediaFile {
                    link: Some("rId4".to_owned()),
                    content_type: None,
                }))
        }),
        ("video to audio", |p| {
            named_mut(p, "Video linked").app.as_mut().unwrap().media =
                Some(Media::AudioFile(MediaFile {
                    link: Some("rId6".to_owned()),
                    content_type: None,
                }))
        }),
        ("wave", |p| {
            named_mut(p, "Wave").app.as_mut().unwrap().media =
                Some(Media::WavAudioFile(EmbeddedWavAudioFile {
                    embed: Some("rId8".to_owned()),
                    name: None,
                }))
        }),
        ("cd", |p| {
            named_mut(p, "CD").app.as_mut().unwrap().media = Some(Media::AudioCd(AudioCd {
                st: Some(AudioCdTime {
                    track: Some(9),
                    time: None,
                }),
                end: Some(AudioCdTime {
                    track: Some(10),
                    time: None,
                }),
            }))
        }),
        ("media gone", |p| {
            named_mut(p, "QuickTime").app.as_mut().unwrap().media = None
        }),
        ("media made", |p| {
            named_mut(p, "Everything").app.as_mut().unwrap().media =
                Some(Media::QuickTimeFile(MediaFile {
                    link: Some("rId13".to_owned()),
                    content_type: None,
                }));
        }),
        ("sound", |p| {
            named_mut(p, "Sound")
                .non_visual
                .as_mut()
                .unwrap()
                .hyperlink_click
                .as_mut()
                .unwrap()
                .sound
                .as_mut()
                .unwrap()
                .name = Some("x".to_owned())
        }),
        ("sound gone", |p| {
            named_mut(p, "Sound")
                .non_visual
                .as_mut()
                .unwrap()
                .hyperlink_click
                .as_mut()
                .unwrap()
                .sound = None
        }),
        ("bullet effects", |p| {
            let body = named_mut(p, "Bullet").text_body.as_mut().unwrap();
            let Some(Bullet::Picture(Some(blip))) =
                &mut body.paragraphs[0].properties.as_mut().unwrap().bullet
            else {
                panic!()
            };
            blip.effects.clear();
            blip.cstate = None;
        }),
        ("bullet to char", |p| {
            let body = named_mut(p, "Bullet").text_body.as_mut().unwrap();
            body.paragraphs[0].properties.as_mut().unwrap().bullet =
                Some(Bullet::Char(Some("x".to_owned())));
        }),
    ];
    for (label, change) in changes {
        check_edit(&source, ordinal, *change, label);
    }
}

/// T1F-04: a picture bullet can be made, and changed, not only kept or removed.
#[test]
fn a_picture_bullet_is_made_from_the_model_where_there_was_none() {
    let source = fixture("generated/text-every-property.pptx");
    let original = model(&source);
    for &ordinal in &original.structure.slides {
        let part = original.get(ordinal).unwrap();
        let rel = open(&source)
            .relationships_of(&original.structure.entry(ordinal).unwrap().part)
            .iter()
            .find(|r| r.rel_type.ends_with("/image"))
            .map(|r| r.id.clone());
        let Some(rel) = rel else { continue };
        let target = part.shape_tree().unwrap().descendants().find(|n| {
            n.text_body.as_ref().is_some_and(|b| {
                b.paragraphs.first().is_some_and(|p| {
                    p.properties
                        .as_ref()
                        .is_none_or(|x| !matches!(x.bullet, Some(Bullet::Picture(_))))
                })
            })
        });
        let Some(target) = target else { continue };
        let id = target.id;
        let label = "bullet made";
        let bytes = check_edit(
            &source,
            ordinal,
            |p| {
                let body = p.node_mut(id).unwrap().text_body.as_mut().unwrap();
                let props = body.paragraphs[0]
                    .properties
                    .get_or_insert_with(Default::default);
                props.bullet = Some(Bullet::Picture(Some(Box::new(Blip {
                    embed: Some(rel.clone()),
                    svg: None,
                    ..Blip::default()
                }))));
            },
            label,
        );
        let text = String::from_utf8_lossy(
            &parts(&bytes)[&original.structure.entry(ordinal).unwrap().part],
        )
        .into_owned();
        assert!(text.contains("<a:buBlip><a:blip r:embed="), "{text}");
        return;
    }
    panic!("no deck to make a bullet in");
}

/// `WR-01` for this tier: a picture's fill, its media and its picture bullets written from
/// the model into a node holding none, and read back the same. An unedited save proves
/// nothing about the writer; this runs it, on every deck.
#[test]
fn every_picture_and_media_node_can_be_written_from_the_model() {
    let (mut written, mut alternate) = (0usize, 0usize);
    for (name, source) in all_decks() {
        let original = model(&source);
        let mut pkg = open(&source);
        for entry in &original.structure.parts {
            let Some(part) = original.get(entry.id.part) else {
                continue;
            };
            let targets: Vec<ObjectId> = part
                .shape_tree()
                .into_iter()
                .flat_map(ShapeNode::descendants)
                .filter(|n| {
                    n.properties.picture.is_some()
                        || n.app
                            .as_ref()
                            .is_some_and(|a| a.media.is_some() || a.media_extension.is_some())
                        || matches!(n.properties.fill, Some(Fill::Blip(_)))
                        || n.non_visual.as_ref().is_some_and(|v| {
                            [&v.hyperlink_click, &v.hyperlink_hover]
                                .iter()
                                .any(|l| l.as_ref().is_some_and(|l| l.sound.is_some()))
                        })
                        || bullets_of(n).next().is_some()
                })
                .map(|n| n.id)
                .collect();
            if targets.is_empty() {
                continue;
            }
            let bytes = pkg.read_part(&entry.part).unwrap();
            let rels = pkg.relationships_of(&entry.part).to_vec();
            let strict = pkg.conformance() == casual_slide_ooxml::Conformance::Strict;
            let (limits, import, xml) = (
                SlideLimits::default(),
                ImportLimits::default(),
                OoxmlLimits::default(),
            );
            for id in targets {
                let mut emptied = part.clone();
                let node = emptied.node_mut(id).unwrap();
                node.properties.picture = None;
                if matches!(node.properties.fill, Some(Fill::Blip(_))) {
                    node.properties.fill = None;
                }
                if let Some(app) = &mut node.app {
                    app.media = None;
                    app.media_extension = None;
                }
                if let Some(v) = &mut node.non_visual {
                    for link in [&mut v.hyperlink_click, &mut v.hyperlink_hover]
                        .into_iter()
                        .flatten()
                    {
                        link.sound = None;
                    }
                }
                if let Some(body) = &mut node.text_body {
                    for p in &mut body.paragraphs {
                        if let Some(props) = &mut p.properties
                            && matches!(props.bullet, Some(Bullet::Picture(_)))
                        {
                            props.bullet = None;
                        }
                    }
                }
                let mut tree = XmlTree::parse(&bytes, &entry.part, &xml).unwrap();
                match patch_slide_part(
                    &mut tree,
                    &entry.part,
                    strict,
                    &rels,
                    &emptied,
                    &limits,
                    &import,
                    &xml,
                ) {
                    Ok(_) => {}
                    Err(e) if e.code() == "CS-SLE-0002" => {
                        alternate += 1;
                        continue;
                    }
                    Err(e) => panic!("{name} {}: emptying: {e}", entry.part),
                }
                // From nothing back to what the model says, which `patch_slide_part` reads
                // back and refuses if it differs, and here again.
                patch_slide_part(
                    &mut tree,
                    &entry.part,
                    strict,
                    &rels,
                    part,
                    &limits,
                    &import,
                    &xml,
                )
                .unwrap_or_else(|e| panic!("{name} {}: writing: {e}", entry.part));
                let again = XmlTree::parse(&tree.to_bytes(), &entry.part, &xml).unwrap();
                let mut findings = Findings::new(100_000);
                let read = import_slide_part(
                    &again,
                    &entry.part,
                    &rels,
                    part.kind,
                    part.id.part,
                    &limits,
                    &import,
                    &mut findings,
                    &mut ColorFindings::default(),
                )
                .unwrap();
                assert_eq!(&read, part, "{name} {}", entry.part);
                written += 1;
            }
        }
    }
    eprintln!("written from the model: {written}; in an alternate branch: {alternate}");
    assert!(written >= 40, "{written}");
}

fn bullets_of(n: &ShapeNode) -> impl Iterator<Item = &Blip> {
    n.text_body
        .iter()
        .flat_map(|b| b.paragraphs.iter())
        .filter_map(|p| match p.properties.as_ref()?.bullet.as_ref()? {
            Bullet::Picture(Some(blip)) => Some(&**blip),
            _ => None,
        })
}

/// `WR-01`, in the deck: the whole of the fixture's every-reference slide written from
/// its model, and F4 clean. Its media nodes emptied, written again, saved and validated.
#[test]
fn the_every_reference_slide_written_from_its_model_validates() {
    let source = fixture(EVERYTHING);
    f4_clean(&source, "as generated");
    let original = model(&source);
    let ordinal = original.structure.slides[0];
    let part = original.get(ordinal).unwrap();
    let mut pkg = open(&source);
    let rels = pkg.relationships_of(SLIDE1).to_vec();
    let bytes = pkg.read_part(SLIDE1).unwrap();
    let (limits, import, xml) = (
        SlideLimits::default(),
        ImportLimits::default(),
        OoxmlLimits::default(),
    );
    let mut emptied = part.clone();
    for name in [
        "Everything",
        "Tiled",
        "Video embedded",
        "Video linked",
        "Audio embedded",
        "Audio linked",
        "Wave",
        "CD",
        "QuickTime",
    ] {
        let node = named_mut(&mut emptied, name);
        node.locks = None;
        node.properties.picture = None;
        node.prefer_relative_resize = None;
        let app = node.app.as_mut().unwrap();
        app.media = None;
        app.media_extension = None;
    }
    named_mut(&mut emptied, "Bullet")
        .text_body
        .as_mut()
        .unwrap()
        .paragraphs[0]
        .properties
        .as_mut()
        .unwrap()
        .bullet = None;
    let mut tree = XmlTree::parse(&bytes, SLIDE1, &xml).unwrap();
    patch_slide_part(
        &mut tree, SLIDE1, false, &rels, &emptied, &limits, &import, &xml,
    )
    .unwrap();
    let emptied_text = String::from_utf8_lossy(&tree.to_bytes()).into_owned();
    assert!(!emptied_text.contains("p14:media"));
    for name in [
        "svgBlip",
        "oEmbedUrl",
        "picLocksAutoForOEmbed",
        "picLocks",
        "noCrop",
        "alphaBiLevel",
    ] {
        assert!(!emptied_text.contains(name), "{name} was not taken away");
    }
    patch_slide_part(
        &mut tree, SLIDE1, false, &rels, part, &limits, &import, &xml,
    )
    .unwrap();
    // Made again from the model: each of these from nothing, and every recolour effect,
    // since a blip writes those and no others.
    let written = String::from_utf8_lossy(&tree.to_bytes()).into_owned();
    for name in [
        "svgBlip",
        "oEmbedUrl",
        "picLocksAutoForOEmbed",
        "picLocks",
        "noCrop",
        "alphaBiLevel",
        "alphaCeiling",
        "alphaFloor",
        "alphaInv",
        "alphaMod",
        "alphaModFix",
        "alphaRepl",
        "biLevel",
        "blur",
        "clrChange",
        "clrRepl",
        "duotone",
        "fillOverlay",
        "grayscl",
        "hsl",
        "lum",
        "tint",
    ] {
        assert!(written.contains(name), "{name} was not written");
    }
    // A whole deck with the part so written.
    let mut edits = casual_slide_package::PartEdits::default();
    edits.replace(SLIDE1.to_owned(), tree.to_bytes());
    let out = pkg.save_with(&edits).unwrap();
    assert_eq!(
        model(&out).get(ordinal),
        Some(part),
        "read back through a package"
    );
    f4_clean(&out, "written from the model");
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("media-written");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("media-every-reference.pptx"), &out).unwrap();
    // And the file's own bytes are not needed for what the model holds.
    assert_ne!(
        tree.to_bytes(),
        bytes,
        "this run must have rewritten something"
    );
}

/// The tier's gate: import, export, import. Every deck with a picture or media: an
/// unedited save is the input, and reading twice gives one answer.
#[test]
fn every_deck_is_a_fixed_point_and_an_unedited_save_is_the_input() {
    let mut pictures_seen = 0;
    for (name, source) in all_decks() {
        let first = model(&source);
        let wanted: Vec<&SlidePart> = first.parts.iter().flatten().collect();
        pictures_seen += wanted.iter().map(|p| pictures(p).len()).sum::<usize>();
        let mut pkg = open(&source);
        let outcome = save_slide_parts(
            &mut pkg,
            &first.structure,
            &wanted,
            &SlideLimits::default(),
            &ImportLimits::default(),
        )
        .unwrap();
        assert_eq!(outcome.bytes, source, "{name}");
        assert!(outcome.changed_parts.is_empty(), "{name}");
        assert_eq!(model(&outcome.bytes), first, "{name}");
    }
    assert!(pictures_seen >= 30, "{pictures_seen}");
}

/// `WR-01` on real decks: every picture on a slide written from the model at once, saved
/// as a deck, and compared under F4 with the deck it came from. Kept on disk for the
/// Open XML SDK to see, since it validates differently (`tools/ooxml-sdk`).
#[test]
fn real_decks_with_their_pictures_written_from_the_model_validate() {
    let mut decks_written = 0;
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("media-written");
    std::fs::create_dir_all(&dir).unwrap();
    for (name, source) in decks("real").into_iter().chain(decks("producers")) {
        let original = model(&source);
        let mut pkg = open(&source);
        let mut edits = casual_slide_package::PartEdits::default();
        for entry in &original.structure.parts {
            let Some(part) = original.get(entry.id.part) else {
                continue;
            };
            let targets = pictures(part);
            if targets.is_empty() {
                continue;
            }
            let bytes = pkg.read_part(&entry.part).unwrap();
            let rels = pkg.relationships_of(&entry.part).to_vec();
            let strict = pkg.conformance() == casual_slide_ooxml::Conformance::Strict;
            let (limits, import, xml) = (
                SlideLimits::default(),
                ImportLimits::default(),
                OoxmlLimits::default(),
            );
            let mut emptied = part.clone();
            for (id, _) in &targets {
                emptied.node_mut(*id).unwrap().properties.picture = None;
            }
            let mut tree = XmlTree::parse(&bytes, &entry.part, &xml).unwrap();
            match patch_slide_part(
                &mut tree,
                &entry.part,
                strict,
                &rels,
                &emptied,
                &limits,
                &import,
                &xml,
            ) {
                Ok(_) => {}
                Err(e) if e.code() == "CS-SLE-0002" => continue,
                Err(e) => panic!("{name}: {e}"),
            }
            patch_slide_part(
                &mut tree,
                &entry.part,
                strict,
                &rels,
                part,
                &limits,
                &import,
                &xml,
            )
            .unwrap_or_else(|e| panic!("{name}: {e}"));
            edits.replace(entry.part.clone(), tree.to_bytes());
        }
        if edits.is_empty() {
            continue;
        }
        let out = pkg.save_with(&edits).unwrap();
        assert_eq!(model(&out), original, "{name}: read back through a package");
        f4_validates(&source, &out, &name);
        let file = name.replace('/', "-");
        std::fs::write(dir.join(file), &out).unwrap();
        decks_written += 1;
    }
    eprintln!("decks with pictures written from the model: {decks_written}");
    assert!(decks_written >= 4, "{decks_written}");
}

/// An external target stays exactly as it was through any edit, in the relationships part
/// too: nothing rewrites, resolves or follows one.
#[test]
fn an_edit_leaves_external_targets_byte_for_byte() {
    let source = fixture(EVERYTHING);
    let ordinal = model(&source).structure.slides[0];
    let edited = check_edit(
        &source,
        ordinal,
        |p| {
            media_of(p, "Video linked").trim = Some(MediaTrim {
                st: Some("2".to_owned()),
                end: None,
            })
        },
        "external",
    );
    let (old, new) = (parts(&source), parts(&edited));
    let rels = "ppt/slides/_rels/slide1.xml.rels";
    assert_eq!(old[rels], new[rels]);
    assert!(String::from_utf8_lossy(&new[rels]).contains("https://example.invalid/clip.mp4"));
    let (catalog, _) = catalog_media(&mut open(&edited), &MediaLimits::default());
    assert_eq!(catalog.external.len(), 14);
}

#[test]
fn a_blip_fill_on_a_shape_holds_the_same_things_as_a_picture() {
    let source = fixture(EVERYTHING);
    let ordinal = model(&source).structure.slides[0];
    check_edit(
        &source,
        ordinal,
        |p| {
            let node = named_mut(p, "Filled");
            let Some(Fill::Blip(f)) = &mut node.properties.fill else {
                panic!()
            };
            let f: &mut BlipFill = f;
            f.source_rect = new_crop();
            f.blip.as_mut().unwrap().svg = Some(SvgBlip {
                embed: Some("rId3".to_owned()),
                link: None,
            });
        },
        "shape fill",
    );
}
