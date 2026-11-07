//! Why a save plan was refused. Nothing was written. Messages name parts, ids and
//! counts, never document content.

use std::fmt;

use casual_slide_ooxml::OoxmlError;

use super::preflight::Violation;
use crate::properties::ExportError;
use crate::slide::SlideExportError;

/// Why a plan could not be saved. The session is intact.
#[derive(Debug)]
#[non_exhaustive]
pub enum WriterError {
    /// A model patch refused.
    Export(ExportError),
    /// A slide-part patch refused.
    Slide(SlideExportError),
    /// The package or the XML refused.
    Ooxml(OoxmlError),
    /// `CS-SAV-0012`: preflight found a violation this save introduced.
    Preflight {
        /// Each new violation.
        violations: Vec<Violation>,
    },
    /// `CS-SAV-0009`: an edit is blocked on a construct that has no safe form yet.
    Blocked {
        /// The construct, a fixed word.
        construct: &'static str,
        /// The part.
        part: String,
    },
    /// `CS-SAV-0017`: an operation names something the deck does not have, or asks for
    /// something the rules refuse.
    BadOperation {
        /// Why, a fixed phrase.
        what: &'static str,
        /// What it named: a part name or an id.
        subject: String,
    },
    /// `CS-SAV-0018`: a writer limit was crossed.
    TooLarge {
        /// Which limit.
        what: &'static str,
        /// The limit.
        limit: u64,
    },
    /// `CS-SAV-0019`: a number space is used up: slide ids, master ids, or a family's names.
    Exhausted {
        /// Which space.
        what: &'static str,
    },
    /// `CS-SAV-0020`: the written package failed the check made after writing it.
    Emission {
        /// The invariant.
        rule: &'static str,
        /// The entry.
        part: String,
    },
}

impl WriterError {
    /// The stable diagnostic code.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Export(err) => err.code(),
            Self::Slide(err) => err.code(),
            Self::Ooxml(err) => err.code(),
            Self::Preflight { .. } => "CS-SAV-0012",
            Self::Blocked { .. } => "CS-SAV-0009",
            Self::BadOperation { .. } => "CS-SAV-0017",
            Self::TooLarge { .. } => "CS-SAV-0018",
            Self::Exhausted { .. } => "CS-SAV-0019",
            Self::Emission { .. } => "CS-SAV-0020",
        }
    }

    /// Whether a refusal names `rule`, as `INV-PN-03`. For tests and reports.
    pub fn names_rule(&self, rule: &str) -> bool {
        match self {
            Self::Preflight { violations } => violations.iter().any(|v| v.rule == rule),
            Self::Emission { rule: r, .. } => *r == rule,
            _ => false,
        }
    }
}

impl fmt::Display for WriterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = self.code();
        match self {
            Self::Export(err) => write!(f, "{err}"),
            Self::Slide(err) => write!(f, "{err}"),
            Self::Ooxml(err) => write!(f, "{err}"),
            Self::Preflight { violations } => {
                write!(
                    f,
                    "[{code}] save refused, new violations: {}",
                    violations.len()
                )?;
                for v in violations.iter().take(8) {
                    write!(f, "; {} on {:?} ({})", v.rule, v.part, v.detail)?;
                }
                Ok(())
            }
            Self::Blocked { construct, part } => {
                write!(f, "[{code}] {part:?}: blocked on {construct}")
            }
            Self::BadOperation { what, subject } => write!(f, "[{code}] {subject:?}: {what}"),
            Self::TooLarge { what, limit } => {
                write!(f, "[{code}] {what} over its limit of {limit}")
            }
            Self::Exhausted { what } => write!(f, "[{code}] no {what} left"),
            Self::Emission { rule, part } => {
                write!(f, "[{code}] {rule} failed after writing {part:?}")
            }
        }
    }
}

impl std::error::Error for WriterError {}

impl From<OoxmlError> for WriterError {
    fn from(err: OoxmlError) -> Self {
        Self::Ooxml(err)
    }
}

impl From<ExportError> for WriterError {
    fn from(err: ExportError) -> Self {
        Self::Export(err)
    }
}

impl From<SlideExportError> for WriterError {
    fn from(err: SlideExportError) -> Self {
        Self::Slide(err)
    }
}
