//! What was repaired or ignored. Hostile input can produce many of these, so the
//! list is bounded and the count stays exact.

/// Most diagnostics kept per compile or evaluation.
const MAX_KEPT: usize = 32;
/// Longest file-supplied text kept in a diagnostic, in bytes.
const MAX_SUBJECT: usize = 64;

/// One repair or omission.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Diagnostic {
    /// The formula's first token is no known operator. The guide evaluates to 0.
    UnknownOperator(String),
    /// Too few operands, so the guide evaluates to 0; or surplus, which are ignored.
    WrongOperandCount(String),
    /// A name not defined at this point, which may be a forward reference. It reads
    /// as 0.
    UnknownGuide(String),
    /// A formula or guide name beyond the length limit. Treated as absent.
    TooLong,
    /// An adjust value naming no adjust of this geometry. Ignored.
    UnknownAdjust(String),
    /// More adjust values than the guide cap. The rest are ignored.
    TooManyAdjusts,
    /// A path that draws before any `moveTo`. It starts at the origin.
    PathWithoutMoveTo,
}

/// A bounded list of diagnostics.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Diagnostics {
    kept: Vec<Diagnostic>,
    total: usize,
}

impl Diagnostics {
    /// Records one, keeping it if there is room.
    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.total = self.total.saturating_add(1);
        if self.kept.len() < MAX_KEPT {
            self.kept.push(diagnostic);
        }
    }

    /// Those kept, oldest first.
    pub fn kept(&self) -> &[Diagnostic] {
        &self.kept
    }

    /// How many were recorded, kept or not.
    pub fn total(&self) -> usize {
        self.total
    }

    /// Whether anything was recorded.
    pub fn is_empty(&self) -> bool {
        self.total == 0
    }
}

/// File text, cut to the limit on a character boundary.
pub(crate) fn clip(text: &str) -> String {
    let mut end = text.len().min(MAX_SUBJECT);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    text.get(..end).unwrap_or_default().to_owned()
}
