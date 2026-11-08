# 53 — PowerPoint for the web: the shell we build to

The compact target for `webapp/` (`UX-003`, owner 2026-11-08: "UX as well as UI, options and the way it
works, is not at all similar to PowerPoint's"). One table per region, in PowerPoint's order. Detail and
sources stay in [49](49-POWERPOINT-COMMAND-INVENTORY.md) (commands, Microsoft's control-ID lists S1 and
S2) and [50](50-POWERPOINT-UX-STUDY.md) (behaviour); this page wins over the sketch in
[47](47-POWERPOINT-SHELL-AND-FEATURE-MAP.md).

**Evidence.** **[D]** Microsoft documents it (through docs/49 or 50, or read again on 2026-11-08).
**[R]** recalled from the product as shipped in 2024–2026, no page found. No live session was run: Microsoft's
support pages name few groups (the Simplified-ribbon page names one), and web search found no current
tab-by-tab list. `UX-002` measures; where the live product disagrees, it wins.

**Ours, not Microsoft's** (docs/47): the accent is our cyan (`DEC-03`), icons are Fluent UI System Icons
(MIT), the mark and name are OpenPresent. No Microsoft logo, artwork, theme or template.

**In the build.** Every command below is in `webapp/ribbon.model.js`. One that works runs; the rest are
drawn disabled through the registry with the reason, never faked (owner instruction 46).

## Window

| Region | Target | Ev. |
| --- | --- | --- |
| Title bar | Mark, file name (click: rename, version history), Search (Alt+Q) centred | [D] |
| Tab row, right end | Comments, Editing mode (Editing, Reviewing, Viewing), Present (split: From Beginning, From Current Slide, Presenter View), Share | [R] |
| Ribbon | Classic by default (`DEC-10`); Ribbon Display Options at the band's right end: Classic, Simplified, Always show, Show tabs only; double-click a tab or Ctrl+F1 hides it | [D] options, [R] place |
| Left | Slide thumbnails, numbered, hidden ones struck, drag to reorder | [D] |
| Centre | Slide on grey, fit to window; notes pane below, toggled from the status bar | [D] |
| Right | Task panes: Format Shape, Format Background, Animation, Comments, Accessibility | [D] |

## Quick Access Toolbar

Web: Undo and Redo sit in Home's first group, not a title-bar toolbar ([R]; no web QAT is documented, docs/49).
We keep desktop's three in the title bar as well: Save, Undo, Redo, Start From Beginning ([D] desktop S1).

## Ribbon, Classic

Large buttons are 24 px icons with a label under; small ones 16 px with a label beside where PowerPoint
labels them. `L` large, `S` small labelled, `s` small icon-only, `C` combo, `▾` drops a menu, `↘` dialog launcher.

| Tab | Groups in order, and their commands | Ev. |
| --- | --- | --- |
| File | Backstage, below | [D] |
| Home | **Undo**: s Undo ▾, s Redo · **Clipboard** ↘: L Paste ▾, S Cut, S Copy ▾, S Format Painter · **Slides**: L New Slide ▾, S Layout ▾, S Reset, S Section ▾ · **Font** ↘: C Font, C Size, s Grow, s Shrink, s Clear; s B, I, U, S, Subscript, Superscript, Change Case ▾, Highlight ▾, Font Color ▾ · **Paragraph** ↘: s Bullets ▾, Numbering ▾, Decrease List Level, Increase List Level, Line Spacing ▾; s Left, Center, Right, Justify, Columns ▾, Text Direction ▾, Align Text ▾, Convert to SmartArt ▾ · **Drawing** ↘: L Shapes ▾, L Arrange ▾, L Quick Styles ▾, S Shape Fill ▾, S Shape Outline ▾, S Shape Effects ▾ · **Editing**: S Find, S Replace, S Select ▾ · **Voice**: L Dictate · **Designer**: L Designer | [D] groups S1/S2; [R] web Undo group |
| Insert | **Slides**: L New Slide ▾ · **Tables**: L Table ▾ · **Images**: L Pictures ▾ · **Camera**: L Cameo · **Illustrations**: L Shapes ▾, L Icons, L 3D Models, L SmartArt, L Chart · **Forms**: L Forms · **Add-ins**: L Add-ins · **Links**: L Link, L Action · **Comments**: L Comment · **Text**: L Text Box, L Header & Footer, L WordArt ▾ · **Symbols**: L Equation, L Symbol · **Media**: L Video ▾, L Audio ▾ | [D] S1 |
| Draw | **Tools**: L Select, L Lasso, L Eraser, L Pen, L Highlighter · **Convert**: L Ink to Shape, L Ink to Math · **Replay**: L Ink Replay | [D] S1, [R] web |
| Design | **Themes**: gallery ▾ · **Variants**: gallery ▾ · **Customize**: L Slide Size ▾, L Format Background · **Designer**: L Designer | [D] S1 |
| Transitions | **Preview**: L Preview · **Transition to This Slide**: gallery ▾, L Effect Options ▾ · **Timing**: S Duration, S Apply To All, S On Mouse Click, S After | [D] S1 |
| Animations | **Preview**: L Preview · **Animation**: gallery ▾, L Effect Options ▾ · **Advanced Animation**: L Add Animation ▾, S Animation Pane, S Trigger ▾, S Animation Painter · **Timing**: S Start ▾, S Duration, S Delay, S Move Earlier, S Move Later | [D] S1 |
| Slide Show | **Start Slide Show**: L From Beginning, L From Current Slide, L Presenter View · **Set Up**: L Set Up Slide Show, L Hide Slide, L Rehearse with Coach · **Captions & Subtitles**: S Always Use Subtitles, S Subtitle Settings ▾ | [D] S1 |
| Review | **Proofing**: L Spelling ▾, L Thesaurus · **Accessibility**: L Check Accessibility · **Language**: L Translate, L Language ▾ · **Comments**: L New Comment, S Delete ▾, S Previous, S Next, L Show Comments | [D] S1; Compare is desktop only |
| View | **Presentation Views**: L Normal, L Outline View, L Slide Sorter, L Notes Page, L Reading View · **Master Views**: L Slide Master · **Show**: S Ruler, S Gridlines, S Guides, S Notes · **Zoom**: L Zoom ▾, L Fit to Window · **Color**: S Grayscale | [D] S1 |
| Help | **Help**: L Help, L Contact Support, L Feedback, L Keyboard Shortcuts, L About | [D] S1, ours |
| Shape Format (contextual, accent label, after Help, while a shape is selected) | **Insert Shapes**: L Shapes ▾, S Edit Shape ▾, S Text Box · **Shape Styles** ↘: gallery ▾, S Shape Fill ▾, S Shape Outline ▾, S Shape Effects ▾ · **WordArt Styles** ↘: S Text Fill ▾, S Text Outline ▾, S Text Effects ▾ · **Accessibility**: L Alt Text · **Arrange**: L Bring Forward ▾, L Send Backward ▾, L Selection Pane, S Align ▾, S Group ▾, S Rotate ▾ · **Size** ↘: S Height, S Width | [D] S1 |

**Simplified ribbon** ([D] S10, S11): one row, the same tabs, each group's most used commands as icon
buttons, the rest behind the group's "…" overflow; galleries become dropdowns.

## Command audit, 2026-11-09 (`UX-006`)

Owner, 2026-11-09: "many features on the toolbar are disabled or not implemented, like transitions". Every command
id the ribbon model names was read from the running editor's registry (`commands.js`) in a browser on this
build: **works** is a registered command that runs; **disabled** is drawn disabled with the reason "not available
yet", and each such group has a tracker row; **menu** is a button that only drops its menu, live while anything in
it is. **Faked**, a control that looks live and changes nothing, is none: the gallery tiles that said "not
available" (Morph, Reveal, Float In…) now apply or were removed, Home > Layout no longer adds a new slide under
that name (it is disabled until `RIB-02`), and Zoom and Slide Size run from search open their choices.

| Tab / group | Works | Disabled | Row |
| --- | --- | --- | --- |
| Home / Undo | Undo, Redo | | |
| Home / Clipboard | Paste, Cut, Copy, Duplicate | Keep Text Only, Format Painter, Clipboard launcher | `RIB-03` |
| Home / Slides | New Slide (and its menu), Reset; Section: Add, Rename, Remove, Collapse All, Expand All, and headers in the thumbnail pane that collapse | Layout (on an existing slide), Reuse Slides | `RIB-02` |
| Home / Font | Font, Size, Grow, Shrink, Bold, Italic, Underline, Highlight, Font Color | Clear, Strikethrough, Subscript, Superscript, Change Case (4), Font… | `RIB-01` |
| Home / Paragraph | Bullets, the four alignments | Numbering, list levels, Line Spacing, Columns, Text Direction, Align Text, Paragraph… | `RIB-01` |
| Home / Drawing | Shapes, Quick Styles, Fill, Outline, Format Shape pane; Arrange: Bring to Front, Forward, Send Backward, to Back, Align (6), Distribute (2), Rotate Right, Left, Flip Vertical, Horizontal | Group, Ungroup, Selection Pane, Shape Effects | `RIB-03` |
| Home / Editing | Find, Replace, Select All | Select menu's Selection Pane | `RIB-03` |
| Home / Voice, Designer | | Dictate, Designer | `RIB-09` |
| Insert / Slides, Tables, Images | New Slide, Table, Pictures | Reuse Slides | `RIB-02` |
| Insert / Illustrations | Shapes, Chart (4) | Icons, 3D Models, SmartArt | `RIB-05` |
| Insert / Camera, Forms, Add-ins, Media | | Cameo, Forms, Add-ins, Video, Audio | `RIB-05` |
| Insert / Links | Link (Ctrl+K) on a selected shape: `http`, `https`, `mailto`, stored and never fetched | Link on selected text, Action | `RIB-04` |
| Insert / Comments | | Comment (the comments session's) | `COL` rows |
| Insert / Text | Text Box, WordArt (eight styles: fill, outline, text warp) | Header & Footer | `RIB-04` |
| Insert / Symbols | Symbol | Equation | `RIB-05` |
| Draw / all | | Select, Lasso, Eraser, Pen, Highlighter, Ink to Shape, Ink to Math, Ink Replay | `RIB-06` |
| Design / all | Themes, Variants, Slide Size (3), Format Background | Designer | `RIB-09` |
| Transitions / all | Preview; gallery None, Fade, Push, Wipe, Split, Reveal, Cut, Uncover, Cover, Zoom; Effect Options (18, the effect's own shown); Duration; Apply To All; On Mouse Click; After | | |
| Animations / all but Trigger | Preview; gallery and Add Animation (12 effects); Effect Options (8); Animation Pane; Animation Painter; Start; Duration; Delay; Move Earlier, Later | Trigger | `RIB-07` |
| Slide Show / Start | From Beginning, From Current, Presenter View | | |
| Slide Show / Set Up, Captions | Hide Slide | Set Up Slide Show, Rehearse with Coach, Always Use Subtitles, Subtitle Settings | `RIB-07` |
| Review / all | | Spelling, Thesaurus, Check Accessibility, Translate, Language; Comments (the comments session's) | `RIB-08` |
| View / Presentation Views | Normal, Outline View, Slide Sorter, Notes Page, Reading View | | |
| View / Master, Show, Color | Notes | Slide Master, Ruler, Gridlines, Guides, Grayscale | `RIB-09` |
| View / Zoom | Zoom (6), Fit to Window | | |
| Help | Keyboard Shortcuts, About | Help, Contact Support, Feedback | `RIB-09` |
| Shape Format | Shapes, Text Box, Shape Styles, Fill, Outline, Format Text pane, Arrange as Home, Size launcher | Edit Shape, Edit Points, Change Shape, Text Fill, Text Outline, Text Effects, Shape Effects, Alt Text, Group, Selection Pane, Height, Width | `RIB-10` |
| Picture Format, Table Design, Layout, Chart Design | Reset Picture (2), Picture Styles, Crop, Arrange, Table Styles, rows and columns (6), Edit Text, Edit Data | Alt Text, Height, Width | `RIB-10` |
| File | New, Open, render report, Save, Save a Copy, PDF, PNG, Share | Properties, Version History, Print (the File session's) | File rows |
| Right-click menus | as the ribbon | Layout; Edit Text, Edit Points, Group, Alt Text; Font…, Paragraph…, Numbering; New Comment | as above |

Menu buttons (Arrange, Align, Rotate, Group, Section, Select, Change Case) are drawn live while any command in their
menu is. Insert > Slide Number and Date & Time open Header & Footer in PowerPoint and wait with it (`RIB-04`).

## File (Backstage)

| Page | Contents | Ev. |
| --- | --- | --- |
| Home, New | Blank Presentation, then templates (ours) | [D] |
| Open | Browse this device | [D] |
| Info | Properties; what this build leaves out (ours) | [D] |
| Save a Copy, Download | Save, Save a Copy, Download as PDF, PNG of the slide | [D] |
| Print | Print, Ctrl+P | [D] |
| Share | Share with people | [D] |
| About, Options | Theme; ribbon labels | [D] |
| Back | Arrow at the top of the rail; Esc | [D] |

## Thumbnail pane, status bar, menus, pane

| Region | Target | Ev. |
| --- | --- | --- |
| Thumbnails | Number left of each slide; current one framed in the accent; hidden slide's number struck and dimmed; Ctrl+M inserts after it | [D] |
| Status bar, left | Slide *x* of *y*; language; Accessibility | [D] slide, language |
| Status bar, right | Notes, Comments, Normal, Slide Sorter, Reading View, Slide Show; zoom − slider + and percentage; Fit slide to current window | [D] buttons, [R] web slider |
| Right-click, a slide (canvas, nothing selected) | Cut, Copy, Paste · Layout ▸, Reset Slide · Format Background… · New Comment | [R] |
| Right-click, a thumbnail | Cut, Copy, Paste · New Slide, Duplicate Slide, Delete Slide · Add Section · Layout ▸, Reset Slide · Format Background… · Hide Slide · New Comment | [D] docs/50 §4 |
| Right-click, a shape | Cut, Copy, Paste · Edit Text, Edit Points · Group ▸, Bring to Front ▸, Send to Back ▸ · Link · Edit Alt Text… · Size and Position… · Format Shape… · New Comment | [R] |
| Right-click, text | Cut, Copy, Paste · Font…, Paragraph… · Bullets ▸, Numbering ▸ · Link · Format Shape… · New Comment | [R] |
| Keyboard | Shift+F10 or the Menu key opens the menu for the focus | [D] desktop |
| Format Shape pane | Right side, title and ×. **Shape Options** \| **Text Options**. Shape Options' three icons: Fill & Line (Fill: No, Solid, Gradient, Picture, Pattern, Slide background; Color, Transparency. Line: No, Solid, Gradient; Color, Transparency, Width, Dash, Cap, Join), Effects (Shadow, Reflection, Glow, Soft Edges, 3-D Format, 3-D Rotation), Size & Properties (Size: Height, Width, Rotation, Lock aspect ratio; Position: Horizontal, Vertical; Text Box: alignment, direction, autofit, margins, wrap; Alt Text) | [D] docs/50 §18, [R] layout |

## Keyboard (web table, docs/49 S3)

| Keys | Does | Keys | Does |
| --- | --- | --- | --- |
| Ctrl+M | New slide | Ctrl+D | Duplicate object |
| Ctrl+Z, Ctrl+Y | Undo, redo | Ctrl+X, C, V | Cut, copy, paste |
| Ctrl+B, I, U | Bold, italic, underline | Ctrl+L, E, R, J | Align left, centre, right, justify |
| Ctrl+Shift+>, < | Font size up, down | Ctrl+Space | Clear formatting |
| Ctrl+K | Link | Ctrl+F | Find |
| Ctrl+[ , ] | Back, forward one layer (web) | Ctrl+G, Ctrl+Shift+G | Group, ungroup |
| F5, Shift+F5, Alt+F5 | Present from start, current, presenter | Alt+Q | Search |
| Ctrl+F1 | Show or hide the ribbon | Ctrl+F6 | Next region |
| Shift+F10 | Context menu | Alt+Period | KeyTips (web) |
| Delete | Delete the selection | Arrows, Ctrl+arrows | Nudge, nudge finer |
