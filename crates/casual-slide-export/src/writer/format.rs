//! Changing a file's type: one `Override` (`INV-CT-08`, `INV-CT-10`), and dropping a VBA
//! project on the owner's confirmation (`INV-CT-06`). The whole grid of format pairs is
//! `T1N-06`; this is what the content type rules of `T1N-02` stand on.

use super::draft::{CONTENT_TYPES_PART, Draft, rel_kind, set_attr};
use super::error::WriterError;
use super::names::key;

/// The main part's type for a file's extension (`INV-SA-01`). `None` for an extension
/// this writer does not retype to.
pub(crate) fn main_type_for(extension: &str) -> Option<&'static str> {
    Some(match extension.to_ascii_lowercase().as_str() {
        "pptx" => {
            "application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"
        }
        "ppsx" => "application/vnd.openxmlformats-officedocument.presentationml.slideshow.main+xml",
        "potx" => "application/vnd.openxmlformats-officedocument.presentationml.template.main+xml",
        "pptm" => "application/vnd.ms-powerpoint.presentation.macroEnabled.main+xml",
        "ppsm" => "application/vnd.ms-powerpoint.slideshow.macroEnabled.main+xml",
        "potm" => "application/vnd.ms-powerpoint.template.macroEnabled.main+xml",
        _ => return None,
    })
}

impl Draft<'_> {
    fn vba_relationships(&mut self) -> Result<Vec<String>, WriterError> {
        let main = self.main.clone();
        Ok(self
            .relationships(&main)?
            .into_iter()
            .filter(|r| rel_kind(&r.rel_type).ends_with("/vbaProject"))
            .map(|r| r.id)
            .collect())
    }

    /// `Op::Retype`. VBA in a macro-free type needs [`Draft::drop_vba`] first.
    pub(crate) fn retype(&mut self, extension: &str) -> Result<(), WriterError> {
        let wanted = main_type_for(extension).ok_or_else(|| WriterError::BadOperation {
            what: "not a type this writer retypes to",
            subject: extension
                .chars()
                .filter(char::is_ascii_alphanumeric)
                .take(8)
                .collect(),
        })?;
        let macro_type = wanted.contains("macroEnabled");
        if !macro_type && !self.vba_relationships()?.is_empty() {
            return Err(WriterError::VbaNeedsConfirmation {
                part: self.main.clone(),
            });
        }
        let goal = key(&self.main);
        let main = self.main.clone();
        let tree = self.tree(CONTENT_TYPES_PART)?;
        let found = tree.child_elements(tree.root()).into_iter().find(|&el| {
            tree.local_name(el) == "Override"
                && tree
                    .plain_attribute(el, "PartName")
                    .is_some_and(|n| key(&n) == goal)
        });
        match found {
            Some(el) => set_attr(tree, el, "ContentType", wanted, CONTENT_TYPES_PART)?,
            None => self.declare(&main, wanted)?,
        }
        self.log.requested_extension = Some(extension.to_ascii_lowercase());
        Ok(())
    }

    /// `Op::DropVba`: the relationship goes here, what only it reached goes by the sweep,
    /// and the `Default` naming the VBA type goes in [`Draft::settle_vba`].
    pub(crate) fn drop_vba(&mut self) -> Result<(), WriterError> {
        let ids = self.vba_relationships()?;
        let main = self.main.clone();
        if ids.is_empty() {
            return Err(WriterError::BadOperation {
                what: "the deck has no VBA project",
                subject: main,
            });
        }
        for id in &ids {
            self.remove_relationship(&main, id)?;
        }
        self.log.vba_dropped = true;
        self.note(
            "CS-SAV-0002",
            "INV-CT-06",
            &main,
            &format!("{} VBA project dropped on confirmation", ids.len()),
        );
        Ok(())
    }

    /// `INV-CT-06` (Inferred exception): after the sweep, a `Default` for the VBA type that
    /// nothing uses is removed. Every other unused `Default` stays.
    pub(crate) fn settle_vba(&mut self) -> Result<(), WriterError> {
        if !self.log.vba_dropped {
            return Ok(());
        }
        let in_use: Vec<String> = self
            .names
            .keys()
            .filter_map(|k| {
                k.rsplit('/')
                    .next()
                    .and_then(|f| f.rsplit_once('.'))
                    .map(|(_, e)| e.to_owned())
            })
            .collect();
        let tree = self.tree(CONTENT_TYPES_PART)?;
        let root = tree.root();
        for el in tree.child_elements(root) {
            if tree.local_name(el) != "Default" {
                continue;
            }
            let vba = tree
                .plain_attribute(el, "ContentType")
                .is_some_and(|ct| ct.eq_ignore_ascii_case("application/vnd.ms-office.vbaProject"));
            let used = tree
                .plain_attribute(el, "Extension")
                .is_some_and(|e| in_use.contains(&e.to_ascii_lowercase()));
            if vba && !used {
                tree.remove_child(root, el);
            }
        }
        Ok(())
    }
}
