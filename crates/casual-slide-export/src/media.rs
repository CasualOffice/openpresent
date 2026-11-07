//! Patch audio and video references into `nvPr`, and replace a picture or a clip in the
//! model (ADR-013, Tier 1H).
//!
//! Replacing media changes relationship ids in a part and says what must follow in its
//! `.rels`: [`MediaReplacement`] is the plan, and [`MediaReplacement::parts_released`]
//! says which parts lose their last referrer. Writing relationship parts, and removing
//! parts, is the writer's (Tier 1N); nothing here touches them, or a media part's bytes,
//! so a shared part cannot be overwritten from here (`INV-SP-02`, `INV-SP-04`).

use std::collections::BTreeMap;
use std::fmt;

use casual_slide_import::media::{P14, P173};
use casual_slide_import::properties::{Elem, ImportLimits, ns};
use casual_slide_import::slide::SlideLimits;
use casual_slide_model::fill::{Blip, Fill};
use casual_slide_model::media::{
    AudioCdTime, EmbeddedWav, Media, MediaCatalog, MediaExtension, MediaFile, TracksInfo,
};
use casual_slide_model::shape::{AppNonVisual, ObjectId, ShapeNode};
use casual_slide_model::slide::SlidePart;
use casual_slide_model::text::Bullet;
use casual_slide_ooxml::{OoxmlError, OoxmlLimits, Relationship, TargetMode, XmlTree};

use crate::drawing::{Node, Space, insert_new, landmark, patch};
use crate::properties::{ExportError, Writer};
use crate::slide::{SlideExportError, patch_slide_part};

/// `MS-PPTX` "Media Extensions".
const MEDIA_EXTENSION_URI: &str = "{DAA4B4D4-6D71-4841-9C94-3DE7FCFB9230}";
/// `MS-PPTX` "TracksInfo Extensions".
const TRACKS_EXTENSION_URI: &str = "{3AFAAA56-56D3-431D-BCD4-E75A35582382}";

fn file_node(local: &'static str, f: &MediaFile) -> Node {
    Node::a(local)
        .rel("link", f.link.as_ref())
        .lex("contentType", f.content_type.as_ref())
}

fn cd_time_node(local: &'static str, t: &AudioCdTime) -> Node {
    Node::a(local)
        .lex("track", t.track.as_ref())
        .lex("time", t.time.as_ref())
}

/// `a:CT_EmbeddedWAVAudioFile`, as `wavAudioFile` and `snd`.
pub(crate) fn wav_node(local: &'static str, w: &EmbeddedWav) -> Node {
    Node::a(local)
        .rel("embed", w.embed.as_ref())
        .lex("name", w.name.as_ref())
}

fn media_node(m: &Media) -> Node {
    match m {
        Media::AudioCd(cd) => Node::a("audioCd")
            .opt(cd.start.as_ref().map(|t| cd_time_node("st", t)))
            .opt(cd.end.as_ref().map(|t| cd_time_node("end", t))),
        Media::WavAudioFile(w) => wav_node("wavAudioFile", w),
        Media::AudioFile(f) => file_node("audioFile", f),
        Media::VideoFile(f) => file_node("videoFile", f),
        Media::QuickTimeFile(f) => file_node("quickTimeFile", f),
    }
}

/// `p14:media`, its `extLst` apart: [`patch_extension`] writes that.
fn extension_node(m: &MediaExtension) -> Node {
    let trim = m.trim.as_ref().map(|t| {
        Node::new(Space::P14, "trim")
            .lex("st", t.start.as_ref())
            .lex("end", t.end.as_ref())
    });
    let fade = m.fade.as_ref().map(|f| {
        Node::new(Space::P14, "fade")
            .lex("in", f.fade_in.as_ref())
            .lex("out", f.fade_out.as_ref())
    });
    let bookmarks = m.bookmarks.as_ref().map(|list| {
        Node::new(Space::P14, "bmkLst").list(list.iter().map(|b| {
            Node::new(Space::P14, "bmk")
                .lex("name", b.name.as_ref())
                .lex("time", b.time.as_ref())
        }))
    });
    Node::new(Space::P14, "media")
        .rel("embed", m.embed.as_ref())
        .rel("link", m.link.as_ref())
        .opt(trim)
        .opt(fade)
        .opt(bookmarks)
}

fn tracks_node(t: &TracksInfo) -> Node {
    let list = t.tracks.as_ref().map(|list| {
        Node::new(Space::P173, "trackLst").list(list.iter().map(|t| {
            Node::new(Space::P173, "track")
                .lex("id", t.id.as_ref())
                .lex("label", t.label.as_ref())
                .lex("lang", t.lang.as_ref())
                .rel("embed", t.embed.as_ref())
                .rel("link", t.link.as_ref())
        }))
    });
    Node::new(Space::P173, "tracksInfo")
        .lex("displayLoc", t.display_location.as_ref())
        .opt(list)
}

/// Where an element keeps its extension list. `nvPr` has `p:extLst`. `p14:media` has
/// `p14:extLst`, by its schema's `elementFormDefault`, and `p:ext` inside either.
#[derive(Clone, Copy)]
struct ListSpace {
    uris: &'static [&'static str],
    prefix: &'static str,
}

const NV_PR_LIST: ListSpace = ListSpace {
    uris: ns::PML,
    prefix: "p",
};
const MEDIA_LIST: ListSpace = ListSpace {
    uris: P14,
    prefix: "p14",
};

/// An item in `el`'s extension list, with its holders.
fn find_item(
    w: &Writer<'_>,
    el: &Elem,
    space: ListSpace,
    uris: &[&str],
    local: &str,
) -> Result<Option<(Elem, Elem, Elem)>, ExportError> {
    let Some(list) = el.first_child(w.tree, w.part, space.uris, "extLst")? else {
        return Ok(None);
    };
    for ext in list.children(w.tree, w.part)? {
        if !ext.is(ns::PML, "ext") {
            continue;
        }
        for item in ext.children(w.tree, w.part)? {
            if item.is(uris, local) {
                return Ok(Some((list, ext, item)));
            }
        }
    }
    Ok(None)
}

/// Bring one `p:ext` entry from `before` to `after`. Another's entry is left where it is,
/// and keeps the holders.
fn patch_extension(
    w: &mut Writer<'_>,
    el: &Elem,
    space: ListSpace,
    uris: &[&str],
    uri: &str,
    before: Option<&Node>,
    after: Option<&Node>,
) -> Result<(), ExportError> {
    if before == after {
        return Ok(());
    }
    let Some(local) = after.or(before).map(Node::local) else {
        return Ok(());
    };
    let found = find_item(w, el, space, uris, local)?;
    let Some(after) = after else {
        if let Some((list, ext, item)) = found {
            w.tree.remove_child(ext.id, item.id);
            w.changes += 1;
            if ext.children(w.tree, w.part)?.is_empty() {
                w.tree.remove_child(list.id, ext.id);
                if list.children(w.tree, w.part)?.is_empty() {
                    w.tree.remove_child(el.id, list.id);
                }
            }
        }
        return Ok(());
    };
    if let Some((_, _, item)) = found {
        return patch(w, &item, before, after);
    }
    let list = match el.first_child(w.tree, w.part, space.uris, "extLst")? {
        Some(list) => list,
        None => {
            let list = w.new_child(el, space.uris, space.prefix, "extLst")?;
            let anchor = landmark(w, el, "extLst")?.map(|l| l.id);
            w.tree.insert_child(el.id, anchor, list.id);
            list
        }
    };
    let ext = w.new_child(&list, ns::PML, "p", "ext")?;
    w.tree.insert_child(list.id, None, ext.id);
    w.attr(&ext, "uri", None::<&String>, Some(&uri.to_owned()))?;
    insert_new(w, &ext, None, after)
}

/// What the schema requires of a media element, for an edit to supply. A file that
/// arrived without it is reported on reading, and left as it is until someone edits it.
fn media_is_complete(media: &Media) -> bool {
    match media {
        Media::AudioCd(cd) => [cd.start, cd.end]
            .iter()
            .all(|t| t.is_some_and(|t| t.track.is_some())),
        Media::WavAudioFile(w) => w.embed.is_some(),
        Media::AudioFile(f) | Media::VideoFile(f) | Media::QuickTimeFile(f) => f.link.is_some(),
    }
}

fn tracks_are_complete(info: &TracksInfo) -> bool {
    info.display_location.is_some()
        && info
            .tracks
            .iter()
            .flatten()
            .all(|t| t.id.is_some() && t.label.is_some())
}

/// Bring `nvPr`'s `EG_Media` child and `p14:media` from `b` to `a`.
pub(crate) fn patch_app_media(
    w: &mut Writer<'_>,
    el: &Elem,
    b: &AppNonVisual,
    a: &AppNonVisual,
) -> Result<(), ExportError> {
    let incomplete = |what| ExportError::UnsupportedEdit {
        what,
        part: w.part.to_owned(),
    };
    if b.media != a.media && !a.media.as_ref().is_none_or(media_is_complete) {
        return Err(incomplete("media leaving out what the schema requires"));
    }
    let tracks = |app: &AppNonVisual| app.media_extension.as_ref().and_then(|m| m.tracks.clone());
    let wanted_tracks = tracks(a);
    if tracks(b) != wanted_tracks && !wanted_tracks.as_ref().is_none_or(tracks_are_complete) {
        return Err(incomplete("a track leaving out what the schema requires"));
    }
    let holder = |app: &AppNonVisual| Node::a("nvPr").opt(app.media.as_ref().map(media_node));
    if b.media != a.media {
        patch(w, el, Some(&holder(b)), &holder(a))?;
    }
    let (be, ae) = (b.media_extension.as_deref(), a.media_extension.as_deref());
    if be == ae {
        return Ok(());
    }
    let (bn, an) = (be.map(extension_node), ae.map(extension_node));
    patch_extension(
        w,
        el,
        NV_PR_LIST,
        P14,
        MEDIA_EXTENSION_URI,
        bn.as_ref(),
        an.as_ref(),
    )?;
    let Some(after) = ae else {
        return Ok(());
    };
    let Some((_, _, item)) = find_item(w, el, NV_PR_LIST, P14, "media")? else {
        return Ok(());
    };
    let (bt, at) = (
        be.and_then(|m| m.tracks.as_ref()).map(tracks_node),
        after.tracks.as_ref().map(tracks_node),
    );
    patch_extension(
        w,
        &item,
        MEDIA_LIST,
        P173,
        TRACKS_EXTENSION_URI,
        bt.as_ref(),
        at.as_ref(),
    )
}

/// Why media could not be replaced. Nothing was changed.
#[derive(Debug)]
#[non_exhaustive]
pub enum MediaEditError {
    /// The node does not hold this reference, or the part has no such node.
    NoSuchReference {
        /// The part.
        part: String,
    },
    /// The reference is half of a blip's SVG and its fallback, which would then show two
    /// different pictures. Replace both, or clear the SVG in the model first.
    StaleSvgPair {
        /// The part.
        part: String,
    },
    /// Writing the part refused.
    Slide(SlideExportError),
}

impl MediaEditError {
    /// The stable diagnostic code.
    pub fn code(&self) -> &'static str {
        match self {
            Self::NoSuchReference { .. } => "CS-MDE-0001",
            Self::StaleSvgPair { .. } => "CS-MDE-0002",
            Self::Slide(err) => err.code(),
        }
    }
}

impl fmt::Display for MediaEditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = self.code();
        match self {
            Self::NoSuchReference { part } => {
                write!(f, "[{code}] {part:?}: the node holds no such reference")
            }
            Self::StaleSvgPair { part } => write!(
                f,
                "[{code}] {part:?}: the reference is half of an SVG and its fallback"
            ),
            Self::Slide(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for MediaEditError {}

impl From<SlideExportError> for MediaEditError {
    fn from(err: SlideExportError) -> Self {
        Self::Slide(err)
    }
}

/// What a replacement should reach.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewMedia {
    /// `Type`, in the form the deck's other relationships use (`INV-RI-09`).
    pub rel_type: String,
    /// `Target`.
    pub target: String,
    /// `TargetMode`.
    pub mode: TargetMode,
}

/// What a replacement did to a part, and what its `.rels` must do to follow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaReplacement {
    /// The id the node now names.
    pub rel_id: String,
    /// A relationship to add, if none reached the target (`INV-RI-10`).
    pub added: Option<Relationship>,
    /// The old relationship, if nothing in the part names it any more.
    pub released: Option<String>,
    /// The part's relationships afterwards. Existing ids are never renumbered.
    pub relationships: Vec<Relationship>,
}

impl MediaReplacement {
    /// Which media parts lose their last relationship from anywhere in the deck, for the
    /// writer to remove. A part another slide still shares is not among them.
    pub fn parts_released(&self, catalog: &MediaCatalog, part: &str) -> Vec<String> {
        let Some(released) = &self.released else {
            return Vec::new();
        };
        catalog
            .reached_by(part, released)
            .filter(|p| !p.outlives(part, released))
            .map(|p| p.part.clone())
            .into_iter()
            .collect()
    }
}

/// `rId` plus one above the highest numeric `rIdN`, checked against every id there
/// (`INV-RI-03`).
fn next_rel_id(rels: &[Relationship]) -> String {
    let mut n = rels
        .iter()
        .filter_map(|r| r.id.strip_prefix("rId")?.parse::<u64>().ok())
        .max()
        .unwrap_or(0);
    loop {
        n = n.saturating_add(1);
        let id = format!("rId{n}");
        if !rels.iter().any(|r| r.id == id) || n == u64::MAX {
            return id;
        }
    }
}

/// Swap a reference in one blip. An SVG pair is refused unless both halves move together.
fn retarget_blip(blip: &mut Blip, old: &str, new: &str) -> Result<usize, ()> {
    let names = |slot: &Option<String>| slot.as_deref() == Some(old);
    let main = names(&blip.embed) || names(&blip.link);
    let svg = blip
        .svg
        .as_ref()
        .is_some_and(|s| names(&s.embed) || names(&s.link));
    if blip.svg.is_some() && main != svg {
        return Err(());
    }
    let mut count = 0;
    let mut swap = |slot: &mut Option<String>| {
        if slot.as_deref() == Some(old) {
            *slot = Some(new.to_owned());
            count += 1;
        }
    };
    swap(&mut blip.embed);
    swap(&mut blip.link);
    if let Some(s) = &mut blip.svg {
        swap(&mut s.embed);
        swap(&mut s.link);
    }
    Ok(count)
}

/// Swap a reference everywhere a node states one: its picture, fill, media, sounds and
/// picture bullets. Anywhere else (a run's fill, an `imgLayer`, retained XML) keeps it,
/// and the count of references in the tree then keeps the relationship.
fn retarget_node(node: &mut ShapeNode, old: &str, new: &str) -> Result<usize, ()> {
    let mut count = 0;
    let slot = |value: &mut Option<String>| {
        if value.as_deref() == Some(old) {
            *value = Some(new.to_owned());
            1
        } else {
            0
        }
    };
    let mut blips: Vec<&mut Blip> = Vec::new();
    if let Some(p) = &mut node.properties.picture {
        blips.extend(p.blip.as_mut());
    }
    if let Some(Fill::Blip(f)) = &mut node.properties.fill {
        blips.extend(f.blip.as_mut());
    }
    if let Some(body) = &mut node.text_body {
        for p in &mut body.paragraphs {
            if let Some(Bullet::Picture(Some(blip))) =
                p.properties.as_mut().and_then(|p| p.bullet.as_mut())
            {
                blips.push(blip);
            }
        }
    }
    for blip in blips {
        count += retarget_blip(blip, old, new)?;
    }
    if let Some(nv) = &mut node.non_visual {
        for link in [&mut nv.hyperlink_click, &mut nv.hyperlink_hover]
            .into_iter()
            .flatten()
        {
            if let Some(sound) = &mut link.sound {
                count += slot(&mut sound.embed);
            }
        }
    }
    if let Some(app) = &mut node.app {
        match &mut app.media {
            Some(Media::WavAudioFile(w)) => count += slot(&mut w.embed),
            Some(Media::AudioFile(f) | Media::VideoFile(f) | Media::QuickTimeFile(f)) => {
                count += slot(&mut f.link);
            }
            Some(Media::AudioCd(_)) | None => {}
        }
        if let Some(m) = &mut app.media_extension {
            count += slot(&mut m.embed);
            count += slot(&mut m.link);
            for t in m
                .tracks
                .iter_mut()
                .flat_map(|t| t.tracks.iter_mut())
                .flatten()
            {
                count += slot(&mut t.embed);
                count += slot(&mut t.link);
            }
        }
    }
    Ok(count)
}

/// How many times the part's XML names each relationship, in any element and any
/// `mc:AlternateContent` branch, retained content included. What the model cannot see
/// counts: this is why a relationship is released from here and not from the model.
pub fn relationship_references(
    tree: &XmlTree,
    part: &str,
) -> Result<BTreeMap<String, usize>, OoxmlError> {
    let mut out: BTreeMap<String, usize> = BTreeMap::new();
    let mut stack = vec![Elem::root(tree, part)?];
    while let Some(el) = stack.pop() {
        for (name, value) in tree.attributes(el.id, part)? {
            let Some((prefix, _)) = name.split_once(':') else {
                continue;
            };
            if prefix != "xmlns"
                && el
                    .scope
                    .uri(prefix)
                    .is_some_and(|uri| ns::REL.contains(&uri))
            {
                let entry = out.entry(value).or_default();
                *entry = entry.saturating_add(1);
            }
        }
        stack.extend(el.children(tree, part)?);
    }
    Ok(out)
}

/// Replace what a node's reference reaches, in `model` and in `tree`, and say what the
/// part's relationships must become. The part's `.rels` is the writer's to write, and
/// until it does, the id named here resolves only in [`MediaReplacement::relationships`].
#[allow(clippy::too_many_arguments)]
pub fn replace_media(
    tree: &mut XmlTree,
    part: &str,
    strict: bool,
    rels: &[Relationship],
    model: &mut SlidePart,
    node: ObjectId,
    old_id: &str,
    new: &NewMedia,
    limits: &SlideLimits,
    import_limits: &ImportLimits,
    xml_limits: &OoxmlLimits,
) -> Result<MediaReplacement, MediaEditError> {
    let reuse = rels
        .iter()
        .find(|r| r.rel_type == new.rel_type && r.target == new.target && r.mode == new.mode);
    let added = match reuse {
        Some(_) => None,
        None => Some(Relationship {
            id: next_rel_id(rels),
            rel_type: new.rel_type.clone(),
            target: new.target.clone(),
            mode: new.mode,
        }),
    };
    let rel_id = reuse
        .map(|r| r.id.clone())
        .or_else(|| added.as_ref().map(|r| r.id.clone()))
        .unwrap_or_default();
    let mut wanted = model.clone();
    let no_reference = || MediaEditError::NoSuchReference {
        part: part.to_owned(),
    };
    let target = wanted.node_mut(node).ok_or_else(no_reference)?;
    let count =
        retarget_node(target, old_id, &rel_id).map_err(|()| MediaEditError::StaleSvgPair {
            part: part.to_owned(),
        })?;
    if count == 0 {
        return Err(no_reference());
    }
    let mut relationships = rels.to_vec();
    relationships.extend(added.clone());
    patch_slide_part(
        tree,
        part,
        strict,
        &relationships,
        &wanted,
        limits,
        import_limits,
        xml_limits,
    )?;
    let names = relationship_references(tree, part)
        .map_err(|e| MediaEditError::Slide(SlideExportError::from(e)))?;
    let released = (old_id != rel_id && !names.contains_key(old_id)).then(|| old_id.to_owned());
    if let Some(id) = &released {
        relationships.retain(|r| r.id != *id);
    }
    *model = wanted;
    Ok(MediaReplacement {
        rel_id,
        added,
        released,
        relationships,
    })
}
