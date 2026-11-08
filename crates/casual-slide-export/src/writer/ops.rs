//! File-level operations: what Phase 4's ops stand on (`T1N-04`).

use std::collections::BTreeSet;

use casual_slide_ooxml::{ElementId, PresentationPackage, Relationship, TargetMode, XmlTree};

use super::clone::ClonePolicy;
use super::draft::{Draft, rel_kind, set_attr, sibling_name};
use super::error::WriterError;
use super::names::{key, target_for};
use super::notes::{check_text, new_slide_xml};
use super::plan::SavePlan;
use super::preflight::is_section_guid;
use super::view::{pml_child, pml_children, prefix_is_relationships, rel_attribute};
use crate::media::{MediaReplacement, NewMedia};

const PML: &str = "application/vnd.openxmlformats-officedocument.presentationml.";
const AUTHORS_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.presentationml.commentAuthors+xml";

/// Names a slide. An index is read when the operation runs, so it sees earlier operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlideRef {
    /// A position in `sldIdLst`.
    Index(usize),
    /// A slide id.
    Id(u64),
    /// A slide part.
    Part(String),
}

/// One file-level change. Each is applied in plan order, on what earlier ones left.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Op {
    /// Add a slide with empty copies of its layout's placeholders. `position` defaults
    /// to last; `layout` to the layout of the slide before it, or the first master's first.
    AddSlide {
        /// Where in `sldIdLst`.
        position: Option<usize>,
        /// A layout part.
        layout: Option<String>,
    },
    /// Copy a slide and what belongs to it, placing the copy just after it.
    DuplicateSlide {
        /// Which.
        slide: SlideRef,
    },
    /// Delete a slide, with its notes, comments and entries in custom shows and sections.
    DeleteSlide {
        /// Which.
        slide: SlideRef,
    },
    /// Move a slide. It joins the section of the slide now before it.
    MoveSlide {
        /// Which.
        slide: SlideRef,
        /// Its new position.
        position: usize,
    },
    /// Add notes to a slide with none, creating the notes master if the deck has none.
    AddNotes {
        /// Which.
        slide: SlideRef,
        /// The notes' text.
        text: String,
    },
    /// Remove a slide's notes.
    RemoveNotes {
        /// Which.
        slide: SlideRef,
    },
    /// Add a picture to a slide. Bytes matching a media part reuse it. The user's file
    /// name is not asked for, and so cannot reach a part name.
    AddPicture {
        /// Which slide.
        slide: SlideRef,
        /// The image.
        bytes: Vec<u8>,
        /// Its extension, with no dot.
        extension: String,
        /// Its media type.
        content_type: String,
    },
    /// Remove a picture by its `cNvPr/@id`, and its relationship if nothing else there
    /// uses it. The media part goes only with its last referrer.
    RemovePicture {
        /// Which slide.
        slide: SlideRef,
        /// The shape.
        shape_id: u32,
    },
    /// Remove any shape (`sp`, `pic`, `grpSp`, `graphicFrame`, `cxnSp`, `contentPart`)
    /// by its `cNvPr/@id`, with every relationship only it used; a shape inside
    /// `mc:AlternateContent` takes the whole choice with it. Refused for a shape an
    /// animation targets, or an id two shapes share (`VS-06`).
    RemoveShape {
        /// Which slide.
        slide: SlideRef,
        /// The shape.
        shape_id: u32,
    },
    /// Move a top-level shape of the shape tree to place `index` among its shapes, in
    /// z-order, by its `cNvPr/@id` (`UX-006`, Bring Forward, Send Backward). The element
    /// moves whole, with its `mc:AlternateContent` where it is in one; nothing else
    /// changes. A place past the last puts it last.
    MoveShape {
        /// Which slide.
        slide: SlideRef,
        /// The shape.
        shape_id: u32,
        /// Its new place among the tree's shapes, from 0 (the back).
        index: usize,
    },
    /// Point one relationship of a slide at new media. Other referrers keep the old
    /// part (`INV-SP-04`).
    ReplaceMedia {
        /// Which slide.
        slide: SlideRef,
        /// The relationship.
        rel_id: String,
        /// The new bytes.
        bytes: Vec<u8>,
        /// Their extension.
        extension: String,
        /// Their media type.
        content_type: String,
    },
    /// Save what [`crate::media::replace_media`] planned for one node: the part's new
    /// XML, the relationship it names, and the media behind it. The new id is the one the
    /// XML already holds. Build with [`media_replacement_op`].
    ApplyMediaReplacement {
        /// The part the node is in.
        part: String,
        /// That part's bytes after the replacement.
        xml: Vec<u8>,
        /// The relationship to add, if none reached the media.
        added: Option<Relationship>,
        /// The relationship nothing in the part names any more.
        released: Option<String>,
        /// The media's bytes. None for a link, or for a relationship already there.
        media: Option<MediaBytes>,
    },
    /// Add a layout to a master, as a copy of a layout.
    AddLayout {
        /// The master part.
        master: String,
        /// The layout part to copy.
        copy_of: String,
    },
    /// Remove a layout no slide uses, and not the last.
    RemoveLayout {
        /// The layout part.
        layout: String,
    },
    /// Add a master, as a copy of a master, with its own layouts and theme.
    AddMaster {
        /// The master part to copy.
        copy_of: String,
    },
    /// Remove a master none of whose layouts is in use, and not the last.
    RemoveMaster {
        /// The master part.
        master: String,
    },
    /// Give a master's theme these bytes. A theme with another referrer is copied first
    /// (`INV-SP-04`), and the presentation's own theme follows the first master's.
    EditTheme {
        /// The master part.
        master: String,
        /// The whole patched theme part, as `XmlTree::to_bytes` gives it.
        bytes: Vec<u8>,
    },
    /// Add a section, with an id supplied by the host. `at` splits the section holding
    /// that slide; with `None`, the new section is empty and last. A deck with no list
    /// gets one, holding every slide in this section.
    AddSection {
        /// The display name.
        name: String,
        /// A brace-wrapped upper-case GUID, never sampled here.
        id: String,
        /// The first slide of the new section.
        at: Option<SlideRef>,
    },
    /// Remove a section. Its slides join the one before it, or else the one after it.
    RemoveSection {
        /// The section id.
        id: String,
    },
    /// Move a slide into a named section, and `sldIdLst` with it.
    MoveSlideToSection {
        /// Which slide.
        slide: SlideRef,
        /// The section id.
        section: String,
        /// Its position in that section.
        index: usize,
    },
    /// Add a comment author, with the next free id.
    AddCommentAuthor {
        /// The display name.
        name: String,
        /// The initials.
        initials: String,
    },
    /// Add a custom show of these slides, with the next consecutive id (`INV-ID-05`).
    AddCustomShow {
        /// The display name.
        name: String,
        /// Which slides, in order.
        slides: Vec<SlideRef>,
    },
    /// Copy a shape within its slide, with new ids, its animation if whole clicks animate
    /// only it, and a new `a16:creationId` if it has one.
    CopyShape {
        /// Which slide.
        slide: SlideRef,
        /// The shape's `cNvPr/@id`.
        shape_id: u32,
        /// The new `creationId`, supplied because randomness is never sampled here.
        creation_id: Option<String>,
    },
    /// Insert a new, empty shape into a slide's shape tree at `index` among its shapes,
    /// with this `cNvPr/@id` and name: a later patch of the slide part brings it to its
    /// model, so the one patch path writes everything in it (`VS-08`). A picture names
    /// its image's bytes, which reuse a media part holding the same bytes. Refused for an
    /// id the slide already uses.
    InsertShape {
        /// Which slide.
        slide: SlideRef,
        /// Its place among the shape tree's shapes, in z-order; past the end is last.
        index: usize,
        /// What it is.
        shape: NewShape,
        /// Its `cNvPr/@id`.
        shape_id: u32,
        /// Its `cNvPr/@name`.
        name: String,
    },
    /// Change the file's type to this extension's: `pptx`, `ppsx`, `potx`, `pptm`, `ppsm`
    /// or `potm`. One `Override` changes. VBA in a macro-free type needs [`Op::DropVba`].
    Retype {
        /// The extension, with no dot.
        extension: String,
    },
    /// Drop the VBA project: the host has the user's confirmation.
    DropVba,
    /// Attach a new part to a source, with a fresh name in `dir`. For parts the writer
    /// has no operation of its own for.
    AttachPart {
        /// The part that will hold the relationship.
        source: String,
        /// The relationship's kind: `tags`, or a whole type URI.
        kind: String,
        /// The folder.
        dir: String,
        /// The name before its number.
        stem: String,
        /// The extension, which may be empty.
        extension: String,
        /// The content type.
        content_type: String,
        /// The bytes.
        bytes: Vec<u8>,
    },
}

/// What [`Op::InsertShape`] inserts.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum NewShape {
    /// `p:sp`: a text box or a preset shape.
    Shape,
    /// `p:cxnSp`: a line or an arrow.
    Connector,
    /// `p:pic`, stretched over its box.
    Picture(MediaBytes),
    /// `p:graphicFrame` holding a table of these column widths and row heights, EMU,
    /// each cell an empty paragraph (`VS-11`). Its look and text are the patch's.
    Table {
        /// `gridCol/@w`, one a column.
        columns: Vec<i64>,
        /// `tr/@h`, one a row.
        rows: Vec<i64>,
    },
    /// `p:graphicFrame` holding a chart (`VS-11`): a new chart part with these bytes,
    /// in which [`NewShape::WORKBOOK_REL`] is replaced by the relationship to a new
    /// embedded workbook with these bytes.
    Chart {
        /// The chart part's XML.
        chart: String,
        /// The `.xlsx` the chart's data is edited in.
        workbook: Vec<u8>,
    },
}

impl NewShape {
    /// What a new chart's XML says where its workbook's relationship id goes.
    pub const WORKBOOK_REL: &'static str = "OPENPRESENT-WORKBOOK-REL";
}

/// New media, for [`Op::ApplyMediaReplacement`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaBytes {
    /// The bytes.
    pub bytes: Vec<u8>,
    /// Their extension, with no dot.
    pub extension: String,
    /// Their media type.
    pub content_type: String,
}

/// The operation that saves a planned replacement. `tree_bytes` is the part's XML after
/// [`crate::media::replace_media`] patched it; `media` is what [`new_media_for`] was given.
pub fn media_replacement_op(
    part: &str,
    replacement: &MediaReplacement,
    tree_bytes: Vec<u8>,
    media: Option<MediaBytes>,
) -> Op {
    Op::ApplyMediaReplacement {
        part: part.to_owned(),
        xml: tree_bytes,
        added: replacement.added.clone(),
        released: replacement.released.clone(),
        media,
    }
}

/// Where these bytes will live, for [`crate::media::replace_media`] to plan with: the media
/// part that already holds them (`INV-SP-03`), or the name a new one would take.
pub fn new_media_for(
    package: &mut PresentationPackage,
    part: &str,
    media: &MediaBytes,
) -> Result<NewMedia, WriterError> {
    let mut draft = SavePlan::new().stage(package)?;
    let existing = draft.existing_media(&media.bytes)?;
    let name = match existing {
        Some(name) => name,
        None => draft.families.mint(
            "ppt/media",
            media_stem(&media.content_type),
            &media.extension,
        )?,
    };
    Ok(NewMedia {
        rel_type: draft.rel_type(media_kind(&media.content_type)),
        target: target_for(part, &name, false),
        mode: TargetMode::Internal,
    })
}

fn media_stem(content_type: &str) -> &'static str {
    if is_image(content_type) {
        "image"
    } else {
        "media"
    }
}

fn media_kind(content_type: &str) -> &'static str {
    let lower = content_type.to_ascii_lowercase();
    if lower.starts_with("audio/") {
        "audio"
    } else if lower.starts_with("video/") {
        "video"
    } else {
        "image"
    }
}

fn bad(what: &'static str, subject: impl Into<String>) -> WriterError {
    WriterError::BadOperation {
        what,
        subject: subject.into(),
    }
}

/// Every element of a tree. A tree's depth is bounded when it is parsed.
pub(crate) fn all_elements(tree: &XmlTree) -> Vec<ElementId> {
    all_elements_under(tree, tree.root())
}

/// `top` and every element under it.
fn all_elements_under(tree: &XmlTree, top: ElementId) -> Vec<ElementId> {
    let mut out = Vec::new();
    let mut stack = vec![top];
    while let Some(el) = stack.pop() {
        out.push(el);
        let mut children = tree.child_elements(el);
        children.reverse();
        stack.extend(children);
    }
    out
}

/// Elements holding an attribute in the relationships namespace with this value.
fn uses_of(tree: &XmlTree, id: &str) -> Vec<ElementId> {
    all_elements(tree)
        .into_iter()
        .filter(|&el| {
            tree.declared_attributes(el).iter().any(|(name, value)| {
                value == id
                    && name
                        .split_once(':')
                        .is_some_and(|(prefix, _)| prefix_is_relationships(tree, el, prefix))
            })
        })
        .collect()
}

fn is_image(content_type: &str) -> bool {
    content_type.to_ascii_lowercase().starts_with("image/")
}

impl Draft<'_> {
    pub(crate) fn apply(&mut self, op: &Op) -> Result<(), WriterError> {
        match op {
            Op::AddSlide { position, layout } => self.add_slide(*position, layout.as_deref()),
            Op::DuplicateSlide { slide } => self.duplicate_slide(slide),
            Op::DeleteSlide { slide } => self.delete_slide(slide),
            Op::MoveSlide { slide, position } => self.move_slide(slide, *position),
            Op::AddNotes { slide, text } => self.add_notes(slide, text),
            Op::RemoveNotes { slide } => self.remove_notes(slide),
            Op::AddPicture {
                slide,
                bytes,
                extension,
                content_type,
            } => self.add_picture(slide, bytes, extension, content_type),
            Op::RemovePicture { slide, shape_id } => self.remove_picture(slide, *shape_id),
            Op::RemoveShape { slide, shape_id } => self.remove_shape(slide, *shape_id),
            Op::MoveShape {
                slide,
                shape_id,
                index,
            } => self.move_shape(slide, *shape_id, *index),
            Op::ReplaceMedia {
                slide,
                rel_id,
                bytes,
                extension,
                content_type,
            } => self.replace_media(slide, rel_id, bytes, extension, content_type),
            Op::ApplyMediaReplacement {
                part,
                xml,
                added,
                released,
                media,
            } => self.apply_media_replacement(
                part,
                xml,
                added.as_ref(),
                released.as_deref(),
                media.as_ref(),
            ),
            Op::AddLayout { master, copy_of } => self.add_layout(master, copy_of),
            Op::RemoveLayout { layout } => self.remove_layout(layout),
            Op::AddMaster { copy_of } => self.add_master(copy_of),
            Op::RemoveMaster { master } => self.remove_master(master),
            Op::EditTheme { master, bytes } => self.edit_theme(master, bytes),
            Op::AddSection { name, id, at } => self.add_section(name, id, at.as_ref()),
            Op::RemoveSection { id } => self.remove_section(id),
            Op::MoveSlideToSection {
                slide,
                section,
                index,
            } => self.move_slide_to_section(slide, section, *index),
            Op::AddCommentAuthor { name, initials } => self.add_comment_author(name, initials),
            Op::AddCustomShow { name, slides } => self.add_custom_show(name, slides),
            Op::CopyShape {
                slide,
                shape_id,
                creation_id,
            } => self.copy_shape(slide, *shape_id, creation_id.as_deref()),
            Op::InsertShape {
                slide,
                index,
                shape,
                shape_id,
                name,
            } => self.insert_shape(slide, *index, shape, *shape_id, name),
            Op::Retype { extension } => self.retype(extension),
            Op::DropVba => self.drop_vba(),
            Op::AttachPart {
                source,
                kind,
                dir,
                stem,
                extension,
                content_type,
                bytes,
            } => self.attach_part(source, kind, dir, stem, extension, content_type, bytes),
        }
    }

    fn first_layout(&mut self) -> Result<String, WriterError> {
        let main = self.main.clone();
        let rows: Vec<String> = {
            let tree = self.tree(&main)?;
            pml_child(tree, tree.root(), "sldMasterIdLst")
                .map(|list| {
                    pml_children(tree, list, "sldMasterId")
                        .into_iter()
                        .filter_map(|el| rel_attribute(tree, el, "id"))
                        .collect()
                })
                .unwrap_or_default()
        };
        let rels = self.relationships(&main)?;
        for rel_id in rows {
            let Some(master) = rels
                .iter()
                .find(|r| r.id == rel_id)
                .and_then(|r| self.resolve(&main, r))
            else {
                continue;
            };
            for rel in self.relationships(&master)? {
                if rel_kind(&rel.rel_type) == "slideLayout"
                    && let Some(layout) = self.resolve(&master, &rel)
                {
                    return Ok(layout);
                }
            }
        }
        Err(bad("the deck has no layout", main))
    }

    fn layout_of(&mut self, slide: &str) -> Result<Option<String>, WriterError> {
        for rel in self.relationships(slide)? {
            if rel_kind(&rel.rel_type) == "slideLayout" {
                return Ok(self.resolve(slide, &rel));
            }
        }
        Ok(None)
    }

    fn add_slide(
        &mut self,
        position: Option<usize>,
        layout: Option<&str>,
    ) -> Result<(), WriterError> {
        let entries = self.slide_entries()?;
        let position = position.unwrap_or(entries.len());
        if position > entries.len() {
            return Err(bad("position past the end", position.to_string()));
        }
        let previous = position.checked_sub(1).and_then(|i| entries.get(i));
        let layout = match layout {
            Some(layout) => self
                .actual(layout)
                .ok_or_else(|| bad("no such layout", layout))?,
            None => {
                let from_previous = match previous.and_then(|p| p.part.clone()) {
                    Some(part) => self.layout_of(&part)?,
                    None => None,
                };
                match from_previous {
                    Some(layout) => layout,
                    None => self.first_layout()?,
                }
            }
        };
        let body = {
            let conformance = self.conformance;
            let tree = self.tree(&layout)?;
            new_slide_xml(conformance, tree)
        };
        let part = self.families.mint("ppt/slides", "slide", "xml")?;
        self.add_part(&part, Some(&format!("{PML}slide+xml")), body.into_bytes())?;
        self.add_relationship(&part, "slideLayout", &layout)?;
        self.link_new_slide(&part, position, previous.map(|p| p.raw_id.clone()))
    }

    fn link_new_slide(
        &mut self,
        part: &str,
        position: usize,
        previous: Option<String>,
    ) -> Result<(), WriterError> {
        let main = self.main.clone();
        let rel_id = self.add_relationship(&main, "slide", part)?;
        let id = self.insert_slide_entry(&rel_id, position)?;
        self.place_in_section(&id, previous.as_deref())?;
        self.log.new_slide_ids.insert(id);
        self.note_new_slide_sections()?;
        self.new_slides.push(part.to_owned());
        Ok(())
    }

    fn duplicate_slide(&mut self, slide: &SlideRef) -> Result<(), WriterError> {
        let (at, entry) = self.slide(slide)?;
        let part = Self::slide_part(&entry)?;
        let clones = self.deep_clone(&part, ClonePolicy::Slide)?;
        let clone = clones
            .get(&part)
            .cloned()
            .ok_or_else(|| bad("no such slide", part.clone()))?;
        self.link_new_slide(&clone, at.saturating_add(1), Some(entry.raw_id))
    }

    /// The parts still reached from the root without passing through `skip`, by key.
    fn survivors_without(&mut self, skip: &str) -> Result<BTreeSet<String>, WriterError> {
        let skip_key = key(skip);
        let mut seen = BTreeSet::new();
        let mut queue = std::collections::VecDeque::from([String::new()]);
        while let Some(part) = queue.pop_front() {
            if key(&part) == skip_key || !seen.insert(key(&part)) {
                continue;
            }
            for rel in self.relationships(&part)? {
                if let Some(target) = self.resolve(&part, &rel) {
                    queue.push_back(target);
                }
            }
        }
        Ok(seen)
    }

    /// Every slide id a zoom elsewhere in the deck points to (`INV-FB-05`).
    fn refuse_if_zoom_target(&mut self, id: &str, part: &str) -> Result<(), WriterError> {
        let slides: Vec<String> = self
            .slide_entries()?
            .into_iter()
            .filter_map(|e| e.part)
            .collect();
        for slide in slides {
            if key(&slide) == key(part) {
                continue;
            }
            let bytes = self.xml_bytes(&slide)?;
            if !bytes.windows(8).any(|w| w == b"sldZmObj") {
                continue;
            }
            let tree = XmlTree::parse(&bytes, &slide, &self.xml)?;
            let hit = all_elements(&tree).into_iter().any(|el| {
                tree.local_name(el) == "sldZmObj"
                    && tree.plain_attribute(el, "sldId").as_deref() == Some(id)
            });
            if hit {
                // `docs/36` has the deleted target's zoom replaced by its fallback. That
                // belongs with the other fallbacks; until then, block rather than dangle.
                return Err(WriterError::Blocked {
                    construct: "a slide zoom naming the slide",
                    part: slide,
                });
            }
        }
        Ok(())
    }

    /// Take out what uses a relationship to a deleted slide, so nothing dangles.
    fn detach_uses(&mut self, source: &str, id: &str) -> Result<(), WriterError> {
        let tree = self.tree(source)?;
        let mut removed_link = false;
        for el in uses_of(tree, id) {
            let local = tree.local_name(el).to_owned();
            match local.as_str() {
                "hlinkClick" | "hlinkHover" | "hlinkMouseOver" => removed_link = true,
                // `sldIdLst`, a custom show's `sldLst`, the outline view's list.
                "sldId" | "sld" => {}
                _ => {
                    return Err(WriterError::Blocked {
                        construct: "a reference to the slide of an unknown kind",
                        part: source.to_owned(),
                    });
                }
            }
            if let Some(parent) = tree.parent_element(el) {
                tree.remove_child(parent, el);
            }
        }
        if removed_link {
            self.note(
                "CS-SAV-0023",
                "INV-SP-09",
                source,
                "a link to a deleted slide was removed",
            );
        }
        Ok(())
    }

    /// `INV-SP-09` (Inferred): the slide's notes and comments go with it by the sweep,
    /// and its entries in custom shows, sections and links go here.
    fn delete_slide(&mut self, slide: &SlideRef) -> Result<(), WriterError> {
        let (_, entry) = self.slide(slide)?;
        let part = Self::slide_part(&entry)?;
        self.refuse_if_zoom_target(&entry.raw_id, &part)?;
        let listed = self.slide_entries()?;
        let survivors = self.survivors_without(&part)?;
        let sources: Vec<String> = std::iter::once(String::new())
            .chain(self.names.values().cloned())
            .filter(|s| survivors.contains(&key(s)))
            .collect();
        let goal = key(&part);
        for source in sources {
            for rel in self.relationships(&source)? {
                if self.resolve(&source, &rel).is_some_and(|t| key(&t) == goal) {
                    if !source.is_empty() {
                        self.detach_uses(&source, &rel.id)?;
                    }
                    self.remove_relationship(&source, &rel.id)?;
                }
            }
        }
        // The list entry may have no relationship left to find it by.
        self.remove_slide_element(entry.element)?;
        // A part listed twice loses both entries, with their sections.
        let left: BTreeSet<String> = self
            .slide_entries()?
            .into_iter()
            .map(|e| e.raw_id)
            .collect();
        for gone in listed.iter().filter(|e| !left.contains(&e.raw_id)) {
            self.drop_from_sections(&gone.raw_id)?;
            if let Some(id) = gone.id {
                self.log.retired_slide_ids.insert(id);
            }
        }
        // The slide itself goes by the sweep, or stays where only an orphan points to it.
        Ok(())
    }

    fn move_slide(&mut self, slide: &SlideRef, position: usize) -> Result<(), WriterError> {
        let (at, entry) = self.slide(slide)?;
        let entries = self.slide_entries()?;
        if position >= entries.len() {
            return Err(bad("position past the end", position.to_string()));
        }
        if position == at {
            return Ok(());
        }
        let current: Vec<ElementId> = entries.iter().map(|e| e.element).collect();
        let mut order: Vec<&super::structure::SlideEntry> = entries
            .iter()
            .filter(|e| e.element != entry.element)
            .collect();
        let previous = position
            .checked_sub(1)
            .and_then(|i| order.get(i))
            .map(|p| p.raw_id.clone());
        order.insert(position.min(order.len()), &entry);
        let new: Vec<ElementId> = order.iter().map(|e| e.element).collect();
        let main = self.main.clone();
        let tree = self.tree(&main)?;
        if let Some(list) = pml_child(tree, tree.root(), "sldIdLst") {
            tree.replace_children(list, &current, &new);
        }
        self.place_in_section(&entry.raw_id, previous.as_deref())?;
        self.note_new_slide_sections()
    }

    /// `INV-SP-03` (Inferred): bytes matching a media part reuse that part. Otherwise a
    /// new part, named by family, never by the user's file (`INV-PN-02`).
    pub(crate) fn media_part(
        &mut self,
        bytes: &[u8],
        extension: &str,
        content_type: &str,
    ) -> Result<String, WriterError> {
        if bytes.len() as u64 > self.limits.max_media_bytes {
            return Err(WriterError::TooLarge {
                what: "bytes of media",
                limit: self.limits.max_media_bytes,
            });
        }
        if let Some(existing) = self.existing_media(bytes)? {
            return Ok(existing);
        }
        let name = self
            .families
            .mint("ppt/media", media_stem(content_type), extension)?;
        self.add_part(&name, Some(content_type), bytes.to_vec())?;
        Ok(name)
    }

    /// The media part already holding these bytes.
    fn existing_media(&mut self, bytes: &[u8]) -> Result<Option<String>, WriterError> {
        let media: Vec<String> = self
            .names
            .iter()
            .filter(|(k, _)| k.starts_with("ppt/media/"))
            .map(|(_, n)| n.clone())
            .collect();
        for existing in media {
            if self.part_bytes(&existing)? == bytes {
                return Ok(Some(existing));
            }
        }
        Ok(None)
    }

    /// What [`Op::ApplyMediaReplacement`] does: the media part comes first, so the
    /// relationship can name it; the old relationship goes, and the sweep takes the old
    /// part only if nothing else in the deck reaches it (`INV-SP-02`, `INV-SP-04`).
    fn apply_media_replacement(
        &mut self,
        part: &str,
        xml: &[u8],
        added: Option<&Relationship>,
        released: Option<&str>,
        media: Option<&MediaBytes>,
    ) -> Result<(), WriterError> {
        let actual = self.actual(part).ok_or_else(|| bad("no such part", part))?;
        let new = XmlTree::parse(xml, &actual, &self.xml)?;
        let old = self.tree(&actual)?;
        if old.local_name(old.root()) != new.local_name(new.root()) {
            return Err(bad("a replacement with another root element", actual));
        }
        self.set_part_bytes(&actual, xml.to_vec());
        let name = match media {
            Some(m) => Some(self.media_part(&m.bytes, &m.extension, &m.content_type)?),
            None => None,
        };
        if let Some(rel) = added {
            let written = match (&name, rel.mode) {
                (Some(name), TargetMode::Internal) => target_for(&actual, name, false),
                (None, TargetMode::External) => rel.target.clone(),
                _ => return Err(bad("a new relationship with no media", rel.id.clone())),
            };
            self.add_relationship_exact(&actual, rel, &written)?;
        }
        if let Some(id) = released {
            self.remove_relationship(&actual, id)?;
        }
        Ok(())
    }

    fn add_picture(
        &mut self,
        slide: &SlideRef,
        bytes: &[u8],
        extension: &str,
        content_type: &str,
    ) -> Result<(), WriterError> {
        let (_, entry) = self.slide(slide)?;
        let part = Self::slide_part(&entry)?;
        let media = self.media_part(bytes, extension, content_type)?;
        let rel_id = self.reuse_relationship(&part, "image", &media)?;
        let conformance = self.conformance;
        let rel_uri = conformance
            .spell("http://schemas.openxmlformats.org/officeDocument/2006/relationships");
        let dml = conformance.spell("http://schemas.openxmlformats.org/drawingml/2006/main");
        let tree = self.tree(&part)?;
        let sp_tree = pml_child(tree, tree.root(), "cSld")
            .and_then(|c| pml_child(tree, c, "spTree"))
            .ok_or_else(|| bad("the slide has no shape tree", part.clone()))?;
        let next_id = all_elements(tree)
            .into_iter()
            .filter(|&el| tree.local_name(el) == "cNvPr")
            .filter_map(|el| tree.plain_attribute(el, "id")?.parse::<u64>().ok())
            .max()
            .unwrap_or(1)
            .saturating_add(1);
        let r_prefix = tree.prefix_for_namespace(sp_tree, rel_uri);
        let a_prefix = tree.prefix_for_namespace(sp_tree, dml);
        let p = |local: &str| sibling_name(tree, sp_tree, local);
        let names = (
            p("pic"),
            p("nvPicPr"),
            p("cNvPr"),
            p("cNvPicPr"),
            p("nvPr"),
            p("blipFill"),
            p("spPr"),
        );
        let a = match a_prefix.as_deref() {
            Some("") => String::new(),
            Some(prefix) => format!("{prefix}:"),
            None => "a:".to_owned(),
        };
        let declare_a = a_prefix.is_none();
        let (r_attr, declare_r) = match r_prefix.as_deref() {
            Some(prefix) if !prefix.is_empty() => (format!("{prefix}:embed"), false),
            _ => ("r:embed".to_owned(), true),
        };
        let pic = tree.new_element(&names.0);
        if declare_a {
            set_attr(tree, pic, "xmlns:a", dml, &part)?;
        }
        if declare_r {
            set_attr(tree, pic, "xmlns:r", rel_uri, &part)?;
        }
        let nv = tree.new_element(&names.1);
        let c_nv = tree.new_element(&names.2);
        set_attr(tree, c_nv, "id", &next_id.to_string(), &part)?;
        set_attr(tree, c_nv, "name", &format!("Picture {next_id}"), &part)?;
        let c_pic = tree.new_element(&names.3);
        let nv_pr = tree.new_element(&names.4);
        tree.insert_child(nv, None, c_nv);
        tree.insert_child(nv, None, c_pic);
        tree.insert_child(nv, None, nv_pr);
        let fill = tree.new_element(&names.5);
        let blip = tree.new_element(&format!("{a}blip"));
        set_attr(tree, blip, &r_attr, &rel_id, &part)?;
        let stretch = tree.new_element(&format!("{a}stretch"));
        let rect = tree.new_element(&format!("{a}fillRect"));
        tree.insert_child(stretch, None, rect);
        tree.insert_child(fill, None, blip);
        tree.insert_child(fill, None, stretch);
        let sp_pr = tree.new_element(&names.6);
        let xfrm = tree.new_element(&format!("{a}xfrm"));
        let off = tree.new_element(&format!("{a}off"));
        set_attr(tree, off, "x", "0", &part)?;
        set_attr(tree, off, "y", "0", &part)?;
        let ext = tree.new_element(&format!("{a}ext"));
        set_attr(tree, ext, "cx", "914400", &part)?;
        set_attr(tree, ext, "cy", "914400", &part)?;
        tree.insert_child(xfrm, None, off);
        tree.insert_child(xfrm, None, ext);
        let geometry = tree.new_element(&format!("{a}prstGeom"));
        set_attr(tree, geometry, "prst", "rect", &part)?;
        let av = tree.new_element(&format!("{a}avLst"));
        tree.insert_child(geometry, None, av);
        tree.insert_child(sp_pr, None, xfrm);
        tree.insert_child(sp_pr, None, geometry);
        tree.insert_child(pic, None, nv);
        tree.insert_child(pic, None, fill);
        tree.insert_child(pic, None, sp_pr);
        // Before the shape tree's own `extLst`, where the schema has it.
        let before = pml_child(tree, sp_tree, "extLst");
        tree.insert_child(sp_tree, before, pic);
        Ok(())
    }

    fn remove_picture(&mut self, slide: &SlideRef, shape_id: u32) -> Result<(), WriterError> {
        let (_, entry) = self.slide(slide)?;
        let part = Self::slide_part(&entry)?;
        let wanted = shape_id.to_string();
        let tree = self.tree(&part)?;
        let found = all_elements(tree).into_iter().find(|&el| {
            tree.local_name(el) == "pic"
                && pml_child(tree, el, "nvPicPr")
                    .and_then(|nv| pml_child(tree, nv, "cNvPr"))
                    .and_then(|c| tree.plain_attribute(c, "id"))
                    .as_deref()
                    == Some(wanted.as_str())
        });
        let Some(pic) = found else {
            return Err(bad("no such picture", wanted));
        };
        self.remove_element(&part, pic)
    }

    fn move_shape(
        &mut self,
        slide: &SlideRef,
        shape_id: u32,
        index: usize,
    ) -> Result<(), WriterError> {
        const SHAPES: [&str; 7] = [
            "sp",
            "pic",
            "grpSp",
            "graphicFrame",
            "cxnSp",
            "contentPart",
            "AlternateContent",
        ];
        let (_, entry) = self.slide(slide)?;
        let part = Self::slide_part(&entry)?;
        let wanted = shape_id.to_string();
        let tree = self.tree(&part)?;
        let sp_tree = pml_child(tree, tree.root(), "cSld")
            .and_then(|c| pml_child(tree, c, "spTree"))
            .ok_or_else(|| bad("the slide has no shape tree", part.clone()))?;
        let shapes: Vec<ElementId> = tree
            .child_elements(sp_tree)
            .into_iter()
            .filter(|&el| SHAPES.contains(&tree.local_name(el)))
            .collect();
        let names = |el: ElementId| {
            all_elements_under(tree, el).into_iter().any(|c| {
                tree.local_name(c) == "cNvPr"
                    && tree.plain_attribute(c, "id").as_deref() == Some(wanted.as_str())
            })
        };
        let found: Vec<ElementId> = shapes.iter().copied().filter(|&el| names(el)).collect();
        let shape = match found.as_slice() {
            [one] => *one,
            [] => return Err(bad("no such shape at the top of the tree", wanted)),
            _ => return Err(bad("a shape id two shapes share", wanted)),
        };
        let others: Vec<ElementId> = shapes.into_iter().filter(|&el| el != shape).collect();
        let before = others.get(index).copied();
        if before.is_none() && others.last().is_none() {
            return Ok(());
        }
        // Already in place: nothing to write.
        let now = tree
            .child_elements(sp_tree)
            .into_iter()
            .filter(|&el| SHAPES.contains(&tree.local_name(el)))
            .position(|el| el == shape);
        if now == Some(index.min(others.len())) {
            return Ok(());
        }
        tree.remove_child(sp_tree, shape);
        tree.insert_child(sp_tree, before, shape);
        Ok(())
    }

    fn remove_shape(&mut self, slide: &SlideRef, shape_id: u32) -> Result<(), WriterError> {
        const SHAPES: [&str; 6] = ["sp", "pic", "grpSp", "graphicFrame", "cxnSp", "contentPart"];
        let (_, entry) = self.slide(slide)?;
        let part = Self::slide_part(&entry)?;
        let wanted = shape_id.to_string();
        let tree = self.tree(&part)?;
        let elements = all_elements(tree);
        let mut found: Vec<ElementId> = elements
            .iter()
            .copied()
            .filter(|&el| {
                tree.local_name(el) == "cNvPr"
                    && tree.plain_attribute(el, "id").as_deref() == Some(wanted.as_str())
            })
            .filter_map(|c| {
                tree.parent_element(c)
                    .and_then(|nv| tree.parent_element(nv))
            })
            .filter(|&shape| SHAPES.contains(&tree.local_name(shape)))
            .map(|shape| {
                // A shape in `mc:Choice` or `mc:Fallback` goes with the whole
                // `mc:AlternateContent`, so no branch is left naming it alone.
                let mut top = shape;
                while let Some(parent) = tree.parent_element(top) {
                    if matches!(
                        tree.local_name(parent),
                        "Choice" | "Fallback" | "AlternateContent"
                    ) {
                        top = parent;
                    } else {
                        break;
                    }
                }
                top
            })
            .collect();
        found.dedup();
        let shape = match found.as_slice() {
            [one] => *one,
            [] => return Err(bad("no such shape", wanted)),
            _ => return Err(bad("a shape id two shapes share", wanted)),
        };
        // `INV-ID-06`: an animation naming the shape would dangle.
        let animated = elements.iter().any(|&el| {
            tree.plain_attribute(el, "spid").as_deref() == Some(wanted.as_str()) && {
                let mut at = tree.parent_element(el);
                let mut in_timing = false;
                while let Some(here) = at {
                    if tree.local_name(here) == "timing" {
                        in_timing = true;
                        break;
                    }
                    at = tree.parent_element(here);
                }
                in_timing
            }
        });
        if animated {
            return Err(bad("an animated shape", wanted));
        }
        self.remove_element(&part, shape)
    }

    /// Remove `pic`, an element of `part`, and each relationship only it used.
    fn remove_element(&mut self, part: &str, pic: ElementId) -> Result<(), WriterError> {
        let part = part.to_owned();
        let tree = self.tree(&part)?;
        let held: Vec<String> = all_elements(tree)
            .into_iter()
            .filter(|&el| {
                let mut at = Some(el);
                while let Some(here) = at {
                    if here == pic {
                        return true;
                    }
                    at = tree.parent_element(here);
                }
                false
            })
            .flat_map(|el| {
                tree.declared_attributes(el)
                    .into_iter()
                    .filter(move |(name, _)| name.contains(':') && !name.starts_with("xmlns"))
                    .map(|(_, value)| value)
                    .collect::<Vec<_>>()
            })
            .collect();
        if let Some(parent) = tree.parent_element(pic) {
            tree.remove_child(parent, pic);
        }
        let rels = self.relationships(&part)?;
        for id in held {
            if rels.iter().any(|r| r.id == id) {
                let tree = self.tree(&part)?;
                if uses_of(tree, &id).is_empty() {
                    self.remove_relationship(&part, &id)?;
                }
            }
        }
        Ok(())
    }

    fn replace_media(
        &mut self,
        slide: &SlideRef,
        rel_id: &str,
        bytes: &[u8],
        extension: &str,
        content_type: &str,
    ) -> Result<(), WriterError> {
        let (_, entry) = self.slide(slide)?;
        let part = Self::slide_part(&entry)?;
        if !self.relationships(&part)?.iter().any(|r| r.id == rel_id) {
            return Err(bad("no such relationship", rel_id));
        }
        let media = self.media_part(bytes, extension, content_type)?;
        self.retarget_relationship(&part, rel_id, &media)
    }

    /// A master's `sldLayoutIdLst`, created at its schema position if absent.
    fn layout_list(&mut self, master: &str) -> Result<ElementId, WriterError> {
        let tree = self.tree(master)?;
        let root = tree.root();
        if let Some(list) = pml_child(tree, root, "sldLayoutIdLst") {
            return Ok(list);
        }
        let name = sibling_name(tree, root, "sldLayoutIdLst");
        let new = tree.new_element(&name);
        // `CT_SlideMaster`: cSld, clrMap, sldLayoutIdLst, transition, timing, hf, txStyles.
        let before = tree.child_elements(root).into_iter().find(|&c| {
            matches!(
                tree.local_name(c),
                "transition" | "timing" | "hf" | "txStyles" | "extLst"
            )
        });
        tree.insert_child(root, before, new);
        Ok(new)
    }

    fn add_layout(&mut self, master: &str, copy_of: &str) -> Result<(), WriterError> {
        let master = self
            .actual(master)
            .ok_or_else(|| bad("no such master", master))?;
        let source = self
            .actual(copy_of)
            .ok_or_else(|| bad("no such layout", copy_of))?;
        let clones = self.deep_clone(&source, ClonePolicy::Layout)?;
        let clone = clones
            .get(&source)
            .cloned()
            .ok_or_else(|| bad("no such layout", source.clone()))?;
        // The clone points back to the master that holds it, whichever the source's was.
        for rel in self.relationships(&clone)? {
            if rel_kind(&rel.rel_type) == "slideMaster" {
                self.retarget_relationship(&clone, &rel.id, &master)?;
            }
        }
        let rel_id = self.add_relationship(&master, "slideLayout", &clone)?;
        let id = self.mint_pool_id()?;
        let list = self.layout_list(&master)?;
        self.append_id_entry(&master, list, "sldLayoutId", Some(id), &rel_id)
    }

    fn slides_on_layouts(&mut self, layouts: &[String]) -> Result<bool, WriterError> {
        let keys: BTreeSet<String> = layouts.iter().map(|l| key(l)).collect();
        for entry in self.slide_entries()? {
            if let Some(part) = entry.part
                && let Some(layout) = self.layout_of(&part)?
                && keys.contains(&key(&layout))
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn master_of_layout(&mut self, layout: &str) -> Result<Option<String>, WriterError> {
        for rel in self.relationships(layout)? {
            if rel_kind(&rel.rel_type) == "slideMaster" {
                return Ok(self.resolve(layout, &rel));
            }
        }
        Ok(None)
    }

    /// `INV-SP-06` (the refusal Inferred): at least one layout stays, and none in use goes.
    fn remove_layout(&mut self, layout: &str) -> Result<(), WriterError> {
        let layout = self
            .actual(layout)
            .ok_or_else(|| bad("no such layout", layout))?;
        if self.slides_on_layouts(std::slice::from_ref(&layout))? {
            return Err(bad("the layout is in use", layout));
        }
        let master = self
            .master_of_layout(&layout)?
            .ok_or_else(|| bad("the layout has no master", layout.clone()))?;
        let mut rel_id = None;
        for rel in self.relationships(&master)? {
            if self
                .resolve(&master, &rel)
                .is_some_and(|t| key(&t) == key(&layout))
            {
                rel_id = Some(rel.id);
            }
        }
        let rel_id = rel_id.ok_or_else(|| bad("the master does not hold it", layout.clone()))?;
        let others = self
            .relationships(&master)?
            .iter()
            .filter(|r| rel_kind(&r.rel_type) == "slideLayout")
            .count();
        if others <= 1 {
            return Err(bad("a master keeps at least one layout", layout));
        }
        let mut retired = Vec::new();
        let tree = self.tree(&master)?;
        if let Some(list) = pml_child(tree, tree.root(), "sldLayoutIdLst") {
            for el in pml_children(tree, list, "sldLayoutId") {
                if rel_attribute(tree, el, "id").as_deref() == Some(rel_id.as_str()) {
                    if let Some(n) = tree
                        .plain_attribute(el, "id")
                        .and_then(|v| v.parse::<u64>().ok())
                    {
                        retired.push(n);
                    }
                    tree.remove_child(list, el);
                }
            }
        }
        self.log.retired_layout_ids.extend(retired);
        self.remove_relationship(&master, &rel_id)
    }

    fn layouts_of(&mut self, master: &str) -> Result<Vec<String>, WriterError> {
        let mut out = Vec::new();
        for rel in self.relationships(master)? {
            if rel_kind(&rel.rel_type) == "slideLayout"
                && let Some(layout) = self.resolve(master, &rel)
            {
                out.push(layout);
            }
        }
        Ok(out)
    }

    fn add_master(&mut self, copy_of: &str) -> Result<(), WriterError> {
        let source = self
            .actual(copy_of)
            .ok_or_else(|| bad("no such master", copy_of))?;
        let clones = self.deep_clone(&source, ClonePolicy::Master)?;
        let clone = clones
            .get(&source)
            .cloned()
            .ok_or_else(|| bad("no such master", source.clone()))?;
        // The body is a byte copy, so its layout list holds the source's ids.
        let rows: Vec<ElementId> = {
            let tree = self.tree(&clone)?;
            pml_child(tree, tree.root(), "sldLayoutIdLst")
                .map(|list| pml_children(tree, list, "sldLayoutId"))
                .unwrap_or_default()
        };
        for el in rows {
            let id = self.mint_pool_id()?;
            let tree = self.tree(&clone)?;
            set_attr(tree, el, "id", &id.to_string(), &clone)?;
        }
        let main = self.main.clone();
        let rel_id = self.add_relationship(&main, "slideMaster", &clone)?;
        let id = self.mint_pool_id()?;
        let list = self.ensure_list("sldMasterIdLst")?;
        self.append_id_entry(&main, list, "sldMasterId", Some(id), &rel_id)
    }

    /// `sldMasterIdLst`: each entry's relationship id and the part it lands on.
    fn master_entries(&mut self) -> Result<Vec<(String, Option<String>)>, WriterError> {
        let main = self.main.clone();
        let rows: Vec<String> = {
            let tree = self.tree(&main)?;
            pml_child(tree, tree.root(), "sldMasterIdLst")
                .map(|list| {
                    pml_children(tree, list, "sldMasterId")
                        .into_iter()
                        .map(|el| rel_attribute(tree, el, "id").unwrap_or_default())
                        .collect()
                })
                .unwrap_or_default()
        };
        let rels = self.relationships(&main)?;
        Ok(rows
            .into_iter()
            .map(|rel_id| {
                let part = rels
                    .iter()
                    .find(|r| r.id == rel_id)
                    .and_then(|r| self.resolve(&main, r));
                (rel_id, part)
            })
            .collect())
    }

    fn theme_of(&mut self, part: &str) -> Result<Option<(String, String)>, WriterError> {
        for rel in self.relationships(part)? {
            if rel_kind(&rel.rel_type) == "theme"
                && let Some(theme) = self.resolve(part, &rel)
            {
                return Ok(Some((rel.id, theme)));
            }
        }
        Ok(None)
    }

    /// `INV-SP-08` (Unverified): the presentation's own theme relationship follows the
    /// first master's theme.
    fn sync_presentation_theme(&mut self) -> Result<(), WriterError> {
        let main = self.main.clone();
        let Some((rel_id, current)) = self.theme_of(&main)? else {
            return Ok(());
        };
        let first = self
            .master_entries()?
            .into_iter()
            .find_map(|(_, part)| part);
        let Some(first) = first else {
            return Ok(());
        };
        if let Some((_, theme)) = self.theme_of(&first)?
            && key(&theme) != key(&current)
        {
            self.retarget_relationship(&main, &rel_id, &theme)?;
        }
        Ok(())
    }

    fn remove_master(&mut self, master: &str) -> Result<(), WriterError> {
        let master = self
            .actual(master)
            .ok_or_else(|| bad("no such master", master))?;
        let entries = self.master_entries()?;
        if entries.len() <= 1 {
            return Err(bad("a deck keeps at least one master", master));
        }
        let goal = key(&master);
        let (rel_id, _) = entries
            .iter()
            .find(|(_, part)| part.as_deref().is_some_and(|p| key(p) == goal))
            .cloned()
            .ok_or_else(|| bad("no such master", master.clone()))?;
        let layouts = self.layouts_of(&master)?;
        if self.slides_on_layouts(&layouts)? {
            return Err(bad("a layout of the master is in use", master));
        }
        let main = self.main.clone();
        let mut retired = Vec::new();
        let tree = self.tree(&main)?;
        if let Some(list) = pml_child(tree, tree.root(), "sldMasterIdLst") {
            for el in pml_children(tree, list, "sldMasterId") {
                if rel_attribute(tree, el, "id").as_deref() == Some(rel_id.as_str()) {
                    if let Some(n) = tree
                        .plain_attribute(el, "id")
                        .and_then(|v| v.parse::<u64>().ok())
                    {
                        retired.push(n);
                    }
                    tree.remove_child(list, el);
                }
            }
        }
        self.log.retired_layout_ids.extend(retired);
        self.remove_relationship(&main, &rel_id)?;
        self.sync_presentation_theme()
    }

    /// `INV-SP-04`, `INV-SP-07` (Inferred): a theme edited under one master stops being
    /// shared. The presentation's own relationship does not count as a referrer.
    fn edit_theme(&mut self, master: &str, bytes: &[u8]) -> Result<(), WriterError> {
        let master = self
            .actual(master)
            .ok_or_else(|| bad("no such master", master))?;
        let (rel_id, theme) = self
            .theme_of(&master)?
            .ok_or_else(|| bad("the master has no theme", master.clone()))?;
        XmlTree::parse(bytes, &theme, &self.xml)?;
        let main_key = key(&self.main);
        let goal = key(&theme);
        let mut referrers = 0usize;
        let sources: Vec<String> = self.names.values().cloned().collect();
        for source in sources {
            if key(&source) == main_key {
                continue;
            }
            for rel in self.relationships(&source)? {
                if rel_kind(&rel.rel_type) == "theme"
                    && self.resolve(&source, &rel).is_some_and(|t| key(&t) == goal)
                {
                    referrers = referrers.saturating_add(1);
                }
            }
        }
        if referrers > 1 {
            let (dir, file) = theme
                .rsplit_once('/')
                .unwrap_or(("ppt/theme", theme.as_str()));
            let (stem, ext) = file.rsplit_once('.').unwrap_or((file, "xml"));
            let name = self.families.mint(dir, stem, ext)?;
            let content_type = self.content_type_of(&theme)?;
            self.add_part(&name, content_type.as_deref(), bytes.to_vec())?;
            // Images the theme uses stay shared; only its text changed.
            let rels = self.relationships(&theme)?;
            for rel in rels {
                if let Some(target) = self.resolve(&theme, &rel) {
                    self.add_relationship_typed(&name, &rel.rel_type, &target)?;
                }
            }
            self.retarget_relationship(&master, &rel_id, &name)?;
            self.sync_presentation_theme()?;
        } else {
            self.set_part_bytes(&theme, bytes.to_vec());
        }
        Ok(())
    }

    fn add_section(
        &mut self,
        name: &str,
        id: &str,
        at: Option<&SlideRef>,
    ) -> Result<(), WriterError> {
        let main = self.main.clone();
        check_text(name, &main)?;
        // The host supplies the id (`INV-SL-07`), so a bad one is its error, not the writer's.
        if !is_section_guid(id) {
            return Err(bad("not a brace-wrapped upper-case GUID", "section id"));
        }
        if self
            .sections()?
            .is_some_and(|sections| sections.iter().any(|s| s.id == id))
        {
            return Err(bad("a section already has this id", id));
        }
        self.log.section_list_requested = true;
        let Some(mut sections) = self.sections()? else {
            return self.create_section_list(id, name);
        };
        match at {
            None => self.append_section(id, name, None),
            Some(slide) => {
                let (_, entry) = self.slide(slide)?;
                let holder = sections
                    .iter()
                    .position(|s| s.slides.contains(&entry.raw_id))
                    .ok_or_else(|| bad("the slide is in no section", entry.raw_id.clone()))?;
                self.append_section(id, name, Some(holder))?;
                let Some(row) = sections.get_mut(holder) else {
                    return Ok(());
                };
                let from = row
                    .slides
                    .iter()
                    .position(|s| s == &entry.raw_id)
                    .unwrap_or(0);
                let moved = row.slides.split_off(from);
                sections.push(super::view::SectionFacts {
                    id: id.to_owned(),
                    slides: moved,
                });
                self.write_sections(&sections)?;
                self.note_new_slide_sections()
            }
        }
    }

    fn remove_section(&mut self, id: &str) -> Result<(), WriterError> {
        let Some(mut sections) = self.sections()? else {
            return Err(bad("the deck has no sections", id));
        };
        let at = sections
            .iter()
            .position(|s| s.id == id)
            .ok_or_else(|| bad("no such section", id))?;
        let row = sections.remove(at);
        if let Some(previous) = at.checked_sub(1).and_then(|i| sections.get_mut(i)) {
            previous.slides.extend(row.slides);
        } else if let Some(next) = sections.first_mut() {
            let mut slides = row.slides;
            slides.append(&mut next.slides);
            next.slides = slides;
        }
        self.write_sections(&sections)?;
        self.remove_section_element(id)?;
        self.log.removed_sections.insert(id.to_owned());
        self.note_new_slide_sections()
    }

    fn move_slide_to_section(
        &mut self,
        slide: &SlideRef,
        section: &str,
        index: usize,
    ) -> Result<(), WriterError> {
        let (_, entry) = self.slide(slide)?;
        let Some(mut sections) = self.sections()? else {
            return Err(bad("the deck has no sections", section));
        };
        for s in &mut sections {
            s.slides.retain(|id| id != &entry.raw_id);
        }
        let row = sections
            .iter_mut()
            .find(|s| s.id == section)
            .ok_or_else(|| bad("no such section", section))?;
        if index > row.slides.len() {
            return Err(bad("position past the end", index.to_string()));
        }
        row.slides.insert(index, entry.raw_id.clone());
        self.write_sections(&sections)?;
        self.order_slides_by_sections()?;
        self.note_new_slide_sections()
    }

    fn add_custom_show(&mut self, name: &str, slides: &[SlideRef]) -> Result<(), WriterError> {
        let main = self.main.clone();
        check_text(name, &main)?;
        let mut rels = Vec::with_capacity(slides.len());
        for slide in slides {
            rels.push(self.slide(slide)?.1.rel_id);
        }
        let list = self.ensure_list("custShowLst")?;
        let tree = self.tree(&main)?;
        let id = pml_children(tree, list, "custShow")
            .into_iter()
            .filter_map(|el| tree.plain_attribute(el, "id")?.parse::<u64>().ok())
            .max()
            .map_or(0, |top| top.saturating_add(1));
        let show_name = sibling_name(tree, list, "custShow");
        let show = tree.new_element(&show_name);
        set_attr(tree, show, "name", name, &main)?;
        set_attr(tree, show, "id", &id.to_string(), &main)?;
        let holder_name = sibling_name(tree, list, "sldLst");
        let holder = tree.new_element(&holder_name);
        let entry_name = sibling_name(tree, list, "sld");
        for rel in &rels {
            let entry = tree.new_element(&entry_name);
            set_attr(tree, entry, "r:id", rel, &main)?;
            tree.insert_child(holder, None, entry);
        }
        tree.insert_child(show, None, holder);
        tree.insert_child(list, None, show);
        Ok(())
    }

    fn add_comment_author(&mut self, name: &str, initials: &str) -> Result<(), WriterError> {
        let main = self.main.clone();
        let mut part = None;
        for rel in self.relationships(&main)? {
            if rel_kind(&rel.rel_type) == "commentAuthors" {
                part = self.resolve(&main, &rel);
            }
        }
        let dir = main.rsplit_once('/').map_or("", |(d, _)| d).to_owned();
        let part = match part {
            Some(part) => part,
            None => {
                let name = if dir.is_empty() {
                    "commentAuthors.xml".to_owned()
                } else {
                    format!("{dir}/commentAuthors.xml")
                };
                if self.has_part(&name) {
                    return Err(bad("the name is taken", name));
                }
                let pml = self
                    .conformance
                    .spell("http://schemas.openxmlformats.org/presentationml/2006/main");
                let body = format!("{}<p:cmAuthorLst xmlns:p=\"{pml}\"/>", super::draft::DECL);
                self.add_part(&name, Some(AUTHORS_TYPE), body.into_bytes())?;
                self.add_relationship(&main, "commentAuthors", &name)?;
                name
            }
        };
        check_text(name, &part)?;
        check_text(initials, &part)?;
        let tree = self.tree(&part)?;
        let root = tree.root();
        let next = pml_children(tree, root, "cmAuthor")
            .into_iter()
            .filter_map(|el| tree.plain_attribute(el, "id")?.parse::<u64>().ok())
            .max()
            .map_or(0, |top| top.saturating_add(1));
        let entry = sibling_name(tree, root, "cmAuthor");
        let new = tree.new_element(&entry);
        set_attr(tree, new, "id", &next.to_string(), &part)?;
        set_attr(tree, new, "name", name, &part)?;
        set_attr(tree, new, "initials", initials, &part)?;
        set_attr(tree, new, "lastIdx", "0", &part)?;
        set_attr(tree, new, "clrIdx", &next.to_string(), &part)?;
        tree.insert_child(root, None, new);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)] // The fields of `Op::AttachPart`, passed along.
    fn attach_part(
        &mut self,
        source: &str,
        kind: &str,
        dir: &str,
        stem: &str,
        extension: &str,
        content_type: &str,
        bytes: &[u8],
    ) -> Result<(), WriterError> {
        let source = self
            .actual(source)
            .ok_or_else(|| bad("no such part", source))?;
        let name = self.families.mint(dir, stem, extension)?;
        self.add_part(&name, Some(content_type), bytes.to_vec())?;
        if kind.contains(':') {
            self.add_relationship_typed(&source, kind, &name)?;
        } else {
            self.add_relationship(&source, kind, &name)?;
        }
        Ok(())
    }
}
