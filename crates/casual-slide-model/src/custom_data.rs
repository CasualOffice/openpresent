//! Parts a deck carries for someone else, as plain typed values (`docs/spec/CUSTOM.md`,
//! `docs/spec/REVIEW.md`): tags, custom XML properties, schema libraries, and a list of
//! what is retained whole. Tags and properties are read and can be edited. Change tracking,
//! revision info, and the bodies of custom XML items are retained verbatim and only named.
//!
//! Tag names and values are document data, and reach no finding.

/// `p:CT_TagList`, a `tags` part.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TagList {
    /// `tag`.
    pub tags: Vec<StringTag>,
}

/// `p:CT_StringTag`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StringTag {
    /// `@name`. Document data.
    pub name: Option<String>,
    /// `@val`. Document data.
    pub val: Option<String>,
}

/// `ds:CT_DatastoreItem`, a `customXmlProps` part.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DatastoreItem {
    /// `@itemID`.
    pub item_id: Option<String>,
    // spec-audit: ds:CT_DatastoreSchemaRefs/schemaRef
    /// `schemaRefs`, as `ds:CT_DatastoreSchemaRefs`.
    pub schema_refs: Option<Vec<DatastoreSchemaRef>>,
}

/// `ds:CT_DatastoreSchemaRef`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DatastoreSchemaRef {
    /// `@uri`.
    pub uri: Option<String>,
}

/// `sl:CT_SchemaLibrary`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SchemaLibrary {
    /// `schema`.
    pub schemas: Vec<Schema>,
}

/// `sl:CT_Schema`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Schema {
    /// `@uri`.
    pub uri: Option<String>,
    /// `@manifestLocation`.
    pub manifest_location: Option<String>,
    /// `@schemaLocation`.
    pub schema_location: Option<String>,
    /// `@schemaLanguage`.
    pub schema_language: Option<String>,
}

/// A custom XML item and its properties. The item's body is the owner's data: retained
/// verbatim, and never read into the model unless it is a schema library.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CustomXmlItem {
    /// The item's part name.
    pub part: String,
    /// Its properties part's name, if it has one.
    pub properties_part: Option<String>,
    /// What the properties part says.
    pub properties: Option<DatastoreItem>,
    /// What the item says, if it is a schema library.
    pub schema_library: Option<SchemaLibrary>,
}

/// A tag part, by name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagPart {
    /// The part's name.
    pub part: String,
    /// What it says.
    pub list: TagList,
}

/// What a deck holds of its owner's data.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeckCustomData {
    /// Every `tags` part, wherever it was linked from.
    pub tags: Vec<TagPart>,
    /// Every custom XML item.
    pub custom_xml: Vec<CustomXmlItem>,
    /// Parts of these kinds that could not be read. Set aside: retained byte for byte, and
    /// named in the report (owner instruction 24).
    pub set_aside: Vec<String>,
}

/// What a retained part is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RetainedKind {
    /// `pc:chgInfo`, a `changesInfo` part.
    ChangesInfo,
    /// `p1510:revInfo`, a `revisionInfo` part.
    RevisionInfo,
    /// A custom XML item's body.
    CustomXml,
    /// Sensitivity label metadata, which is read and retained too.
    LabelInfo,
}

impl RetainedKind {
    /// A stable name, for a report.
    pub fn name(self) -> &'static str {
        match self {
            Self::ChangesInfo => "changesInfo",
            Self::RevisionInfo => "revisionInfo",
            Self::CustomXml => "customXml",
            Self::LabelInfo => "classificationlabels",
        }
    }
}

/// One part kept whole.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RetainedPart {
    /// The part's name.
    pub part: String,
    /// What it is.
    pub kind: RetainedKind,
}
