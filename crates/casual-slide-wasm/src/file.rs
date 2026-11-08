//! The File tab's own verbs (`FILE-01`): File > Info's properties and Advanced
//! Properties, Save As, and the edit counter autosave polls.
//!
//! Properties are the model's ([`DocumentProperties`], Tier 1B), edited as one
//! [`Operation::SetDocumentProperties`] per change, so Ctrl+Z takes a property back as
//! it takes back a shape. A save patches `docProps/core.xml`, `app.xml` and `custom.xml`
//! through the one writer ([`SavePlan::deck`]), node by node (ADR-013): a property
//! nobody touched is never rewritten, and one that was never stated stays absent.

use casual_slide_export::writer::{FileFormat, SavePlan, save_as};
use casual_slide_import::presentation::import_deck;
use casual_slide_import::properties::ImportLimits;
use casual_slide_model::properties::{
    AppProperties, CoreProperties, CustomProperties, CustomProperty, DocumentProperties, Float64,
    Keywords, Lexical, Variant, W3cDateTime,
};
use casual_slide_model::shape::ShapeNode;
use casual_slide_model::text::{TextBody, TextItem};
use casual_slide_ooxml::{OoxmlLimits, PresentationKind, PresentationPackage};
use casual_slide_transaction::Operation;
use wasm_bindgen::prelude::*;

use crate::design::DeckDoc;
use crate::{BridgeError, Viewer, js, with_viewer};

/// The properties File > Info edits, in the order it shows them, as the bridge names
/// them: Title, Subject, Author, Tags, Categories, Comments, Company, Manager.
pub const PROPERTY_KEYS: [&str; 8] = [
    "title",
    "subject",
    "author",
    "tags",
    "categories",
    "comments",
    "company",
    "manager",
];

/// The longest property value kept, in characters. Office's own fields are far
/// shorter; this only stops a paste of a book from reaching the file.
pub const MAX_PROPERTY_CHARS: usize = 32_767;

/// The longest custom property name, in characters: Office's limit.
pub const MAX_CUSTOM_NAME_CHARS: usize = 255;

/// The `fmtid` every user-defined custom property carries.
const USER_FMTID: &str = "{D5CDD505-2E9C-101B-9397-08002B2CF9AE}";

/// The first `pid` a custom property may take: 0 and 1 are reserved.
const FIRST_PID: i32 = 2;

/// The type of a custom property, as Advanced Properties offers it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomKind {
    /// Text, `vt:lpwstr`.
    Text,
    /// A number: `vt:i4` when whole, `vt:r8` otherwise.
    Number,
    /// A date, `vt:filetime`, from `YYYY-MM-DD`.
    Date,
    /// Yes or no, `vt:bool`.
    YesNo,
}

impl CustomKind {
    /// From the bridge's number: 0 text, 1 number, 2 date, 3 yes or no.
    pub fn from_code(code: u32) -> Option<Self> {
        Some(match code {
            0 => Self::Text,
            1 => Self::Number,
            2 => Self::Date,
            3 => Self::YesNo,
            _ => return None,
        })
    }

    /// The bridge's name for it, as `document_info` reports it.
    fn name(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Number => "number",
            Self::Date => "date",
            Self::YesNo => "yesno",
        }
    }

    /// The value this kind writes for `text`, if `text` is one.
    fn variant(self, text: &str) -> Option<Variant> {
        let t = text.trim();
        Some(match self {
            Self::Text => Variant::Lpwstr(clean(text)),
            Self::Number => match t.parse::<i32>() {
                Ok(n) => Variant::I4(n),
                Err(_) => {
                    let f = Float64::parse_lexical(t)?;
                    if !f.get().is_finite() {
                        return None;
                    }
                    Variant::R8(f)
                }
            },
            Self::Date => {
                let day = t.get(..10).unwrap_or(t);
                let stamp = format!("{day}T00:00:00Z");
                W3cDateTime::parse_lexical(&stamp)?;
                if day.len() != 10 {
                    return None;
                }
                Variant::Filetime(stamp)
            }
            Self::YesNo => match t.to_ascii_lowercase().as_str() {
                "yes" | "true" | "1" => Variant::Bool(true),
                "no" | "false" | "0" => Variant::Bool(false),
                _ => return None,
            },
        })
    }

    /// The kind a stored value shows as, and its text; `None` for a value Advanced
    /// Properties does not edit (a vector, a blob), which is kept as it is.
    fn of(value: &Variant) -> Option<(Self, String)> {
        Some(match value {
            Variant::Lpwstr(s) | Variant::Lpstr(s) | Variant::Bstr(s) => (Self::Text, s.clone()),
            Variant::I1(n) => (Self::Number, n.to_string()),
            Variant::I2(n) => (Self::Number, n.to_string()),
            Variant::I4(n) | Variant::Int(n) => (Self::Number, n.to_string()),
            Variant::I8(n) => (Self::Number, n.to_string()),
            Variant::Ui1(n) => (Self::Number, n.to_string()),
            Variant::Ui2(n) => (Self::Number, n.to_string()),
            Variant::Ui4(n) | Variant::Uint(n) => (Self::Number, n.to_string()),
            Variant::Ui8(n) => (Self::Number, n.to_string()),
            Variant::R4(f) => (Self::Number, f.to_lexical()),
            Variant::R8(f) => (Self::Number, f.to_lexical()),
            Variant::Decimal(s) => (Self::Number, s.clone()),
            Variant::Date(s) | Variant::Filetime(s) => {
                (Self::Date, s.get(..10).unwrap_or(s).to_owned())
            }
            Variant::Bool(b) => (Self::YesNo, if *b { "Yes" } else { "No" }.to_owned()),
            _ => return None,
        })
    }
}

/// `text` without the characters XML 1.0 cannot hold, and no longer than
/// [`MAX_PROPERTY_CHARS`]: what is typed into a field always saves.
fn clean(text: &str) -> String {
    text.chars()
        .filter(|&c| {
            matches!(c, '\t' | '\n' | '\r') || (c >= ' ' && c != '\u{FFFE}' && c != '\u{FFFF}')
        })
        .take(MAX_PROPERTY_CHARS)
        .collect()
}

/// A field's new value: `None` for an empty one, so clearing a field removes the
/// element rather than writing it empty.
fn field(value: &str) -> Option<String> {
    let v = clean(value);
    (!v.is_empty()).then_some(v)
}

fn keywords_text(k: &Keywords) -> String {
    let mut parts: Vec<&str> = Vec::new();
    if !k.text.trim().is_empty() {
        parts.push(k.text.trim());
    }
    parts.extend(k.values.iter().map(|v| v.text.as_str()));
    parts.join("; ")
}

fn count_words(body: &TextBody) -> usize {
    body.paragraphs
        .iter()
        .map(|p| {
            let text: String = p
                .items
                .iter()
                .filter_map(|item| match item {
                    TextItem::Run(r) => r.text.as_deref(),
                    TextItem::Field(f) => f.text.as_deref(),
                    _ => None,
                })
                .collect();
            text.split_whitespace().count()
        })
        .sum()
}

fn words_in(node: &ShapeNode) -> usize {
    node.descendants()
        .map(|n| {
            let own = n.text_body.as_deref().map_or(0, count_words);
            let cells: usize = n
                .table
                .iter()
                .flat_map(|t| t.rows.iter())
                .flat_map(|r| &r.cells)
                .filter_map(|c| c.tx_body.as_deref())
                .map(count_words)
                .sum();
            own.saturating_add(cells)
        })
        .fold(0usize, usize::saturating_add)
}

fn format_of(kind: PresentationKind) -> FileFormat {
    match kind {
        PresentationKind::Presentation => FileFormat::Pptx,
        PresentationKind::Slideshow => FileFormat::Ppsx,
        PresentationKind::Template => FileFormat::Potx,
        PresentationKind::MacroPresentation => FileFormat::Pptm,
        PresentationKind::MacroSlideshow => FileFormat::Ppsm,
        PresentationKind::MacroTemplate => FileFormat::Potm,
    }
}

impl Viewer {
    /// One edit of the properties, as one undo step. Whether anything changed.
    fn set_properties(&mut self, next: DocumentProperties) -> Result<bool, BridgeError> {
        if next == *self.deck.properties() {
            return Ok(false);
        }
        self.typing = None;
        let op = Operation::SetDocumentProperties {
            properties: Box::new(next),
        };
        Ok(self
            .history
            .apply(&mut DeckDoc(&mut self.deck, &mut self.review, None), op)?)
    }

    /// The value File > Info shows for `key`, one of [`PROPERTY_KEYS`]; `None` when the
    /// file does not state it.
    pub fn property(&self, key: &str) -> Result<Option<String>, BridgeError> {
        if !PROPERTY_KEYS.contains(&key) {
            return Err(BridgeError::BadArgument);
        }
        Ok(property_of(self.deck.properties(), key))
    }

    /// Give property `key`, one of [`PROPERTY_KEYS`], this value: one undo step. An
    /// empty value removes it. Whether anything changed.
    pub fn set_property(&mut self, key: &str, value: &str) -> Result<bool, BridgeError> {
        let mut next = self.deck.properties().clone();
        let v = field(value);
        fn core(p: &mut DocumentProperties) -> &mut CoreProperties {
            p.core.get_or_insert_with(CoreProperties::default)
        }
        fn app(p: &mut DocumentProperties) -> &mut AppProperties {
            p.app.get_or_insert_with(AppProperties::default)
        }
        match key {
            "title" => core(&mut next).title = v,
            "subject" => core(&mut next).subject = v,
            "author" => core(&mut next).creator = v,
            "tags" => {
                let c = core(&mut next);
                let lang = c.keywords.as_ref().and_then(|k| k.lang.clone());
                c.keywords = v.map(|text| Keywords {
                    lang,
                    text,
                    values: Vec::new(),
                });
            }
            "categories" => core(&mut next).category = v,
            "comments" => core(&mut next).description = v,
            "company" => app(&mut next).company = v,
            "manager" => app(&mut next).manager = v,
            _ => return Err(BridgeError::BadArgument),
        }
        // A part made only to hold a value that was then cleared is not made.
        if next
            .core
            .as_ref()
            .is_some_and(|c| *c == CoreProperties::default())
            && self.original_properties.core.is_none()
        {
            next.core = None;
        }
        if next
            .app
            .as_ref()
            .is_some_and(|a| *a == AppProperties::default())
            && self.original_properties.app.is_none()
        {
            next.app = None;
        }
        // Treat "the same text as the file states" as no change, so a field that is only
        // tabbed through never edits the deck.
        if self.property(key)? == property_of(&next, key) {
            return Ok(false);
        }
        self.set_properties(next)
    }

    /// Advanced Properties' Custom list: each named property's name, kind and value,
    /// in file order. A value it cannot edit shows as `other`, and is kept.
    pub fn custom_properties(&self) -> Vec<(String, &'static str, String)> {
        let Some(custom) = self.deck.properties().custom.as_ref() else {
            return Vec::new();
        };
        custom
            .properties
            .iter()
            .filter_map(|p| {
                let name = p.name.clone()?;
                Some(match p.value.as_ref().and_then(CustomKind::of) {
                    Some((kind, text)) => (name, kind.name(), text),
                    None => (name, "other", String::new()),
                })
            })
            .collect()
    }

    /// Add custom property `name`, or give the one of that name (compared without
    /// case, as Office does) this kind and value: one undo step. `BadArgument` for an
    /// empty name or a value that is not of the kind.
    pub fn set_custom_property(
        &mut self,
        name: &str,
        kind: CustomKind,
        value: &str,
    ) -> Result<bool, BridgeError> {
        let name: String = clean(name.trim())
            .chars()
            .take(MAX_CUSTOM_NAME_CHARS)
            .collect();
        if name.is_empty() {
            return Err(BridgeError::BadArgument);
        }
        let variant = kind.variant(value).ok_or(BridgeError::BadArgument)?;
        let mut next = self.deck.properties().clone();
        let custom = next.custom.get_or_insert_with(CustomProperties::default);
        let same = |p: &CustomProperty| {
            p.name
                .as_deref()
                .is_some_and(|n| n.to_lowercase() == name.to_lowercase())
        };
        if let Some(p) = custom.properties.iter_mut().find(|p| same(p)) {
            p.value = Some(variant);
        } else {
            let pid = custom
                .properties
                .iter()
                .map(|p| p.pid)
                .max()
                .map_or(FIRST_PID, |m| m.saturating_add(1).max(FIRST_PID));
            custom.properties.push(CustomProperty {
                fmtid: USER_FMTID.to_owned(),
                pid,
                name: Some(name),
                link_target: None,
                value: Some(variant),
            });
        }
        self.set_properties(next)
    }

    /// Remove custom property `name` (compared without case): one undo step.
    pub fn remove_custom_property(&mut self, name: &str) -> Result<bool, BridgeError> {
        let mut next = self.deck.properties().clone();
        let Some(custom) = next.custom.as_mut() else {
            return Ok(false);
        };
        let wanted = name.trim().to_lowercase();
        custom
            .properties
            .retain(|p| p.name.as_deref().is_none_or(|n| n.to_lowercase() != wanted));
        if custom.properties.is_empty() && self.original_properties.custom.is_none() {
            next.custom = None;
        }
        self.set_properties(next)
    }

    /// The words on the slides, as File > Info counts them: runs and fields of every
    /// shape and table cell, split at white space.
    pub fn word_count(&self) -> usize {
        let parts = self.deck.parts();
        parts
            .structure
            .slides
            .iter()
            .filter_map(|&o| parts.get(o)?.common.as_ref()?.shape_tree.as_ref())
            .map(words_in)
            .fold(0usize, usize::saturating_add)
    }

    /// Everything File > Info shows, as JSON: each of [`PROPERTY_KEYS`] the file
    /// states, `created`, `modified`, `lastModifiedBy`, `slides`, `words`, `format` (the
    /// opened file's extension) and `custom`, `[name, kind, value]` each. Values are
    /// document text: a page shows them through `textContent` only.
    pub fn document_info(&self) -> Result<String, BridgeError> {
        let mut out = serde_json::Map::new();
        for key in PROPERTY_KEYS {
            if let Some(v) = self.property(key)? {
                out.insert(key.to_owned(), v.into());
            }
        }
        if let Some(core) = self.deck.properties().core.as_ref() {
            if let Some(d) = &core.created {
                out.insert("created".into(), d.as_str().into());
            }
            if let Some(d) = &core.modified {
                out.insert("modified".into(), d.as_str().into());
            }
            if let Some(u) = &core.last_modified_by {
                out.insert("lastModifiedBy".into(), u.clone().into());
            }
        }
        out.insert("slides".into(), self.slide_count().into());
        out.insert("words".into(), self.word_count().into());
        out.insert("format".into(), self.format()?.extension().into());
        let custom: Vec<serde_json::Value> = self
            .custom_properties()
            .into_iter()
            .map(|(n, k, v)| serde_json::json!([n, k, v]))
            .collect();
        out.insert("custom".into(), custom.into());
        Ok(serde_json::Value::Object(out).to_string())
    }

    /// The format the deck was opened as, from its main part's content type.
    pub fn format(&self) -> Result<FileFormat, BridgeError> {
        let package = PresentationPackage::open(self.deck.source_bytes(), OoxmlLimits::default())
            .map_err(|e| BridgeError::Save(e.code()))?;
        Ok(format_of(package.kind()))
    }

    /// The deck saved as `extension` (File > Save As): `pptx`, `ppsx`, `potx`, or a
    /// macro-enabled one, through the save-as grid of `docs/36`. Going from a deck with
    /// macros to a type without needs `drop_macros`, the user's yes; the project is never run.
    pub fn save_as(&self, extension: &str, drop_macros: bool) -> Result<Vec<u8>, BridgeError> {
        let wanted = extension.trim_start_matches('.').to_ascii_lowercase();
        let target = [
            FileFormat::Pptx,
            FileFormat::Ppsx,
            FileFormat::Potx,
            FileFormat::Pptm,
            FileFormat::Ppsm,
            FileFormat::Potm,
        ]
        .into_iter()
        .find(|f| f.extension() == wanted)
        .ok_or(BridgeError::BadArgument)?;
        let bytes = self.save()?;
        let open = {
            let package = PresentationPackage::open(bytes.clone(), OoxmlLimits::default())
                .map_err(|e| BridgeError::Save(e.code()))?;
            format_of(package.kind())
        };
        save_as(
            &bytes,
            open,
            target,
            SavePlan::new(),
            drop_macros,
            OoxmlLimits::default(),
        )
        .map(|o| o.bytes)
        .map_err(|e| BridgeError::Save(e.code()))
    }

    /// The last save pass: the property parts brought to File > Info's edits. Only
    /// what was edited is taken from the session; the rest is read from the file the
    /// earlier passes left, so the slide counts they kept in `app.xml` stand.
    pub(crate) fn properties_pass(&self, bytes: Vec<u8>) -> Result<Vec<u8>, BridgeError> {
        let now = self.deck.properties();
        let before = &self.original_properties;
        if now == before {
            return Ok(bytes);
        }
        let save = |code: &'static str| BridgeError::Save(code);
        let mut package =
            PresentationPackage::open(bytes, OoxmlLimits::default()).map_err(|e| save(e.code()))?;
        let mut model = import_deck(&mut package, &ImportLimits::default())
            .map_err(|e| save(e.code()))?
            .deck;
        if now.core != before.core {
            model.properties.core = now.core.clone();
        }
        if now.custom != before.custom {
            model.properties.custom = now.custom.clone();
        }
        let pick = |p: &Option<AppProperties>| {
            p.as_ref()
                .map(|a| (a.company.clone(), a.manager.clone()))
                .unwrap_or_default()
        };
        let (company, manager) = pick(&now.app);
        if (company.clone(), manager.clone()) != pick(&before.app) {
            if now.app.is_none() {
                model.properties.app = None;
            } else {
                let app = model
                    .properties
                    .app
                    .get_or_insert_with(AppProperties::default);
                app.company = company;
                app.manager = manager;
            }
        }
        Ok(SavePlan::new()
            .deck(&model)
            .save(&mut package)
            .map_err(|e| save(e.code()))?
            .bytes)
    }

    /// Edits applied since the deck opened, counting up and never down (an undo is an
    /// edit): what autosave polls to know there is something new to keep.
    pub fn edits_applied(&self) -> u64 {
        self.history.edits_applied()
    }
}

/// Property `key`'s value in `p`.
fn property_of(p: &DocumentProperties, key: &str) -> Option<String> {
    let core = p.core.as_ref();
    let app = p.app.as_ref();
    match key {
        "title" => core.and_then(|c| c.title.clone()),
        "subject" => core.and_then(|c| c.subject.clone()),
        "author" => core.and_then(|c| c.creator.clone()),
        "tags" => core.and_then(|c| c.keywords.as_ref()).map(keywords_text),
        "categories" => core.and_then(|c| c.category.clone()),
        "comments" => core.and_then(|c| c.description.clone()),
        "company" => app.and_then(|a| a.company.clone()),
        "manager" => app.and_then(|a| a.manager.clone()),
        _ => None,
    }
}

/// File > Info: the open deck's properties, counts and custom properties, as JSON
/// (see [`Viewer::document_info`]). Values are document text.
///
/// # Errors
///
/// `CS-WSM-0001` with no deck.
#[wasm_bindgen]
pub fn document_info() -> Result<String, JsError> {
    with_viewer(|v| v.document_info())
}

/// Give a File > Info property a value; empty removes it. One undo step. Whether
/// anything changed.
///
/// # Errors
///
/// `CS-WSM-0001` with no deck, `CS-WSM-0006` for a key not in [`PROPERTY_KEYS`].
#[wasm_bindgen]
pub fn set_document_property(key: &str, value: &str) -> Result<bool, JsError> {
    with_viewer(|v| v.set_property(key, value))
}

/// Add or change a custom property: `kind` 0 text, 1 number, 2 date (`YYYY-MM-DD`),
/// 3 yes or no. One undo step.
///
/// # Errors
///
/// `CS-WSM-0006` for an empty name, an unknown kind, or a value not of the kind.
#[wasm_bindgen]
pub fn set_custom_property(name: &str, kind: u32, value: &str) -> Result<bool, JsError> {
    let kind = CustomKind::from_code(kind).ok_or_else(|| js(BridgeError::BadArgument))?;
    with_viewer(|v| v.set_custom_property(name, kind, value))
}

/// Remove a custom property by name. One undo step.
///
/// # Errors
///
/// `CS-WSM-0001` with no deck.
#[wasm_bindgen]
pub fn remove_custom_property(name: &str) -> Result<bool, JsError> {
    with_viewer(|v| v.remove_custom_property(name))
}

/// The deck as a file of another type (File > Save As): `pptx`, `ppsx`, `potx`, or a
/// macro-enabled one. `drop_macros` is the user's yes to losing a macro project.
///
/// # Errors
///
/// `CS-WSM-0006` for another extension; the writer's `CS-SAV-*` code for a pair it
/// refuses, or a macro project not confirmed for removal.
#[wasm_bindgen]
pub fn save_deck_as(extension: &str, drop_macros: bool) -> Result<Vec<u8>, JsError> {
    with_viewer(|v| v.save_as(extension, drop_macros))
}

/// Edits applied since the deck opened; counts up, never down (autosave, `SAVE-01`).
#[wasm_bindgen]
#[must_use]
pub fn edits_applied() -> f64 {
    crate::SESSION.with(|s| {
        s.borrow()
            .as_ref()
            .map_or(0.0, |v| v.edits_applied() as f64)
    })
}

#[cfg(test)]
mod tests;
