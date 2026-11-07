//! The save-as matrix of `docs/36` (`WR-05`, `T1N-06`): what each pair of formats does.
//! A retype is one `Override`, plus the VBA project's drop where that was confirmed.
//! What needs a tier not built yet says which, under `CS-SAV-0013`.

use casual_slide_ooxml::{OoxmlLimits, PresentationPackage};

use super::draft::WriterOutcome;
use super::error::WriterError;
use super::ops::Op;
use super::plan::SavePlan;

/// A format on the matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FileFormat {
    /// `.pptx`
    Pptx,
    /// `.pptm`
    Pptm,
    /// `.ppsx`
    Ppsx,
    /// `.ppsm`
    Ppsm,
    /// `.potx`
    Potx,
    /// `.potm`
    Potm,
    /// `.thmx`
    Thmx,
    /// `.ppam`
    Ppam,
    /// `.odp`
    Odp,
    /// `.pdf`
    Pdf,
    /// `.ppt`
    Ppt,
}

impl FileFormat {
    /// Every format, in the order of the grid in `docs/36`.
    pub const ALL: [Self; 11] = [
        Self::Pptx,
        Self::Pptm,
        Self::Ppsx,
        Self::Ppsm,
        Self::Potx,
        Self::Potm,
        Self::Thmx,
        Self::Ppam,
        Self::Odp,
        Self::Pdf,
        Self::Ppt,
    ];

    /// The extension, with no dot.
    pub fn extension(self) -> &'static str {
        match self {
            Self::Pptx => "pptx",
            Self::Pptm => "pptm",
            Self::Ppsx => "ppsx",
            Self::Ppsm => "ppsm",
            Self::Potx => "potx",
            Self::Potm => "potm",
            Self::Thmx => "thmx",
            Self::Ppam => "ppam",
            Self::Odp => "odp",
            Self::Pdf => "pdf",
            Self::Ppt => "ppt",
        }
    }

    fn is_macro_type(self) -> bool {
        matches!(self, Self::Pptm | Self::Ppsm | Self::Potm)
    }
}

/// A cell of the grid, in the codes of `docs/36`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveAsCell {
    /// `=`: same format.
    Same,
    /// `T`: retype, no VBA present.
    Retype,
    /// `K`: retype, keeping VBA.
    RetypeKeepVba,
    /// `V`: retype, dropping VBA on confirmation.
    RetypeDropVba,
    /// `N`: a new deck from a theme.
    NewFromTheme,
    /// `C`: convert through the model.
    Convert,
    /// `CV`: convert, dropping VBA on confirmation.
    ConvertDropVba,
    /// `R`: render through Tier 2G.
    Render,
    /// `X`: refused.
    Refused,
}

impl SaveAsCell {
    /// The code `docs/36` writes for this cell.
    pub fn code(self) -> &'static str {
        match self {
            Self::Same => "=",
            Self::Retype => "T",
            Self::RetypeKeepVba => "K",
            Self::RetypeDropVba => "V",
            Self::NewFromTheme => "N",
            Self::Convert => "C",
            Self::ConvertDropVba => "CV",
            Self::Render => "R",
            Self::Refused => "X",
        }
    }
}

/// The cell for opening `open` and saving as `save`.
pub fn save_as_cell(open: FileFormat, save: FileFormat) -> SaveAsCell {
    use FileFormat as F;
    use SaveAsCell as C;
    if open == save {
        return if matches!(open, F::Pdf | F::Ppt) {
            C::Refused
        } else {
            C::Same
        };
    }
    let package_row = !matches!(open, F::Thmx | F::Ppam | F::Odp | F::Pdf | F::Ppt);
    let package_column = !matches!(save, F::Thmx | F::Ppam | F::Odp | F::Pdf | F::Ppt);
    match (open, save) {
        (F::Pdf, _) | (F::Ppam, _) | (_, F::Thmx | F::Ppam | F::Ppt) => C::Refused,
        (_, F::Pdf) => C::Render,
        (F::Pptm | F::Ppsm | F::Potm, F::Odp) => C::ConvertDropVba,
        (_, F::Odp) => C::Convert,
        (F::Thmx, _) => C::NewFromTheme,
        (F::Odp, _) => C::Convert,
        (F::Ppt, _) if save.is_macro_type() => C::Refused,
        (F::Ppt, _) => C::Convert,
        _ if package_row && package_column => {
            if !open.is_macro_type() {
                C::Retype
            } else if save.is_macro_type() {
                C::RetypeKeepVba
            } else {
                C::RetypeDropVba
            }
        }
        _ => C::Refused,
    }
}

fn has_vba(package: &PresentationPackage) -> bool {
    package
        .relationships_of(package.main_part())
        .iter()
        .any(|r| r.rel_type.ends_with("/vbaProject"))
}

/// Save `input`, opened as `open`, as `save`, with `plan`'s edits. `confirm_vba_drop` is the
/// host's confirmation for `V`; without it a macro-free type over VBA is `CS-SAV-0003`.
pub fn save_as(
    input: &[u8],
    open: FileFormat,
    save: FileFormat,
    plan: SavePlan<'_>,
    confirm_vba_drop: bool,
    limits: OoxmlLimits,
) -> Result<WriterOutcome, WriterError> {
    let refused = |why: &'static str| WriterError::FormatPair {
        open: open.extension(),
        save: save.extension(),
        why,
    };
    match save_as_cell(open, save) {
        SaveAsCell::Refused => Err(refused("this pair is not offered")),
        SaveAsCell::NewFromTheme => Err(refused(
            "a deck from a theme waits for a PowerPoint-written .thmx (F-007)",
        )),
        // `.ppt` is refused by name: `INV-SA-10`.
        SaveAsCell::Convert | SaveAsCell::ConvertDropVba if open == FileFormat::Ppt => {
            match PresentationPackage::open(input.to_vec(), limits) {
                Err(err) => Err(err.into()),
                Ok(_) => Err(refused("conversion waits for Phase 10")),
            }
        }
        SaveAsCell::Convert | SaveAsCell::ConvertDropVba => {
            Err(refused("conversion waits for Phase 10"))
        }
        SaveAsCell::Render => Err(refused("rendering waits for Tier 2G")),
        SaveAsCell::Same if plan.is_unchanged() => Ok(WriterOutcome::unedited(input.to_vec())),
        SaveAsCell::Same => {
            // `INV-SA-08`, `INV-SA-09`, `INV-SA-11`: unedited or not at all.
            if matches!(open, FileFormat::Thmx | FileFormat::Ppam | FileFormat::Odp) {
                return Err(WriterError::Blocked {
                    construct: "a format this writer does not edit",
                    part: open.extension().to_owned(),
                });
            }
            let mut package = PresentationPackage::open(input.to_vec(), limits)?;
            plan.save(&mut package)
        }
        SaveAsCell::Retype | SaveAsCell::RetypeKeepVba | SaveAsCell::RetypeDropVba => {
            let mut package = PresentationPackage::open(input.to_vec(), limits)?;
            let mut plan = plan;
            if !save.is_macro_type() && confirm_vba_drop && has_vba(&package) {
                plan = plan.op(Op::DropVba);
            }
            plan.op(Op::Retype {
                extension: save.extension().to_owned(),
            })
            .save(&mut package)
        }
    }
}

/// `INV-SA-07`: *New from template*. The result is a `.pptx` with no VBA, no signature, and,
/// with a time in `plan`'s host inputs, a new `created`. Nothing needs confirming, since a
/// new deck never inherited the project.
pub fn new_from_template(
    input: &[u8],
    plan: SavePlan<'_>,
    limits: OoxmlLimits,
) -> Result<WriterOutcome, WriterError> {
    let mut package = PresentationPackage::open(input.to_vec(), limits)?;
    let mut plan = plan.acknowledge_signature().new_created();
    if has_vba(&package) {
        plan = plan.op(Op::DropVba);
    }
    plan.op(Op::Retype {
        extension: "pptx".to_owned(),
    })
    .save(&mut package)
}
