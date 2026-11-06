# 48 — PowerPoint feature study

Studied 2026-11-07. Three independent views, because each alone misleads:

| View | Answers | Source | Trust |
| --- | --- | --- | --- |
| **The file format** | What a deck can contain | Ecma schemas plus MS-PPTX and MS-ODRAWXML: [spec/](spec/README.md), 8730 rows | Generated, complete for what schemas express |
| **The product** | What users can do | Microsoft support pages, below | **Partly stale; see Conflicts** |
| **The running app** | What it looks and feels like | `UX-001` by online research; live only through `PV-01` | Not yet done |

## The scope decision this exposes

Microsoft's pages describe **PowerPoint for the web as a subset**: slide masters, presenter view,
custom shows, recording and more are listed as desktop-only. "PowerPoint online clone, same features"
and "enterprise grade" point different ways on those.

**Decided (`DEC-06`), both web and desktop:** the **web product sets the UI style and flows**, and the
**desktop product sets the feature bar**. A deck from desktop must be fully faithful, and an enterprise
buyer expects masters and presenter view. Where the web has no UI for a desktop feature, we follow
desktop behaviour in the web's visual language.

## What Microsoft documents

From [Compare PowerPoint features](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b),
[How features behave on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint) and the
[service description](https://learn.microsoft.com/en-us/office365/servicedescriptions/office-online-service-description/powerpoint-online).
`P` is the phase in [06](06-ROADMAP-AND-DELIVERY.md).

| Feature | Win | Web | Mac | P |
| --- | --- | --- | --- | --- |
| Play audio, across slides, in background | Y | Y | Y | 8 |
| Add audio | Y | N | Y | 6 |
| Trim, bookmark, fade audio | Y | N | Sub | 6 |
| Right-to-left languages, text direction | Y | **conflict** | Y | 2D, 5 |
| Comments, @mentions, real-time co-authoring | Y | Y | Y | 9 |
| Recent-activity notifications | Y | N | Y | 9 |
| Revision highlighting | Y | Limited | Y | 9 |
| Headers, footers, date, numbering | Y | **conflict** | Y | 4 |
| Add a theme; background colour and image; slide size and orientation | Y | Y | Y | 7 |
| Edit the Slide Master | Y | N | Y | 7 |
| Alt, Ctrl, Fn shortcuts | Y | Y | Ctrl, Fn | 3 |
| Custom shortcuts | Y | N | Y | 10 |
| Tables; shapes | Y | Y | Y | 4, 6 |
| SmartArt | Y | **conflict** | Y | 6 |
| Add charts | Y | **conflict** | Y | 6 |
| Pictures, online pictures, picture styles | Y | Y | Y | 4 |
| Compress pictures | Y | N | Y | 6 |
| Print slides, notes, handouts | Y | Y | Y | 2G |
| Add, remove, rearrange slides | Y | Y | Y | 4 |
| Transitions; animations | Y | Limited | Y | 8 |
| Animation triggers | Y | N | N | 8 |
| Video, including YouTube | Y | **conflict** | Y | 6 |
| Trim, bookmark, fade video | Y | N | Y | 6 |
| Lists, hyperlinks, text boxes, paragraph formatting, super and subscript | Y | Y | Y | 5 |
| Highlighting | Y | Sub | Sub | 5 |
| Speaker notes | Y | Y | Y | 4 |
| Self-running, presenter view, recording, custom show | Y | N | Y | 8 |

Sub = subscribers only. Further web behaviour recorded from those pages:

- **Open:** up to 2 GB. `.pptx`, `.ppsx`, `.odp`. `.ppt` and `.pps` viewed, converted to edit. Templates and macro-enabled formats view only. Password, signed or IRM files need desktop.
- **Save:** automatic. Download as `.pptx`, `.pdf`, `.odp`, JPEG per slide.
- **Views:** Normal, Slide Sorter, Reading, Slide Show. Outline and Master are desktop only. Rulers and gridlines desktop; smart guides on web.
- **Editing:** Find on web, Replace listed as not available. Merge and split table cells desktop. Gradient, eyedropper and effects desktop. Picture effects and background removal desktop.
- **Displayed, not editable on web:** ActiveX and OLE, charts and equations, ink, desktop-only animations and transitions, macros, linked pictures (preserved, placeholders in editing).
- **Undo:** "an infinite number of recent actions" during a session.
- **Proofing:** built-in dictionary; no custom dictionary, translation or thesaurus.

Shortcuts: [spec/web-shortcuts.md](spec/web-shortcuts.md), 111 recorded bindings. Ribbon access keys on the same
page: Alt+Q Search, F File, H Home, N Insert, J then I Draw, G Design, K Transitions, A Animations, S Slide
Show, R Review, W View, E Recording, Y then 2 Help. That list is from the page's **Windows desktop** section, and
the web's own must be confirmed live.

## Conflicts: why these pages cannot be the specification

Microsoft's own pages disagree, and at least one is visibly old.

| Topic | Disagreement |
| --- | --- |
| RTL | Compare table says no; the behaviour page says yes |
| Headers and footers | Yes on two pages, display-only on the service description |
| SmartArt | "Limited" against full editing |
| Charts | Not supported on two pages, in a product that has likely moved on |
| Video | Yes against "inserting from your PC needs desktop" |
| Themes | "Add a theme" yes against themes not being modifiable |
| Transitions in show | "Eight transitions" against "only Fade and Wipe" |
| Dates and age | The behaviour page requires **Silverlight and Flash**. The service description is dated 2023-04 |

**So:** this table plans phases. It does not decide behaviour. `UX-001` measures the live product, and until
then a "web: no" here is not evidence that the web lacks it.

## What the format adds beyond any product page

Schema-derived, so complete where the pages are vague ([spec/](spec/README.md)):

| Construct | Count or set |
| --- | --- |
| Preset shapes | 187 (`ST_ShapeType`) |
| Text warps | 41 (`ST_TextShapeType`) |
| Pattern fills | 54 |
| Named colours; scheme colours | 190; 17 |
| Placeholder types; layout types | 16; 36 |
| Autonumber schemes | 41 |
| Underline styles; dash presets; arrowheads | 18; 11; 6 |
| Preset shadows; bevels; materials; cameras; light rigs | 20; 12; 15; 62; 27 |
| Base transitions | 21 (blinds to zoom), plus 19 in the 2010 namespace, plus `prstTrans` and Morph |
| Classic chart families | 16, including 3D and of-pie |
| 2016+ chart layouts | 8: box-whisker, clustered column, funnel, Pareto, region map, sunburst, treemap, waterfall |
| Timing node types | 9, across 6 preset classes |
| Extension namespaces, PowerPoint | 20 specs, from sections to reactions and task details |
| Extension namespaces, drawing | 35 in scope, from background removal to SVG and 3D models |

## Other things the study settled

- **Modern comments are a different part** (`2018/8/main`) from legacy comments. Both must round-trip.
- **Zoom objects carry a fallback** as a picture or group in `mc:AlternateContent`, which sets the render order.
- **Sensitivity labels matter to enterprises**, and applying them with user-defined permissions now exists on the web
  ([MC1208688](https://mc.merill.net/message/MC1208688)): see ENT-04.
- **Ecma's geometry file has a defect:** `upDownArrow` appears twice and `upArrow` is absent, in the December 2016
  edition's `presetShapeDefinitions.xml`. A naive generator ships 186 shapes and silently loses a common arrow. `GEO-01`.
- **LibreOffice is a limited reference.** Public bug reports describe it diverging from PowerPoint on autofit,
  font widths and SmartArt lines (for example [149997](https://www.libreoffice.org/bugzilla/show_bug.cgi?id=149997)). Those
  reports are old and may be fixed, but they are why it cannot be the fidelity target.

## What this study has not covered

- The live product (`UX-001`). No screenshot, measurement or ribbon inventory has been taken.
- Copilot and Designer behaviour (`DEC-05`).
- MS-OI29500 implementation notes, which say where Office departs from the standard. Required reading before each tier (`SPEC-02`).
- A competitor comparison with ONLYOFFICE and Google Slides.
