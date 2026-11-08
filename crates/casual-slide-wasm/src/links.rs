//! Insert > Link on a shape (`UX-006`): a click hyperlink to a web or mail address,
//! stored in the file as an external relationship and never fetched. The address is
//! user text: only `http:`, `https:` and `mailto:` are taken, without spaces or
//! control characters, and at most [`MAX_LINK_BYTES`] long, so nothing a file or a
//! page could run is ever written as a link.

use std::collections::BTreeMap;

use casual_slide_export::writer::{Op, SavePlan, SlideRef};
use casual_slide_model::shape::{Hyperlink, ObjectId};
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage};
use casual_slide_render::Deck;
use casual_slide_transaction::{Operation, TxnError};

use crate::{BridgeError, Viewer};

/// The longest address taken, in bytes.
pub const MAX_LINK_BYTES: usize = 2048;

/// The prefix of a relationship id this session gives a new link.
const NEW_LINK: &str = "rIdOpLink";

/// Whether `url` may be stored as a link: `http:`, `https:` or `mailto:`, no
/// whitespace or control characters, at most [`MAX_LINK_BYTES`].
pub fn link_allowed(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    !url.is_empty()
        && url.len() <= MAX_LINK_BYTES
        && !url.chars().any(|c| c.is_whitespace() || c.is_control())
        && (lower.starts_with("https://")
            || lower.starts_with("http://")
            || lower.starts_with("mailto:"))
}

impl Viewer {
    /// Give shape `node` of slide `index` a click link to `url`, or take its link away
    /// with `None`: one undo step.
    ///
    /// # Errors
    ///
    /// `CS-WSM-0006` for an address [`link_allowed`] refuses; `CS-TXN-0001` for no such
    /// shape.
    pub fn set_shape_link(
        &mut self,
        index: usize,
        node: u32,
        url: Option<&str>,
    ) -> Result<bool, BridgeError> {
        let part = self.slide_ordinal(index)?;
        let id = ObjectId { part, node };
        let link = match url.map(str::trim) {
            None => None,
            Some(url) if link_allowed(url) => {
                self.minted = self.minted.saturating_add(1);
                let rel = format!("{NEW_LINK}{}", self.minted);
                self.new_links.insert(rel.clone(), url.to_owned());
                Some(Box::new(Hyperlink {
                    rel_id: Some(rel),
                    ..Hyperlink::default()
                }))
            }
            Some(_) => return Err(BridgeError::BadArgument),
        };
        if self
            .deck
            .parts()
            .get(part)
            .and_then(|p| p.node(id))
            .is_none()
        {
            return Err(TxnError::NoSuchObject(id).into());
        }
        self.edit(Operation::SetLink { node: id, link }, false)
    }

    /// The address shape `node` of slide `index` links to: a link added this session
    /// gives its address, one from the file an empty string, and none `None`.
    pub fn shape_link(&self, index: usize, node: u32) -> Option<String> {
        let part = self.slide_ordinal(index).ok()?;
        let link = self
            .deck
            .parts()
            .get(part)?
            .node(ObjectId { part, node })?
            .non_visual
            .as_ref()?
            .hyperlink_click
            .as_ref()?;
        Some(
            link.rel_id
                .as_ref()
                .and_then(|r| self.new_links.get(r))
                .cloned()
                .unwrap_or_default(),
        )
    }

    /// The save pass for links: each link this session added that the model still
    /// holds gets its external relationship, numbered above the slide's highest as
    /// the writer requires (`INV-RI-03`), before the patch writes the `hlinkClick`
    /// naming it. Returns the bytes and, for the patch, each model id's file id.
    pub(crate) fn links_pass(
        &self,
        bytes: Vec<u8>,
    ) -> Result<(Vec<u8>, BTreeMap<String, String>), BridgeError> {
        let mut map = BTreeMap::new();
        if self.new_links.is_empty() {
            return Ok((bytes, map));
        }
        let package = PresentationPackage::open(bytes.clone(), OoxmlLimits::default())
            .map_err(|e| BridgeError::Save(e.code()))?;
        let file = Deck::open(bytes.clone())?;
        let fparts = file.parts();
        let model = self.deck.parts();
        let mut plan = SavePlan::new();
        let mut any = false;
        for (i, &o) in fparts.structure.slides.iter().enumerate() {
            let (Some(entry), Some(m)) = (fparts.structure.entry(o), model.slide(i)) else {
                continue;
            };
            let Some(tree) = m.shape_tree() else {
                continue;
            };
            let mut top = package
                .relationships_of(&entry.part)
                .iter()
                .filter_map(|r| r.id.strip_prefix("rId")?.parse::<u64>().ok())
                .max()
                .unwrap_or(0);
            for node in tree.descendants() {
                let rel = node
                    .non_visual
                    .as_ref()
                    .and_then(|nv| nv.hyperlink_click.as_ref())
                    .and_then(|l| l.rel_id.as_ref());
                let Some((id, url)) = rel.and_then(|r| self.new_links.get_key_value(r)) else {
                    continue;
                };
                top = top.saturating_add(1);
                let file_id = format!("rId{top}");
                map.insert(id.clone(), file_id.clone());
                plan = plan.op(Op::AddLink {
                    slide: SlideRef::Part(entry.part.clone()),
                    id: file_id,
                    target: url.clone(),
                });
                any = true;
            }
        }
        if !any {
            return Ok((bytes, map));
        }
        Ok((Self::run(&plan, bytes)?, map))
    }
}
