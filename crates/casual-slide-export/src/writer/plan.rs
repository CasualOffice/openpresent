//! A save plan: every change to one deck, held together until it is applied.

use casual_slide_import::properties::ImportLimits;
use casual_slide_import::slide::SlideLimits;
use casual_slide_model::presentation::Deck;
use casual_slide_model::slide::{DeckStructure, SlidePart};
use casual_slide_ooxml::{PresentationPackage, XmlTree};

use super::bookkeeping::HostInputs;
use super::draft::{Draft, WriterOutcome};
use super::error::WriterError;
use super::ops::Op;
use crate::presentation::deck_changes;
use crate::slide::slide_part_changes;

/// Ceilings on one plan. A host may tighten them; none can be switched off.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WriterLimits {
    /// Operations in one plan. `CS-SAV-0018`.
    pub max_plan_ops: usize,
    /// Parts one clone may copy. A hostile deck can hang any number of parts off one
    /// slide. `CS-SAV-0018`.
    pub max_clone_parts: usize,
    /// Bytes one clone may copy, on top of the package budget. `CS-SAV-0018`.
    pub max_clone_bytes: u64,
    /// Bytes of one new media part. `CS-SAV-0018`.
    pub max_media_bytes: u64,
}

impl Default for WriterLimits {
    fn default() -> Self {
        Self {
            max_plan_ops: 4_096,
            max_clone_parts: 10_000,
            max_clone_bytes: 1 << 30,
            max_media_bytes: 1 << 30,
        }
    }
}

#[derive(Debug, Clone)]
struct SlidePatches<'a> {
    structure: &'a DeckStructure,
    parts: &'a [&'a SlidePart],
    limits: &'a SlideLimits,
}

/// All the changes to a deck, in one save. Built up, then applied with
/// [`SavePlan::save`]; nothing reaches the package unless all of it passes.
///
/// Model patches run first, as the deck stands in the input, so a model holds what was
/// imported; replaced parts next; then [`Op`]s in the order they were added.
#[derive(Debug, Clone, Default)]
pub struct SavePlan<'a> {
    deck: Option<&'a Deck>,
    slides: Option<SlidePatches<'a>>,
    replacements: Vec<(String, Vec<u8>)>,
    ops: Vec<Op>,
    import_limits: ImportLimits,
    limits: WriterLimits,
    host: Option<HostInputs>,
    signature_acknowledged: bool,
    new_created: bool,
}

impl<'a> SavePlan<'a> {
    /// A plan that changes nothing.
    pub fn new() -> Self {
        Self::default()
    }

    /// The limits to read back through, and to hold the plan to.
    pub fn with_limits(mut self, import_limits: ImportLimits, limits: WriterLimits) -> Self {
        self.import_limits = import_limits;
        self.limits = limits;
        self
    }

    /// Bring properties, the presentation part, `presProps` and `viewProps` to this model.
    pub fn deck(mut self, deck: &'a Deck) -> Self {
        self.deck = Some(deck);
        self
    }

    /// Bring these slide-like parts to their models.
    pub fn slide_parts(
        mut self,
        structure: &'a DeckStructure,
        parts: &'a [&'a SlidePart],
        limits: &'a SlideLimits,
    ) -> Self {
        self.slides = Some(SlidePatches {
            structure,
            parts,
            limits,
        });
        self
    }

    /// Give an existing XML part these bytes, as `XmlTree::to_bytes` gives them after an
    /// edit. They must parse, and keep the part's root element.
    pub fn replace_part(mut self, part: impl Into<String>, bytes: Vec<u8>) -> Self {
        self.replacements.push((part.into(), bytes));
        self
    }

    /// Add a file-level operation.
    pub fn op(mut self, op: Op) -> Self {
        self.ops.push(op);
        self
    }

    /// What only the host knows: the time, the user, the application and a fresh
    /// thumbnail. With none given, `core.xml` is left alone (`docs/36`).
    pub fn host(mut self, host: HostInputs) -> Self {
        self.host = Some(host);
        self
    }

    /// The host has told the user that editing removes the deck's signatures
    /// (`INV-SG-02`). Without this, an edit of a signed deck is `CS-SAV-0015`.
    pub fn acknowledge_signature(mut self) -> Self {
        self.signature_acknowledged = true;
        self
    }

    /// `created` takes the supplied time too: a new deck from a template (`INV-SA-07`).
    pub(crate) fn new_created(mut self) -> Self {
        self.new_created = true;
        self
    }

    /// Whether the plan asks for nothing.
    pub(crate) fn is_unchanged(&self) -> bool {
        self.deck.is_none()
            && self.slides.is_none()
            && self.replacements.is_empty()
            && self.ops.is_empty()
    }

    /// Apply the plan to a draft, with nothing written. This is where a test hook can
    /// reach in, and what [`SavePlan::save`] is made of.
    pub fn stage<'p>(
        &self,
        package: &'p mut PresentationPackage,
    ) -> Result<Draft<'p>, WriterError> {
        if self.ops.len() > self.limits.max_plan_ops {
            return Err(WriterError::TooLarge {
                what: "operations in one plan",
                limit: self.limits.max_plan_ops as u64,
            });
        }
        // Model patches read the input as it stands, before the draft holds it.
        let deck_edits = match self.deck {
            Some(deck) => deck_changes(package, deck, &self.import_limits)?,
            None => Vec::new(),
        };
        let slide_edits = match &self.slides {
            Some(s) => {
                slide_part_changes(package, s.structure, s.parts, s.limits, &self.import_limits)?
            }
            None => Vec::new(),
        };
        let mut draft = Draft::new(package, self.limits);
        for (name, bytes) in deck_edits.into_iter().chain(slide_edits) {
            draft.apply_patched(&name, bytes);
        }
        for (name, bytes) in &self.replacements {
            let actual = draft
                .actual(name)
                .ok_or_else(|| WriterError::BadOperation {
                    what: "no such part",
                    subject: name.clone(),
                })?;
            let new = XmlTree::parse(bytes, &actual, &draft.xml)?;
            let old = draft.tree(&actual)?;
            if old.local_name(old.root()) != new.local_name(new.root()) {
                return Err(WriterError::BadOperation {
                    what: "a replacement with another root element",
                    subject: actual,
                });
            }
            draft.set_part_bytes(&actual, bytes.clone());
        }
        for op in &self.ops {
            draft.apply(op)?;
        }
        draft.log.retype_only = self.deck.is_none()
            && self.slides.is_none()
            && self.replacements.is_empty()
            && !self.ops.is_empty()
            && self
                .ops
                .iter()
                .all(|op| matches!(op, Op::Retype { .. } | Op::DropVba));
        draft.sweep()?;
        draft.settle_vba()?;
        draft.bookkeep(
            self.host.as_ref(),
            self.signature_acknowledged,
            self.new_created,
        )?;
        Ok(draft)
    }

    /// Apply the plan, run preflight, and write. With nothing dirty, the input bytes.
    pub fn save(&self, package: &mut PresentationPackage) -> Result<WriterOutcome, WriterError> {
        self.stage(package)?.emit()
    }
}
