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

## File (Backstage)

| Page | Contents | Ev. |
| --- | --- | --- |
| Home, New | Blank Presentation, then templates (ours) | [D] |
| Open | Browse this device | [D] |
| Info | Properties beside the page: Slides and Words, then Title, Tags, Comments, Categories, Subject, Company, Manager edited in place; Related Dates; Related People (Author, Last Modified By). Advanced Properties: Summary, Statistics, Custom. What this build leaves out (ours) | [D] |
| Save As | Save, Save As (name and type: .pptx, .potx, .ppsx; F12), Download a Copy, Rename | [D] |
| Export | Download as PDF, PNG of the slide | [D] |
| Print | Print, Ctrl+P: the browser's print, one slide a page | [D] |
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

## Co-authoring: who is here and where

Read on 2026-11-08 from Microsoft's "Work together on PowerPoint presentations"
(<https://support.microsoft.com/office/0c30ee3f-8674-4f0e-97be-89cf2892a34d>) and "Share and collaborate in
PowerPoint" (<https://support.microsoft.com/en-us/office/share-and-collaborate-in-powerpoint-07fcfb59-f410-4e52-9c60-3b069a805ee5>).

| Region | Target | Ev. | Built (`COL-04`) |
| --- | --- | --- | --- |
| Header, right end | A badge with each present person's picture or initials; "their thumbnail (image or initials) will appear in the top-right corner of the ribbon" (Windows), "a badge ... for each person who currently has the file open" (web) | [D] | Initials in each person's colour, beside Share |
| The badges, selected | "Select that badge to see where in the slide deck they currently are" (web); the Share pane also names everyone viewing or editing | [D] | A list: name, colour, slide; a name goes to their slide |
| Thumbnails | "a label appear at the top right corner of the slide thumbnail with the name of the person currently on that slide" (web); an identity icon on the thumbnail, hovered for the name (Windows) | [D] | Name label in their colour at the thumbnail's top right |
| The slide | "a border around the affected area" (Windows); "Colored flags show where each person is editing" (Share and collaborate page) | [D] | Outline in their colour round their selection, and a flag with their name on their caret or selection |
| Joining, leaving | "PowerPoint alerts you when people enter or leave your presentation" (Mac) | [D] Mac only | The badges change; no alert |
| Cursor name labels as such | Not found on Microsoft's pages: none describe a remote text caret | none | Built as a flag on the caret, ours |

No presence in the status bar: the footer roster this build had first is removed.

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
