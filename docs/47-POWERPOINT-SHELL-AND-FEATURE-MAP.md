# 47 — The PowerPoint shell and feature map

**Superseded on scope by [48](48-POWERPOINT-FEATURE-STUDY.md), and phases by [06](06-ROADMAP-AND-DELIVERY.md).** This page remains the UI sketch.

**Provenance warning.** This inventory is written from recall of PowerPoint for the web, not
measured from the running product, and Microsoft changes it regularly. It is good enough to plan
phases. It is not good enough to build pixels from. `UX-001` replaces it with a measured target and
an owner-approved prototype, as OpenCalc's docs/95 did for Excel. **Where the live product disagrees
with this page, the live product wins.**

## The rule

Same structure, behaviour, keyboard, dialogs and wording, so a PowerPoint user needs no teaching.

| Adopted from PowerPoint | Ours |
| --- | --- |
| Title bar, search, Comments, Share, Present | Name and logo |
| Tab set, per-tab groups and their order; contextual tabs; collapse order | Accent colour (`DEC-03`), not PowerPoint's orange-red |
| Thumbnail rail, canvas, notes pane, status bar, view buttons, zoom | Icons: Fluent UI System Icons (MIT), as OpenCalc |
| File page structure; shortcuts and KeyTips | Templates, themes and help content: all authored by us |
| Font metrics through compatible faces | Inter for UI; open faces for slide text |

Microsoft's themes, templates, stock content and icon artwork are **not** copied. We author our own.

The shell reuses OpenCalc's `ribbon.js`, `ribbon.model.js` pattern, `commands.js` registry and token
spine. Classic and Simplified ribbon both exist, and phones get the bottom command bar. Web
PowerPoint defaults to Simplified, and that default is one thing `UX-001` should confirm.

## Main window

```
┌ Title bar: logo · file name · search · Comments · Share · Present ─────────┐
├ Tabs · Ribbon ─────────────────────────────────────────────────────────────┤
│ Thumbnail │                                                    │ Task pane │
│ rail      │        Slide canvas (fit to window, on gray)       │ (format,  │
│ sections  │                                                    │ animation,│
│ drag      ├────────────────────────────────────────────────────┤ comments) │
│           │ Notes pane (resizable)                             │           │
├ Status: Slide 3 of 24 · language · Notes · view buttons · zoom · Fit ──────┤
└────────────────────────────────────────────────────────────────────────────┘
```

Views: Normal, Slide Sorter, Reading, Slide Show, Presenter; Master when editing masters.

## Ribbon and phase

Level is Edit unless marked. Phase numbers are from [06](06-ROADMAP-AND-DELIVERY.md).

| Tab | Groups and verbs | Phase |
| --- | --- | --- |
| **File** | New, Open, Info, Save a Copy, Download (PDF, ODP), Print, Share, Version History, Options | 3, 9 |
| **Home** | Undo; Clipboard (Paste options, Cut, Copy, Format Painter); Slides (New, Layout, Reset, Section, Duplicate, Hide) | 4 |
| | Font (family, size, grow, shrink, clear, B/I/U/S, case, spacing, highlight, colour); Paragraph (bullets, numbers, indent, spacing, align, columns, direction, vertical align) | 5 |
| | Drawing (Shapes, Arrange, Quick Styles, Fill, Outline, Effects); Editing (Find, Replace, Select) | 4, 5 |
| **Insert** | New Slide; Tables; Pictures; Shapes; Text Box, WordArt, Header and Footer; Link; Comment | 4–6, 9 |
| | Icons (our set); Chart; SmartArt (Render, Phase 6; edit later); Video, Audio; Equation (Render), Symbol; Action | 6 |
| **Draw** | Pens, highlighter, eraser, lasso, ink to shape | 6 |
| **Design** | Themes, Variants, Slide Size, Format Background | 7 |
| **Transitions** | Preview, gallery, Effect Options, duration, Apply to All, Advance | 8 |
| **Animations** | Preview, gallery, Effect Options, Add Animation, Animation Pane, Start, Duration, Delay, Reorder, Trigger | 8 |
| **Slide Show** | From Beginning, From Current, Set Up, Custom Show, Hide Slide, Rehearse Timings, Captions (host seam) | 8 |
| **Review** | Spelling (host seam), Accessibility Checker, Comments, Show Changes | 5, 9, 10 |
| **View** | Normal, Sorter, Reading, Masters, Notes, Ruler, Gridlines, Guides, Zoom, Fit | 3, 7 |
| **Help** | Help, Shortcuts, Feedback (our content) | 3 |
| **Shape Format** (contextual) | Insert shapes, Edit Points, Quick Styles, Fill, Outline, Effects, WordArt styles, Arrange, Size | 4 |
| **Picture Format** (contextual) | Crop, Corrections, Recolour, Transparency, Effects, Picture Styles, Alt Text, Arrange, Size | 4, 6 |
| **Table Design, Table Layout** (contextual) | Styles, Borders, Shading, Rows and Columns, Merge, Align, Size | 6 |
| **Chart** (contextual) | Change Type, Edit Data, Elements, Styles | 6 |
| **Video, Audio** (contextual) | Play, Trim, Start, Volume, Poster | 6 |

Out of scope pending `DEC-05`: Designer and Copilot. A verb we lack is left off the ribbon.

## On-canvas behaviour

Eight resize handles, rotate handle, yellow adjust handles. Smart guides and equal-spacing hints.
Shift constrains; Ctrl-drag duplicates; arrows nudge, Ctrl+arrows nudge finer. Marquee and Shift-click selection. Connector
snapping to sites that follow when a shape moves. Double-click a shape to edit text; Esc steps out of
text, then deselects. Placeholder prompts and icons on empty placeholders. Paste keeps source or
destination formatting. Tab cycles objects. Right-click menus per object kind with the mini-toolbar.

## Shortcuts to hold exactly

Recalled from desktop. **The web's recorded list is [spec/web-shortcuts.md](spec/web-shortcuts.md) and differs: Ctrl+[ sends back there, for one.** Ctrl+M new slide; Ctrl+D duplicate; Ctrl+G and Ctrl+Shift+G group and ungroup; Ctrl+Shift+C and V format
painter; F5 and Shift+F5 present; Ctrl+[ and ] font size; Ctrl+B, I, U; Ctrl+L, E, R, J align; Ctrl+K link;
Ctrl+F, Ctrl+H find and replace. In show: Space, N, Enter, arrows advance; P and Backspace go back; B and W
blank; number + Enter jump; Esc ends. Browsers reserve some of these. How the web product handles that
is part of `UX-001`, along with its KeyTips.
