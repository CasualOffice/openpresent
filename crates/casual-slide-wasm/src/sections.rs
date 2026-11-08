//! Home > Section (`UX-006`): Add Section, Rename Section, Remove Section, each one
//! undo step (`Operation::SetSections`), shown as headers in the thumbnail pane, which
//! collapse. The list is read from `p14:sectionLst` when a deck opens, a section
//! starting at its first slide; on save, sections added, renamed or removed become the
//! writer's `AddSection`, `RenameSection` and `RemoveSection`, which keep `sldIdLst`
//! and the list in step. Names are document text.

use casual_slide_export::writer::{Op, SavePlan, SlideRef};
use casual_slide_import::presentation::import_presentation;
use casual_slide_import::properties::{Findings, ImportLimits};
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage, XmlTree, resolve_internal};
use casual_slide_transaction::{Operation, SectionMark};

use crate::design::DeckDoc;
use crate::{BridgeError, Viewer};

/// The longest section name taken, in characters.
pub const MAX_SECTION_NAME: usize = 255;

/// The sections of a deck's bytes, each starting at the part of its first slide. A
/// section with no slides, or one the reader cannot place, is left out of the pane and
/// stays in the file as it was.
pub(crate) fn read_sections(
    bytes: Vec<u8>,
    structure: &casual_slide_model::slide::DeckStructure,
) -> Vec<SectionMark> {
    let Ok(mut package) = PresentationPackage::open(bytes, OoxmlLimits::default()) else {
        return Vec::new();
    };
    let main = package.main_part().to_owned();
    let rels = package.relationships_of(&main).to_vec();
    let Ok(xml) = package.read_xml_part(&main) else {
        return Vec::new();
    };
    let Ok(tree) = XmlTree::parse(&xml, &main, package.limits()) else {
        return Vec::new();
    };
    let mut findings = Findings::default();
    let Ok(p) = import_presentation(&tree, &main, &rels, &ImportLimits::default(), &mut findings)
    else {
        return Vec::new();
    };
    let Some(list) = p.sections else {
        return Vec::new();
    };
    let slides = p.slides.unwrap_or_default();
    let part_of = |sld: u32| -> Option<u32> {
        let rel = slides.iter().find(|s| s.id == sld)?;
        let target = rels.iter().find(|r| r.id == rel.rel_id)?;
        let name = resolve_internal(&main, &target.target)?;
        structure
            .parts
            .iter()
            .find(|e| e.part == name)
            .map(|e| e.id.part)
    };
    list.sections
        .into_iter()
        .filter_map(|s| {
            let first = s.sld_id_lst.as_ref()?.first().copied()?;
            Some(SectionMark {
                id: s.id?,
                name: s.name.unwrap_or_default(),
                first: part_of(first)?,
            })
        })
        .collect()
}

fn clean(name: &str) -> Result<String, BridgeError> {
    let name: String = name.trim().chars().filter(|c| !c.is_control()).collect();
    if name.is_empty() || name.chars().count() > MAX_SECTION_NAME {
        return Err(BridgeError::BadArgument);
    }
    Ok(name)
}

/// A brace-wrapped upper-case GUID, as `p14:section/@id` holds.
fn is_guid(id: &str) -> bool {
    let Some(inner) = id.strip_prefix('{').and_then(|s| s.strip_suffix('}')) else {
        return false;
    };
    let groups: Vec<&str> = inner.split('-').collect();
    groups.iter().map(|g| g.len()).eq([8, 4, 4, 4, 12])
        && inner
            .chars()
            .all(|c| c == '-' || c.is_ascii_digit() || ('A'..='F').contains(&c))
}

impl Viewer {
    /// The sections, in order, each with its first slide's place; a section whose
    /// first slide is gone is left out.
    pub fn sections(&self) -> Vec<(SectionMark, usize)> {
        let slides = &self.deck.parts().structure.slides;
        let mut out: Vec<(SectionMark, usize)> = self
            .sections
            .iter()
            .filter_map(|s| Some((s.clone(), slides.iter().position(|&o| o == s.first)?)))
            .collect();
        out.sort_by_key(|(_, i)| *i);
        out
    }

    fn set_sections(&mut self, sections: Vec<SectionMark>) -> Result<bool, BridgeError> {
        self.typing = None;
        let changed = self.history.apply(
            &mut DeckDoc(&mut self.deck, &mut self.review, Some(&mut self.sections)),
            Operation::SetSections { sections },
        )?;
        Ok(changed)
    }

    /// Add Section at slide `index`, named `name`, with id `id` (a GUID the host makes,
    /// as time and randomness are supplied, never sampled here). With no sections yet,
    /// the slides before it go in a "Default Section" with id `default_id`, as
    /// PowerPoint does.
    ///
    /// # Errors
    ///
    /// `CS-WSM-0006` for an empty or overlong name, an id that is not a GUID or is taken,
    /// or a slide that already starts a section.
    pub fn add_section(
        &mut self,
        index: usize,
        name: &str,
        id: &str,
        default_id: &str,
    ) -> Result<bool, BridgeError> {
        let name = clean(name)?;
        let first = self.slide_ordinal(index)?;
        let mut list = self.sections.clone();
        if !is_guid(id) || list.iter().any(|s| s.id == id || s.first == first) {
            return Err(BridgeError::BadArgument);
        }
        if list.is_empty() && index > 0 {
            let start = self.slide_ordinal(0)?;
            if !is_guid(default_id) || default_id == id {
                return Err(BridgeError::BadArgument);
            }
            list.push(SectionMark {
                id: default_id.to_owned(),
                name: "Default Section".to_owned(),
                first: start,
            });
        }
        list.push(SectionMark {
            id: id.to_owned(),
            name,
            first,
        });
        self.set_sections(list)
    }

    /// Rename section `at` (its place in [`Viewer::sections`]).
    ///
    /// # Errors
    ///
    /// `CS-WSM-0006` for no such section or a bad name.
    pub fn rename_section(&mut self, at: usize, name: &str) -> Result<bool, BridgeError> {
        let name = clean(name)?;
        let id = self
            .sections()
            .get(at)
            .map(|s| s.0.id.clone())
            .ok_or(BridgeError::BadArgument)?;
        let mut list = self.sections.clone();
        if let Some(s) = list.iter_mut().find(|s| s.id == id) {
            s.name = name;
        }
        self.set_sections(list)
    }

    /// Remove section `at`; its slides join the section before it, as PowerPoint's
    /// Remove Section does.
    ///
    /// # Errors
    ///
    /// `CS-WSM-0006` for no such section.
    pub fn remove_section(&mut self, at: usize) -> Result<bool, BridgeError> {
        let id = self
            .sections()
            .get(at)
            .map(|s| s.0.id.clone())
            .ok_or(BridgeError::BadArgument)?;
        let mut list = self.sections.clone();
        list.retain(|s| s.id != id);
        self.set_sections(list)
    }

    /// The save pass for sections: removed, then renamed, then added, in the order
    /// they stand, each new one at its first slide.
    pub(crate) fn sections_pass(&self, bytes: Vec<u8>) -> Result<Vec<u8>, BridgeError> {
        if self.sections == self.original_sections {
            return Ok(bytes);
        }
        let file = casual_slide_render::Deck::open(bytes.clone())?;
        let fparts = file.parts();
        let model = &self.deck.parts().structure.slides;
        // The file's part for a model slide, by place.
        let part_name = |ordinal: u32| -> Option<String> {
            let i = model.iter().position(|&o| o == ordinal)?;
            let o = *fparts.structure.slides.get(i)?;
            Some(fparts.structure.entry(o)?.part.clone())
        };
        let mut plan = SavePlan::new();
        for old in &self.original_sections {
            if !self.sections.iter().any(|s| s.id == old.id) {
                plan = plan.op(Op::RemoveSection { id: old.id.clone() });
            }
        }
        for (now, _) in self.sections() {
            match self.original_sections.iter().find(|s| s.id == now.id) {
                Some(old) if old.name != now.name => {
                    plan = plan.op(Op::RenameSection {
                        id: now.id.clone(),
                        name: now.name.clone(),
                    });
                }
                Some(_) => {}
                None => {
                    let at = part_name(now.first).ok_or(BridgeError::Save("CS-WSM-0009"))?;
                    plan = plan.op(Op::AddSection {
                        name: now.name.clone(),
                        id: now.id.clone(),
                        at: Some(SlideRef::Part(at)),
                    });
                }
            }
        }
        Self::run(&plan, bytes)
    }
}
