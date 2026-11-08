# 96 — PowerPoint fit-gap: features

**Status: analysis, 2026-11-07. Task `FIT-01`.** **What works today is in [§Parity today](#parity-today), audited 2026-11-09 (`FIT-02`).** Owner instruction 12 in [03](03-OWNER-INSTRUCTIONS.md): the goal is a clone. One row per PowerPoint feature, on desktop and web, says what we do about it. Nothing here is built by being listed: no row claims anything works today, and the "Supported", "Retained" and "Matches PowerPoint" claims keep the tests set in [06](06-ROADMAP-AND-DELIVERY.md).

**What this page is not.** Commands, ribbon placement and keys are for docs/49, and detailed behaviour for docs/50, both being written separately. This page stops at whether a feature exists for us and why. It is also not the file-format inventory: [spec/](spec/README.md) holds 8730 rows on what a deck can contain, and a file construct with no user-facing verb appears here only when a decision hangs on it.

## How this was produced, and how far to trust it

Assembled on 2026-11-07 from Microsoft's public help: the topic index at [support.microsoft.com/powerpoint](https://support.microsoft.com/en-us/powerpoint), its six category pages, the platform comparison, the what's-new pages for 2019, 2021 and 2024, and targeted searches for each feature named in the task. Page text was treated as data.

**Nobody ran PowerPoint for this.** No row rests on a live session; that is `UX-001` and `PV-01`. Page reads went through a summarising fetch tool or search excerpts, so wording is close, not byte-checked. Before quoting a page in product copy, reopen it. Links were built from Microsoft's index and search results; Microsoft answers plain scripted requests with 403, so **they were not all re-resolved**, though every page fetched for a row, and two sampled from the listings, did resolve.

| Mark | Meaning |
| --- | --- |
| **Documented** | A Microsoft page was read and says so. Three rows rest on weaker sources, each named in its link: a third-party mirror of a Message Center notice (two rows), and a Microsoft Q&A answer (one). URL given |
| **Documented (listing)** | Only the title was seen in Microsoft's help index. The feature exists; its platforms and limits were **not** read |
| **Observed** | Read directly in this repository's vendored schemas or checklist. A file fact, never PowerPoint's behaviour |
| **Inferred** | No page found, or recalled. **Check before relying on it**; `UX-001` is the usual place |

| Mark | Rows |
| --- | ---: |
| Documented, page read | 252 |
| Documented, listing only | 41 |
| Observed | 10 |
| Inferred | 15 |

**The column "PowerPoint has it on"** records what the cited pages state: W Windows, M Mac, Web. `?` means not stated in what was read, and an absent letter proves nothing unless it says "not". Microsoft's pages disagree with each other and some are old, as [48](48-POWERPOINT-FEATURE-STUDY.md) showed, with one page still requiring Silverlight and Flash. **This column never limits us:** `DEC-06` sets the full desktop feature set in both hosts. Where a browser cannot do what a desktop can, the row says so.

**Phases** are [06](06-ROADMAP-AND-DELIVERY.md) as of `06d0be4` on `phase-0/foundation`, whose tiers include 1N, 2D1 to 2D6, 2H to 2K, 8A and 8B. The worktree this page was written in was cut before that commit, at `d8f627c`. `n/a` means no phase, since we do not build it.

## Dispositions

**Edit is the default.** Every other disposition states its reason in the row, and the generator that assembled this table failed on a non-Edit row with an empty reason.

| Disposition | Meaning |
| --- | --- |
| **Edit** | Modelled, rendered, editable, written back ([01](01-ORD.md)) |
| **Render** | Drawn and kept on save, not editable ([01](01-ORD.md)) |
| **Retain** | Kept verbatim and named in the compatibility report, not drawn or not acted on ([01](01-ORD.md)) |
| **Host seam** | The verb and UI are ours where they fit, but the work needs a service, a store, a licence or a tenant we do not ship. **Off, and silent on the network, by default** (`ENT-07`). A host may supply it |
| **Refuse** | We do not build it. Any data the feature stored in a deck still survives a save, so refusing a feature never loses a file's content (`ENT-21`) |

**Why a row is not Edit.** Almost every reason falls into one of these, and a reason outside them should make a reader stop and check:

1. **It runs foreign code** (macros, ActiveX, COM, `.ppam`, program actions). Security, and the owner's "never".
2. **It needs a service we do not ship** (Copilot, Designer, translate, speech, stock content). `DEC-05` and the air-gapped default.
3. **The tenant owns it** (sharing, labels, rights, signatures, version storage). `ENT-*` seams.
4. **Microsoft has retired it** (Compare, Slide Library, Present Live, cloud reuse). A clone of today's product need not bring it back.
5. **It is single-platform and has a cross-platform substitute** (XPS, WMF, WMV, MOV, web page export).
6. **The browser, the OS or the format decides it** (burning a disc, printer settings, window arranging, mixed orientation).
7. **It is only a stored file fact with no verb, or a fallback picture is all we can draw** (the Retain and Render rows).

## Count by area

| Area | Edit | Render | Retain | Host seam | Refuse | Total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Files and formats | 19 | 0 | 1 | 0 | 5 | 25 |
| Macros, code and embedded programs | 1 | 1 | 8 | 3 | 6 | 19 |
| Slides and views | 22 | 0 | 1 | 1 | 3 | 27 |
| Objects, shapes and arrangement | 17 | 0 | 0 | 0 | 0 | 17 |
| Text | 23 | 0 | 0 | 6 | 1 | 30 |
| Pictures and graphics | 19 | 0 | 1 | 4 | 0 | 24 |
| Tables, charts, SmartArt, equations, 3D | 16 | 1 | 0 | 3 | 0 | 20 |
| Ink and drawing | 10 | 0 | 0 | 2 | 0 | 12 |
| Transitions and animation | 16 | 0 | 0 | 0 | 0 | 16 |
| Audio and video | 13 | 0 | 0 | 3 | 0 | 16 |
| Slide show and recording | 25 | 0 | 0 | 6 | 4 | 35 |
| Design | 10 | 0 | 1 | 2 | 0 | 13 |
| Copilot (`DEC-05`) | 0 | 0 | 0 | 8 | 0 | 8 |
| Print and share | 7 | 0 | 0 | 3 | 1 | 11 |
| Collaboration and review | 7 | 0 | 1 | 1 | 1 | 10 |
| Protection, compliance, accessibility | 12 | 0 | 1 | 5 | 0 | 18 |
| Application shell | 7 | 0 | 0 | 0 | 1 | 8 |
| Mobile (`MOB-01`) | 6 | 0 | 0 | 2 | 1 | 9 |
| **Total** | **230** | **2** | **14** | **49** | **23** | **318** |

## The macro notice

Macros are **never run**, in any host, at any setting: owner, in [01](01-ORD.md), and `SEC-C07`. The notice:

> **This presentation contains macros. OpenPresent keeps them in the file, but never runs them.** Buttons or actions that depend on a macro will do nothing here.

- **When:** on opening any deck with a VBA project, whatever its extension, not only `.pptm`, `.ppsm` and `.potm`. It is a persistent bar in the editor and a dismissible one in Viewer, and it does not block reading or editing.
- **Not a prompt.** There is no "Enable" button, since nothing can be enabled.
- **In a show:** clicking a macro or program action does nothing to the deck and shows one line: "This action runs a macro or program, which OpenPresent never does." `SEC-N04`.
- **Text:** the same on web and desktop. `ENT-22` lets an administrator add to it, not remove it. The wording above is ours, as a proposal; `UX-001` should see what PowerPoint shows and docs/49 and 50 place it.
- **On save:** the project is kept verbatim. Saving to `.pptx` drops it, and the save report names that (`WR-05`). Document Inspector reports macros without removing them, which is also what Microsoft's does.
- **Why this is not "Retain, so nothing to do":** a deck whose whole purpose is a macro will look broken, and the notice is how the user learns why rather than blaming the product.

## The rows

### Files and formats

| ID | Feature | PowerPoint has it on | Disposition | Phase or tier | Reason or note | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| PP-FIT-001 | Open and save `.pptx` | W M Web | Edit | 1A to 1N |  | Documented: [File formats](https://support.microsoft.com/en-us/powerpoint/file-formats-that-are-supported-in-powerpoint), [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint) |
| PP-FIT-002 | Strict Open XML `.pptx` | W | Edit | 1N | `STRICT-01`: normalise on read, write back in source form. | Documented: [File formats](https://support.microsoft.com/en-us/powerpoint/file-formats-that-are-supported-in-powerpoint) |
| PP-FIT-003 | PowerPoint Show `.ppsx` | W M | Edit | 1N, 8A | Must not auto-start without a user gesture (`SEC-014`). | Documented: [File formats](https://support.microsoft.com/en-us/powerpoint/file-formats-that-are-supported-in-powerpoint) |
| PP-FIT-004 | Template `.potx`: open, make a deck, save as | W M; view only on Web | Edit | 1N, 7 |  | Documented: [File formats](https://support.microsoft.com/en-us/powerpoint/file-formats-that-are-supported-in-powerpoint), [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint), [Create a template](https://support.microsoft.com/en-us/PowerPoint/create-and-save-a-powerpoint-template) |
| PP-FIT-005 | Macro-enabled files `.pptm`, `.ppsm`, `.potm`: open and edit the deck | W M; view only on Web | Edit | 1L, 1N, 3 | Slides are editable. The VBA project is the next section's Retain row, with the notice. | Documented: [File formats](https://support.microsoft.com/en-us/powerpoint/file-formats-that-are-supported-in-powerpoint), [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint) |
| PP-FIT-006 | PDF export | W M Web | Edit | 2G | Tagged (`PDF-01`). | Documented: [File formats](https://support.microsoft.com/en-us/powerpoint/file-formats-that-are-supported-in-powerpoint), [Save as PDF](https://support.microsoft.com/en-us/PowerPoint/training/save-powerpoint-presentations-as-pdf-files), [New in 2024](https://support.microsoft.com/en-us/office/what-s-new-in-powerpoint-2024-for-windows-and-mac-4330d63f-ffbb-470d-bbc2-a5f340317fbc) |
| PP-FIT-007 | XPS export | W | Refuse | 1N (`WR-05`) | Windows-only on Microsoft's own table, and PDF covers the need. Recommendation to `WR-05`, which owns the save-as matrix. | Documented: [File formats](https://support.microsoft.com/en-us/powerpoint/file-formats-that-are-supported-in-powerpoint) |
| PP-FIT-008 | Office Theme `.thmx`: save and apply | W | Edit | 1N, 7 |  | Documented: [File formats](https://support.microsoft.com/en-us/powerpoint/file-formats-that-are-supported-in-powerpoint), [Create a theme](https://support.microsoft.com/en-us/PowerPoint/create-your-own-theme-in-powerpoint) |
| PP-FIT-009 | PowerPoint XML Presentation `.xml` | W M | Edit | 1N | A new reader for a flat form: parser rules and a fuzz target apply. Storage form not confirmed from a sample. | Documented: [File formats](https://support.microsoft.com/en-us/powerpoint/file-formats-that-are-supported-in-powerpoint) |
| PP-FIT-010 | Slide as PNG, JPEG, GIF, TIFF, BMP | W M; JPEG on Web | Edit | 2G |  | Documented: [File formats](https://support.microsoft.com/en-us/powerpoint/file-formats-that-are-supported-in-powerpoint), [Slide as image](https://support.microsoft.com/en-us/office/save-a-slide-as-an-image-or-as-a-separate-presentation-file-58a32e76-2026-431c-9d2b-e8312e49dae7), [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint) |
| PP-FIT-011 | Slide as WMF or EMF | W | Refuse | 2G | Windows-only on Microsoft's table, and a metafile writer serves only this row. We still read and draw EMF (2E). | Documented: [File formats](https://support.microsoft.com/en-us/powerpoint/file-formats-that-are-supported-in-powerpoint) |
| PP-FIT-012 | Picture Presentation (each slide a picture) | W | Edit | 2G, 1N |  | Documented: [File formats](https://support.microsoft.com/en-us/powerpoint/file-formats-that-are-supported-in-powerpoint) |
| PP-FIT-013 | Outline as RTF: save | W M | Edit | 5, 1N |  | Documented: [File formats](https://support.microsoft.com/en-us/powerpoint/file-formats-that-are-supported-in-powerpoint) |
| PP-FIT-014 | OpenDocument `.odp`: open and save | W M Web | Edit | 10 | Microsoft's ODP table lists losses; ours are reported the same way. | Documented: [File formats](https://support.microsoft.com/en-us/powerpoint/file-formats-that-are-supported-in-powerpoint), [ODP](https://support.microsoft.com/en-us/powerpoint/use-powerpoint-to-save-or-open-a-presentation-in-the-opendocument-presentation-odp-format), [New in 2024](https://support.microsoft.com/en-us/office/what-s-new-in-powerpoint-2024-for-windows-and-mac-4330d63f-ffbb-470d-bbc2-a5f340317fbc) |
| PP-FIT-015 | Web Page `.htm`, Single File Web Page `.mht`: save | W | Refuse | 1N | Windows-only on Microsoft's table, and output needs its own viewer and script surface (`SEC-E01`). `p:htmlPubPr` and `p:webPr` stay as Retain (below). PDF and video cover sharing. | Documented: [File formats](https://support.microsoft.com/en-us/powerpoint/file-formats-that-are-supported-in-powerpoint) |
| PP-FIT-016 | Web publish settings in the file | ? | Retain | 1B | No file output uses them under the row above, so they are kept verbatim only. | Observed: `p:CT_PresentationProperties/htmlPubPr`, `p:CT_WebProperties` in [checklist](spec/checklist.tsv) |
| PP-FIT-017 | Legacy `.ppt`, `.pps`: open | W; Web converts to edit | Edit | 10 (`PPT-01`) | Import, then edited and saved as `.pptx` (`DEC-08`). | Documented: [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint) |
| PP-FIT-018 | Legacy `.ppt`: save | ? | Refuse | 10 | `DEC-08` is import-only. Saving the binary form would be a second writer for no user need. | Inferred: no save row on the formats page for `.ppt`; `DEC-08` decides regardless |
| PP-FIT-019 | Pack and Go `.ppz` | not supported | Refuse | n/a | Microsoft itself does not open these, so a clone need not. | Documented: [File formats](https://support.microsoft.com/en-us/powerpoint/file-formats-that-are-supported-in-powerpoint) |
| PP-FIT-020 | Compatibility Checker | W M | Edit | 10 | Our rule list reports what older PowerPoint would lose; depends on `PV-01` for proof. | Documented: [Compatibility Checker](https://support.microsoft.com/en-us/powerpoint/compatibility-checker-in-powerpoint) |
| PP-FIT-021 | Compatibility mode for old files | W | Edit | 1B, 10 | Banner and convert. Rules not read in detail. | Documented (listing): [Compatibility mode](https://support.microsoft.com/en-us/powerpoint/use-compatibility-mode-to-work-with-different-versions-of-powerpoint) |
| PP-FIT-022 | Document properties (File, Info) | W M | Edit | 1A, 3 |  | Inferred: `docProps` are in [parts](spec/parts.md); the Info page is not separately documented in what was read; confirm in `UX-001` |
| PP-FIT-023 | AutoSave | W M Web; cloud files only | Edit | 4 (`SAVE-01`) | Local desktop files need our own journal, where PowerPoint offers AutoSave only for cloud files. | Documented: [AutoSave](https://support.microsoft.com/en-us/office/collab-files/what-is-autosave), [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint) |
| PP-FIT-024 | AutoRecover | W M | Edit | 4 (`SAVE-01`) |  | Documented: [Recover files](https://support.microsoft.com/en-us/powerpoint/recover-your-powerpoint-files), [Reduce file size](https://support.microsoft.com/en-us/powerpoint/reduce-the-file-size-of-your-powerpoint-presentations) |
| PP-FIT-025 | Open up to 2 GB | Web | Edit | 1A, `CAP-01` | The figure is Microsoft's; ours is set by `CAP-01` and stated, not assumed equal. | Documented: [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint) |

### Macros, code and embedded programs

| ID | Feature | PowerPoint has it on | Disposition | Phase or tier | Reason or note | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| PP-FIT-026 | VBA project: keep in the file | W M; Web views | Retain | 1L, 3 | Bytes kept (`SEC-C07`). Never parsed for execution. Notice below on every open. | Documented: [File formats](https://support.microsoft.com/en-us/powerpoint/file-formats-that-are-supported-in-powerpoint), [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint), [Run a macro](https://support.microsoft.com/en-us/office/run-a-macro-in-powerpoint-fa2ecee4-9985-490a-9d99-74b3c726bc5a) |
| PP-FIT-027 | Run a macro | W M; not on Web | Refuse | n/a | **Never** (owner, `01-ORD`). Our notice says so. Microsoft's own page warns of the risk; the web product does not run them either. | Documented: [Run a macro](https://support.microsoft.com/en-us/office/run-a-macro-in-powerpoint-fa2ecee4-9985-490a-9d99-74b3c726bc5a), [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint), [Enable or disable macros](https://support.microsoft.com/en-us/office/enable-or-disable-macros-in-microsoft-365-files-12b036fd-d140-4e74-b45e-16fed1a7e5c6) |
| PP-FIT-028 | Trust Center macro settings | W M | Refuse | n/a | Nothing runs, so there is nothing to enable. `ENT-22` holds only the notice text. | Documented (listing): [Enable or disable macros](https://support.microsoft.com/en-us/office/enable-or-disable-macros-in-microsoft-365-files-12b036fd-d140-4e74-b45e-16fed1a7e5c6) |
| PP-FIT-029 | Developer tab, VBA editor, record macro | W; M differs | Refuse | n/a | An editor for code we refuse to run would mislead. A verb we lack stays off the ribbon (47). | Documented: [Run a macro](https://support.microsoft.com/en-us/office/run-a-macro-in-powerpoint-fa2ecee4-9985-490a-9d99-74b3c726bc5a) |
| PP-FIT-030 | Action: Run macro | W M | Retain | 1D, 8A | Stored action kept; clicking it in a show does nothing but say why (`SEC-N04`). | Documented: [Action buttons](https://support.microsoft.com/en-us/powerpoint/add-commands-to-your-presentation-with-action-buttons) |
| PP-FIT-031 | Action: Run program | W M | Retain | 1D, 8A | Stored action kept, never launched (`SEC-N04`). | Documented: [Action buttons](https://support.microsoft.com/en-us/powerpoint/add-commands-to-your-presentation-with-action-buttons) |
| PP-FIT-032 | ActiveX controls in a slide | W; Web views | Retain | 1L, 2I | Never executed (`SEC-C07`). The fallback image is drawn and the control's bytes kept. | Documented: [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint), [Presentation into a video](https://support.microsoft.com/en-us/powerpoint/turn-your-presentation-into-a-video) |
| PP-FIT-033 | PowerPoint add-in `.ppam`: load and run | W M; Web cannot open | Refuse | n/a | It is VBA by another name: the row above applies. | Documented: [File formats](https://support.microsoft.com/en-us/powerpoint/file-formats-that-are-supported-in-powerpoint), [Load a PowerPoint add-in](https://support.microsoft.com/en-us/powerpoint/add-or-load-a-powerpoint-add-in), [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint) |
| PP-FIT-034 | COM add-ins | W | Refuse | n/a | Native Windows code in-process, against the threat model and impossible on the web. | Inferred: from the Options page's add-in options and general knowledge; no COM page was read |
| PP-FIT-035 | Office Add-ins (store, My Add-ins, task pane) | W M; Web not stated | Host seam | 10 (`ENT-16`) | Runs third-party web code and needs a store. The host may offer add-ins through the SDK; none load by default, and none in air-gapped mode. Office.js compatibility is not promised. | Documented: [Install add-ins](https://support.microsoft.com/en-us/office/add-ins/view-manage-and-install-add-ins-for-excel-powerpoint-and-word) |
| PP-FIT-036 | Add-in parts stored in a deck | ? | Retain | 1M | Kept verbatim, not started (`SEC-014`). The checklist has no `webextension` row: `SPEC-04` must list the parts. | Observed: 0 `webextension` rows in [checklist](spec/checklist.tsv) |
| PP-FIT-037 | Embedded OLE object: show | W; Web views | Render | 1L, 2I | Stored fallback picture is drawn, payload kept. Full activation is the next row. | Documented: [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint), [Linked and embedded objects](https://support.microsoft.com/en-us/office/linked-objects-and-embedded-objects-0bf81db2-8aa3-4148-be4a-c8b6e55e0d7c) |
| PP-FIT-038 | Embedded OLE object: double-click to edit | W M | Refuse | n/a | Needs the owning program on the machine and runs it on untrusted data (`SEC-C07`). The next row covers the sister apps. | Documented: [Linked and embedded objects](https://support.microsoft.com/en-us/office/linked-objects-and-embedded-objects-0bf81db2-8aa3-4148-be4a-c8b6e55e0d7c) |
| PP-FIT-039 | Embedded Excel or Word object: edit | W M | Host seam | 10 | Through the OpenCalc SDK (`CROSS-01`), or the host's sibling app. Not an OLE server. | Documented: [Linked and embedded objects](https://support.microsoft.com/en-us/office/linked-objects-and-embedded-objects-0bf81db2-8aa3-4148-be4a-c8b6e55e0d7c), [Excel data in PowerPoint](https://support.microsoft.com/en-us/powerpoint/insert-and-update-excel-data-in-powerpoint) |
| PP-FIT-040 | Insert Object, from file, embed | W M | Edit | 6, 1L | Stored with an icon; opened only by saving it out, after confirmation. Writing a real OLE container is a spike. | Documented: [Excel data in PowerPoint](https://support.microsoft.com/en-us/powerpoint/insert-and-update-excel-data-in-powerpoint), [Linked and embedded objects](https://support.microsoft.com/en-us/office/linked-objects-and-embedded-objects-0bf81db2-8aa3-4148-be4a-c8b6e55e0d7c) |
| PP-FIT-041 | Insert Object, from file, link; Edit Links | W M | Host seam | 10 | A browser has no file path, and `file:` is refused (`SEC-N03`). Desktop or a host can resolve links. Retained until then. | Documented: [Linked and embedded objects](https://support.microsoft.com/en-us/office/linked-objects-and-embedded-objects-0bf81db2-8aa3-4148-be4a-c8b6e55e0d7c), [Excel data in PowerPoint](https://support.microsoft.com/en-us/powerpoint/insert-and-update-excel-data-in-powerpoint) |
| PP-FIT-042 | Smart tags | ? | Retain | 1M | Old format element with no UI found on any page read. | Observed: `p:CT_SmartTags` in [checklist](spec/checklist.tsv) |
| PP-FIT-043 | Tags, customer data, custom XML | ? | Retain | 1M | Spec area `CUSTOM` is Retain: no user-facing verb exists except Inspector removal. | Observed: `CUSTOM` in [spec/README](spec/README.md) |
| PP-FIT-044 | Script links | ? | Retain | 1M | Never run. Meaning not researched. | Observed: `x:drawing/2021/scriptlink` in [checklist](spec/checklist.tsv) |

### Slides and views

| ID | Feature | PowerPoint has it on | Disposition | Phase or tier | Reason or note | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| PP-FIT-045 | Add, rearrange, duplicate, delete slides | W M Web | Edit | 4 |  | Documented: [Compare features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b), [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint) |
| PP-FIT-046 | Layouts: choose, reset | Web, W, M | Edit | 4 |  | Documented: [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint) |
| PP-FIT-047 | Slide size and orientation | W M Web | Edit | 7 |  | Documented: [Compare features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b) |
| PP-FIT-048 | Mixed orientation in one deck | not possible | Refuse | n/a | PowerPoint does not allow it, per Microsoft, and the format has one size per deck. Same workaround: action links between two decks. | Documented: [Mixed orientation](https://support.microsoft.com/en-us/PowerPoint/can-i-use-portrait-and-landscape-slide-orientation-in-the-same-presentation) |
| PP-FIT-049 | Sections | ? | Edit | 1M, 4 |  | Documented (listing): [Sections](https://support.microsoft.com/en-us/PowerPoint/training/organize-your-powerpoint-slides-into-sections) |
| PP-FIT-050 | Hide slide; reveal in show | W M Web | Edit | 4, 8A |  | Documented: [Hide a slide](https://support.microsoft.com/en-us/office/hide-or-show-a-slide-8313e1ec-3e20-4464-952f-387931554d69), [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint) |
| PP-FIT-051 | Reuse slides, keep source formatting | W M; local files only | Edit | 4 | Microsoft removed cloud reuse, not the verb, in early 2026. A third-party blog overstates this as total removal; the MC text says local files stay. | Documented: [Reuse slides](https://support.microsoft.com/en-us/PowerPoint/reuse-import-slides-from-another-presentation), [MC1179161, third-party mirror](https://mc.merill.net/message/MC1179161) |
| PP-FIT-052 | Reuse slides from cloud storage | retired | Host seam | 10 | Microsoft retired it; a clone need not lead. The host's file picker can feed the same verb (`ENT-16`), which costs almost nothing. | Documented: [Reuse slides](https://support.microsoft.com/en-us/PowerPoint/reuse-import-slides-from-another-presentation), [MC1179161, third-party mirror](https://mc.merill.net/message/MC1179161) |
| PP-FIT-053 | Slide update link metadata | ? | Retain | 1D | Trace of a shared slide's source. No current page; kept verbatim, and the Slide Library row explains why. | Observed: `p:<sldSyncPr>` in [checklist](spec/checklist.tsv) |
| PP-FIT-054 | Slide Library: publish, reuse from library | retired with SharePoint | Refuse | n/a | Microsoft's Q&A says it is gone from SharePoint Online. `ENT-17`'s organisation library is the modern equivalent. Evidence weaker: the support article's URL now serves a sharing page, and the Q&A is not a support page. | Documented: [Slide Library in SharePoint Online, Microsoft Q&A](https://learn.microsoft.com/en-us/answers/questions/5056527/sharepoint-online-slide-library) |
| PP-FIT-055 | Slides from Outline (`.docx`, `.rtf`, `.txt`) | W M (RTF only on Mac); not Web | Edit | 5 | `.txt` and `.rtf` first. `.docx` needs a new hostile-input parser or OpenDoc's. Heading 1 and 2 map to title and bullet as Microsoft states. | Documented: [From an outline](https://support.microsoft.com/en-us/office/create-a-powerpoint-presentation-from-an-outline-f6294909-04e9-4020-b9a8-4587b112692c), [Import a Word outline](https://support.microsoft.com/en-gb/office/import-a-word-outline-into-powerpoint-a25f6e01-9a19-4c0d-a108-7f533e42dfe9), [Import a Word document](https://support.microsoft.com/en-us/powerpoint/import-a-word-document-into-a-powerpoint-presentation) |
| PP-FIT-056 | Normal view | W M Web | Edit | 3 |  | Documented: [Choose the right view](https://support.microsoft.com/en-au/office/choose-the-right-view-for-the-task-in-powerpoint-21332d8d-adbc-4717-a2c6-e25a697b40e9), [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint) |
| PP-FIT-057 | Outline view | W M; not Web | Edit | 5 | Phase 3's charter lists no outline. Tab and Shift+Tab change level. **Added to 5**, see roadmap asks. | Documented: [Outline view](https://support.microsoft.com/en-us/office/create-and-print-a-presentation-in-outline-view-3516310c-c9c0-4d4f-8c11-2759313477a5), [Choose the right view](https://support.microsoft.com/en-au/office/choose-the-right-view-for-the-task-in-powerpoint-21332d8d-adbc-4717-a2c6-e25a697b40e9), [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint) |
| PP-FIT-058 | Slide Sorter view | W M Web | Edit | 3 |  | Documented: [Choose the right view](https://support.microsoft.com/en-au/office/choose-the-right-view-for-the-task-in-powerpoint-21332d8d-adbc-4717-a2c6-e25a697b40e9), [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint) |
| PP-FIT-059 | Notes Page view | W M | Edit | 4, 2A | Editable notes-page layout, not only the pane. Phase 3 lists a notes pane only. See roadmap asks. | Documented: [Choose the right view](https://support.microsoft.com/en-au/office/choose-the-right-view-for-the-task-in-powerpoint-21332d8d-adbc-4717-a2c6-e25a697b40e9) |
| PP-FIT-060 | Reading view | W M Web | Edit | 3 |  | Documented: [Choose the right view](https://support.microsoft.com/en-au/office/choose-the-right-view-for-the-task-in-powerpoint-21332d8d-adbc-4717-a2c6-e25a697b40e9), [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint) |
| PP-FIT-061 | Slide Master, Handout Master, Notes Master views | W M; not Web | Edit | 7 | `DEC-06`. | Documented: [Choose the right view](https://support.microsoft.com/en-au/office/choose-the-right-view-for-the-task-in-powerpoint-21332d8d-adbc-4717-a2c6-e25a697b40e9), [Compare features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b), [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint) |
| PP-FIT-062 | Grayscale and Pure Black and White view | W (old versions); current not stated | Edit | 7, 2G | Per-object setting (`bwMode`, in the format) drives both view and print. Microsoft documents the view only in an article for 2002 to 2007; third-party guides place it on today's View tab. Confirm in `UX-001`. | Documented: [Grayscale view, article for 2002 to 2007](https://support.microsoft.com/en-au/topic/black-and-white-or-grayscale-slides-are-displayed-in-color-when-you-send-them-to-word-a88c82f3-a9c8-7eac-4fc6-a5cc12f52066), [Print handouts, notes or slides](https://support.microsoft.com/en-us/office/print-your-handouts-notes-or-slides-91c62c83-9032-497c-ab76-cae8f3e1a402) |
| PP-FIT-063 | Speaker notes pane | W M Web | Edit | 3, 4 |  | Documented: [Compare features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b), [Speaker notes](https://support.microsoft.com/en-us/PowerPoint/training/add-speaker-notes-to-your-slides) |
| PP-FIT-064 | Ruler | W M; not Web | Edit | 5 |  | Documented: [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint) |
| PP-FIT-065 | Gridlines, snap to grid | W M; not Web | Edit | 4 |  | Documented: [Gridlines](https://support.microsoft.com/en-us/powerpoint/work-with-gridlines-and-use-snap-to-grid-in-powerpoint), [Snap](https://support.microsoft.com/en-au/office/adjust-snap-strength-or-turn-snap-off-94dadb54-f6b1-45f5-9795-c3ec2a1f8671), [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint), [New in 2024](https://support.microsoft.com/en-us/office/what-s-new-in-powerpoint-2024-for-windows-and-mac-4330d63f-ffbb-470d-bbc2-a5f340317fbc) |
| PP-FIT-066 | Drawing guides | W M | Edit | 4 |  | Documented: [Guides](https://support.microsoft.com/en-us/office/guides-for-arranging-things-on-a-slide-33854dfa-e0d1-43ff-8971-667b19512de3) |
| PP-FIT-067 | Smart guides | W M Web | Edit | 4 |  | Documented: [Guides](https://support.microsoft.com/en-us/office/guides-for-arranging-things-on-a-slide-33854dfa-e0d1-43ff-8971-667b19512de3), [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint) |
| PP-FIT-068 | Zoom, Fit | W M Web | Edit | 3 |  | Documented: [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint) |
| PP-FIT-069 | New Window, Arrange All, Cascade | W | Refuse | n/a | Browser tabs and OS windows do this. No Microsoft page read names it, so `UX-001` should confirm before the row is closed. | Inferred: recall of the desktop View tab; not on any page read |
| PP-FIT-070 | Open in a chosen view | W | Edit | 3 | Option named on the Advanced page. | Documented: [Options: Advanced](https://support.microsoft.com/en-us/powerpoint/powerpoint-options-advanced) |
| PP-FIT-071 | Saved view state in the file | W M | Edit | 1B |  | Observed: `VIEW` in [spec/README](spec/README.md) |

### Objects, shapes and arrangement

| ID | Feature | PowerPoint has it on | Disposition | Phase or tier | Reason or note | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| PP-FIT-072 | Shapes gallery (187 presets) | W M Web | Edit | 1E, 2B, 4 | `GEO-01` for the missing `upArrow`. | Documented: [Compare features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b), [Add shapes](https://support.microsoft.com/en-us/powerpoint/add-shapes) |
| PP-FIT-073 | Text box | W M Web | Edit | 4, 5 |  | Documented: [Compare features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b) |
| PP-FIT-074 | Freeform, Scribble, Edit Points | W M | Edit | 4 |  | Documented: [Freeform and edit points](https://support.microsoft.com/en-us/office/graphics-visuals/draw-or-edit-a-freeform-shape) |
| PP-FIT-075 | Merge Shapes: Union, Combine, Fragment, Intersect, Subtract | W M; Web not stated | Edit | 4, 1E | Needs a path boolean engine and no such tier exists: see roadmap asks. Selection order changes the result, as Microsoft says. | Documented: [Merge shapes](https://support.microsoft.com/en-us/PowerPoint/draw-a-picture-by-combining-and-merging-shapes) |
| PP-FIT-076 | Connectors, glued to sites | ? | Edit | 4 | Named in 47. No Microsoft page came back in two searches. | Inferred: 47's canvas section, itself written from recall; spec `SHAPE` has connection sites |
| PP-FIT-077 | Align, distribute | W M | Edit | 4 |  | Documented: [Align or arrange](https://support.microsoft.com/en-us/office/graphics-visuals/align-or-arrange-objects), [Guides](https://support.microsoft.com/en-us/office/guides-for-arranging-things-on-a-slide-33854dfa-e0d1-43ff-8971-667b19512de3) |
| PP-FIT-078 | Group, ungroup | W M | Edit | 4 |  | Documented: [Group](https://support.microsoft.com/en-us/office/graphics-visuals/group-or-ungroup-shapes-pictures-or-other-objects), [Keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/powerpoint/use-keyboard-shortcuts-to-create-powerpoint-presentations) |
| PP-FIT-079 | Rotate, flip | W M | Edit | 4 |  | Documented (listing): [Rotate](https://support.microsoft.com/en-us/topic/399e7a92-87e9-4d86-a03a-be120056fe3b) |
| PP-FIT-080 | Select individual objects | W M | Edit | 4 |  | Documented (listing): [Select objects](https://support.microsoft.com/en-us/PowerPoint/select-individual-objects-on-a-slide) |
| PP-FIT-081 | Selection pane | W M | Edit | 4 |  | Documented: [Selection pane](https://support.microsoft.com/en-us/powerpoint/use-the-selection-pane-to-manage-objects-in-documents) |
| PP-FIT-082 | Format Painter | W M; Web conflicting | Edit | 4 | Web page says both yes and no. Not a blocker: `DEC-06`. | Documented: [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint), [Keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/powerpoint/use-keyboard-shortcuts-to-create-powerpoint-presentations) |
| PP-FIT-083 | Eyedropper | W M; not Web | Edit | 4 | Inside the slide everywhere. Outside the window, Microsoft documents it on Mac. In a browser only where the platform offers a picker (Inferred). | Documented: [Eyedropper](https://support.microsoft.com/en-us/powerpoint/use-eyedropper-to-match-colors-on-your-slide) |
| PP-FIT-084 | Colours dialog, hex input | W | Edit | 4 |  | Documented: [New in 2021](https://support.microsoft.com/en-gb/office/what-s-new-in-powerpoint-2021-for-windows-dac3eb1b-483b-4043-a0c9-5c01c2c1a8f2) |
| PP-FIT-085 | Fills, outlines, effects | W M; part on Web | Edit | 1E, 2B, 2C, 4 |  | Documented: [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint) |
| PP-FIT-086 | Sketched outline style | W | Edit | 4 |  | Documented: [New in 2021](https://support.microsoft.com/en-gb/office/what-s-new-in-powerpoint-2021-for-windows-dac3eb1b-483b-4043-a0c9-5c01c2c1a8f2) |
| PP-FIT-087 | Draw ruler | W | Edit | 6 |  | Documented: [New in 2021](https://support.microsoft.com/en-gb/office/what-s-new-in-powerpoint-2021-for-windows-dac3eb1b-483b-4043-a0c9-5c01c2c1a8f2), [New in 2019](https://support.microsoft.com/en-us/office/what-s-new-in-powerpoint-2019-for-windows-8355a56a-f643-42d2-8454-784fa9b3d109), [Draw ruler](https://support.microsoft.com/en-us/powerpoint/draw-straight-lines-or-align-things-with-the-ruler-in-powerpoint) |
| PP-FIT-088 | Maximum number of undos | W; Web unbounded in session | Edit | 4 |  | Documented: [Options: Advanced](https://support.microsoft.com/en-us/powerpoint/powerpoint-options-advanced), [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint) |

### Text

| ID | Feature | PowerPoint has it on | Disposition | Phase or tier | Reason or note | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| PP-FIT-089 | Font, size, bold, italic and the rest | W M Web | Edit | 1F, 2D1, 5 |  | Documented: [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint) |
| PP-FIT-090 | Text colour | W M Web | Edit | 5 |  | Documented (listing): [Text colour](https://support.microsoft.com/en-us/PowerPoint/change-the-color-of-text-on-a-powerpoint-slide) |
| PP-FIT-091 | Text highlighter | W; Sub on Web and Mac | Edit | 5 |  | Documented: [Compare features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b), [Highlight](https://support.microsoft.com/en-us/PowerPoint/highlight-your-text), [New in 2019](https://support.microsoft.com/en-us/office/what-s-new-in-powerpoint-2019-for-windows-8355a56a-f643-42d2-8454-784fa9b3d109) |
| PP-FIT-092 | Superscript, subscript | W M Web | Edit | 5 |  | Documented: [Compare features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b), [Superscript](https://support.microsoft.com/en-us/PowerPoint/format-text-as-superscript-or-subscript) |
| PP-FIT-093 | Clear formatting | W M | Edit | 5 |  | Documented (listing): [Clear formatting](https://support.microsoft.com/en-us/PowerPoint/clear-all-text-formatting) |
| PP-FIT-094 | Alignment, spacing, indents | W M Web | Edit | 5 |  | Documented: [Alignment, indent, spacing](https://support.microsoft.com/en-us/office/change-text-alignment-indentation-and-spacing-in-powerpoint-36df6486-0118-4e95-b9bd-835eb047ac88), [Hanging indent](https://support.microsoft.com/en-us/powerpoint/create-or-remove-a-hanging-indent) |
| PP-FIT-095 | Bullets, numbering | W M Web | Edit | 5, 2D3 |  | Documented: [Compare features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b), [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint) |
| PP-FIT-096 | Text columns | W M | Edit | 5, 2D4 |  | Documented: [Format Object pane](https://support.microsoft.com/en-gb/office/graphics-visuals/format-object-text-box-pane), [ODP](https://support.microsoft.com/en-us/powerpoint/use-powerpoint-to-save-or-open-a-presentation-in-the-opendocument-presentation-odp-format) |
| PP-FIT-097 | Right-to-left text and direction | W M; Web conflicting | Edit | 2D2, 5 | `ENT-14`. Conflict recorded in 48. | Documented: [Compare features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b) |
| PP-FIT-098 | Vertical and rotated text | W M | Edit | 2D4, 5 |  | Documented: [Format Object pane](https://support.microsoft.com/en-gb/office/graphics-visuals/format-object-text-box-pane) |
| PP-FIT-099 | AutoFit, shrink on overflow, resize shape | W M | Edit | 2D6, 5 | Placeholder options differ from text box options, per Microsoft. | Documented: [Fit text in a shape](https://support.microsoft.com/en-us/office/fit-text-in-a-shape-185febad-c7ce-4ff8-ab39-abf5fad7ab37) |
| PP-FIT-100 | Split body text across slides | W | Edit | 5 | AutoFit option. | Documented (listing): [Split body text](https://support.microsoft.com/en-us/powerpoint/split-body-text-between-two-slides) |
| PP-FIT-101 | WordArt, text warps | W M; Web not insert | Edit | 2D6, 5 |  | Documented: [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint), [Shape, text box and WordArt keys](https://support.microsoft.com/en-us/office/keyboard-shortcuts-for-working-with-shapes-text-boxes-and-wordart), [Format Text Effects pane](https://support.microsoft.com/en-gb/office/graphics-visuals/format-text-effects-text-box-pane) |
| PP-FIT-102 | Wrap text around a circle | W | Edit | 2D6, 5 | Done through warps. | Documented (listing): [Wrap text around a shape](https://support.microsoft.com/en-us/topic/7b58b220-2db6-4f08-93c9-0fe69be48d30) |
| PP-FIT-103 | Symbols | W M; limited Web | Edit | 5 |  | Documented: [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint) |
| PP-FIT-104 | Hyperlinks, link colour | W M Web | Edit | 5, 3 | Allowlist `SEC-N03`. | Documented: [Compare features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b), [Hyperlink](https://support.microsoft.com/en-us/powerpoint/training/add-a-hyperlink-to-a-slide), [Hyperlink colour](https://support.microsoft.com/en-us/PowerPoint/change-the-color-of-hyperlink-text-throughout-a-presentation) |
| PP-FIT-105 | Find | W M Web | Edit | 5 |  | Documented: [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint) |
| PP-FIT-106 | Replace | W M; not Web | Edit | 5 |  | Documented: [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint), [Change and replace fonts](https://support.microsoft.com/en-us/powerpoint/change-the-fonts-in-a-presentation) |
| PP-FIT-107 | Replace Fonts | W | Edit | 5 |  | Documented: [Change and replace fonts](https://support.microsoft.com/en-us/powerpoint/change-the-fonts-in-a-presentation) |
| PP-FIT-108 | Embed fonts (all or used characters) | W M | Edit | 1B, 10 (`FONT-01`) | Read and draw from 2D1 after `FONT-01`; writing needs a rights check. | Documented: [Embedding fonts](https://support.microsoft.com/en-us/office/fonts/benefits-of-embedding-custom-fonts), [Reduce file size](https://support.microsoft.com/en-us/powerpoint/reduce-the-file-size-of-your-powerpoint-presentations) |
| PP-FIT-109 | Cloud fonts | ? | Host seam | 2D1 (`TXT-01`) | Download from Microsoft is not available to us. A host supplies a font library (`ENT-17`); air-gapped has none. | Inferred: Microsoft's cloud font service is known from recall; not on a page read. The seam is required by `ENT-07` either way |
| PP-FIT-110 | Spelling | W M Web | Host seam | 5 | 47 already calls it a seam: dictionaries are not bundled by default in air-gapped form. A local checker can sit behind it. | Documented: [Spelling on the web](https://support.microsoft.com/en-us/office/check-spelling-and-grammar-in-powerpoint-for-the-web-36069a9a-a502-4db1-b5ac-9c556b4fd060), [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint) |
| PP-FIT-111 | Editor pane | Web retired? | Refuse | n/a | A summary of MC1179161 says it left the web in January 2026 (not confirmed in the MC text read). Spelling in place stays, by the row above. | Inferred: a University of Toronto notice quoted in search results; not read directly |
| PP-FIT-112 | Thesaurus | ? | Host seam | 5 | Needs a licensed data set. 48 records it as absent on the web, from a Microsoft page dated 2023. | Inferred: 48's reading of the service description; no thesaurus page came back in search |
| PP-FIT-113 | Proofing language per run | W M Web | Edit | 5 |  | Documented: [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint) |
| PP-FIT-114 | AutoCorrect | ? | Edit | 5 | No Microsoft page returned; assumed from recall. 50 (behaviour) should own its rules. | Inferred: recall; not on a page read |
| PP-FIT-115 | Translate | W M; 365 and 2021+ | Host seam | 10 | Microsoft states internet and connected experiences are required. Air-gapped default (`ENT-07`) forbids a built-in call, so a host supplies the service. Same UI: pick language, Insert. | Documented: [Translate text](https://support.microsoft.com/en-us/office/translate-text-into-a-different-language-287380e4-a56c-48a1-9977-f2dca89ce93f) |
| PP-FIT-116 | Dictate | W M Web | Host seam | 10 | Speech service. Browser speech APIs may send audio to a vendor, so off unless the host configures one. | Documented: [Dictate](https://support.microsoft.com/en-us/office/dictate-your-presentations-and-slide-notes-in-powerpoint-97f3373e-58b9-4e39-b413-83d6d2c09055) |
| PP-FIT-117 | Immersive Reader, Speak | platforms not stated | Host seam | 10 | Read-aloud voices and reading aids belong to the host or OS. The reformatting half could be ours later; not studied. | Documented: [Immersive Reader](https://support.microsoft.com/en-us/powerpoint/use-immersive-reader-in-powerpoint) |
| PP-FIT-118 | Microsoft Search box | W | Edit | 3 | Commands and help only. Web results are a host seam. | Documented: [New in 2021](https://support.microsoft.com/en-gb/office/what-s-new-in-powerpoint-2021-for-windows-dac3eb1b-483b-4043-a0c9-5c01c2c1a8f2) |

### Pictures and graphics

| ID | Feature | PowerPoint has it on | Disposition | Phase or tier | Reason or note | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| PP-FIT-119 | Insert pictures | W M Web | Edit | 4 |  | Documented: [Compare features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b), [Insert a picture](https://support.microsoft.com/en-us/topic/3c51edf4-22e1-460a-b372-9329a8724344) |
| PP-FIT-120 | Online pictures | W M Web | Host seam | 10 | Bing and Microsoft content are not available to us (`01-ORD`, stock row). Host library only. | Documented: [Compare features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b) |
| PP-FIT-121 | Stock images, icons, cut-outs | W; subscribers | Host seam | 10 (`ENT-17`) | Licensed Microsoft content, out of scope in `01-ORD`. | Documented: [New in 2021](https://support.microsoft.com/en-gb/office/what-s-new-in-powerpoint-2021-for-windows-dac3eb1b-483b-4043-a0c9-5c01c2c1a8f2) |
| PP-FIT-122 | Icons | W M | Edit | 6 | Same verb with our open set (47). Library is the row above. | Documented: [Icons](https://support.microsoft.com/en-us/PowerPoint/icons-a-new-kind-of-office-clip-art) |
| PP-FIT-123 | Clip art | W | Host seam | 10 | Same library seam. Listed page. | Documented (listing): [Clip art](https://support.microsoft.com/en-us/topic/0a01ae25-973c-4c2c-8eaf-8c8e1f9ab530) |
| PP-FIT-124 | Crop, delete cropped areas | W M; Web crop only | Edit | 4 |  | Documented: [Crop](https://support.microsoft.com/en-us/office/graphics-visuals/crop-a-picture-in-office), [Reduce picture size](https://support.microsoft.com/en-us/office/graphics-visuals/reduce-the-file-size-of-a-picture-in-microsoft-office) |
| PP-FIT-125 | Crop to shape | W M | Edit | 4 |  | Documented: [Crop to shape](https://support.microsoft.com/en-us/office/graphics-visuals/crop-a-picture-to-fit-in-a-shape) |
| PP-FIT-126 | Compress Pictures | W M; not Web | Edit | 6 | Options: resolution, delete cropped, one or all. `DEC-06` brings it to web; re-encoding is local. | Documented: [Reduce picture size](https://support.microsoft.com/en-us/office/graphics-visuals/reduce-the-file-size-of-a-picture-in-microsoft-office), [Compare features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b), [Reduce file size](https://support.microsoft.com/en-us/powerpoint/reduce-the-file-size-of-your-powerpoint-presentations) |
| PP-FIT-127 | Discard editing data; image default resolution | W | Edit | 6 | Options; carried in the file settings. | Documented: [Options: Advanced](https://support.microsoft.com/en-us/powerpoint/powerpoint-options-advanced), [Reduce file size](https://support.microsoft.com/en-us/powerpoint/reduce-the-file-size-of-your-powerpoint-presentations) |
| PP-FIT-128 | Picture corrections, colour, styles | W M; part on Web | Edit | 4, 2C |  | Documented: [Compare features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b), [Format pictures](https://support.microsoft.com/en-us/powerpoint/training/format-pictures-on-slides), [Edit pictures](https://support.microsoft.com/en-us/office/graphics-visuals/powerpointexp/edit-pictures) |
| PP-FIT-129 | Artistic effects | W M | Edit | 6, 2C | Stored as a rendered result plus source. Only one at a time, per Microsoft. We need our own filters: no format fixes their look. | Documented: [Edit pictures](https://support.microsoft.com/en-us/office/graphics-visuals/powerpointexp/edit-pictures) |
| PP-FIT-130 | Remove Background | W M; not on Web | Edit | 6 | Automatic detection plus marks. A local method is required. Greyed for vectors, as Microsoft says. | Documented: [Remove background](https://support.microsoft.com/en-us/office/graphics-visuals/remove-the-background-of-a-picture-in-office), [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint), [New in 2019](https://support.microsoft.com/en-us/office/what-s-new-in-powerpoint-2019-for-windows-8355a56a-f643-42d2-8454-784fa9b3d109) |
| PP-FIT-131 | Transparency | W M | Edit | 4 |  | Documented: [Icons](https://support.microsoft.com/en-us/PowerPoint/icons-a-new-kind-of-office-clip-art) |
| PP-FIT-132 | SVG: insert, recolour | W M | Edit | 2E, 4 | Sanitised (`SEC-E01`). | Documented: [Edit SVG](https://support.microsoft.com/en-us/office/graphics-visuals/edit-svg-images-in-microsoft-365), [New in 2019](https://support.microsoft.com/en-us/office/what-s-new-in-powerpoint-2019-for-windows-8355a56a-f643-42d2-8454-784fa9b3d109) |
| PP-FIT-133 | Convert SVG to shape | W M | Edit | 6 |  | Documented: [Edit SVG](https://support.microsoft.com/en-us/office/graphics-visuals/edit-svg-images-in-microsoft-365), [Icons](https://support.microsoft.com/en-us/PowerPoint/icons-a-new-kind-of-office-clip-art) |
| PP-FIT-134 | Linked pictures | W; placeholder on Web | Retain | 1H | Stored link and cached image are kept. Never fetched automatically (`ENT-22` external content). Inserting one is the next row. | Documented: [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint) |
| PP-FIT-135 | Picture link: insert | W | Host seam | 10 | A browser cannot resolve a local path. Desktop or host supplies it. Web links obey `SEC-N03`. | Inferred: from linked-picture behaviour on the web page; no insert page was read |
| PP-FIT-136 | Animated GIF: insert, play | W M Web | Edit | 1H, 8A |  | Documented (listing): [Insert an animated GIF](https://support.microsoft.com/en-us/PowerPoint/add-an-animated-gif-to-a-slide) |
| PP-FIT-137 | Insert screenshot, window list | W; M not stated; not Web | Edit | 6 | Screen clipping everywhere the platform offers capture. The browser picker has no window gallery, so the web verb starts with its own chooser (Inferred). | Documented: [Insert a screenshot](https://support.microsoft.com/en-us/topic/56ade2ac-7285-4e7b-a0b1-40646b275703), [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint) |
| PP-FIT-138 | Do not auto-hyperlink screenshot | W | Edit | 6 |  | Documented: [Options: Advanced](https://support.microsoft.com/en-us/powerpoint/powerpoint-options-advanced) |
| PP-FIT-139 | Screen recording | W | Edit | 6 | Area, audio, pointer; embed or save; 64 by 64 minimum. Needs a capture API; `LEG-01` for codecs. Absent from 06. | Documented: [Record your screen](https://support.microsoft.com/en-gb/office/record-your-screen-in-powerpoint-0b4c3f65-534c-4cf1-9c59-402b6e9d79d0), [Add, format and record video](https://support.microsoft.com/en-us/powerpoint/training/add-format-and-record-video-in-powerpoint) |
| PP-FIT-140 | Photo album: create and edit | W | Edit | 6, 1B | Layouts, frames, captions, black and white, rotate, contrast, brightness, theme. `p:photoAlbum` is in the format. Mac and web not stated. | Documented: [Photo album](https://support.microsoft.com/en-us/office/create-and-share-a-photo-album-6febdf8b-5179-4e63-90a6-0fb68df97ec6) |
| PP-FIT-141 | Watermark | W | Edit | 7 | Done with a master picture or text. | Documented (listing): [Watermark](https://support.microsoft.com/en-us/office/add-a-watermark-to-your-slides-1246244d-f465-4e4f-b9f9-49acdae00ff1), [Background picture](https://support.microsoft.com/en-us/PowerPoint/training/add-a-background-picture-to-slides) |
| PP-FIT-142 | Timeline, org chart from templates or SmartArt | W | Edit | 6 |  | Documented (listing): [Timeline](https://support.microsoft.com/en-us/topic/9c4448a9-99c7-4b0e-8eff-0dcf535f223c), [Organization chart](https://support.microsoft.com/en-us/topic/9b51f667-11b7-4971-a757-a08a36684ee6) |

### Tables, charts, SmartArt, equations, 3D

| ID | Feature | PowerPoint has it on | Disposition | Phase or tier | Reason or note | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| PP-FIT-143 | Tables: insert, rows, columns, size | W M Web | Edit | 1G, 2E, 6 |  | Documented: [Compare features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b), [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint) |
| PP-FIT-144 | Merge and split cells | W M; not Web | Edit | 6 |  | Documented: [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint) |
| PP-FIT-145 | Table styles, borders, shading | W M | Edit | 6 |  | Documented (listing): [Table look](https://support.microsoft.com/en-us/PowerPoint/change-the-look-of-a-table), [Table border](https://support.microsoft.com/en-us/topic/21357b24-3d01-4821-9b7b-4f99f886864a), [Table size](https://support.microsoft.com/en-us/PowerPoint/change-the-size-of-a-table-column-or-row-in-powerpoint), [Table rows](https://support.microsoft.com/en-us/PowerPoint/add-or-delete-table-rows-and-columns) |
| PP-FIT-146 | Animate table parts | W | Edit | 8B | Microsoft answers this with a workaround; ours does the same until the format says otherwise. | Documented (listing): [Animate table parts](https://support.microsoft.com/en-us/PowerPoint/i-want-to-animate-individual-cells-rows-or-columns-in-my-table) |
| PP-FIT-147 | Paste Excel range: embed, picture, text | W M | Edit | 6 |  | Documented: [Excel data in PowerPoint](https://support.microsoft.com/en-us/powerpoint/insert-and-update-excel-data-in-powerpoint) |
| PP-FIT-148 | Linked Excel chart or range: update | W M | Host seam | 10 | The source file is out of the browser's reach. Desktop or OpenCalc SDK (`CROSS-01`). | Documented: [Excel data in PowerPoint](https://support.microsoft.com/en-us/powerpoint/insert-and-update-excel-data-in-powerpoint) |
| PP-FIT-149 | Charts: classic families | W M; Web conflicting | Edit | 1J, 2F, 6 | Data through the OpenCalc SDK. | Documented: [Compare features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b), [Charts](https://support.microsoft.com/en-us/PowerPoint/training/use-charts-and-graphs-in-your-presentation), [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint) |
| PP-FIT-150 | Charts: 2016+ (funnel and others) | W M | Edit | 1J, 2H, 6 |  | Documented: [New in 2019](https://support.microsoft.com/en-us/office/what-s-new-in-powerpoint-2019-for-windows-8355a56a-f643-42d2-8454-784fa9b3d109) |
| PP-FIT-151 | Map chart: geocoding | W | Host seam | 10 | Region names need a geographic service (Inferred). The chart draws from cached data; the lookup is a seam. Air-gapped has none. | Documented: [New in 2019](https://support.microsoft.com/en-us/office/what-s-new-in-powerpoint-2019-for-windows-8355a56a-f643-42d2-8454-784fa9b3d109) |
| PP-FIT-152 | SmartArt: create, edit, text pane | W M; limited Web | Edit | 1K, 2J, 6 | `DGM-01`. | Documented: [Compare features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b), [Create SmartArt](https://support.microsoft.com/en-us/office/graphics-visuals/create-a-smartart-graphic-from-scratch), [All SmartArt, described](https://support.microsoft.com/en-au/office/all-smartart-graphics-described-cf1a453b-de4a-4217-8da0-1aff97bb32cd) |
| PP-FIT-153 | SmartArt: convert to shapes | W | Edit | 6 |  | Documented: [SmartArt to shapes](https://support.microsoft.com/en-us/powerpoint/convert-your-smartart-graphic-into-shapes) |
| PP-FIT-154 | Equation: insert, gallery of structures | W M; not Web | Edit | 1K, 2K, 6 | `DGM-01` makes this Edit. 47 still says Render. | Documented: [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint), [Equations by keyboard](https://support.microsoft.com/en-us/accessibility/office-accessibility/use-keyboard-to-work-with-equations), [Write an equation](https://support.microsoft.com/en-us/office/write-an-equation-or-formula-1d01cabc-ceb1-458d-bc70-7f9737722702) |
| PP-FIT-155 | Equation: linear, UnicodeMath, LaTeX | W | Edit | 6 |  | Documented: [Write an equation](https://support.microsoft.com/en-us/office/write-an-equation-or-formula-1d01cabc-ceb1-458d-bc70-7f9737722702), [Equations by keyboard](https://support.microsoft.com/en-us/accessibility/office-accessibility/use-keyboard-to-work-with-equations) |
| PP-FIT-156 | Equation: save to gallery | W | Edit | 6 | Microsoft's steps are Word's, with PowerPoint in Applies To. | Documented: [Write an equation](https://support.microsoft.com/en-us/office/write-an-equation-or-formula-1d01cabc-ceb1-458d-bc70-7f9737722702) |
| PP-FIT-157 | Old Equation Editor objects | ? | Render | 1L, 2I | Stored as OLE with a picture: we draw that, and cannot convert without the old layout rules. | Inferred: from OLE retention; no page read on old objects |
| PP-FIT-158 | 3D models: show | W M; Web plays | Edit | 1L, 2F, 8A | Spec's `MODEL3D` says Retain; a clone needs them drawn live. Needs model parsers and a renderer no tier owns. See roadmap asks. | Documented: [3D models](https://support.microsoft.com/en-us/office/graphics-visuals/get-creative-with-3d-models), [Animated 3D graphics](https://support.microsoft.com/en-us/office/insert-animated-3d-graphics-6f08009a-3da5-400d-a706-8e23f304cd72), [New in 2019](https://support.microsoft.com/en-us/office/what-s-new-in-powerpoint-2019-for-windows-8355a56a-f643-42d2-8454-784fa9b3d109) |
| PP-FIT-159 | 3D models: insert, rotate, pan, zoom | W M; not Web | Edit | 6 | Microsoft names no file formats on the page read. Formats follow from a sample before design. | Documented: [3D models](https://support.microsoft.com/en-us/office/graphics-visuals/get-creative-with-3d-models), [Animated 3D graphics](https://support.microsoft.com/en-us/office/insert-animated-3d-graphics-6f08009a-3da5-400d-a706-8e23f304cd72) |
| PP-FIT-160 | 3D models: online library | W M | Host seam | 10 | Microsoft's content; host library only. | Documented: [3D models](https://support.microsoft.com/en-us/office/graphics-visuals/get-creative-with-3d-models) |
| PP-FIT-161 | 3D animation effects | W | Edit | 8B | Schema namespace `x:drawing/2018/animation/model3d`. | Documented (listing): [3D animation effects](https://support.microsoft.com/en-us/powerpoint/3d-animation-effects-in-powerpoint) |
| PP-FIT-162 | Shape 3D format and rotation | W M | Edit | 1E, 2C, 6 |  | Observed: `D3` in [spec/README](spec/README.md) |

### Ink and drawing

| ID | Feature | PowerPoint has it on | Disposition | Phase or tier | Reason or note | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| PP-FIT-163 | Draw tab: pens, pencil, highlighter | W M | Edit | 1K, 2I, 6 |  | Documented: [Draw and write with ink](https://support.microsoft.com/en-us/office/draw-and-write-with-ink-in-office-6d76c674-7f4b-414d-b67f-b3ffef6ccf53), [New in 2019](https://support.microsoft.com/en-us/office/what-s-new-in-powerpoint-2019-for-windows-8355a56a-f643-42d2-8454-784fa9b3d109) |
| PP-FIT-164 | Ink effects, pen set | W | Edit | 6 |  | Documented: [New in 2019](https://support.microsoft.com/en-us/office/what-s-new-in-powerpoint-2019-for-windows-8355a56a-f643-42d2-8454-784fa9b3d109) |
| PP-FIT-165 | Point, stroke and segment erasers | W | Edit | 6 |  | Documented: [New in 2019](https://support.microsoft.com/en-us/office/what-s-new-in-powerpoint-2019-for-windows-8355a56a-f643-42d2-8454-784fa9b3d109), [New in 2021](https://support.microsoft.com/en-gb/office/what-s-new-in-powerpoint-2021-for-windows-dac3eb1b-483b-4043-a0c9-5c01c2c1a8f2) |
| PP-FIT-166 | Lasso select | W | Edit | 6 |  | Documented: [New in 2021](https://support.microsoft.com/en-gb/office/what-s-new-in-powerpoint-2021-for-windows-dac3eb1b-483b-4043-a0c9-5c01c2c1a8f2), [Replay ink](https://support.microsoft.com/en-us/office/replay-your-ink-strokes-in-office-fa4f044f-810b-43fe-b774-da04a0b37496) |
| PP-FIT-167 | Automatic inking setting | W | Edit | 6 |  | Documented (listing): [Automatic inking](https://support.microsoft.com/en-us/office/turn-off-automatic-inking-913b4bc9-4c92-4483-b434-a4e881c1556f) |
| PP-FIT-168 | Use pen to select by default | W | Edit | 6 |  | Documented: [Options: Advanced](https://support.microsoft.com/en-us/powerpoint/powerpoint-options-advanced) |
| PP-FIT-169 | Ink to Shape | W M? mobile | Edit | 6 | Needs our own shape recogniser; local, simple. Desktop form was not confirmed on a page read. | Documented: [Ink to shapes and math](https://support.microsoft.com/en-us/office/convert-your-ink-to-shapes-and-math-equations-36fc0d83-ecdb-47d9-a844-c2b889820c59), [Draw and write with ink](https://support.microsoft.com/en-us/office/draw-and-write-with-ink-in-office-6d76c674-7f4b-414d-b67f-b3ffef6ccf53) |
| PP-FIT-170 | Ink to Text | W | Host seam | 6 | Handwriting recognition is a model. Browser or OS handwriting if the host allows; never a default network call. | Inferred: search did not return this page; verb recalled from the Draw tab |
| PP-FIT-171 | Ink to Math | W | Host seam | 6 | Same recognition seam. The result enters the equation editor as Edit. | Documented: [Ink to shapes and math](https://support.microsoft.com/en-us/office/convert-your-ink-to-shapes-and-math-equations-36fc0d83-ecdb-47d9-a844-c2b889820c59), [Write an equation](https://support.microsoft.com/en-us/office/write-an-equation-or-formula-1d01cabc-ceb1-458d-bc70-7f9737722702) |
| PP-FIT-172 | Ink Replay button | W M 2021+ | Edit | 6 | Works in Normal view only, as Microsoft states. | Documented: [Replay ink](https://support.microsoft.com/en-us/office/replay-your-ink-strokes-in-office-fa4f044f-810b-43fe-b774-da04a0b37496), [New in 2021](https://support.microsoft.com/en-gb/office/what-s-new-in-powerpoint-2021-for-windows-dac3eb1b-483b-4043-a0c9-5c01c2c1a8f2) |
| PP-FIT-173 | Replay and Rewind ink animations | W M 2021+ | Edit | 1I, 8B | Default two seconds; lasso first. `inkAction` is in the format. | Documented: [Replay ink](https://support.microsoft.com/en-us/office/replay-your-ink-strokes-in-office-fa4f044f-810b-43fe-b774-da04a0b37496) |
| PP-FIT-174 | Digital pen button advances slides | W | Edit | 8A | Only where the pen button reaches the page (Inferred); otherwise desktop. | Documented: [New in 2019](https://support.microsoft.com/en-us/office/what-s-new-in-powerpoint-2019-for-windows-8355a56a-f643-42d2-8454-784fa9b3d109) |

### Transitions and animation

| ID | Feature | PowerPoint has it on | Disposition | Phase or tier | Reason or note | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| PP-FIT-175 | Transitions gallery, effect options | W M; limited Web | Edit | 1I, 8A, 8B |  | Documented: [Compare features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b), [Transitions](https://support.microsoft.com/en-us/PowerPoint/training/add-change-or-remove-transitions-between-slides) |
| PP-FIT-176 | Transition speed and advance | W M | Edit | 8B |  | Documented (listing): [Transition timing](https://support.microsoft.com/en-us/PowerPoint/set-the-timing-and-speed-of-a-transition) |
| PP-FIT-177 | Morph | W M Web | Edit | 1I, 8A | Falls back to Fade in old versions: stored fallback is honoured. | Documented: [Morph](https://support.microsoft.com/en-us/powerpoint/training/use-the-morph-transition-in-powerpoint) |
| PP-FIT-178 | Summary, Slide and Section Zoom | W M; not Web | Edit | 1M, 2F, 8A | `DEC-06`. | Documented: [Zoom](https://support.microsoft.com/en-us/powerpoint/use-zoom-for-powerpoint-to-bring-your-presentation-to-life) |
| PP-FIT-179 | Animations gallery, classes | W M; limited Web | Edit | 1I, 8A, 8B |  | Documented: [Compare features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b), [Animate](https://support.microsoft.com/en-us/PowerPoint/training/animate-text-or-objects) |
| PP-FIT-180 | Multiple effects on one object | W M | Edit | 8B |  | Documented (listing): [Multiple animations](https://support.microsoft.com/en-us/PowerPoint/apply-multiple-animation-effects-to-one-object) |
| PP-FIT-181 | Animation Painter | W M | Edit | 8B |  | Documented: [Animation painter](https://support.microsoft.com/en-us/office/duplicate-animations-with-the-animation-painter-f2246f1c-87b4-4cb3-81fa-336e538ba8c8) |
| PP-FIT-182 | Animation Pane, start, duration, delay | W M | Edit | 8B |  | Documented: [Animation timing](https://support.microsoft.com/en-us/powerpoint/set-the-start-time-and-speed-of-an-animation-effect) |
| PP-FIT-183 | Motion paths | W M | Edit | 8A, 8B |  | Documented (listing): [Motion path](https://support.microsoft.com/en-us/PowerPoint/add-a-motion-path-animation-effect) |
| PP-FIT-184 | Trigger on click of an object | W; not Web or Mac | Edit | 8A, 8B |  | Documented: [Compare features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b), [Trigger](https://support.microsoft.com/en-us/powerpoint/trigger-an-animation-effect) |
| PP-FIT-185 | Trigger on media bookmark | W | Edit | 8A, 8B |  | Documented: [Media bookmarks](https://support.microsoft.com/en-us/powerpoint/add-bookmarks-to-points-of-interest-in-audio-and-video-clips) |
| PP-FIT-186 | Text by word, by line | W M | Edit | 8B |  | Documented (listing): [Line by line](https://support.microsoft.com/en-us/powerpoint/animate-or-make-words-appear-one-line-at-a-time-in-powerpoint) |
| PP-FIT-187 | Animate a group | W M | Edit | 8B |  | Documented (listing): [Animate a group](https://support.microsoft.com/en-us/PowerPoint/group-and-animate-a-set-of-objects) |
| PP-FIT-188 | Sound effects on animation and hyperlink | W | Edit | 8B |  | Documented (listing): [Sound effects](https://support.microsoft.com/en-us/powerpoint/add-sound-effects-to-an-animation-or-hyperlink) |
| PP-FIT-189 | Remove or turn off animation | W M | Edit | 8B |  | Documented (listing): [Remove animations](https://support.microsoft.com/en-us/PowerPoint/change-remove-or-turn-off-animation-effects-in-powerpoint) |
| PP-FIT-190 | SmartArt animation | W | Edit | 8B |  | Documented: [ODP](https://support.microsoft.com/en-us/powerpoint/use-powerpoint-to-save-or-open-a-presentation-in-the-opendocument-presentation-odp-format) |

### Audio and video

| ID | Feature | PowerPoint has it on | Disposition | Phase or tier | Reason or note | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| PP-FIT-191 | Audio: add, remove | W M; not Web add | Edit | 1H, 6 | `DEC-06`. | Documented: [Compare features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b), [Add audio](https://support.microsoft.com/en-us/powerpoint/training/add-or-delete-audio-in-your-powerpoint-presentation) |
| PP-FIT-192 | Record audio on a slide | W M | Edit | 6 | Absent from 06's Phase 6 list. | Documented: [Add audio](https://support.microsoft.com/en-us/powerpoint/training/add-or-delete-audio-in-your-powerpoint-presentation) |
| PP-FIT-193 | Video from a file, embed | W M Web | Edit | 1H, 6 |  | Documented: [Compare features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b), [Add a video](https://support.microsoft.com/en-us/powerpoint/insert-and-play-a-video-file-from-your-computer), [Embedding video](https://support.microsoft.com/en-us/PowerPoint/embed-a-video-in-powerpoint) |
| PP-FIT-194 | Online video (YouTube and others) | W M Web | Edit | 6, 8A | Stored as a link. Opt-in: the air-gapped default shows the poster only (`SEC-014` web embeds). | Documented: [Compare features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b), [Online video](https://support.microsoft.com/en-us/PowerPoint/training/insert-a-video-from-youtube-or-another-site) |
| PP-FIT-195 | Stream and SharePoint video | W M 2024 | Host seam | 10 | Microsoft's service; a host may play equivalent. | Documented: [New in 2024](https://support.microsoft.com/en-us/office/what-s-new-in-powerpoint-2024-for-windows-and-mac-4330d63f-ffbb-470d-bbc2-a5f340317fbc) |
| PP-FIT-196 | Trim, fade, bookmark | W M; not Web | Edit | 1H, 6 | `x:powerpoint/2010/main:CT_MediaTrim` is in the format. | Documented: [Compare features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b), [Trim](https://support.microsoft.com/en-us/PowerPoint/trim-a-video-music-clip-or-sound-clip-in-powerpoint), [Media bookmarks](https://support.microsoft.com/en-us/powerpoint/add-bookmarks-to-points-of-interest-in-audio-and-video-clips) |
| PP-FIT-197 | Play across slides, in background | W M Web | Edit | 8A |  | Documented: [Compare features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b), [Music across slides](https://support.microsoft.com/en-us/powerpoint/play-music-across-multiple-slides-in-your-slide-show-in-powerpoint) |
| PP-FIT-198 | Sound automatically on a slide | W M | Edit | 8A |  | Documented (listing): [Sound on slide appearing](https://support.microsoft.com/en-us/PowerPoint/play-music-or-other-sounds-automatically-when-a-slide-appears) |
| PP-FIT-199 | Poster frame, video styles | W M | Edit | 6 | Named in 47. Not on a page read. | Inferred: 47's Video row, from recall; `videofmt` page title suggests it |
| PP-FIT-200 | Media formats accepted | W M | Edit | 6 | What each browser decodes differs. Report, do not hide. `BRW-01`, `LEG-01`. | Documented (listing): [Media formats](https://support.microsoft.com/en-us/PowerPoint/video-and-audio-file-formats-supported-in-powerpoint) |
| PP-FIT-201 | Compress Media (1080p, 720p, 480p) | W; not Mac or Web | Edit | 6 | Needs a transcoder, so codec licences (`LEG-01`). Loses embedded subtitles and alternate audio, as Microsoft warns; say so. | Documented: [Compress media](https://support.microsoft.com/en-us/powerpoint/compress-your-media-files), [Reduce file size](https://support.microsoft.com/en-us/powerpoint/reduce-the-file-size-of-your-powerpoint-presentations) |
| PP-FIT-202 | Optimize Media Compatibility | W | Edit | 6 |  | Documented (listing): [Media compatibility tips](https://support.microsoft.com/en-us/powerpoint/tips-for-improving-audio-and-video-playback-and-compatibility-in-powerpoint) |
| PP-FIT-203 | Save Media As | W | Edit | 6 |  | Documented (listing): [Save embedded media](https://support.microsoft.com/en-au/office/save-embedded-media-from-a-presentation-audio-or-video-f773d119-0fb3-4884-a18a-29573ac97b25) |
| PP-FIT-204 | Media captions: insert WebVTT or SRT | W M Web | Edit | 6, 8A | Not for online video, as Microsoft states. Absent from 06's Phase 6 list. | Documented: [Closed captions](https://support.microsoft.com/en-us/powerpoint/add-closed-captions-or-subtitles-to-media-in-powerpoint), [New in 2024](https://support.microsoft.com/en-us/office/what-s-new-in-powerpoint-2024-for-windows-and-mac-4330d63f-ffbb-470d-bbc2-a5f340317fbc) |
| PP-FIT-205 | Media captions: generate by speech | Web | Host seam | 10 | Speech recognition. Microsoft does this on the web only. Air-gapped default off. | Documented: [Closed captions](https://support.microsoft.com/en-us/powerpoint/add-closed-captions-or-subtitles-to-media-in-powerpoint) |
| PP-FIT-206 | Media captions: translate | Web | Host seam | 10 | Same translation seam. | Documented: [Closed captions](https://support.microsoft.com/en-us/powerpoint/add-closed-captions-or-subtitles-to-media-in-powerpoint) |

### Slide show and recording

| ID | Feature | PowerPoint has it on | Disposition | Phase or tier | Reason or note | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| PP-FIT-207 | From Beginning, From Current | W M Web | Edit | 8A |  | Documented: [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint) |
| PP-FIT-208 | Set Up Show: presenter, individual, kiosk | W M; not Web | Edit | 1B, 8A | `p:CT_ShowInfoKiosk`. | Documented: [Self-running](https://support.microsoft.com/en-us/powerpoint/training/create-a-self-running-presentation), [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint) |
| PP-FIT-209 | Self-running, loop until Esc | W M; not Web | Edit | 8A |  | Documented: [Self-running](https://support.microsoft.com/en-us/powerpoint/training/create-a-self-running-presentation), [Compare features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b) |
| PP-FIT-210 | Custom show | W M; not Web | Edit | 8A |  | Documented: [Custom show](https://support.microsoft.com/en-us/office/create-and-present-a-custom-show-09d4d340-3c47-4125-b177-0de3be462c5d), [Compare features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b) |
| PP-FIT-211 | Presenter view | W M; not Web | Edit | 8A | Web uses a second window (Inferred); desktop uses displays. | Documented: [Compare features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b), [Presenter view](https://support.microsoft.com/en-us/powerpoint/training/start-the-presentation-and-see-your-notes-in-presenter-view) |
| PP-FIT-212 | Multiple monitors, swap displays | W M | Edit | 8A | Browser display API limits apply (`BRW-01`). | Documented (listing): [Multiple monitors](https://support.microsoft.com/en-us/PowerPoint/present-on-multiple-monitors-and-view-speaker-notes-privately) |
| PP-FIT-213 | Pen and highlighter in show | W M; Web no save | Edit | 8A |  | Documented: [Draw during a presentation](https://support.microsoft.com/en-us/office/draw-on-slides-during-a-presentation-80a78a11-cb5d-4dfc-a1ad-a26e877da770) |
| PP-FIT-214 | Keep or discard ink on exit | W M; not Web | Edit | 8A | Option to stop asking. A `DEC-06` improvement over the web. | Documented: [Draw during a presentation](https://support.microsoft.com/en-us/office/draw-on-slides-during-a-presentation-80a78a11-cb5d-4dfc-a1ad-a26e877da770), [Options: Advanced](https://support.microsoft.com/en-us/powerpoint/powerpoint-options-advanced) |
| PP-FIT-215 | Laser pointer | W M | Edit | 8A | Trace format `CT_LaserTracePoint`. | Documented: [Laser pointer](https://support.microsoft.com/en-us/powerpoint/turn-your-mouse-into-a-laser-pointer), [Draw during a presentation](https://support.microsoft.com/en-us/office/draw-on-slides-during-a-presentation-80a78a11-cb5d-4dfc-a1ad-a26e877da770) |
| PP-FIT-216 | Hide ink, erase, hide pointer | W | Edit | 8A | Key list belongs to 49. | Documented: [Draw during a presentation](https://support.microsoft.com/en-us/office/draw-on-slides-during-a-presentation-80a78a11-cb5d-4dfc-a1ad-a26e877da770), [Show shortcuts](https://support.microsoft.com/en-us/accessibility/powerpoint/use-keyboard-shortcuts-to-deliver-powerpoint-presentations) |
| PP-FIT-217 | Black or white screen, go to slide | W | Edit | 8A |  | Documented: [Show shortcuts](https://support.microsoft.com/en-us/accessibility/powerpoint/use-keyboard-shortcuts-to-deliver-powerpoint-presentations), [Go to a slide](https://support.microsoft.com/en-us/powerpoint/go-to-a-slide-when-delivering-your-presentation) |
| PP-FIT-218 | Zoom in slide show, to 400% | W M 2024 | Edit | 8A |  | Documented: [New in 2024](https://support.microsoft.com/en-us/office/what-s-new-in-powerpoint-2024-for-windows-and-mac-4330d63f-ffbb-470d-bbc2-a5f340317fbc) |
| PP-FIT-219 | Action buttons, action settings: link, slide, sound | W M | Edit | 1D, 4, 8A | Program and macro actions are Retain rows. | Documented: [Action buttons](https://support.microsoft.com/en-us/powerpoint/add-commands-to-your-presentation-with-action-buttons) |
| PP-FIT-220 | Rehearse Timings | W M | Edit | 8B |  | Documented (listing): [Rehearse timings](https://support.microsoft.com/en-us/PowerPoint/training/rehearse-and-time-the-delivery-of-a-presentation) |
| PP-FIT-221 | Record narration, timings by slide | W M; not Web | Edit | 8B | **Adds it to Phase 8**, whose charter defers to this analysis. Embedded per slide; no sound during transitions. Clear per slide or all. | Documented: [Record your presentation](https://support.microsoft.com/en-us/PowerPoint/record-your-presentation), [Record with narration and timings](https://support.microsoft.com/en-us/powerpoint/training/record-a-slide-show-with-narration-and-slide-timings), [Compare features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b) |
| PP-FIT-222 | Record camera video narration | W; M not split out | Edit | 8B | Camera permission in browser; `LEG-01`. | Documented: [Record your presentation](https://support.microsoft.com/en-us/PowerPoint/record-your-presentation), [New in 2024](https://support.microsoft.com/en-us/office/what-s-new-in-powerpoint-2024-for-windows-and-mac-4330d63f-ffbb-470d-bbc2-a5f340317fbc) |
| PP-FIT-223 | Record ink and laser gestures | W; M not split out | Edit | 8B | `x:powerpoint/2010/main:<showEvtLst>`. | Documented: [Record your presentation](https://support.microsoft.com/en-us/PowerPoint/record-your-presentation), [New in 2021](https://support.microsoft.com/en-gb/office/what-s-new-in-powerpoint-2021-for-windows-dac3eb1b-483b-4043-a0c9-5c01c2c1a8f2) |
| PP-FIT-224 | Teleprompter while recording | W; M not split out | Edit | 8B |  | Documented: [Record your presentation](https://support.microsoft.com/en-us/PowerPoint/record-your-presentation) |
| PP-FIT-225 | Background blur for camera | W | Edit | 8B | Local filter if performance holds; otherwise drop with a note. | Documented: [Record your presentation](https://support.microsoft.com/en-us/PowerPoint/record-your-presentation) |
| PP-FIT-226 | Cameo: live camera on a slide | W M Web; subscribers | Edit | 8A, 4 | One feed per slide; placeholder on layouts; each person's feed is theirs alone. No element by that name in the checklist, so storage is unknown until `F-007` provides a sample: `SPEC-04`. | Documented: [Cameo](https://support.microsoft.com/en-us/PowerPoint/presenting-with-cameo), [New in 2024](https://support.microsoft.com/en-us/office/what-s-new-in-powerpoint-2024-for-windows-and-mac-4330d63f-ffbb-470d-bbc2-a5f340317fbc) |
| PP-FIT-227 | Cameo effects from Teams | Teams | Host seam | 10 | Inherited from Teams, per Microsoft. Nothing for us to clone unless the host provides. | Documented: [Cameo](https://support.microsoft.com/en-us/PowerPoint/presenting-with-cameo) |
| PP-FIT-228 | Reset recording to Cameo | W; M not split out | Edit | 8B |  | Documented: [Record your presentation](https://support.microsoft.com/en-us/PowerPoint/record-your-presentation) |
| PP-FIT-229 | Export recording to video | W M | Edit | 8B | Same path as the next row. | Documented: [Record your presentation](https://support.microsoft.com/en-us/PowerPoint/record-your-presentation) |
| PP-FIT-230 | Export to video: MP4 | W M; Web not stated | Edit | 8B | 1080p default in Recording Studio, 480p to 4K. Our own deterministic capture, then local encoder; `LEG-01` for H.264. Absent from 06. | Documented: [Presentation into a video](https://support.microsoft.com/en-us/powerpoint/turn-your-presentation-into-a-video), [Movie or MP4 on Mac](https://support.microsoft.com/en-us/powerpoint/save-a-presentation-as-a-movie-file-or-mp4-in-powerpoint-for-mac), [New in 2019](https://support.microsoft.com/en-us/office/what-s-new-in-powerpoint-2019-for-windows-8355a56a-f643-42d2-8454-784fa9b3d109) |
| PP-FIT-231 | Export to video: WMV, MOV | W (WMV), M (MOV) | Refuse | n/a | Single-platform containers; MP4 plays on all. Not worth a second encoder path. | Documented: [Presentation into a video](https://support.microsoft.com/en-us/powerpoint/turn-your-presentation-into-a-video), [File formats](https://support.microsoft.com/en-us/powerpoint/file-formats-that-are-supported-in-powerpoint) |
| PP-FIT-232 | Upload to Clipchamp | W; subscribers | Host seam | 10 | Microsoft's service; `ENT-16` target. | Documented: [Presentation into a video](https://support.microsoft.com/en-us/powerpoint/turn-your-presentation-into-a-video) |
| PP-FIT-233 | Export to animated GIF | W M | Edit | 8B | Quality, seconds per slide (default 1 s). Loops; skips hidden slides. Absent from 06. | Documented: [Animated GIF](https://support.microsoft.com/en-us/powerpoint/make-an-animated-gif-from-a-slide-show), [File formats](https://support.microsoft.com/en-us/powerpoint/file-formats-that-are-supported-in-powerpoint) |
| PP-FIT-234 | Package for CD: copy to folder | W; not Mac or Web | Edit | 10 | Fonts and linked files in one zip, with Inspector first. A browser delivers a download rather than a folder. | Documented: [Package for CD](https://support.microsoft.com/en-us/topic/ac1ae03e-c5f2-4a0d-8bcb-3c48741117c9) |
| PP-FIT-235 | Package for CD: burn a disc | W | Refuse | n/a | A browser cannot burn discs, and the folder row meets the need. | Documented: [Package for CD](https://support.microsoft.com/en-us/topic/ac1ae03e-c5f2-4a0d-8bcb-3c48741117c9) |
| PP-FIT-236 | Live subtitles while presenting | W M 365 | Host seam | 10 | Speech and translation services. J key is 49's. | Documented: [Real-time captions](https://support.microsoft.com/en-us/powerpoint/present-with-real-time-automatic-captions-or-subtitles-in-powerpoint) |
| PP-FIT-237 | Speaker Coach | W M Web mobile | Host seam | 10 | Microsoft states it needs a microphone and internet. The analysis service is Microsoft's. Host service, off by default. | Documented: [Speaker Coach](https://support.microsoft.com/en-us/PowerPoint/rehearse-your-slide-show-with-speaker-coach), [New in 2021](https://support.microsoft.com/en-gb/office/what-s-new-in-powerpoint-2021-for-windows-dac3eb1b-483b-4043-a0c9-5c01c2c1a8f2) |
| PP-FIT-238 | Present Live (audience on phones) | deprecated | Refuse | n/a | Microsoft's page says deprecated and no longer available on the web, pointing to Teams, though the help home still links it. Co-viewing could return through Phase 9 if asked. | Documented: [Live Presentations](https://support.microsoft.com/en-us/PowerPoint/present-live-engage-your-audience-with-live-presentations) |
| PP-FIT-239 | PowerPoint Live in Teams, Teams meeting | Teams | Host seam | 10 | A meeting product. SDK embedding is how a host would do this (`ENT-16`). | Documented: [PowerPoint Live in Teams](https://support.microsoft.com/en-us/teams/meetings/share-slides-in-microsoft-teams-meetings-with-powerpoint-live), [Live Presentations](https://support.microsoft.com/en-us/PowerPoint/present-live-engage-your-audience-with-live-presentations) |
| PP-FIT-240 | Broadcast Slide Show | not on Web | Refuse | n/a | Named only as absent from the web. No current page. Superseded by the two rows above (Inferred). | Documented: [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint) |
| PP-FIT-241 | Microsoft Forms in a slide | W; not Web | Host seam | 10 | Microsoft's service and licence bound. A host may embed its own; the file part stays retained. | Documented: [Insert a form or quiz](https://support.microsoft.com/en-us/forms/insert-a-form-or-quiz-into-powerpoint) |

### Design

| ID | Feature | PowerPoint has it on | Disposition | Phase or tier | Reason or note | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| PP-FIT-242 | Themes, variants | W M Web | Edit | 1C, 2A, 7 | Ours, not Microsoft's content (47). | Documented: [Compare features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b), [Themes](https://support.microsoft.com/en-us/PowerPoint/training/add-color-and-design-to-your-slides-with-themes) |
| PP-FIT-243 | Create a theme: colours, fonts | W M | Edit | 7 |  | Documented (listing): [Create a theme](https://support.microsoft.com/en-us/PowerPoint/create-your-own-theme-in-powerpoint) |
| PP-FIT-244 | Multiple themes in a deck | W M | Edit | 7 |  | Documented (listing): [Multiple themes](https://support.microsoft.com/en-us/PowerPoint/use-multiple-themes-in-one-presentation) |
| PP-FIT-245 | Save as theme or template | W M | Edit | 7 |  | Documented (listing): [Theme as template](https://support.microsoft.com/en-us/PowerPoint/save-a-slide-design-theme-as-a-template-in-powerpoint), [Create a template](https://support.microsoft.com/en-us/PowerPoint/create-and-save-a-powerpoint-template) |
| PP-FIT-246 | Slide master: edit, add layouts | W M; not Web | Edit | 1D, 7 |  | Documented: [Compare features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b), [Slide master](https://support.microsoft.com/en-us/PowerPoint/training/what-is-a-slide-master-in-powerpoint), [Customize a master](https://support.microsoft.com/en-us/PowerPoint/training/customize-a-slide-master) |
| PP-FIT-247 | Copy a master between decks | W M | Edit | 7 |  | Documented (listing): [Copy a master](https://support.microsoft.com/en-us/PowerPoint/copy-and-paste-a-slide-master-from-one-presentation-to-another) |
| PP-FIT-248 | Format background | W M Web | Edit | 7 |  | Documented: [Compare features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b), [Slide background](https://support.microsoft.com/en-gb/office/change-the-background-of-slides-3ac2075c-f51b-4fbd-b356-b4c6748ec966), [Background picture](https://support.microsoft.com/en-us/PowerPoint/training/add-a-background-picture-to-slides) |
| PP-FIT-249 | Header, footer, date, slide number | W M Web | Edit | 1D, 4 |  | Documented: [Compare features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b), [Slide number and footer](https://support.microsoft.com/en-us/PowerPoint/insert-or-change-the-slide-numbers-date-or-footer-for-on-screen-slides-in-powerpoint) |
| PP-FIT-250 | Handout and notes header and footer | W M | Edit | 7 |  | Documented (listing): [Handout header and footer](https://support.microsoft.com/en-us/PowerPoint/edit-page-numbering-footers-and-headers-for-handouts-in-powerpoint), [Notes header and footer](https://support.microsoft.com/en-us/PowerPoint/edit-page-numbering-footers-and-headers-for-notes-pages-in-powerpoint) |
| PP-FIT-251 | New presentation, template gallery | W M Web | Edit | 4 (`ASSET-01`) | Our own set. Tenant set through `ENT-17`. | Documented: [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint), [Create a template](https://support.microsoft.com/en-us/PowerPoint/create-and-save-a-powerpoint-template) |
| PP-FIT-252 | Designer (Design Ideas) | W M; subscribers | Host seam | 10 (`DEC-05`) | `DEC-05`: not built now. Needs an AI service, against air-gapped. Permission asked on first use, per Microsoft. Named Design Suggestions for Copilot users. | Documented: [Designer](https://support.microsoft.com/en-us/powerpoint/training/get-design-ideas-for-slides-with-powerpoint-designer), [Slide design with Designer](https://support.microsoft.com/en-us/PowerPoint/create-professional-slide-layouts-with-designer) |
| PP-FIT-253 | Designer metadata in decks | W M | Retain | 1M | `DEC-05`, spec `DESIGN` is Retain. | Observed: `DESIGN` in [spec/README](spec/README.md) |
| PP-FIT-254 | Microsoft Designer image generation | subscribers | Host seam | 10 (`DEC-05`) | Separate service. | Documented: [Microsoft Designer FAQ](https://support.microsoft.com/en-us/designer/frequently-asked-questions-about-microsoft-designer) |

### Copilot (`DEC-05`)

| ID | Feature | PowerPoint has it on | Disposition | Phase or tier | Reason or note | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| PP-FIT-255 | Copilot: create a deck from a prompt | W M; licence | Host seam | 10 (`DEC-05`) | `DEC-05`: a seam, not a feature. Needs a model service; never a default network call. MCP access (`ENT-16`) is the likely path. | Documented: [Copilot from a prompt](https://support.microsoft.com/en-us/PowerPoint/copilot-tutorial-create-a-presentation-with-a-prompt), [Copilot FAQ](https://support.microsoft.com/en-us/powerpoint/frequently-asked-questions-about-copilot-in-powerpoint) |
| PP-FIT-256 | Copilot: create from a file | W M; licence | Host seam | 10 (`DEC-05`) | As above. | Documented: [Copilot FAQ](https://support.microsoft.com/en-us/powerpoint/frequently-asked-questions-about-copilot-in-powerpoint) |
| PP-FIT-257 | Copilot: add a slide from a file | W M; licence | Host seam | 10 (`DEC-05`) | As above. | Documented: [Copilot slide from file](https://support.microsoft.com/en-us/topic/add-a-slide-from-a-file-with-copilot-in-powerpoint-9034b581-38df-46be-a725-986cbbd4b5d4) |
| PP-FIT-258 | Copilot: rewrite, condense, generate text | W M; licence | Host seam | 10 (`DEC-05`) | As above. | Documented: [Copilot text](https://support.microsoft.com/en-us/topic/generate-text-with-copilot-in-powerpoint-422fd407-4780-47d0-85d3-35a2a6ff4e49), [Copilot FAQ](https://support.microsoft.com/en-us/powerpoint/frequently-asked-questions-about-copilot-in-powerpoint) |
| PP-FIT-259 | Copilot: edit, with a choice of model | W M; commercial | Host seam | 10 (`DEC-05`) | As above. Microsoft's picker includes third-party models. | Documented: [Edit with Copilot](https://support.microsoft.com/en-us/powerpoint/edit-with-copilot-in-powerpoint), [Copilot model choice](https://support.microsoft.com/en-us/office/choose-your-model-when-editing-with-copilot-in-powerpoint) |
| PP-FIT-260 | Copilot: summarise, prepare to present | W M; licence | Host seam | 10 (`DEC-05`) | As above. | Documented: [Copilot FAQ](https://support.microsoft.com/en-us/powerpoint/frequently-asked-questions-about-copilot-in-powerpoint), [Prepare with Copilot](https://support.microsoft.com/en-us/microsoft-365-copilot/prepare-your-presentation-with-microsoft-365-copilot) |
| PP-FIT-261 | Copilot: Brand Kit, template check | licence | Host seam | 10 (`DEC-05`) | As above. Our organisation library (`ENT-17`) covers plain brand control without AI. | Documented: [Copilot Brand Kit](https://support.microsoft.com/en-us/powerpoint/copilot/keep-your-presentation-on-brand-with-copilot) |
| PP-FIT-262 | Copilot: interactive slides | licence | Host seam | 10 (`DEC-05`) | As above. Depth not studied. | Documented (listing): [Copilot interactive slides](https://support.microsoft.com/en-us/powerpoint/copilot/create-interactive-slides-with-copilot-in-powerpoint) |

### Print and share

| ID | Feature | PowerPoint has it on | Disposition | Phase or tier | Reason or note | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| PP-FIT-263 | Print slides, handouts, notes | W M Web | Edit | 2G, 3 |  | Documented: [Compare features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b), [Print](https://support.microsoft.com/en-us/PowerPoint/training/print-your-powerpoint-slides-handouts-or-notes), [Print notes](https://support.microsoft.com/en-us/PowerPoint/print-speaker-notes), [Print handouts, notes or slides](https://support.microsoft.com/en-us/office/print-your-handouts-notes-or-slides-91c62c83-9032-497c-ab76-cae8f3e1a402) |
| PP-FIT-264 | Print Outline | W | Edit | 5, 2G |  | Documented: [Outline view](https://support.microsoft.com/en-us/office/create-and-print-a-presentation-in-outline-view-3516310c-c9c0-4d4f-8c11-2759313477a5) |
| PP-FIT-265 | Print in colour, grayscale, pure black and white | W M | Edit | 2G | `p:ST_PrintColorMode`. Pure black and white drops gray tones and some effects, per Microsoft. | Documented: [Print handouts, notes or slides](https://support.microsoft.com/en-us/office/print-your-handouts-notes-or-slides-91c62c83-9032-497c-ab76-cae8f3e1a402), [Options: Advanced](https://support.microsoft.com/en-us/powerpoint/powerpoint-options-advanced), [Choose the right view](https://support.microsoft.com/en-au/office/choose-the-right-view-for-the-task-in-powerpoint-21332d8d-adbc-4717-a2c6-e25a697b40e9) |
| PP-FIT-266 | Print: scale to fit, frame slides, print hidden | W | Edit | 2G |  | Documented: [Options: Advanced](https://support.microsoft.com/en-us/powerpoint/powerpoint-options-advanced) |
| PP-FIT-267 | Printer-specific options: TrueType as graphics, background | W | Refuse | n/a | Browser and OS print system owns these. PDF is what we hand to it. | Documented: [Options: Advanced](https://support.microsoft.com/en-us/powerpoint/powerpoint-options-advanced) |
| PP-FIT-268 | Create Handouts in Word | W | Host seam | 10 | Writes into Word. A sister-app seam to OpenDoc, otherwise PDF handouts meet the need. | Documented (listing): [Handouts in Word](https://support.microsoft.com/en-us/office/edit-or-print-powerpoint-handouts-in-word-09256fc0-3b3e-4a43-9c83-3cfde594fed4) |
| PP-FIT-269 | Insert PDF content: as picture or embedded object | W; not Web | Edit | 6 | Microsoft's methods are a screen clipping, or Object from file. Neither parses the PDF, so we add no PDF parser. Displaying a PDF's pages needs the `Insert Object` row. | Documented: [Insert PDF content](https://support.microsoft.com/en-gb/powerpoint/insert-pdf-file-content-into-a-powerpoint-presentation) |
| PP-FIT-270 | Import from other applications | W | Edit | 6 | Via paste and Object rows. | Documented (listing): [Import from other applications](https://support.microsoft.com/en-us/powerpoint/import-content-from-other-applications-into-powerpoint) |
| PP-FIT-271 | Share link, invite | W M Web | Host seam | 10 (`ENT-02`) | `ENT-02`: sharing belongs to the host. Button and flow are ours. | Documented: [Share](https://support.microsoft.com/en-us/PowerPoint/training/share-your-powerpoint-presentation-with-others), [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint) |
| PP-FIT-272 | Link to a slide | W | Edit | 9 | Host resolves the URL form. | Documented: [New in 2021](https://support.microsoft.com/en-gb/office/what-s-new-in-powerpoint-2021-for-windows-dac3eb1b-483b-4043-a0c9-5c01c2c1a8f2) |
| PP-FIT-273 | Email as attachment | W | Host seam | 10 | Host mail, or `mailto:` without a file. Listed in the Share page; depth not read. | Documented (listing): [Share](https://support.microsoft.com/en-us/PowerPoint/training/share-your-powerpoint-presentation-with-others) |

### Collaboration and review

| ID | Feature | PowerPoint has it on | Disposition | Phase or tier | Reason or note | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| PP-FIT-274 | Comments: add, edit, delete, resolve | W M Web | Edit | 1M, 9 |  | Documented: [Compare features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b), [Comments](https://support.microsoft.com/en-us/PowerPoint/training/add-change-hide-or-delete-comments-in-a-presentation), [Modern comments](https://support.microsoft.com/en-us/powerpoint/modern-comments-in-powerpoint) |
| PP-FIT-275 | Legacy comments in old decks | W M | Edit | 1M, 9 | Both forms round-trip (48). | Documented: [Modern comments](https://support.microsoft.com/en-us/powerpoint/modern-comments-in-powerpoint) |
| PP-FIT-276 | @mentions | W M Web | Edit | 9 | People lookup comes from the host. | Documented: [Compare features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b) |
| PP-FIT-277 | Comment reactions | W M 2024 | Edit | 9 | `x:powerpoint/2022/03/main` is in the format. | Documented: [New in 2024](https://support.microsoft.com/en-us/office/what-s-new-in-powerpoint-2024-for-windows-and-mac-4330d63f-ffbb-470d-bbc2-a5f340317fbc) |
| PP-FIT-278 | Real-time co-authoring, presence | W M Web | Edit | 9 |  | Documented: [Compare features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b), [Work together](https://support.microsoft.com/en-us/powerpoint/work-together-on-powerpoint-presentations), [New in 2021](https://support.microsoft.com/en-gb/office/what-s-new-in-powerpoint-2021-for-windows-dac3eb1b-483b-4043-a0c9-5c01c2c1a8f2) |
| PP-FIT-279 | Revision highlighting | W M; limited Web | Edit | 1M, 9 |  | Documented: [Compare features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b), [Document Inspector](https://support.microsoft.com/en-us/office/collab-files/remove-hidden-data-and-personal-information-by-inspecting-documents-presentations-or-workbooks) |
| PP-FIT-280 | Recent-activity notifications | W M; not Web | Edit | 9 |  | Documented: [Compare features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b) |
| PP-FIT-281 | Version history | W M; cloud files | Host seam | 9 (`ENT-09`) | `ENT-09`. The host stores versions; our UI browses and restores. | Documented: [Restore a version](https://support.microsoft.com/en-gb/office/restore-a-previous-version-of-a-file-without-unwanted-changes-bdb2cafa-d588-475c-97d7-20e8b9949b84) |
| PP-FIT-282 | Compare and Merge | retired from W 2502; never M or Web | Refuse | n/a | Microsoft's page retires it from version 2502 and recommends co-authoring. A clone of today's product does not need it. Change data in old decks is the next row. | Documented: [Track changes page](https://support.microsoft.com/en-us/powerpoint/track-changes-in-your-presentation) |
| PP-FIT-283 | Revision and merge data in old decks | ? | Retain | 1M | Spec `REVIEW` is Retain, and Inspector removes it. | Observed: `REVIEW` in [spec/README](spec/README.md) |

### Protection, compliance, accessibility

| ID | Feature | PowerPoint has it on | Disposition | Phase or tier | Reason or note | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| PP-FIT-284 | Password to open: open a deck | W M; Web needs desktop | Edit | 10 (`CFB-01`, `ENT-05`) | Better than the web product, by `DEC-06`. Decrypt in the client (07). | Documented: [Password protection](https://support.microsoft.com/en-us/powerpoint/password-protection-for-presentations-in-powerpoint), [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint) |
| PP-FIT-285 | Password to open: set on save | W M | Edit | 10 | Lost passwords cannot be recovered; say so, as Microsoft does. | Documented: [Password protection](https://support.microsoft.com/en-us/powerpoint/password-protection-for-presentations-in-powerpoint) |
| PP-FIT-286 | Password to modify | W M | Edit | 1B, 10 | `p:modifyVerifier`. Not encryption: anyone can open it and save under another name, as Microsoft states. We ask for it, open read-only without it, and do not claim security. | Documented: [Password protection](https://support.microsoft.com/en-us/powerpoint/password-protection-for-presentations-in-powerpoint) |
| PP-FIT-287 | Mark as Final | W; Web cannot open | Edit | 10 | Status Final, read-only, marks off; anyone can undo. **Not a security feature**, Microsoft says, and neither is ours. Stored form not checked. | Documented: [Mark as Final](https://support.microsoft.com/en-us/office/collab-files/help-prevent-changes-to-a-final-version-of-a-file), [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint) |
| PP-FIT-288 | Restrict Access (IRM) | W; Web needs desktop | Host seam | 10 (`ENT-06`) | `ENT-06`: refuses clearly and keeps bytes, unless a host rights service exists. No Microsoft key service is ours to reach. | Documented: [Restrict changes](https://support.microsoft.com/en-us/powerpoint/restrict-changes-to-powerpoint-presentations), [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint) |
| PP-FIT-289 | Sensitivity label: read, show, keep | W M iOS Web | Edit | 1M, 10 (`ENT-04`) | Marks on slides render. Part list unknown: `SPEC-04`. | Documented: [Sensitivity labels](https://support.microsoft.com/en-us/office/security-privacy/apply-sensitivity-labels-to-your-files) |
| PP-FIT-290 | Sensitivity label: apply, mandatory, default, auto | W M iOS; custom permissions not on Web | Host seam | 10 (`ENT-04`) | Labels and policy live in the tenant. Host supplies the list and enforces no-download and no-print through `ENT-08`. | Documented: [Sensitivity labels](https://support.microsoft.com/en-us/office/security-privacy/apply-sensitivity-labels-to-your-files) |
| PP-FIT-291 | Dynamic watermark from a label | W M | Host seam | 10 (`ENT-04`, `ENT-08`) | Viewer-specific, set by the tenant; host policy. | Documented: [Sensitivity labels](https://support.microsoft.com/en-us/office/security-privacy/apply-sensitivity-labels-to-your-files) |
| PP-FIT-292 | Digital signature: keep, warn, drop on edit | W M; Web needs desktop | Retain | 1A, 1N (`WR-04`) | Spec `DSIG` is Retain. Editing invalidates it, so it is dropped with notice (07). | Documented: [Digital signature](https://support.microsoft.com/en-us/topic/70d26dc9-be10-46f1-8efa-719c8b3f1a2d), [Features on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint) |
| PP-FIT-293 | Digital signature: add, validate | W | Host seam | 10 | Needs a certificate store and trust roots. | Documented (listing): [Digital signature](https://support.microsoft.com/en-us/topic/70d26dc9-be10-46f1-8efa-719c8b3f1a2d), [Signatures and certificates](https://support.microsoft.com/en-us/topic/8186cd15-e7ac-4a16-8597-22bd163e8e96) |
| PP-FIT-294 | Document Inspector | W; others not stated | Edit | 10 | Finds and removes comments, properties, revision data, invisible and off-slide content, notes, custom XML. Reports, without removing, embedded objects and macros. Must also catch stale thumbnails (`SEC-014`). | Documented: [Document Inspector](https://support.microsoft.com/en-us/office/collab-files/remove-hidden-data-and-personal-information-by-inspecting-documents-presentations-or-workbooks), [Package for CD](https://support.microsoft.com/en-us/topic/ac1ae03e-c5f2-4a0d-8bcb-3c48741117c9) |
| PP-FIT-295 | Remove personal information on save | W | Edit | 10 |  | Documented: [Document Inspector](https://support.microsoft.com/en-us/office/collab-files/remove-hidden-data-and-personal-information-by-inspecting-documents-presentations-or-workbooks) |
| PP-FIT-296 | Accessibility Checker, in the background | W M | Edit | 4 (`A11Y-01`) |  | Documented: [Accessibility Checker](https://support.microsoft.com/en-us/accessibility/office-accessibility/improve-accessibility-with-the-accessibility-checker), [New in 2021](https://support.microsoft.com/en-gb/office/what-s-new-in-powerpoint-2021-for-windows-dac3eb1b-483b-4043-a0c9-5c01c2c1a8f2), [Accessible presentations](https://support.microsoft.com/en-us/accessibility/powerpoint/make-your-powerpoint-presentations-accessible-to-people-with-disabilities) |
| PP-FIT-297 | Reading order pane | W | Edit | 4 (`A11Y-01`) |  | Documented: [New in 2021](https://support.microsoft.com/en-gb/office/what-s-new-in-powerpoint-2021-for-windows-dac3eb1b-483b-4043-a0c9-5c01c2c1a8f2), [Accessible presentations](https://support.microsoft.com/en-us/accessibility/powerpoint/make-your-powerpoint-presentations-accessible-to-people-with-disabilities) |
| PP-FIT-298 | Alt text | W M | Edit | 4 |  | Documented: [Accessible presentations](https://support.microsoft.com/en-us/accessibility/powerpoint/make-your-powerpoint-presentations-accessible-to-people-with-disabilities) |
| PP-FIT-299 | Automatic alt text | W | Host seam | 10 | Vision model. Not on a page read in detail. | Inferred: recall; `a11y` page covers alt text in general |
| PP-FIT-300 | Accessibility ribbon, tools | W M 2024 | Edit | 4 |  | Documented: [New in 2024](https://support.microsoft.com/en-us/office/what-s-new-in-powerpoint-2024-for-windows-and-mac-4330d63f-ffbb-470d-bbc2-a5f340317fbc), [Accessibility tools](https://support.microsoft.com/en-us/accessibility/powerpoint/accessibility-tools-for-powerpoint) |
| PP-FIT-301 | Accessible PDF | W M 2024 | Edit | 2G | `PDF-01`. | Documented: [New in 2024](https://support.microsoft.com/en-us/office/what-s-new-in-powerpoint-2024-for-windows-and-mac-4330d63f-ffbb-470d-bbc2-a5f340317fbc) |

### Application shell

| ID | Feature | PowerPoint has it on | Disposition | Phase or tier | Reason or note | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| PP-FIT-302 | Customise the ribbon, export settings | W M | Edit | 10 | Per-user. Host may hide commands (`ENT-17`). | Documented: [Customize the ribbon](https://support.microsoft.com/en-us/office/foundations-experiences/customize-the-ribbon-in-office) |
| PP-FIT-303 | Quick Access Toolbar | W M | Edit | 3 |  | Documented: [Quick Access Toolbar](https://support.microsoft.com/en-us/office/customize-the-quick-access-toolbar) |
| PP-FIT-304 | Customise shortcuts | W M; not Web | Edit | 10 |  | Documented: [Compare features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b) |
| PP-FIT-305 | Dark mode, Office theme | ? | Edit | 3 | OpenCalc has token spine for this. Searches did not return Microsoft's page. | Inferred: recall; not on a page read |
| PP-FIT-306 | Touch mode | W | Edit | 10 (`MOB-01`) |  | Documented (listing): [Touch mode](https://support.microsoft.com/en-us/word/turn-touch-mode-on-or-off) |
| PP-FIT-307 | Recent files, pinned | W | Edit | 3 | Local on desktop; host list on web. | Documented: [Options: Advanced](https://support.microsoft.com/en-us/powerpoint/powerpoint-options-advanced) |
| PP-FIT-308 | Advanced options set | W | Edit | 3, 4 | Each option maps to a row or gets one in 50. Not each is copied. | Documented: [Options: Advanced](https://support.microsoft.com/en-us/powerpoint/powerpoint-options-advanced) |
| PP-FIT-309 | Disable slide show hardware acceleration | W | Refuse | n/a | Native graphics setting, with no browser equivalent. | Documented: [Options: Advanced](https://support.microsoft.com/en-us/powerpoint/powerpoint-options-advanced) |

### Mobile (`MOB-01`)

| ID | Feature | PowerPoint has it on | Disposition | Phase or tier | Reason or note | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| PP-FIT-310 | Responsive phone shell: view, present, light edit | iOS, Android apps | Edit | 10 (`MOB-01`) | Decided: a responsive shell after Phase 7, as OpenCalc's. Microsoft's apps play all animation but insert fewer effects, so phone verbs are a subset by design, with no loss on save. | Documented: [Animations on mobile](https://support.microsoft.com/en-us/office/add-animation-effects-in-powerpoint-on-a-mobile-device-f8bbbb00-06dc-4b3c-8c0a-7e7f17df6e42), [Mobile apps](https://support.microsoft.com/en-us/office/foundations-experiences/microsoft-365-mobile-apps) |
| PP-FIT-311 | Tablet editing | iPad, Android | Edit | 10 (`MOB-01`) | Free tier ends above 10.1 inches at Microsoft; no such rule here. | Documented: [Mobile apps](https://support.microsoft.com/en-us/office/foundations-experiences/microsoft-365-mobile-apps), [Android tablets help](https://support.microsoft.com/en-us/office/powerpoint-for-android-tablets-help-2ada1d22-3784-4943-bc47-9d1ede42875c) |
| PP-FIT-312 | Native iOS and Android apps | yes | Refuse | n/a | `DEC-06` names web and desktop; `MOB-01` chose a responsive shell. Owner can reverse this cheaply: the shell keeps working. | Documented: [iPhone features](https://support.microsoft.com/en-us/office/powerpoint-for-iphone-features-47fcaa07-cfa6-417e-b734-8761806b4078), [Android phones help](https://support.microsoft.com/en-us/office/powerpoint-for-android-phones-help-f6714e00-0ee2-48d1-bd3d-e1997565861f) |
| PP-FIT-313 | Presenter view on phone | iOS | Edit | 10 (`MOB-01`) |  | Documented: [iPhone features](https://support.microsoft.com/en-us/office/powerpoint-for-iphone-features-47fcaa07-cfa6-417e-b734-8761806b4078) |
| PP-FIT-314 | Morph and Designer on mobile | subscribers | Edit | 10 (`MOB-01`) | Morph plays everywhere; Designer follows its own Host seam row. | Documented: [Morph](https://support.microsoft.com/en-us/powerpoint/training/use-the-morph-transition-in-powerpoint), [iPhone features](https://support.microsoft.com/en-us/office/powerpoint-for-iphone-features-47fcaa07-cfa6-417e-b734-8761806b4078) |
| PP-FIT-315 | Ink annotations saved on phone | Android | Edit | 10 (`MOB-01`) |  | Documented: [Android phones help](https://support.microsoft.com/en-us/office/powerpoint-for-android-phones-help-f6714e00-0ee2-48d1-bd3d-e1997565861f) |
| PP-FIT-316 | Insert phone pictures, camera | W via Android; iOS | Edit | 10 (`MOB-01`) | File input and camera on phone. The Windows-to-Android hand-off is Microsoft's own and is not copied. | Documented: [New in 2024](https://support.microsoft.com/en-us/office/what-s-new-in-powerpoint-2024-for-windows-and-mac-4330d63f-ffbb-470d-bbc2-a5f340317fbc), [Pictures from a mobile device](https://support.microsoft.com/en-us/office/foundations-experiences/add-pictures-to-a-file-by-using-your-mobile-device) |
| PP-FIT-317 | IRM on mobile | Android | Host seam | 10 (`ENT-06`) | Same seam as desktop. | Documented: [Android phones help](https://support.microsoft.com/en-us/office/powerpoint-for-android-phones-help-f6714e00-0ee2-48d1-bd3d-e1997565861f) |
| PP-FIT-318 | Copilot on mobile | partly | Host seam | 10 (`DEC-05`) | Same seam. Microsoft's page contradicts itself on phones. | Documented: [Copilot on mobile](https://support.microsoft.com/en-us/office/copilot-in-powerpoint-for-mobile-devices-7b3ce1ab-cfe4-47e8-a157-cecadbf0fefb) |

## Parity today

**Status: audit, 2026-11-09. Task `FIT-02`, re-audited on `main` at `6bf7e2c` (`PAR-01`), counts kept current by each `PAR` row since.** The owner, 2026-11-08: many PowerPoint features are missing, and every gap must be found. This section marks each of the 318 rows above, and 34 more that the ribbon, menus, panes, keys and file operations have and the rows above miss (`PP-FIT-319` to `352`), with what the editor does **today**. The first audit was of `main` at `7539ea5` (35 of 352 working, 9.9%); the re-audit clicked through what merged since, and each later row's change names the `PAR` row that made it. The rows above stay the plan; this table is the state.

**How it was checked.** The live editor's code was read (`webapp/`, `crates/casual-slide-wasm`), the engine built with `wasm-pack`, and the editor served on `127.0.0.1` and driven in Chrome: the command registry read at run time (242 commands, 120 of them registered disabled with "Not available yet in this build"), real decks opened (`sdk-strict-animclr`, `pptxts-modern-comments`), shapes inserted and the Arrange menu opened with a multiple selection. A Works mark rests on the registry and code, and the decisive verbs were clicked; nothing was compared against a running PowerPoint (`UX-002`, `PV-01`).

| Today | Meaning |
| --- | --- |
| **Works** | The user can do it, end to end, and it saves |
| **Partial** | Some of it works; the row says what is missing |
| **Disabled** | Its button or menu item is present and greyed out, saying it is not available yet (owner instruction 46) |
| **Faked** | Looks enabled and does nothing, or pretends. **None found:** gallery tiles this build cannot write (Morph, Reveal, Float In…) say so in the status bar and change nothing |
| **Absent** | No control at all |

**Rank** is the order a PowerPoint user meets the gap: 1 everyday editing, 2 formatting, 3 review and sharing, 4 advanced. "Decided: not built" marks Refuse and Host seam rows, which stay as the table above decides them. **Backlog** names the [14](14-EXECUTION-TRACKER.md) row that closes it; `PAR-01` to `PAR-66` are ranked in the same order.

### Counts

| | Works | Partial | Disabled | Faked | Absent | Total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| The 318 rows above | 43 (13.5%) | 46 (14.5%) | 52 (16.4%) | 0 | 177 (55.7%) | 318 |
| With the 34 shell rows | 55 (15.6%) | 53 (15.1%) | 59 (16.8%) | 0 | 185 (52.6%) | 352 |
| Of those, the 280 we build (Edit, Render, Retain) | 55 (19.6%) | 52 (18.6%) | 48 (17.1%) | 0 | 125 (44.6%) | 280 |

**Plainly: of the 280 features we mean to build, 55 work (19.6%), 52 work in part (18.6%), and 173 are missing (61.8%),** whether drawn disabled or not there at all. **Of all 352, 55 work (15.6%).** Of the 55 that work, 11 are file facts kept on save rather than verbs a user runs.

**Largest gaps a PowerPoint user hits first:** no Group; no Find or Replace; no links; no change of layout; no Slide Sorter; no sections; slides cannot be copied between browser tabs; no z-order; no alt text.

**Re-audit, 2026-11-09 (`PAR-01`).** Merged after `7539ea5` and clicked in Chrome on `127.0.0.1` with `pptxts-modern-comments.pptx`: the File tab (`FILE-01`): New, Open, Info with editable properties, Save As with `.pptx`, `.potx` and `.ppsx`, Print, and rename in the title bar; autosave and the recovery bar (`SAVE-01`), a draft seen written after quiet; the Comments pane (`UX-011`), a deck's thread shown, a reply posted, saved and reopened; notes typed, saved and reopened; the Selection Pane; a hidden slide's struck, dimmed thumbnail; the lists, selection, rich paste, keys, slide copy and smart guides rows `PAR-06` and `PAR-10` to `PAR-15` had already updated. Found and fixed in the same pass: New Slide failed on every deck that had a slide (the engine's `i64` layout reached the page as a BigInt), and closing the tab gave no warning. The ribbon session's Arrange, Find, Outline, Slide Sorter, WordArt and Symbol were not on `main` and are not counted.

### The table

| ID | Feature | Disposition | Today | What is missing | Rank | Backlog |
| --- | --- | --- | --- | --- | --- | --- |
| PP-FIT-001 | Open and save `.pptx` | Edit | Works | Opens in the browser by picker or drop; Save and Save a Copy download it, byte-identical when unedited (`VS-04`); edits patch named nodes. No save in place: every save is a download |  |  |
| PP-FIT-002 | Strict Open XML `.pptx` | Edit | Works | Opens normalised; the Strict fixture opens and draws |  |  |
| PP-FIT-003 | PowerPoint Show `.ppsx` | Edit | Partial | Opens in the editor, not as a show; Save As writes `.ppsx` (`FILE-01`) | 4 advanced | `PAR-57` |
| PP-FIT-004 | Template `.potx`: open, make a deck, save as | Edit | Partial | A `.potx` opens as a deck and Save As writes `.potx` (`FILE-01`); no "new deck from this template" | 4 advanced | `PAR-49` |
| PP-FIT-005 | Macro-enabled files `.pptm`, `.ppsm`, `.potm`: open and edit the deck | Edit | Partial | `.pptm`, `.ppsm`, `.potm` open and edit; the macro notice docs/96 specifies is not shown | 3 review and sharing | `PAR-47` |
| PP-FIT-006 | PDF export | Edit | Works | File, Export, Download as PDF: every slide, searchable text |  |  |
| PP-FIT-007 | XPS export | Refuse | Absent | Refused by decision | decided: not built |  |
| PP-FIT-008 | Office Theme `.thmx`: save and apply | Edit | Absent | No `.thmx` save or browse-for-theme | 4 advanced | `PAR-49` |
| PP-FIT-009 | PowerPoint XML Presentation `.xml` | Edit | Absent | Flat `.xml` not accepted by the picker | 4 advanced | `PAR-57` |
| PP-FIT-010 | Slide as PNG, JPEG, GIF, TIFF, BMP | Edit | Partial | PNG of the current slide only (1920 px); no JPEG, GIF, TIFF, BMP, no all-slides export | 4 advanced | `PAR-57` |
| PP-FIT-011 | Slide as WMF or EMF | Refuse | Absent | Refused by decision | decided: not built |  |
| PP-FIT-012 | Picture Presentation (each slide a picture) | Edit | Absent | No picture-presentation export | 4 advanced | `PAR-57` |
| PP-FIT-013 | Outline as RTF: save | Edit | Absent | No outline RTF export | 4 advanced | `PAR-57` |
| PP-FIT-014 | OpenDocument `.odp`: open and save | Edit | Absent | `.odp` neither opens nor saves | 4 advanced | `PAR-57` |
| PP-FIT-015 | Web Page `.htm`, Single File Web Page `.mht`: save | Refuse | Absent | Refused by decision | decided: not built |  |
| PP-FIT-016 | Web publish settings in the file | Retain | Works | Kept by the unedited save and node patching; not separately clicked |  |  |
| PP-FIT-017 | Legacy `.ppt`, `.pps`: open | Edit | Absent | `.ppt` refused (`DEC-08`, Phase 10) | 4 advanced | `PAR-57` |
| PP-FIT-018 | Legacy `.ppt`: save | Refuse | Absent | Refused by decision | decided: not built |  |
| PP-FIT-019 | Pack and Go `.ppz` | Refuse | Absent | Refused by decision | decided: not built |  |
| PP-FIT-020 | Compatibility Checker | Edit | Absent | No Compatibility Checker | 4 advanced | `PAR-65` |
| PP-FIT-021 | Compatibility mode for old files | Edit | Absent | No compatibility-mode banner or convert | 4 advanced | `PAR-65` |
| PP-FIT-022 | Document properties (File, Info) | Edit | Works | File, Info shows Slides and Words and edits Title, Subject, Author, Tags, Categories, Comments, Company, Manager and custom properties, each one undo step, saved in `docProps` (`FILE-01`) |  |  |
| PP-FIT-023 | AutoSave | Edit | Partial | The deck is drafted to this browser's IndexedDB after 5 s of quiet, at most 60 s apart and when the tab hides (`SAVE-01`); the file itself is never saved in place, as PowerPoint's AutoSave saves the cloud file | 1 everyday editing | `SAVE-01` |
| PP-FIT-024 | AutoRecover | Edit | Works | On the next open a bar offers Restore, Download, Discard; the undo history is not recovered (`SAVE-09`) |  |  |
| PP-FIT-025 | Open up to 2 GB | Edit | Partial | Opens in-browser; no measured limit (`CAP-01`) or stated maximum in the UI | 4 advanced | `PAR-65` |
| PP-FIT-026 | VBA project: keep in the file | Retain | Works | VBA part kept byte-for-byte by the save path; not separately clicked |  |  |
| PP-FIT-027 | Run a macro | Refuse | Absent | Refused by decision | decided: not built |  |
| PP-FIT-028 | Trust Center macro settings | Refuse | Absent | Refused by decision | decided: not built |  |
| PP-FIT-029 | Developer tab, VBA editor, record macro | Refuse | Absent | Refused by decision | decided: not built |  |
| PP-FIT-030 | Action: Run macro | Retain | Partial | Kept in the file; in a show a click does nothing, but the line saying why (`SEC-N04`) is not shown | 3 review and sharing | `PAR-47` |
| PP-FIT-031 | Action: Run program | Retain | Partial | As 030 | 3 review and sharing | `PAR-47` |
| PP-FIT-032 | ActiveX controls in a slide | Retain | Works | Kept in the file; not drawn |  |  |
| PP-FIT-033 | PowerPoint add-in `.ppam`: load and run | Refuse | Absent | Refused by decision | decided: not built |  |
| PP-FIT-034 | COM add-ins | Refuse | Absent | Refused by decision | decided: not built |  |
| PP-FIT-035 | Office Add-ins (store, My Add-ins, task pane) | Host seam | Disabled | Insert, Add-ins drawn disabled; host seam not built | decided: not built |  |
| PP-FIT-036 | Add-in parts stored in a deck | Retain | Works | Kept in the file |  |  |
| PP-FIT-037 | Embedded OLE object: show | Render | Partial | Fallback picture drawn where the engine reaches it; not confirmed on a real OLE deck | 4 advanced | `PAR-65` |
| PP-FIT-038 | Embedded OLE object: double-click to edit | Refuse | Absent | Refused by decision | decided: not built |  |
| PP-FIT-039 | Embedded Excel or Word object: edit | Host seam | Absent | Host seam, not built | decided: not built |  |
| PP-FIT-040 | Insert Object, from file, embed | Edit | Absent | No Insert Object | 4 advanced | `PAR-60` |
| PP-FIT-041 | Insert Object, from file, link; Edit Links | Host seam | Absent | Host seam, not built | decided: not built |  |
| PP-FIT-042 | Smart tags | Retain | Works | Kept in the file |  |  |
| PP-FIT-043 | Tags, customer data, custom XML | Retain | Works | Kept in the file |  |  |
| PP-FIT-044 | Script links | Retain | Works | Kept in the file |  |  |
| PP-FIT-045 | Add, rearrange, duplicate, delete slides | Edit | Works | New Slide (Ctrl+M), Duplicate, Delete, Hide, drag to reorder in thumbnails, all undoable |  |  |
| PP-FIT-046 | Layouts: choose, reset | Edit | Partial | New Slide from Layout picks from the deck's layouts; Layout (change an existing slide's layout) and Reset Slide are disabled | 1 everyday editing | `PAR-07` |
| PP-FIT-047 | Slide size and orientation | Edit | Works | Design, Slide Size: Standard, Widescreen, Custom, with Maximize or Ensure Fit |  |  |
| PP-FIT-048 | Mixed orientation in one deck | Refuse | Absent | Refused by decision | decided: not built |  |
| PP-FIT-049 | Sections | Edit | Disabled | Section menu present, every item disabled; sections in a file are not shown in the thumbnail pane | 1 everyday editing | `PAR-09` |
| PP-FIT-050 | Hide slide; reveal in show | Edit | Works | Hide Slide works and the show skips it; the thumbnail's number is struck through and its picture dimmed (`PAR-16`) |  |  |
| PP-FIT-051 | Reuse slides, keep source formatting | Edit | Disabled | Reuse Slides is in New Slide's menu, disabled | 4 advanced | `PAR-66` |
| PP-FIT-052 | Reuse slides from cloud storage | Host seam | Absent | Host seam, not built | decided: not built |  |
| PP-FIT-053 | Slide update link metadata | Retain | Works | Kept in the file |  |  |
| PP-FIT-054 | Slide Library: publish, reuse from library | Refuse | Absent | Refused by decision | decided: not built |  |
| PP-FIT-055 | Slides from Outline (`.docx`, `.rtf`, `.txt`) | Edit | Absent | No Slides from Outline | 4 advanced | `PAR-66` |
| PP-FIT-056 | Normal view | Edit | Works | Thumbnails, stage, notes pane, status bar |  |  |
| PP-FIT-057 | Outline view | Edit | Disabled | View, Outline View disabled | 2 formatting | `PAR-32` |
| PP-FIT-058 | Slide Sorter view | Edit | Disabled | View, Slide Sorter and its status-bar button disabled | 1 everyday editing | `PAR-08` |
| PP-FIT-059 | Notes Page view | Edit | Disabled | View, Notes Page disabled | 4 advanced | `PAR-62` |
| PP-FIT-060 | Reading view | Edit | Works | View, Reading View, from the ribbon and the status bar; Esc returns |  |  |
| PP-FIT-061 | Slide Master, Handout Master, Notes Master views | Edit | Disabled | View, Slide Master disabled; no Handout or Notes Master | 4 advanced | `PAR-49` |
| PP-FIT-062 | Grayscale and Pure Black and White view | Edit | Disabled | View, Grayscale disabled | 4 advanced | `PAR-62` |
| PP-FIT-063 | Speaker notes pane | Edit | Works | Notes are typed under the slide, the notes slide created when there is none, saved and reopened (`UX-011`) |  |  |
| PP-FIT-064 | Ruler | Edit | Disabled | View, Ruler disabled | 2 formatting | `PAR-31` |
| PP-FIT-065 | Gridlines, snap to grid | Edit | Disabled | View, Gridlines disabled; no snap to grid | 2 formatting | `PAR-31` |
| PP-FIT-066 | Drawing guides | Edit | Disabled | View, Guides disabled | 2 formatting | `PAR-31` |
| PP-FIT-067 | Smart guides | Edit | Works | Smart guides while moving (`PAR-15`): edges and centres snap to other shapes' and the slide's, red dashed guides; Alt turns it off. Spacing guides and resize snapping not yet |  |  |
| PP-FIT-068 | Zoom, Fit | Edit | Works | Zoom menu, slider, percentage, Fit to Window |  |  |
| PP-FIT-069 | New Window, Arrange All, Cascade | Refuse | Absent | Refused by decision | decided: not built |  |
| PP-FIT-070 | Open in a chosen view | Edit | Absent | Always opens in Normal view; stored view not honoured | 4 advanced | `PAR-62` |
| PP-FIT-071 | Saved view state in the file | Edit | Absent | `viewProps` kept but not read or written back from the UI | 4 advanced | `PAR-62` |
| PP-FIT-072 | Shapes gallery (187 presets) | Edit | Works | Shapes gallery with every engine preset in PowerPoint's groups; click or drag to draw; adjust handles |  |  |
| PP-FIT-073 | Text box | Edit | Works | Insert Text Box, click to type |  |  |
| PP-FIT-074 | Freeform, Scribble, Edit Points | Edit | Disabled | Edit Points is in Edit Shape and the right-click menu, disabled; no Freeform or Scribble tool | 4 advanced | `PAR-58` |
| PP-FIT-075 | Merge Shapes: Union, Combine, Fragment, Intersect, Subtract | Edit | Absent | No Merge Shapes | 4 advanced | `PAR-58` |
| PP-FIT-076 | Connectors, glued to sites | Edit | Partial | Connectors draw point to point; not glued to connection sites, so they do not follow moved shapes | 2 formatting | `PAR-28` |
| PP-FIT-077 | Align, distribute | Edit | Works | Align left, centre, right, top, middle, bottom; distribute; one undo step |  |  |
| PP-FIT-078 | Group, ungroup | Edit | Disabled | Group and Ungroup (Ctrl+G) disabled; the keys do nothing | 1 everyday editing | `PAR-03` |
| PP-FIT-079 | Rotate, flip | Edit | Partial | Rotate by handle and in Size & Position; Rotate Right 90° and Flip Horizontal disabled; no Flip Vertical | 2 formatting | `PAR-24` |
| PP-FIT-080 | Select individual objects | Edit | Partial | Click, Shift or Ctrl-click, marquee, Tab; cannot select a shape inside a group | 1 everyday editing | `PAR-03` |
| PP-FIT-081 | Selection pane | Edit | Works | Selection Pane lists the slide's shapes front first; rename, show and hide, Show All, Hide All, drag to reorder; saves the name, `@hidden` and the order (`UX-011`) |  |  |
| PP-FIT-082 | Format Painter | Edit | Disabled | Format Painter disabled; no Ctrl+Shift+C/V | 2 formatting | `PAR-26` |
| PP-FIT-083 | Eyedropper | Edit | Absent | No eyedropper in colour menus | 2 formatting | `PAR-27` |
| PP-FIT-084 | Colours dialog, hex input | Edit | Partial | More Colors opens the browser's colour picker; no PowerPoint Colors dialog with hex, RGB, HSL and transparency | 2 formatting | `PAR-27` |
| PP-FIT-085 | Fills, outlines, effects | Edit | Partial | Shape Fill solid or none, Shape Outline colour, weight, dash, arrows, four Shape Styles; gradient, picture, pattern fills and every effect (shadow, glow, reflection, soft edges, bevel) disabled | 2 formatting | `PAR-22` |
| PP-FIT-086 | Sketched outline style | Edit | Absent | No sketched outline | 4 advanced | `PAR-58` |
| PP-FIT-087 | Draw ruler | Edit | Absent | No Draw ruler | 4 advanced | `PAR-58` |
| PP-FIT-088 | Maximum number of undos | Edit | Partial | Undo and Redo work across edits; depth not set or stated, no history list | 4 advanced | `PAR-61` |
| PP-FIT-089 | Font, size, bold, italic and the rest | Edit | Works | Font, size, grow, shrink, bold, italic, underline, Strikethrough, Character Spacing and Change Case work and draw (`PAR-67`); the Font dialog (Ctrl+Shift+F, the group's launcher) sets font, style, size, colour, underline style, single and double strikethrough, superscript and subscript with an offset, and spacing, one undo step (`PAR-20`). No Small Caps, All Caps or Equalize Character Height in the dialog |  |  |
| PP-FIT-090 | Text colour | Edit | Works | Font Color with theme, standard and More Colors |  |  |
| PP-FIT-091 | Text highlighter | Edit | Works | Text Highlight Color, drawn (`UX-005`) |  |  |
| PP-FIT-092 | Superscript, subscript | Edit | Works | Superscript (Ctrl+Shift+=) and Subscript (Ctrl+=) toggle, pressed at the caret, saved as PowerPoint's `baseline` 30% and -25% and undone (`PAR-20`); drawn raised or lowered at two thirds of the size, the size **Inferred** (`PAR-67`); any offset from the Font dialog |  |  |
| PP-FIT-093 | Clear formatting | Edit | Works | Clear All Formatting (Ctrl+Space) removes every character property but language, links and proofing marks, one undo step (`PAR-20`) |  |  |
| PP-FIT-094 | Alignment, spacing, indents | Edit | Partial | Left, centre, right, justify (and keys); Line Spacing, indents and the Paragraph dialog disabled | 2 formatting | `PAR-21` |
| PP-FIT-095 | Bullets, numbering | Edit | Works | Bullets and Numbering galleries, Increase and Decrease List Level, Tab and Shift+Tab, Line Spacing (`PAR-06`) |  |  |
| PP-FIT-096 | Text columns | Edit | Disabled | Columns disabled | 2 formatting | `PAR-21` |
| PP-FIT-097 | Right-to-left text and direction | Edit | Absent | No right-to-left paragraph control; Arabic decks draw | 4 advanced | `PAR-64` |
| PP-FIT-098 | Vertical and rotated text | Edit | Disabled | Text Direction disabled | 2 formatting | `PAR-21` |
| PP-FIT-099 | AutoFit, shrink on overflow, resize shape | Edit | Partial | AutoFit stored in the file is drawn; no control to set it (Format Shape Text Options disabled) | 2 formatting | `PAR-23` |
| PP-FIT-100 | Split body text across slides | Edit | Absent | No split across slides | 4 advanced | `PAR-64` |
| PP-FIT-101 | WordArt, text warps | Edit | Disabled | Insert WordArt and the Text Fill, Outline and Effects menus disabled | 2 formatting | `PAR-29` |
| PP-FIT-102 | Wrap text around a circle | Edit | Absent | No text warp | 4 advanced | `PAR-58` |
| PP-FIT-103 | Symbols | Edit | Disabled | Insert Symbol disabled | 2 formatting | `PAR-30` |
| PP-FIT-104 | Hyperlinks, link colour | Edit | Disabled | Insert Link (Ctrl+K) disabled; links in a file are not clickable in a show | 1 everyday editing | `PAR-05` |
| PP-FIT-105 | Find | Edit | Disabled | Find (Ctrl+F) disabled; Ctrl+F reaches the browser's find | 1 everyday editing | `PAR-04` |
| PP-FIT-106 | Replace | Edit | Disabled | Replace (Ctrl+H) disabled | 1 everyday editing | `PAR-04` |
| PP-FIT-107 | Replace Fonts | Edit | Absent | No Replace Fonts | 4 advanced | `PAR-64` |
| PP-FIT-108 | Embed fonts (all or used characters) | Edit | Absent | No font embedding | 4 advanced | `PAR-64` |
| PP-FIT-109 | Cloud fonts | Host seam | Absent | Host seam, not built | decided: not built |  |
| PP-FIT-110 | Spelling | Host seam | Disabled | Spelling disabled; no red squiggles; host seam not built | decided: not built |  |
| PP-FIT-111 | Editor pane | Refuse | Absent | Refused by decision | decided: not built |  |
| PP-FIT-112 | Thesaurus | Host seam | Disabled | Thesaurus disabled; host seam not built | decided: not built |  |
| PP-FIT-113 | Proofing language per run | Edit | Disabled | Language disabled; run language kept but not settable | 4 advanced | `PAR-64` |
| PP-FIT-114 | AutoCorrect | Edit | Absent | No AutoCorrect (straight quotes stay straight, no auto-bullets) | 2 formatting | `PAR-41` |
| PP-FIT-115 | Translate | Host seam | Disabled | Translate disabled; host seam | decided: not built |  |
| PP-FIT-116 | Dictate | Host seam | Disabled | Dictate disabled; host seam | decided: not built |  |
| PP-FIT-117 | Immersive Reader, Speak | Host seam | Absent | Host seam, not built | decided: not built |  |
| PP-FIT-118 | Microsoft Search box | Edit | Partial | Search box in the title bar runs commands; no help or document search | 4 advanced | `PAR-61` |
| PP-FIT-119 | Insert pictures | Edit | Works | Insert, Pictures from this device; draws and saves |  |  |
| PP-FIT-120 | Online pictures | Host seam | Absent | Host seam, not built | decided: not built |  |
| PP-FIT-121 | Stock images, icons, cut-outs | Host seam | Absent | Host seam, not built | decided: not built |  |
| PP-FIT-122 | Icons | Edit | Disabled | Insert, Icons disabled | 2 formatting | `PAR-30` |
| PP-FIT-123 | Clip art | Host seam | Absent | Host seam, not built | decided: not built |  |
| PP-FIT-124 | Crop, delete cropped areas | Edit | Partial | Crop by handles, Reset Picture; no Crop to Aspect Ratio, Fill, Fit, no delete cropped areas | 2 formatting | `PAR-33` |
| PP-FIT-125 | Crop to shape | Edit | Absent | No Crop to Shape | 2 formatting | `PAR-33` |
| PP-FIT-126 | Compress Pictures | Edit | Absent | No Compress Pictures | 4 advanced | `PAR-60` |
| PP-FIT-127 | Discard editing data; image default resolution | Edit | Absent | No option | 4 advanced | `PAR-60` |
| PP-FIT-128 | Picture corrections, colour, styles | Edit | Partial | Four basic Picture Styles; no Corrections, Color, picture border or effects | 2 formatting | `PAR-33` |
| PP-FIT-129 | Artistic effects | Edit | Absent | No artistic effects | 4 advanced | `PAR-60` |
| PP-FIT-130 | Remove Background | Edit | Absent | No Remove Background | 4 advanced | `PAR-60` |
| PP-FIT-131 | Transparency | Edit | Absent | No picture or shape transparency control | 2 formatting | `PAR-22` |
| PP-FIT-132 | SVG: insert, recolour | Edit | Partial | SVG decks draw (fallback); inserting an `.svg` and recolouring not built | 4 advanced | `PAR-58` |
| PP-FIT-133 | Convert SVG to shape | Edit | Absent | No convert to shape | 4 advanced | `PAR-58` |
| PP-FIT-134 | Linked pictures | Retain | Works | Kept in the file |  |  |
| PP-FIT-135 | Picture link: insert | Host seam | Absent | Host seam, not built | decided: not built |  |
| PP-FIT-136 | Animated GIF: insert, play | Edit | Partial | GIF inserts as a still; does not play | 4 advanced | `PAR-60` |
| PP-FIT-137 | Insert screenshot, window list | Edit | Absent | No Insert Screenshot | 4 advanced | `PAR-60` |
| PP-FIT-138 | Do not auto-hyperlink screenshot | Edit | Absent | No screenshot, so no option | 4 advanced | `PAR-60` |
| PP-FIT-139 | Screen recording | Edit | Absent | No screen recording | 4 advanced | `PAR-53` |
| PP-FIT-140 | Photo album: create and edit | Edit | Absent | No Photo Album | 4 advanced | `PAR-60` |
| PP-FIT-141 | Watermark | Edit | Absent | No watermark verb | 4 advanced | `PAR-60` |
| PP-FIT-142 | Timeline, org chart from templates or SmartArt | Edit | Absent | No timeline or org chart from SmartArt | 4 advanced | `PAR-54` |
| PP-FIT-143 | Tables: insert, rows, columns, size | Edit | Partial | Insert Table by grid, insert and delete rows and columns, type in a cell through a field; no drag to resize rows or columns, no caret editing in a cell (`VS-20`) | 1 everyday editing | `VS-20` |
| PP-FIT-144 | Merge and split cells | Edit | Absent | No Merge or Split Cells | 2 formatting | `PAR-34` |
| PP-FIT-145 | Table styles, borders, shading | Edit | Partial | Six Medium Style 2 tiles from the deck's styles; no borders, shading, header-row or banded-row options | 2 formatting | `PAR-34` |
| PP-FIT-146 | Animate table parts | Edit | Absent | No table-part animation | 4 advanced | `PAR-66` |
| PP-FIT-147 | Paste Excel range: embed, picture, text | Edit | Absent | Pasting from Excel gives plain text only, in a text box | 4 advanced | `PAR-66` |
| PP-FIT-148 | Linked Excel chart or range: update | Host seam | Absent | Host seam, not built | decided: not built |  |
| PP-FIT-149 | Charts: classic families | Edit | Partial | Existing charts draw; Insert Chart offers column, bar, line, pie, and Edit Data edits values; no chart styles, elements, change type or formatting | 2 formatting | `PAR-35` |
| PP-FIT-150 | Charts: 2016+ (funnel and others) | Edit | Partial | 2016+ charts draw their fallback where stored; not insertable | 4 advanced | `PAR-66` |
| PP-FIT-151 | Map chart: geocoding | Host seam | Absent | Host seam, not built | decided: not built |  |
| PP-FIT-152 | SmartArt: create, edit, text pane | Edit | Disabled | Insert SmartArt disabled; stored SmartArt draws from its drawing part; not editable | 4 advanced | `PAR-54` |
| PP-FIT-153 | SmartArt: convert to shapes | Edit | Absent | No convert to shapes | 4 advanced | `PAR-54` |
| PP-FIT-154 | Equation: insert, gallery of structures | Edit | Disabled | Insert Equation disabled; stored equations draw only as their fallback | 4 advanced | `PAR-55` |
| PP-FIT-155 | Equation: linear, UnicodeMath, LaTeX | Edit | Absent | No equation input | 4 advanced | `PAR-55` |
| PP-FIT-156 | Equation: save to gallery | Edit | Absent | No equation gallery | 4 advanced | `PAR-55` |
| PP-FIT-157 | Old Equation Editor objects | Render | Partial | Drawn from the fallback picture where present | 4 advanced | `PAR-55` |
| PP-FIT-158 | 3D models: show | Edit | Partial | 3D models show their fallback picture only | 4 advanced | `PAR-56` |
| PP-FIT-159 | 3D models: insert, rotate, pan, zoom | Edit | Disabled | Insert 3D Models disabled | 4 advanced | `PAR-56` |
| PP-FIT-160 | 3D models: online library | Host seam | Absent | Host seam, not built | decided: not built |  |
| PP-FIT-161 | 3D animation effects | Edit | Absent | No 3D effects | 4 advanced | `PAR-56` |
| PP-FIT-162 | Shape 3D format and rotation | Edit | Absent | Shape 3D Format and Rotation disabled in Format Shape Effects | 4 advanced | `PAR-56` |
| PP-FIT-163 | Draw tab: pens, pencil, highlighter | Edit | Disabled | Draw tab: Select, Lasso, Eraser, Pen, Highlighter all disabled | 4 advanced | `PAR-51` |
| PP-FIT-164 | Ink effects, pen set | Edit | Absent | No ink | 4 advanced | `PAR-51` |
| PP-FIT-165 | Point, stroke and segment erasers | Edit | Disabled | Eraser disabled | 4 advanced | `PAR-51` |
| PP-FIT-166 | Lasso select | Edit | Disabled | Lasso Select disabled | 4 advanced | `PAR-51` |
| PP-FIT-167 | Automatic inking setting | Edit | Absent | No ink | 4 advanced | `PAR-51` |
| PP-FIT-168 | Use pen to select by default | Edit | Absent | No ink | 4 advanced | `PAR-51` |
| PP-FIT-169 | Ink to Shape | Edit | Disabled | Ink to Shape disabled | 4 advanced | `PAR-51` |
| PP-FIT-170 | Ink to Text | Host seam | Absent | Host seam, not built | decided: not built |  |
| PP-FIT-171 | Ink to Math | Host seam | Disabled | Ink to Math disabled; host seam | decided: not built |  |
| PP-FIT-172 | Ink Replay button | Edit | Disabled | Ink Replay disabled | 4 advanced | `PAR-51` |
| PP-FIT-173 | Replay and Rewind ink animations | Edit | Absent | No ink | 4 advanced | `PAR-51` |
| PP-FIT-174 | Digital pen button advances slides | Edit | Absent | No pen input in the show | 4 advanced | `PAR-51` |
| PP-FIT-175 | Transitions gallery, effect options | Edit | Partial | Fade, Push, Wipe, Split, Cut set and play; Morph, Reveal and the rest change nothing and say so; Preview and Effect Options disabled | 2 formatting | `PAR-36` |
| PP-FIT-176 | Transition speed and advance | Edit | Partial | Duration and Apply To All work; On Mouse Click and After disabled | 2 formatting | `PAR-36` |
| PP-FIT-177 | Morph | Edit | Partial | Morph tile present, writes nothing; a stored Morph plays as a fade | 4 advanced | `PAR-59` |
| PP-FIT-178 | Summary, Slide and Section Zoom | Edit | Absent | No Zoom (summary, slide, section) | 4 advanced | `PAR-59` |
| PP-FIT-179 | Animations gallery, classes | Edit | Partial | Appear, Fade, Fly In, Pulse, Disappear, Fade Out write and play; Float In, Split, Wipe, Zoom and all others do nothing and say so; Preview and Effect Options disabled | 2 formatting | `PAR-37` |
| PP-FIT-180 | Multiple effects on one object | Edit | Works | Add Animation adds after the others; the pane lists them |  |  |
| PP-FIT-181 | Animation Painter | Edit | Disabled | Animation Painter disabled | 2 formatting | `PAR-37` |
| PP-FIT-182 | Animation Pane, start, duration, delay | Edit | Partial | Animation Pane lists, reorders, deletes; Start works; Duration and Delay disabled | 2 formatting | `PAR-37` |
| PP-FIT-183 | Motion paths | Edit | Absent | No motion paths | 4 advanced | `PAR-59` |
| PP-FIT-184 | Trigger on click of an object | Edit | Disabled | Trigger disabled | 4 advanced | `PAR-59` |
| PP-FIT-185 | Trigger on media bookmark | Edit | Absent | No bookmark triggers | 4 advanced | `PAR-59` |
| PP-FIT-186 | Text by word, by line | Edit | Absent | No by-word or by-letter control | 4 advanced | `PAR-59` |
| PP-FIT-187 | Animate a group | Edit | Partial | A group animates as one in a show where the file has it; no verb | 4 advanced | `PAR-59` |
| PP-FIT-188 | Sound effects on animation and hyperlink | Edit | Absent | No animation sounds | 4 advanced | `PAR-59` |
| PP-FIT-189 | Remove or turn off animation | Edit | Partial | Remove Animation and the None tile work; no "turn off animation" show option (Set Up Show disabled) | 2 formatting | `PAR-37` |
| PP-FIT-190 | SmartArt animation | Edit | Absent | No SmartArt animation | 4 advanced | `PAR-54` |
| PP-FIT-191 | Audio: add, remove | Edit | Disabled | Insert Audio disabled | 4 advanced | `PAR-50` |
| PP-FIT-192 | Record audio on a slide | Edit | Absent | No audio recording | 4 advanced | `PAR-53` |
| PP-FIT-193 | Video from a file, embed | Edit | Disabled | Insert Video disabled; video in a deck shows its poster frame only and does not play | 4 advanced | `PAR-50` |
| PP-FIT-194 | Online video (YouTube and others) | Edit | Absent | No online video | 4 advanced | `PAR-50` |
| PP-FIT-195 | Stream and SharePoint video | Host seam | Absent | Host seam, not built | decided: not built |  |
| PP-FIT-196 | Trim, fade, bookmark | Edit | Absent | No Playback tab | 4 advanced | `PAR-50` |
| PP-FIT-197 | Play across slides, in background | Edit | Absent | No media playback | 4 advanced | `PAR-50` |
| PP-FIT-198 | Sound automatically on a slide | Edit | Absent | No media playback | 4 advanced | `PAR-50` |
| PP-FIT-199 | Poster frame, video styles | Edit | Absent | No Video Format tab | 4 advanced | `PAR-50` |
| PP-FIT-200 | Media formats accepted | Edit | Absent | No media | 4 advanced | `PAR-50` |
| PP-FIT-201 | Compress Media (1080p, 720p, 480p) | Edit | Absent | No media | 4 advanced | `PAR-50` |
| PP-FIT-202 | Optimize Media Compatibility | Edit | Absent | No media | 4 advanced | `PAR-50` |
| PP-FIT-203 | Save Media As | Edit | Absent | No media | 4 advanced | `PAR-50` |
| PP-FIT-204 | Media captions: insert WebVTT or SRT | Edit | Absent | No captions | 4 advanced | `PAR-50` |
| PP-FIT-205 | Media captions: generate by speech | Host seam | Absent | Host seam, not built | decided: not built |  |
| PP-FIT-206 | Media captions: translate | Host seam | Absent | Host seam, not built | decided: not built |  |
| PP-FIT-207 | From Beginning, From Current | Edit | Works | F5, Shift+F5, the Present button and the status bar; full screen |  |  |
| PP-FIT-208 | Set Up Show: presenter, individual, kiosk | Edit | Disabled | Set Up Slide Show disabled | 4 advanced | `PAR-52` |
| PP-FIT-209 | Self-running, loop until Esc | Edit | Absent | No kiosk or loop setting | 4 advanced | `PAR-52` |
| PP-FIT-210 | Custom show | Edit | Absent | No custom shows | 4 advanced | `PAR-52` |
| PP-FIT-211 | Presenter view | Edit | Works | Presenter View: current and next slide, notes, timer, count |  |  |
| PP-FIT-212 | Multiple monitors, swap displays | Edit | Partial | Presenter View opens a second window; no display swap or monitor choice | 4 advanced | `PAR-52` |
| PP-FIT-213 | Pen and highlighter in show | Edit | Absent | No pen or highlighter in the show | 4 advanced | `PAR-51` |
| PP-FIT-214 | Keep or discard ink on exit | Edit | Absent | No ink in the show | 4 advanced | `PAR-51` |
| PP-FIT-215 | Laser pointer | Edit | Absent | No laser pointer | 4 advanced | `PAR-51` |
| PP-FIT-216 | Hide ink, erase, hide pointer | Edit | Absent | No pointer options | 4 advanced | `PAR-51` |
| PP-FIT-217 | Black or white screen, go to slide | Edit | Works | B, W, number then Enter, Esc, as PowerPoint |  |  |
| PP-FIT-218 | Zoom in slide show, to 400% | Edit | Absent | No zoom in the show | 4 advanced | `PAR-52` |
| PP-FIT-219 | Action buttons, action settings: link, slide, sound | Edit | Disabled | Insert Action disabled; links and action buttons do nothing in a show | 1 everyday editing | `PAR-05` |
| PP-FIT-220 | Rehearse Timings | Edit | Absent | No Rehearse Timings | 4 advanced | `PAR-52` |
| PP-FIT-221 | Record narration, timings by slide | Edit | Absent | No recording | 4 advanced | `PAR-53` |
| PP-FIT-222 | Record camera video narration | Edit | Absent | No recording | 4 advanced | `PAR-53` |
| PP-FIT-223 | Record ink and laser gestures | Edit | Absent | No recording | 4 advanced | `PAR-53` |
| PP-FIT-224 | Teleprompter while recording | Edit | Absent | No recording | 4 advanced | `PAR-53` |
| PP-FIT-225 | Background blur for camera | Edit | Absent | No recording | 4 advanced | `PAR-53` |
| PP-FIT-226 | Cameo: live camera on a slide | Edit | Disabled | Insert Cameo disabled | 4 advanced | `PAR-53` |
| PP-FIT-227 | Cameo effects from Teams | Host seam | Absent | Host seam, not built | decided: not built |  |
| PP-FIT-228 | Reset recording to Cameo | Edit | Absent | No recording | 4 advanced | `PAR-53` |
| PP-FIT-229 | Export recording to video | Edit | Absent | No recording | 4 advanced | `PAR-53` |
| PP-FIT-230 | Export to video: MP4 | Edit | Absent | No video export | 4 advanced | `PAR-53` |
| PP-FIT-231 | Export to video: WMV, MOV | Refuse | Absent | Refused by decision | decided: not built |  |
| PP-FIT-232 | Upload to Clipchamp | Host seam | Absent | Host seam, not built | decided: not built |  |
| PP-FIT-233 | Export to animated GIF | Edit | Absent | No GIF export | 4 advanced | `PAR-53` |
| PP-FIT-234 | Package for CD: copy to folder | Edit | Absent | No package to folder | 4 advanced | `PAR-53` |
| PP-FIT-235 | Package for CD: burn a disc | Refuse | Absent | Refused by decision | decided: not built |  |
| PP-FIT-236 | Live subtitles while presenting | Host seam | Disabled | Always Use Subtitles disabled; host seam | decided: not built |  |
| PP-FIT-237 | Speaker Coach | Host seam | Disabled | Rehearse with Coach disabled; host seam | decided: not built |  |
| PP-FIT-238 | Present Live (audience on phones) | Refuse | Absent | Refused by decision | decided: not built |  |
| PP-FIT-239 | PowerPoint Live in Teams, Teams meeting | Host seam | Absent | Host seam, not built | decided: not built |  |
| PP-FIT-240 | Broadcast Slide Show | Refuse | Absent | Refused by decision | decided: not built |  |
| PP-FIT-241 | Microsoft Forms in a slide | Host seam | Disabled | Insert Forms disabled; host seam | decided: not built |  |
| PP-FIT-242 | Themes, variants | Edit | Partial | Six built-in palettes and variants apply; a deck's own or another deck's themes are not offered | 2 formatting | `PAR-38` |
| PP-FIT-243 | Create a theme: colours, fonts | Edit | Absent | No Customize Colors or Fonts | 2 formatting | `PAR-38` |
| PP-FIT-244 | Multiple themes in a deck | Edit | Absent | Multiple masters draw; no verb to add or pick one | 4 advanced | `PAR-49` |
| PP-FIT-245 | Save as theme or template | Edit | Absent | No save as theme or template | 4 advanced | `PAR-49` |
| PP-FIT-246 | Slide master: edit, add layouts | Edit | Disabled | Slide Master view disabled | 4 advanced | `PAR-49` |
| PP-FIT-247 | Copy a master between decks | Edit | Absent | No copy master | 4 advanced | `PAR-49` |
| PP-FIT-248 | Format background | Edit | Partial | Format Background: solid and gradient, apply to all; no picture or pattern fill, no hide background graphics | 2 formatting | `PAR-39` |
| PP-FIT-249 | Header, footer, date, slide number | Edit | Disabled | Header & Footer disabled; stored footers draw | 2 formatting | `PAR-40` |
| PP-FIT-250 | Handout and notes header and footer | Edit | Absent | No handout or notes headers | 4 advanced | `PAR-49` |
| PP-FIT-251 | New presentation, template gallery | Edit | Works | File, New offers Blank Presentation and six templates of our own, one for each built-in theme (Plain, Harbour, Meadow, Ember, Slate, Orchid), each tile in its theme's colours; a template opens as a new deck in its colours and fonts with nothing to undo (`PAR-18`). Opening a `.potx` as a template is `PAR-49` |  |  |
| PP-FIT-252 | Designer (Design Ideas) | Host seam | Disabled | Designer disabled; host seam | decided: not built |  |
| PP-FIT-253 | Designer metadata in decks | Retain | Works | Kept in the file |  |  |
| PP-FIT-254 | Microsoft Designer image generation | Host seam | Absent | Host seam, not built | decided: not built |  |
| PP-FIT-255 | Copilot: create a deck from a prompt | Host seam | Absent | Host seam, not built (`DEC-05`) | decided: not built |  |
| PP-FIT-256 | Copilot: create from a file | Host seam | Absent | Host seam, not built (`DEC-05`) | decided: not built |  |
| PP-FIT-257 | Copilot: add a slide from a file | Host seam | Absent | Host seam, not built (`DEC-05`) | decided: not built |  |
| PP-FIT-258 | Copilot: rewrite, condense, generate text | Host seam | Absent | Host seam, not built (`DEC-05`) | decided: not built |  |
| PP-FIT-259 | Copilot: edit, with a choice of model | Host seam | Absent | Host seam, not built (`DEC-05`) | decided: not built |  |
| PP-FIT-260 | Copilot: summarise, prepare to present | Host seam | Absent | Host seam, not built (`DEC-05`) | decided: not built |  |
| PP-FIT-261 | Copilot: Brand Kit, template check | Host seam | Absent | Host seam, not built (`DEC-05`) | decided: not built |  |
| PP-FIT-262 | Copilot: interactive slides | Host seam | Absent | Host seam, not built (`DEC-05`) | decided: not built |  |
| PP-FIT-263 | Print slides, handouts, notes | Edit | Partial | File, Print and Ctrl+P print every slide, one a page, through the browser's dialog (`FILE-01`); no handouts, notes pages, outline, grayscale or frame | 3 review and sharing | `PAR-43` |
| PP-FIT-264 | Print Outline | Edit | Absent | No Print Outline | 3 review and sharing | `PAR-43` |
| PP-FIT-265 | Print in colour, grayscale, pure black and white | Edit | Absent | No print colour options | 3 review and sharing | `PAR-43` |
| PP-FIT-266 | Print: scale to fit, frame slides, print hidden | Edit | Absent | No print options | 3 review and sharing | `PAR-43` |
| PP-FIT-267 | Printer-specific options: TrueType as graphics, background | Refuse | Absent | Refused by decision | decided: not built |  |
| PP-FIT-268 | Create Handouts in Word | Host seam | Absent | Host seam, not built | decided: not built |  |
| PP-FIT-269 | Insert PDF content: as picture or embedded object | Edit | Absent | No Insert PDF | 4 advanced | `PAR-60` |
| PP-FIT-270 | Import from other applications | Edit | Absent | No import beyond the open formats | 4 advanced | `PAR-60` |
| PP-FIT-271 | Share link, invite | Host seam | Partial | Share makes a room link to co-edit through this origin's server; no view-only or named people, needs the collab server running | 3 review and sharing | `PAR-48` |
| PP-FIT-272 | Link to a slide | Edit | Absent | No link to a slide | 3 review and sharing | `PAR-48` |
| PP-FIT-273 | Email as attachment | Host seam | Absent | Host seam, not built | decided: not built |  |
| PP-FIT-274 | Comments: add, edit, delete, resolve | Edit | Works | Comments pane: a deck's modern comments shown by slide, with markers; New, reply, resolve, delete thread, Previous and Next; saved and reopened (`UX-011`) |  |  |
| PP-FIT-275 | Legacy comments in old decks | Edit | Absent | Legacy comments not shown | 3 review and sharing | `PAR-42` |
| PP-FIT-276 | @mentions | Edit | Partial | An @mention is shown as a mention in the comment's text and sent nowhere: this build has no people to notify (`UX-011`) | 3 review and sharing | `PAR-42` |
| PP-FIT-277 | Comment reactions | Edit | Absent | No reactions on comments | 3 review and sharing | `PAR-42` |
| PP-FIT-278 | Real-time co-authoring, presence | Edit | Partial | Live co-editing with carets and roster when served by the collab server (`VS-09`); not available on a plain static host | 3 review and sharing | `PAR-48` |
| PP-FIT-279 | Revision highlighting | Edit | Absent | No revision highlighting | 3 review and sharing | `PAR-48` |
| PP-FIT-280 | Recent-activity notifications | Edit | Absent | No activity feed | 3 review and sharing | `PAR-48` |
| PP-FIT-281 | Version history | Host seam | Disabled | File, Info, Version History disabled; host seam | decided: not built |  |
| PP-FIT-282 | Compare and Merge | Refuse | Absent | Refused by decision | decided: not built |  |
| PP-FIT-283 | Revision and merge data in old decks | Retain | Works | Kept in the file |  |  |
| PP-FIT-284 | Password to open: open a deck | Edit | Absent | Encrypted decks are refused, no password prompt | 3 review and sharing | `PAR-46` |
| PP-FIT-285 | Password to open: set on save | Edit | Absent | No password on save | 3 review and sharing | `PAR-46` |
| PP-FIT-286 | Password to modify | Edit | Absent | No password to modify | 3 review and sharing | `PAR-46` |
| PP-FIT-287 | Mark as Final | Edit | Absent | No Mark as Final; a final deck is not shown read-only | 3 review and sharing | `PAR-46` |
| PP-FIT-288 | Restrict Access (IRM) | Host seam | Absent | Host seam, not built | decided: not built |  |
| PP-FIT-289 | Sensitivity label: read, show, keep | Edit | Absent | A label in the file is kept but not shown | 3 review and sharing | `PAR-46` |
| PP-FIT-290 | Sensitivity label: apply, mandatory, default, auto | Host seam | Absent | Host seam, not built | decided: not built |  |
| PP-FIT-291 | Dynamic watermark from a label | Host seam | Absent | Host seam, not built | decided: not built |  |
| PP-FIT-292 | Digital signature: keep, warn, drop on edit | Retain | Partial | Signature parts kept; no warning that an edit drops their validity | 3 review and sharing | `PAR-46` |
| PP-FIT-293 | Digital signature: add, validate | Host seam | Absent | Host seam, not built | decided: not built |  |
| PP-FIT-294 | Document Inspector | Edit | Absent | No Document Inspector | 3 review and sharing | `PAR-45` |
| PP-FIT-295 | Remove personal information on save | Edit | Absent | No remove personal information | 3 review and sharing | `PAR-45` |
| PP-FIT-296 | Accessibility Checker, in the background | Edit | Disabled | Check Accessibility disabled | 3 review and sharing | `PAR-44` |
| PP-FIT-297 | Reading order pane | Edit | Absent | No reading order pane | 3 review and sharing | `PAR-44` |
| PP-FIT-298 | Alt text | Edit | Disabled | Alt Text on Shape and Picture Format disabled | 3 review and sharing | `PAR-44` |
| PP-FIT-299 | Automatic alt text | Host seam | Absent | Host seam, not built | decided: not built |  |
| PP-FIT-300 | Accessibility ribbon, tools | Edit | Absent | No accessibility ribbon | 3 review and sharing | `PAR-44` |
| PP-FIT-301 | Accessible PDF | Edit | Partial | PDF has real text; not tagged (`PDF-01`) | 3 review and sharing | `PAR-44` |
| PP-FIT-302 | Customise the ribbon, export settings | Edit | Absent | No ribbon customisation | 4 advanced | `PAR-61` |
| PP-FIT-303 | Quick Access Toolbar | Edit | Partial | Save, Undo, Redo, Start From Beginning in the title bar; not customisable | 4 advanced | `PAR-61` |
| PP-FIT-304 | Customise shortcuts | Edit | Absent | No shortcut customisation | 4 advanced | `PAR-61` |
| PP-FIT-305 | Dark mode, Office theme | Edit | Works | Theme: Automatic, Light, Dark |  |  |
| PP-FIT-306 | Touch mode | Edit | Absent | No touch mode | 4 advanced | `PAR-61` |
| PP-FIT-307 | Recent files, pinned | Edit | Works | File, Open and New list recent decks, kept in this browser's IndexedDB with their bytes: pin, unpin, remove, and open again from the list (`PAR-19`). Nothing uploaded |  |  |
| PP-FIT-308 | Advanced options set | Edit | Absent | No Options page | 4 advanced | `PAR-61` |
| PP-FIT-309 | Disable slide show hardware acceleration | Refuse | Absent | Refused by decision | decided: not built |  |
| PP-FIT-310 | Responsive phone shell: view, present, light edit | Edit | Absent | No phone layout | 4 advanced | `PAR-63` |
| PP-FIT-311 | Tablet editing | Edit | Absent | No touch editing | 4 advanced | `PAR-63` |
| PP-FIT-312 | Native iOS and Android apps | Refuse | Absent | Refused by decision | decided: not built |  |
| PP-FIT-313 | Presenter view on phone | Edit | Absent | No phone presenter view | 4 advanced | `PAR-63` |
| PP-FIT-314 | Morph and Designer on mobile | Edit | Absent | No mobile shell | 4 advanced | `PAR-63` |
| PP-FIT-315 | Ink annotations saved on phone | Edit | Absent | No ink | 4 advanced | `PAR-51` |
| PP-FIT-316 | Insert phone pictures, camera | Edit | Absent | No camera insert | 4 advanced | `PAR-63` |
| PP-FIT-317 | IRM on mobile | Host seam | Absent | Host seam, not built | decided: not built |  |
| PP-FIT-318 | Copilot on mobile | Host seam | Absent | Host seam, not built | decided: not built |  |
| PP-FIT-319 | Undo and Redo history list (Undo ▾) (Ribbon, Home) | Edit | Partial | Single steps work; no list of steps to undo several at once | 4 advanced | `PAR-61` |
| PP-FIT-320 | Paste Options: Keep Source Formatting, Use Destination Theme, Picture, Keep Text Only (Ctrl+Shift+V) (Ribbon, Home; right-click) | Edit | Partial | Keep Source Formatting, Use Destination Theme and Keep Text Only (Ctrl+Shift+V) in the Paste menu, redoing the last paste (`PAR-11`); no Picture option and no floating Paste Options button | 1 everyday editing | `PAR-11` |
| PP-FIT-321 | Paste from other applications: rich text, HTML, images (Clipboard) | Edit | Works | HTML text keeps bold, italic, underline, size, colour and lists; a PNG or JPEG pastes as a picture (`PAR-11`). RTF-only sources paste as plain text |  |  |
| PP-FIT-322 | Clipboard pane (Ribbon, Home launcher) | Edit | Disabled | Launcher disabled | 4 advanced | `PAR-61` |
| PP-FIT-323 | Font and Paragraph dialogs (Launchers; right-click) | Edit | Partial | Font… works from the launcher, the right-click menu and Ctrl+Shift+F (`PAR-20`); Paragraph… disabled | 2 formatting | `PAR-21` |
| PP-FIT-324 | Change Case (Ribbon, Home) | Edit | Works | Sentence case, lowercase, UPPERCASE, Capitalize Each Word and tOGGLE cASE, each run keeping its look, one undo step (`PAR-20`) |  |  |
| PP-FIT-325 | Character spacing (Ribbon, Home) | Edit | Works | Character Spacing on the Home tab: Very Tight, Tight, Normal, Loose, Very Loose, and More Spacing… opening the Font dialog's Character Spacing page, saved as `spc` (`PAR-20`) and drawn (`PAR-67`) |  |  |
| PP-FIT-326 | Height and Width boxes on Shape and Picture Format, lock aspect ratio (Ribbon, contextual) | Edit | Disabled | Ribbon boxes disabled; Size & Position pane edits size and position | 2 formatting | `PAR-24` |
| PP-FIT-327 | Change Shape (Ribbon, Shape Format; right-click) | Edit | Disabled | Disabled | 2 formatting | `PAR-24` |
| PP-FIT-328 | Duplicate object (Ctrl+D) and arrow-key nudge (Keys) | Edit | Works | Ctrl+D, arrows, Ctrl+arrows |  |  |
| PP-FIT-329 | Copy and paste slides, within and between decks (Thumbnails; keys) | Edit | Partial | Slides copy and paste in the thumbnail pane, several with Ctrl+ and Shift+click, within a deck and into another opened in the same tab, with pictures (`PAR-10`). Not between two browser tabs; groups, tables and charts stay behind in another deck, and the status bar says how many | 1 everyday editing | `PAR-10` |
| PP-FIT-330 | Status bar: Slide x of y, language, accessibility (Status bar) | Edit | Partial | Slide count shown; no language or accessibility indicator | 4 advanced | `PAR-61` |
| PP-FIT-331 | Ribbon Display Options: Classic, Simplified, show tabs only, Ctrl+F1 (Ribbon) | Edit | Works | Toggle and Ctrl+F1 |  |  |
| PP-FIT-332 | KeyTips (Alt, Alt+Period) (Keys) | Edit | Works | Letters over tabs and commands (`UX-003`) |  |  |
| PP-FIT-333 | Right-click menus for slide, thumbnail, shape and text (Menus) | Edit | Partial | Present; Layout, Reset Slide, Add Section, Edit Text, Edit Points, Group, Bring to Front, Link, Alt Text, Font…, Paragraph…, Numbering and New Comment disabled | 1 everyday editing | `PAR-07`, `PAR-14` |
| PP-FIT-334 | Shift+F10 menu and Ctrl+F6 region cycling (Keys) | Edit | Works | Both registered |  |  |
| PP-FIT-335 | Editing, Reviewing, Viewing modes (Tab row) | Edit | Disabled | Only Editing; the other two disabled | 3 review and sharing | `PAR-48` |
| PP-FIT-336 | Rename the file from the title bar (Title bar) | Edit | Works | Click the name in the title bar, type, Enter: Save, Save As and Export use it (`FILE-01`) |  |  |
| PP-FIT-337 | Format Shape pane: Text Options (text box margins, wrap, autofit, vertical alignment) (Pane) | Edit | Disabled | Size & Properties edits size, position, rotation; Text Options and Text Box settings disabled | 2 formatting | `PAR-23` |
| PP-FIT-338 | Help, Contact Support, Feedback (Ribbon, Help) | Edit | Disabled | Disabled; Keyboard Shortcuts and About work | 4 advanced | `PAR-61` |
| PP-FIT-339 | Unsaved-changes warning on leaving the page (File operation) | Edit | Works | Closing or reloading the tab with unsaved edits asks first, through the browser's own leave prompt, as OpenCalc (`PAR-01`); nothing is asked for an unedited deck |  |  |
| PP-FIT-340 | Save As with a choice of format (`.pptx`, `.ppsx`, `.potx`, `.odp`, `.pdf`) (File) | Edit | Partial | Save As (F12) offers PowerPoint Presentation, Template and Show (`FILE-01`); no `.odp`, and PDF is under Export | 4 advanced | `PAR-57` |
| PP-FIT-341 | Text selection by double-click (word) and triple-click (paragraph), Ctrl+arrow by word, Ctrl+Home and End (Text editing) | Edit | Works | Double and triple click, Ctrl+arrows by word and paragraph, Ctrl+Home and End, Shift to extend (`PAR-12`) |  |  |
| PP-FIT-342 | Table Layout: cell size, alignment in cell, cell margins, distribute rows and columns, text direction (Ribbon, contextual) | Edit | Absent | Only insert and delete rows and columns | 2 formatting | `PAR-34` |
| PP-FIT-343 | Chart Design and Format: chart styles, add element, quick layout, change chart type, select data (Ribbon, contextual) | Edit | Absent | Only Edit Data | 2 formatting | `PAR-35` |
| PP-FIT-344 | Picture Format: Change Picture, Compress, Picture Border and Effects (Ribbon, contextual) | Edit | Absent | Only Crop, Reset and four styles | 2 formatting | `PAR-33` |
| PP-FIT-345 | Lock and unlock an object (Right-click) | Edit | Absent | No lock verb | 4 advanced | `PAR-61` |
| PP-FIT-346 | Set as Default Shape or Text Box (Right-click) | Edit | Absent | No verb | 4 advanced | `PAR-61` |
| PP-FIT-347 | Save as Picture (right-click a picture or shape) (Right-click) | Edit | Absent | No verb | 4 advanced | `PAR-61` |
| PP-FIT-348 | Ctrl+mouse wheel and pinch zoom on the slide (Keys and pointer) | Edit | Absent | Zoom only from the slider and the menu | 4 advanced | `PAR-61` |
| PP-FIT-349 | Placeholder prompts ("Click to add title") on new slides (Stage) | Edit | Works | Empty placeholders show their prompt (`PLACEHOLDER` flag) |  |  |
| PP-FIT-350 | Slide show right-click menu and on-screen controls (Slide show) | Edit | Absent | Only keys and clicks; no menu or control bar | 4 advanced | `PAR-52` |
| PP-FIT-351 | Keyboard shortcuts: Ctrl+K, Ctrl+F, Ctrl+H, Ctrl+G, Ctrl+Shift+G, Ctrl+[ and ], Ctrl+Space, Ctrl+P, Ctrl+Shift+C and V (Keys) | Edit | Works | Each key runs its command or names it as not available yet, ahead of the browser (`PAR-13`, `editor.keys.js`) |  |  |
| PP-FIT-352 | Bring Forward, Bring to Front, Send Backward, Send to Back (Ctrl+[ and ]) (Ribbon, Arrange; right-click) | Edit | Disabled | All four disabled on the ribbon and in the menu | 1 everyday editing | `PAR-14` |

## Roadmap asks this analysis raises

This page changes no other document. These are for whoever owns [06](06-ROADMAP-AND-DELIVERY.md), [14](14-EXECUTION-TRACKER.md) and [spec/](spec/README.md); each is a decision taken here under instruction 12, and it holds until the owner says otherwise.

| Ask | Why | Rows |
| --- | --- | --- |
| Phase 8's "Out" says recording is out "unless the gap analysis adds it". **This analysis adds it,** to 8B: narration, timings, camera, ink, teleprompter | Windows and Mac have it, and `DEC-06` sets the desktop bar | Record rows, Cameo |
| **No phase owns video and GIF export.** Add to 8B, with an encoder decision under `LEG-01` | A deterministic frame capture exists nowhere yet, and H.264 carries licensing questions | Export rows |
| **No tier owns a path boolean engine.** Add to 1E or 4 | Merge Shapes cannot be done without one | Merge Shapes |
| **No tier owns a 3D renderer or model parsers.** Spec `MODEL3D` says Retain and has no Edit UI; this page says Edit. Change the spec area and add a tier after `2F`, or the owner overrides to Render | A clone needs live 3D. Parsers are new hostile input, so `openslide-parser` rules and fuzz targets apply | 3D rows |
| Add Outline view to Phase 5, and Notes Page view to Phase 4 | Phase 3 lists only Normal, Sorter, Reading and a notes pane | Outline, Notes Page |
| Add to Phase 6: record audio, screen recording, screenshot, caption files, photo album, Compress Pictures, Compress Media | None is on Phase 6's list, though 48 has Compress Pictures at 6 | Those rows |
| `SPEC-04` must find: cameo storage, add-in parts (0 `webextension` rows), the sensitivity label part, caption tracks | The checklist has no row under those names. A `F-007` deck with each will settle it | Cameo, add-ins, labels |
| Fix 47's `SmartArt (Render...)` and `Equation (Render)` | `DGM-01` made both Edit | SmartArt, Equation |
| Fix 01's "Out of scope": macro wording stands, but Designer and Copilot now read as `DEC-05` seams | Consistency only | Designer, Copilot |
| `WR-05` to confirm XPS, WMF, EMF, WMV, MOV, HTML and MHT refusals, and `.xml` and `.thmx` as Edit | That row owns the save-as matrix. This page recommends; it does not overrule | Format rows |
| Decide who writes `.docx` for Slides from Outline: OpenDoc's reader, or a new one | A new hostile-input parser otherwise | Slides from Outline |

## Findings a reader should know

- **Reuse Slides was not removed.** Search results and a vendor blog say it was taken out in January 2026. The Message Center notice as mirrored ([MC1179161](https://mc.merill.net/message/MC1179161)) and Microsoft's own page say cloud reuse went, and local files remain. The row says so. The mirror is third party; Microsoft's copy is admin-only.
- **Compare and Merge is retired,** from version 2502 on Windows, and Mac and web never had it. The task listed it, so it has a row, and the row is Refuse.
- **Present Live is deprecated** per its own page, yet Microsoft's help home still links it. Refused on that page's word; `UX-001` can confirm.
- **Slide Library evidence is the weakest of the required rows.** The support article's URL now serves a sharing page, and what remains is Microsoft Q&A answers. The Refuse stands either way: `ENT-17` covers the need.
- **3D on the web:** Microsoft says the web cannot insert a 3D graphic but plays one inserted on desktop. Formats were not named on the page read. **The rows do not list file formats**, and they should not be filled from memory.
- **Grayscale view** is documented by Microsoft only in an article for PowerPoint 2002 to 2007, and the older article first found now returns 404. Current pages document grayscale printing, not the view. Third-party guides put it on today's View tab. Edit stands, since print needs the same setting; `UX-001` confirms the view.
- **Microsoft's table disagrees with itself** in the same ways 48 lists, and its Footer and Slide Master rows lost cell alignment in the fetched text, so those were mapped in order.

## What this sweep did not reach

- **Observed in a running PowerPoint: none.** Every row needs `UX-001` for how it looks, and `PV-01` for proof.
- **Listing-only rows** have a title and no platforms read. They exist; their limits are unknown.
- **Mac and iOS specifics** were read only where a page split them. Absent is not "no".
- **The Inferred rows** are the first to check: connectors, poster frames, AutoCorrect, Ink to Text, COM add-ins, automatic alt text, dark mode, cloud fonts and window arranging came back from no page read, or from recall.
- **Depth inside a row.** "Charts" is one family here, and a feature's sub-options belong to 49, 50 and [spec/](spec/README.md).
- **Help outside the index.** The category pages list a small share of Microsoft's articles, so a missing row is possible. Add one, with its mark, rather than assuming no gap.
