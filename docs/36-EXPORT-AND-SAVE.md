# 36 — Export and save (Tier 1N)

The design for the writer: tracker rows `WR-02`, `WR-03`, `WR-04` and `WR-05`. Owning crate: `casual-slide-export`, with
render-on-save hosted in `casual-slide-sdk`. Status: the `PN`, `RI`, `CT`, `SP`, `ID`, `SL`, `NM`, `AP`, `TH`, `SG` and `SA` families are built, in `crates/casual-slide-export/src/writer/` (`T1N-01` to `T1N-09`; Tier 1N closed 2026-11-08), see [Implementation status](#implementation-status); `PE` and `FB` are not. The fidelity contract it
delivers is in [22](22-NORMALIZED-SCHEMA.md); the edit mechanism is ADR-013 and `ARC-01` in
[28](28-PACKAGE-AND-PART-GRAPH.md); the ladder and `PV-01` are in [51](51-SPEC-CHECKLIST-AND-FIDELITY.md). The shape follows
OpenCalc's `36-EXPORT-AND-ROUNDTRIP-DESIGN.md`.

## How to read the evidence marks

Every rule carries one mark. **PowerPoint was not available when this was written. Nothing here was observed in PowerPoint,
and nothing here may be cited as matching it.** `PV-01` confirms or overturns the marks later.

| Mark | Meaning |
| --- | --- |
| **Documented** | A published specification or Microsoft source says it, linked. For the Open XML SDK, what is documented is the SDK's rule, not PowerPoint's behaviour. |
| **Inferred** | Reasoned from documented facts, a third-party report, or our own policy. The basis is stated. A third-party report of a repair prompt is Inferred, because we did not reproduce it. |
| **Unverified** | No source was found. The rule is our best choice, and `PV-01` must test it. |

**Microsoft publishes no list of what triggers a repair prompt.** A spec violation is Documented as a violation; that it
prompts is at best Inferred. See [Repair triggers](#repair-triggers).

## The save pipeline

Two writers, chosen by whether anything is dirty, as in OpenCalc.

1. **Nothing dirty:** return the input bytes (F1). This includes a session whose edits were all undone.
2. **Something dirty:** the steps below, in `-export`, and a pure function of the session's state and the supplied inputs.

| Step | Does | Reads |
| --- | --- | --- |
| Patch | For each dirty object, write its values to the `XmlTree` nodes they came from (`ARC-01`) | Model, tree |
| Structure | Turn added and removed slides, layouts, masters, notes and media into a part plan: names, relationships, content types, ids | Part graph |
| Fallbacks | List each stale fallback with its [policy](#stale-fallbacks); the SDK supplies images | Dirty set |
| Bookkeeping | `presentation.xml` lists, section list, `app.xml`, `core.xml`, thumbnail, signature parts | Host inputs |
| Preflight | Run every `INV-*` below on the plan, against the input. **A violation this save introduced refuses the save** with a typed error, leaving the session intact. A violation that arrived in a retained part is reported and left alone | Plan, input |
| Emit | `Package::rewrite`: untouched parts copied raw, so F2 holds | Plan |

Priority 1 in [AGENTS.md](../AGENTS.md) is never to corrupt a deck, so a refused save is the correct outcome of a writer bug.
**A third-party deck that already breaks a rule must still open and save**, or the user is trapped; silently correcting it would break F2.
The forced-rewrite mode of `WR-01` runs this same pipeline.

**Inputs the host supplies**, because time and randomness are never sampled: the timestamp, the user's display name, GUIDs for new
sections, the renderer output, and the application name. With the same inputs and version, the bytes are the same.

**Every save returns dispositions**, per part or construct, in OpenCalc's vocabulary: `unchanged-copy`, `regenerate`, `merge`,
`omit`, `block`. This is what the part-hash allow-list in [22](22-NORMALIZED-SCHEMA.md) checks against, and a silent drop
fails CI.

## Part naming

| ID | Rule | Evidence |
| --- | --- | --- |
| `INV-PN-01` | A part name is `/`-separated non-empty segments. No segment ends in `.`, contains `%2F` or `%5C`, or percent-encodes an unreserved character | Documented: [OPC 6.2.2.2][opc] |
| `INV-PN-02` | **A name we mint is ASCII, and only `[A-Za-z0-9_.-]` in each segment.** A non-ASCII name that arrived is kept, named in the report, and never copied into a new name | Documented for the reason: [Office will not load a package with non-ASCII part names][oi-naming], though OPC [only recommends support][opc]. The narrower set is Inferred, our policy |
| `INV-PN-03` | No two names are equal under ASCII case-insensitive matching | Documented: [OPC 6.2.2.3][opc]. Reported as corrupting: [python-pptx #67][py67] |
| `INV-PN-04` | No name is derivable from another: `/a/b` may not coexist with `/a` | Documented: [OPC 6.2.2.3][opc] |
| `INV-PN-05` | A new name takes the family's next free number: one above the highest in that family, compared case-insensitively. Numbers in the names of removed parts are not reused in that save | Inferred: our policy, to avoid the collision reported in [python-pptx #67][py67]. OPC does not require numbering |
| `INV-PN-06` | **Existing parts are never renamed,** and gaps left by removal stay. Renaming would break F2 and every relationship to the part | Inferred from the F2 contract. That PowerPoint accepts gaps is Unverified |
| `INV-PN-07` | Removing a part also removes its `_rels/<name>.rels`, its `Override`, and every relationship to it | Documented for the name mapping: [OPC 6.5.2.3][opc]. The cascade is Inferred |
| `INV-PN-08` | A ZIP item name is the part name without the leading `/`, with non-ASCII percent-encoded. Item names are ASCII and unique | Documented: [OPC 7.3.3 to 7.3.5][opc] |
| `INV-PN-09` | New entries use DEFLATE, with no ZIP encryption. A retained `Store` entry is copied as it is | Documented: [OPC 7.3.6][opc]; [Office also writes Store][oi-store] (Microsoft files this under "Part 2 Section 8"; the content is about 7.3.6) |
| `INV-PN-10` | Local and central ZIP headers agree | Documented: [OPC Annex B.2][opc] |
| `INV-PN-11` | `[Content_Types].xml` keeps its position, first in anything we create | Inferred: [OPC 7.3.7][opc] sets only the name. OpenCalc's detector reads the first local header, so others may too |

Family directories, as a convention rather than a requirement. Inferred for slides, masters, layouts, notes and themes: the [SDK part data][sdk-slide]
and the [ECMA examples][p1] use them. Unverified for the rest, where readers should follow relationships and not names: `ppt/slides/slideN.xml`, `ppt/slideLayouts/slideLayoutN.xml`,
`ppt/slideMasters/slideMasterN.xml`, `ppt/notesSlides/notesSlideN.xml`, `ppt/notesMasters/notesMasterN.xml`,
`ppt/handoutMasters/handoutMasterN.xml`, `ppt/theme/themeN.xml`, `ppt/media/imageN.ext` and `mediaN.ext`, `ppt/charts/chartN.xml`,
`ppt/embeddings/`, `ppt/diagrams/dataN.xml` with `layoutN`, `quickStyleN`, `colorsN`, `drawingN`, `ppt/ink/inkN.xml`,
`ppt/tags/tagN.xml`. **A media part name never includes the user's filename,** which would leak it and would bring percent
encoding into play.

## Relationship ids

| ID | Rule | Evidence |
| --- | --- | --- |
| `INV-RI-01` | An `Id` is unique within its relationships part and valid as `xsd:ID`, so it cannot start with a digit | Documented: [OPC 6.5.3.4][opc] |
| `INV-RI-02` | **Existing ids are never renumbered.** References live in part bodies and, for SmartArt, in a *different* part: `dgm:dataModelExt/@relId` sits in the data part and names a relationship used to find the drawing | Inferred: [relId][dme-block] names the drawing and [the drawing is the target of a relationship from the slide][diag-part]. That the id resolves against the slide's rels is our reading |
| `INV-RI-03` | A new id is `rId` plus one above the highest numeric `rIdN` in that part, then checked against every other id there. Allocated in document order at save, so it is deterministic | Inferred: our policy; only uniqueness is required |
| `INV-RI-04` | Every `r:*` attribute in a part we wrote resolves in that part's rels | Documented as an SDK rule: [`RelationshipExistConstraint`][sdk-pml] on `sldId`, `sldMasterId`, `sldLayoutId`. General form Inferred |
| `INV-RI-05` | A relationship to an unknown type is retained, is counted as a reference, and is never followed if external | Inferred: our contract in [parts.md](spec/parts.md). [OPC 6.5.3.4][opc] does not restrict types |
| `INV-RI-06` | Relationships parts are not the source or target of a relationship, and hold no `xml:base` | Documented: [OPC 6.5.2.1, 6.5.3.1][opc] |
| `INV-RI-07` | An internal target is a relative reference, written in the same form as its siblings. A retained target is never rewritten | Documented for relative: [OPC 6.5.3.4][opc]. The rest is Inferred from the F2 contract |
| `INV-RI-08` | Explicit and implicit follow Office, not the standard, where they differ: master to layout is explicit; presentation to slide layout does not exist; VML is implicit; no slide, layout, master or notes part has a thumbnail relationship | Documented: [master][oi-master], [presentation][oi-pres], [slide][oi-slide], [layout][oi-layout], [notes master][oi-nm], [notes slide][oi-ns] |
| `INV-RI-09` | Relationship type URIs are compared as exact strings and written in the form the file arrived in (Strict or Transitional). Core properties use the package namespace | Documented for comparison and the core type: [OPC 6.5.3.4][opc], [Office's core relationship][oi-core]. Written form is Inferred: [parts.md](spec/parts.md) |
| `INV-RI-10` | Adding a reference to a target that already has a relationship of that type from this source reuses it | Inferred: our policy, keeps bookkeeping small |

## Content types

| ID | Rule | Evidence |
| --- | --- | --- |
| `INV-CT-01` | Every part other than a relationships part matches a `Default` or an `Override` | Documented: [OPC 7.2.3.2.1][opc] |
| `INV-CT-02` | At most one `Default` per extension and one `Override` per part name, under ASCII case-insensitive comparison | Documented: [OPC 7.2.3.2.1, 7.2.3.5][opc] |
| `INV-CT-03` | Adding follows the standard's steps: no extension means `Override`; a matching `Default` with the same type (case-insensitive) means no change; with a different type means `Override`; no `Default` means add one | Documented: [OPC 7.2.3.4][opc] |
| `INV-CT-04` | Where the standard allows either: XML parts get `Override`, binary media gets `Default`. An image with an unusual extension keeps its name and gets an `Override` | Inferred: our policy. A main part must be distinguishable from other `.xml` parts, so it cannot rest on a `Default` |
| `INV-CT-05` | **Removing a part removes its `Override`.** A phantom `Override` is never written | Inferred: [PptxGenJS #1449][pgx] links phantom `slideMaster` overrides to the prompt, though a later commenter removed them and still saw it |
| `INV-CT-06` | An unused `Default` stays, keeping the stream's edit minimal. **Except a macro content type:** when the VBA project is dropped, a `Default` naming `application/vnd.ms-office.vbaProject` is removed if no part uses it | Documented that unused is allowed: [OPC 7.2.3.2.1][opc]. The exception is Inferred; [the VBA content type][sdk-vba] should not survive in a macro-free file |
| `INV-CT-07` | The stream has no attributes on `Types`, no markup-compatibility use, and no DOCTYPE | Documented: [OPC 6.2.5, 7.2.3.2.2, 7.2.3.2.3][opc] |
| `INV-CT-08` | Changing format changes only the main part's `Override`. No other type changes | Inferred from [the three types of one part][p1] and from the SDK's [type table][sdk-pres], which lists all seven main part types |
| `INV-CT-09` | A `Default` for `rels` is kept, though the standard exempts relationships parts | Inferred: our policy, since removing it gains nothing. [OPC 7.2.3.2.1][opc] does not require it |
| `INV-CT-10` | The media type of the main part agrees with the extension of the file requested. See [Save-as](#save-as-matrix) | Unverified: what PowerPoint does on a mismatch |

## Shared parts: layouts, media, themes

**Reference counts are not stored.** They are recomputed from the relationship graph at save, by a walk from the package root, so
they cannot drift and they cover parts the model does not know about. A theme's image, a diagram's image and an unknown part's
target all count.

| ID | Rule | Evidence |
| --- | --- | --- |
| `INV-SP-01` | A part is removed only if this session's edits made it unreachable. A part that arrived orphaned stays and stays reported (`CS-FND-0005`) | Inferred from "everything they name is retained", [28](28-PACKAGE-AND-PART-GRAPH.md). `SEC-014` asks whether orphan data is a threat: an answer changes this rule |
| `INV-SP-02` | A media part goes only with its last incoming relationship, from any source: slide, layout, master, notes, diagram data, chart, theme, `a14:imgLayer` or unknown | Documented that many sources are legal: [Media part][pptx-media], [Diagram data][oi-diag], [imgProps][odraw-pics3]. The walk is Inferred |
| `INV-SP-03` | Inserting media whose hash matches a part reuses that part | Inferred: by content hash, [22](22-NORMALIZED-SCHEMA.md) |
| `INV-SP-04` | **Copy on write:** editing a part with more than one referrer gives the editing object its own new part | Inferred: our policy, otherwise an edit leaks to other slides |
| `INV-SP-05` | Each slide has exactly one relationship to a layout | Inferred: [ECMA 13.3.9][p1] says a layout is the target of an implicit relationship from its slides. [PHPPresentation #266][php] reports the prompt with none. The [SDK does not require it][sdk-slide], so its validator will not catch it |
| `INV-SP-06` | At least one master and one layout. Each layout is on exactly one master's `sldLayoutIdLst` and points back to it. Removing a layout in use is refused at the op layer | Documented: [ECMA 13.3.9, 13.3.10][p1], [SDK requires a master][sdk-prespart]. Refusal is Inferred, our policy |
| `INV-SP-07` | At most one theme per master, and we never make two masters or a notes master share one. A theme shared on arrival stays shared until edited, then `INV-SP-04` applies | Documented: [ECMA 14.2.7][p1] "zero or one Theme part per" master. Inferred from [PptxGenJS #1449][pgx] for the notes master |
| `INV-SP-08` | The presentation's own theme relationship follows the first master's theme when that master goes | Unverified: which theme it should name |
| `INV-SP-09` | Deleting a slide removes its notes slide and its comments part, and its entries in custom shows, sections and zooms | Inferred: [ECMA 13.3.5][p1] gives one notes slide per slide with notes; custom-show rule in `INV-ID-05` |
| `INV-SP-10` | Copying a slide shares layout and media, and clones notes, charts with their workbooks, diagrams, ink and OLE. The clone's `relId` is rewritten. An unknown target is cloned and reported | Inferred: our policy. [python-pptx #132][py132] has a report of duplicated slides breaking, with `a16:creationId` suspected and disputed |

## Slide, master, layout and other ids

| ID | Rule | Evidence |
| --- | --- | --- |
| `INV-ID-01` | A slide id is in `[256, 2147483648)`, unique in the presentation | Documented: [`ST_SlideId`](../schemas/ooxml/pml.xsd), [ECMA 19.2.1.33][p1]. Reported: [python-pptx #89][py89], and [this Q&A][qa-copy] where the code's range looks wrong (the cause there is Inferred) |
| `INV-ID-02` | A new slide id is one above the highest; if that is out of range, the lowest free. Existing ids never change, because sections and zooms hold them | Inferred: our policy |
| `INV-ID-03` | Master and layout ids are `>= 2147483648`, and **drawn from one pool across all masters and layouts** | Documented for range and uniqueness: [ECMA 19.2.1.36, 19.3.1.40, 19.3.1.41][p1], [schema](../schemas/ooxml/pml.xsd). One shared pool is Inferred: a master's id is unique "throughout the presentation" and a layout's "within the presentation file" |
| `INV-ID-04` | A new master or layout id is one above the highest in that pool, capped at `4294967295`; on overflow, the lowest free. An id that arrived out of range is left and reported | Inferred: our policy |
| `INV-ID-05` | Custom shows: `id` is unique and `custShow` references name one that exists; ids are consecutive when we mint them; `p:sld` entries for a deleted slide go | Documented: [uniqueness and PowerPoint's consecutive writing][oi-cs6], [must be linked][oi-cs5] |
| `INV-ID-06` | Shape `cNvPr/@id` values are unique per slide, and a timing `spid` names a shape that exists. Copying a shape remaps both, and gives it a new `a16:creationId` | Documented as an SDK rule for PowerPoint: [`ReferenceExistConstraint` on `spid`][sdk-pml]. Uniqueness per slide is Inferred from [22](22-NORMALIZED-SCHEMA.md); whether a duplicate prompts is Unverified. `creationId` is Inferred from [python-pptx #132][py132] |
| `INV-ID-07` | Comment author `id` is unique | Documented as an SDK rule: [`UniqueAttributeValueConstraint`][sdk-pml] |
| `INV-ID-08` | New lists go at their schema position: `sldMasterIdLst`, `notesMasterIdLst`, `handoutMasterIdLst`, `sldIdLst` | Documented: [ECMA Part 1, Annex A.3][p1], vendored at [`CT_Presentation`](../schemas/ooxml/pml.xsd) |
| `INV-ID-09` | `firstSlideNum` stays within `0` to `9999` | Documented: [PowerPoint restricts it][oi-presel] |

## Section list

| ID | Rule | Evidence |
| --- | --- | --- |
| `INV-SL-01` | Sections live in `presentation.xml`'s `extLst`, `ext` uri `{521415D9-36F7-43E2-AB2F-B90AF26B5E84}`. A deck with none gets none on save | Documented: [MS-PPTX 2.2.5][pptx-sec]. Absent stays absent |
| `INV-SL-02` | The list holds at least one section: removing the last removes the `ext`, keeping other `ext` entries | Documented: [`minOccurs="1"`][pptx-seclist] |
| `INV-SL-03` | A section may be empty | Documented: [`minOccurs="0"`][pptx-sldlist] |
| `INV-SL-04` | Each `p14:sldId` names a slide in `sldIdLst`; a deleted slide's entry goes | Documented for the type: [`ST_SlideId`][pptx-entry]. Reference integrity is Inferred |
| `INV-SL-05` | **Each slide is in exactly one section, and reading the sections in order gives `sldIdLst` order** | Inferred: consistent with [the only example][pptx-secex], and no source states it. What PowerPoint does otherwise is Unverified |
| `INV-SL-06` | A new slide joins the section of the slide before it; at position 0, the first section | Inferred: our choice, no source specifies it |
| `INV-SL-07` | A section `id` is a brace-wrapped upper-case GUID, unique in the list, supplied by the op rather than sampled | Documented: [`s:ST_Guid`, unique][pptx-section] |

## Auto-created notes master

| ID | Rule | Evidence |
| --- | --- | --- |
| `INV-NM-01` | At most one notes master, the target of the presentation's explicit relationship and of each notes slide's implicit one | Documented: [ECMA 13.3.4][p1] |
| `INV-NM-02` | The first notes slide on a deck without a master creates one, in one change: `notesMaster1.xml`, its own new theme, presentation relationship, `notesMasterIdLst`, both `Override`s, and a report entry | Inferred: required by `INV-NM-01`. Own theme is from [PptxGenJS #1449][pgx], where PowerPoint's repair reportedly made `theme2.xml` |
| `INV-NM-03` | The created master has `cSld` and `clrMap`, plus placeholders `hdr`, `dt`, `sldImg`, `body`, `ftr`, `sldNum` sized from `notesSz`, from a template we author | Documented for root and `clrMap`: [ECMA 13.3.4][p1]. The six placeholders are Inferred from [PptxGenJS #1449][pgx], which says PowerPoint strips malformed ones. **Exact form is Unverified; compare with a PowerPoint-written master under `F-007`** |
| `INV-NM-04` | A notes slide has relationships to its slide and to the notes master | Documented: [ECMA 13.3.5][p1], [Office allows the slide relationship][oi-ns] |
| `INV-NM-05` | Removing the last notes leaves the master where it is; no handout master is ever auto-created | Inferred: absent stays absent, present stays present |

## `app.xml` and `core.xml`

Both are bookkeeping parts on the allow-list in [22](22-NORMALIZED-SCHEMA.md). Patched, never rebuilt, and **neither is created if absent.**

| ID | Rule | Evidence |
| --- | --- | --- |
| `INV-AP-01` | `Slides`, `HiddenSlides` and `Notes`, when present, equal the deck's counts | Documented meanings: [ECMA 22.2.2.9, 22.2.2.17, 22.2.2.24][p1]. Whether a stale count prompts is Unverified, so the fixture asks |
| `INV-AP-02` | `HeadingPairs` keeps its order and variant types; the sum of its counts is at least the number of `TitlesOfParts`; each vector's `size` matches | Documented: [order and types][oi-heading], [titles limit][oi-titles] |
| `INV-AP-03` | Slide titles are updated in place. If the groups are not recognised, **both** elements are removed and reported, never one, never stale | Inferred: our policy. Which groups PowerPoint writes is Unverified beyond the [example][oi-titles] |
| `INV-AP-04` | `AppVersion`, if we write it, matches `[0-9]{1,2}\.[0-9]{1,4}` | Documented: [Office's constraint][oi-appver] |
| `INV-AP-05` | On an edited save, `Application` becomes the host-supplied name; a file edited here should not claim another author's product. `TotalTime`, `Words`, `Paragraphs` and `MMClips` stay untouched | Inferred: our policy. Whether PowerPoint reads `Application`, or anything reads the stale counts, is Unverified |
| `INV-AP-06` | `DigSig` is removed when signatures are dropped | Documented that it exists: [ECMA 22.2.2.6][p1], [format][oi-digsig]. Inferred that it goes with them |
| `INV-AP-07` | At most one core properties part and one relationship to it, with [the right type][oi-core] | Documented: [OPC 8.2][opc] |
| `INV-AP-08` | `dcterms:modified` is set to the supplied time, with `xsi:type="dcterms:W3CDTF"`. With no time supplied, it is left alone. `created` never changes | Documented for form: [OPC 8.3.4.3][opc]. Policy Inferred |
| `INV-AP-09` | `lastModifiedBy` is set if the host supplies a name; `revision` goes up by one if it parses as an integer | Documented meanings: [OPC 8.1][opc]. Increment is Inferred |
| `INV-AP-10` | Retained `keywords` markup with `xml:lang` or `value` stays as it is | Documented: [OPC][opc] allows it, and [Office discards it on its own save][oi-keywords] (Microsoft files this under "Part 2 Section 7.3.6"; the content is about keywords) |

## Thumbnails

A stale thumbnail is not a cosmetic problem: it can show content the user deleted (`SEC-014`).

| ID | Rule | Evidence |
| --- | --- | --- |
| `INV-TH-01` | At most one package thumbnail relationship, internal, to a part with no relationships of its own | Documented: [ECMA 15.2.16][p1], [OPC 9][opc] |
| `INV-TH-02` | **After any edited save, a package thumbnail is either fresh or gone, never stale.** Removal takes the part, the relationship, and any `Override` | Inferred: our policy, for the reason above |
| `INV-TH-03` | Regenerated from the first slide in `sldIdLst`, by `-sdk`, as `docProps/thumbnail.png`. An existing `.jpeg` part is removed rather than overwritten. A deck opened with no thumbnail does not gain one; one dropped by a save within budget limits, such as an autosave, can return on a later save | Documented that any image type is allowed: [ECMA 15.2.16][p1]. Which slide PowerPoint uses, hidden or not, is Unverified |

## Percent-encoding

The bug behind `OPC-01`, confirmed by running: a space in a target against `%20` in the entry gave a false dangling and a false orphan.
That row fixes the reader. These are the rules for writing.

| ID | Rule | Evidence |
| --- | --- | --- |
| `INV-PE-01` | Only non-ASCII is encoded in the step from part name to ZIP item. So `/a%20b.png` is the ZIP item `a%20b.png`, literally | Documented: [OPC 7.3.4, 7.3.5][opc] |
| `INV-PE-02` | A name we mint needs no encoding, so `%` never appears in one | Inferred from `INV-PN-02` |
| `INV-PE-03` | A retained `Target` or `PartName` is written byte for byte. An attribute is not decoded and re-encoded on the way through | Inferred from the F2 contract and ADR-013 |
| `INV-PE-04` | Matching a target to a part uses one normalisation, in one place, the same at read and save: percent-encode space and brackets, encode a bare `%`, and treat raw and encoded non-ASCII as the same name, with ASCII case-insensitive comparison. Hostile forms stay refused by `is_safe_part_path`. `OPC-01` builds this | Documented for the kinds of change: [OPC Annex A][opc], which is informative. The choice of steps is Inferred |
| `INV-PE-05` | A new `Override` `PartName` uses the part name exactly as stored | Documented: [OPC 7.2.3.2.5, 7.2.3.5][opc] |
| `INV-PE-06` | A new external target from the user is encoded only where `xsd:anyURI` needs it, never encoding an existing `%XX` twice, and never fetched | Documented for the type: [OPC 6.5.3.4][opc]. The rest is Inferred. No-fetch is [AGENTS.md](../AGENTS.md) |

## Signatures dropped on edit (`WR-04`)

[07](07-SECURITY-THREAT-MODEL.md) promises the signature parts are dropped explicitly, in the report. Microsoft's page says a signed
presentation [becomes read-only][sig]. [One 2017 thread][qa-sig] reports Word and Excel removing rather than invalidating a signature on edit, with
a later reply saying it was fixed; nothing read covers PowerPoint. We never verify, repair or create a signature.

| ID | Rule | Evidence |
| --- | --- | --- |
| `INV-SG-01` | Signed means the package has a Digital Signature Origin relationship. Finding `CS-FND-0009` is raised on open | Documented: [OPC 10.4.2][opc] |
| `INV-SG-02` | **A signed deck opens read-only.** The first op is refused until the host calls an acknowledgement, and the host owns how it asks | Documented for read-only: [Microsoft][sig]. The acknowledgement is Inferred, our design |
| `INV-SG-03` | The drop removes the origin part, every signature part, every certificate part, their rels, the package relationship and their `Override`s | Documented for the parts: [OPC 10.4, Annex E][opc]. The cascade is Inferred |
| `INV-SG-04` | **Each drop writes one report entry, `CS-SAV-0001`, with the count of signatures removed,** and no signer names, per the rule that messages never carry document content | Inferred: our policy, from [07](07-SECURITY-THREAT-MODEL.md) |
| `INV-SG-05` | **The whole set goes, even if the edit touched a part no signature covers.** We do not read coverage | Inferred: our policy. Documented that other content can stay mutable: [OPC 10.3][opc] |
| `INV-SG-06` | All edits undone means nothing is dirty: the input bytes return, signature valid | Inferred from F1 |
| `INV-SG-07` | Save-as to another format drops signatures like an edit. An unedited save in the same format keeps them | Documented: [references to parts include the media type][opc] (OPC 10.5.7.3), so a retype invalidates one that covers the main part. Dropping all of them is Inferred |
| `INV-SG-08` | The VBA project signature, a separate part under the project, is untouched while the project is retained and goes with it | Unverified: no source read covers its location. Treated as reachable only through the project, under `INV-SP-01` |

A signature line picture ([`signatureLine`][odraw-legacy]) is left in place and named in the same report entry, since it may still look signed.

## Stale fallbacks

An edit can make a stored rendering wrong. For each one, the writer picks **regenerate**, **drop** or **block the edit**. The trigger may be
indirect: a theme change alters a SmartArt drawing's colours, and a slide edit alters a zoom preview.

**Render-on-save is hosted in `-sdk`,** because `-export` may not depend on `-render` ([19](19-WORKSPACE-SCAFFOLD-DESIGN.md)). So export
exposes two steps: `plan()` returns the stale list, then the SDK renders each entry and calls `finish(images)`. Rendering is a pure function of the
document, the engine version and the font manifest (`TXT-01`). An item that fails to render, or exceeds the host's budget, takes the "if render
is unavailable" column; **a render failure never fails a save.**

`-render` arrives in Phase 2 and editing these constructs in Phase 6, which is the `WR-03` gate. Until each row's regenerate path exists, its
"if render is unavailable" column applies.

| ID | Construct, what it stores | Direct trigger | Indirect trigger | Policy | If render is unavailable | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| `INV-FB-01` | **SmartArt drawing.** Diagram Layout part, root `dsp:drawing`: the last successful layout | Data edit | Theme, colour map, fonts, frame size | **Regenerate**, which needs a diagram layout engine (row below). Move is safe | **Block** data edit. Indirect: keep, report stale | Documented: [part][diag-part], [overview][diag]. Unverified: whether PowerPoint re-lays out on load |
| `INV-FB-02` | **ChartEx.** Part `application/vnd.ms-office.chartex+xml`, referenced by `chart` in the chartex namespace | Data or format edit | Theme, fonts | Classify the fallback. A picture: **regenerate**. A static notice: nothing is stale, keep it | **Block** edit | Documented: [part][cx-part], [`chart`][cx-chart]. Unverified: no source read documents the slide-level `AlternateContent`, or what its fallback holds |
| `INV-FB-03` | **OMML.** `a14:m` in `mc:Choice`, fallback `p:sp` or `p:graphicFrame` | Any text edit in that shape, since the fallback is the whole shape | Theme, fonts, size | **Regenerate** the fallback, as math layout allows. Other shape properties patch both branches | **Block** text edit in that shape. Indirect: keep, report stale | Documented: [structure][math]. Inferred: whole-shape staleness. Unverified: that the fallback is a picture |
| `INV-FB-04` | **SVG's PNG.** Main `a:blip` is the PNG, `svgBlip` the SVG | SVG replaced or recoloured | None for size or crop: one blip, same crop | **Regenerate** PNG from SVG. **Never drop the PNG** | **Block** SVG edit. On replacing a picture, replace both or neither | Documented: [Office embeds a PNG in the main blip][pics]. Crop is Inferred |
| `INV-FB-05` | **Zoom pictures.** `sldZm`, `sectionZm` fallback `pic`, `summaryZm` fallback `grpSp`; the object has `blipFill` and `imageType` | Target slide or section edited | Theme, master | `imageType="preview"`: **regenerate** both images. A custom cover: never stale. Target deleted: replace the `AlternateContent` with its fallback, with no link, and report | Keep, report stale | Documented: [structure][zoom], [`imageType`][zoom-props]. Inferred: cover is never stale; handling of a deleted target |
| `INV-FB-06` | **Ink.** `contentPart` in `mc:Choice` (p14), fallback `pic` | Stroke edit | None known | **Regenerate.** Move and resize patch both branches | **Block** stroke edit | Documented: [structure][ink], [part][ink-part] |
| `INV-FB-07` | **3D.** `model3d`, glTF by `r:embed`, plus `raster` with renderer name and version | Camera, transform, lights, model | Theme | **Block** the edit. We plan no glTF renderer. Move and resize patch both branches; non-uniform resize is blocked | Same | Documented: [`CT_Model3D`][m3d], [raster][m3d-raster], [SDK type][sdk-m3d]. Unverified: the outer fallback, and whether a stale raster is redrawn |
| `INV-FB-08` | **Package thumbnail** | Any edit | Any edit | **Regenerate,** see `INV-TH-02` | **Drop** | Documented: [ECMA 15.2.16][p1]. Inferred: never stale |

| ID | Rule | Evidence |
| --- | --- | --- |
| `INV-FB-09` | **Both branches stay valid after any edit.** Geometry changes patch `mc:Choice` and `mc:Fallback`. The structure and `Requires` are not altered, and both branches stay, including one we did not pick | Documented for structure: [ISO 29500-3 7.5 to 7.7][mce]. The rest is Inferred; see `MCE-01` |
| `INV-FB-10` | A fallback kept stale is a report entry, `CS-SAV-0007`, naming the slide part and the construct | Inferred, from "no silent data loss" |
| `INV-FB-11` | A host can set stale handling to `drop` for `INV-FB-01`, `-03` and `-05`, since a stale image can disclose removed content. Default is keep and report | Inferred: host owns policy; `SEC-014` |
| `INV-FB-12` | Regenerated images are new parts with `INV-SP-04`; the old ones go only when no referrer is left | Inferred from `INV-SP-02` |

Also retained without edit, never regenerated: OLE preview images and media poster frames, as in [22](22-NORMALIZED-SCHEMA.md); `a14:imgProps`
originals. Editing a picture with [non-destructive data][pics] needs its own row.

### SmartArt with no drawing part

This is legal, not corrupt: [the drawing is an extension][diag] to a format that defines only data, layout, style and colours.

| ID | Rule | Evidence |
| --- | --- | --- |
| `INV-FB-13` | **Open:** retained, with a finding (code proposed below). No drawing is invented | Documented for optional: [overview][diag]; [`CT_RelIds`](../schemas/ooxml/dml-diagram.xsd) requires four ids, none of them a drawing |
| `INV-FB-14` | **Render:** until a diagram layout engine exists, the frame draws an unmistakable placeholder and the report says "parsed, not rendered" | Inferred, from "report outcomes faithfully" in [AGENTS.md](../AGENTS.md). What PowerPoint does is Unverified, though the overview suggests it lays out from data |
| `INV-FB-15` | **Save:** the graphic frame and all four parts are copied unchanged. No drawing, `relId` or relationship is written, so no empty drawing can mislead a reader. Move and resize are allowed; data edits are blocked | Inferred, our policy, and the row below |

**As built, Tier 1K.** `INV-FB-01`: `save_diagram` blocks a data edit the stored drawing does not show as `CS-SAV-0009`, and `Diagram::set_point_text` makes the one data edit it can show, text in the point and in its shapes, with geometry not recomputed. A definition edit is kept, and `DiagramSave::stale_drawing` says so. `INV-FB-03`: an inline equation's edit with its fallback unchanged is refused as `CS-EXP-0009`, and editing both is allowed; an equation in a shape-level `mc:Choice` is read and its edit is `CS-SAV-0009`. `INV-FB-06`: a stroke edit is saved, and `InkSave::stale_fallback` says the picture shows the old strokes. Regenerating any of these is Tier 2's.

## Save-as matrix

Rows open, columns save. Each cell is a fixture: **`RR-SA-<open>-<save>`**, with 121 in all, so `WR-05`'s gate is the grid.

| Code | Means |
| --- | --- |
| `=` | Same format. Unedited: F1, input bytes. Edited: the pipeline above |
| `T` | Retype: change the main part's `Override` only. No VBA present |
| `K` | Retype, **keeping VBA** |
| `V` | Retype, **dropping VBA**, only with the host's explicit confirmation; otherwise refused with `CS-SAV-0003` |
| `N` | New deck from a theme, not a retype, and the report says so |
| `C` | Convert through the model: report names every loss |
| `CV` | Same as `C`, and VBA is dropped with confirmation |
| `R` | Render through Tier 2G. Nothing of the package is carried over; VBA does not run and is not carried |
| `X` | Refused with `CS-SAV-0013`: left off the menu, per "a verb we lack is left off the ribbon" |

| Open \ Save | .pptx | .pptm | .ppsx | .ppsm | .potx | .potm | .thmx | .ppam | .odp | .pdf | .ppt |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **.pptx** | = | T | T | T | T | T | X | X | C | R | X |
| **.pptm** | V | = | V | K | V | K | X | X | CV | R | X |
| **.ppsx** | T | T | = | T | T | T | X | X | C | R | X |
| **.ppsm** | V | K | V | = | V | K | X | X | CV | R | X |
| **.potx** | T | T | T | T | = | T | X | X | C | R | X |
| **.potm** | V | K | V | K | V | = | X | X | CV | R | X |
| **.thmx** | N | N | N | N | N | N | = | X | C | R | X |
| **.ppam** | X | X | X | X | X | X | X | = | X | X | X |
| **.odp** | C | C | C | C | C | C | X | X | = | R | X |
| **.pdf** | X | X | X | X | X | X | X | X | X | X | X |
| **.ppt** | C | X | C | X | C | X | X | X | C | R | X |

Notes for `=`: `.thmx`, `.ppam` and `.odp` keep F1 for an unedited save. `.thmx` and `.ppam` are not editable; the edited writer for `.odp` is Phase 10.

| ID | Rule | Evidence |
| --- | --- | --- |
| `INV-SA-01` | Seven main part types: presentation, slideshow, template, their three macro-enabled forms, and add-in | Documented: [ECMA 13.3.6][p1] for the three plain types, [SDK table][sdk-pres] for all seven |
| `INV-SA-02` | The same schema and parts in all of `.pptx`, `.pptm`, `.ppsx`, `.ppsm`, `.potx`, `.potm`, so the retype is one edit | Documented: [`.pptm` is "the same basic XML format"][ext]; [ECMA 13.3.6][p1]. Extending it to all six is Inferred |
| `INV-SA-03` | **VBA is never in a macro-free output.** `.pptx` and `.ppsx` [cannot contain it][ext]; `.potx` is [made when there is none][ext]. Asked for a macro-free type with VBA present and no confirmation, the save is refused | Documented for the formats. What PowerPoint does with a mismatch is Unverified: OpenCalc's doc reports Excel repairing by deleting the project |
| `INV-SA-04` | **Dropping VBA** removes `vbaProject.bin`, everything reachable only through it, the relationship, its type, and actions that name a macro. Report `CS-SAV-0002` gives counts | Documented: [part and type][sdk-vba]; [`.pptx` cannot contain "Action settings"][ext] (archived 2010 page). Reading that as macro actions only is Inferred, and how a macro action is spelled is Unverified: ECMA Part 1's only `ppaction://` example is `media` |
| `INV-SA-05` | **A macro type with no VBA** is allowed: `T` to `.pptm` adds no part | Inferred: [`.dotm` "does not always contain macro code"][ext] for Word. Unverified for PowerPoint |
| `INV-SA-06` | **VBA, ActiveX and custom UI are retained bytes.** Never run, never recompiled, and the retype leaves them. ActiveX and `customUI` in a macro-free output stay, reported | Inferred: project rule [SEC-C07](07-SECURITY-THREAT-MODEL.md). Whether they are valid in `.pptx` is Unverified |
| `INV-SA-07` | **Template to deck has two verbs.** *Save as* is a retype and keeps VBA in a macro type. *New from template* follows [presentations created from it "do not inherit the VBAProject part"][ext], and also starts with a new `created` and no signature | Documented for VBA. The rest is Inferred |
| `INV-SA-08` | **`.ppam` opens retained and read-only, and only returns its own bytes.** Creating one is refused: [all add-ins are macro-enabled][ext], so we would need to build a valid one with no way to test it | Documented for macro-enabled. Refusal is Inferred, and revisited after `PV-01` |
| `INV-SA-09` | **`.thmx` opens as an offer to start a deck or apply the theme, not as a deck.** Save-as `.thmx` is refused until a PowerPoint-written file is in `fixtures/producers/` | **Unverified:** no Microsoft spec found. [PRONOM][pronom] gives `themeManager+xml` as the signature; layout from secondary sources only |
| `INV-SA-10` | **`.ppt` import is Phase 10,** saved as `.pptx` or another macro-free type (`DEC-08`), and refused as `CS-OPC-0005` until then. Writing `.ppt` is refused. A `.ppt` holding VBA needs the same confirmation as `CV`; `.pptm` and other macro types are refused until `PPT-01` decides whether the project can be carried | Inferred: project decision `DEC-08`. VBA storage is Unverified |
| `INV-SA-11` | **`.odp` is Phase 10.** The report names losses per [Microsoft's table][odp]: themes, comments and macros unsupported, charts, tables, masters partial. VBA does not survive. `.odp` to `.pptm` adds no part | Documented for PowerPoint's own conversion. Inferred for ours, measured when built |
| `INV-SA-12` | **`.pdf` is output only.** `R` needs Tier 2G; opening a PDF is refused | Inferred: project scope in [06](06-ROADMAP-AND-DELIVERY.md) |
| `INV-SA-13` | `.ppsx` opened here does not autoplay and is edited like a deck. Auto-show on open is a threat in `SEC-014` | Inferred, from [`.ppsx` being "auto-running"][ext] |
| `INV-SA-14` | Save-as leaves the file's Strict or Transitional form alone | Inferred: project decision `STRICT-01`, [parts.md](spec/parts.md) |

Other outputs named by `WR-05`. JPEG and PNG per slide: Tier 2G, from the [Download list](48-POWERPOINT-FEATURE-STUDY.md) (Inferred: project scope). `.xps`, outline `.rtf`, `.sldx`,
`.sldm`, `.pps`, `.pot`, `.ppa`: refused, `CS-SAV-0013` (Inferred: our policy). [Microsoft lists][formats] `.xps` as Windows-only. Each awaits a row if a user need appears.

## Repair triggers

What would have to be true for a file to prompt. **The prompt is never Documented.** "Reported" means a third party says they saw it, with the URL, and we did not reproduce it.

| Trigger | Source | Status of the prompt | Guarded by |
| --- | --- | --- | --- |
| Slide with no layout relationship | [PHPPresentation #266][php] | Inferred: reported, with fix merged | `INV-SP-05` |
| Duplicate slide id | [python-pptx #89][py89] | Inferred: reported as a "warning" | `INV-ID-01` |
| Slide id out of range | [Q&A: repair copying slides][qa-copy]: error reproduced. The posted code starts slide ids from `2147483648`, but the thread never names the cause | Inferred: our reading of the code | `INV-ID-01` |
| Duplicate ZIP entries | [python-pptx #67][py67] | Inferred: reported as corrupt | `INV-PN-03` |
| Stale slide list entry after delete | [python-pptx #67][py67]: mixed reports | Inferred | `INV-PN-07`, `INV-SP-09` |
| Phantom `slideMaster` `Override` | [PptxGenJS #1449][pgx]; a commenter removed them and still saw the prompt | Inferred, weak | `INV-CT-05` |
| Notes master missing or malformed placeholders; sharing a theme | [PptxGenJS #1449][pgx]: post-repair state described | Inferred | `INV-NM-02`, `-03` |
| Invalid preset, such as `roundedRectangle` | [PptxGenJS #1449][pgx] | Inferred. Schema catches it (F4) | Patch checks enums |
| `custDataLst` in `presentation.xml` and a slide, called valid by the answerer | [Microsoft Q&A][qa-cust]: the accepted answer removed them and the file opened, with no "full explanation" | Inferred: isolated by experiment, cause unknown. **A validator may not catch this** | Retained only, never minted. No invariant yet: no rule can be drawn |
| Non-ASCII part name | [MS-OI29500][oi-naming] | Documented as refusing to load, not as a repair | `INV-PN-02` |
| Notes master relationship to a missing theme | [LibreOffice bug 155426][lo], seen as a search snippet only; the page refused automated access | Unverified | `INV-RI-04` |
| Thumbnail relationship on a slide, layout or master; other relationship shapes Office "does not allow" | [MS-OI29500][oi-slide] | Inferred from "does not allow" | `INV-RI-08` |
| Custom show `id` not in the list | [MS-OI29500][oi-cs5] "requires" | Inferred | `INV-ID-05` |
| Timing `spid` with no shape | [SDK][sdk-pml], scoped to PowerPoint | Inferred | `INV-ID-06` |
| `AppVersion` or `HeadingPairs` out of form | [MS-OI29500][oi-appver], [here][oi-heading] | Inferred | `INV-AP-02`, `-04` |
| `firstSlideNum` above 9999 | [MS-OI29500][oi-presel] | Inferred | `INV-ID-09` |
| Duplicate or dangling relationship id | [OPC 6.5.3.4][opc]; [SDK][sdk-pml] | Inferred | `INV-RI-01`, `-04` |
| Empty `sectionLst` | [MS-PPTX][pptx-seclist] | Inferred from schema | `INV-SL-02` |
| VBA in a macro-free type | [`.pptx` cannot contain][ext]. Excel behaviour in OpenCalc's doc | Unverified for PowerPoint | `INV-SA-03` |
| Section list out of step | No source | Unverified | `INV-SL-05` |
| Main part type against file extension | No source | Unverified | `INV-CT-10` |
| Stale `app.xml` counts | No source | Unverified | `INV-AP-01` |
| Master id equal to a layout id | No source for the prompt | Unverified | `INV-ID-03` |
| Duplicate shape id | No source | Unverified | `INV-ID-06` |
| Malformed `AlternateContent` | [ISO 29500-3][mce] | Inferred from schema | `INV-FB-09` |

**F4 and `F-006` are necessary and not sufficient.** A file reported valid can still prompt (`custDataLst`), and the SDK's part data does not require a layout relationship.
So `PV-01` stays a release gate, and "matches PowerPoint" stays unclaimable until it passes.

## Repair-risk fixtures

**One per invariant.** Each is generated by `fixtures/tools/generate.py`, as a pair:

- **Good:** the deck after the stated edit, through the writer. Preflight, F4, `F-006` and F7 must pass.
- **Seeded:** the same deck with this one invariant broken. With the break injected into the plan by a test hook, preflight must refuse the save, so the check has been seen to fail. Given as an input deck, it must open and save with a report entry.

**`PV-01` opens both.** That converts each Inferred or Unverified mark above into an observation: the good one must not prompt, and whether the seeded one prompts tells us what the mark should say. Until `F-006` and `F4-01` land, only preflight and F7 run.

| Fixture | Invariant | Edit on the good deck | Seeded break |
| --- | --- | --- | --- |
| `RR-PN-01` | `INV-PN-01` | Add a slide | Name with a trailing dot, and with `%2F` |
| `RR-PN-02` | `INV-PN-02` | Add an image from `résumé.png` | Part named `ppt/media/é.png` |
| `RR-PN-03` | `INV-PN-03` | Add a slide to a deck holding `Slide1.xml` | Add `slide1.xml` too |
| `RR-PN-04` | `INV-PN-04` | Add a slide | `ppt/slides/slide1.xml/x` |
| `RR-PN-05` | `INV-PN-05` | Delete slide 2, add a slide | Reuse a name within a save |
| `RR-PN-06` | `INV-PN-06` | Delete slide 2 of 3 | Renumber the rest |
| `RR-PN-07` | `INV-PN-07` | Delete a slide with rels | Leave its `.rels` |
| `RR-PN-08` | `INV-PN-08` | Retain a percent-encoded non-ASCII entry | Raw UTF-8 item name |
| `RR-PN-09` | `INV-PN-09` | Edit a `Store` deck | New entry with another method |
| `RR-PN-10` | `INV-PN-10` | Any edit | Local and central sizes differ |
| `RR-PN-11` | `INV-PN-11` | Any edit | `[Content_Types].xml` moved last |
| `RR-RI-01` | `INV-RI-01` | Add an image | Two equal ids; id `1a` |
| `RR-RI-02` | `INV-RI-02` | Edit a slide with SmartArt | Renumber slide rels |
| `RR-RI-03` | `INV-RI-03` | Add to rels with `rId7`, `foo` | Collide with `rId8` |
| `RR-RI-04` | `INV-RI-04` | Delete a picture | Leave `r:embed` dangling |
| `RR-RI-05` | `INV-RI-05` | Edit with an unknown relationship type | Drop it |
| `RR-RI-06` | `INV-RI-06` | Any edit | `xml:base` in a rels part |
| `RR-RI-07` | `INV-RI-07` | Add to a deck with absolute targets | Rewrite a retained target |
| `RR-RI-08` | `INV-RI-08` | Add a layout | Thumbnail relationship on a slide |
| `RR-RI-09` | `INV-RI-09` | Edit a Strict deck | Write a Transitional type |
| `RR-RI-10` | `INV-RI-10` | Use one image twice on a slide | Two relationships |
| `RR-CT-01` | `INV-CT-01` | Add a `.webp` image | No matching entry |
| `RR-CT-02` | `INV-CT-02` | Any edit | Two `Default`s for `PNG` and `png` |
| `RR-CT-03` | `INV-CT-03` | Add an extensionless part; add `.png` with a conflicting `Default` | `Default` left conflicting |
| `RR-CT-04` | `INV-CT-04` | Add a slide and an image | Slide by `Default xml` |
| `RR-CT-05` | `INV-CT-05` | Delete a slide | Leave its `Override` |
| `RR-CT-06` | `INV-CT-06` | Drop VBA where `bin` `Default` was VBA's | Leave it |
| `RR-CT-07` | `INV-CT-07` | Any edit | Attribute on `Types` |
| `RR-CT-08` | `INV-CT-08` | `.pptx` to `.potx` | Change a slide's type too |
| `RR-CT-09` | `INV-CT-09` | Any edit | No `rels` `Default` |
| `RR-CT-10` | `INV-CT-10` | `.pptx` to `.ppsx` | Slideshow type with `.pptx` extension |
| `RR-SP-01` | `INV-SP-01` | Edit a deck with an orphan | Sweep the orphan |
| `RR-SP-02` | `INV-SP-02` | Delete one of two slides sharing an image, and an image shared with a theme | Delete the image |
| `RR-SP-03` | `INV-SP-03` | Insert a duplicate image | A second part |
| `RR-SP-04` | `INV-SP-04` | Replace a shared image on one slide | Overwrite in place |
| `RR-SP-05` | `INV-SP-05` | Add a slide | No layout relationship |
| `RR-SP-06` | `INV-SP-06` | Remove an unused layout | Layout not on the list; no layouts |
| `RR-SP-07` | `INV-SP-07` | Add a master; edit a shared theme | Two masters on one theme |
| `RR-SP-08` | `INV-SP-08` | Remove the first master | Theme relationship dangling |
| `RR-SP-09` | `INV-SP-09` | Delete a slide with notes and comments | Leave the notes slide |
| `RR-SP-10` | `INV-SP-10` | Duplicate a slide with chart, SmartArt, ink, OLE | Share the chart; keep the old `relId` |
| `RR-ID-01` | `INV-ID-01` | Add a slide | Id 255; id 2147483648; duplicate |
| `RR-ID-02` | `INV-ID-02` | Add to a deck whose top id is 2147483647 | Overflow |
| `RR-ID-03` | `INV-ID-03` | Add a layout | Layout id equal to a master id |
| `RR-ID-04` | `INV-ID-04` | Add to a deck at `4294967295` | Wrap-around |
| `RR-ID-05` | `INV-ID-05` | Delete a slide in a custom show | `custShow` naming a missing id |
| `RR-ID-06` | `INV-ID-06` | Copy an animated shape | `spid` not remapped; duplicate id |
| `RR-ID-07` | `INV-ID-07` | Add a comment author | Duplicate |
| `RR-ID-08` | `INV-ID-08` | Add notes to a deck with no list | List after `sldSz` |
| `RR-ID-09` | `INV-ID-09` | Set `firstSlideNum` to 9999 | 10000 |
| `RR-SL-01` | `INV-SL-01` | Add a slide with no sections | Section list minted |
| `RR-SL-02` | `INV-SL-02` | Remove the only section | Empty list |
| `RR-SL-03` | `INV-SL-03` | Move all slides out of a section | Section removed |
| `RR-SL-04` | `INV-SL-04` | Delete a slide | Entry left |
| `RR-SL-05` | `INV-SL-05` | Move a slide across sections | Order differs; slide in none; slide in two |
| `RR-SL-06` | `INV-SL-06` | Add at the start and at a boundary | Unassigned |
| `RR-SL-07` | `INV-SL-07` | Add a section | Lower-case GUID; duplicate |
| `RR-NM-01` | `INV-NM-01` | Add notes to a deck with a master | A second master |
| `RR-NM-02` | `INV-NM-02` | First notes on a deck with none | Theme shared; no `Override` |
| `RR-NM-03` | `INV-NM-03` | Same, on 16:9 and 4:3 | Missing `clrMap`; missing placeholder |
| `RR-NM-04` | `INV-NM-04` | Add notes | No relationship to master |
| `RR-NM-05` | `INV-NM-05` | Remove the last notes | Master removed |
| `RR-AP-01` | `INV-AP-01` | Add, hide, delete a slide | Counts left stale |
| `RR-AP-02` | `INV-AP-02` | Add a slide | More titles than the sum |
| `RR-AP-03` | `INV-AP-03` | Edit with unknown groups | One of the two removed |
| `RR-AP-04` | `INV-AP-04` | Any edit | `AppVersion` of `1.2.3` |
| `RR-AP-05` | `INV-AP-05` | Edit a PowerPoint-written deck | `Application` retained |
| `RR-AP-06` | `INV-AP-06` | Edit a signed deck with `DigSig` | Retained |
| `RR-AP-07` | `INV-AP-07` | Any edit | Two core parts |
| `RR-AP-08` | `INV-AP-08` | Edit with and without a time | No `xsi:type`; `created` changed |
| `RR-AP-09` | `INV-AP-09` | Edit with integer and text `revision` | Non-integer overwritten |
| `RR-AP-10` | `INV-AP-10` | Edit with `xml:lang` keywords | Flattened |
| `RR-TH-01` | `INV-TH-01` | Any edit | Two thumbnail relationships |
| `RR-TH-02` | `INV-TH-02` | Edit with no renderer | Stale part kept |
| `RR-TH-03` | `INV-TH-03` | Edit with a renderer, from `.jpeg` | Both parts present |
| `RR-PE-01` | `INV-PE-01` | Edit a deck with `my%20image.png` | Decoded item name |
| `RR-PE-02` | `INV-PE-02` | Add `a b%.png` | `%` in a minted name |
| `RR-PE-03` | `INV-PE-03` | Edit with a target of `a b.png` and of `a%20b.png` | Target normalised on write |
| `RR-PE-04` | `INV-PE-04` | Same, plus `%zz` | Dangling or double match |
| `RR-PE-05` | `INV-PE-05` | Add a slide beside a percent-encoded part | `PartName` re-encoded |
| `RR-PE-06` | `INV-PE-06` | Set a link with space and `%41` | `%2541` |
| `RR-SG-01` | `INV-SG-01` | Open a signed deck | Origin part absent, signature present |
| `RR-SG-02` | `INV-SG-02` | Op with and without acknowledgement | Op accepted unacknowledged |
| `RR-SG-03` | `INV-SG-03` | Edit a signed deck | Certificate left |
| `RR-SG-04` | `INV-SG-04` | Same | No entry |
| `RR-SG-05` | `INV-SG-05` | Edit outside signed coverage | Partial kept |
| `RR-SG-06` | `INV-SG-06` | Edit, undo, save | Bytes differ |
| `RR-SG-07` | `INV-SG-07` | Signed `.pptx` to `.potx`; unedited same-format | Kept across retype |
| `RR-SG-08` | `INV-SG-08` | Edit a deck with signed VBA and a signed package | VBA signature removed |
| `RR-FB-01` | `INV-FB-01` | Theme change on SmartArt | Silent stale |
| `RR-FB-02` | `INV-FB-02` | Move a ChartEx; attempt an edit | Edit accepted |
| `RR-FB-03` | `INV-FB-03` | Text edit next to an equation, attempted | Accepted, fallback stale |
| `RR-FB-04` | `INV-FB-04` | Replace a picture holding SVG and PNG | Only one replaced |
| `RR-FB-05` | `INV-FB-05` | Edit a zoom target; delete a target; cover image | Dangling zoom |
| `RR-FB-06` | `INV-FB-06` | Move ink; attempt stroke edit | One branch moved |
| `RR-FB-07` | `INV-FB-07` | Move 3D; attempt rotate | Edit accepted |
| `RR-FB-08` | `INV-FB-08` | Edit, with and without renderer | Stale kept |
| `RR-FB-09` | `INV-FB-09` | Move each `AlternateContent` construct | Branch removed; `Requires` changed |
| `RR-FB-10` | `INV-FB-10` | Stale kept | No entry |
| `RR-FB-11` | `INV-FB-11` | Host sets `drop` | Kept |
| `RR-FB-12` | `INV-FB-12` | Regenerate a shared preview | Overwritten |
| `RR-FB-13` | `INV-FB-13` | Open SmartArt with no drawing | Drawing invented |
| `RR-FB-14` | `INV-FB-14` | Render it | Blank, no report |
| `RR-FB-15` | `INV-FB-15` | Move it; save | `relId` written |
| `RR-SA-01` | `INV-SA-01` | Each retype | Wrong type string |
| `RR-SA-02` | `INV-SA-02` | Each retype, part-hash compare | Another part changes |
| `RR-SA-03` | `INV-SA-03` | `.pptm` to `.pptx` without confirmation | Saved anyway |
| `RR-SA-04` | `INV-SA-04` | With confirmation; deck has macro actions | Action left |
| `RR-SA-05` | `INV-SA-05` | `.pptx` to `.pptm` | A part added |
| `RR-SA-06` | `INV-SA-06` | Retype with ActiveX and `customUI` | Dropped silently |
| `RR-SA-07` | `INV-SA-07` | `.potm`: both verbs | New deck has VBA |
| `RR-SA-08` | `INV-SA-08` | Open `.ppam`, save; attempt edit | Edit accepted |
| `RR-SA-09` | `INV-SA-09` | Open `.thmx` from `F-007` | Treated as a deck |
| `RR-SA-10` | `INV-SA-10` | `.ppt` before and after Phase 10 | `.ppt` written |
| `RR-SA-11` | `INV-SA-11` | `.pptx` with each listed feature to `.odp` | Loss unreported |
| `RR-SA-12` | `INV-SA-12` | Open a PDF | Not refused |
| `RR-SA-13` | `INV-SA-13` | Open `.ppsx` | Show starts |
| `RR-SA-14` | `INV-SA-14` | Retype a Strict deck | Form changes |

Plus the 121 grid fixtures `RR-SA-<open>-<save>`, which use the same `INV-SA-*` checks.

## Implementation status

Tier 1N, `T1N-01` to `T1N-08`. **Nothing here was observed in PowerPoint; the marks above stand as written.** "Built" means the rule runs in preflight or at emission, a good deck passes it, and a seeded break was seen refused, with the rule also seen to fail when disabled.

**The writer.** A `SavePlan` holds model patches, replaced parts and `Op`s. `SavePlan::stage` applies them to a `Draft` beside the untouched input; `Draft::preflight` runs every rule on both, so what the save added is the difference; `Draft::emit` writes, then reads the ZIP back. `save_deck` and `save_slide_parts` are plans of one kind of change. Model patches run first, on the deck as it arrived; then replaced parts; then operations in order; then the sweep.

**New shapes and a blank deck (`VS-08`).** `Op::InsertShape` puts an empty `p:sp`, `p:cxnSp` or `p:pic` with its `cNvPr/@id` and name into a slide's shape tree at a z-order place; a picture's bytes go through the same media reuse as `AddPicture`. The shape's look and text come from a later slide-part patch, so a new shape passes the same read-back and round-trip check as an edited one, and nothing is serialized from the model whole. The editor's save runs three plans: slides (`AddSlide`, `DuplicateSlide`, `DeleteSlide`, `MoveSlide`), then shapes (`RemoveShape`, `InsertShape`), then the patch, pairing shapes by `cNvPr/@id` (`casual-slide-wasm` `build.rs`). `blank::blank_deck` is File > New: plain XML parts in `src/blank/` zipped by `casual-slide-package`'s `build_package`, deterministic; a deck built from it passes the SDK validator (`casual-slide-wasm` `tests/build.rs`).

**Where this departs from the text above, on purpose:**

- **A clone keeps its relationship ids.** `INV-SP-10` says the clone's `relId` is rewritten. Its rels part is a copy with the same ids, so `relId` needs no rewriting and the slide stays a byte copy; preflight checks that it lands on the clone's own drawing. Same outcome, fewer edits.
- **A model may renumber on purpose.** `INV-ID-02` and `-04` judge what the writer allocates. When a `Deck` patch rewrites the main part they stand aside, since an existing test asks for exactly that, and `INV-SL-04` is what stops one that orphans a section entry.
- **`INV-RI-04`, `INV-ID-06` and `INV-ID-07` read bodies only of parts the save wrote**, or a save would parse every slide. So a seeded deck with one of these breaks is reported when that part is next written, not on any save.
- **Recovery decides what a seeded deck does, where this page and [29](29-RECOVERY.md) differ.** "Given as an input deck, it must open and save with a report entry" is read under instruction 24, and every seeded deck meets it: it opens, saves, and reports through the writer or, for what the reader repaired, in the recovery report. `fixtures/rr/manifest.tsv` says which. `RR-PN-01`'s names cannot be written by `Package::rewrite`, so no seeded file is committed, and a test builds them in a ZIP by hand. A break that needs two packages to see, such as a renumbered id, opens and saves with no entry, since one package does not show it.
- **Bookkeeping follows the save, not the arrival.** `INV-AP-01` is held to what this save changed: a count that arrived stale, with nothing here changing it, stays and is reported, because correcting it would touch a part no one edited (F2). A change in the count brings the field to the truth.
- **A `Deck` patch answers for `app.xml`.** Like `INV-ID-02`, `INV-AP-01` to `-03` stand aside when a model patch rewrites the main part or `app.xml`: the model states both, so a slide removed by a `Deck` patch alone leaves the counts as that model has them, and that is reported as `INV-AP-01`, not refused. What moves the slides after the patch, an operation or a slide patch, the counts and titles still follow (`T1B-02`), in the same save, even if that patch wrote `app.xml`.
- **`core.xml` moves only with host inputs.** `INV-AP-09` reads as if `revision` rises on any edited save. A plan with no `HostInputs` is a library patch and leaves `core.xml` alone, so model round trips stay fixed points. The SDK supplies them.
- **`save_deck`, `save_slide_parts` and the table-styles save acknowledge for their caller.** `INV-SG-02`'s refusal is on `SavePlan`; these three are whole edits already asked for, and return `signature_invalidated`. Each still drops the set.
- **An `RR-*` deck comes from the writer, not `generate.py`.** Python builds the inputs (`fixtures/tools/generate_rr.py`), and `tests/writer_rr.rs` writes `fixtures/rr/good` and `fixtures/rr/seeded`, with `RR_WRITE=1`. `fixtures/rr/manifest.tsv` says what became of each seeded deck: a name `Package::rewrite` refuses cannot be written, and two entries naming one part cannot be opened, which are second lines of defence.

**Guards seen red, 2026-11-08.** Each `INV-AP`, `-TH`, `-SG` and `-SA` preflight guard was switched off in turn and `writer_rr`, `writer_saveas` and `writer` run: 28 of 38 went red, and so did removing `check_zip` from `emit`. Of the 10 left green, 9 now have a seed in `writer_rr` and were seen red; the tenth, `INV-AP-03`'s one-of-two-removed check, duplicated the both-or-neither check and was removed. **What PowerPoint does with any of these decks is not proven here:** there is no PowerPoint on this machine, so every Inferred or Unverified mark stands until `PV-01`.

**Not run: F7.** The owner stopped LibreOffice while it was crashing on their machine, so results of this writer are validated with F4 and `F-006` only until told otherwise. A partial earlier run is no evidence of a pass.

**Not built, and not in these rows:** a zoom naming a deleted slide is refused (`CS-SAV-0009`) rather than replaced by its fallback, which is `INV-FB-05`. The `PE` and `FB` families have no rows here, and no `RR-PE-*` or `RR-FB-*` fixtures: stale fallbacks wait on `WR-03` in Phase 6. The thumbnail's regenerate path takes a PNG from the host, and nothing can make one until Tier 2G; so today every edit takes the drop path, and one dropped does not return (`INV-TH-03` allows it). What a signature line is found by, `signatureline` in a VML part, is Unverified.

**The save-as grid (`T1N-06`).** `save_as` follows the grid above, and `tests/writer_saveas.rs` reads the grid from this page, so the two cannot drift. All 121 pairs have a row in `fixtures/rr/sa/manifest.tsv`. **39 are written and pass F4; the other 82 are refusals,** and a refusal's fixture is its row and its input in `fixtures/rr/inputs`. Of those, `N`, `C`, `CV` and `R` are verbs not built: conversion waits for Phase 10, render for Tier 2G, and a deck from a theme for a PowerPoint-written `.thmx` (`F-007`). They answer `CS-SAV-0013` saying which, and `.ppt` answers `CS-OPC-0005`. So the grid is defined and held, not implemented in every cell. `.ppam`, `.thmx` and `.odp` inputs are synthetic.

| Invariant | Status | Fixture | Evidence mark, for `PV-01` |
| --- | --- | --- | --- |
| `INV-PN-01` | Built | `RR-PN-01` | Documented |
| `INV-PN-02` | Built | `RR-PN-02` | Documented, Inferred |
| `INV-PN-03` | Built | `RR-PN-03` | Documented |
| `INV-PN-04` | Built | `RR-PN-04` | Documented |
| `INV-PN-05` | Built | `RR-PN-05` | Inferred |
| `INV-PN-06` | Built | `RR-PN-06` | Inferred, Unverified |
| `INV-PN-07` | Built | `RR-PN-07` | Documented, Inferred |
| `INV-PN-08` | Built | `RR-PN-08` | Documented |
| `INV-PN-09` | Built | `RR-PN-09` | Documented |
| `INV-PN-10` | Built | `RR-PN-10` | Documented |
| `INV-PN-11` | Built | `RR-PN-11` | Inferred |
| `INV-RI-01` | Built | `RR-RI-01` | Documented |
| `INV-RI-02` | Built | `RR-RI-02` | Inferred |
| `INV-RI-03` | Built | `RR-RI-03` | Inferred |
| `INV-RI-04` | Built | `RR-RI-04` | Documented, Inferred |
| `INV-RI-05` | Built | `RR-RI-05` | Inferred |
| `INV-RI-06` | Built | `RR-RI-06` | Documented |
| `INV-RI-07` | Built | `RR-RI-07` | Documented, Inferred |
| `INV-RI-08` | Built | `RR-RI-08` | Documented |
| `INV-RI-09` | Built | `RR-RI-09` | Documented, Inferred |
| `INV-RI-10` | Built | `RR-RI-10` | Inferred |
| `INV-CT-01` | Built | `RR-CT-01` | Documented |
| `INV-CT-02` | Built | `RR-CT-02` | Documented |
| `INV-CT-03` | Built | `RR-CT-03` | Documented |
| `INV-CT-04` | Built | `RR-CT-04` | Inferred |
| `INV-CT-05` | Built | `RR-CT-05` | Inferred |
| `INV-CT-06` | Built | `RR-CT-06` | Documented, Inferred |
| `INV-CT-07` | Built | `RR-CT-07` | Documented |
| `INV-CT-08` | Built | `RR-CT-08` | Inferred |
| `INV-CT-09` | Built | `RR-CT-09` | Inferred |
| `INV-CT-10` | Built | `RR-CT-10` | Unverified |
| `INV-SP-01` | Built | `RR-SP-01` | Inferred |
| `INV-SP-02` | Built | `RR-SP-02` | Documented, Inferred |
| `INV-SP-03` | Built | `RR-SP-03` | Inferred |
| `INV-SP-04` | Built | `RR-SP-04` | Inferred |
| `INV-SP-05` | Built | `RR-SP-05` | Inferred |
| `INV-SP-06` | Built | `RR-SP-06` | Documented, Inferred |
| `INV-SP-07` | Built | `RR-SP-07` | Documented, Inferred |
| `INV-SP-08` | Built | `RR-SP-08` | Unverified |
| `INV-SP-09` | Built | `RR-SP-09` | Inferred |
| `INV-SP-10` | Built | `RR-SP-10` | Inferred |
| `INV-ID-01` | Built | `RR-ID-01` | Documented, Inferred |
| `INV-ID-02` | Built | `RR-ID-02` | Inferred |
| `INV-ID-03` | Built | `RR-ID-03` | Documented, Inferred |
| `INV-ID-04` | Built | `RR-ID-04` | Inferred |
| `INV-ID-05` | Built | `RR-ID-05` | Documented |
| `INV-ID-06` | Built | `RR-ID-06` | Documented, Inferred, Unverified |
| `INV-ID-07` | Built | `RR-ID-07` | Documented |
| `INV-ID-08` | Built | `RR-ID-08` | Documented |
| `INV-ID-09` | Built | `RR-ID-09` | Documented |
| `INV-SL-01` | Built | `RR-SL-01` | Documented |
| `INV-SL-02` | Built | `RR-SL-02` | Documented |
| `INV-SL-03` | Built | `RR-SL-03` | Documented |
| `INV-SL-04` | Built | `RR-SL-04` | Documented, Inferred |
| `INV-SL-05` | Built | `RR-SL-05` | Inferred, Unverified |
| `INV-SL-06` | Built | `RR-SL-06` | Inferred |
| `INV-SL-07` | Built | `RR-SL-07` | Documented |
| `INV-NM-01` | Built | `RR-NM-01` | Documented |
| `INV-NM-02` | Built | `RR-NM-02` | Inferred |
| `INV-NM-03` | Built | `RR-NM-03` | Documented, Inferred, Unverified |
| `INV-NM-04` | Built | `RR-NM-04` | Documented |
| `INV-NM-05` | Built | `RR-NM-05` | Inferred |
| `INV-AP-01` | Built | `RR-AP-01` | Documented, Unverified |
| `INV-AP-02` | Built | `RR-AP-02` | Documented |
| `INV-AP-03` | Built | `RR-AP-03` | Inferred, Unverified |
| `INV-AP-04` | Built | `RR-AP-04` | Documented |
| `INV-AP-05` | Built | `RR-AP-05` | Inferred, Unverified |
| `INV-AP-06` | Built | `RR-AP-06` | Documented, Inferred |
| `INV-AP-07` | Built | `RR-AP-07` | Documented |
| `INV-AP-08` | Built | `RR-AP-08` | Documented, Inferred |
| `INV-AP-09` | Built, with host inputs | `RR-AP-09` | Documented, Inferred |
| `INV-AP-10` | Built | `RR-AP-10` | Documented |
| `INV-TH-01` | Built | `RR-TH-01` | Documented |
| `INV-TH-02` | Built | `RR-TH-02` | Inferred |
| `INV-TH-03` | Built, with no renderer to call it | `RR-TH-03` | Documented, Unverified |
| `INV-SG-01` | Built | `RR-SG-01` | Documented |
| `INV-SG-02` | Built | `RR-SG-02` | Documented, Inferred |
| `INV-SG-03` | Built | `RR-SG-03` | Documented, Inferred |
| `INV-SG-04` | Built | `RR-SG-04` | Inferred |
| `INV-SG-05` | Built | `RR-SG-05` | Inferred |
| `INV-SG-06` | Built, no seed: nothing dirty is the input bytes | `RR-SG-06` | Inferred |
| `INV-SG-07` | Built | `RR-SG-07` | Documented, Inferred |
| `INV-SG-08` | Built | `RR-SG-08` | Unverified |
| `INV-SA-01` | Built | `RR-SA-01` | Documented |
| `INV-SA-02` | Built | `RR-SA-02` | Documented, Inferred |
| `INV-SA-03` | Built | `RR-SA-03` | Documented, Unverified |
| `INV-SA-04` | Built | `RR-SA-04` | Documented, Inferred, Unverified |
| `INV-SA-05` | Built | `RR-SA-05` | Inferred, Unverified |
| `INV-SA-06` | Built | `RR-SA-06` | Inferred, Unverified |
| `INV-SA-07` | Built | `writer_saveas.rs` | Documented, Inferred |
| `INV-SA-08` | Built | `RR-SA-ppam-*` | Documented, Inferred |
| `INV-SA-09` | Refused until `F-007` has a `.thmx` | `RR-SA-thmx-*` | Unverified |
| `INV-SA-10` | Refused until Phase 10 | `RR-SA-ppt-*` | Inferred, Unverified |
| `INV-SA-11` | Refused until Phase 10 | `RR-SA-odp-*` | Documented, Inferred |
| `INV-SA-12` | Built | `RR-SA-pdf-*` | Inferred |
| `INV-SA-13` | Built: nothing here starts a show | `writer_saveas.rs` | Inferred |
| `INV-SA-14` | Built | `RR-SA-14` | Inferred |

**For `PV-01`:** open both decks of each of these, each marked Inferred or Unverified above: `INV-PN-02`, `INV-PN-05`, `INV-PN-06`, `INV-PN-07`, `INV-PN-11`, `INV-RI-02`, `INV-RI-03`, `INV-RI-04`, `INV-RI-05`, `INV-RI-07`, `INV-RI-09`, `INV-RI-10`, `INV-CT-04`, `INV-CT-05`, `INV-CT-06`, `INV-CT-08`, `INV-CT-09`, `INV-CT-10`, `INV-SP-01`, `INV-SP-02`, `INV-SP-03`, `INV-SP-04`, `INV-SP-05`, `INV-SP-06`, `INV-SP-07`, `INV-SP-08`, `INV-SP-09`, `INV-SP-10`, `INV-ID-01`, `INV-ID-02`, `INV-ID-03`, `INV-ID-04`, `INV-ID-06`, `INV-SL-04`, `INV-SL-05`, `INV-SL-06`, `INV-NM-02`, `INV-NM-03`, `INV-NM-05`. Also unsourced and in the code, so also to observe: the placeholder copies a new slide gets from its layout, `sldIdLst` left empty when the last slide goes, a duplicated slide keeping its `p14:creationId` and `a16:creationId`s, and a 3D model shared rather than copied by a clone.

## Proposed for other documents

This page owns only itself. The session merging it should carry these, since each is a finding from writing it.

**Index:** add this page to [00](00-README.md).

**Tracker rows for [14](14-EXECUTION-TRACKER.md):**

- A **diagram layout engine**: `INV-FB-01` regenerate has nothing to run, and neither does a SmartArt with no drawing. Without it, SmartArt editing stays blocked, whatever Phase 6 says.
- **`Package` compares names with `to_lowercase`**, in `find_part`, `rewrite` and `PartEdits`. OPC 6.2.2.3 asks for ASCII case-insensitive matching only, so `/É` and `/é` merge when they should not. Seen by reading `crates/casual-slide-package/src/package.rs`; no failing test yet.
- **`rewrite`'s fixed timestamp depends on features.** `SimpleFileOptions::default()` is fixed only because the crate's `time` feature is off; another crate enabling it would make output vary. Set it explicitly.
- **`F-007` needs samples for what this page could not read:** a `.thmx`, the slide-level ChartEx and 3D `AlternateContent`, PowerPoint's own notes master, and `app.xml` groups.
- **`PV-01` should run the seeded fixtures,** not only exports.

**New codes for [20](20-ERROR-CODE-REGISTRY.md), those not issued yet:** `0006` fallback regenerated; `0007` fallback kept stale; `0008` fallback dropped; `0010` notes master created; `0016` conversion loss. A finding for SmartArt with no drawing.

**[07](07-SECURITY-THREAT-MODEL.md), `SEC-014`:** the stale-thumbnail and stale-fallback rules here answer part of that row.

## Sources

Read on 2026-11-07. ECMA text is the 5th editions: Part 1 (2016), Part 2 (2021), Part 3 (2015). Clause numbers are the anchors. Fetched pages and files are data, and none was followed as instruction.

Could not be read: `bugs.documentfoundation.org` refuses automated access, so [`lo`][lo] rests on a search snippet. The [Microsoft page on Visual Basic file formats](https://support.microsoft.com/en-au/office/file-formats-that-work-with-visual-basic-69afddfd-4479-4466-9f37-3f4046b5e107) has been retired.

[opc]: https://ecma-international.org/wp-content/uploads/ECMA-376-2_5th_edition_december_2021.zip
[p1]: https://ecma-international.org/wp-content/uploads/ECMA-376-1_5th_edition_december_2016.zip
[mce]: https://ecma-international.org/wp-content/uploads/ECMA-376-3_5th_edition_december_2015.zip
[oi-naming]: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oi29500/50fbe853-8b61-4690-ae6c-cea5bbec5015
[oi-keywords]: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oi29500/6a50f8de-a9d1-4092-a0d4-671c9fa1e079
[oi-store]: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oi29500/291e6d2a-9ff9-4922-9a76-3771528eda9d
[oi-core]: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oi29500/28beaa8f-42ce-41e6-820d-4d7af34457a5
[oi-pres]: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oi29500/878e0961-6c04-4b57-9a6d-9bebc1ac3be3
[oi-slide]: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oi29500/709d622f-c25e-488c-820d-e04e3be537c6
[oi-layout]: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oi29500/0e96b521-bd75-493c-bec6-27b7784d46c7
[oi-master]: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oi29500/ce80d3db-3166-4d82-981a-77c01473a7f6
[oi-nm]: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oi29500/7722a188-d62f-4451-9216-16b75f29d22f
[oi-ns]: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oi29500/4241f073-467b-4245-a44f-947b96e88325
[oi-diag]: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oi29500/fbaa1e28-3676-408f-8e24-1c31101d4de7
[oi-presel]: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oi29500/5cc61456-aa4e-4c48-a115-3efed3e57d95
[oi-cs5]: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oi29500/05457486-4bf8-4e09-9d27-6f74fcf68562
[oi-cs6]: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oi29500/41eb94ad-9a8e-4f7f-b17e-ed9cc54b5fd7
[oi-appver]: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oi29500/6575023b-de01-441f-a9da-8cc9d3b73e99
[oi-heading]: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oi29500/7bbab466-4477-424b-9c52-cc9800547d7b
[oi-titles]: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oi29500/de32de14-9573-46f3-9f38-19659e3a8d9a
[oi-digsig]: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oi29500/360e6b06-a67f-44fa-a5bf-8ade7fbe7212
[pptx-sec]: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-pptx/1f21a089-944d-410b-bd47-4f5e692c2532
[pptx-seclist]: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-pptx/b499c39b-7c50-4a58-88e6-f86e6046be50
[pptx-section]: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-pptx/5bb610d5-ced8-44ed-a4b6-cd3d5f938906
[pptx-sldlist]: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-pptx/7bde1772-1da3-4311-b7f1-dcf5f7fe96c4
[pptx-entry]: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-pptx/73225424-0c3d-44a8-8908-eb9500460125
[pptx-secex]: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-pptx/5bd8a237-8753-4d05-b757-c5e84e5d1ba6
[pptx-media]: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-pptx/922b7818-6e5f-4641-a9c5-fab4063ec124
[ink]: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-pptx/0a7df81a-b34d-4846-8e1f-db55b34f696b
[zoom]: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-pptx/4873e445-37f4-48b8-aa79-9a0fbb5e2b67
[zoom-props]: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-pptx/059e3722-139d-4e41-9841-d53eecaf73f6
[diag]: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-odrawxml/c36c1152-3aec-45e5-8f16-936da2678e5d
[diag-part]: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-odrawxml/ec96c9e0-757a-4f24-9725-c52d3c34e310
[dme-block]: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-odrawxml/66cc6c59-45c6-438e-82ba-7d67de771f68
[cx-part]: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-odrawxml/5d0d453e-adac-43be-a797-59b9916593dd
[cx-chart]: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-odrawxml/820c131d-ab7c-42c1-8286-3373e44b6a76
[math]: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-odrawxml/853b19c7-68a9-4f9a-a2ae-5e6cb0d02e62
[pics]: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-odrawxml/7e1f1524-1569-4aa2-a6c9-aab2d855bd48
[odraw-pics3]: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-odrawxml/f10da8b6-40f3-4527-9c3d-9f5c2e9cff45
[odraw-legacy]: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-odrawxml/ad9da457-032f-44a7-b0dd-1e4cb920bf55
[ink-part]: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-odrawxml/096dacae-0d2c-4861-bc4d-c8e4c6405ad3
[m3d]: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-odrawxml/fd676695-83fa-4e92-8b65-4503ecbff9cc
[m3d-raster]: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-odrawxml/7483ab8d-6c9a-47f4-8f70-ca15693699aa
[sdk-pres]: https://github.com/dotnet/Open-XML-SDK/blob/main/src/DocumentFormat.OpenXml/Packaging/PresentationDocument.cs
[sdk-slide]: https://github.com/dotnet/Open-XML-SDK/blob/main/data/parts/SlidePart.json
[sdk-prespart]: https://github.com/dotnet/Open-XML-SDK/blob/main/data/parts/PresentationPart.json
[sdk-vba]: https://github.com/dotnet/Open-XML-SDK/blob/main/data/parts/VbaProjectPart.json
[sdk-m3d]: https://github.com/dotnet/Open-XML-SDK/blob/main/data/parts/Model3DReferenceRelationshipPart.json
[sdk-pml]: https://github.com/dotnet/Open-XML-SDK/blob/main/generated/DocumentFormat.OpenXml/DocumentFormat.OpenXml.Generator/DocumentFormat.OpenXml.Generator.OpenXmlGenerator/schemas_openxmlformats_org_presentationml_2006_main.g.cs
[ext]: https://learn.microsoft.com/en-us/previous-versions/office/office-2010/cc179191(v=office.14)
[formats]: https://support.microsoft.com/en-us/office/file-formats-that-are-supported-in-powerpoint-252c6fa0-a4bc-41be-ac82-b77c9773f9dc
[odp]: https://support.microsoft.com/office/94805e84-1b09-4c98-a8b5-0da2a52242a0
[sig]: https://support.microsoft.com/en-us/office/add-or-remove-a-digital-signature-for-microsoft-365-files-70d26dc9-be10-46f1-8efa-719c8b3f1a2d
[qa-sig]: https://learn.microsoft.com/en-us/answers/questions/57f96d64-ae02-4830-b01a-886ad29527ea/office-signatures-are-removed-not-invalidated-when?forum=msoffice-msoffice_word-mso_other-mso_o365b
[pronom]: https://pronom.nationalarchives.gov.uk/fmt/524
[php]: https://github.com/PHPOffice/PHPPresentation/issues/266
[py89]: https://github.com/scanny/python-pptx/issues/89
[py67]: https://github.com/scanny/python-pptx/issues/67
[py132]: https://github.com/scanny/python-pptx/issues/132
[pgx]: https://github.com/gitbrent/PptxGenJS/issues/1449
[qa-copy]: https://learn.microsoft.com/en-ie/answers/questions/539472/getting-a-repair-error-when-copy-slide-from-one-pr
[qa-cust]: https://learn.microsoft.com/en-gb/answers/questions/2281200/why-does-this-pptx-need-repaired
[lo]: https://bugs.documentfoundation.org/show_bug.cgi?id=155426
