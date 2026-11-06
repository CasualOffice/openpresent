//! Package inspection errors, each with a stable code (`docs/20-ERROR-CODE-REGISTRY.md`).
//!
//! Part names may appear in a message. Document text never does, so a diagnostic
//! can be shipped to a log.

use core::fmt;

use casual_slide_package::PackageError;

/// Why a package could not be inspected. All refusals are clean.
#[derive(Debug)]
#[non_exhaustive]
pub enum OoxmlError {
    /// The underlying package could not be admitted or read.
    Package(PackageError),
    /// A part's XML was malformed.
    MalformedXml { part: String, reason: String },
    /// A part declares a document type. OOXML never needs one, and it is the
    /// route to entity expansion and external entities.
    DoctypeRefused { part: String },
    /// An XML part is larger than the walker accepts.
    XmlTooLarge { part: String, limit: usize },
    /// A part has more elements than allowed.
    TooManyElements { part: String, limit: usize },
    /// A part nests deeper than allowed.
    TooDeep { part: String, limit: usize },
    /// An element carries more attributes than allowed.
    TooManyAttributes { part: String, limit: usize },
    /// A part, or the package, declares too many relationships.
    TooManyRelationships { limit: usize },
    /// `[Content_Types].xml` has too many entries.
    TooManyContentTypes { limit: usize },
    /// A part the format requires is absent.
    MissingPart { name: String },
    /// The main part's content type is not a presentation.
    NotAPresentation { content_type: String },
    /// A Compound File: a legacy `.ppt`, or an encrypted or rights-protected deck.
    CompoundFile,
}

impl OoxmlError {
    /// The stable diagnostic code for this error.
    pub fn code(&self) -> &'static str {
        match self {
            OoxmlError::Package(err) => err.code(),
            OoxmlError::TooManyElements { .. } => "CS-XML-0001",
            OoxmlError::TooDeep { .. } => "CS-XML-0002",
            OoxmlError::TooManyAttributes { .. } => "CS-XML-0003",
            OoxmlError::MalformedXml { .. } => "CS-XML-0004",
            OoxmlError::DoctypeRefused { .. } => "CS-XML-0005",
            OoxmlError::XmlTooLarge { .. } => "CS-XML-0006",
            OoxmlError::MissingPart { .. } => "CS-OPC-0001",
            OoxmlError::TooManyRelationships { .. } => "CS-OPC-0002",
            OoxmlError::TooManyContentTypes { .. } => "CS-OPC-0003",
            OoxmlError::NotAPresentation { .. } => "CS-OPC-0004",
            OoxmlError::CompoundFile => "CS-OPC-0005",
        }
    }
}

impl fmt::Display for OoxmlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] ", self.code())?;
        match self {
            OoxmlError::Package(err) => write!(f, "{err}"),
            OoxmlError::MalformedXml { part, reason } => {
                write!(f, "malformed XML in {part:?}: {reason}")
            }
            OoxmlError::DoctypeRefused { part } => write!(f, "document type refused in {part:?}"),
            OoxmlError::XmlTooLarge { part, limit } => {
                write!(f, "{part:?} exceeds the XML size limit {limit}")
            }
            OoxmlError::TooManyElements { part, limit } => {
                write!(f, "{part:?} exceeds the element limit {limit}")
            }
            OoxmlError::TooDeep { part, limit } => {
                write!(f, "{part:?} exceeds the depth limit {limit}")
            }
            OoxmlError::TooManyAttributes { part, limit } => {
                write!(
                    f,
                    "{part:?} exceeds the per-element attribute limit {limit}"
                )
            }
            OoxmlError::TooManyRelationships { limit } => {
                write!(f, "relationship count exceeds limit {limit}")
            }
            OoxmlError::TooManyContentTypes { limit } => {
                write!(f, "content type count exceeds limit {limit}")
            }
            OoxmlError::MissingPart { name } => write!(f, "required part missing: {name:?}"),
            OoxmlError::NotAPresentation { content_type } => {
                write!(f, "main part is not a presentation: {content_type:?}")
            }
            OoxmlError::CompoundFile => {
                write!(
                    f,
                    "a Compound File: legacy .ppt, or encrypted or rights-protected"
                )
            }
        }
    }
}

impl std::error::Error for OoxmlError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            OoxmlError::Package(err) => Some(err),
            _ => None,
        }
    }
}

impl From<PackageError> for OoxmlError {
    fn from(err: PackageError) -> Self {
        OoxmlError::Package(err)
    }
}
