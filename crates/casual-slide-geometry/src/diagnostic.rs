//! What was repaired or ignored. Hostile input can produce many of these, so the
//! list is bounded and the count stays exact.

/// Most diagnostics kept per compile or evaluation.
const MAX_KEPT: usize = 32;

/// One repair or omission. Says what happened, and never holds text from the file
/// (`SEC-019`): a formula and a guide name are the part's to choose. The part keeps them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Diagnostic {
    /// The formula's first token is no known operator. The guide evaluates to 0.
    UnknownOperator,
    /// Too few operands, so the guide evaluates to 0; or surplus, which are ignored.
    WrongOperandCount,
    /// A name not defined at this point, which may be a forward reference. It reads
    /// as 0.
    UnknownGuide,
    /// A formula or guide name beyond the length limit. Treated as absent.
    TooLong,
    /// An adjust value naming no adjust of this geometry. Ignored.
    UnknownAdjust,
    /// More adjust values than the guide cap. The rest are ignored.
    TooManyAdjusts,
    /// A path that draws before any `moveTo`. It starts at the origin.
    PathWithoutMoveTo,
}

impl Diagnostic {
    /// The stable diagnostic code, in `docs/20`.
    pub fn code(self) -> &'static str {
        match self {
            Self::UnknownOperator => "CS-GEF-0001",
            Self::WrongOperandCount => "CS-GEF-0002",
            Self::UnknownGuide => "CS-GEF-0003",
            Self::TooLong => "CS-GEF-0004",
            Self::UnknownAdjust => "CS-GEF-0005",
            Self::TooManyAdjusts => "CS-GEF-0006",
            Self::PathWithoutMoveTo => "CS-GEF-0007",
        }
    }
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
