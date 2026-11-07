//! What an edited save owes the package beyond the edit (`T1N-05`, `docs/36`): `app.xml`
//! counts and titles (`INV-AP-*`), `core.xml` from the host's inputs, the thumbnail, which
//! is fresh or gone (`INV-TH-*`), and the signature set, which goes whole (`INV-SG-*`).
//!
//! None of it runs when nothing is dirty, so an edit undone returns the input bytes (F1,
//! `INV-SG-06`). A part that will not parse is left alone and named in the report: a save
//! is not the place a retained part's fault becomes a refusal (`docs/29`).

use casual_slide_ooxml::{ElementId, Relationship, XmlTree};

use super::draft::{Draft, sibling_name};
use super::error::WriterError;
use super::names::key;
use super::notes::check_text;
use super::preflight::{Found, add};
use super::view::{AppFacts, CoreFacts, SlideFacts, View, counts_of};

const ORIGIN: &str = "/digital-signature/origin";
const SIGNATURE: &str = "/digital-signature/signature";
const THUMBNAIL: &str = "/metadata/thumbnail";
const CORE: &str = "/metadata/core-properties";
const NEW_THUMBNAIL: &str = "docProps/thumbnail.png";
/// `MS-OI29500` gives this example group; a localized name is not recognised, and takes
/// the `INV-AP-03` path. Which groups PowerPoint writes is Unverified.
const SLIDE_TITLES: &str = "Slide Titles";
/// Unverified: what PowerPoint writes for a slide with no title.
pub(crate) const UNTITLED: &str = "PowerPoint Presentation";

/// What only the host knows, since time and randomness are never sampled here (`docs/36`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HostInputs {
    /// `dcterms:modified`, as `YYYY-MM-DDThh:mm:ssZ` (`INV-AP-08`).
    pub modified: Option<String>,
    /// `cp:lastModifiedBy` (`INV-AP-09`).
    pub user: Option<String>,
    /// `Application` (`INV-AP-05`).
    pub application: Option<String>,
    /// A fresh PNG of the first slide (`INV-TH-03`). None means the thumbnail goes.
    pub thumbnail: Option<Vec<u8>>,
}

/// What the bookkeeping did, for preflight to judge.
#[derive(Debug, Clone, Default)]
pub(crate) struct Bookkeeping {
    /// The host's inputs, when a plan gave them.
    pub host: Option<HostInputs>,
    /// The signed deck's edit was acknowledged (`INV-SG-02`).
    pub acknowledged: bool,
    /// Signatures removed, when the set was dropped.
    pub signatures_dropped: Option<usize>,
    /// The thumbnail written is new.
    pub fresh_thumbnail: bool,
    /// A new-from-template verb: `created` is meant to change (`INV-SA-07`).
    pub new_created: bool,
}

fn has_type(rel: &Relationship, suffix: &str) -> bool {
    rel.rel_type.ends_with(suffix)
}

fn is_w3cdtf_seconds(value: &str) -> bool {
    let shape = "dddd-dd-ddTdd:dd:ddZ";
    value.len() == shape.len()
        && value.bytes().zip(shape.bytes()).all(|(v, s)| match s {
            b'd' => v.is_ascii_digit(),
            other => v == other,
        })
}

fn child_named(tree: &XmlTree, parent: ElementId, local: &str) -> Option<ElementId> {
    tree.child_elements(parent)
        .into_iter()
        .find(|&el| tree.local_name(el) == local)
}

impl Draft<'_> {
    /// All of it, after the sweep. A plan that left nothing dirty stays that way.
    pub(crate) fn bookkeep(
        &mut self,
        host: Option<&HostInputs>,
        acknowledged: bool,
        new_created: bool,
    ) -> Result<(), WriterError> {
        if !self.is_dirty()? {
            return Ok(());
        }
        self.log.books.host = host.cloned();
        self.log.books.acknowledged = acknowledged;
        self.log.books.new_created = new_created;
        self.drop_signatures(acknowledged)?;
        self.settle_thumbnail(host.and_then(|h| h.thumbnail.as_deref()))?;
        self.sweep()?;
        self.settle_app(host)?;
        if let Some(host) = host {
            self.settle_core(host)?;
        }
        self.note_retained_macro_parts()
    }

    /// `INV-SA-06` (Inferred): ActiveX and custom UI in a macro-free output stay, reported.
    fn note_retained_macro_parts(&mut self) -> Result<(), WriterError> {
        let Some(extension) = self.log.requested_extension.clone() else {
            return Ok(());
        };
        if extension.ends_with('m') {
            return Ok(());
        }
        let names: Vec<String> = self
            .names
            .values()
            .filter(|n| key(n).contains("activex/") || key(n).starts_with("customui/"))
            .cloned()
            .collect();
        for name in names {
            self.note(
                "CS-SAV-0024",
                "INV-SA-06",
                &name,
                "ActiveX or custom UI retained in a macro-free file, never run",
            );
        }
        Ok(())
    }

    /// `INV-SG-03` to `INV-SG-05` (Inferred): the origin relationship goes, and the sweep
    /// takes everything only it reached. The VBA project's own signature is reached through
    /// the project, and stays with it (`INV-SG-08`).
    fn drop_signatures(&mut self, acknowledged: bool) -> Result<(), WriterError> {
        let origins: Vec<Relationship> = self
            .relationships("")?
            .into_iter()
            .filter(|r| has_type(r, ORIGIN))
            .collect();
        let Some(first) = origins.first() else {
            return Ok(());
        };
        let origin = self.resolve("", first);
        let part = origin.clone().unwrap_or_else(|| first.target.clone());
        // INV-SG-02 (the acknowledgement Inferred): the host owns how it asks.
        if !acknowledged {
            return Err(WriterError::SignatureNeedsAcknowledgement { part });
        }
        let mut count = 0usize;
        if let Some(origin) = &origin {
            count = self
                .relationships(origin)?
                .iter()
                .filter(|r| has_type(r, SIGNATURE))
                .count();
        }
        for rel in &origins {
            self.remove_relationship("", &rel.id)?;
        }
        self.log.books.signatures_dropped = Some(count);
        let line = self.has_signature_line();
        let detail = if line {
            format!("{count} signatures removed; a signature line picture remains on a slide")
        } else {
            format!("{count} signatures removed")
        };
        self.note("CS-SAV-0001", "INV-SG-04", &part, &detail);
        Ok(())
    }

    /// A signature line picture may still look signed. Only VML can hold one.
    fn has_signature_line(&mut self) -> bool {
        let vml: Vec<String> = self
            .names
            .values()
            .filter(|n| key(n).contains("vmldrawing"))
            .cloned()
            .collect();
        vml.iter().any(|name| {
            self.xml_bytes(name).is_ok_and(|b| {
                b.to_ascii_lowercase()
                    .windows(13)
                    .any(|w| w == b"signatureline")
            })
        })
    }

    /// `INV-TH-02`, `INV-TH-03` (Inferred): fresh or gone. A deck with none gains none.
    fn settle_thumbnail(&mut self, fresh: Option<&[u8]>) -> Result<(), WriterError> {
        let rels: Vec<Relationship> = self
            .relationships("")?
            .into_iter()
            .filter(|r| has_type(r, THUMBNAIL))
            .collect();
        let Some((first, others)) = rels.split_first() else {
            return Ok(());
        };
        for rel in others {
            self.remove_relationship("", &rel.id)?;
        }
        let Some(bytes) = fresh else {
            let part = self
                .resolve("", first)
                .unwrap_or_else(|| first.target.clone());
            self.remove_relationship("", &first.id)?;
            self.note(
                "CS-SAV-0004",
                "INV-TH-02",
                &part,
                "a stale thumbnail was removed, with no renderer to replace it",
            );
            return Ok(());
        };
        let current = self.resolve("", first);
        let same = current
            .as_deref()
            .is_some_and(|c| key(c) == key(NEW_THUMBNAIL));
        if same {
            self.set_part_bytes(NEW_THUMBNAIL, bytes.to_vec());
        } else {
            let name = if self.has_part(NEW_THUMBNAIL) {
                self.families.mint("docProps", "thumbnail", "png")?
            } else {
                NEW_THUMBNAIL.to_owned()
            };
            self.add_part(&name, Some("image/png"), bytes.to_vec())?;
            self.retarget_relationship("", &first.id, &name)?;
        }
        self.log.books.fresh_thumbnail = true;
        self.note(
            "CS-SAV-0005",
            "INV-TH-03",
            NEW_THUMBNAIL,
            "the thumbnail was regenerated",
        );
        Ok(())
    }

    /// `INV-AP-01` to `-06`: `app.xml` follows what this save changed. A count that arrived
    /// stale, with nothing here changing it, stays and is reported (F2).
    fn settle_app(&mut self, host: Option<&HostInputs>) -> Result<(), WriterError> {
        let (before, after) = self.build_views()?;
        let Some(now) = after.app.clone() else {
            return Ok(());
        };
        let drop_dig_sig = self.log.books.signatures_dropped.is_some() && now.dig_sig;
        if drop_dig_sig {
            let tree = self.tree(&now.part)?;
            let root = tree.root();
            if let Some(el) = child_named(tree, root, "DigSig") {
                tree.remove_child(root, el);
            }
        }
        // A `Deck` patch answers for the slide list it wrote and for an `app.xml` it wrote.
        // What moved the slides after it, an operation or a slide patch, this follows.
        let stated = self.log.model_patched.contains(&key(&now.part));
        let was = before.app.clone().unwrap_or_default();
        let part = now.part.clone();
        let base = self.log.deck_slides.as_ref().unwrap_or(&before.slides);
        let (old, new) = (counts_of(base), after.counts());
        let wanted = [
            (
                "Slides",
                old.0 != new.0 || (!stated && now.slides != was.slides),
                new.0,
            ),
            (
                "HiddenSlides",
                old.1 != new.1 || (!stated && now.hidden_slides != was.hidden_slides),
                new.1,
            ),
            (
                "Notes",
                old.2 != new.2 || (!stated && now.notes != was.notes),
                new.2,
            ),
        ];
        let application = host.and_then(|h| h.application.as_deref());
        if let Some(name) = application {
            check_text(name, &part)?;
        }
        let titles = title_plan(&before, base, &after, &now, &was);
        let tree = self.tree(&part)?;
        let root = tree.root();
        for (local, changed, value) in wanted {
            if changed && let Some(el) = child_named(tree, root, local) {
                tree.set_text(el, &value.to_string());
            }
        }
        if let Some(name) = application
            && let Some(el) = child_named(tree, root, "Application")
        {
            tree.set_text(el, name);
        }
        match titles {
            TitlePlan::Leave => {}
            TitlePlan::Remove => {
                for local in ["HeadingPairs", "TitlesOfParts"] {
                    if let Some(el) = child_named(tree, root, local) {
                        tree.remove_child(root, el);
                    }
                }
                self.note(
                    "CS-SAV-0011",
                    "INV-AP-03",
                    &part,
                    "HeadingPairs and TitlesOfParts held groups not recognised, and were removed",
                );
            }
            TitlePlan::Write { start, old, titles } => write_titles(tree, start, old, &titles),
        }
        Ok(())
    }

    /// A `Deck` patch states `app.xml` and the slide list itself, so a count it leaves out of
    /// step is reported rather than refused, as `INV-ID-02` stands aside for it.
    pub(crate) fn model_owns_app(&self, part: &str) -> bool {
        self.log.model_edited_main || self.log.model_patched.contains(&key(part))
    }

    /// `INV-AP-08`, `INV-AP-09`: `core.xml` from what the host supplied. Absent stays absent.
    fn settle_core(&mut self, host: &HostInputs) -> Result<(), WriterError> {
        let found = self
            .relationships("")?
            .into_iter()
            .find(|r| has_type(r, CORE))
            .and_then(|r| self.resolve("", &r));
        let Some(part) = found else {
            return Ok(());
        };
        if let Some(when) = &host.modified
            && !is_w3cdtf_seconds(when)
        {
            return Err(WriterError::BadOperation {
                what: "a modified time not in YYYY-MM-DDThh:mm:ssZ form",
                subject: part,
            });
        }
        if let Some(user) = &host.user {
            check_text(user, &part)?;
        }
        let new_created = self.log.books.new_created;
        let tree = self.tree(&part)?;
        let root = tree.root();
        if let Some(when) = &host.modified {
            if let Some(el) = child_named(tree, root, "modified") {
                tree.set_text(el, when);
                if tree
                    .prefix_for_namespace(el, "http://www.w3.org/2001/XMLSchema-instance")
                    .is_some_and(|p| p == "xsi")
                {
                    let _ = tree.set_attribute(el, "xsi:type", "dcterms:W3CDTF");
                }
            }
            if new_created && let Some(el) = child_named(tree, root, "created") {
                tree.set_text(el, when);
            }
        }
        if let Some(user) = &host.user
            && let Some(el) = child_named(tree, root, "lastModifiedBy")
        {
            tree.set_text(el, user);
        }
        if let Some(el) = child_named(tree, root, "revision")
            && let Some(next) = tree
                .text(el)
                .and_then(|t| t.trim().parse::<u64>().ok())
                .and_then(|n| n.checked_add(1))
        {
            tree.set_text(el, &next.to_string());
        }
        Ok(())
    }
}

/// What to do with the title lists.
enum TitlePlan {
    Leave,
    Remove,
    Write {
        /// Where the slide group's titles start in `TitlesOfParts`.
        start: usize,
        /// How many there were.
        old: usize,
        titles: Vec<String>,
    },
}

/// `INV-AP-03`: nothing unless the slides or a title changed since `base`, what a `Deck`
/// patch left or else the input. Then the slide group is brought up to date in place, or
/// both lists go, never one, and never stale.
fn title_plan(
    before: &View,
    base: &[SlideFacts],
    after: &View,
    now: &AppFacts,
    was: &AppFacts,
) -> TitlePlan {
    let keys = |slides: &[SlideFacts]| slides.iter().map(|s| s.key.clone()).collect::<Vec<_>>();
    let changed = keys(base) != keys(&after.slides)
        || after
            .slides
            .iter()
            .any(|s| base.iter().any(|b| b.key == s.key && b.title != s.title));
    let (Some(groups), Some(stored)) = (&now.groups, &now.titles) else {
        return TitlePlan::Leave;
    };
    if !changed || was.groups.is_none() {
        return TitlePlan::Leave;
    }
    let counts: Option<Vec<usize>> = groups.iter().map(|(_, n)| *n).collect();
    let at = groups.iter().position(|(name, _)| name == SLIDE_TITLES);
    let (Some(counts), Some(at)) = (counts, at) else {
        return TitlePlan::Remove;
    };
    let start: usize = counts.iter().take(at).sum();
    let old = counts.get(at).copied().unwrap_or(0);
    if counts.iter().sum::<usize>() != stored.len() || old != before.slides.len() {
        return TitlePlan::Remove;
    }
    let titles = after
        .slides
        .iter()
        .map(|slide| {
            let earlier = before.slides.iter().position(|b| b.key == slide.key);
            let kept = earlier.and_then(|i| {
                let same = before.slides.get(i)?.title == slide.title;
                let held = stored.get(start.checked_add(i)?)?;
                same.then(|| held.clone())
            });
            kept.unwrap_or_else(|| slide.title.clone())
        })
        .collect();
    TitlePlan::Write { start, old, titles }
}

fn write_titles(tree: &mut XmlTree, start: usize, old: usize, titles: &[String]) {
    let root = tree.root();
    let Some(heading) =
        child_named(tree, root, "HeadingPairs").and_then(|h| child_named(tree, h, "vector"))
    else {
        return;
    };
    let Some(list) =
        child_named(tree, root, "TitlesOfParts").and_then(|t| child_named(tree, t, "vector"))
    else {
        return;
    };
    let items = tree.child_elements(list);
    let name = items.first().map_or_else(
        || "vt:lpstr".to_owned(),
        |&el| tree.qualified_name(el).to_owned(),
    );
    let name = if name.is_empty() {
        sibling_name(tree, list, "lpstr")
    } else {
        name
    };
    let end = start.saturating_add(old);
    for &el in items.iter().skip(start).take(old) {
        tree.remove_child(list, el);
    }
    let before = items.get(end).copied();
    for title in titles {
        let el = tree.new_element(&name);
        tree.set_text(el, title);
        tree.insert_child(list, before, el);
    }
    let size = items.len().saturating_sub(old).saturating_add(titles.len());
    let _ = tree.set_attribute(list, "size", &size.to_string());
    // The count sits in the variant after the group's name.
    let variants = tree.child_elements(heading);
    for pair in variants.chunks(2) {
        let [label, count] = pair else { continue };
        let is_slides = tree
            .child_elements(*label)
            .first()
            .and_then(|&el| tree.text(el))
            .is_some_and(|t| t == SLIDE_TITLES);
        if is_slides && let Some(&value) = tree.child_elements(*count).first() {
            tree.set_text(value, &titles.len().to_string());
        }
    }
}

/// The title a slide's `app.xml` entry holds: its title placeholder's text. Unverified
/// against what PowerPoint writes, and on `PV-01`'s list.
pub(crate) fn slide_title(tree: &XmlTree) -> String {
    let mut stack = vec![tree.root()];
    let mut budget = 100_000usize;
    while let Some(el) = stack.pop() {
        budget = budget.saturating_sub(1);
        if budget == 0 {
            break;
        }
        if tree.local_name(el) == "sp" && is_title(tree, el) {
            let mut text = String::new();
            let mut inner = vec![el];
            while let Some(here) = inner.pop() {
                budget = budget.saturating_sub(1);
                if budget == 0 {
                    break;
                }
                if tree.local_name(here) == "t" {
                    text.push_str(&tree.text(here).unwrap_or_default());
                }
                let mut children = tree.child_elements(here);
                children.reverse();
                inner.extend(children);
            }
            if !text.trim().is_empty() {
                return text;
            }
        }
        let mut children = tree.child_elements(el);
        children.reverse();
        stack.extend(children);
    }
    UNTITLED.to_owned()
}

fn is_title(tree: &XmlTree, sp: ElementId) -> bool {
    child_named(tree, sp, "nvSpPr")
        .and_then(|nv| child_named(tree, nv, "nvPr"))
        .and_then(|nv| child_named(tree, nv, "ph"))
        .and_then(|ph| tree.plain_attribute(ph, "type"))
        .is_some_and(|t| t == "title" || t == "ctrTitle")
}

fn count_is(field: &Option<String>, truth: usize) -> bool {
    field
        .as_deref()
        .is_none_or(|text| text.trim().parse::<usize>().ok() == Some(truth))
}

fn is_app_version(value: &str) -> bool {
    let Some((major, minor)) = value.split_once('.') else {
        return false;
    };
    let digits = |s: &str, max: usize| {
        !s.is_empty() && s.len() <= max && s.bytes().all(|b| b.is_ascii_digit())
    };
    digits(major, 2) && digits(minor, 4)
}

fn app_state(view: &View, app: &AppFacts, found: &mut Found) {
    let part = app.part.as_str();
    let (slides, hidden, notes) = view.counts();
    // INV-AP-01 (Documented meanings).
    if !count_is(&app.slides, slides) {
        add(found, "INV-AP-01", part, "Slides");
    }
    if !count_is(&app.hidden_slides, hidden) {
        add(found, "INV-AP-01", part, "HiddenSlides");
    }
    if !count_is(&app.notes, notes) {
        add(found, "INV-AP-01", part, "Notes");
    }
    // INV-AP-02 (Documented).
    if let Some(groups) = &app.groups {
        let sum: usize = groups.iter().filter_map(|(_, n)| *n).sum();
        let titles = app.titles.as_ref().map_or(0, Vec::len);
        if sum < titles {
            add(
                found,
                "INV-AP-02",
                part,
                "more titles than the groups count",
            );
        }
        if app.heading_size.as_deref() != Some(app.heading_items.to_string().as_str()) {
            add(found, "INV-AP-02", part, "HeadingPairs size");
        }
        // INV-AP-03 (Inferred): the slide group, when it can be read, is not stale.
        if let Some((_, Some(n))) = groups.iter().find(|(name, _)| name == SLIDE_TITLES)
            && *n != slides
        {
            add(found, "INV-AP-03", part, "slide titles out of step");
        }
    }
    if let Some(titles) = &app.titles
        && app.titles_size.as_deref() != Some(titles.len().to_string().as_str())
    {
        add(found, "INV-AP-02", part, "TitlesOfParts size");
    }
    // INV-AP-03: both, or neither.
    if app.groups.is_some() != app.titles.is_some() {
        add(found, "INV-AP-03", part, "one of the two lists alone");
    }
    // INV-AP-04 (Documented).
    if app
        .app_version
        .as_deref()
        .is_some_and(|v| !is_app_version(v))
    {
        add(found, "INV-AP-04", part, "AppVersion out of form");
    }
}

/// Rules one package can show.
pub(crate) fn state_rules(view: &View, found: &mut Found) {
    if let Some(app) = &view.app {
        app_state(view, app, found);
    }
    let root: &[Relationship] = view.rels.get("").map_or(&[], |r| r.items.as_slice());
    // INV-AP-07 (Documented).
    let cores: Vec<&Relationship> = root.iter().filter(|r| has_type(r, CORE)).collect();
    if cores.len() > 1 {
        add(found, "INV-AP-07", "_rels/.rels", "two core properties");
    }
    // INV-TH-01 (Documented).
    let thumbs: Vec<&Relationship> = root.iter().filter(|r| has_type(r, THUMBNAIL)).collect();
    if thumbs.len() > 1 {
        add(found, "INV-TH-01", "_rels/.rels", "two thumbnails");
    }
    for rel in thumbs {
        if let Some(part) = view.resolved("", rel)
            && view.rels.get(&part).is_some_and(|r| !r.items.is_empty())
        {
            add(found, "INV-TH-01", &part, "a thumbnail with relationships");
        }
    }
    // INV-SG-01 (Documented): a signature part, and nothing to say the package is signed.
    let has_origin = root.iter().any(|r| has_type(r, ORIGIN));
    if !has_origin
        && let Some(part) = view
            .parts_of_type("digital-signature-xmlsignature+xml")
            .first()
    {
        add(found, "INV-SG-01", part, "a signature with no origin");
    }
    // INV-SA-03 (Documented): VBA is never in a macro-free output.
    let macro_free = view
        .content_type(&view.main)
        .is_some_and(|ct| !ct.contains("macroEnabled"));
    let vba = view
        .rels
        .get(&view.main)
        .is_some_and(|r| r.items.iter().any(|i| has_type(i, "/vbaProject")));
    if macro_free && vba {
        add(found, "INV-SA-03", &view.main, "VBA in a macro-free type");
    }
}

fn core_rules(draft: &Draft<'_>, before: &CoreFacts, after: &CoreFacts, found: &mut Found) {
    let part = after.part.as_str();
    let core_patched = draft.log.model_patched.contains(&key(part));
    if core_patched {
        return;
    }
    // INV-AP-10 (Documented).
    if before.keywords != after.keywords {
        add(found, "INV-AP-10", part, "keywords markup changed");
    }
    // INV-AP-08: `created` never changes.
    if before.created != after.created && !draft.log.books.new_created {
        add(found, "INV-AP-08", part, "created changed");
    }
    let Some(host) = &draft.log.books.host else {
        return;
    };
    // INV-AP-08 (Documented for form), with a time supplied.
    if let Some(when) = &host.modified
        && before.modified.is_some()
        && (after.modified.as_ref() != Some(when)
            || after.modified_type.as_deref() != Some("dcterms:W3CDTF"))
    {
        add(found, "INV-AP-08", part, "modified not set in W3CDTF form");
    }
    if host.modified.is_none() && before.modified != after.modified {
        add(found, "INV-AP-08", part, "modified changed with no time");
    }
    // INV-AP-09 (Inferred): one up if an integer; text left alone.
    if let Some(user) = &host.user
        && before.last_modified_by.is_some()
        && after.last_modified_by.as_ref() != Some(user)
    {
        add(found, "INV-AP-09", part, "lastModifiedBy not set");
    }
    let number = before
        .revision
        .as_deref()
        .and_then(|t| t.trim().parse::<u64>().ok());
    let wanted = match number {
        Some(n) => n.checked_add(1).map(|n| n.to_string()),
        None => before.revision.clone(),
    };
    if after.revision != wanted {
        add(found, "INV-AP-09", part, "revision wrong");
    }
}

fn signature_rules(draft: &Draft<'_>, before: &View, after: &View, found: &mut Found) {
    let root = |v: &View| -> Vec<Relationship> {
        v.rels.get("").map(|r| r.items.clone()).unwrap_or_default()
    };
    let origins: Vec<Relationship> = root(before)
        .into_iter()
        .filter(|r| has_type(r, ORIGIN))
        .collect();
    if origins.is_empty() {
        return;
    }
    // INV-SG-02.
    if !draft.log.books.acknowledged {
        add(found, "INV-SG-02", "_rels/.rels", "edited unacknowledged");
    }
    // INV-SG-05 (Inferred): the whole set goes.
    if root(after).iter().any(|r| has_type(r, ORIGIN)) {
        add(
            found,
            "INV-SG-05",
            "_rels/.rels",
            "the origin relationship stayed",
        );
    }
    // INV-SG-03 (Inferred): everything only the origin reached goes.
    let from: Vec<String> = origins
        .iter()
        .filter_map(|r| before.resolved("", r))
        .collect();
    for member in before.reach_of(&from) {
        if let Some(name) = after.names.get(&member) {
            add(found, "INV-SG-03", name, "a signature part stayed");
        }
    }
    // INV-SG-04 (Inferred): one entry.
    if !draft.report.iter().any(|e| e.code == "CS-SAV-0001") {
        add(found, "INV-SG-04", "_rels/.rels", "no report entry");
    }
    // INV-AP-06 (Inferred).
    if after.app.as_ref().is_some_and(|a| a.dig_sig) {
        add(found, "INV-AP-06", "docProps/app.xml", "DigSig stayed");
    }
}

/// Rules that compare the draft with the input.
pub(crate) fn change_rules(draft: &Draft<'_>, before: &View, after: &View, found: &mut Found) {
    if let (Some(was), Some(now)) = (&before.app, &after.app) {
        // INV-AP-03 (Inferred): never one of the two removed.
        if was.groups.is_some()
            && was.titles.is_some()
            && now.groups.is_some() != now.titles.is_some()
        {
            add(found, "INV-AP-03", &now.part, "one of the two removed");
        }
        // INV-AP-05 (Inferred).
        if let Some(name) = draft
            .log
            .books
            .host
            .as_ref()
            .and_then(|h| h.application.as_ref())
            && was.application.is_some()
            && now.application.as_ref() != Some(name)
        {
            add(found, "INV-AP-05", &now.part, "Application retained");
        }
    }
    if let (Some(was), Some(now)) = (&before.core, &after.core) {
        core_rules(draft, was, now, found);
    }
    signature_rules(draft, before, after, found);
    // INV-TH-02 (Inferred): fresh or gone, never stale.
    let thumbs = |v: &View| -> Vec<String> {
        v.rels
            .get("")
            .map(|r| {
                r.items
                    .iter()
                    .filter(|i| has_type(i, THUMBNAIL))
                    .filter_map(|i| v.resolved("", i))
                    .collect()
            })
            .unwrap_or_default()
    };
    let (old, new) = (thumbs(before), thumbs(after));
    if !draft.log.books.fresh_thumbnail {
        for part in new.iter().filter(|p| old.contains(p)) {
            add(found, "INV-TH-02", part, "a stale thumbnail stayed");
        }
    }
    // INV-TH-03: an old part is removed rather than overwritten under another name.
    if draft.log.books.fresh_thumbnail {
        for part in old.iter().filter(|p| !new.contains(p)) {
            if after.names.contains_key(&key(part)) {
                add(found, "INV-TH-03", part, "the old thumbnail stayed");
            }
        }
    }
    // INV-SG-08 (Unverified): the VBA project's signature stays with the project.
    let vba_signature = |v: &View| -> Vec<String> {
        v.rels
            .iter()
            .flat_map(|(source, r)| {
                r.items
                    .iter()
                    .filter(|i| has_type(i, "/vbaProjectSignature"))
                    .filter_map(move |i| v.resolved(source, i))
            })
            .collect()
    };
    if !draft.log.vba_dropped {
        for part in vba_signature(before) {
            if !after.names.contains_key(&key(&part)) {
                add(found, "INV-SG-08", &part, "the VBA signature went");
            }
        }
    }
}

/// `INV-SA-01`, `-02`, `-05`, `-06`: what a plan that only changes the format may touch.
pub(crate) fn retype_rules(draft: &Draft<'_>, before: &View, after: &View, found: &mut Found) {
    if draft.log.requested_extension.is_some()
        && !after
            .content_type(&after.main)
            .is_some_and(super::format::is_main_type)
    {
        add(
            found,
            "INV-SA-01",
            &after.main,
            "not one of the seven types",
        );
    }
    if !draft.log.retype_only {
        return;
    }
    let books = &draft.log.books;
    // Bookkeeping's parts, and with VBA dropped the parts macro actions sat in.
    let allowed = |name: &str| {
        let k = key(name);
        k == key(super::draft::CONTENT_TYPES_PART)
            || k.starts_with("docprops/")
            || k == "_rels/.rels"
            || (draft.log.vba_dropped
                && (k.starts_with("ppt/slides/")
                    || k.starts_with("ppt/slidelayouts/")
                    || k.starts_with("ppt/slidemasters/")
                    || k.ends_with("_rels/presentation.xml.rels")))
    };
    let added =
        |name: &str| after.names.contains_key(&key(name)) && !before.names.contains_key(&key(name));
    for name in draft.overlay_names() {
        if allowed(name) {
            continue;
        }
        if added(name) {
            add(found, "INV-SA-05", name, "a retype added a part");
        } else {
            add(found, "INV-SA-02", name, "a retype changed a part");
        }
    }
    if !draft.log.vba_dropped && books.signatures_dropped.is_none() {
        for name in &draft.log.removed {
            if !key(name).starts_with("docprops/") {
                add(found, "INV-SA-06", name, "a retype removed a part");
            }
        }
    }
}
