//! The one writer of Tier 1N: a [`SavePlan`] holds every change to a deck, and is
//! applied atomically or not at all (`docs/36-EXPORT-AND-SAVE.md`).
//!
//! The pipeline is staged, so a refused plan leaves nothing behind:
//!
//! 1. **Stage.** [`SavePlan::stage`] builds a [`Draft`] over the input: model patches
//!    first (deck, then slide parts), then replaced parts, then file-level operations
//!    in the order they were asked for, then the sweep of what the edits orphaned.
//! 2. **Preflight.** [`Draft::preflight`] runs every implemented `INV-*` rule on the
//!    input and on the draft. A violation this save introduced refuses it with
//!    `CS-SAV-0012`; one that arrived in the input is reported and left alone, because a
//!    third-party deck that already breaks a rule must still open and save.
//! 3. **Emit.** [`Draft::emit`] writes through `Package::rewrite`, then checks the
//!    result at ZIP level before anyone sees it: every part not named keeps its
//!    compressed bytes, and headers agree.
//!
//! PowerPoint was not available when these rules were written or built. Marks in the
//! comments are those of `docs/36`, kept as they stand there: a rule marked Inferred or
//! Unverified is implemented, and `PV-01` is what confirms or overturns it.

mod clone;
mod draft;
mod error;
mod format;
mod names;
mod notes;
mod ops;
mod plan;
mod preflight;
mod shapes;
mod structure;
mod view;
mod zipcheck;

pub use draft::{Disposition, Draft, ReportEntry, WriterOutcome};
pub use error::WriterError;
pub use ops::{Op, SlideRef};
pub use plan::{SavePlan, WriterLimits, repair_deck};
pub use preflight::{Preflight, Violation};
pub use zipcheck::check_zip;

pub(crate) use draft::read_relationships;
pub(crate) use names::{next_rel_id, target_for};
