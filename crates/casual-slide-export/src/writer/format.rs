//! Changing a file's type: one `Override` (`INV-CT-08`, `INV-CT-10`), and dropping a VBA
//! project on the owner's confirmation (`INV-CT-06`). The whole grid of format pairs is
//! `T1N-06`; this is what the content type rules of `T1N-02` stand on.

use super::draft::{CONTENT_TYPES_PART, Draft, rel_kind, set_attr};
use super::error::WriterError;
use super::names::key;
use super::ops::all_elements;
use super::preflight::{Found, add};
use casual_slide_ooxml::XmlTree;

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

/// Whether this is a main part's type, of the seven (`INV-SA-01`).
pub(crate) fn is_main_type(content_type: &str) -> bool {
    ["pptx", "ppsx", "potx", "pptm", "ppsm", "potm"]
        .iter()
        .filter_map(|e| main_type_for(e))
        .any(|t| t.eq_ignore_ascii_case(content_type))
        || content_type
            .eq_ignore_ascii_case("application/vnd.ms-powerpoint.addin.macroEnabled.main+xml")
}

const MACRO_ACTION: &[u8] = b"ppaction://macro";

fn holds_macro_action(bytes: &[u8]) -> bool {
    bytes
        .windows(MACRO_ACTION.len())
        .any(|w| w.eq_ignore_ascii_case(MACRO_ACTION))
}

fn slide_like(name_key: &str) -> bool {
    !name_key.contains("/_rels/")
        && (name_key.starts_with("ppt/slides/")
            || name_key.starts_with("ppt/slidelayouts/")
            || name_key.starts_with("ppt/slidemasters/"))
}

/// `INV-SA-04`: with the project gone, no action names a macro.
pub(crate) fn macro_action_rules(
    draft: &mut Draft<'_>,
    found: &mut Found,
) -> Result<(), WriterError> {
    if !draft.log.vba_dropped {
        return Ok(());
    }
    let names: Vec<String> = draft
        .names
        .iter()
        .filter(|(k, _)| slide_like(k))
        .map(|(_, n)| n.clone())
        .collect();
    for name in names {
        if draft.xml_bytes(&name).is_ok_and(|b| holds_macro_action(&b)) {
            add(found, "INV-SA-04", &name, "an action still names a macro");
        }
    }
    Ok(())
}

impl Draft<'_> {
    /// `INV-SA-04` (how a macro action is spelled is Unverified): take out each link whose
    /// action names a macro, and say how many. A part that will not parse is left.
    fn remove_macro_actions(&mut self) -> Result<(), WriterError> {
        let names: Vec<String> = self
            .names
            .iter()
            .filter(|(k, _)| slide_like(k))
            .map(|(_, n)| n.clone())
            .collect();
        let mut count = 0usize;
        for name in names {
            let Ok(bytes) = self.xml_bytes(&name) else {
                continue;
            };
            if !holds_macro_action(&bytes) || XmlTree::parse(&bytes, &name, &self.xml).is_err() {
                continue;
            }
            let tree = self.tree(&name)?;
            for el in all_elements(tree) {
                let is_macro = tree
                    .plain_attribute(el, "action")
                    .is_some_and(|a| holds_macro_action(a.as_bytes()));
                if is_macro && let Some(parent) = tree.parent_element(el) {
                    tree.remove_child(parent, el);
                    count = count.saturating_add(1);
                }
            }
        }
        if count > 0 {
            let main = self.main.clone();
            self.note(
                "CS-SAV-0014",
                "INV-SA-04",
                &main,
                &format!("{count} actions naming a macro removed"),
            );
        }
        Ok(())
    }

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
        self.remove_macro_actions()?;
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
