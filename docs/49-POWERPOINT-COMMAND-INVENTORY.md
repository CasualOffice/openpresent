# 49 — PowerPoint command inventory

Checked 2026-11-07. Generated from Microsoft's published control-ID workbooks, then overlaid with KeyTips, shortcuts and web facts from dated sources. No signed-in session was used. Where the live product disagrees, the live product wins.

## What this cannot tell you

**Not obtainable from public sources:**

| Gap | Effect |
| --- | --- |
| Control size (large or small) | No source publishes it. The Type column gives the control kind only. Size needs screenshots or a live session. |
| Visible labels per command | The workbook gives IDs, not labels. IDs are the record here. Group labels are decamelised and marked `I`. |
| A full KeyTip list | Microsoft says every command has one but publishes only part. Documented and Observed KeyTips are filled in; a blank cell means no source, not no KeyTip. |
| A full web command list | Microsoft publishes only the 171 control IDs that add-ins may reuse (S2), last changed 2021-01-07. `?` means no source says either way. Absence from S2 is not evidence of absence. |
| Anything after version 2402 in the workbook | S1 is the newest published: Current Channel 2402, committed 2024-03-18. It is 32 months old. Later additions known from other sources are in [Later than the workbook](#later-than-the-workbook). |
| Group collapse order when the window narrows | Not documented for desktop or web. |
| File page contents as rendered | S1 lists IDs for desktop; web has key paths only. |
| Labels and items for object context menus on the web | No source. |

**Only a live signed-in session can confirm:**

1. The web tab set and each web tab's commands, in both ribbon modes, for the current build.
2. Which of the old and new groups in Record really show on desktop.
3. Simplified or Classic as the web default. Docs [47](47-POWERPOINT-SHELL-AND-FEATURE-MAP.md) assumes Simplified; no source here states the default.
4. Every web conflict below: audio, charts, equations, WordArt, Replace, Presenter View, Slide Show tab, rulers, outline.
5. Control sizes, labels, tooltips, and the KeyTips left blank.
6. Whether a command is gated by licence, tenant policy or rollout ring. Copilot items especially.

Microsoft's web support pages contradict one another. S8 carries an August 2026 date stamp and still mentions Silverlight and Flash, so a recent date stamp does not make a statement current.

## Evidence

Each fact carries `D` (a source states it), `O` (a dated screenshot or tutorial shows it) or `I` (inferred here), plus a source ID.

**Reading the tables.** `Dt` is desktop presence: `Y` means the row is in S1 and so `D S1` for Windows desktop, version 2402. `Web` is `Yes`, `No` or `?`. Row order is source order; S1 does not state that this is the visual order, so treating it as such is `I`. Groups repeated across tabs are printed once and then referenced, only when the rows match exactly. An ID listed twice in a group is in the source twice, usually a state variant. Web key paths follow Alt+Windows logo key or Alt+Period (`D S3`, S16).

Types: S1 `button`, `toggleButton` as toggle, `splitButton` as split, `gallery`, `menu` as dropdown, `dropDown` as dropdown-list, `comboBox` as combo, `checkBox` as check, `labelControl` as label, `button (dialogBoxLauncher)` as launcher, `control` left as is.

### Sources

Page date is the page's own `ms.date` stamp or stated date. All were fetched 2026-11-07.

| ID | Source | Page date |
| --- | --- | --- |
| S1 | [Fluent UI identifiers, Microsoft 365 Current Channel, powerpointcontrols.xlsx](https://github.com/OfficeDev/office-fluent-ui-command-identifiers/blob/main/Microsoft%20365/Current%20Channel/powerpointcontrols.xlsx): 5230 rows | Version 2402, committed 2024-03-18 |
| S2 | [office-control-ids, PowerPoint on the web Ribbon IDs.xlsx](https://github.com/OfficeDev/office-control-ids/tree/main/PowerPoint): 171 controls, 48 single-line groups, 51 multiline groups | Committed 2021-01-07 |
| S3 | [Keyboard shortcuts to create presentations](https://support.microsoft.com/en-us/office/use-keyboard-shortcuts-to-create-powerpoint-presentations-ebb3d20e-dcd4-444f-a38e-bb5c5ed180f4) | 2026-01-30 |
| S4 | [Keyboard shortcuts to deliver presentations](https://support.microsoft.com/en-us/office/use-keyboard-shortcuts-to-deliver-powerpoint-presentations-1524ffce-bd2a-45f4-9a7f-f18b992b93a0) | 2026-11-05 |
| S5 | [Use the keyboard with the ribbon](https://support.microsoft.com/en-us/office/use-the-keyboard-to-work-with-the-ribbon-954cd3f7-2f77-4983-978d-c09b20e31f0e) | 2026-01-21 |
| S6 | [File menu shortcuts](https://support.microsoft.com/en-us/topic/keyboard-shortcuts-for-the-file-menu-in-office-for-windows-1c4b07af-be2e-4c19-8d33-c600a8904043) | 2026-02-19 |
| S7 | [Compare features across platforms](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b) | 2026-08-19 |
| S8 | [How features behave on the web](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint) | 2026-08-19, content partly stale |
| S9 | [Web service description](https://learn.microsoft.com/en-us/office365/servicedescriptions/office-online-service-description/powerpoint-online) | 2023-04-25 |
| S10 | [Simplified Ribbon on the web](https://support.microsoft.com/topic/f05d2d91-ad15-44b0-b401-e85735b409cb) | 2025-11-18 |
| S11 | [Show or hide the ribbon](https://support.microsoft.com/en-us/office/show-or-hide-the-ribbon-in-office-d946b26e-0c8c-402d-a0f7-c6efa296b527) | 2026-03-26 |
| S12 | [Customize the Quick Access Toolbar](https://support.microsoft.com/en-us/office/customize-the-quick-access-toolbar-43fff1c9-ebc4-4963-bdbd-c2b6b0739e52) | 2026-03-20 |
| S13 | [Customize the ribbon](https://support.microsoft.com/en-us/office/customize-the-ribbon-in-office-00f24ca7-6021-48d3-9514-a31a460ecb31) | 2026-04-02 |
| S14 | [Where are the menus and toolbars](https://support.microsoft.com/en-us/office/where-are-the-menus-and-toolbars-e25451c0-8a1f-428c-afb4-d91e98807bd4) | 2025-10-30 |
| S15 | [Search in Microsoft 365 apps](https://support.microsoft.com/office/find-what-you-need-with-microsoft-search-in-office-2457d4d8-48a8-4ad4-ab89-5a0657aa8446) | 2026-04-17 |
| S16 | [Screen reader: explore and navigate](https://support.microsoft.com/en-us/accessibility/powerpoint/use-a-screen-reader-to-explore-and-navigate-powerpoint) | 2025-01-01 |
| S17 | [Screen reader: basic tasks](https://support.microsoft.com/en-us/accessibility/powerpoint/basic-tasks-to-create-a-presentation-in-powerpoint-with-a-screen-reader) | 2025-01-01 |
| S18 | [Screen reader: text](https://support.microsoft.com/en-us/accessibility/powerpoint/use-a-screen-reader-to-add-and-format-text-in-powerpoint) | 2025-01-01 |
| S19 | [Screen reader: pictures and tables](https://support.microsoft.com/en-us/accessibility/powerpoint/use-a-screen-reader-to-insert-and-edit-pictures-and-tables-in-powerpoint) | 2026-01-08 |
| S20 | [Screen reader: audio and video](https://support.microsoft.com/en-us/accessibility/powerpoint/use-a-screen-reader-to-insert-audio-or-video-in-powerpoint) | 2025-01-01 |
| S21 | [Screen reader: charts](https://support.microsoft.com/en-us/accessibility/powerpoint/use-a-screen-reader-to-create-charts-in-powerpoint) | 2026-07-06 |
| S22 | [Screen reader: slides](https://support.microsoft.com/en-us/accessibility/powerpoint/use-a-screen-reader-to-work-with-slides-in-powerpoint) | 2025-01-01 |
| S23 | [Screen reader: notes and comments](https://support.microsoft.com/en-us/accessibility/powerpoint/use-a-screen-reader-to-read-or-add-speaker-notes-and-comments-in-powerpoint) | 2025-01-01 |
| S24 | [Screen reader: print](https://support.microsoft.com/en-us/accessibility/powerpoint/use-a-screen-reader-to-print-presentations-in-powerpoint) | 2025-01-01 |
| S25 | [Screen reader: save](https://support.microsoft.com/en-us/accessibility/powerpoint/use-a-screen-reader-to-save-your-presentation-in-powerpoint) | 2025-01-01 |
| S26 | [Screen reader: share and co-author](https://support.microsoft.com/en-us/accessibility/powerpoint/use-a-screen-reader-to-share-and-co-author-presentations-in-powerpoint) | 2025-01-01 |
| S29 | [Microsoft 365 Apps release notes, Current Channel](https://support.microsoft.com/en-us/office/what-s-new-in-microsoft-365-95c8d81d-08ba-42c1-914f-bca4603e1426) | 2026-11-04 |
| M1 | [MC1419795, archive of Message Center](https://mc.merill.net/message/MC1419795) | Updated 2026-07-20 |
| M2 | [MC1461152, archive of Message Center](https://mc.merill.net/message/MC1461152) | Updated 2026-10-07 |
| B1 | [Microsoft 365 Insider blog: animating on the web](https://techcommunity.microsoft.com/blog/microsoft365insiderblog/a-new-way-to-animate-your-content-in-powerpoint-for-the-web/4223946) | 2024-08-25 |
| B2 | [Windows Central on the web Draw tab](https://www.windowscentral.com/software-apps/powerpoint-presentations-just-got-better-with-these-improvements) | 2023-06-12 |
| O1 | [Indezine: keyboard sequences, 365 for Windows](https://www.indezine.com/products/powerpoint/learn/customize/keyboard-sequences-ppt365.html) | 2019-12-11 |
| O2 | [Indezine: status bar, 365 for Windows](https://www.indezine.com/products/powerpoint/learn/interface/365/status-bar.html) | 2023-09-11 |
| O3 | [Indezine: Backstage, 365 for Windows](https://www.indezine.com/products/powerpoint/learn/interface/365/file-menu-backstage-view.html) | 2025-07-31 |
| O4 | [Indezine: ribbon and tabs, 365 for Windows](https://www.indezine.com/products/powerpoint/learn/interface/365/ribbon-and-tabs.html) | 2023-09-11 |
| O5 | [Indezine: status bar, web](https://www.indezine.com/products/powerpoint/learn/interface/powerpoint-online/status-bar.html) | 2023-09-22 |
| O6 | [Nuts & Bolts: where the Copilot button is](https://nutsandboltsspeedtraining.com/tutorials/where-is-copilot-in-powerpoint) | 2026-09 |

Tried without result: Computerworld's ribbon reference (the table is a gated download), Office Watch's KeyTip and Copilot articles (bot check), the OnMSFT article (blocked). A search summary of the Office Watch Copilot article is the only thing read from it, so it is not cited as a fact.

## Conflicts

Newest source preferred unless noted. None is settled without a live check.

| # | Topic | Sources disagree | Position |
| --- | --- | --- | --- |
| C1 | Switching ribbon mode on the web | S10 2025: a toggle labelled Simplified Ribbon, top right. S11 2026: Ribbon Display Options menu with Classic Ribbon and Single Line Ribbon. S16 2025: a switch announced "On, Simplified ribbon". S3 2026: Alt+Period, Z, R | Ribbon Display Options (S11, S3) |
| C2 | Web search box | S10: Tell me, near the right of the ribbon. S16: Search at top, may be named Tell Me. S9: Tell Me | Search, title bar area; verify |
| C3 | Adding audio and local video on the web | S7 and S8: no audio. S2 lists `SoundInsertMenu2`, S20 gives a web audio path. S7: video yes. S8: local video needs desktop | Audio no (S7), video yes (S7); verify |
| C4 | Charts, equations, WordArt on the web | S8, S9: no insert. No positive source found. S8 is partly stale | No; verify, high chance of being out of date |
| C5 | Replace on the web | S8: not available. S3 Web has only Find. S2 has only `FindDialog` | No; verify |
| C6 | Animation depth on the web | S9 2023: gallery, 37 animations, 8 transitions, no custom. B1 2024: Animation Pane exists | Pane yes; counts unreliable |
| C7 | Ctrl+D on desktop | S3 Windows table prints Ctrl+D for both closing a presentation and duplicating objects | Duplicate is the reliable reading. S3's Close entry looks wrong; confirm the Close key live |
| C8 | Presenter View and Broadcast on the web | S7, S8, S9: no. S2 (2021) lists `SlideShowUsePresenterView`, `BroadcastSlideShow` under web View | No (S7 is newest) |
| C9 | Headers and footers on the web | S9: no. S7, S8, S2: yes | Yes |
| C10 | Slide Show tab on the web | S2 has no such tab; `GroupStartSlideShowOnline` is under View. S17 gives S, B and S, C, which implies a tab with access key S. S9 mentions none | Tab probably exists; verify |
| C11 | Tab order | S1: Recording before Review. S3, S14: Record listed after View | S14 and S3 list tabs for description, not position. Use S1 order; verify |
| C12 | Copilot on desktop Home | S1 2402: `GroupAIAssistance` with Copilot. O6 2026: floats over the slide by default; Move to Top places it in Home | O6 |
| C13 | Selection Pane key | S3: Alt+H, S, L, P. O1 2019: Alt+H, G, P | Both are paths: Select then Selection Pane, Arrange then Selection Pane (`I`) |
| C14 | Ctrl+F1 on desktop | S3 Ribbon section: expand or collapse. S3 Views section: full screen. S5: collapse | Collapse |
| C15 | Transitions access key | S3: Alt+K. GoSkills, undated: Alt+T | Alt+K |
| C16 | Compare and Merge | S1 lists `ReviewCompareAndMerge` and `TabMerge`. S29, version 2502: disabled | Disabled; S1 predates it |
| C17 | Web download path | S8: File, Download As. S25: F, A, C and S26: F, S, P | Both key paths documented; rendered labels need a live check |
| C18 | Ruler, gridlines, outline on the web | S8: desktop only. S1 gives desktop IDs; S2 lists none on web | No; verify |

## Later than the workbook

Not in S1, so not in the tables.

| Item | Where | Fact | Evidence |
| --- | --- | --- | --- |
| Interactive Slide button | Home and Insert, desktop and web | Rollout mid to end October 2026 | D M2 |
| Review Presentation button | Review, web | Rollout late June to early July 2026 | D M1 |
| Visualize Slide button | Design, web | Rollout late June to early July 2026 | D M1 |
| Copilot button states | Floating (default), docked, or Home via Move to Top. Right-click: Dock, Move to Top, Undock, Move out of Ribbon | Alt+C activates when floating or docked. In ribbon: Alt+H, F, X hits only the lower split part | O O6 2026-09 |
| Compare and Merge | Review | Disabled, version 2502 | D S29 |
| QuickStarter | Not checked against a tab | Disabled, version 2410 | D S29 |
| Copilot edit | Copilot pane | Listed in release notes | D S29 |
| Shift+F8 | Copilot rewrite of a selected text box | Shortcut | D S3 |

## Shell

Sources are per row. Checked 2026-11-07.

### Ribbon modes and collapse

| Fact | Desktop | Web | Evidence |
| --- | --- | --- | --- |
| Display options | Full-screen mode, Show tabs only, Always show Ribbon | Classic Ribbon, Single Line Ribbon, Always show ribbon, Show tabs only, Adjust automatically | D S11 |
| Simplified or single line mode | Not offered among S11 desktop options | Yes | D S11; desktop absence I |
| Switch mode | n/a | Ribbon Display Options; Alt+Period, Z, R (Mac Ctrl+Period, Z, R) | D S11, S3; see C1 |
| Ribbon Display Options place | Lower right of ribbon | Right corner of ribbon | D S11 |
| Collapse | Ctrl+F1; right-click a tab, Collapse the ribbon; double-click tab | No Ctrl+F1 documented. Show tabs only, Adjust automatically | D S11, S5; O O4 for double-click |
| Restore when hidden | Ellipsis at top right, or Alt, in Full-screen mode | Move cursor to top; Ctrl+F6 or Win+Alt; on touch, scroll up or tap title bar | D S11 |
| Overflow on single line | n/a | Ellipsis buttons, can be several per tab, open more commands; arrows open similar ones | D S10 |
| Groups by mode | One set | Different: e.g. single-line Insert is one group `GroupInsertOnline`; multiline Insert has separate groups | D S2 |
| Move between groups | Ctrl+Left or Right in lower ribbon | Ctrl+Left or Right in single line; no Mac shortcut | D S5, S3 |
| Control-level collapse order | Not sourced | Not sourced | Gap |
| Dialog launcher | Corner of some groups; S1 lists 50 | Not sourced; `PowerPointParagraphDialog` listed in S2 | D S14, S1, S2 |
| Customization | File, Options, Customize Ribbon: add, rename, reorder tabs and groups; export, import, reset | Not sourced | D S13 |

### Focus and keyboard

| Fact | Desktop | Web | Evidence |
| --- | --- | --- | --- |
| Region cycle forward | F6: slide, status bar, ribbon tabs, notes if open, thumbnails | Ctrl+F6: slide, notes, status bar, title bar, ribbon, thumbnails. Comments pane if open | D S16, S3 |
| Region cycle back | Shift+F6 | Ctrl+Shift+F6 (Mac Shift+Cmd+F6) | D S3 |
| Show KeyTips | Alt or F10 | Alt+Period or Alt+Windows logo key (Mac Ctrl+Period) | D S3 |
| KeyTip back, cancel | Esc back one step; Alt cancels | Not sourced | D S5 |
| Ribbon tooltip | Ctrl+Shift+F10 | Not sourced | D S3 |
| Open split or list | Alt+Down | Alt+Down for lists; Tab to More for galleries | D S3, S5 |
| Context menu | Shift+F10 or Menu key | Not sourced | D S3 |
| Help on control | F1 | Browser owns F1 | D S3 |
| Mac KeyTips | Off by default; 16.89 or later | n/a | D S5 |

### Tabs and access keys

| Tab | Desktop | Web | Evidence |
| --- | --- | --- | --- |
| File | Alt+F | F | D S3, S16 |
| Home | Alt+H | H | D S3, S16 |
| Insert | Alt+N | N | D S3, S19 |
| Draw | Alt+J, I | ? | D S3; web tab O B2 2023 |
| Design | Alt+G | G | D S3, S17 |
| Transitions | Alt+K | ? | D S3; web exists D S2 |
| Animations | Alt+A | ? | D S3; web exists D S2 |
| Slide Show | Alt+S | S | D S3, S17; see C10 |
| Record | Alt+E | No | D S3, S7 |
| Review | Alt+R | R | D S3, S23 |
| View | Alt+W | W | D S3, S23 |
| Help | Alt+Y, 2 | ? | D S3 |
| Contextual | Alt+J, then letter | J, then letter | D S19, S20, S21 |
| Slide Master | Alt+M | No | D S17, S7 |
| Header area | Alt+Z, then letter | Z, then letter | D S26, S17 |

### Title bar

| Item | Desktop | Web | Evidence |
| --- | --- | --- | --- |
| Quick Access Toolbar | Upper left by default | Not sourced | D S12, S16 |
| AutoSave | Toggle `AutoSaveSwitch` | Saves automatically; no Save command | D S1; D S3, S9 |
| Search | Top of window, Alt+Q | Alt+Q | D S15, S3; see C2 |
| Share | Alt+Z, S | Tab from ribbon focus to Share; F, Z, S | D S26 |
| Open in desktop | n/a | Z, M, O | D S17 |
| Window controls | Upper right, with Ribbon display options first; IDs `WindowMinimize`, `WindowRestore`, `WindowClose` | Browser | D S16, S1 |
| Web top bar | n/a | Announced as "List of Microsoft services" in the focus ring, with the mode switch | D S16 |
| Rename | Not sourced | F, A, R | D S25 |
| Comments, Present, Editing mode buttons | Not sourced in current pages | Not sourced | Live only |

### Quick Access Toolbar

Source S1 lists the toolbar's stock items; behaviour from S12, S17. Checked 2026-11-07.

| # | Control ID | Type | Shortcut |
| --- | --- | --- | --- |
| 1 | AutoSaveSwitch | toggle |  |
| 2 | FileNewDefault | button | Ctrl+N [D S3] |
| 3 | FileOpenUsingBackstage | button | Ctrl+O [D S3] |
| 4 | FileSave | button | Ctrl+S [D S3] |
| 5 | FileSendAsAttachment | button |  |
| 6 | FilePrintQuick | button | Ctrl+P [D S3] |
| 7 | PrintPreviewAndPrint | button | Ctrl+P [D S3] |
| 8 | Spelling | button |  |
| 9 | Undo | gallery | Ctrl+Z [D S3] |
| 10 | RedoOrRepeat | button | Ctrl+Y [D S3] |
| 11 | SlideShowFromBeginning | button | F5 [D S3] |
| 12 | PointerModeOptions | gallery |  |

| Behaviour | Fact | Evidence |
| --- | --- | --- |
| Position | Above the ribbon by default, or below | D S12 |
| Labels | Show or hide command labels when below | D S12 |
| Hide | Right-click ribbon, or the customize menu | D S12 |
| Add | Right-click a command; or More Commands, including Commands Not in the Ribbon and All commands | D S12 |
| Contents limit | Commands only, not list contents; single line; no size option | D S12 |
| Keys | Items gain an access key by position (I; S3 only says adding to QAT creates one) | D S3, I |
| Export, import | Supported | D S12 |
| Web | No source for a QAT; S12 does not list web | Gap |

### Search

| Fact | Evidence |
| --- | --- |
| Finds commands, help, files, people, definitions; Get help on entry | D S15 |
| Alt+Q to focus, type, Down, Enter runs | D S3, S15, S16 |
| Named Tell Me on some builds | D S5, S16 |
| Web: runs commands the same way from Alt+Q | D S3 Web |

### Status bar

| Item | Desktop | Web | Evidence |
| --- | --- | --- | --- |
| Slide number, count | Yes | Yes | O O2, O5 2023 |
| Spell check indicator | Yes | Not shown in O5 | O O2 |
| Language | Option | Yes | O O5; O2 summary |
| Help Improve Office | Not shown | Yes | O O5 |
| Notes toggle | Yes | Yes | O O2, O5 |
| Comments | Not sourced | Not sourced | Live only |
| View buttons | Normal, Slide Sorter, Reading, Slide Show; Shift+click for masters | Editing, Reading, Slide Show | O O2, O5; Sorter on web D S8 suggests O5 is dated |
| Zoom slider | Yes | Not shown; Zoom to dialog | O O2, O5 |
| Zoom level | Opens Zoom dialog | Zoom to | O O2, O5 |
| Fit slide to window | Yes | Yes | O O2, O5 |
| Customize Status Bar menu | Right-click; Theme and Accessibility Checker off by default | Not sourced | O O2 |

## File

### Pages

Names O O3 2025-07-31. Keys D S6 2026-02-19, except where marked. Web D per row. IDs from S1; pairing a page name with its ID is `I`. Checked 2026-11-07.

| Page | S1 ID | Desktop key | Web key |
| --- | --- | --- | --- |
| Home | `PlaceTabHome` | Alt+F, H | Not sourced |
| New | `TabOfficeStart` | Alt+F, N | F, N [D S17] |
| Open | `TabRecent` | Alt+F, O | F, O [D S17] |
| Info | `TabInfo` | Alt+F, I | Not sourced |
| Save | `FileSave` | Alt+F, S | None; saves automatically [D S9] |
| Save As | `TabSave` | Alt+F, A; dialog A, 1 or A, O | F, A: rename R, download C [D S25] |
| Print | `TabPrint` | Alt+F, P | F, P: R, N, H [D S24] |
| Share | `TabShare`, also `ShareDocument` | Alt+F, Z [O O1 2019] | F, Z, S [D S26] |
| Export | `TabPublish` | Alt+F, E | F, S, P for PDF [D S26] |
| Close | `FileClose` | Not sourced | Not sourced |
| Account | `TabHelp` | Alt+F, D | Not sourced |
| Feedback | `TabOfficeFeedback`, `OfficeFeedback` | Alt+F, K | Not sourced |
| Options | `ApplicationOptionsDialog` | Alt+F, T | Not sourced |

### Inner keys, desktop

D S6 unless marked.

| Page | Action | Key |
| --- | --- | --- |
| New | Template from app list | Z, then 1 to 9 or A to P, then C |
| New | Search online templates | S |
| New | Predefined search | U, then 1 to 7 |
| Open | Search recent | R, E |
| Open | Recent files; recent folders | R, Y, 2; R, Y, 3 |
| Open | Shared files | Y, 1 |
| Open | Company OneDrive; SharePoint sites | S, 1; S, 2 |
| Open | Personal OneDrive; this device | K; C |
| Open | System Open dialog; unsaved recovery | O; U |
| Save As | Add cloud location; rename | A, A; A, Y, 3 |
| Print | Printer; range; slide numbers; orientation; paper | I; A; S; O; L |
| Print, S24 | Printer I; range A; layout H; duplex D; collate C; orientation O; color Y, 1 | With Alt+P prefix |
| Print | Print all at defaults | Alt+P, P [D S3] |
| Export | PDF or XPS; change file type; handouts in Word | A; C; H, A |
| Info | Copy path; open location; protect; inspect | C; F; P; I, I |
| Account | Photo; sign out; switch; updates | C; E; S; R |
| Options | Display; Accessibility; Proofing; theme | D; A; P; G, Tab, T |
| Any | Exit File | Esc |

### Web behaviour

| Fact | Evidence |
| --- | --- |
| Download As: pptx, pdf, odp, JPEG series | D S8 |
| Print: slides, notes pages, handouts | D S8, S24; D S7 |
| No password, IRM or digital signature editing | D S8, S9 |
| ppt.new or PowerPoint.new creates a deck | D S3 |
| Options, Account, Info pages: no source | Gap |

### Backstage controls

Source S1, checked 2026-11-07. Row number is the source position in the backstage block. Web: `?` throughout, per above.

| # | Control ID | Type | Page (Tab) | Group | Parent | Dt |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | NoTcid | control | (File menu item) |  |  | Y |
| 2 | (unnamed in source) | button | (File menu item) |  |  | Y |
| 3 | PlaceTabHome | tab | (File menu item) |  |  | Y |
| 4 | HomePageSlabCreateNew | group | PlaceTabHome |  |  | Y |
| 5 | (unnamed in source) | group | PlaceTabHome |  |  | Y |
| 6 | (unnamed in source) | control | PlaceTabHome |  |  | Y |
| 7 | HomePlaceRecommendedDocuments | group | PlaceTabHome |  |  | Y |
| 8 | GroupRecentDocuments | group | PlaceTabHome |  |  | Y |
| 9 | (unnamed in source) | button | PlaceTabHome | GroupRecentDocuments |  | Y |
| 10 | TabInfo | tab | (File menu item) |  |  | Y |
| 11 | GroupUploadErrors | group | TabInfo |  |  | Y |
| 12 | GroupUpdatesAvailable | group | TabInfo |  |  | Y |
| 13 | (unnamed in source) | group | TabInfo |  |  | Y |
| 14 | GroupBroadcastResumeWarning | group | TabInfo |  |  | Y |
| 15 | GroupBroadcastEndWarning | group | TabInfo |  |  | Y |
| 16 | GroupOptimizeCompatibility | group | TabInfo |  |  | Y |
| 17 | GroupCompressMedia | group | TabInfo |  |  | Y |
| 18 | GroupARMPreview | group | TabInfo |  |  | Y |
| 19 | GroupWorkflows | group | TabInfo |  |  | Y |
| 20 | GrupPolicyWarning | group | TabInfo |  |  | Y |
| 21 | GroupSecurityInformation | group | TabInfo |  |  | Y |
| 22 | GroupSecurityWarning | group | TabInfo |  |  | Y |
| 23 | (unnamed in source) | group | TabInfo |  |  | Y |
| 24 | AppGuardTrustPromotion | button | TabInfo |  |  | Y |
| 25 | GroupProtectedView | group | TabInfo |  |  | Y |
| 26 | GroupReadOnly | group | TabInfo |  |  | Y |
| 27 | GroupDownloading | group | TabInfo |  |  | Y |
| 28 | GroupReadOnlyServer | group | TabInfo |  |  | Y |
| 29 | GroupReadOnlyOffline | group | TabInfo |  |  | Y |
| 30 | GroupReadOnlyCheckOutRequired | group | TabInfo |  |  | Y |
| 31 | GroupCachedOfflineMergeable | group | TabInfo |  |  | Y |
| 32 | GroupReadOnlyCachedMergeable | group | TabInfo |  |  | Y |
| 33 | GroupReadOnlyCached | group | TabInfo |  |  | Y |
| 34 | GroupCheckIn | group | TabInfo |  |  | Y |
| 35 | FileCheckOutDiscard | button | TabInfo | GroupCheckIn |  | Y |
| 36 | GroupFileConvert | group | TabInfo |  |  | Y |
| 37 | UpgradeDocument | button | TabInfo | GroupFileConvert |  | Y |
| 38 | OfficePolicyTips | group | TabInfo |  |  | Y |
| 39 | GroupAllAuthors | group | TabInfo |  |  | Y |
| 40 | GroupSignatures | group | TabInfo |  |  | Y |
| 41 | GroupPermissions | group | TabInfo |  |  | Y |
| 42 | FileMarkAsFinal | control | TabInfo | GroupPermissions |  | Y |
| 43 | FileMarkAsReadOnlyRecommended | control | TabInfo | GroupPermissions |  | Y |
| 44 | FileDocumentEncrypt | control | TabInfo | GroupPermissions |  | Y |
| 45 | UsagePermissions | button | TabInfo | GroupPermissions |  | Y |
| 46 | FileAddDigitalSignature | button | TabInfo | GroupPermissions |  | Y |
| 47 | GroupPrepareForSharing | group | TabInfo |  |  | Y |
| 48 | AllowPersonallyIdentifiableInformation | control | TabInfo | GroupPrepareForSharing |  | Y |
| 49 | DocuemntVersionHistory | group | TabInfo |  |  | Y |
| 50 | GroupVersions | group | TabInfo |  |  | Y |
| 51 | GroupThumbnail | group | TabInfo |  |  | Y |
| 52 | GroupDocumentProperties | group | TabInfo |  |  | Y |
| 53 | (unnamed in source) | button | (File menu item) |  |  | Y |
| 54 | TabOfficeStart | tab | (File menu item) |  |  | Y |
| 55 | GroupOfficeStart | group | TabOfficeStart |  |  | Y |
| 56 | (unnamed in source) | button | (File menu item) |  |  | Y |
| 57 | TabRecent | tab | (File menu item) |  |  | Y |
| 58 | GroupOpenPlaces | task form | TabRecent |  |  | Y |
| 79 | GroupModernOpenPlaces | group | TabRecent |  |  | Y |
| 160 | OfficeExtensionsGetAddinsButtonOnBackstageBelowOpenDivider | button | (File menu item) |  |  | Y |
| 161 | FileSave | button | (File menu item) |  |  | Y |
| 162 | TabSave | tab | (File menu item) |  |  | Y |
| 163 | SaveGroup | task form | TabSave |  |  | Y |
| 178 | ModernSaveGroup | group | TabSave |  |  | Y |
| 250 | MoveDocument | button | (File menu item) |  |  | Y |
| 251 | HistoryTab | button | (File menu item) |  |  | Y |
| 252 | TabPrint | tab | (File menu item) |  |  | Y |
| 253 | GroupPrintProtectedView | group | TabPrint |  |  | Y |
| 254 | GroupDownloading | group | TabPrint |  |  | Y |
| 255 | GroupPrintSettings | group | TabPrint |  |  | Y |
| 256 | FilePrintCustomRange | control | TabPrint | GroupPrintSettings |  | Y |
| 257 | GroupPrintPreview | group | TabPrint |  |  | Y |
| 258 | ShareDocument | button | (File menu item) |  |  | Y |
| 259 | TabShare | tab | (File menu item) |  |  | Y |
| 260 | GroupShare | task form | TabShare |  |  | Y |
| 261 | Share | category | TabShare | GroupShare |  | Y |
| 262 | ShareWithPeopleFromSharePane | task | TabShare | GroupShare | Share | Y |
| 263 | SlabShareWithPeopleFromSharePane | group | TabShare | GroupShare | Share > ShareWithPeopleFromSharePane | Y |
| 264 | SendUsingEmail | task | TabShare | GroupShare | Share | Y |
| 265 | GroupSendUsingEmail | group | TabShare | GroupShare | Share > SendUsingEmail | Y |
| 266 | GroupSendAsAttachment | group | TabShare | GroupShare | Share > SendUsingEmail | Y |
| 267 | GroupSendAsLink | group | TabShare | GroupShare | Share > SendUsingEmail | Y |
| 268 | GroupSendAsPdf | group | TabShare | GroupShare | Share > SendUsingEmail | Y |
| 269 | GroupSendAsXps | group | TabShare | GroupShare | Share > SendUsingEmail | Y |
| 270 | GroupSendAsInternetFax | group | TabShare | GroupShare | Share > SendUsingEmail | Y |
| 271 | BroadcastSlideShowBackstage | task | TabShare | GroupShare | Share | Y |
| 272 | GroupBroadcast | group | TabShare | GroupShare | Share > BroadcastSlideShowBackstage | Y |
| 273 | FilePublishSlides | task | TabShare | GroupShare | Share | Y |
| 274 | GroupPublishSlides | group | TabShare | GroupShare | Share > FilePublishSlides | Y |
| 275 | FilePublishSlides | button | TabShare | GroupShare | Share > FilePublishSlides | Y |
| 276 | SendByInstantMessage | task | TabShare | GroupShare | Share | Y |
| 277 | GroupSendByInstantMessage | group | TabShare | GroupShare | Share > SendByInstantMessage | Y |
| 278 | Workflow | category | TabShare | GroupShare |  | Y |
| 279 | WorkflowStart | task | TabShare | GroupShare | Workflow | Y |
| 280 | GroupWorkflowStart | group | TabShare | GroupShare | Workflow > WorkflowStart | Y |
| 281 | WorkflowsUnavailable | task | TabShare | GroupShare | Workflow | Y |
| 282 | GroupWorkflowsUnavailable | group | TabShare | GroupShare | Workflow > WorkflowsUnavailable | Y |
| 283 | TabPublish | tab | (File menu item) |  |  | Y |
| 284 | PublishGroup | task form | TabPublish |  |  | Y |
| 285 | FileTypes | category | TabPublish | PublishGroup |  | Y |
| 286 | FileSaveAsPdfOrXps | task | TabPublish | PublishGroup | FileTypes | Y |
| 287 | GroupCreatePdfOrXpsDocumentSlab | group | TabPublish | PublishGroup | FileTypes > FileSaveAsPdfOrXps | Y |
| 288 | GroupCreatePdfOrXpsDocument | group | TabPublish | PublishGroup | FileTypes > FileSaveAsPdfOrXps | Y |
| 289 | ExportToVideo | task | TabPublish | PublishGroup | FileTypes | Y |
| 290 | GroupExportToVideo | group | TabPublish | PublishGroup | FileTypes > ExportToVideo | Y |
| 291 | ExportToAnimatedGif | task | TabPublish | PublishGroup | FileTypes | Y |
| 292 | GroupExportToAnimatedGif | group | TabPublish | PublishGroup | FileTypes > ExportToAnimatedGif | Y |
| 293 | FilePackageForCD | task | TabPublish | PublishGroup | FileTypes | Y |
| 294 | GroupPackageForCD | group | TabPublish | PublishGroup | FileTypes > FilePackageForCD | Y |
| 295 | CreateHandoutsInWord | task | TabPublish | PublishGroup | FileTypes | Y |
| 296 | GroupCreateHandoutsInWord | group | TabPublish | PublishGroup | FileTypes > CreateHandoutsInWord | Y |
| 297 | FileTypeChange | task | TabPublish | PublishGroup | FileTypes | Y |
| 298 | GroupFileSave | group | TabPublish | PublishGroup | FileTypes > FileTypeChange | Y |
| 299 | UploadToMSStream | task | TabPublish | PublishGroup | FileTypes | Y |
| 300 | UploadToMSStreamExportGroup | group | TabPublish | PublishGroup | FileTypes > UploadToMSStream | Y |
| 301 | FileClose | button | (File menu item) |  |  | Y |
| 302 | TabHelp | tab | (File menu item) |  |  | Y |
| 303 | GroupBackstageIdentity | group | TabHelp |  |  | Y |
| 304 | GroupBackstagePrivacy | group | TabHelp |  |  | Y |
| 305 | ManagePrivacy | button | TabHelp | GroupBackstagePrivacy |  | Y |
| 306 | Personalization | group | TabHelp |  |  | Y |
| 307 | Theming | group | TabHelp |  |  | Y |
| 308 | ManageServices | group | TabHelp |  |  | Y |
| 309 | GroupBackstageSignInDisabled | group | TabHelp |  |  | Y |
| 310 | GroupBackstageSignIn | group | TabHelp |  |  | Y |
| 311 | (unnamed in source) | button | TabHelp | GroupBackstageSignIn |  | Y |
| 312 | GroupOfficeBranding | group | TabHelp |  |  | Y |
| 313 | GroupClickToRunUpdateStatus | group | TabHelp |  |  | Y |
| 314 | GroupUpdateChannel | group | TabHelp |  |  | Y |
| 315 | UpdateChannel | button | TabHelp | GroupUpdateChannel |  | Y |
| 316 | GroupOfficeInsider | group | TabHelp |  |  | Y |
| 317 | GroupOfficeInsiderRegistration | button | TabHelp | GroupOfficeInsider |  | Y |
| 318 | GroupOfficeInsiderChannel | button | TabHelp | GroupOfficeInsider |  | Y |
| 319 | GroupOfficeInsiderAbout | button | TabHelp | GroupOfficeInsider |  | Y |
| 320 | GroupOfficeInsiderHelp | button | TabHelp | GroupOfficeInsider |  | Y |
| 321 | GroupAboutOfficeProducts | group | TabHelp |  |  | Y |
| 322 | GroupWhatsNewInOfficeProducts | group | TabHelp |  |  | Y |
| 323 | (unnamed in source) | button | (File menu item) |  |  | Y |
| 324 | TabOfficeFeedback | tab | (File menu item) |  |  | Y |
| 325 | GroupOfficeSendSmile | group | TabOfficeFeedback |  |  | Y |
| 326 | GroupOfficeSendFrown | group | TabOfficeFeedback |  |  | Y |
| 327 | GroupOfficeSendSuggestion | group | TabOfficeFeedback |  |  | Y |
| 328 | OfficeFeedback | button | (File menu item) |  |  | Y |
| 329 | OfficeFeedbackBackstage | button | (File menu item) |  |  | Y |
| 330 | ApplicationOptionsDialog | button | (File menu item) |  |  | Y |


`TabRecent` / `GroupOpenPlaces`, 20 rows in order: ButtonCategoryRecentDocuments (category), ButtonTaskRecentDocuments (task under ButtonCategoryRecentDocuments), GroupRecentDocuments (group under ButtonCategoryRecentDocuments), (unnamed) (control under ButtonCategoryRecentDocuments), ButtonOpenBrowse (button under ButtonCategoryRecentDocuments), GroupRecoverUnsavedFiles (group under ButtonCategoryRecentDocuments), ButtonTaskFixItHub (task under ButtonCategoryRecentDocuments), FixItHub (group under ButtonCategoryRecentDocuments), ButtonTaskSharedWithMe (task under ButtonCategoryRecentDocuments), GroupSharedWithMeDocuments (group under ButtonCategoryRecentDocuments), (unnamed) (button under ButtonCategoryRecentDocuments), ButtonTaskDynamicServiceProvider (task under ButtonCategoryRecentDocuments), GroupRecentPlaces (group under ButtonCategoryRecentDocuments), (unnamed) (control under ButtonCategoryRecentDocuments), (unnamed) (group under ButtonCategoryRecentDocuments), GroupGettingStartedOpenMRUPlacesOrNavSlab (group under ButtonCategoryRecentDocuments), (unnamed) (button under ButtonCategoryRecentDocuments), SkyDriveUpsellGroup (group under ButtonCategoryRecentDocuments), LoadingServicesGroup (group under ButtonCategoryRecentDocuments), GroupAddLocation (group under ButtonCategoryRecentDocuments).

`TabRecent` / `GroupModernOpenPlaces`, 80 rows in order: ButtonGroupRecentDocuments (control), ButtonGroupCloudServices (control), ButtonCategoryCloudServices (category), ButtonTaskRecentDocuments (task under ButtonCategoryCloudServices), GroupRecentDocumentsNew (group under ButtonCategoryCloudServices), (unnamed) (control under ButtonCategoryCloudServices), ButtonOpenBrowse (button under ButtonCategoryCloudServices), ButtonTaskFixItHub (task under ButtonCategoryCloudServices), CloudFixItHub (group under ButtonCategoryCloudServices), ButtonTaskSharedWithMe (task under ButtonCategoryCloudServices), GroupSharedWithMeDocumentsNew (group under ButtonCategoryCloudServices), (unnamed) (button under ButtonCategoryCloudServices), ButtonTaskDynamicServiceProvider (task under ButtonCategoryCloudServices), GroupRecentCloudPlaces (group under ButtonCategoryCloudServices), (unnamed) (control under ButtonCategoryCloudServices), (unnamed) (group under ButtonCategoryCloudServices), GroupCloudGettingStartedOpenMRUPlacesOrNavSlab (group under ButtonCategoryCloudServices), (unnamed) (button under ButtonCategoryCloudServices), ButtonTaskDynamicServiceProvider (control under ButtonCategoryCloudServices), CloudSkyDriveUpsellGroup (group under ButtonCategoryCloudServices), LoadingCloudServicesGroup (group under ButtonCategoryCloudServices), ButtonTaskRecentDocuments (task), GroupRecentDocumentsNew (group under ButtonTaskRecentDocuments), (unnamed) (control under ButtonTaskRecentDocuments), ButtonOpenBrowse (button under ButtonTaskRecentDocuments), GroupRecentDocumentsNew (group), (unnamed) (control under GroupRecentDocumentsNew), ButtonOpenBrowse (button under GroupRecentDocumentsNew), (unnamed) (control), ButtonOpenBrowse (button), ButtonTaskFixItHub (task), CloudFixItHub (group under ButtonTaskFixItHub), CloudFixItHub (group), ButtonTaskSharedWithMe (task), GroupSharedWithMeDocumentsNew (group under ButtonTaskSharedWithMe), (unnamed) (button under ButtonTaskSharedWithMe), GroupSharedWithMeDocumentsNew (group), (unnamed) (button under GroupSharedWithMeDocumentsNew), (unnamed) (button), ButtonTaskDynamicServiceProvider (task), GroupRecentCloudPlaces (group under ButtonTaskDynamicServiceProvider), (unnamed) (control under ButtonTaskDynamicServiceProvider), (unnamed) (group under ButtonTaskDynamicServiceProvider), GroupCloudGettingStartedOpenMRUPlacesOrNavSlab (group under ButtonTaskDynamicServiceProvider), (unnamed) (button under ButtonTaskDynamicServiceProvider), GroupRecentCloudPlaces (group), (unnamed) (control under GroupRecentCloudPlaces), (unnamed) (group), GroupCloudGettingStartedOpenMRUPlacesOrNavSlab (group), (unnamed) (button under GroupCloudGettingStartedOpenMRUPlacesOrNavSlab), ButtonTaskDynamicServiceProvider (control), GroupRecentPinnableCloudPlaces (group), (unnamed) (control under GroupRecentPinnableCloudPlaces), (unnamed) (group), PinnableGroupCloudGettingStartedOpenMRUPlacesOrNavSlab (group), (unnamed) (button under PinnableGroupCloudGettingStartedOpenMRUPlacesOrNavSlab), CloudSkyDriveUpsellGroup (group under ButtonTaskDynamicServiceProvider), (unnamed) (button under ButtonTaskDynamicServiceProvider), CloudSkyDriveUpsellGroup (group), (unnamed) (button under CloudSkyDriveUpsellGroup), LoadingCloudServicesGroup (group under ButtonTaskDynamicServiceProvider), LoadingCloudServicesGroup (group), ButtonGroupLocalServices (control), ButtonCategoryLocalServices (category), ButtonTaskDynamicServiceProvider (task under ButtonCategoryLocalServices), GroupRecentLocalPlaces (group under ButtonCategoryLocalServices), (unnamed) (control under ButtonCategoryLocalServices), GroupLocalGettingStartedOpenMRUPlacesOrNavSlab (group under ButtonCategoryLocalServices), (unnamed) (button under ButtonCategoryLocalServices), GroupAddLocationLocal (group under ButtonCategoryLocalServices), GroupRecentLocalPlaces (group under ButtonTaskDynamicServiceProvider), (unnamed) (control under ButtonTaskDynamicServiceProvider), GroupLocalGettingStartedOpenMRUPlacesOrNavSlab (group under ButtonTaskDynamicServiceProvider), (unnamed) (button under ButtonTaskDynamicServiceProvider), GroupRecentLocalPlaces (group), (unnamed) (control under GroupRecentLocalPlaces), GroupLocalGettingStartedOpenMRUPlacesOrNavSlab (group), (unnamed) (button under GroupLocalGettingStartedOpenMRUPlacesOrNavSlab), GroupAddLocationLocal (group under ButtonTaskDynamicServiceProvider), GroupAddLocationLocal (group).

`TabSave` / `SaveGroup`, 14 rows in order: Save (category), (unnamed) (task under Save), (unnamed) (group under Save), AppGuardTrustPromotion (button under Save), ButtonTaskDynamicServiceProvider (task under Save), GroupDynamicServiceProvider (group under Save), TabSave (control under Save), SlabSaveAsAccessibility (group under Save), (unnamed) (group under Save), GroupGettingStartedSaveAsMRUPlacesOrNavSlab (group under Save), (unnamed) (button under Save), SkyDriveUpsellGroup (group under Save), LoadingServicesGroup (group under Save), GroupAddLocation (group under Save).

`TabSave` / `ModernSaveGroup`, 71 rows in order: ButtonGroupsSave (control), ButtonGroupItemSaveCloudServices (control), SaveCloudServices (category), (unnamed) (task under SaveCloudServices), (unnamed) (group under SaveCloudServices), AppGuardTrustPromotion (button under SaveCloudServices), ButtonTaskDynamicServiceProvider (task under SaveCloudServices), GroupDynamicSaveCloudServiceProvider (group under SaveCloudServices), TabSave (control under SaveCloudServices), SlabCloudSaveAsAccessibility (group under SaveCloudServices), (unnamed) (group under SaveCloudServices), GroupGettingStartedSaveAsMRUCloudPlacesOrNavSlab (group under SaveCloudServices), (unnamed) (button under SaveCloudServices), ButtonTaskDynamicServiceProvider (control under SaveCloudServices), CloudSkyDriveUpsellGroup (group under SaveCloudServices), LoadingCloudServicesGroup (group under SaveCloudServices), (unnamed) (task), (unnamed) (group), AppGuardTrustPromotion (button), ButtonTaskDynamicServiceProvider (task), GroupDynamicSaveCloudServiceProvider (group under ButtonTaskDynamicServiceProvider), TabSave (control under ButtonTaskDynamicServiceProvider), SlabCloudSaveAsAccessibility (group under ButtonTaskDynamicServiceProvider), (unnamed) (group under ButtonTaskDynamicServiceProvider), GroupGettingStartedSaveAsMRUCloudPlacesOrNavSlab (group under ButtonTaskDynamicServiceProvider), (unnamed) (button under ButtonTaskDynamicServiceProvider), GroupDynamicSaveCloudServiceProvider (group), TabSave (control under GroupDynamicSaveCloudServiceProvider), TabSave (control), SlabCloudSaveAsAccessibility (group), (unnamed) (group), GroupGettingStartedSaveAsMRUCloudPlacesOrNavSlab (group), (unnamed) (button under GroupGettingStartedSaveAsMRUCloudPlacesOrNavSlab), (unnamed) (button), ButtonTaskDynamicServiceProvider (control), GroupDynamicSavePinnableCloudServiceProvider (group), TabSave (control under GroupDynamicSavePinnableCloudServiceProvider), GroupPinnableCloudSaveAsAccessibility (group), (unnamed) (group), GroupPinnableGettingStartedSaveAsMRUCloudPlacesOrNav (group), (unnamed) (button under GroupPinnableGettingStartedSaveAsMRUCloudPlacesOrNav), CloudSkyDriveUpsellGroup (group under ButtonTaskDynamicServiceProvider), (unnamed) (button under ButtonTaskDynamicServiceProvider), CloudSkyDriveUpsellGroup (group), (unnamed) (button under CloudSkyDriveUpsellGroup), LoadingCloudServicesGroup (group under ButtonTaskDynamicServiceProvider), LoadingCloudServicesGroup (group), ButtonGroupItemSaveLocalServices (control), LocalSave (category), ButtonTaskDynamicServiceProvider (task under LocalSave), GroupDynamicSaveLocalServiceProvider (group under LocalSave), TabSave (control under LocalSave), SlabLocalSaveAsAccessibility (group under LocalSave), (unnamed) (group under LocalSave), GroupGettingStartedSaveAsMRULocalPlacesOrNavSlab (group under LocalSave), (unnamed) (button under LocalSave), GroupAddLocationLocal (group under LocalSave), GroupDynamicSaveLocalServiceProvider (group under ButtonTaskDynamicServiceProvider), TabSave (control under ButtonTaskDynamicServiceProvider), SlabLocalSaveAsAccessibility (group under ButtonTaskDynamicServiceProvider), (unnamed) (group under ButtonTaskDynamicServiceProvider), GroupGettingStartedSaveAsMRULocalPlacesOrNavSlab (group under ButtonTaskDynamicServiceProvider), (unnamed) (button under ButtonTaskDynamicServiceProvider), GroupDynamicSaveLocalServiceProvider (group), TabSave (control under GroupDynamicSaveLocalServiceProvider), SlabLocalSaveAsAccessibility (group), (unnamed) (group), GroupGettingStartedSaveAsMRULocalPlacesOrNavSlab (group), (unnamed) (button under GroupGettingStartedSaveAsMRULocalPlacesOrNavSlab), GroupAddLocationLocal (group under ButtonTaskDynamicServiceProvider), GroupAddLocationLocal (group).
## Core ribbon tabs


### Home

ID `TabHome`. KeyTip: Alt+H [D S3]. Source S1, checked 2026-11-07.

Groups in source order: `GroupClipboard`, `GroupSlides`, `GroupFont`, `GroupParagraph`, `GroupDrawing`, `GroupEditing`, `GroupVoiceTools`, `GroupActivation`, `GroupClassifyLabelProtect`, `GroupDesignerOptions`, `GroupOfficeExtensionsAddinFlyout`, `GroupAIAssistance`.

**GroupClipboard** (I: "Clipboard")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | PasteMenu | split |  | Y | ? |  | Ctrl+V [D S3]; web Ctrl+V [D S3 Web] |
| 2 | Paste | button | PasteMenu | Y | Yes [D S2] |  | Ctrl+V [D S3]; web Ctrl+V [D S3 Web] |
| 3 | PasteGallery | gallery | PasteMenu | Y | ? |  |  |
| 4 | PasteSpecialDialog | button | PasteMenu > PasteGallery | Y | ? |  | Ctrl+Alt+V [D S3] |
| 5 | Cut | button |  | Y | Yes [D S2] |  | Ctrl+X [D S3]; web Ctrl+X [D S3 Web] |
| 6 | CopySplitButton | split |  | Y | ? |  | Ctrl+C [D S3]; web Ctrl+C [D S3 Web] |
| 7 | Copy | button | CopySplitButton | Y | Yes [D S2] |  | Ctrl+C [D S3]; web Ctrl+C [D S3 Web] |
| 8 | PasteDuplicate | button | CopySplitButton | Y | Yes [D S2] |  | Ctrl+D [D S3]; web Ctrl+D [D S3 Web] |
| 9 | FormatPainter | control |  | Y | Yes [D S2] |  | Ctrl+Shift+C / Ctrl+Shift+V [D S3] |
| 10 | ShowClipboard | launcher |  | Y | Yes [D S2] | Alt+H, F, O [D S3] |  |

**GroupSlides** (I: "Slides")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | SlideNewGallery | gallery |  | Y | Yes [D S2] | Alt+H, I [D S17]; web H, I [D S17 Web] | Ctrl+M [D S3]; web Ctrl+M [D S3 Web] |
| 2 | DuplicateSelectedSlides | button | SlideNewGallery | Y | ? |  | Ctrl+Shift+D [D S3] |
| 3 | SlidesFromOutline | button | SlideNewGallery | Y | ? | Alt+H, I, L [O O1 2019] |  |
| 4 | SlidesReuseSlides | toggle | SlideNewGallery | Y | ? | Alt+H, I, R [O O1 2019] |  |
| 5 | ReuseSlides | toggle | SlideNewGallery | Y | Yes [D S2] |  |  |
| 6 | PasteAsSlide | toggle | SlideNewGallery | Y | ? |  |  |
| 7 | ReuseSlides | toggle |  | Y | Yes [D S2] |  |  |
| 8 | SlideLayoutGallery | gallery |  | Y | Yes [D S2] | Alt+H, L [D S3] |  |
| 9 | SlideReset | button |  | Y | ? | Alt+H, Q [O O1 2019] |  |
| 10 | SectionMenu | dropdown |  | Y | ? | Alt+H, T, 1 [D S22] |  |
| 11 | SectionAdd | button | SectionMenu | Y | ? |  |  |
| 12 | SectionRename | button | SectionMenu | Y | ? |  |  |
| 13 | SectionMergeWithPrevious | button | SectionMenu | Y | ? |  |  |
| 14 | SectionRemoveAll | button | SectionMenu | Y | ? |  |  |
| 15 | SectionCollapseAll | button | SectionMenu | Y | ? |  |  |
| 16 | SectionExpandAll | button | SectionMenu | Y | ? |  |  |

**GroupFont** (I: "Font")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | Font | combo |  | Y | Yes [D S2] |  |  |
| 2 | FontSize | combo |  | Y | Yes [D S2] | Alt+H, F, S [D S3] |  |
| 3 | FontSizeIncrease | button |  | Y | Yes [D S2] |  | Ctrl+Shift+> [D S3]; web Ctrl+Shift+> [D S3 Web] |
| 4 | FontSizeDecrease | button |  | Y | ? |  | Ctrl+Shift+< [D S3]; web Ctrl+Shift+< [D S3 Web] |
| 5 | ClearFormatting | button |  | Y | Yes [D S2] |  | Ctrl+Spacebar [D S3]; web Ctrl+Spacebar [D S3 Web] |
| 6 | Bold | toggle |  | Y | Yes [D S2] |  | Ctrl+B [D S3]; web Ctrl+B [D S3 Web] |
| 7 | Italic | toggle |  | Y | Yes [D S2] |  | Ctrl+I [D S3]; web Ctrl+I [D S3 Web] |
| 8 | Underline | toggle |  | Y | Yes [D S2] |  | Ctrl+U [D S3]; web Ctrl+U [D S3 Web] |
| 9 | Shadow | toggle |  | Y | ? |  |  |
| 10 | Strikethrough | toggle |  | Y | Yes [D S2] |  |  |
| 11 | CharacterSpacingGallery | gallery |  | Y | ? |  |  |
| 12 | ChangeCaseGallery | gallery |  | Y | Yes [D S2] |  | Shift+F3 [D S3] |
| 13 | TextHighlightColorPickerLicensed | gallery |  | Y | ? | Alt+H, T, C [D S18] |  |
| 14 | HighlightingStop | button | TextHighlightColorPickerLicensed | Y | Yes [D S2] |  |  |
| 15 | FontColorPicker | gallery |  | Y | Yes [D S2] | Alt+H, F, C [D S18] |  |
| 16 | FontColorMoreColorsDialogPowerPoint | button | FontColorPicker | Y | ? |  |  |
| 17 | EyedropperFillText | button | FontColorPicker | Y | ? |  |  |
| 18 | FontDialogPowerPoint | launcher |  | Y | ? |  | Ctrl+T or Ctrl+Shift+F [D S3] |

**GroupParagraph** (I: "Paragraph")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | BulletsGallery | gallery |  | Y | Yes [D S2] | Alt+H, U [D S18] |  |
| 2 | BulletsAndNumberingBulletsDialog | button | BulletsGallery | Y | ? |  |  |
| 3 | NumberingGallery | gallery |  | Y | Yes [D S2] | Alt+H, N [D S18] |  |
| 4 | BulletsAndNumberingNumberingDialog | button | NumberingGallery | Y | ? |  |  |
| 5 | IndentDecrease | button |  | Y | Yes [D S2] | Alt+H, A, O [D S18] |  |
| 6 | IndentIncrease | button |  | Y | Yes [D S2] | Alt+H, A, I [D S18] |  |
| 7 | LineSpacingGalleryPowerPoint | gallery |  | Y | Yes [D S2] |  |  |
| 8 | AlignLeft | toggle |  | Y | Yes [D S2] |  | Ctrl+L [D S3]; web Ctrl+L [D S3 Web] |
| 9 | AlignCenter | toggle |  | Y | Yes [D S2] |  | Ctrl+E [D S3]; web Ctrl+E [D S3 Web] |
| 10 | AlignRight | toggle |  | Y | Yes [D S2] |  | Ctrl+R [D S3]; web Ctrl+R [D S3 Web] |
| 11 | AlignJustify | toggle |  | Y | Yes [D S2] |  | Ctrl+J [D S3]; web Ctrl+J [D S3 Web] |
| 12 | AlignJustifyMenu | dropdown |  | Y | ? |  |  |
| 13 | AlignJustifyWithMixedLanguages | toggle | AlignJustifyMenu | Y | ? |  |  |
| 14 | AlignJustifyLow | toggle | AlignJustifyMenu | Y | ? |  |  |
| 15 | ParagraphDistributed | toggle |  | Y | ? |  |  |
| 16 | AlignJustifyThai | toggle |  | Y | ? |  |  |
| 17 | TextDirectionLeftToRight | toggle |  | Y | Yes [D S2] |  |  |
| 18 | TextDirectionRightToLeft | toggle |  | Y | Yes [D S2] |  |  |
| 19 | TableColumnsGallery | gallery |  | Y | ? |  |  |
| 20 | ParagraphMoreColumnsDialog | button | TableColumnsGallery | Y | ? |  |  |
| 21 | TextDirectionGallery | gallery |  | Y | ? |  |  |
| 22 | TextDirectionMoreOptionsDialog | button | TextDirectionGallery | Y | ? |  |  |
| 23 | TextAlignGallery | gallery |  | Y | Yes [D S2] |  |  |
| 24 | TextAlignMoreOptionsDialog | button | TextAlignGallery | Y | ? |  |  |
| 25 | ConvertToSmartArt | gallery |  | Y | ? |  |  |
| 26 | ConvertToSmartArtMoreSmartArtGraphicsDialog | button | ConvertToSmartArt | Y | ? |  |  |
| 27 | PowerPointParagraphDialog | launcher |  | Y | Yes [D S2] |  |  |

**GroupDrawing** (I: "Drawing")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | GalleryAllShapesAndTextboxes | gallery |  | Y | ? |  |  |
| 2 | ObjectsArrangeMenu | dropdown |  | Y | ? |  |  |
| 3 | ObjectBringToFront | button | ObjectsArrangeMenu | Y | Yes, web tab FormatShape [D S2] |  | web Ctrl+Shift+] [D S3 Web] |
| 4 | ObjectSendToBack | button | ObjectsArrangeMenu | Y | Yes, web tab FormatShape [D S2] |  | web Ctrl+Shift+[ [D S3 Web] |
| 5 | ObjectBringForward | button | ObjectsArrangeMenu | Y | Yes, web tab FormatShape [D S2] |  | Ctrl+Shift+] [D S3]; web Ctrl+] [D S3 Web] |
| 6 | ObjectSendBackward | button | ObjectsArrangeMenu | Y | Yes, web tab FormatShape [D S2] |  | Ctrl+Shift+[ [D S3]; web Ctrl+[ [D S3 Web] |
| 7 | ObjectsGroup | button | ObjectsArrangeMenu | Y | Yes, web tab FormatShape [D S2] |  | Ctrl+G [D S3] |
| 8 | ObjectsUngroup | button | ObjectsArrangeMenu | Y | Yes, web tab FormatShape [D S2] |  | Ctrl+Shift+G [D S3] |
| 9 | ObjectsRegroup | button | ObjectsArrangeMenu | Y | ? |  | Ctrl+Shift+J [D S3] |
| 10 | ObjectAlignMenu | dropdown | ObjectsArrangeMenu | Y | Yes, web tab FormatShape [D S2] | Alt+H, G, A [O O1 2019] |  |
| 11 | ObjectsAlignLeftSmart | button | ObjectsArrangeMenu > ObjectAlignMenu | Y | Yes, web tab FormatShape [D S2] | Alt+H, G, A, L [O O1 2019] |  |
| 12 | ObjectsAlignCenterHorizontalSmart | button | ObjectsArrangeMenu > ObjectAlignMenu | Y | Yes, web tab FormatShape [D S2] | Alt+H, G, A, C [O O1 2019] |  |
| 13 | ObjectsAlignRightSmart | button | ObjectsArrangeMenu > ObjectAlignMenu | Y | Yes, web tab FormatShape [D S2] | Alt+H, G, A, R [O O1 2019] |  |
| 14 | ObjectsAlignTopSmart | button | ObjectsArrangeMenu > ObjectAlignMenu | Y | Yes, web tab FormatShape [D S2] | Alt+H, G, A, T [O O1 2019] |  |
| 15 | ObjectsAlignMiddleVerticalSmart | button | ObjectsArrangeMenu > ObjectAlignMenu | Y | Yes, web tab FormatShape [D S2] | Alt+H, G, A, M [O O1 2019] |  |
| 16 | ObjectsAlignBottomSmart | button | ObjectsArrangeMenu > ObjectAlignMenu | Y | Yes, web tab FormatShape [D S2] | Alt+H, G, A, B [O O1 2019] |  |
| 17 | AlignDistributeHorizontally | button | ObjectsArrangeMenu > ObjectAlignMenu | Y | Yes [D S2] | Alt+H, G, A, H [O O1 2019] |  |
| 18 | AlignDistributeVertically | button | ObjectsArrangeMenu > ObjectAlignMenu | Y | Yes [D S2] | Alt+H, G, A, V [O O1 2019] |  |
| 19 | ObjectsAlignRelativeToContainerSmart | toggle | ObjectsArrangeMenu > ObjectAlignMenu | Y | Yes, web tab FormatShape [D S2] | Alt+H, G, A, A [O O1 2019] |  |
| 20 | ObjectsAlignSelectedSmart | toggle | ObjectsArrangeMenu > ObjectAlignMenu | Y | ? | Alt+H, G, A, O [O O1 2019] |  |
| 21 | ObjectRotateGallery | gallery | ObjectsArrangeMenu | Y | Yes, web tab FormatShape [D S2] |  |  |
| 22 | ObjectRotationOptionsDialog | button | ObjectsArrangeMenu > ObjectRotateGallery | Y | Yes, web tab FormatShape [D S2] |  |  |
| 23 | SelectionPane | toggle | ObjectsArrangeMenu | Y | Yes [D S2] | Alt+H, S, L, P [D S3; path Alt+H, G, P also O O1 2019] | Alt+F10 [D S3] |
| 24 | ShapeQuickStylesHome | gallery |  | Y | ? |  |  |
| 25 | ShapeStylesOtherThemeFillsGallery | gallery | ShapeQuickStylesHome | Y | ? |  |  |
| 26 | ShapeFillColorPicker | gallery |  | Y | Yes, web tab FormatShape [D S2] |  |  |
| 27 | ObjectFillMoreColorsDialog | button | ShapeFillColorPicker | Y | ? |  |  |
| 28 | EyedropperFill | button | ShapeFillColorPicker | Y | ? |  |  |
| 29 | ObjectPictureFill | button | ShapeFillColorPicker | Y | ? |  |  |
| 30 | GradientGallery | gallery | ShapeFillColorPicker | Y | ? |  |  |
| 31 | ShapeFillMoreGradientsDialog | button | ShapeFillColorPicker > GradientGallery | Y | ? |  |  |
| 32 | ShapeFillTextureGallery | gallery | ShapeFillColorPicker | Y | ? |  |  |
| 33 | MoreTextureOptions | button | ShapeFillColorPicker > ShapeFillTextureGallery | Y | ? |  |  |
| 34 | ShapeOutlineColorPicker | gallery |  | Y | Yes, web tab FormatShape [D S2] |  |  |
| 35 | ObjectBorderOutlineColorMoreColorsDialog | button | ShapeOutlineColorPicker | Y | ? |  |  |
| 36 | EyedropperOutline | button | ShapeOutlineColorPicker | Y | ? |  |  |
| 37 | OutlineWeightGallery | gallery | ShapeOutlineColorPicker | Y | ? |  |  |
| 38 | LineStylesDialog | button | ShapeOutlineColorPicker > OutlineWeightGallery | Y | ? |  |  |
| 39 | GallerySketchyShapes | gallery | ShapeOutlineColorPicker | Y | ? |  |  |
| 40 | MoreLines2 | button | ShapeOutlineColorPicker > GallerySketchyShapes | Y | ? |  |  |
| 41 | OutlineDashesGallery | gallery | ShapeOutlineColorPicker | Y | ? |  |  |
| 42 | LineStylesDialog | button | ShapeOutlineColorPicker > OutlineDashesGallery | Y | ? |  |  |
| 43 | ArrowStyleGallery | gallery | ShapeOutlineColorPicker | Y | ? |  |  |
| 44 | ArrowsMore | button | ShapeOutlineColorPicker > ArrowStyleGallery | Y | ? |  |  |
| 45 | ShapeEffectsMenu | dropdown |  | Y | ? |  |  |
| 46 | ObjectEffectPresetGallery | gallery | ShapeEffectsMenu | Y | ? |  |  |
| 47 | _3DBevelOptionsDialog | button | ShapeEffectsMenu > ObjectEffectPresetGallery | Y | ? |  |  |
| 48 | ObjectEffectShadowGallery | gallery | ShapeEffectsMenu | Y | ? |  |  |
| 49 | ShadowOptionsDialog | button | ShapeEffectsMenu > ObjectEffectShadowGallery | Y | ? |  |  |
| 50 | ReflectionGallery | gallery | ShapeEffectsMenu | Y | ? |  |  |
| 51 | ReflectionsMoreOptions | button | ShapeEffectsMenu > ReflectionGallery | Y | ? |  |  |
| 52 | ObjectEffectGlowGallery | gallery | ShapeEffectsMenu | Y | ? |  |  |
| 53 | GlowColorPicker | gallery | ShapeEffectsMenu > ObjectEffectGlowGallery | Y | ? |  |  |
| 54 | GlowColorMoreColorsDialog | button | ShapeEffectsMenu > ObjectEffectGlowGallery | Y | ? |  |  |
| 55 | EyedropperGlow | button | ShapeEffectsMenu > ObjectEffectGlowGallery | Y | ? |  |  |
| 56 | GlowsMoreOptions | button | ShapeEffectsMenu > ObjectEffectGlowGallery | Y | ? |  |  |
| 57 | ObjectEffectSoftEdgesGallery | gallery | ShapeEffectsMenu | Y | ? |  |  |
| 58 | SoftEdgesMoreOptions | button | ShapeEffectsMenu > ObjectEffectSoftEdgesGallery | Y | ? |  |  |
| 59 | BevelShapeGallery | gallery | ShapeEffectsMenu | Y | ? |  |  |
| 60 | _3DBevelOptionsDialog | button | ShapeEffectsMenu > BevelShapeGallery | Y | ? |  |  |
| 61 | _3DRotationGallery | gallery | ShapeEffectsMenu | Y | ? |  |  |
| 62 | _3DRotationOptionsDialog | button | ShapeEffectsMenu > _3DRotationGallery | Y | ? |  |  |
| 63 | ObjectFormatDialog | launcher |  | Y | ? |  |  |

**GroupEditing** (I: "Editing")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | FindDialog | button |  | Y | Yes [D S2] |  | Ctrl+F [D S3]; web Ctrl+F [D S3 Web] |
| 2 | ReplaceMenu | split |  | Y | No [D S8]; see C5 |  | Ctrl+H [D S3] |
| 3 | ReplaceDialog | button | ReplaceMenu | Y | No [D S8]; see C5 |  | Ctrl+H [D S3] |
| 4 | FontsReplaceFonts | button | ReplaceMenu | Y | No [D S8]; see C5 |  |  |
| 5 | SelectMenu | dropdown |  | Y | ? |  |  |
| 6 | SelectAll | button | SelectMenu | Y | ? |  | Ctrl+A [D S3]; web Ctrl+A [D S3 Web] |
| 7 | ObjectsSelect | toggle | SelectMenu | Y | ? |  |  |
| 8 | SelectionPane | toggle | SelectMenu | Y | Yes [D S2] | Alt+H, S, L, P [D S3; path Alt+H, G, P also O O1 2019] | Alt+F10 [D S3] |

**GroupVoiceTools** (I: "Voice Tools")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | DictationMenu | split |  | Y | ? |  |  |
| 2 | Dictate | toggle | DictationMenu | Y | ? |  |  |
| 3 | DictateInMenu | button | DictationMenu | Y | ? |  |  |
| 4 | SetDictationLanguageGallery | gallery | DictationMenu | Y | ? |  |  |

**GroupActivation** (I: "Activation")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ActivateOffice | button |  | Y | ? |  |  |

**GroupClassifyLabelProtect** (I: "Classify Label Protect")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ClassifyLabelProtect | dropdown |  | Y | Yes [D S2] |  |  |
| 2 | (unnamed in source) | gallery | ClassifyLabelProtect | Y | ? |  |  |
| 3 | (unnamed in source) | toggle | ClassifyLabelProtect | Y | ? |  |  |
| 4 | (unnamed in source) | gallery | ClassifyLabelProtect | Y | ? |  |  |
| 5 | (unnamed in source) | gallery | ClassifyLabelProtect | Y | ? |  |  |
| 6 | (unnamed in source) | button | ClassifyLabelProtect | Y | ? |  |  |
| 7 | (unnamed in source) | button | ClassifyLabelProtect | Y | ? |  |  |

**GroupDesignerOptions** (I: "Designer Options")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | DesignerPane | toggle |  | Y | Yes [D S2] |  |  |

**GroupOfficeExtensionsAddinFlyout** (I: "Office Extensions Addin Flyout")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | OfficeExtensionsShowAddinFlyout | button |  | Y | ? |  |  |
| 2 | OfficeExtensionsShowAddinFlyout | toggle |  | Y | ? |  |  |

**GroupAIAssistance** (I: "AIAssistance")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | DesignerPane | toggle |  | Y | Yes [D S2] |  |  |
| 2 | CopilotDraftSlides | button |  | Y | ? |  |  |
| 3 | Copilot | button |  | Y | ? | Alt+H, F, X reaches lower part of split only [O O6 2026-09] |  |

Web evidence for this tab (S2, web tab `Home`). Single-line groups: `GroupClipboard`, `GroupDesignerHome`, `GroupDrawing`, `GroupEditing`, `GroupFont`, `GroupParagraph`, `GroupSlides`, `GroupVoiceTools`. Multiline groups: `GroupClipboard`, `GroupDrawing`, `GroupEditing`, `GroupFont`, `GroupParagraph`, `GroupSlides`, `GroupVoiceTools`, `GroupDesignerHome`.

Web control IDs with no match on this desktop tab: `ChangeCaseLowerOnline`, `ChangeCaseSentenceOnline`, `ChangeCaseToggleOnline`, `ChangeCaseUpperOnline`, `DeleteOnline`, `FontSizeDecrese`, `LineSpacing`, `OutlineColorPicker`, `Redo`, `ShapeStyleGallery`, `SlideHide`, `Subscript`, `Superscript`, `TextHiglightColorPickerLicensed`, `Undo`, `UndoRedoOnline`.


### Insert

ID `TabInsert`. KeyTip: Alt+N [D S3]. Source S1, checked 2026-11-07.

Groups in source order: `GroupSlides2`, `GroupInsertTables`, `GroupImages`, `GroupChunkCameoCamera`, `GroupInsertIllustrations`, `GroupContent`, `GroupPowerBI`, `GroupMSForms`, `GroupOfficeExtension`, `GroupInsertLinks`, `GroupInsertComments`, `GroupInsertText`, `GroupInsertSymbols`, `GroupInsertMediaClips`.

**GroupSlides2** (I: "Slides2")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | SlideNewGalleryInsert | gallery |  | Y | Yes [D S2] | web N, S, I [D S22 Web] | Ctrl+M [D S3] |
| 2 | DuplicateSelectedSlides | button | SlideNewGalleryInsert | Y | ? |  | Ctrl+Shift+D [D S3] |
| 3 | SlidesFromOutline | button | SlideNewGalleryInsert | Y | ? |  |  |
| 4 | SlidesReuseSlides | toggle | SlideNewGalleryInsert | Y | ? |  |  |
| 5 | ReuseSlides | toggle | SlideNewGalleryInsert | Y | Yes, web tab Home [D S2] |  |  |
| 6 | PasteAsSlide | toggle | SlideNewGalleryInsert | Y | ? |  |  |
| 7 | ReuseSlides | toggle |  | Y | Yes, web tab Home [D S2] |  |  |

**GroupInsertTables** (I: "Insert Tables")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | TableInsertGallery | gallery |  | Y | Yes [D S2] | Alt+N, T [D S19]; web N, T [D S19 Web] |  |
| 2 | TableInsert | button | TableInsertGallery | Y | ? |  |  |
| 3 | TableDrawTable | toggle | TableInsertGallery | Y | ? |  |  |
| 4 | ExcelSpreadsheetInsert | button | TableInsertGallery | Y | ? | Alt+N, T, X [O O1 2019] |  |

**GroupImages** (I: "Images")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | FlyoutAnchorInsertPictures | gallery |  | Y | ? | Alt+N, P [D S19]; web N, F [D S19 Web] |  |
| 2 | PictureInsertFromFilePowerPoint | button | FlyoutAnchorInsertPictures | Y | ? | Alt+N, P, D [D S3]; web N, F, P [D S19 Web] |  |
| 3 | InsertMobilePicture | button | FlyoutAnchorInsertPictures | Y | ? |  |  |
| 4 | InsertM365Picture | button | FlyoutAnchorInsertPictures | Y | ? |  |  |
| 5 | ClipArtInsertDialog | button | FlyoutAnchorInsertPictures | Y | ? | Alt+N, P, O (online pictures; ID mapping I) [D S19]; web N, F, F (Bing; ID mapping I) [D S19 Web] |  |
| 6 | ScreenshotInsertGallery | gallery |  | Y | ? |  |  |
| 7 | ScreenClipping | button | ScreenshotInsertGallery | Y | ? |  |  |
| 8 | PhotoAlbumInsertMenu | split |  | Y | ? |  |  |
| 9 | PhotoAlbumInsert | button | PhotoAlbumInsertMenu | Y | ? |  |  |
| 10 | PhotoAlbumEdit | button | PhotoAlbumInsertMenu | Y | ? |  |  |

**GroupChunkCameoCamera** (I: "Chunk Cameo Camera")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | DropdownInsertCameo | split |  | Y | ? |  |  |
| 2 | InsertCameo | button | DropdownInsertCameo | Y | ? |  |  |
| 3 | InsertCameoAllSlides | button | DropdownInsertCameo | Y | ? |  |  |

**GroupInsertIllustrations** (I: "Insert Illustrations")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ShapesInsertGallery | gallery |  | Y | Yes, web tab FormatShape [D S2] | Alt+N, S, H [D S3] |  |
| 2 | IconInsertFromFile | button |  | Y | Yes [D S2] |  |  |
| 3 | Insert3DModelFallback | button |  | Y | ? |  |  |
| 4 | Insert3DModelDropdown | split |  | Y | ? |  |  |
| 5 | Insert3DModelDefault | button | Insert3DModelDropdown | Y | ? |  |  |
| 6 | Insert3DModelFromFile | button | Insert3DModelDropdown | Y | ? |  |  |
| 7 | Insert3DModelFromFileConsumer | button | Insert3DModelDropdown | Y | ? |  |  |
| 8 | Insert3DModelFromOnline | button | Insert3DModelDropdown | Y | ? |  |  |
| 9 | Insert3DModelFromOnlineConsumer | button | Insert3DModelDropdown | Y | ? |  |  |
| 10 | SmartArtInsert | button |  | Y | Yes [D S2] | Alt+N, M [D S19]; web N, M [D S19 Web] |  |
| 11 | ChartInsert | button |  | Y | No [D S8]; see C4 | Alt+N, C [D S21] |  |

**GroupContent** (I: "Content")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | Collector | toggle |  | Y | ? |  |  |

**GroupPowerBI** (I: "Power BI")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | InsertPowerBI | button |  | Y | ? |  |  |

**GroupMSForms** (I: "MSForms")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | InsertMSForms | button |  | Y | ? |  |  |

**GroupOfficeExtension** (I: "Office Extension")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | OfficeExtensionsAppStore | button |  | Y | Yes [D S2] |  |  |
| 2 | OfficeExtensionsGallery2 | gallery |  | Y | ? |  |  |
| 3 | OfficeExtensionsDialog | button | OfficeExtensionsGallery2 | Y | ? |  |  |
| 4 | OfficeExtensionsManageOtherAddins | button | OfficeExtensionsGallery2 | Y | ? |  |  |
| 5 | ExtensibilityExperimentation | button | OfficeExtensionsGallery2 | Y | ? |  |  |
| 6 | OfficeExtensionsFeaturedApp1 | button |  | Y | ? |  |  |
| 7 | OfficeExtensionsFeaturedApp2 | button |  | Y | ? |  |  |
| 8 | OfficeExtensionsFeaturedApp3 | button |  | Y | ? |  |  |
| 9 | OfficeExtensionsFeaturedApp4 | button |  | Y | ? |  |  |
| 10 | OfficeExtensionsFeaturedApp5 | button |  | Y | ? |  |  |
| 11 | OfficeExtensionsFeaturedApp6 | button |  | Y | ? |  |  |

**GroupInsertLinks** (I: "Insert Links")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | InsertZoomMenu | dropdown |  | Y | ? |  |  |
| 2 | MSPPTInsertTableofContents | button | InsertZoomMenu | Y | ? |  |  |
| 3 | SectionZoomInsert | button | InsertZoomMenu | Y | ? |  |  |
| 4 | SlideZoomInsert | button | InsertZoomMenu | Y | ? |  |  |
| 5 | InsertLinkGallery | gallery |  | Y | ? |  |  |
| 6 | (unnamed in source) | button | InsertLinkGallery | Y | ? |  |  |
| 7 | HyperlinkInsert | button |  | Y | Yes [D S2] |  | Ctrl+K [D S3]; web Ctrl+K [D S3 Web] |
| 8 | ActionInsert | button |  | Y | ? |  |  |

**GroupInsertComments** (I: "Insert Comments")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | InsertNewComment | button |  | Y | Yes [D S2] |  | Ctrl+Alt+M [D S3]; web Ctrl+Alt+M [D S3 Web] |

**GroupInsertText** (I: "Insert Text")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | TextBoxInsert | toggle |  | Y | Yes [D S2] | Alt+N, X [D S3] |  |
| 2 | TextBoxInsertMenu | split |  | Y | Yes, web tab FormatShape [D S2] |  |  |
| 3 | TextBoxInsertHorizontal | toggle | TextBoxInsertMenu | Y | ? |  |  |
| 4 | TextBoxInsertVertical | toggle | TextBoxInsertMenu | Y | ? |  |  |
| 5 | HeaderFooterInsert | button |  | Y | Yes [D S2] |  |  |
| 6 | WordArtInsertGallery | gallery |  | Y | No [D S8]; see C4 | Alt+N, W [D S3] |  |
| 7 | DateAndTimeInsert | button |  | Y | ? |  |  |
| 8 | NumberInsert | button |  | Y | Yes [D S2] |  |  |
| 9 | OleObjectctInsert | button |  | Y | No [D S8]; see C4 | Alt+N, J [D S3] |  |

**GroupInsertSymbols** (I: "Insert Symbols")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | InsertBuildingBlocksEquationsGallery | gallery |  | Y | No [D S8]; see C4 |  |  |
| 2 | EquationInsertNew | button | InsertBuildingBlocksEquationsGallery | Y | No [D S8]; see C4 |  | Alt+= [D S3] |
| 3 | InkEquation | button | InsertBuildingBlocksEquationsGallery | Y | No [D S8]; see C4 |  |  |
| 4 | SymbolInsert | button |  | Y | Yes [D S2] |  |  |

**GroupInsertMediaClips** (I: "Insert Media Clips")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | MovieInsert02 | dropdown |  | Y | Yes [D S2] | Alt+N, V [D S20] |  |
| 2 | InsertVideoFromDevice | button | MovieInsert02 | Y | ? | Alt+N, V, T (this device; ID mapping I) [D S20] |  |
| 3 | MovieFromFileInsert | button | MovieInsert02 | Y | ? |  |  |
| 4 | InsertM365Video | button | MovieInsert02 | Y | ? | Alt+N, V, S (stock; ID mapping I) [D S20] |  |
| 5 | MovieFromClipOrganizerInsert | button | MovieInsert02 | Y | ? | Alt+N, V, O (online; ID mapping I) [D S20]; web N, V, O (online video; ID mapping I) [D S20 Web] |  |
| 6 | SoundInsertMenu02 | dropdown |  | Y | No [D S7, S8]; S2 lists SoundInsertMenu2, see C3 | Alt+N, O [D S20] |  |
| 7 | SoundInsertFromFile | button | SoundInsertMenu02 | Y | No [D S7, S8]; S2 lists SoundInsertMenu2, see C3 | Alt+N, O, P [D S20] |  |
| 8 | SoundRecord | button | SoundInsertMenu02 | Y | No [D S7, S8]; S2 lists SoundInsertMenu2, see C3 | Alt+N, O, R [D S20] |  |
| 9 | ObjectScreenRecording | button |  | Y | ? |  |  |

Web evidence for this tab (S2, web tab `Insert`). Single-line groups: `GroupInsertOnline`. Multiline groups: `GroupImages`, `GroupInsertComments`, `GroupInsertIllustrations`, `GroupInsertLinks`, `GroupInsertMediaClips`, `GroupInsertSymbols`, `GroupInsertTables`, `GroupInsertText`, `GroupSlides2`, `GroupReuseSlidesOnline`, `GroupInsertFooterOnline`.

Web control IDs with no match on this desktop tab: `FlyoutAnchorInsertPicture`, `SoundInsertMenu2`.


### Draw

ID `TabDrawInk`. KeyTip: Alt+J, I [D S3]; web Draw tab seen 2023 [O B2]. Source S1, checked 2026-11-07.

Groups in source order: `GroupInputMode`, `GroupWrite`, `GroupDrawingTools`, `GroupPensOneNote`, `GroupPens2`, `GroupDrawWithTouch`, `GroupStencils`, `GroupInkConvertOneNote`, `InkReplay`, `GroupInkClose`, `GroupPenAndInkHelp`.

**GroupInputMode** (I: "Input Mode")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | InkSelect | toggle |  | Y | ? |  |  |
| 2 | FingerPaintingMode | toggle |  | Y | ? |  |  |
| 3 | DrawInkMode | toggle |  | Y | ? |  |  |

**GroupWrite** (I: "Write")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | InkSelect | toggle |  | Y | ? |  |  |
| 2 | FingerPaintingMode | toggle |  | Y | ? |  |  |
| 3 | DrawInkMode | toggle |  | Y | ? |  |  |
| 4 | InkEraserMenu | split |  | Y | ? |  |  |
| 5 | CurrentEraser | toggle | InkEraserMenu | Y | ? |  |  |
| 6 | InkEraser | toggle | InkEraserMenu | Y | ? |  |  |
| 7 | PointEraserSmall | toggle | InkEraserMenu | Y | ? |  |  |
| 8 | PointEraserMedium | toggle | InkEraserMenu | Y | ? |  |  |
| 9 | SegmentEraser | toggle | InkEraserMenu | Y | ? |  |  |
| 10 | LassoSelect | toggle |  | Y | ? |  |  |

**GroupDrawingTools** (I: "Drawing Tools")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | InkToolboxWithEraserAndLasso | control |  | Y | ? |  |  |

**GroupPensOneNote** (I: "Pens One Note")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | InkToolbox | control |  | Y | ? |  |  |

**GroupPens2** (I: "Pens2")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | PensGallery2 | gallery |  | Y | ? |  |  |
| 2 | PenAddFavorite | button | PensGallery2 | Y | ? |  |  |
| 3 | InkColorPicker2 | gallery |  | Y | ? |  |  |
| 4 | InkColorMoreColorsDialog | button | InkColorPicker2 | Y | ? |  |  |
| 5 | EyedropperOutline | button | InkColorPicker2 | Y | ? |  |  |
| 6 | LineThickness2 | gallery |  | Y | ? |  |  |
| 7 | LineStylesDialog | button | LineThickness2 | Y | ? |  |  |

**GroupDrawWithTouch** (I: "Draw With Touch")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | FingerPaintingMode | toggle |  | Y | ? |  |  |

**GroupStencils** (I: "Stencils")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ShowRulerStencil | toggle |  | Y | ? |  |  |

**GroupInkConvertOneNote** (I: "Ink Convert One Note")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | DiagrammingOnline | toggle |  | Y | ? |  |  |
| 2 | InkToTextAnalysis | toggle |  | Y | ? |  |  |
| 3 | InkToShapeAnalysis | toggle |  | Y | ? |  |  |
| 4 | InkToTextAnalysis | button |  | Y | ? |  |  |
| 5 | InkToShapeAnalysis | button |  | Y | ? |  |  |
| 6 | SelectionToMathConvert | button |  | Y | ? |  |  |
| 7 | InkToMathSplitButton | split |  | Y | ? |  |  |
| 8 | InkToMathAnalysis | toggle | InkToMathSplitButton | Y | ? |  |  |
| 9 | InkToMathAnalysis | button | InkToMathSplitButton | Y | ? |  |  |
| 10 | MathInputEditor | button | InkToMathSplitButton | Y | ? |  |  |

**InkReplay** (I: "Ink Replay")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | Replay | toggle |  | Y | ? |  |  |

**GroupInkClose** (I: "Ink Close")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | InkStopInkingReadingView | button |  | Y | ? |  |  |

**GroupPenAndInkHelp** (I: "Pen And Ink Help")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | PenAndInkHelp | button |  | Y | ? |  |  |


### Design

ID `TabDesign`. KeyTip: Alt+G [D S3]. Source S1, checked 2026-11-07.

Groups in source order: `GroupSlideThemes`, `GroupThemeVariants`, `GroupCustomizeThemeOptions`, `GroupDesignerOptions`.

**GroupSlideThemes** (I: "Slide Themes")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | SlideThemesGallery | gallery |  | Y | Yes [D S2] | Alt+G, H [D S3]; web G, H [D S17 Web] |  |
| 2 | ThemeSearchOfficeOnlinePowerPoint | button | SlideThemesGallery | Y | ? |  |  |
| 3 | ThemeBrowseForThemesPowerPoint | button | SlideThemesGallery | Y | ? | Alt+G, H, M [O O1 2019] |  |
| 4 | ThemeSaveCurrentPowerPoint | button | SlideThemesGallery | Y | ? | Alt+G, H, S [O O1 2019] |  |

**GroupThemeVariants** (I: "Theme Variants")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ThemeVariantsGallery | gallery |  | Y | Yes [D S2] |  |  |
| 2 | ThemeColorsGallery | gallery | ThemeVariantsGallery | Y | ? |  |  |
| 3 | ThemeColorsCreateNew | button | ThemeVariantsGallery > ThemeColorsGallery | Y | ? | Alt+G, V, C, C [O O1 2019] |  |
| 4 | ThemeColorsReset | button | ThemeVariantsGallery > ThemeColorsGallery | Y | ? |  |  |
| 5 | ThemeFontsGallery | gallery | ThemeVariantsGallery | Y | ? |  |  |
| 6 | ThemeFontsCreateNew | button | ThemeVariantsGallery > ThemeFontsGallery | Y | ? | Alt+G, V, F, C [O O1 2019] |  |
| 7 | ThemeEffectsGallery | gallery | ThemeVariantsGallery | Y | ? |  |  |
| 8 | SlideBackgroundStylesGallery | gallery | ThemeVariantsGallery | Y | ? |  |  |
| 9 | SlideBackgroundFormatDialog | button | ThemeVariantsGallery > SlideBackgroundStylesGallery | Y | Yes [D S2] |  |  |
| 10 | SlideBackgroundReset | button | ThemeVariantsGallery > SlideBackgroundStylesGallery | Y | ? |  |  |

**GroupCustomizeThemeOptions** (I: "Customize Theme Options")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | PowerPointPageSetup | gallery |  | Y | ? | Alt+G, S [D S22] |  |
| 2 | CustomSlideSize | button | PowerPointPageSetup | Y | Yes [D S2] | Alt+G, S, C [D S22] |  |
| 3 | LaunchFormatBackground | button |  | Y | ? | Alt+G, T, F [O O1 2019] |  |

**GroupDesignerOptions** (I: "Designer Options")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | DesignerPane | toggle |  | Y | Yes, web tab Home [D S2] |  |  |

Web evidence for this tab (S2, web tab `Design`). Single-line groups: `GroupCustomizeThemeOptions`, `GroupDesignerOptions`, `GroupSetSlideOnline`, `GroupSlideThemes`, `GroupThemeVariants`. Multiline groups: `GroupDesignerOptions`, `GroupSlideThemes`, `GroupThemeVariants`.


### Transitions

ID `TabTransitions`. KeyTip: Alt+K [D S3]; S2 web IDs exist. Source S1, checked 2026-11-07.

Groups in source order: `GroupPreviewTransitions`, `GroupTransitionToThisSlide`, `GroupTransitionStyles`.

**GroupPreviewTransitions** (I: "Preview Transitions")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | TransitionPreview | button |  | Y | ? |  |  |

**GroupTransitionToThisSlide** (I: "Transition To This Slide")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | AnimationTransitionGallery | gallery |  | Y | ? |  |  |
| 2 | AnimationTransitionVariantGallery | gallery |  | Y | Yes [D S2] |  |  |

**GroupTransitionStyles** (I: "Transition Styles")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | AnimationTransitionSoundGallery | dropdown-list |  | Y | ? |  |  |
| 2 | TransitionSoundLoopUntilNextSound | toggle | AnimationTransitionSoundGallery | Y | ? |  |  |
| 3 | TransitionDuration | control |  | Y | ? |  |  |
| 4 | SlideTransitionApplyToAll | button |  | Y | Yes [D S2] |  |  |
| 5 | AdvanceSlideLabel | label |  | Y | ? |  |  |
| 6 | SlideTransitionOnMouseClick | check |  | Y | ? |  |  |
| 7 | SlideTransitionAutomaticallyAfter | check |  | Y | ? |  |  |
| 8 | TransitionTimeAutomaticallyAfter | control |  | Y | ? |  |  |

Web evidence for this tab (S2, web tab `Transitions`). Single-line groups: `GroupTransitionEffectOptionsOnline`, `GroupTransitionStyles`, `GroupTransitionToThisSlide`. Multiline groups: `GroupTransitionStyles`, `GroupTransitionToThisSlide`.

Web control IDs with no match on this desktop tab: `AnimationTrasitionGallery`.


### Animations

ID `TabAnimations`. KeyTip: Alt+A [D S3]. Source S1, checked 2026-11-07.

Groups in source order: `GroupPreview`, `GroupAnimations`, `GroupAnimationCustom`, `GroupAnimationTiming`.

**GroupPreview** (I: "Preview")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | AnimationPreviewMenu | split |  | Y | ? |  |  |
| 2 | AnimationPreview | button | AnimationPreviewMenu | Y | ? |  |  |
| 3 | AnimationAutoPreview | toggle | AnimationPreviewMenu | Y | ? |  |  |

**GroupAnimations** (I: "Animations")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | AnimationGallery | gallery |  | Y | Yes [D S2] |  |  |
| 2 | AnimationCustomEntranceDialog | button | AnimationGallery | Y | ? |  |  |
| 3 | AnimationCustomEmphasisDialog | button | AnimationGallery | Y | ? |  |  |
| 4 | AnimationCustomExitDialog | button | AnimationGallery | Y | ? |  |  |
| 5 | AnimationCustomPathDialog | button | AnimationGallery | Y | ? |  |  |
| 6 | AnimationCustomActionVerbDialog | button | AnimationGallery | Y | ? |  |  |
| 7 | EffectOptionsMenu | dropdown |  | Y | Yes [D S2] |  |  |
| 8 | EffectOptionsGallery | gallery | EffectOptionsMenu | Y | ? |  |  |
| 9 | EffectOptionsColorsGallery | gallery | EffectOptionsMenu | Y | ? |  |  |
| 10 | AnimationChangeGallery | gallery | EffectOptionsMenu | Y | ? |  |  |
| 11 | BuildOrderChangeGallery | gallery | EffectOptionsMenu | Y | ? |  |  |
| 12 | EffectOptionsDialog | launcher |  | Y | ? |  |  |

**GroupAnimationCustom** (I: "Animation Custom")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | AnimationAddGallery | gallery |  | Y | ? |  |  |
| 2 | AnimationCustomAddEntranceDialog | button | AnimationAddGallery | Y | ? |  |  |
| 3 | AnimationCustomAddEmphasisDialog | button | AnimationAddGallery | Y | ? |  |  |
| 4 | AnimationCustomAddExitDialog | button | AnimationAddGallery | Y | ? |  |  |
| 5 | AnimationCustomAddPathDialog | button | AnimationAddGallery | Y | ? |  |  |
| 6 | AnimationCustomAddActionVerbDialog | button | AnimationAddGallery | Y | ? |  |  |
| 7 | AnimationCustom | toggle |  | Y | Yes, Animation Pane [D B1 2024-08]; S9 says gallery only, see C6 | Alt+A, C [O O1 2019] |  |
| 8 | AnimationTriggerAddMenu | dropdown |  | Y | ? |  |  |
| 9 | AnimationTriggerAddOnClick | gallery | AnimationTriggerAddMenu | Y | ? |  |  |
| 10 | AnimationTriggerAddOnMediaBookmark | gallery | AnimationTriggerAddMenu | Y | ? |  |  |
| 11 | AnimationPainter | control |  | Y | ? |  | Alt+Shift+C / Alt+Shift+V [D S3] |

**GroupAnimationTiming** (I: "Animation Timing")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | AnimationStartDropdown | dropdown-list |  | Y | ? |  |  |
| 2 | AnimationDuration | control |  | Y | ? |  |  |
| 3 | AnimationDelay | control |  | Y | ? |  |  |
| 4 | AnimationReorderLabel | label |  | Y | ? |  |  |
| 5 | AnimationMoveEarlier | button |  | Y | Yes [D S2] | Alt+A, E [O O1 2019] |  |
| 6 | AnimationMoveLater | button |  | Y | Yes [D S2] | Alt+A, L [O O1 2019] |  |

Web evidence for this tab (S2, web tab `Animations`). Single-line groups: `GroupAnimationEffectOptionsOnline`, `GroupAnimations`, `GroupAnimationTiming`. Multiline groups: `GroupAnimations`, `GroupAnimationTiming`.


### Slide Show

ID `TabSlideShow`. KeyTip: Alt+S [D S3]; web keys S, B and S, C [D S17]. Source S1, checked 2026-11-07.

Groups in source order: `GroupSlideShowStart`, `GroupRehearse`, `GroupSlideShowSetup`, `GroupMonitors`, `GroupLiveSubtitles`.

**GroupSlideShowStart** (I: "Slide Show Start")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | SlideShowFromBeginning | button |  | Y | ? | Alt+S, B [O O1 2019]; web S, B [D S17 Web] | F5 [D S3] |
| 2 | SlideShowFromCurrent | button |  | Y | ? | Alt+S, C [O O1 2019]; web S, C [D S17 Web] | Shift+F5 [O O1 2019] |
| 3 | PresentInTeamsMeeting | button |  | Y | ? |  |  |
| 4 | SlideshowBroadcastMenu | split |  | Y | No [D S8, S9] |  |  |
| 5 | BroadcastSlideShow | button | SlideshowBroadcastMenu | Y | No [D S8, S9] |  |  |
| 6 | BroadcastSlideShowLync | button | SlideshowBroadcastMenu | Y | ? |  |  |
| 7 | BroadcastSlideShowOfficeHosted | button | SlideshowBroadcastMenu | Y | ? |  |  |
| 8 | BroadcastSlideShowGeneric | button | SlideshowBroadcastMenu | Y | ? |  |  |
| 9 | SlideShowCustomMenu | dropdown |  | Y | ? |  |  |

**GroupRehearse** (I: "Rehearse")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | Rehearse | button |  | Y | Yes, web tab View [D S2] |  |  |

**GroupSlideShowSetup** (I: "Slide Show Setup")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | SlideShowSetUpDialog | button |  | Y | No [D S8, S9] |  |  |
| 2 | SlideHide | toggle |  | Y | Yes, web tab Home [D S2] | Alt+S, H [O O1 2019] |  |
| 3 | SlideShowRehearseTimings | button |  | Y | No [D S7: Record presentations] |  |  |
| 4 | RecordSlideShowMenu | split |  | Y | No [D S7: Record presentations] |  |  |
| 5 | RecordNarrationFromCurrentSlide | button | RecordSlideShowMenu | Y | No [D S7: Record presentations] | Alt+S, N, R [O O1 2019] |  |
| 6 | RecordNarration | button | RecordSlideShowMenu | Y | No [D S7: Record presentations] |  |  |
| 7 | SlideShowClearMenu | dropdown | RecordSlideShowMenu | Y | ? |  |  |
| 8 | SlideShowClearCurrentSlideTiming | button | RecordSlideShowMenu > SlideShowClearMenu | Y | ? |  |  |
| 9 | SlideShowClearAllSlidesTimings | button | RecordSlideShowMenu > SlideShowClearMenu | Y | ? |  |  |
| 10 | SlideShowClearCurrentSlideNarration | button | RecordSlideShowMenu > SlideShowClearMenu | Y | ? |  |  |
| 11 | SlideShowClearAllSlidesNarrations | button | RecordSlideShowMenu > SlideShowClearMenu | Y | ? |  |  |
| 12 | SlideShowPlayNarrations | check |  | Y | ? |  |  |
| 13 | SlideShowUseRehearsedTimings | check |  | Y | ? |  |  |
| 14 | MediaControlsShow | check |  | Y | ? |  |  |
| 15 | HideSlide | button |  | Y | ? |  |  |
| 16 | KeepSlidesUpdated | check |  | Y | ? |  |  |

**GroupMonitors** (I: "Monitors")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | SlideShowShowPresentationOnGallery | dropdown-list |  | Y | No [D S8, S9] |  |  |
| 2 | SlideShowUsePresenterView | check |  | Y | Yes, web tab View [D S2] |  | Alt+F5 [D S3] |

**GroupLiveSubtitles** (I: "Live Subtitles")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | UseSubtitles | check |  | Y | ? |  |  |
| 2 | SubtitleSettingsMenu | dropdown |  | Y | ? |  |  |
| 3 | SubtitlesSpokenLanguage | gallery | SubtitleSettingsMenu | Y | Yes, web tab View [D S2] |  |  |
| 4 | SubtitlesDisplayLanguage | gallery | SubtitleSettingsMenu | Y | Yes, web tab View [D S2] |  |  |
| 5 | SubtitlesMicrophoneSettings | gallery | SubtitleSettingsMenu | Y | ? |  |  |
| 6 | SubtitlePositionBottomOverlayed | toggle | SubtitleSettingsMenu | Y | ? |  |  |
| 7 | SubtitlePositionTopOverlayed | toggle | SubtitleSettingsMenu | Y | ? |  |  |
| 8 | SubtitlePositionBelowSlide | toggle | SubtitleSettingsMenu | Y | ? |  |  |
| 9 | SubtitlePositionAboveSlide | toggle | SubtitleSettingsMenu | Y | ? |  |  |
| 10 | SubtitleMoreSettings | button | SubtitleSettingsMenu | Y | ? |  |  |


### Record (source name Recording)

ID `TabRecording`. KeyTip: Alt+E [D S3]. Source S1, checked 2026-11-07. Source holds current groups (`...TabRecord`) and older groups (`GroupRecord`, `GroupSaveRecording`) together; which show live is unconfirmed. Web: Record presentations not supported [D S7], so `?` in these rows reads as No.

Groups in source order: `GroupPreviewTabRecord`, `GroupChunkCameoCamera`, `GroupRecordTabRecord`, `GroupEditTabRecord`, `GroupExportTabRecord`, `GroupRecord`, `GroupContentRecording`, `GroupAutoPlayMediaRecording`, `GroupSaveRecording`, `GroupHelpTabRecord`.

**GroupPreviewTabRecord** (I: "Preview Tab Record")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | PreviewRecordedVideo | button |  | Y | ? |  |  |

**GroupChunkCameoCamera** (I: "Chunk Cameo Camera"). 3 rows identical to `TabInsert`.

**GroupRecordTabRecord** (I: "Record Tab Record")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | RecordNarration | button |  | Y | No [D S7: Record presentations] |  |  |
| 2 | RecordNarrationFromCurrentSlide | button |  | Y | No [D S7: Record presentations] |  |  |
| 3 | ObjectScreenRecordingAutoPlay | button |  | Y | ? |  |  |
| 4 | SoundRecordAutoPlay | button |  | Y | ? |  |  |

**GroupEditTabRecord** (I: "Edit Tab Record")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ClearRecordingMenu | dropdown |  | Y | ? |  |  |
| 2 | ClearRecordingOnCurrentSlide | button | ClearRecordingMenu | Y | ? |  |  |
| 3 | ClearRecordingOnAllSlides | button | ClearRecordingMenu | Y | ? |  |  |
| 4 | ResetToCameoMenu | dropdown |  | Y | ? |  |  |
| 5 | ResetToCameoOnCurrentSlide | button | ResetToCameoMenu | Y | ? |  |  |
| 6 | ResetToCameoOnAllSlides | button | ResetToCameoMenu | Y | ? |  |  |

**GroupExportTabRecord** (I: "Export Tab Record")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | FileSaveAsPowerPointSlideShow | button |  | Y | ? |  |  |
| 2 | ExportVideoRecording | button |  | Y | ? |  |  |

**GroupRecord** (I: "Record")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | RecordSlideShowMenu | split |  | Y | No [D S7: Record presentations] |  |  |
| 2 | RecordNarrationFromCurrentSlide | button | RecordSlideShowMenu | Y | No [D S7: Record presentations] |  |  |
| 3 | RecordNarration | button | RecordSlideShowMenu | Y | No [D S7: Record presentations] |  |  |
| 4 | SlideShowClearMenu | dropdown | RecordSlideShowMenu | Y | ? |  |  |
| 5 | SlideShowClearCurrentSlideTiming | button | RecordSlideShowMenu > SlideShowClearMenu | Y | ? |  |  |
| 6 | SlideShowClearAllSlidesTimings | button | RecordSlideShowMenu > SlideShowClearMenu | Y | ? |  |  |
| 7 | SlideShowClearCurrentSlideNarration | button | RecordSlideShowMenu > SlideShowClearMenu | Y | ? |  |  |
| 8 | SlideShowClearAllSlidesNarrations | button | RecordSlideShowMenu > SlideShowClearMenu | Y | ? |  |  |

**GroupContentRecording** (I: "Content Recording")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | InsertMSForms | button |  | Y | ? |  |  |
| 2 | ScreenshotInsertGallery | gallery |  | Y | ? |  |  |
| 3 | ScreenClipping | button | ScreenshotInsertGallery | Y | ? |  |  |

**GroupAutoPlayMediaRecording** (I: "Auto Play Media Recording")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ObjectScreenRecordingAutoPlay | button |  | Y | ? |  |  |
| 2 | MovieFromFileRecordingAutoPlay | button |  | Y | ? |  |  |
| 3 | SoundInsertMenuAutoPlay | dropdown |  | Y | ? |  |  |
| 4 | SoundInsertFromFileAutoPlay | button | SoundInsertMenuAutoPlay | Y | ? |  |  |
| 5 | SoundRecordAutoPlay | button | SoundInsertMenuAutoPlay | Y | ? |  |  |

**GroupSaveRecording** (I: "Save Recording")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | FileSaveAsPowerPointSlideShow | button |  | Y | ? |  |  |
| 2 | ExportToVideoRecording | button |  | Y | ? |  |  |
| 3 | UploadToMSStreamRecording | button |  | Y | ? |  |  |

**GroupHelpTabRecord** (I: "Help Tab Record")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | LearnAboutRecord | button |  | Y | ? |  |  |


### Review

ID `TabReview`. KeyTip: Alt+R [D S3]. Source S1, checked 2026-11-07. GroupReviewCompare: Compare and Merge disabled in version 2502 [D S29], source predates that.

Groups in source order: `GroupProofing`, `GroupAccessibility`, `GroupInsights`, `GroupLanguage`, `GroupChineseTranslation`, `GroupActivity`, `GroupComments`, `GroupReviewCompare`, `GroupInk`.

**GroupProofing** (I: "Proofing")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | Spelling | button |  | Y | ? |  |  |
| 2 | Thesaurus | button |  | Y | ? |  |  |

**GroupAccessibility** (I: "Accessibility")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | MenuAccessibility | split |  | Y | ? |  |  |
| 2 | AccessibilityChecker | button | MenuAccessibility | Y | Yes [D S2] |  |  |
| 3 | AltTextPaneRibbon | toggle | MenuAccessibility | Y | Yes, web tab FormatPicture [D S2] |  |  |
| 4 | ReadingOrderPane | toggle | MenuAccessibility | Y | ? |  |  |
| 5 | OpenOptionsEaseOfAcess | button | MenuAccessibility | Y | ? |  |  |
| 6 | AccessibilityHelp | button | MenuAccessibility | Y | ? |  |  |

**GroupInsights** (I: "Insights")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | Insights | button |  | Y | Yes [D S2] | Alt+R, R, S (legacy Smart Lookup sequence) [O O1 2019] |  |

**GroupLanguage** (I: "Language")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | TranslateMenu | dropdown |  | Y | ? |  |  |
| 2 | TranslateSelected | button | TranslateMenu | Y | ? |  |  |
| 3 | MiniTranslator | toggle | TranslateMenu | Y | ? |  |  |
| 4 | TranslationLanguageOptions | button | TranslateMenu | Y | ? |  |  |
| 5 | Translator | button |  | Y | ? |  |  |
| 6 | HangulHanjaConversionPowerPoint | button |  | Y | ? |  |  |
| 7 | LanguageCommands | dropdown |  | Y | ? |  |  |
| 8 | SetLanguage | button | LanguageCommands | Y | ? |  |  |
| 9 | LanguagePreferences | button | LanguageCommands | Y | ? |  |  |

**GroupChineseTranslation** (I: "Chinese Translation")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | TranslateToSimplifiedChinese | button |  | Y | ? |  |  |
| 2 | TranslateToTraditionalChinese | button |  | Y | ? |  |  |
| 3 | ChineseTranslationDialog | button |  | Y | ? |  |  |

**GroupActivity** (I: "Activity")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | MarkAllRevTrackHighlightsAsRead | button |  | Y | ? |  |  |
| 2 | ToggleRevTrackCanvasHighlights | toggle |  | Y | ? |  |  |

**GroupComments** (I: "Comments")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ReviewNewComment | button |  | Y | Yes [D S2] | Alt+R, C [D S23]; web R, C, 1 [D S23 Web] | Ctrl+Alt+M [D S3]; web Ctrl+Alt+M [D S3 Web] |
| 2 | ReviewDeleteCommentsMenuPowerPoint | split |  | Y | ? |  |  |
| 3 | ReviewDeleteCommentPowerPoint | button | ReviewDeleteCommentsMenuPowerPoint | Y | Yes [D S2] |  |  |
| 4 | ReviewDeleteAllMarkupOnSlide | button | ReviewDeleteCommentsMenuPowerPoint | Y | ? |  |  |
| 5 | ReviewDeleteAllMarkupInPresentation | button | ReviewDeleteCommentsMenuPowerPoint | Y | ? |  |  |
| 6 | ReviewPreviousCommentPowerPoint | button |  | Y | Yes [D S2] |  |  |
| 7 | ReviewNextCommentPowerPoint | button |  | Y | Yes [D S2] |  |  |
| 8 | ShowCommentsMenu | split |  | Y | ? |  |  |
| 9 | CommentsPaneSplitButton | toggle | ShowCommentsMenu | Y | ? |  |  |
| 10 | CommentsPane | toggle | ShowCommentsMenu | Y | Yes [D S2] | Alt+R, P, P [D S23]; web R, P, P [D S23 Web] |  |
| 11 | ReviewShowOrHideMarkup | toggle | ShowCommentsMenu | Y | ? |  |  |
| 12 | ShowAllCommentMarkers | toggle | ShowCommentsMenu | Y | ? |  |  |

**GroupReviewCompare** (I: "Review Compare")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ReviewCompareAndMerge | button |  | Y | No [D S9 2023] |  |  |
| 2 | ReviewRevisionAcceptMenu | split |  | Y | ? |  |  |
| 3 | ReviewRevisionAccept | button | ReviewRevisionAcceptMenu | Y | ? |  |  |
| 4 | ReviewSlideRevisionAccept | button | ReviewRevisionAcceptMenu | Y | ? |  |  |
| 5 | ReviewPresentationRevisionAccept | button | ReviewRevisionAcceptMenu | Y | ? |  |  |
| 6 | ReviewRevisionRejectMenu | split |  | Y | ? |  |  |
| 7 | ReviewRevisionReject | button | ReviewRevisionRejectMenu | Y | ? |  |  |
| 8 | ReviewSlideRevisionReject | button | ReviewRevisionRejectMenu | Y | ? |  |  |
| 9 | ReviewPresentationRevisionReject | button | ReviewRevisionRejectMenu | Y | ? |  |  |
| 10 | ReviewPreviousChange | button |  | Y | ? |  |  |
| 11 | ReviewNextChange | button |  | Y | ? |  |  |
| 12 | ReviewReviewingPanePowerPoint | toggle |  | Y | ? |  |  |
| 13 | ReviewEndReviewPowerPoint | button |  | Y | ? |  |  |

**GroupInk** (I: "Ink")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | InkingStart | button |  | Y | ? |  |  |
| 2 | HideInkSplitButton | split |  | Y | ? |  |  |
| 3 | HideInk | toggle | HideInkSplitButton | Y | ? |  |  |
| 4 | InkDeleteAllOnSlide | button | HideInkSplitButton | Y | ? |  |  |
| 5 | InkDeleteAll | button | HideInkSplitButton | Y | ? |  |  |

Web evidence for this tab (S2, web tab `Review`). Single-line groups: `GroupAccessibility`, `GroupComments`, `GroupInsights`, `GroupLanguage`, `GroupProofing`, `GroupReviewCompare`. Multiline groups: `GroupAccessibility`, `GroupComments`, `GroupInsights`, `GroupProofing`, `GroupReviewCompare`.

Web control IDs with no match on this desktop tab: `EditorPaneOnline`, `GrammarOptionsOnline`, `HideProofingErrorsOnline`, `MarkActivitiesAsReadOnline`, `ReviewShowOrHideMark`, `ShowHideChangesOnline`.


### View

ID `TabView`. KeyTip: Alt+W [D S3]. Source S1, checked 2026-11-07.

Groups in source order: `GroupPresentationViews`, `GroupMasterViews`, `GroupViewShowHide`, `GroupViewDirection`, `GroupZoom`, `GroupColorGrayscale`, `GroupWindow`, `GroupMacros`.

**GroupPresentationViews** (I: "Presentation Views")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ViewThumbnailViewPowerPoint | toggle |  | Y | Yes [D S2] | Alt+W, L (Normal; ID mapping I) [D S17] |  |
| 2 | ViewOutlineViewPowerPoint | toggle |  | Y | No [D S8]; verify live | Alt+W, P, O [D S17] |  |
| 3 | ViewSlideSorterView | toggle |  | Y | ? | Alt+W, I [D S17] |  |
| 4 | ViewNotesPageView | toggle |  | Y | Yes [D S2] | Alt+W, T [O O1 2019] |  |
| 5 | ViewSlideShowReadingView | button |  | Y | ? | Alt+W, D [D S17] |  |

**GroupMasterViews** (I: "Master Views")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ViewSlideMasterView | toggle |  | Y | No [D S7, S8] | Alt+W, M [D S17] |  |
| 2 | ViewHandoutMasterView | toggle |  | Y | No [D S7, S8] | Alt+W, H [O O1 2019] |  |
| 3 | ViewNotesMasterView | toggle |  | Y | No [D S7, S8] | Alt+W, K [O O1 2019] |  |

**GroupViewShowHide** (I: "View Show Hide")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ViewRulerPowerPoint | check |  | Y | No [D S8]; smart guides only |  |  |
| 2 | ViewGridlinesPowerPoint | check |  | Y | No [D S8]; smart guides only |  | Shift+F9 [D S3] |
| 3 | GuidesShowHide | check |  | Y | No [D S8]; smart guides only |  | Alt+F9 [D S3] |
| 4 | ShowNotes | toggle |  | Y | ? | Alt+W, P, N [D S3]; web W, P, N [D S23 Web] |  |
| 5 | GridSettings | launcher |  | Y | No [D S8]; smart guides only | Alt+W, X [O O1 2019] |  |

**GroupViewDirection** (I: "View Direction")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ViewDirectionMenu | dropdown |  | Y | ? |  |  |
| 2 | ViewDirectionLeftToRight | toggle | ViewDirectionMenu | Y | ? |  |  |
| 3 | ViewDirectionRightToLeft | toggle | ViewDirectionMenu | Y | ? |  |  |

**GroupZoom** (I: "Zoom")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ZoomDialog | button |  | Y | Yes [D S2] | Alt+W, Q [D S3] |  |
| 2 | ZoomFitToWindow | button |  | Y | Yes [D S2] | Alt+W, F [O O1 2019] | Ctrl+Alt+O [D S3] |

**GroupColorGrayscale** (I: "Color Grayscale")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ViewDisplayInHighContrast | toggle |  | Y | ? |  |  |
| 2 | ViewDisplayInColor | toggle |  | Y | ? |  |  |
| 3 | ViewDisplayInGrayscale | toggle |  | Y | ? |  |  |
| 4 | ViewDisplayInPureBlackAndWhite | toggle |  | Y | ? |  |  |

**GroupWindow** (I: "Window")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | WindowNew | button |  | Y | ? | Alt+W, N [O O1 2019] |  |
| 2 | WindowsArrangeAll | button |  | Y | ? |  |  |
| 3 | WindowsCascade | button |  | Y | ? | Alt+W, E [O O1 2019] |  |
| 4 | WindowMoveSplit | button |  | Y | ? |  |  |
| 5 | WindowSwitchWindowsMenuPowerPoint | dropdown |  | Y | ? |  |  |
| 6 | WindowMoreWindowsDialog | toggle | WindowSwitchWindowsMenuPowerPoint | Y | ? |  |  |

**GroupMacros** (I: "Macros")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | MacroPlay | button |  | Y | ? |  |  |

Web evidence for this tab (S2, web tab `View`). Single-line groups: `GroupPresentationViews`, `GroupViewShowHide`, `GroupZoom`. Multiline groups: `GroupViewShowHide`, `GroupZoom`, `GroupStartSlideShowOnline`.

Web control IDs with no match on this desktop tab: `BroadcastSlideShow`, `PlayFromCurrentSlideOnline`, `Rehearse`, `SlideShowUsePresenterView`, `SubtitlesDisplayLanguage`, `SubtitlesPositionAboveSlide`, `SubtitlesPositionBelowSlide`, `SubtitlesPositionBottomOverlayed`, `SubtitlesPositionTopOverlayed`, `SubtitlesSpokenLanguage`, `ViewSlideShowView`, `ViewSlideSortedView`.


### Help

ID `HelpTab`. KeyTip: Alt+Y, 2 [D S3]. Source S1, checked 2026-11-07.

Groups in source order: `GroupHelpAndSupport`.

**GroupHelpAndSupport** (I: "Help And Support")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | Help | button |  | Y | ? |  |  |
| 2 | ContactUs | button |  | Y | ? |  |  |
| 3 | OfficeFeedbackHelpTab | button |  | Y | ? |  |  |
| 4 | OfficeFeedbackIcon | button |  | Y | ? |  |  |
| 5 | ShowTraining | button |  | Y | ? |  |  |
| 6 | WhatsNewRecentUpdates | button |  | Y | ? |  |  |


## Contextual tabs requested


### Shape Format (tab set `TabSetDrawingTools`)

ID `TabDrawingToolsFormat`. KeyTip: Alt+J, D [I from Office 2007 path in S3 and Alt+J, D, T, X in S18]. Source S1, checked 2026-11-07.

Groups in source order: `GroupShapes`, `GroupShapeStyles`, `GroupWordArtStyles`, `GroupAltText`, `GroupArrangeWith3DEditor`, `GroupSize`.

**GroupShapes** (I: "Shapes")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ShapesInsertGallery | gallery |  | Y | Yes [D S2] |  |  |
| 2 | ObjectEditShapeMenu | dropdown |  | Y | ? |  |  |
| 3 | ShapeChangeShapeGallery | gallery | ObjectEditShapeMenu | Y | ? |  |  |
| 4 | ObjectEditPoints | toggle | ObjectEditShapeMenu | Y | ? |  |  |
| 5 | ShapeRerouteConnectors | toggle | ObjectEditShapeMenu | Y | ? |  |  |
| 6 | TextBoxInsert | toggle |  | Y | Yes, web tab Insert [D S2] |  |  |
| 7 | TextBoxInsertMenu | split |  | Y | Yes [D S2] |  |  |
| 8 | TextBoxInsertHorizontal | toggle | TextBoxInsertMenu | Y | ? |  |  |
| 9 | TextBoxInsertVertical | toggle | TextBoxInsertMenu | Y | ? |  |  |
| 10 | TextboxFlyoutMenuAnchor | dropdown |  | Y | ? |  |  |
| 11 | TextBoxInsertHorizontal | toggle | TextboxFlyoutMenuAnchor | Y | ? |  |  |
| 12 | TextBoxInsertVertical | toggle | TextboxFlyoutMenuAnchor | Y | ? |  |  |
| 13 | CombineShapesGallery | gallery |  | Y | ? |  |  |

**GroupShapeStyles** (I: "Shape Styles")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ShapeStylesGallery | gallery |  | Y | Yes [D S2] |  |  |
| 2 | ShapeStylesOtherThemeFillsGallery | gallery | ShapeStylesGallery | Y | ? |  |  |
| 3 | ShapeFillColorPicker | gallery |  | Y | Yes [D S2] |  |  |
| 4 | ObjectFillMoreColorsDialog | button | ShapeFillColorPicker | Y | ? |  |  |
| 5 | EyedropperFill | button | ShapeFillColorPicker | Y | ? |  |  |
| 6 | ObjectPictureFill | button | ShapeFillColorPicker | Y | ? |  |  |
| 7 | GradientGallery | gallery | ShapeFillColorPicker | Y | ? |  |  |
| 8 | ShapeFillMoreGradientsDialog | button | ShapeFillColorPicker > GradientGallery | Y | ? |  |  |
| 9 | ShapeFillTextureGallery | gallery | ShapeFillColorPicker | Y | ? |  |  |
| 10 | MoreTextureOptions | button | ShapeFillColorPicker > ShapeFillTextureGallery | Y | ? |  |  |
| 11 | ShapeOutlineColorPicker | gallery |  | Y | Yes [D S2] |  |  |
| 12 | ObjectBorderOutlineColorMoreColorsDialog | button | ShapeOutlineColorPicker | Y | ? |  |  |
| 13 | EyedropperOutline | button | ShapeOutlineColorPicker | Y | ? |  |  |
| 14 | OutlineWeightGallery | gallery | ShapeOutlineColorPicker | Y | ? |  |  |
| 15 | LineStylesDialog | button | ShapeOutlineColorPicker > OutlineWeightGallery | Y | ? |  |  |
| 16 | GallerySketchyShapes | gallery | ShapeOutlineColorPicker | Y | ? |  |  |
| 17 | MoreLines2 | button | ShapeOutlineColorPicker > GallerySketchyShapes | Y | ? |  |  |
| 18 | OutlineDashesGallery | gallery | ShapeOutlineColorPicker | Y | ? |  |  |
| 19 | LineStylesDialog | button | ShapeOutlineColorPicker > OutlineDashesGallery | Y | ? |  |  |
| 20 | ArrowStyleGallery | gallery | ShapeOutlineColorPicker | Y | ? |  |  |
| 21 | ArrowsMore | button | ShapeOutlineColorPicker > ArrowStyleGallery | Y | ? |  |  |
| 22 | ShapeEffectsMenu | dropdown |  | Y | ? |  |  |
| 23 | ObjectEffectPresetGallery | gallery | ShapeEffectsMenu | Y | ? |  |  |
| 24 | _3DBevelOptionsDialog | button | ShapeEffectsMenu > ObjectEffectPresetGallery | Y | ? |  |  |
| 25 | ObjectEffectShadowGallery | gallery | ShapeEffectsMenu | Y | ? |  |  |
| 26 | ShadowOptionsDialog | button | ShapeEffectsMenu > ObjectEffectShadowGallery | Y | ? |  |  |
| 27 | ReflectionGallery | gallery | ShapeEffectsMenu | Y | ? |  |  |
| 28 | ReflectionsMoreOptions | button | ShapeEffectsMenu > ReflectionGallery | Y | ? |  |  |
| 29 | ObjectEffectGlowGallery | gallery | ShapeEffectsMenu | Y | ? |  |  |
| 30 | GlowColorPicker | gallery | ShapeEffectsMenu > ObjectEffectGlowGallery | Y | ? |  |  |
| 31 | GlowColorMoreColorsDialog | button | ShapeEffectsMenu > ObjectEffectGlowGallery | Y | ? |  |  |
| 32 | EyedropperGlow | button | ShapeEffectsMenu > ObjectEffectGlowGallery | Y | ? |  |  |
| 33 | GlowsMoreOptions | button | ShapeEffectsMenu > ObjectEffectGlowGallery | Y | ? |  |  |
| 34 | ObjectEffectSoftEdgesGallery | gallery | ShapeEffectsMenu | Y | ? |  |  |
| 35 | SoftEdgesMoreOptions | button | ShapeEffectsMenu > ObjectEffectSoftEdgesGallery | Y | ? |  |  |
| 36 | BevelShapeGallery | gallery | ShapeEffectsMenu | Y | ? |  |  |
| 37 | _3DBevelOptionsDialog | button | ShapeEffectsMenu > BevelShapeGallery | Y | ? |  |  |
| 38 | _3DRotationGallery | gallery | ShapeEffectsMenu | Y | ? |  |  |
| 39 | _3DRotationOptionsDialog | button | ShapeEffectsMenu > _3DRotationGallery | Y | ? |  |  |
| 40 | ObjectFormatDialog | launcher |  | Y | ? |  |  |

**GroupWordArtStyles** (I: "Word Art Styles")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | TextStylesGallery | gallery |  | Y | ? |  |  |
| 2 | WordArtClear | button | TextStylesGallery | Y | ? |  |  |
| 3 | TextFillColorPicker | gallery |  | Y | ? |  |  |
| 4 | TextFillColorMoreColorsDialog | button | TextFillColorPicker | Y | ? |  |  |
| 5 | EyedropperFillText | button | TextFillColorPicker | Y | ? |  |  |
| 6 | TextPictureFill | button | TextFillColorPicker | Y | ? |  |  |
| 7 | TextFillGradientGallery | gallery | TextFillColorPicker | Y | ? |  |  |
| 8 | TextFillMoreGradientsDialog | button | TextFillColorPicker > TextFillGradientGallery | Y | ? |  |  |
| 9 | TextFillTextureGallery | gallery | TextFillColorPicker | Y | ? |  |  |
| 10 | TextFillMoreTextures | button | TextFillColorPicker > TextFillTextureGallery | Y | ? |  |  |
| 11 | TextOutlineColorPicker | gallery |  | Y | ? |  |  |
| 12 | TextOutlineColorMoreColorsDialog | button | TextOutlineColorPicker | Y | ? |  |  |
| 13 | EyedropperOutlineText | button | TextOutlineColorPicker | Y | ? |  |  |
| 14 | TextOutlineWeightGallery | gallery | TextOutlineColorPicker | Y | ? |  |  |
| 15 | TextOutlineMoreLinesDialog | button | TextOutlineColorPicker > TextOutlineWeightGallery | Y | ? |  |  |
| 16 | TextOutlineDashesGallery | gallery | TextOutlineColorPicker | Y | ? |  |  |
| 17 | TextOutlineMoreLinesDialog | button | TextOutlineColorPicker > TextOutlineDashesGallery | Y | ? |  |  |
| 18 | TextEffectsMenu | dropdown |  | Y | ? | Alt+J, D, T, X [D S18] |  |
| 19 | TextEffectShadowGallery | gallery | TextEffectsMenu | Y | ? |  |  |
| 20 | TextEffectsMoreShadowsDialog | button | TextEffectsMenu > TextEffectShadowGallery | Y | ? |  |  |
| 21 | TextReflectionGallery | gallery | TextEffectsMenu | Y | ? |  |  |
| 22 | TextEffectsReflectionsMoreOptions | button | TextEffectsMenu > TextReflectionGallery | Y | ? |  |  |
| 23 | TextEffectGlowGallery | gallery | TextEffectsMenu | Y | ? |  |  |
| 24 | TextGlowColorPicker | gallery | TextEffectsMenu > TextEffectGlowGallery | Y | ? |  |  |
| 25 | TextGlowColorMoreColorsDialog | button | TextEffectsMenu > TextEffectGlowGallery | Y | ? |  |  |
| 26 | EyedropperGlowText | button | TextEffectsMenu > TextEffectGlowGallery | Y | ? |  |  |
| 27 | TextEffectsGlowsMoreOptions | button | TextEffectsMenu > TextEffectGlowGallery | Y | ? |  |  |
| 28 | BevelTextGallery | gallery | TextEffectsMenu | Y | ? |  |  |
| 29 | TextEffectsBevelMore3DOptionsDialog | button | TextEffectsMenu > BevelTextGallery | Y | ? |  |  |
| 30 | TextEffects3DRotationGallery | gallery | TextEffectsMenu | Y | ? |  |  |
| 31 | TextEffects3DRotationOptionsDialog | button | TextEffectsMenu > TextEffects3DRotationGallery | Y | ? |  |  |
| 32 | TextEffectTransformGallery | gallery | TextEffectsMenu | Y | ? |  |  |
| 33 | WordArtFormatDialog | launcher |  | Y | ? |  |  |

**GroupAltText** (I: "Alt Text")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | AltTextPaneRibbon | toggle |  | Y | Yes, web tab FormatPicture [D S2] |  |  |

**GroupArrangeWith3DEditor** (I: "Arrange With3 DEditor")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ObjectBringForwardMenu | split |  | Y | ? |  |  |
| 2 | ObjectBringForward | button | ObjectBringForwardMenu | Y | Yes [D S2] |  | Ctrl+Shift+] [D S3]; web Ctrl+] [D S3 Web] |
| 3 | ObjectBringToFront | button | ObjectBringForwardMenu | Y | Yes [D S2] |  | web Ctrl+Shift+] [D S3 Web] |
| 4 | ObjectSendBackwardMenu | split |  | Y | ? |  |  |
| 5 | ObjectSendBackward | button | ObjectSendBackwardMenu | Y | Yes [D S2] |  | Ctrl+Shift+[ [D S3]; web Ctrl+[ [D S3 Web] |
| 6 | ObjectSendToBack | button | ObjectSendBackwardMenu | Y | Yes [D S2] |  | web Ctrl+Shift+[ [D S3 Web] |
| 7 | SelectionPane | toggle |  | Y | Yes, web tab Home [D S2] |  | Alt+F10 [D S3] |
| 8 | ObjectAlignMenu | dropdown |  | Y | Yes [D S2] |  |  |
| 9 | ObjectsAlignLeftSmart | button | ObjectAlignMenu | Y | Yes [D S2] |  |  |
| 10 | ObjectsAlignCenterHorizontalSmart | button | ObjectAlignMenu | Y | Yes [D S2] |  |  |
| 11 | ObjectsAlignRightSmart | button | ObjectAlignMenu | Y | Yes [D S2] |  |  |
| 12 | ObjectsAlignTopSmart | button | ObjectAlignMenu | Y | Yes [D S2] |  |  |
| 13 | ObjectsAlignMiddleVerticalSmart | button | ObjectAlignMenu | Y | Yes [D S2] |  |  |
| 14 | ObjectsAlignBottomSmart | button | ObjectAlignMenu | Y | Yes [D S2] |  |  |
| 15 | AlignDistributeHorizontally | button | ObjectAlignMenu | Y | Yes, web tab Home [D S2] |  |  |
| 16 | AlignDistributeVertically | button | ObjectAlignMenu | Y | Yes, web tab Home [D S2] |  |  |
| 17 | ObjectsAlignRelativeToContainerSmart | toggle | ObjectAlignMenu | Y | Yes [D S2] |  |  |
| 18 | ObjectsAlignSelectedSmart | toggle | ObjectAlignMenu | Y | ? |  |  |
| 19 | ObjectsGroupMenu | dropdown |  | Y | Yes [D S2] |  |  |
| 20 | ObjectsGroup | button | ObjectsGroupMenu | Y | Yes [D S2] |  | Ctrl+G [D S3] |
| 21 | ObjectsRegroup | button | ObjectsGroupMenu | Y | ? |  | Ctrl+Shift+J [D S3] |
| 22 | ObjectsUngroup | button | ObjectsGroupMenu | Y | Yes [D S2] |  | Ctrl+Shift+G [D S3] |
| 23 | ObjectRotateGallery | gallery |  | Y | Yes [D S2] |  |  |
| 24 | ObjectRotationOptionsDialog | button | ObjectRotateGallery | Y | Yes [D S2] |  |  |

**GroupSize** (I: "Size")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ShapeHeight | control |  | Y | ? |  |  |
| 2 | ShapeWidth | control |  | Y | ? |  |  |
| 3 | ObjectSizeAndPositionDialog | launcher |  | Y | ? |  |  |

Web evidence for this tab (S2, web tab `FormatShape`). Single-line groups: `GroupArrange`, `GroupShapesChart`, `GroupShapeStyles`, `GroupSize`. Multiline groups: `GroupShapesChart`, `GroupShapeStyles`.

Web control IDs with no match on this desktop tab: `ObjectFlipHorizontal`, `ObjectFlipVertical`, `ObjectRotateLeft90`, `ObjectRotateRight90`.


### Picture Format (tab set `TabSetPictureTools`)

ID `TabPictureToolsFormat`. KeyTip: Alt+J, P [D S19]. Source S1, checked 2026-11-07.

Groups in source order: `GroupImagePlay`, `GroupPictureTools`, `GroupPictureStyles`, `GroupAltText`, `GroupArrangeWith3DEditor`, `GroupPictureSize`.

**GroupImagePlay** (I: "Image Play")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ImagePlay | button |  | Y | ? |  |  |

**GroupPictureTools** (I: "Picture Tools")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | PictureBackgroundRemoval | toggle |  | Y | No [D S8] |  |  |
| 2 | AutoBackgroundRemoval | toggle |  | Y | No [D S8] |  |  |
| 3 | PictureCorrectionsMenu | dropdown |  | Y | ? |  |  |
| 4 | PictureSharpenSoftenGallery | gallery | PictureCorrectionsMenu | Y | ? |  |  |
| 5 | PictureBrightnessAndContrastGallery | gallery | PictureCorrectionsMenu | Y | ? |  |  |
| 6 | PictureCorrectionsDialog | button | PictureCorrectionsMenu | Y | ? |  |  |
| 7 | PictureColorMenu | dropdown |  | Y | ? |  |  |
| 8 | PictureSaturationGallery | gallery | PictureColorMenu | Y | ? |  |  |
| 9 | PictureColorTempertatureGallery | gallery | PictureColorMenu | Y | ? |  |  |
| 10 | PictureRecolorGallery | gallery | PictureColorMenu | Y | ? |  |  |
| 11 | RecolorColorPicker | gallery | PictureColorMenu | Y | ? |  |  |
| 12 | PictureRecolorMoreColorsDialog | button | PictureColorMenu > RecolorColorPicker | Y | ? |  |  |
| 13 | EyedropperRecolor | button | PictureColorMenu > RecolorColorPicker | Y | ? |  |  |
| 14 | PictureSetTransparentColor | toggle | PictureColorMenu | Y | ? |  |  |
| 15 | PictureColorDialog | button | PictureColorMenu | Y | ? |  |  |
| 16 | PictureArtisticEffectsGallery | gallery |  | Y | ? |  |  |
| 17 | ArtisticEffectsDialog | button | PictureArtisticEffectsGallery | Y | ? |  |  |
| 18 | PictureTransparencyGallery | gallery |  | Y | ? |  |  |
| 19 | PictureMoreTransparency | button | PictureTransparencyGallery | Y | ? |  |  |
| 20 | PicturesCompress | button |  | Y | No [D S7] |  |  |
| 21 | PictureChange | button |  | Y | Yes [D S2] |  |  |
| 22 | PictureChangeMenu | dropdown |  | Y | Yes [D S2] |  |  |
| 23 | PictureChangeFromFile | button | PictureChangeMenu | Y | ? |  |  |
| 24 | PictureChangeFromMobile | button | PictureChangeMenu | Y | ? |  |  |
| 25 | PictureChangeFromM365 | button | PictureChangeMenu | Y | ? |  |  |
| 26 | PictureChangeFromOnline | button | PictureChangeMenu | Y | ? |  |  |
| 27 | ChangeIcon | button | PictureChangeMenu | Y | ? |  |  |
| 28 | PictureChangeFromClipboard | button | PictureChangeMenu | Y | ? |  |  |
| 29 | PictureResetSplitButton | split |  | Y | ? |  |  |
| 30 | PictureReset | button | PictureResetSplitButton | Y | Yes [D S2] |  |  |
| 31 | PictureResetAndSize | button | PictureResetSplitButton | Y | ? |  |  |

**GroupPictureStyles** (I: "Picture Styles")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | PictureStylesGallery | gallery |  | Y | Yes [D S2] |  |  |
| 2 | OutlineColorPicker | gallery |  | Y | Yes, web tab Home [D S2] |  |  |
| 3 | ObjectBorderOutlineColorMoreColorsDialog | button | OutlineColorPicker | Y | ? |  |  |
| 4 | EyedropperOutline | button | OutlineColorPicker | Y | ? |  |  |
| 5 | OutlineWeightGallery | gallery | OutlineColorPicker | Y | ? |  |  |
| 6 | LineStylesDialog | button | OutlineColorPicker > OutlineWeightGallery | Y | ? |  |  |
| 7 | GallerySketchyShapes | gallery | OutlineColorPicker | Y | ? |  |  |
| 8 | MoreLines2 | button | OutlineColorPicker > GallerySketchyShapes | Y | ? |  |  |
| 9 | OutlineDashesGallery | gallery | OutlineColorPicker | Y | ? |  |  |
| 10 | LineStylesDialog | button | OutlineColorPicker > OutlineDashesGallery | Y | ? |  |  |
| 11 | PictureEffectsMenu | dropdown |  | Y | ? |  |  |
| 12 | PictureEffectsPresetGallery | gallery | PictureEffectsMenu | Y | ? |  |  |
| 13 | _3DBevelOptionsDialog | button | PictureEffectsMenu > PictureEffectsPresetGallery | Y | ? |  |  |
| 14 | PictureEffectsShadowGallery | gallery | PictureEffectsMenu | Y | ? |  |  |
| 15 | ShadowOptionsDialog | button | PictureEffectsMenu > PictureEffectsShadowGallery | Y | ? |  |  |
| 16 | PictureReflectionGallery | gallery | PictureEffectsMenu | Y | ? |  |  |
| 17 | ReflectionsMoreOptions | button | PictureEffectsMenu > PictureReflectionGallery | Y | ? |  |  |
| 18 | PictureEffectsGlowGallery | gallery | PictureEffectsMenu | Y | ? |  |  |
| 19 | GlowColorPicker | gallery | PictureEffectsMenu > PictureEffectsGlowGallery | Y | ? |  |  |
| 20 | GlowColorMoreColorsDialog | button | PictureEffectsMenu > PictureEffectsGlowGallery | Y | ? |  |  |
| 21 | EyedropperGlow | button | PictureEffectsMenu > PictureEffectsGlowGallery | Y | ? |  |  |
| 22 | GlowsMoreOptions | button | PictureEffectsMenu > PictureEffectsGlowGallery | Y | ? |  |  |
| 23 | PictureEffectsSoftEdgesGallery | gallery | PictureEffectsMenu | Y | ? |  |  |
| 24 | SoftEdgesMoreOptions | button | PictureEffectsMenu > PictureEffectsSoftEdgesGallery | Y | ? |  |  |
| 25 | _3DBevelPictureTopGallery | gallery | PictureEffectsMenu | Y | ? |  |  |
| 26 | _3DBevelOptionsDialog | button | PictureEffectsMenu > _3DBevelPictureTopGallery | Y | ? |  |  |
| 27 | PictureRotationGallery | gallery | PictureEffectsMenu | Y | ? |  |  |
| 28 | _3DRotationOptionsDialog | button | PictureEffectsMenu > PictureRotationGallery | Y | ? |  |  |
| 29 | PicturesConvertToSmartArt | gallery |  | Y | ? |  |  |
| 30 | PictureFormatDialog | launcher |  | Y | ? |  |  |

**GroupAltText** (I: "Alt Text")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | AltTextPaneRibbon | toggle |  | Y | Yes [D S2] |  |  |

**GroupArrangeWith3DEditor** (I: "Arrange With3 DEditor")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ObjectBringForwardMenu | split |  | Y | ? |  |  |
| 2 | ObjectBringForward | button | ObjectBringForwardMenu | Y | Yes, web tab FormatShape [D S2] |  | Ctrl+Shift+] [D S3]; web Ctrl+] [D S3 Web] |
| 3 | ObjectBringToFront | button | ObjectBringForwardMenu | Y | Yes, web tab FormatShape [D S2] |  | web Ctrl+Shift+] [D S3 Web] |
| 4 | ObjectSendBackwardMenu | split |  | Y | ? |  |  |
| 5 | ObjectSendBackward | button | ObjectSendBackwardMenu | Y | Yes, web tab FormatShape [D S2] |  | Ctrl+Shift+[ [D S3]; web Ctrl+[ [D S3 Web] |
| 6 | ObjectSendToBack | button | ObjectSendBackwardMenu | Y | Yes, web tab FormatShape [D S2] |  | web Ctrl+Shift+[ [D S3 Web] |
| 7 | SelectionPane | toggle |  | Y | Yes, web tab Home [D S2] |  | Alt+F10 [D S3] |
| 8 | ObjectAlignMenu | dropdown |  | Y | Yes, web tab FormatShape [D S2] |  |  |
| 9 | ObjectsAlignLeftSmart | button | ObjectAlignMenu | Y | Yes, web tab FormatShape [D S2] |  |  |
| 10 | ObjectsAlignCenterHorizontalSmart | button | ObjectAlignMenu | Y | Yes, web tab FormatShape [D S2] |  |  |
| 11 | ObjectsAlignRightSmart | button | ObjectAlignMenu | Y | Yes, web tab FormatShape [D S2] |  |  |
| 12 | ObjectsAlignTopSmart | button | ObjectAlignMenu | Y | Yes, web tab FormatShape [D S2] |  |  |
| 13 | ObjectsAlignMiddleVerticalSmart | button | ObjectAlignMenu | Y | Yes, web tab FormatShape [D S2] |  |  |
| 14 | ObjectsAlignBottomSmart | button | ObjectAlignMenu | Y | Yes, web tab FormatShape [D S2] |  |  |
| 15 | AlignDistributeHorizontally | button | ObjectAlignMenu | Y | Yes, web tab Home [D S2] |  |  |
| 16 | AlignDistributeVertically | button | ObjectAlignMenu | Y | Yes, web tab Home [D S2] |  |  |
| 17 | ObjectsAlignRelativeToContainerSmart | toggle | ObjectAlignMenu | Y | Yes, web tab FormatShape [D S2] |  |  |
| 18 | ObjectsAlignSelectedSmart | toggle | ObjectAlignMenu | Y | ? |  |  |
| 19 | ObjectsGroupMenu | dropdown |  | Y | Yes, web tab FormatShape [D S2] |  |  |
| 20 | ObjectsGroup | button | ObjectsGroupMenu | Y | Yes, web tab FormatShape [D S2] |  | Ctrl+G [D S3] |
| 21 | ObjectsRegroup | button | ObjectsGroupMenu | Y | ? |  | Ctrl+Shift+J [D S3] |
| 22 | ObjectsUngroup | button | ObjectsGroupMenu | Y | Yes, web tab FormatShape [D S2] |  | Ctrl+Shift+G [D S3] |
| 23 | ObjectRotateGallery | gallery |  | Y | Yes, web tab FormatShape [D S2] |  |  |
| 24 | ObjectRotationOptionsDialog | button | ObjectRotateGallery | Y | Yes, web tab FormatShape [D S2] |  |  |

**GroupPictureSize** (I: "Picture Size")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | PictureCropTools | split |  | Y | Yes [D S2] | Alt+J, P, V [D S19] |  |
| 2 | PictureCrop | toggle | PictureCropTools | Y | ? | Alt+J, P, V (menu; S then shape, A then ratio) [D S19] |  |
| 3 | PictureShapeGallery | gallery | PictureCropTools | Y | ? |  |  |
| 4 | PictureCropAspectRatioMenu | dropdown | PictureCropTools | Y | ? |  |  |
| 5 | PictureCropAspectRatio1To1 | button | PictureCropTools > PictureCropAspectRatioMenu | Y | ? |  |  |
| 6 | PictureCropAspectRatio2To3 | button | PictureCropTools > PictureCropAspectRatioMenu | Y | ? |  |  |
| 7 | PictureCropAspectRatio3To4 | button | PictureCropTools > PictureCropAspectRatioMenu | Y | ? |  |  |
| 8 | PictureCropAspectRatio3To5 | button | PictureCropTools > PictureCropAspectRatioMenu | Y | ? |  |  |
| 9 | PictureCropAspectRatio4To5 | button | PictureCropTools > PictureCropAspectRatioMenu | Y | ? |  |  |
| 10 | PictureCropAspectRatio3To2 | button | PictureCropTools > PictureCropAspectRatioMenu | Y | ? |  |  |
| 11 | PictureCropAspectRatio4To3 | button | PictureCropTools > PictureCropAspectRatioMenu | Y | ? |  |  |
| 12 | PictureCropAspectRatio5To3 | button | PictureCropTools > PictureCropAspectRatioMenu | Y | ? |  |  |
| 13 | PictureCropAspectRatio5To4 | button | PictureCropTools > PictureCropAspectRatioMenu | Y | ? |  |  |
| 14 | PictureCropAspectRatio16To9 | button | PictureCropTools > PictureCropAspectRatioMenu | Y | ? |  |  |
| 15 | PictureCropAspectRatio16To10 | button | PictureCropTools > PictureCropAspectRatioMenu | Y | ? |  |  |
| 16 | PictureFillCrop | button | PictureCropTools | Y | ? |  |  |
| 17 | PictureFitCrop | button | PictureCropTools | Y | ? |  |  |
| 18 | ShapeHeight | control |  | Y | ? |  |  |
| 19 | ShapeWidth | control |  | Y | ? |  |  |
| 20 | ObjectSizeAndPositionDialog | launcher |  | Y | ? |  |  |

Web evidence for this tab (S2, web tab `FormatPicture`). Single-line groups: none listed. Multiline groups: none listed.

Web control IDs with no match on this desktop tab: `ArrangeAnchorOnline`.


### Table Design (tab set `TabSetTableTools`)

ID `TabTableToolsDesign`. KeyTip: Alt+J, T [D S19]. Source S1, checked 2026-11-07.

Groups in source order: `GroupTableStyleOptionsPowerPoint`, `GroupTableStylesPowerPoint`, `GroupTextStylesTable`, `GroupDrawBorders`.

**GroupTableStyleOptionsPowerPoint** (I: "Table Style Options Power Point")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | TableStyleFirstRowPowerPoint | check |  | Y | ? |  |  |
| 2 | TableStyleTotalRowPowerPoint | check |  | Y | ? |  |  |
| 3 | TableStyleBandedRowsPowerPoint | check |  | Y | ? |  |  |
| 4 | TableStyleFirstColumnPowerPoint | check |  | Y | ? |  |  |
| 5 | TableStyleLastColumnPowerPoint | check |  | Y | ? |  |  |
| 6 | TableStyleBandedColumnsPowerPoint | check |  | Y | ? |  |  |

**GroupTableStylesPowerPoint** (I: "Table Styles Power Point")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | TableStylesGallery | gallery |  | Y | Yes [D S2] | Alt+J, T, A [D S19]; web J, L, A [D S19 Web] |  |
| 2 | TableStyleClearTable | button | TableStylesGallery | Y | ? |  |  |
| 3 | ShadingColorPicker | gallery |  | Y | Yes [D S2] | Alt+J, T, H [D S19]; web J, L, H [D S19 Web] |  |
| 4 | ObjectFillMoreColorsDialog | button | ShadingColorPicker | Y | ? |  |  |
| 5 | EyedropperFill | button | ShadingColorPicker | Y | ? |  |  |
| 6 | ObjectPictureFill | button | ShadingColorPicker | Y | ? |  |  |
| 7 | GradientGallery | gallery | ShadingColorPicker | Y | ? |  |  |
| 8 | ShapeFillMoreGradientsDialog | button | ShadingColorPicker > GradientGallery | Y | ? |  |  |
| 9 | ShapeFillTextureGallery | gallery | ShadingColorPicker | Y | ? |  |  |
| 10 | MoreTextureOptions | button | ShadingColorPicker > ShapeFillTextureGallery | Y | ? |  |  |
| 11 | TableBackgroundGallery | gallery | ShadingColorPicker | Y | ? |  |  |
| 12 | TableFillColorMoreColorsDialog | button | ShadingColorPicker > TableBackgroundGallery | Y | ? |  |  |
| 13 | EyedropperFill | button | ShadingColorPicker > TableBackgroundGallery | Y | ? |  |  |
| 14 | TableBackgroundPictureFill | button | ShadingColorPicker > TableBackgroundGallery | Y | ? |  |  |
| 15 | TableBordersMenu | split |  | Y | Yes [D S2] | Alt+J, T, B [D S19]; web J, L, B (web path differs from desktop) [D S19 Web] |  |
| 16 | BorderNone | toggle | TableBordersMenu | Y | ? |  |  |
| 17 | BordersAll | toggle | TableBordersMenu | Y | ? |  |  |
| 18 | BorderOutside | toggle | TableBordersMenu | Y | ? |  |  |
| 19 | BorderInside | toggle | TableBordersMenu | Y | ? |  |  |
| 20 | BorderTop | toggle | TableBordersMenu | Y | ? |  |  |
| 21 | BorderBottom | toggle | TableBordersMenu | Y | ? |  |  |
| 22 | BorderLeft | toggle | TableBordersMenu | Y | ? |  |  |
| 23 | BorderRight | toggle | TableBordersMenu | Y | ? |  |  |
| 24 | BorderInsideHorizontal | toggle | TableBordersMenu | Y | ? |  |  |
| 25 | BorderInsideVertical | toggle | TableBordersMenu | Y | ? |  |  |
| 26 | BorderDiagonalDown | toggle | TableBordersMenu | Y | ? |  |  |
| 27 | BorderDiagonalUp | toggle | TableBordersMenu | Y | ? |  |  |
| 28 | TableEffectsMenu | dropdown |  | Y | ? |  |  |
| 29 | TableEffectsCellBevelGallery | gallery | TableEffectsMenu | Y | ? |  |  |
| 30 | ObjectEffectShadowGallery | gallery | TableEffectsMenu | Y | ? |  |  |
| 31 | ShadowOptionsDialog | button | TableEffectsMenu > ObjectEffectShadowGallery | Y | ? |  |  |
| 32 | ReflectionGallery | gallery | TableEffectsMenu | Y | ? |  |  |
| 33 | ReflectionsMoreOptions | button | TableEffectsMenu > ReflectionGallery | Y | ? |  |  |

**GroupTextStylesTable** (I: "Text Styles Table")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | TableTextStylesGallery | gallery |  | Y | ? |  |  |
| 2 | WordArtClear | button | TableTextStylesGallery | Y | ? |  |  |
| 3 | TextFillColorPicker | gallery |  | Y | ? |  |  |
| 4 | TextFillColorMoreColorsDialog | button | TextFillColorPicker | Y | ? |  |  |
| 5 | EyedropperFillText | button | TextFillColorPicker | Y | ? |  |  |
| 6 | TextPictureFill | button | TextFillColorPicker | Y | ? |  |  |
| 7 | TextFillGradientGallery | gallery | TextFillColorPicker | Y | ? |  |  |
| 8 | TextFillMoreGradientsDialog | button | TextFillColorPicker > TextFillGradientGallery | Y | ? |  |  |
| 9 | TextFillTextureGallery | gallery | TextFillColorPicker | Y | ? |  |  |
| 10 | TextFillMoreTextures | button | TextFillColorPicker > TextFillTextureGallery | Y | ? |  |  |
| 11 | TextOutlineColorPicker | gallery |  | Y | ? |  |  |
| 12 | TextOutlineColorMoreColorsDialog | button | TextOutlineColorPicker | Y | ? |  |  |
| 13 | EyedropperOutlineText | button | TextOutlineColorPicker | Y | ? |  |  |
| 14 | TextOutlineWeightGallery | gallery | TextOutlineColorPicker | Y | ? |  |  |
| 15 | TextOutlineMoreLinesDialog | button | TextOutlineColorPicker > TextOutlineWeightGallery | Y | ? |  |  |
| 16 | TextOutlineDashesGallery | gallery | TextOutlineColorPicker | Y | ? |  |  |
| 17 | TextOutlineMoreLinesDialog | button | TextOutlineColorPicker > TextOutlineDashesGallery | Y | ? |  |  |
| 18 | TextEffectsMenu | dropdown |  | Y | ? |  |  |
| 19 | TextEffectShadowGallery | gallery | TextEffectsMenu | Y | ? |  |  |
| 20 | TextEffectsMoreShadowsDialog | button | TextEffectsMenu > TextEffectShadowGallery | Y | ? |  |  |
| 21 | TextReflectionGallery | gallery | TextEffectsMenu | Y | ? |  |  |
| 22 | TextEffectsReflectionsMoreOptions | button | TextEffectsMenu > TextReflectionGallery | Y | ? |  |  |
| 23 | TextEffectGlowGallery | gallery | TextEffectsMenu | Y | ? |  |  |
| 24 | TextGlowColorPicker | gallery | TextEffectsMenu > TextEffectGlowGallery | Y | ? |  |  |
| 25 | TextGlowColorMoreColorsDialog | button | TextEffectsMenu > TextEffectGlowGallery | Y | ? |  |  |
| 26 | EyedropperGlowText | button | TextEffectsMenu > TextEffectGlowGallery | Y | ? |  |  |
| 27 | TextEffectsGlowsMoreOptions | button | TextEffectsMenu > TextEffectGlowGallery | Y | ? |  |  |
| 28 | BevelTextGallery | gallery | TextEffectsMenu | Y | ? |  |  |
| 29 | TextEffectsBevelMore3DOptionsDialog | button | TextEffectsMenu > BevelTextGallery | Y | ? |  |  |
| 30 | TextEffects3DRotationGallery | gallery | TextEffectsMenu | Y | ? |  |  |
| 31 | TextEffects3DRotationOptionsDialog | button | TextEffectsMenu > TextEffects3DRotationGallery | Y | ? |  |  |
| 32 | TextEffectTransformGallery | gallery | TextEffectsMenu | Y | ? |  |  |
| 33 | WordArtFormatDialog | launcher |  | Y | ? |  |  |

**GroupDrawBorders** (I: "Draw Borders")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | TableDrawBorderPenStyle | dropdown-list |  | Y | ? | Alt+J, T, L [D S19] |  |
| 2 | TableDrawBorderPenWeight | dropdown-list |  | Y | Yes [D S2] |  |  |
| 3 | TableBorderPenColorPicker | gallery |  | Y | Yes [D S2] |  |  |
| 4 | TableBorderColorMoreColorsDialog | button | TableBorderPenColorPicker | Y | ? |  |  |
| 5 | TableDrawTable | toggle |  | Y | ? |  |  |
| 6 | TableEraser | toggle |  | Y | ? |  |  |

Web evidence for this tab (S2, web tab `Table`). Single-line groups: `GroupAlignment`, `GroupMerge`, `GroupTable`, `GroupTableCellSize`, `GroupTableRowsAndColumns`, `GroupTableShadingOnline`, `GroupTableStyleOptionsPowerPoint`, `GroupTableStylesPowerPoint`. Multiline groups: `GroupTableStyleOptionsPowerPoint`, `GroupTableStylesPowerPoint`, `GroupAlignment`, `GroupMerge`, `GroupTable`, `GroupTableCellSize`, `GroupTableRowsAndColumns`, `GroupTableSize`, `GroupTableShadingOnline`, `GroupTablePropertiesOnline`.

Web control IDs with no match on this desktop tab: `MergeCells`, `SetSizeAnchorOnline`, `SplitCells`, `TableAlternativeTextOnline`, `TableColumnsDistribute`, `TableDeleteRownsAndColumnsMenuWord`, `TableInsertColumnLeft`, `TableInsertColumnRight`, `TableInsertOnline`, `TableInsertRowAbove`, `TableInsertRowBelow`, `TableRowsDistribute`, `TableSelectMenuPowerPoint`.


### Table Layout

ID `TabTableToolsLayout`. KeyTip: Alt+J, L [D S19]. Source S1, checked 2026-11-07.

Groups in source order: `GroupTable`, `GroupTableRowsAndColumns`, `GroupMerge`, `GroupTableCellSize`, `GroupAlignment`, `GroupTableSize`, `GroupArrange`.

**GroupTable** (I: "Table")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | TableSelectMenuPowerPoint | dropdown |  | Y | Yes [D S2] |  |  |
| 2 | TableSelect | button | TableSelectMenuPowerPoint | Y | ? |  |  |
| 3 | TableColumnSelect | button | TableSelectMenuPowerPoint | Y | ? |  |  |
| 4 | TableRowSelect | button | TableSelectMenuPowerPoint | Y | ? |  |  |
| 5 | TableShowGridlines | toggle |  | Y | ? |  |  |

**GroupTableRowsAndColumns** (I: "Table Rows And Columns")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | TableDeleteRowsAndColumnsMenuWord | dropdown |  | Y | ? | Alt+J, L, D [D S19] |  |
| 2 | TableDeleteColumns | button | TableDeleteRowsAndColumnsMenuWord | Y | ? |  |  |
| 3 | TableDeleteRows | button | TableDeleteRowsAndColumnsMenuWord | Y | ? |  |  |
| 4 | TableDelete | button | TableDeleteRowsAndColumnsMenuWord | Y | ? |  |  |
| 5 | TableInsertRowsAbove | button |  | Y | ? |  |  |
| 6 | TableInsertRowsBelow | button |  | Y | ? |  |  |
| 7 | TableInsertColumnsLeft | button |  | Y | ? |  |  |
| 8 | TableInsertColumnsRight | button |  | Y | ? |  |  |

**GroupMerge** (I: "Merge")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | MergeCells | button |  | Y | Yes [D S2] |  |  |
| 2 | SplitCells | button |  | Y | Yes [D S2] |  |  |

**GroupTableCellSize** (I: "Table Cell Size")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | TableRowHeight | control |  | Y | ? |  |  |
| 2 | TableColumnWidth | control |  | Y | ? |  |  |
| 3 | TableRowsDistribute | button |  | Y | Yes [D S2] |  |  |
| 4 | TableColumnsDistribute | button |  | Y | Yes [D S2] |  |  |

**GroupAlignment** (I: "Alignment")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | AlignLeft | toggle |  | Y | Yes, web tab Home [D S2] |  | Ctrl+L [D S3]; web Ctrl+L [D S3 Web] |
| 2 | AlignCenter | toggle |  | Y | Yes, web tab Home [D S2] |  | Ctrl+E [D S3]; web Ctrl+E [D S3 Web] |
| 3 | AlignRight | toggle |  | Y | Yes, web tab Home [D S2] |  | Ctrl+R [D S3]; web Ctrl+R [D S3 Web] |
| 4 | TableCellAlignTop | toggle |  | Y | ? |  |  |
| 5 | TableCellAlignCenterVertically | toggle |  | Y | ? |  |  |
| 6 | TableCellAlignBottom | toggle |  | Y | ? |  |  |
| 7 | TableSetLeftToRight | toggle |  | Y | ? |  |  |
| 8 | TableSetRightToLeft | toggle |  | Y | ? |  |  |
| 9 | TextDirectionGallery | gallery |  | Y | ? |  |  |
| 10 | TextDirectionMoreOptionsDialog | button | TextDirectionGallery | Y | ? |  |  |
| 11 | TableCellMarginsGallery | gallery |  | Y | ? |  |  |
| 12 | TableCellCustomMarginsDialog | button | TableCellMarginsGallery | Y | ? |  |  |

**GroupTableSize** (I: "Table Size")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | TableHeight | control |  | Y | ? |  |  |
| 2 | TableWidth | control |  | Y | ? |  |  |
| 3 | TableLockAspectRatio | check |  | Y | ? |  |  |

**GroupArrange** (I: "Arrange")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ObjectBringForwardMenu | split |  | Y | ? |  |  |
| 2 | ObjectBringForward | button | ObjectBringForwardMenu | Y | Yes, web tab FormatShape [D S2] |  | Ctrl+Shift+] [D S3]; web Ctrl+] [D S3 Web] |
| 3 | ObjectBringToFront | button | ObjectBringForwardMenu | Y | Yes, web tab FormatShape [D S2] |  | web Ctrl+Shift+] [D S3 Web] |
| 4 | ObjectSendBackwardMenu | split |  | Y | ? |  |  |
| 5 | ObjectSendBackward | button | ObjectSendBackwardMenu | Y | Yes, web tab FormatShape [D S2] |  | Ctrl+Shift+[ [D S3]; web Ctrl+[ [D S3 Web] |
| 6 | ObjectSendToBack | button | ObjectSendBackwardMenu | Y | Yes, web tab FormatShape [D S2] |  | web Ctrl+Shift+[ [D S3 Web] |
| 7 | SelectionPane | toggle |  | Y | Yes, web tab Home [D S2] |  | Alt+F10 [D S3] |
| 8 | ObjectAlignMenu | dropdown |  | Y | Yes, web tab FormatShape [D S2] |  |  |
| 9 | ObjectsAlignLeftSmart | button | ObjectAlignMenu | Y | Yes, web tab FormatShape [D S2] |  |  |
| 10 | ObjectsAlignCenterHorizontalSmart | button | ObjectAlignMenu | Y | Yes, web tab FormatShape [D S2] |  |  |
| 11 | ObjectsAlignRightSmart | button | ObjectAlignMenu | Y | Yes, web tab FormatShape [D S2] |  |  |
| 12 | ObjectsAlignTopSmart | button | ObjectAlignMenu | Y | Yes, web tab FormatShape [D S2] |  |  |
| 13 | ObjectsAlignMiddleVerticalSmart | button | ObjectAlignMenu | Y | Yes, web tab FormatShape [D S2] |  |  |
| 14 | ObjectsAlignBottomSmart | button | ObjectAlignMenu | Y | Yes, web tab FormatShape [D S2] |  |  |
| 15 | AlignDistributeHorizontally | button | ObjectAlignMenu | Y | Yes, web tab Home [D S2] |  |  |
| 16 | AlignDistributeVertically | button | ObjectAlignMenu | Y | Yes, web tab Home [D S2] |  |  |
| 17 | ObjectsAlignRelativeToContainerSmart | toggle | ObjectAlignMenu | Y | Yes, web tab FormatShape [D S2] |  |  |
| 18 | ObjectsAlignSelectedSmart | toggle | ObjectAlignMenu | Y | ? |  |  |
| 19 | ObjectsGroupMenu | dropdown |  | Y | Yes, web tab FormatShape [D S2] |  |  |
| 20 | ObjectsGroup | button | ObjectsGroupMenu | Y | Yes, web tab FormatShape [D S2] |  | Ctrl+G [D S3] |
| 21 | ObjectsRegroup | button | ObjectsGroupMenu | Y | ? |  | Ctrl+Shift+J [D S3] |
| 22 | ObjectsUngroup | button | ObjectsGroupMenu | Y | Yes, web tab FormatShape [D S2] |  | Ctrl+Shift+G [D S3] |
| 23 | ObjectRotateGallery | gallery |  | Y | Yes, web tab FormatShape [D S2] |  |  |
| 24 | ObjectRotationOptionsDialog | button | ObjectRotateGallery | Y | Yes, web tab FormatShape [D S2] |  |  |
| 25 | ObjectsArrangeMenu | dropdown |  | Y | ? |  |  |
| 26 | ObjectBringToFront | button | ObjectsArrangeMenu | Y | Yes, web tab FormatShape [D S2] |  | web Ctrl+Shift+] [D S3 Web] |
| 27 | ObjectSendToBack | button | ObjectsArrangeMenu | Y | Yes, web tab FormatShape [D S2] |  | web Ctrl+Shift+[ [D S3 Web] |
| 28 | ObjectBringForward | button | ObjectsArrangeMenu | Y | Yes, web tab FormatShape [D S2] |  | Ctrl+Shift+] [D S3]; web Ctrl+] [D S3 Web] |
| 29 | ObjectSendBackward | button | ObjectsArrangeMenu | Y | Yes, web tab FormatShape [D S2] |  | Ctrl+Shift+[ [D S3]; web Ctrl+[ [D S3 Web] |
| 30 | ObjectsGroup | button | ObjectsArrangeMenu | Y | Yes, web tab FormatShape [D S2] |  | Ctrl+G [D S3] |
| 31 | ObjectsUngroup | button | ObjectsArrangeMenu | Y | Yes, web tab FormatShape [D S2] |  | Ctrl+Shift+G [D S3] |
| 32 | ObjectsRegroup | button | ObjectsArrangeMenu | Y | ? |  | Ctrl+Shift+J [D S3] |
| 33 | ObjectAlignMenu | dropdown | ObjectsArrangeMenu | Y | Yes, web tab FormatShape [D S2] |  |  |
| 34 | ObjectsAlignLeftSmart | button | ObjectsArrangeMenu > ObjectAlignMenu | Y | Yes, web tab FormatShape [D S2] |  |  |
| 35 | ObjectsAlignCenterHorizontalSmart | button | ObjectsArrangeMenu > ObjectAlignMenu | Y | Yes, web tab FormatShape [D S2] |  |  |
| 36 | ObjectsAlignRightSmart | button | ObjectsArrangeMenu > ObjectAlignMenu | Y | Yes, web tab FormatShape [D S2] |  |  |
| 37 | ObjectsAlignTopSmart | button | ObjectsArrangeMenu > ObjectAlignMenu | Y | Yes, web tab FormatShape [D S2] |  |  |
| 38 | ObjectsAlignMiddleVerticalSmart | button | ObjectsArrangeMenu > ObjectAlignMenu | Y | Yes, web tab FormatShape [D S2] |  |  |
| 39 | ObjectsAlignBottomSmart | button | ObjectsArrangeMenu > ObjectAlignMenu | Y | Yes, web tab FormatShape [D S2] |  |  |
| 40 | AlignDistributeHorizontally | button | ObjectsArrangeMenu > ObjectAlignMenu | Y | Yes, web tab Home [D S2] |  |  |
| 41 | AlignDistributeVertically | button | ObjectsArrangeMenu > ObjectAlignMenu | Y | Yes, web tab Home [D S2] |  |  |
| 42 | ObjectsAlignRelativeToContainerSmart | toggle | ObjectsArrangeMenu > ObjectAlignMenu | Y | Yes, web tab FormatShape [D S2] |  |  |
| 43 | ObjectsAlignSelectedSmart | toggle | ObjectsArrangeMenu > ObjectAlignMenu | Y | ? |  |  |
| 44 | ObjectRotateGallery | gallery | ObjectsArrangeMenu | Y | Yes, web tab FormatShape [D S2] |  |  |
| 45 | ObjectRotationOptionsDialog | button | ObjectsArrangeMenu > ObjectRotateGallery | Y | Yes, web tab FormatShape [D S2] |  |  |
| 46 | SelectionPane | toggle | ObjectsArrangeMenu | Y | Yes, web tab Home [D S2] |  | Alt+F10 [D S3] |

Web evidence for this tab (S2, web tab `Table`). Single-line groups: `GroupAlignment`, `GroupMerge`, `GroupTable`, `GroupTableCellSize`, `GroupTableRowsAndColumns`, `GroupTableShadingOnline`, `GroupTableStyleOptionsPowerPoint`, `GroupTableStylesPowerPoint`. Multiline groups: `GroupTableStyleOptionsPowerPoint`, `GroupTableStylesPowerPoint`, `GroupAlignment`, `GroupMerge`, `GroupTable`, `GroupTableCellSize`, `GroupTableRowsAndColumns`, `GroupTableSize`, `GroupTableShadingOnline`, `GroupTablePropertiesOnline`.

Web control IDs with no match on this desktop tab: `SetSizeAnchorOnline`, `ShadingColorPicker`, `TableAlternativeTextOnline`, `TableBorderPenColorPicker`, `TableBordersMenu`, `TableDeleteRownsAndColumnsMenuWord`, `TableDrawBorderPenWeight`, `TableInsertColumnLeft`, `TableInsertColumnRight`, `TableInsertOnline`, `TableInsertRowAbove`, `TableInsertRowBelow`, `TableStylesGallery`.


### Chart Design (tab set `TabSetChartTools`)

ID `TabChartToolsDesignNew`. KeyTip: Alt+J, C [D S21]. Source S1, checked 2026-11-07.

Groups in source order: `GroupChartLayouts`, `GroupChartStyles`, `GroupChartData`, `GroupChartType`.

**GroupChartLayouts** (I: "Chart Layouts")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | AddChartElementMenu | dropdown |  | Y | ? |  |  |
| 2 | ChartAxesGallery | gallery | AddChartElementMenu | Y | ? |  |  |
| 3 | ChartAxesOptionsDialog | button | AddChartElementMenu > ChartAxesGallery | Y | ? |  |  |
| 4 | ChartAxisTitlesGallery | gallery | AddChartElementMenu | Y | ? |  |  |
| 5 | ChartAxesTitlesOptionsDialog | button | AddChartElementMenu > ChartAxisTitlesGallery | Y | ? |  |  |
| 6 | ChartTitle | gallery | AddChartElementMenu | Y | ? |  |  |
| 7 | ChartTitleOptionsDialog | button | AddChartElementMenu > ChartTitle | Y | ? |  |  |
| 8 | ChartDataLabel | gallery | AddChartElementMenu | Y | ? |  |  |
| 9 | ChartDataLabelDialog | button | AddChartElementMenu > ChartDataLabel | Y | ? |  |  |
| 10 | ChartDataTable | gallery | AddChartElementMenu | Y | ? |  |  |
| 11 | ChartDataTableOptionsDialog | button | AddChartElementMenu > ChartDataTable | Y | ? |  |  |
| 12 | ChartErrorBars | gallery | AddChartElementMenu | Y | ? |  |  |
| 13 | ChartErrorBarsOptionsDialog | button | AddChartElementMenu > ChartErrorBars | Y | ? |  |  |
| 14 | ChartGridlinesGallery | gallery | AddChartElementMenu | Y | ? |  |  |
| 15 | ChartGridlineOptionsDialog | button | AddChartElementMenu > ChartGridlinesGallery | Y | ? |  |  |
| 16 | ChartLegend | gallery | AddChartElementMenu | Y | ? |  |  |
| 17 | ChartLegendOptionsDialogDialog | button | AddChartElementMenu > ChartLegend | Y | ? |  |  |
| 18 | ChartLines | gallery | AddChartElementMenu | Y | ? |  |  |
| 19 | ChartTrendline | gallery | AddChartElementMenu | Y | ? |  |  |
| 20 | ChartTrendlineOptionsDialog | button | AddChartElementMenu > ChartTrendline | Y | ? |  |  |
| 21 | ChartUpDownBars | gallery | AddChartElementMenu | Y | ? |  |  |
| 22 | ChartUpDownBarsOptionsDialog | button | AddChartElementMenu > ChartUpDownBars | Y | ? |  |  |
| 23 | ChartLayoutGallery | gallery |  | Y | ? |  |  |

**GroupChartStyles** (I: "Chart Styles")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ChartColorsGallery | gallery |  | Y | ? |  |  |
| 2 | ChartStylesGallery | gallery |  | Y | ? |  |  |

**GroupChartData** (I: "Chart Data")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ChartSwitchRowColumn | button |  | Y | ? |  |  |
| 2 | ChartEditDataSource | button |  | Y | ? |  |  |
| 3 | ChartEditDataMenu | split |  | Y | ? |  |  |
| 4 | ChartShowDataGrid | button | ChartEditDataMenu | Y | ? |  |  |
| 5 | ChartShowDataContextualMenu | button | ChartEditDataMenu | Y | ? |  |  |
| 6 | ChartRefresh | button |  | Y | ? |  |  |

**GroupChartType** (I: "Chart Type")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ChartChangeType | button |  | Y | ? |  |  |


### Format, chart (tab set `TabSetChartTools`)

ID `TabChartToolsFormatNew`. KeyTip: not sourced. Source S1, checked 2026-11-07.

Groups in source order: `GroupChartCurrentSelection`, `GroupShapesChart`, `GroupChartShapeStyles`, `GroupWordArtStyles`, `GroupAltText`, `GroupArrange`, `GroupSize`.

**GroupChartCurrentSelection** (I: "Chart Current Selection")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ChartElementSelector | combo |  | Y | ? |  |  |
| 2 | ChartFormatSelection | button |  | Y | ? |  |  |
| 3 | ChartResetToMatchStyle | button |  | Y | ? |  |  |

**GroupShapesChart** (I: "Shapes Chart")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ShapesInsertGallery | gallery |  | Y | Yes, web tab FormatShape [D S2] |  |  |
| 2 | ShapeChangeShapeGallery | gallery |  | Y | ? |  |  |

**GroupChartShapeStyles** (I: "Chart Shape Styles")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ShapeStylesGallery | gallery |  | Y | Yes, web tab FormatShape [D S2] |  |  |
| 2 | ShapeStylesOtherThemeFillsGallery | gallery | ShapeStylesGallery | Y | ? |  |  |
| 3 | ShapeFillColorPicker | gallery |  | Y | Yes, web tab FormatShape [D S2] |  |  |
| 4 | ObjectFillMoreColorsDialog | button | ShapeFillColorPicker | Y | ? |  |  |
| 5 | EyedropperFill | button | ShapeFillColorPicker | Y | ? |  |  |
| 6 | ObjectPictureFill | button | ShapeFillColorPicker | Y | ? |  |  |
| 7 | GradientGallery | gallery | ShapeFillColorPicker | Y | ? |  |  |
| 8 | ShapeFillMoreGradientsDialog | button | ShapeFillColorPicker > GradientGallery | Y | ? |  |  |
| 9 | ShapeFillTextureGallery | gallery | ShapeFillColorPicker | Y | ? |  |  |
| 10 | MoreTextureOptions | button | ShapeFillColorPicker > ShapeFillTextureGallery | Y | ? |  |  |
| 11 | ColorPickerChartLineStyles | gallery |  | Y | ? |  |  |
| 12 | ObjectBorderOutlineColorMoreColorsDialog | button | ColorPickerChartLineStyles | Y | ? |  |  |
| 13 | EyedropperOutline | button | ColorPickerChartLineStyles | Y | ? |  |  |
| 14 | OutlineWeightGallery | gallery | ColorPickerChartLineStyles | Y | ? |  |  |
| 15 | LineStylesDialog | button | ColorPickerChartLineStyles > OutlineWeightGallery | Y | ? |  |  |
| 16 | OutlineDashesGallery | gallery | ColorPickerChartLineStyles | Y | ? |  |  |
| 17 | LineStylesDialog | button | ColorPickerChartLineStyles > OutlineDashesGallery | Y | ? |  |  |
| 18 | ArrowStyleGallery | gallery | ColorPickerChartLineStyles | Y | ? |  |  |
| 19 | ArrowsMore | button | ColorPickerChartLineStyles > ArrowStyleGallery | Y | ? |  |  |
| 20 | ShapeEffectsMenu | dropdown |  | Y | ? |  |  |
| 21 | ObjectEffectPresetGallery | gallery | ShapeEffectsMenu | Y | ? |  |  |
| 22 | _3DBevelOptionsDialog | button | ShapeEffectsMenu > ObjectEffectPresetGallery | Y | ? |  |  |
| 23 | ObjectEffectShadowGallery | gallery | ShapeEffectsMenu | Y | ? |  |  |
| 24 | ShadowOptionsDialog | button | ShapeEffectsMenu > ObjectEffectShadowGallery | Y | ? |  |  |
| 25 | ReflectionGallery | gallery | ShapeEffectsMenu | Y | ? |  |  |
| 26 | ReflectionsMoreOptions | button | ShapeEffectsMenu > ReflectionGallery | Y | ? |  |  |
| 27 | ObjectEffectGlowGallery | gallery | ShapeEffectsMenu | Y | ? |  |  |
| 28 | GlowColorPicker | gallery | ShapeEffectsMenu > ObjectEffectGlowGallery | Y | ? |  |  |
| 29 | GlowColorMoreColorsDialog | button | ShapeEffectsMenu > ObjectEffectGlowGallery | Y | ? |  |  |
| 30 | EyedropperGlow | button | ShapeEffectsMenu > ObjectEffectGlowGallery | Y | ? |  |  |
| 31 | GlowsMoreOptions | button | ShapeEffectsMenu > ObjectEffectGlowGallery | Y | ? |  |  |
| 32 | ObjectEffectSoftEdgesGallery | gallery | ShapeEffectsMenu | Y | ? |  |  |
| 33 | SoftEdgesMoreOptions | button | ShapeEffectsMenu > ObjectEffectSoftEdgesGallery | Y | ? |  |  |
| 34 | BevelShapeGallery | gallery | ShapeEffectsMenu | Y | ? |  |  |
| 35 | _3DBevelOptionsDialog | button | ShapeEffectsMenu > BevelShapeGallery | Y | ? |  |  |
| 36 | _3DRotationGallery | gallery | ShapeEffectsMenu | Y | ? |  |  |
| 37 | _3DRotationOptionsDialog | button | ShapeEffectsMenu > _3DRotationGallery | Y | ? |  |  |
| 38 | ObjectFormatDialog | launcher |  | Y | ? |  |  |

**GroupWordArtStyles** (I: "Word Art Styles")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | TextStylesGallery | gallery |  | Y | ? |  |  |
| 2 | WordArtClear | button | TextStylesGallery | Y | ? |  |  |
| 3 | TextFillColorPicker | gallery |  | Y | ? |  |  |
| 4 | TextFillColorMoreColorsDialog | button | TextFillColorPicker | Y | ? |  |  |
| 5 | EyedropperFillText | button | TextFillColorPicker | Y | ? |  |  |
| 6 | TextPictureFill | button | TextFillColorPicker | Y | ? |  |  |
| 7 | TextFillGradientGallery | gallery | TextFillColorPicker | Y | ? |  |  |
| 8 | TextFillMoreGradientsDialog | button | TextFillColorPicker > TextFillGradientGallery | Y | ? |  |  |
| 9 | TextFillTextureGallery | gallery | TextFillColorPicker | Y | ? |  |  |
| 10 | TextFillMoreTextures | button | TextFillColorPicker > TextFillTextureGallery | Y | ? |  |  |
| 11 | TextOutlineColorPicker | gallery |  | Y | ? |  |  |
| 12 | TextOutlineColorMoreColorsDialog | button | TextOutlineColorPicker | Y | ? |  |  |
| 13 | EyedropperOutlineText | button | TextOutlineColorPicker | Y | ? |  |  |
| 14 | TextOutlineWeightGallery | gallery | TextOutlineColorPicker | Y | ? |  |  |
| 15 | TextOutlineMoreLinesDialog | button | TextOutlineColorPicker > TextOutlineWeightGallery | Y | ? |  |  |
| 16 | TextOutlineDashesGallery | gallery | TextOutlineColorPicker | Y | ? |  |  |
| 17 | TextOutlineMoreLinesDialog | button | TextOutlineColorPicker > TextOutlineDashesGallery | Y | ? |  |  |
| 18 | TextEffectsMenu | dropdown |  | Y | ? |  |  |
| 19 | TextEffectShadowGallery | gallery | TextEffectsMenu | Y | ? |  |  |
| 20 | TextEffectsMoreShadowsDialog | button | TextEffectsMenu > TextEffectShadowGallery | Y | ? |  |  |
| 21 | TextReflectionGallery | gallery | TextEffectsMenu | Y | ? |  |  |
| 22 | TextEffectsReflectionsMoreOptions | button | TextEffectsMenu > TextReflectionGallery | Y | ? |  |  |
| 23 | TextEffectGlowGallery | gallery | TextEffectsMenu | Y | ? |  |  |
| 24 | TextGlowColorPicker | gallery | TextEffectsMenu > TextEffectGlowGallery | Y | ? |  |  |
| 25 | TextGlowColorMoreColorsDialog | button | TextEffectsMenu > TextEffectGlowGallery | Y | ? |  |  |
| 26 | EyedropperGlowText | button | TextEffectsMenu > TextEffectGlowGallery | Y | ? |  |  |
| 27 | TextEffectsGlowsMoreOptions | button | TextEffectsMenu > TextEffectGlowGallery | Y | ? |  |  |
| 28 | BevelTextGallery | gallery | TextEffectsMenu | Y | ? |  |  |
| 29 | TextEffectsBevelMore3DOptionsDialog | button | TextEffectsMenu > BevelTextGallery | Y | ? |  |  |
| 30 | TextEffects3DRotationGallery | gallery | TextEffectsMenu | Y | ? |  |  |
| 31 | TextEffects3DRotationOptionsDialog | button | TextEffectsMenu > TextEffects3DRotationGallery | Y | ? |  |  |
| 32 | TextEffectTransformGallery | gallery | TextEffectsMenu | Y | ? |  |  |
| 33 | WordArtFormatDialog | launcher |  | Y | ? |  |  |

**GroupAltText** (I: "Alt Text"). 1 rows identical to `TabDrawingToolsFormat`.

**GroupArrange** (I: "Arrange"). 46 rows identical to `TabTableToolsLayout`.

**GroupSize** (I: "Size"). 3 rows identical to `TabDrawingToolsFormat`.


### SmartArt Design (tab set `TabSetSmartArtTools`)

ID `TabSmartArtToolsDesign`. KeyTip: Alt+J, S [D S19]. Source S1, checked 2026-11-07.

Groups in source order: `GroupSmartArtCreateGraphic`, `GroupSmartArtLayouts`, `GroupSmartArtQuickStyles`, `GroupSmartArtReset`.

**GroupSmartArtCreateGraphic** (I: "Smart Art Create Graphic")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | SmartArtAddShapeSplitMenu | split |  | Y | ? |  |  |
| 2 | SmartArtAddShape | button | SmartArtAddShapeSplitMenu | Y | ? |  |  |
| 3 | SmartArtAddShapeAfter | button | SmartArtAddShapeSplitMenu | Y | ? |  |  |
| 4 | SmartArtAddShapeBefore | button | SmartArtAddShapeSplitMenu | Y | ? |  |  |
| 5 | SmartArtAddShapeAbove | button | SmartArtAddShapeSplitMenu | Y | ? |  |  |
| 6 | SmartArtAddShapeBelow | button | SmartArtAddShapeSplitMenu | Y | ? |  |  |
| 7 | SmartArtAddAssistant | button | SmartArtAddShapeSplitMenu | Y | ? |  |  |
| 8 | SmartArtAddBullet | button |  | Y | ? |  |  |
| 9 | SmartArtTextPane | toggle |  | Y | Yes [D S2] |  |  |
| 10 | SmartArtPromote | button |  | Y | Yes [D S2] |  |  |
| 11 | SmartArtDemote | button |  | Y | Yes [D S2] |  |  |
| 12 | SmartArtRightToLeft | toggle |  | Y | Yes [D S2] |  |  |
| 13 | SmartArtReorderUp | button |  | Y | ? |  |  |
| 14 | SmartArtReorderDown | button |  | Y | ? |  |  |
| 15 | SmartArtOrganizationChartMenu | dropdown |  | Y | ? |  |  |
| 16 | SmartArtOrganizationChartStandard | button | SmartArtOrganizationChartMenu | Y | ? |  |  |
| 17 | SmartArtOrganizationChartBoth | button | SmartArtOrganizationChartMenu | Y | ? |  |  |
| 18 | SmartArtOrganizationChartLeftHanging | button | SmartArtOrganizationChartMenu | Y | ? |  |  |
| 19 | SmartArtOrganizationChartRightHanging | button | SmartArtOrganizationChartMenu | Y | ? |  |  |

**GroupSmartArtLayouts** (I: "Smart Art Layouts")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | SmartArtLayoutGallery | gallery |  | Y | Yes [D S2] |  |  |
| 2 | SmartArtMoreLayoutsDialog | button | SmartArtLayoutGallery | Y | ? |  |  |

**GroupSmartArtQuickStyles** (I: "Smart Art Quick Styles")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | SmartArtChangeColorsGallery | gallery |  | Y | Yes [D S2] |  |  |
| 2 | SmartArtRecolorImages | toggle | SmartArtChangeColorsGallery | Y | ? |  |  |
| 3 | SmartArtStylesGallery | gallery |  | Y | Yes [D S2] |  |  |

**GroupSmartArtReset** (I: "Smart Art Reset")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | SmartArtResetGraphic | button |  | Y | Yes [D S2] |  |  |
| 2 | ConvertSmartArtMenu | dropdown |  | Y | ? |  |  |
| 3 | SmartArtConvedrtToText | button | ConvertSmartArtMenu | Y | ? |  |  |
| 4 | SmartArtConvertToShapes | button | ConvertSmartArtMenu | Y | ? |  |  |

Web evidence for this tab (S2, web tab `SmartArtDesign`). Single-line groups: `GroupSmartArtCreateGraphic`, `GroupSmartArtLayouts`, `GroupSmartArtQuickStyles`, `GroupSmartArtReset`. Multiline groups: `GroupSmartArtCreateGraphic`, `GroupSmartArtLayouts`, `GroupSmartArtQuickStyles`, `GroupSmartArtReset`.


### Format, SmartArt

ID `TabSmartArtToolsFormat`. KeyTip: Alt+J, O [D S19]. Source S1, checked 2026-11-07.

Groups in source order: `GroupSmartArtShapes`, `GroupShapeStyles`, `GroupWordArtStyles`, `GroupAltText`, `GroupArrangeWith3DEditor`, `GroupSmartArtSize`.

**GroupSmartArtShapes** (I: "Smart Art Shapes")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | SmartArtEditIn2D | toggle |  | Y | ? |  |  |
| 2 | ShapeChangeShapeGallery | gallery |  | Y | ? |  |  |
| 3 | SmartArtLargerShape | button |  | Y | ? |  |  |
| 4 | SmartArtSmallerShape | button |  | Y | ? |  |  |

**GroupShapeStyles** (I: "Shape Styles")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ShapeStylesGallery | gallery |  | Y | Yes, web tab FormatShape [D S2] |  |  |
| 2 | ShapeStylesOtherThemeFillsGallery | gallery | ShapeStylesGallery | Y | ? |  |  |
| 3 | ShapeFillColorPicker | gallery |  | Y | Yes, web tab FormatShape [D S2] |  |  |
| 4 | ObjectFillMoreColorsDialog | button | ShapeFillColorPicker | Y | ? |  |  |
| 5 | EyedropperFill | button | ShapeFillColorPicker | Y | ? |  |  |
| 6 | ObjectPictureFill | button | ShapeFillColorPicker | Y | ? |  |  |
| 7 | GradientGallery | gallery | ShapeFillColorPicker | Y | ? |  |  |
| 8 | ShapeFillMoreGradientsDialog | button | ShapeFillColorPicker > GradientGallery | Y | ? |  |  |
| 9 | ShapeFillTextureGallery | gallery | ShapeFillColorPicker | Y | ? |  |  |
| 10 | MoreTextureOptions | button | ShapeFillColorPicker > ShapeFillTextureGallery | Y | ? |  |  |
| 11 | ShapeOutlineColorPicker | gallery |  | Y | Yes, web tab FormatShape [D S2] |  |  |
| 12 | ObjectBorderOutlineColorMoreColorsDialog | button | ShapeOutlineColorPicker | Y | ? |  |  |
| 13 | EyedropperOutline | button | ShapeOutlineColorPicker | Y | ? |  |  |
| 14 | OutlineWeightGallery | gallery | ShapeOutlineColorPicker | Y | ? |  |  |
| 15 | LineStylesDialog | button | ShapeOutlineColorPicker > OutlineWeightGallery | Y | ? |  |  |
| 16 | GallerySketchyShapes | gallery | ShapeOutlineColorPicker | Y | ? |  |  |
| 17 | MoreLines2 | button | ShapeOutlineColorPicker > GallerySketchyShapes | Y | ? |  |  |
| 18 | OutlineDashesGallery | gallery | ShapeOutlineColorPicker | Y | ? |  |  |
| 19 | LineStylesDialog | button | ShapeOutlineColorPicker > OutlineDashesGallery | Y | ? |  |  |
| 20 | ArrowStyleGallery | gallery | ShapeOutlineColorPicker | Y | ? |  |  |
| 21 | ArrowsMore | button | ShapeOutlineColorPicker > ArrowStyleGallery | Y | ? |  |  |
| 22 | ShapeEffectsMenu | dropdown |  | Y | ? |  |  |
| 23 | ObjectEffectPresetGallery | gallery | ShapeEffectsMenu | Y | ? |  |  |
| 24 | _3DBevelOptionsDialog | button | ShapeEffectsMenu > ObjectEffectPresetGallery | Y | ? |  |  |
| 25 | ObjectEffectShadowGallery | gallery | ShapeEffectsMenu | Y | ? |  |  |
| 26 | ShadowOptionsDialog | button | ShapeEffectsMenu > ObjectEffectShadowGallery | Y | ? |  |  |
| 27 | ReflectionGallery | gallery | ShapeEffectsMenu | Y | ? |  |  |
| 28 | ReflectionsMoreOptions | button | ShapeEffectsMenu > ReflectionGallery | Y | ? |  |  |
| 29 | ObjectEffectGlowGallery | gallery | ShapeEffectsMenu | Y | ? |  |  |
| 30 | GlowColorPicker | gallery | ShapeEffectsMenu > ObjectEffectGlowGallery | Y | ? |  |  |
| 31 | GlowColorMoreColorsDialog | button | ShapeEffectsMenu > ObjectEffectGlowGallery | Y | ? |  |  |
| 32 | EyedropperGlow | button | ShapeEffectsMenu > ObjectEffectGlowGallery | Y | ? |  |  |
| 33 | GlowsMoreOptions | button | ShapeEffectsMenu > ObjectEffectGlowGallery | Y | ? |  |  |
| 34 | ObjectEffectSoftEdgesGallery | gallery | ShapeEffectsMenu | Y | ? |  |  |
| 35 | SoftEdgesMoreOptions | button | ShapeEffectsMenu > ObjectEffectSoftEdgesGallery | Y | ? |  |  |
| 36 | BevelShapeGallery | gallery | ShapeEffectsMenu | Y | ? |  |  |
| 37 | _3DBevelOptionsDialog | button | ShapeEffectsMenu > BevelShapeGallery | Y | ? |  |  |
| 38 | _3DRotationGallery | gallery | ShapeEffectsMenu | Y | ? |  |  |
| 39 | _3DRotationOptionsDialog | button | ShapeEffectsMenu > _3DRotationGallery | Y | ? |  |  |
| 40 | ObjectFormatDialog | launcher |  | Y | ? |  |  |

**GroupWordArtStyles** (I: "Word Art Styles"). 33 rows identical to `TabChartToolsFormatNew`.

**GroupAltText** (I: "Alt Text"). 1 rows identical to `TabDrawingToolsFormat`.

**GroupArrangeWith3DEditor** (I: "Arrange With3 DEditor"). 24 rows identical to `TabPictureToolsFormat`.

**GroupSmartArtSize** (I: "Smart Art Size")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ShapeHeight | control |  | Y | ? |  |  |
| 2 | ShapeWidth | control |  | Y | ? |  |  |
| 3 | ObjectSizeAndPositionDialog | launcher |  | Y | ? |  |  |


### Equation (tab set `TabSetEquationTools`)

ID `TabEquationToolsDesign`. KeyTip: not sourced. Source S1, checked 2026-11-07.

Groups in source order: `GroupEquationTools`, `GroupEquationSymbols`, `GroupEquationStructures`.

**GroupEquationTools** (I: "Equation Tools")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | EquationInsertGallery | gallery |  | Y | ? |  |  |
| 2 | InkEquation | button |  | Y | No [D S8]; see C4 |  |  |
| 3 | EquationProfessional | button |  | Y | ? |  |  |
| 4 | EquationLinearFormat | button |  | Y | ? |  |  |
| 5 | EquationNormalText | toggle |  | Y | ? |  |  |
| 6 | EquationOptions | launcher |  | Y | ? |  |  |

**GroupEquationSymbols** (I: "Equation Symbols")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | EquationSymbolsInsertGallery | gallery |  | Y | ? |  |  |

**GroupEquationStructures** (I: "Equation Structures")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | EquationFractionGallery | gallery |  | Y | ? |  |  |
| 2 | EquationScriptGallery | gallery |  | Y | ? |  |  |
| 3 | EquationRadicalGallery | gallery |  | Y | ? |  |  |
| 4 | EquationIntegralGallery | gallery |  | Y | ? |  |  |
| 5 | EquationLargeOperatorGallery | gallery |  | Y | ? |  |  |
| 6 | EquationDelimiterGallery | gallery |  | Y | ? |  |  |
| 7 | EquationFunctionGallery | gallery |  | Y | ? |  |  |
| 8 | EquationAccentGallery | gallery |  | Y | ? |  |  |
| 9 | EquationLimitGallery | gallery |  | Y | ? |  |  |
| 10 | EquationOperatorGallery | gallery |  | Y | ? |  |  |
| 11 | EquationMatrixGallery | gallery |  | Y | ? |  |  |


### Video Format (tab set `TabSetVideoTools`)

ID `TabVideoToolsDesign`. KeyTip: Alt+J, P [D S20]. Source S1, checked 2026-11-07.

Groups in source order: `GroupMediaPreview`, `GroupVideoToolsAdjust`, `GroupVideoStyles`, `GroupAltText`, `GroupArrange`, `GroupVideoSize`.

**GroupMediaPreview** (I: "Media Preview")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | MediaPlayPreview | button |  | Y | ? |  | Ctrl+Spacebar [D S3] |

**GroupVideoToolsAdjust** (I: "Video Tools Adjust")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | VideoCorrectionsMenu | dropdown |  | Y | ? |  |  |
| 2 | PictureBrightnessAndContrastGallery | gallery | VideoCorrectionsMenu | Y | ? |  |  |
| 3 | VideoCorrectionsDialog | button | VideoCorrectionsMenu | Y | ? |  |  |
| 4 | VideoColorMenu | dropdown |  | Y | ? |  |  |
| 5 | PictureRecolorGallery | gallery | VideoColorMenu | Y | ? |  |  |
| 6 | RecolorColorPicker | gallery | VideoColorMenu | Y | ? |  |  |
| 7 | PictureRecolorMoreColorsDialog | button | VideoColorMenu > RecolorColorPicker | Y | ? |  |  |
| 8 | EyedropperRecolor | button | VideoColorMenu > RecolorColorPicker | Y | ? |  |  |
| 9 | VideoColorDialog | button | VideoColorMenu | Y | ? |  |  |
| 10 | VideoPosterFrameGallery | dropdown |  | Y | ? |  |  |
| 11 | VideoPosterFrameCurrent | button | VideoPosterFrameGallery | Y | ? |  |  |
| 12 | VideoPosterFrameFromFile | button | VideoPosterFrameGallery | Y | ? |  |  |
| 13 | VideoPosterFrameReset | button | VideoPosterFrameGallery | Y | ? |  |  |
| 14 | VideoDesignResetSplitButton | split |  | Y | ? |  |  |
| 15 | VideoToolsDesignReset | button | VideoDesignResetSplitButton | Y | ? |  |  |
| 16 | VideoDesignResetAndSize | button | VideoDesignResetSplitButton | Y | ? |  |  |

**GroupVideoStyles** (I: "Video Styles")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | VideoStylesGallery | gallery |  | Y | ? |  |  |
| 2 | VideoShapeGallery | gallery |  | Y | ? |  |  |
| 3 | VideoBorderGallery | gallery |  | Y | ? |  |  |
| 4 | ObjectBorderOutlineColorMoreColorsDialog | button | VideoBorderGallery | Y | ? |  |  |
| 5 | EyedropperOutline | button | VideoBorderGallery | Y | ? |  |  |
| 6 | OutlineWeightGallery | gallery | VideoBorderGallery | Y | ? |  |  |
| 7 | LineStylesDialog | button | VideoBorderGallery > OutlineWeightGallery | Y | ? |  |  |
| 8 | OutlineDashesGallery | gallery | VideoBorderGallery | Y | ? |  |  |
| 9 | LineStylesDialog | button | VideoBorderGallery > OutlineDashesGallery | Y | ? |  |  |
| 10 | VideoEffectsGallery | dropdown |  | Y | ? |  |  |
| 11 | PictureEffectsPresetGallery | gallery | VideoEffectsGallery | Y | ? |  |  |
| 12 | _3DBevelOptionsDialog | button | VideoEffectsGallery > PictureEffectsPresetGallery | Y | ? |  |  |
| 13 | PictureEffectsShadowGallery | gallery | VideoEffectsGallery | Y | ? |  |  |
| 14 | ShadowOptionsDialog | button | VideoEffectsGallery > PictureEffectsShadowGallery | Y | ? |  |  |
| 15 | PictureReflectionGallery | gallery | VideoEffectsGallery | Y | ? |  |  |
| 16 | ReflectionsMoreOptions | button | VideoEffectsGallery > PictureReflectionGallery | Y | ? |  |  |
| 17 | PictureEffectsGlowGallery | gallery | VideoEffectsGallery | Y | ? |  |  |
| 18 | GlowColorPicker | gallery | VideoEffectsGallery > PictureEffectsGlowGallery | Y | ? |  |  |
| 19 | GlowColorMoreColorsDialog | button | VideoEffectsGallery > PictureEffectsGlowGallery | Y | ? |  |  |
| 20 | EyedropperGlow | button | VideoEffectsGallery > PictureEffectsGlowGallery | Y | ? |  |  |
| 21 | GlowsMoreOptions | button | VideoEffectsGallery > PictureEffectsGlowGallery | Y | ? |  |  |
| 22 | PictureEffectsSoftEdgesGallery | gallery | VideoEffectsGallery | Y | ? |  |  |
| 23 | SoftEdgesMoreOptions | button | VideoEffectsGallery > PictureEffectsSoftEdgesGallery | Y | ? |  |  |
| 24 | _3DBevelPictureTopGallery | gallery | VideoEffectsGallery | Y | ? |  |  |
| 25 | _3DBevelOptionsDialog | button | VideoEffectsGallery > _3DBevelPictureTopGallery | Y | ? |  |  |
| 26 | PictureRotationGallery | gallery | VideoEffectsGallery | Y | ? |  |  |
| 27 | _3DRotationOptionsDialog | button | VideoEffectsGallery > PictureRotationGallery | Y | ? |  |  |
| 28 | ObjectFormatDialog | launcher |  | Y | ? |  |  |

**GroupAltText** (I: "Alt Text"). 1 rows identical to `TabDrawingToolsFormat`.

**GroupArrange** (I: "Arrange"). 46 rows identical to `TabTableToolsLayout`.

**GroupVideoSize** (I: "Video Size")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | VideoCrop | toggle |  | Y | ? |  |  |
| 2 | VideoHeight | control |  | Y | ? |  |  |
| 3 | VideoWidth | control |  | Y | ? |  |  |
| 4 | ObjectSizeAndPositionDialog | launcher |  | Y | ? |  |  |


### Playback, video

ID `TabVideoToolsEdit`. KeyTip: Alt+J, N [D S20]. Source S1, checked 2026-11-07.

Groups in source order: `GroupMediaPreview`, `GroupVideoBookmark`, `GroupVideoEditing`, `GroupVideoOptions`, `GroupClosedCaptions`, `GroupSaveMedia`.

**GroupMediaPreview** (I: "Media Preview"). 1 rows identical to `TabVideoToolsDesign`.

**GroupVideoBookmark** (I: "Video Bookmark")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | VideoBookmarkAdd | button |  | Y | No [D S7: trim, bookmark, fade] |  |  |
| 2 | VideoBookmarkRemove | button |  | Y | No [D S7: trim, bookmark, fade] |  |  |

**GroupVideoEditing** (I: "Video Editing")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | VideoToolsTrim | button |  | Y | No [D S7: trim, bookmark, fade] |  |  |
| 2 | FadeDurationLabelVideo | label |  | Y | ? |  |  |
| 3 | VideoFadeInTime | control |  | Y | No [D S7: trim, bookmark, fade] |  |  |
| 4 | VideoFadeOutTime | control |  | Y | No [D S7: trim, bookmark, fade] |  |  |

**GroupVideoOptions** (I: "Video Options")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | VideoVolumeGallery | gallery |  | Y | ? |  |  |
| 2 | VideoStartGallery | dropdown-list |  | Y | ? |  |  |
| 3 | MoviePlayFullScreen | check |  | Y | ? |  |  |
| 4 | MediaClipToolsHideDuringShowVideo | check |  | Y | ? |  |  |
| 5 | MediaClipLoopUntilStopped | check |  | Y | ? |  |  |
| 6 | MovieRewindAfterPlaying | check |  | Y | ? |  |  |

**GroupClosedCaptions** (I: "Closed Captions")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ClosedCaptionSplitDropdown | split |  | Y | ? | Alt+J, N, C [D S18] |  |
| 2 | InsertClosedCaption | button | ClosedCaptionSplitDropdown | Y | ? |  |  |
| 3 | RemoveClosedCaption | button | ClosedCaptionSplitDropdown | Y | ? |  |  |

**GroupSaveMedia** (I: "Save Media")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ObjectSaveMedia | button |  | Y | ? |  |  |


### Audio Format (tab set `TabSetAudioTools`)

ID `TabAudioToolsFormat`. KeyTip: Alt+J, P [D S20 web text; desktop I]. Source S1, checked 2026-11-07.

Groups in source order: `GroupAudioToolsAdjust`, `GroupPictureStyles`, `GroupArrange`, `GroupPictureSize`.

**GroupAudioToolsAdjust** (I: "Audio Tools Adjust")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | PictureBackgroundRemoval | toggle |  | Y | No [D S8] |  |  |
| 2 | PictureCorrectionsMenu | dropdown |  | Y | ? |  |  |
| 3 | PictureSharpenSoftenGallery | gallery | PictureCorrectionsMenu | Y | ? |  |  |
| 4 | PictureBrightnessAndContrastGallery | gallery | PictureCorrectionsMenu | Y | ? |  |  |
| 5 | PictureCorrectionsDialog | button | PictureCorrectionsMenu | Y | ? |  |  |
| 6 | PictureColorMenu | dropdown |  | Y | ? |  |  |
| 7 | PictureSaturationGallery | gallery | PictureColorMenu | Y | ? |  |  |
| 8 | PictureColorTempertatureGallery | gallery | PictureColorMenu | Y | ? |  |  |
| 9 | PictureRecolorGallery | gallery | PictureColorMenu | Y | ? |  |  |
| 10 | RecolorColorPicker | gallery | PictureColorMenu | Y | ? |  |  |
| 11 | PictureRecolorMoreColorsDialog | button | PictureColorMenu > RecolorColorPicker | Y | ? |  |  |
| 12 | EyedropperRecolor | button | PictureColorMenu > RecolorColorPicker | Y | ? |  |  |
| 13 | PictureSetTransparentColor | toggle | PictureColorMenu | Y | ? |  |  |
| 14 | PictureColorDialog | button | PictureColorMenu | Y | ? |  |  |
| 15 | PictureArtisticEffectsGallery | gallery |  | Y | ? |  |  |
| 16 | ArtisticEffectsDialog | button | PictureArtisticEffectsGallery | Y | ? |  |  |
| 17 | PicturesCompress | button |  | Y | No [D S7] |  |  |
| 18 | PictureChange | button |  | Y | Yes, web tab FormatPicture [D S2] |  |  |
| 19 | PictureChangeMenu | dropdown |  | Y | Yes, web tab FormatPicture [D S2] |  |  |
| 20 | PictureChangeFromFile | button | PictureChangeMenu | Y | ? |  |  |
| 21 | PictureChangeFromMobile | button | PictureChangeMenu | Y | ? |  |  |
| 22 | PictureChangeFromM365 | button | PictureChangeMenu | Y | ? |  |  |
| 23 | PictureChangeFromOnline | button | PictureChangeMenu | Y | ? |  |  |
| 24 | ChangeIcon | button | PictureChangeMenu | Y | ? |  |  |
| 25 | PictureChangeFromClipboard | button | PictureChangeMenu | Y | ? |  |  |
| 26 | PictureResetSplitButton | split |  | Y | ? |  |  |
| 27 | PictureReset | button | PictureResetSplitButton | Y | Yes, web tab FormatPicture [D S2] |  |  |
| 28 | PictureResetAndSize | button | PictureResetSplitButton | Y | ? |  |  |

**GroupPictureStyles** (I: "Picture Styles")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | PictureStylesGallery | gallery |  | Y | Yes, web tab FormatPicture [D S2] |  |  |
| 2 | OutlineColorPicker | gallery |  | Y | Yes, web tab Home [D S2] |  |  |
| 3 | ObjectBorderOutlineColorMoreColorsDialog | button | OutlineColorPicker | Y | ? |  |  |
| 4 | EyedropperOutline | button | OutlineColorPicker | Y | ? |  |  |
| 5 | OutlineWeightGallery | gallery | OutlineColorPicker | Y | ? |  |  |
| 6 | LineStylesDialog | button | OutlineColorPicker > OutlineWeightGallery | Y | ? |  |  |
| 7 | GallerySketchyShapes | gallery | OutlineColorPicker | Y | ? |  |  |
| 8 | MoreLines2 | button | OutlineColorPicker > GallerySketchyShapes | Y | ? |  |  |
| 9 | OutlineDashesGallery | gallery | OutlineColorPicker | Y | ? |  |  |
| 10 | LineStylesDialog | button | OutlineColorPicker > OutlineDashesGallery | Y | ? |  |  |
| 11 | PictureEffectsMenu | dropdown |  | Y | ? |  |  |
| 12 | PictureEffectsPresetGallery | gallery | PictureEffectsMenu | Y | ? |  |  |
| 13 | _3DBevelOptionsDialog | button | PictureEffectsMenu > PictureEffectsPresetGallery | Y | ? |  |  |
| 14 | PictureEffectsShadowGallery | gallery | PictureEffectsMenu | Y | ? |  |  |
| 15 | ShadowOptionsDialog | button | PictureEffectsMenu > PictureEffectsShadowGallery | Y | ? |  |  |
| 16 | PictureReflectionGallery | gallery | PictureEffectsMenu | Y | ? |  |  |
| 17 | ReflectionsMoreOptions | button | PictureEffectsMenu > PictureReflectionGallery | Y | ? |  |  |
| 18 | PictureEffectsGlowGallery | gallery | PictureEffectsMenu | Y | ? |  |  |
| 19 | GlowColorPicker | gallery | PictureEffectsMenu > PictureEffectsGlowGallery | Y | ? |  |  |
| 20 | GlowColorMoreColorsDialog | button | PictureEffectsMenu > PictureEffectsGlowGallery | Y | ? |  |  |
| 21 | EyedropperGlow | button | PictureEffectsMenu > PictureEffectsGlowGallery | Y | ? |  |  |
| 22 | GlowsMoreOptions | button | PictureEffectsMenu > PictureEffectsGlowGallery | Y | ? |  |  |
| 23 | PictureEffectsSoftEdgesGallery | gallery | PictureEffectsMenu | Y | ? |  |  |
| 24 | SoftEdgesMoreOptions | button | PictureEffectsMenu > PictureEffectsSoftEdgesGallery | Y | ? |  |  |
| 25 | _3DBevelPictureTopGallery | gallery | PictureEffectsMenu | Y | ? |  |  |
| 26 | _3DBevelOptionsDialog | button | PictureEffectsMenu > _3DBevelPictureTopGallery | Y | ? |  |  |
| 27 | PictureRotationGallery | gallery | PictureEffectsMenu | Y | ? |  |  |
| 28 | _3DRotationOptionsDialog | button | PictureEffectsMenu > PictureRotationGallery | Y | ? |  |  |
| 29 | PicturesConvertToSmartArt | gallery |  | Y | ? |  |  |
| 30 | PictureFormatDialog | launcher |  | Y | ? |  |  |

**GroupArrange** (I: "Arrange"). 46 rows identical to `TabTableToolsLayout`.

**GroupPictureSize** (I: "Picture Size")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | PictureCropTools | split |  | Y | Yes, web tab FormatPicture [D S2] |  |  |
| 2 | PictureCrop | toggle | PictureCropTools | Y | ? |  |  |
| 3 | PictureShapeGallery | gallery | PictureCropTools | Y | ? |  |  |
| 4 | PictureCropAspectRatioMenu | dropdown | PictureCropTools | Y | ? |  |  |
| 5 | PictureCropAspectRatio1To1 | button | PictureCropTools > PictureCropAspectRatioMenu | Y | ? |  |  |
| 6 | PictureCropAspectRatio2To3 | button | PictureCropTools > PictureCropAspectRatioMenu | Y | ? |  |  |
| 7 | PictureCropAspectRatio3To4 | button | PictureCropTools > PictureCropAspectRatioMenu | Y | ? |  |  |
| 8 | PictureCropAspectRatio3To5 | button | PictureCropTools > PictureCropAspectRatioMenu | Y | ? |  |  |
| 9 | PictureCropAspectRatio4To5 | button | PictureCropTools > PictureCropAspectRatioMenu | Y | ? |  |  |
| 10 | PictureCropAspectRatio3To2 | button | PictureCropTools > PictureCropAspectRatioMenu | Y | ? |  |  |
| 11 | PictureCropAspectRatio4To3 | button | PictureCropTools > PictureCropAspectRatioMenu | Y | ? |  |  |
| 12 | PictureCropAspectRatio5To3 | button | PictureCropTools > PictureCropAspectRatioMenu | Y | ? |  |  |
| 13 | PictureCropAspectRatio5To4 | button | PictureCropTools > PictureCropAspectRatioMenu | Y | ? |  |  |
| 14 | PictureCropAspectRatio16To9 | button | PictureCropTools > PictureCropAspectRatioMenu | Y | ? |  |  |
| 15 | PictureCropAspectRatio16To10 | button | PictureCropTools > PictureCropAspectRatioMenu | Y | ? |  |  |
| 16 | PictureFillCrop | button | PictureCropTools | Y | ? |  |  |
| 17 | PictureFitCrop | button | PictureCropTools | Y | ? |  |  |
| 18 | ShapeHeight | control |  | Y | ? |  |  |
| 19 | ShapeWidth | control |  | Y | ? |  |  |
| 20 | ObjectSizeAndPositionDialog | launcher |  | Y | ? |  |  |


### Playback, audio

ID `TabAudioToolsEdit`. KeyTip: Alt+J, N [D S20 web text; desktop I]. Source S1, checked 2026-11-07.

Groups in source order: `GroupMediaPreview`, `GroupAudioBookmark`, `GroupAudioEditing`, `GroupAudioOptions`, `GroupAudioStyles`, `GroupClosedCaptions`, `GroupSaveMedia`.

**GroupMediaPreview** (I: "Media Preview"). 1 rows identical to `TabVideoToolsDesign`.

**GroupAudioBookmark** (I: "Audio Bookmark")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | AudioBookmarkAdd | button |  | Y | No [D S7: trim, bookmark, fade] |  |  |
| 2 | AudioBookmarkRemove | button |  | Y | No [D S7: trim, bookmark, fade] |  |  |

**GroupAudioEditing** (I: "Audio Editing")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | AudioToolsTrim | button |  | Y | No [D S7: trim, bookmark, fade] |  |  |
| 2 | FadeDurationLabelAudio | label |  | Y | ? |  |  |
| 3 | AudioFadeInTime | control |  | Y | No [D S7: trim, bookmark, fade] |  |  |
| 4 | AudioFadeOutTime | control |  | Y | No [D S7: trim, bookmark, fade] |  |  |

**GroupAudioOptions** (I: "Audio Options")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | AudioVolumeGallery | gallery |  | Y | ? |  |  |
| 2 | AudioStartGallery | dropdown-list |  | Y | ? |  |  |
| 3 | MediaClipPlayAcrossSlides | check |  | Y | ? |  |  |
| 4 | MediaClipLoopUntilStopped | check |  | Y | ? |  |  |
| 5 | MediaClipToolsHideDuringShow | check |  | Y | ? |  |  |
| 6 | MovieRewindAfterPlaying | check |  | Y | ? |  |  |

**GroupAudioStyles** (I: "Audio Styles")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | NoAudioStyle | toggle |  | Y | ? |  |  |
| 2 | PlayInBackground | toggle |  | Y | ? |  |  |

**GroupClosedCaptions** (I: "Closed Captions")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ClosedCaptionSplitDropdown | split |  | Y | ? |  |  |
| 2 | InsertClosedCaption | button | ClosedCaptionSplitDropdown | Y | ? |  |  |
| 3 | RemoveClosedCaption | button | ClosedCaptionSplitDropdown | Y | ? |  |  |

**GroupSaveMedia** (I: "Save Media"). 1 rows identical to `TabVideoToolsEdit`.


### Slide Master

ID `TabSlideMaster`. KeyTip: Alt+M [D S17]. Source S1, checked 2026-11-07. Web: editing masters not supported [D S7], so `?` in these rows reads as No.

Groups in source order: `GroupMasterEdit`, `GroupMasterLayout`, `GroupMasterEditTheme`, `GroupBackground`, `GroupSlideSize`, `GroupMasterClose`.

**GroupMasterEdit** (I: "Master Edit")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | SlideNew | button |  | Y | ? |  |  |
| 2 | SlideMasterInsertLayout | button |  | Y | ? |  |  |
| 3 | SlideDelete | button |  | Y | ? |  |  |
| 4 | SlideMasterRenameMaster | button |  | Y | ? |  |  |
| 5 | SlideMasterPreserveMaster | toggle |  | Y | ? |  |  |

**GroupMasterLayout** (I: "Master Layout")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | SlideMasterMasterLayout | button |  | Y | ? |  |  |
| 2 | SlideMasterInsertPlaceholderMenu | split |  | Y | ? |  |  |
| 3 | SlideMasterContentPlaceholderInsert | button | SlideMasterInsertPlaceholderMenu | Y | ? |  |  |
| 4 | SlideMasterVerticalContentPlaceholderInsert | button | SlideMasterInsertPlaceholderMenu | Y | ? |  |  |
| 5 | SlideMasterTextPlaceholderInsert | button | SlideMasterInsertPlaceholderMenu | Y | ? |  |  |
| 6 | SlideMasterVerticalTextPlaceholderInsert | button | SlideMasterInsertPlaceholderMenu | Y | ? |  |  |
| 7 | SlideMasterPicturePlaceholderInsert | button | SlideMasterInsertPlaceholderMenu | Y | ? |  |  |
| 8 | SlideMasterChartPlaceholderInsert | button | SlideMasterInsertPlaceholderMenu | Y | ? |  |  |
| 9 | SlideMasterTablePlaceholderInsert | button | SlideMasterInsertPlaceholderMenu | Y | ? |  |  |
| 10 | SlideMasterDiagramPlaceholderInsert | button | SlideMasterInsertPlaceholderMenu | Y | ? |  |  |
| 11 | SlideMasterMediaPlaceholderInsert | button | SlideMasterInsertPlaceholderMenu | Y | ? |  |  |
| 12 | SlideMasterClipArtPlaceholderInsert | button | SlideMasterInsertPlaceholderMenu | Y | ? |  |  |
| 13 | AddCameoPlaceholder | button | SlideMasterInsertPlaceholderMenu | Y | ? |  |  |
| 14 | SlideMasterShowTitle | check |  | Y | ? |  |  |
| 15 | SlideMasterShowFooters | check |  | Y | ? |  |  |

**GroupMasterEditTheme** (I: "Master Edit Theme")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | SlideThemesGallery | gallery |  | Y | Yes, web tab Design [D S2] |  |  |
| 2 | ThemeSearchOfficeOnlinePowerPoint | button | SlideThemesGallery | Y | ? |  |  |
| 3 | ThemeBrowseForThemesPowerPoint | button | SlideThemesGallery | Y | ? |  |  |
| 4 | ThemeSaveCurrentPowerPoint | button | SlideThemesGallery | Y | ? |  |  |

**GroupBackground** (I: "Background")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ThemeColorsGallery | gallery |  | Y | ? |  |  |
| 2 | ThemeColorsCreateNew | button | ThemeColorsGallery | Y | ? | Alt+M, T, C, C (announced color scheme; mapping I) [D S18] |  |
| 3 | ThemeColorsReset | button | ThemeColorsGallery | Y | ? |  |  |
| 4 | ThemeFontsGallery | gallery |  | Y | ? |  |  |
| 5 | ThemeFontsCreateNew | button | ThemeFontsGallery | Y | ? |  |  |
| 6 | ThemeEffectsGallery | gallery |  | Y | ? |  |  |
| 7 | SlideBackgroundStylesGallery | gallery |  | Y | ? |  |  |
| 8 | SlideBackgroundFormatDialog | button | SlideBackgroundStylesGallery | Y | Yes, web tab Design [D S2] |  |  |
| 9 | SlideBackgroundReset | button | SlideBackgroundStylesGallery | Y | ? |  |  |
| 10 | SlideBackgroundHideGraphics | check |  | Y | ? |  |  |
| 11 | SlideBackgroundFormatDialog | launcher |  | Y | Yes, web tab Design [D S2] |  |  |

**GroupSlideSize** (I: "Slide Size")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | PowerPointPageSetup | gallery |  | Y | ? |  |  |
| 2 | CustomSlideSize | button | PowerPointPageSetup | Y | Yes, web tab Design [D S2] |  |  |

**GroupMasterClose** (I: "Master Close")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | MasterViewClose | button |  | Y | ? | Alt+M, C [D S17] |  |


### Notes Master

ID `TabNotesMaster`. KeyTip: not sourced. Source S1, checked 2026-11-07. Web: Master views desktop only [D S8], so `?` in these rows reads as No.

Groups in source order: `GroupPageSetupNotesMaster`, `GroupPlaceholdersNotesMaster`, `GroupMasterEditTheme`, `GroupBackground`, `GroupMasterClose`.

**GroupPageSetupNotesMaster** (I: "Page Setup Notes Master")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | MasterNotesPageOrientation | gallery |  | Y | ? |  |  |
| 2 | PowerPointPageSetup | gallery |  | Y | ? |  |  |
| 3 | CustomSlideSize | button | PowerPointPageSetup | Y | Yes, web tab Design [D S2] |  |  |

**GroupPlaceholdersNotesMaster** (I: "Placeholders Notes Master")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | MasterShowHeader | check |  | Y | ? |  |  |
| 2 | MasterShowSlideImage | check |  | Y | ? |  |  |
| 3 | MasterShowFooter | check |  | Y | ? |  |  |
| 4 | MasterShowDate | check |  | Y | ? |  |  |
| 5 | MasterShowBody | check |  | Y | ? |  |  |
| 6 | MasterShowPageNumber | check |  | Y | ? |  |  |

**GroupMasterEditTheme** (I: "Master Edit Theme"). 4 rows identical to `TabSlideMaster`.

**GroupBackground** (I: "Background")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ThemeColorsGallery | gallery |  | Y | ? |  |  |
| 2 | ThemeColorsCreateNew | button | ThemeColorsGallery | Y | ? |  |  |
| 3 | ThemeColorsReset | button | ThemeColorsGallery | Y | ? |  |  |
| 4 | ThemeFontsGallery | gallery |  | Y | ? |  |  |
| 5 | ThemeFontsCreateNew | button | ThemeFontsGallery | Y | ? |  |  |
| 6 | ThemeEffectsGallery | gallery |  | Y | ? |  |  |
| 7 | SlideBackgroundStylesGallery | gallery |  | Y | ? |  |  |
| 8 | SlideBackgroundFormatDialog | button | SlideBackgroundStylesGallery | Y | Yes, web tab Design [D S2] |  |  |
| 9 | SlideBackgroundReset | button | SlideBackgroundStylesGallery | Y | ? |  |  |
| 10 | SlideBackgroundHideGraphics | check |  | Y | ? |  |  |
| 11 | SlideBackgroundFormatDialog | launcher |  | Y | Yes, web tab Design [D S2] |  |  |

**GroupMasterClose** (I: "Master Close")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | MasterViewClose | button |  | Y | ? |  |  |


### Handout Master

ID `TabHandoutMaster`. KeyTip: not sourced. Source S1, checked 2026-11-07. Web: Master views desktop only [D S8], so `?` in these rows reads as No.

Groups in source order: `GroupPageSetupHandoutMaster`, `GroupPlaceholdersHandoutMaster`, `GroupMasterEditTheme`, `GroupBackground`, `GroupMasterClose`.

**GroupPageSetupHandoutMaster** (I: "Page Setup Handout Master")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | HandoutOrientation | gallery |  | Y | ? |  |  |
| 2 | PowerPointPageSetup | gallery |  | Y | ? |  |  |
| 3 | CustomSlideSize | button | PowerPointPageSetup | Y | Yes, web tab Design [D S2] |  |  |
| 4 | SlidesPerPageGallery | dropdown |  | Y | ? |  |  |
| 5 | SlidesPerPage1Slide | toggle | SlidesPerPageGallery | Y | ? |  |  |
| 6 | SlidesPerPage2Slides | toggle | SlidesPerPageGallery | Y | ? |  |  |
| 7 | SlidesPerPage3Slides | toggle | SlidesPerPageGallery | Y | ? |  |  |
| 8 | SlidesPerPage4Slides | toggle | SlidesPerPageGallery | Y | ? |  |  |
| 9 | SlidesPerPage6Slides | toggle | SlidesPerPageGallery | Y | ? |  |  |
| 10 | SlidesPerPage9Slides | toggle | SlidesPerPageGallery | Y | ? |  |  |
| 11 | SlidesPerPageSlideOutline | toggle | SlidesPerPageGallery | Y | ? |  |  |

**GroupPlaceholdersHandoutMaster** (I: "Placeholders Handout Master")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | MasterShowHeader | check |  | Y | ? |  |  |
| 2 | MasterShowFooter | check |  | Y | ? |  |  |
| 3 | MasterShowDate | check |  | Y | ? |  |  |
| 4 | MasterShowPageNumber | check |  | Y | ? |  |  |

**GroupMasterEditTheme** (I: "Master Edit Theme"). 4 rows identical to `TabSlideMaster`.

**GroupBackground** (I: "Background"). 11 rows identical to `TabNotesMaster`.

**GroupMasterClose** (I: "Master Close"). 1 rows identical to `TabNotesMaster`.


### Grayscale

ID `TabGrayscale`. KeyTip: not sourced. Source S1, checked 2026-11-07.

Groups in source order: `GroupColorModeSetting`, `GroupColorModeClose`.

**GroupColorModeSetting** (I: "Color Mode Setting")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | BlackAndWhiteAutomatic | toggle |  | Y | ? |  |  |
| 2 | BlackAndWhiteGrayscale | toggle |  | Y | ? |  |  |
| 3 | BlackAndWhiteLightGrayscale | toggle |  | Y | ? |  |  |
| 4 | BlackAndWhiteInverseGrayscale | toggle |  | Y | ? |  |  |
| 5 | BlackAndWhiteGrayWithWhiteFill | toggle |  | Y | ? |  |  |
| 6 | BlackAndWhiteBlackWithGrayscaleFill | toggle |  | Y | ? |  |  |
| 7 | BlackAndWhiteBlackWithWhiteFill | toggle |  | Y | ? |  |  |
| 8 | BlackAndWhiteBlack | toggle |  | Y | ? |  |  |
| 9 | BlackAndWhiteWhite | toggle |  | Y | ? |  |  |
| 10 | BlackAndWhiteDontShow | toggle |  | Y | ? |  |  |

**GroupColorModeClose** (I: "Color Mode Close")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ViewBackToColorView | toggle |  | Y | ? |  |  |


### Ink tools, Pens (tab set `TabSetInkTools`)

ID `TabInkToolsPens`. KeyTip: not sourced. Source S1, checked 2026-11-07. Source pairs this with the older ink flow; Draw is the current surface [I].

Groups in source order: `GroupPensWrite`, `GroupPens`, `GroupDiagramming`, `GroupInkSelect`, `GroupInkFormat`, `GroupInkPens`, `GroupInkClose`.

**GroupPensWrite** (I: "Pens Write")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | InkBallpointPen | toggle |  | Y | ? |  |  |
| 2 | InkHighlighter | toggle |  | Y | ? |  |  |
| 3 | InkEraserMenu | split |  | Y | ? |  |  |
| 4 | CurrentEraser | toggle | InkEraserMenu | Y | ? |  |  |
| 5 | InkEraser | toggle | InkEraserMenu | Y | ? |  |  |
| 6 | PointEraserSmall | toggle | InkEraserMenu | Y | ? |  |  |
| 7 | PointEraserMedium | toggle | InkEraserMenu | Y | ? |  |  |
| 8 | SegmentEraser | toggle | InkEraserMenu | Y | ? |  |  |
| 9 | LassoSelect | toggle |  | Y | ? |  |  |
| 10 | ObjectsSelect | toggle |  | Y | ? |  |  |

**GroupPens** (I: "Pens")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | PensGallery | gallery |  | Y | ? |  |  |
| 2 | PenAddFavorite | button | PensGallery | Y | ? |  |  |
| 3 | InkColorPicker | gallery |  | Y | ? |  |  |
| 4 | InkColorMoreColorsDialog | button | InkColorPicker | Y | ? |  |  |
| 5 | EyedropperOutline | button | InkColorPicker | Y | ? |  |  |
| 6 | LineThickness | gallery |  | Y | ? |  |  |
| 7 | LineStylesDialog | button | LineThickness | Y | ? |  |  |

**GroupDiagramming** (I: "Diagramming")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | DiagrammingOnline | toggle |  | Y | ? |  |  |

**GroupInkSelect** (I: "Ink Select")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ObjectsSelect | toggle |  | Y | ? |  |  |

**GroupInkFormat** (I: "Ink Format")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ObjectsSelect | toggle |  | Y | ? |  |  |

**GroupInkPens** (I: "Ink Pens")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | InkFeltTipPen | toggle |  | Y | ? |  |  |

**GroupInkClose** (I: "Ink Close"). 1 rows identical to `TabDrawInk`.


## Other tabs in the source, not requested by name


### Developer (off by default [O O4])

ID `TabDeveloper`. KeyTip: not sourced. Source S1, checked 2026-11-07.

Groups in source order: `GroupCode`, `GroupAddins`, `GroupControls`.

**GroupCode** (I: "Code")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | VisualBasic | button |  | Y | ? |  |  |
| 2 | MacroPlay | button |  | Y | ? |  |  |
| 3 | MacroSecurity | button |  | Y | ? |  |  |

**GroupAddins** (I: "Addins")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | OfficeExtensionsGallery3 | button |  | Y | ? |  |  |
| 2 | AddInManager | button |  | Y | ? |  |  |
| 3 | ComAddInsDialog | button |  | Y | ? |  |  |

**GroupControls** (I: "Controls")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ActiveXLabel | button |  | Y | ? |  |  |
| 2 | ActiveXTextBox | button |  | Y | ? |  |  |
| 3 | ActiveXSpinButton | button |  | Y | ? |  |  |
| 4 | ActiveXButton | button |  | Y | ? |  |  |
| 5 | ActiveXImage | button |  | Y | ? |  |  |
| 6 | ActiveXScrollBar | button |  | Y | ? |  |  |
| 7 | ActiveXCheckBox | button |  | Y | ? |  |  |
| 8 | ActiveXRadioButton | button |  | Y | ? |  |  |
| 9 | ActiveXComboBox | button |  | Y | ? |  |  |
| 10 | ActiveXListBox | button |  | Y | ? |  |  |
| 11 | ActiveXToggleButton | button |  | Y | ? |  |  |
| 12 | MoreControlsDialog | button |  | Y | ? |  |  |
| 13 | ControlProperties | button |  | Y | ? |  |  |
| 14 | ViewVisualBasicCode | button |  | Y | ? |  |  |


### Add-Ins

ID `TabAddIns`. KeyTip: not sourced. Source S1, checked 2026-11-07.

Groups in source order: `GroupAddInsMenuCommands`, `GroupAddInsToolbarCommands`, `GroupAddInsCustomToolbars`.

**GroupAddInsMenuCommands** (I: "Add Ins Menu Commands"). No child rows in source.

**GroupAddInsToolbarCommands** (I: "Add Ins Toolbar Commands"). No child rows in source.

**GroupAddInsCustomToolbars** (I: "Add Ins Custom Toolbars"). No child rows in source.


### Home while in Slide Master view

ID `TabSlideMasterHome`. KeyTip: not sourced. Source S1, checked 2026-11-07. Source lists this distinct Home variant.

Groups in source order: `GroupClipboard`, `GroupMasterSlides`, `GroupFont`, `GroupParagraph`, `GroupDrawing`, `GroupEditing`.

**GroupClipboard** (I: "Clipboard")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | PasteMenu | split |  | Y | ? |  | Ctrl+V [D S3]; web Ctrl+V [D S3 Web] |
| 2 | Paste | button | PasteMenu | Y | Yes, web tab Home [D S2] |  | Ctrl+V [D S3]; web Ctrl+V [D S3 Web] |
| 3 | PasteGallery | gallery | PasteMenu | Y | ? |  |  |
| 4 | PasteSpecialDialog | button | PasteMenu > PasteGallery | Y | ? |  | Ctrl+Alt+V [D S3] |
| 5 | Cut | button |  | Y | Yes, web tab Home [D S2] |  | Ctrl+X [D S3]; web Ctrl+X [D S3 Web] |
| 6 | CopySplitButton | split |  | Y | ? |  | Ctrl+C [D S3]; web Ctrl+C [D S3 Web] |
| 7 | Copy | button | CopySplitButton | Y | Yes, web tab Home [D S2] |  | Ctrl+C [D S3]; web Ctrl+C [D S3 Web] |
| 8 | PasteDuplicate | button | CopySplitButton | Y | Yes, web tab Home [D S2] |  | Ctrl+D [D S3]; web Ctrl+D [D S3 Web] |
| 9 | FormatPainter | control |  | Y | Yes, web tab Home [D S2] |  | Ctrl+Shift+C / Ctrl+Shift+V [D S3] |
| 10 | ShowClipboard | launcher |  | Y | Yes, web tab Home [D S2] |  |  |

**GroupMasterSlides** (I: "Master Slides")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | SlideNewGallery | gallery |  | Y | Yes, web tab Home [D S2] |  | Ctrl+M [D S3]; web Ctrl+M [D S3 Web] |
| 2 | DuplicateSelectedSlides | button | SlideNewGallery | Y | ? |  | Ctrl+Shift+D [D S3] |
| 3 | SlidesFromOutline | button | SlideNewGallery | Y | ? |  |  |
| 4 | SlidesReuseSlides | toggle | SlideNewGallery | Y | ? |  |  |
| 5 | ReuseSlides | toggle | SlideNewGallery | Y | Yes, web tab Home [D S2] |  |  |
| 6 | PasteAsSlide | toggle | SlideNewGallery | Y | ? |  |  |
| 7 | SlideLayoutGallery | gallery |  | Y | Yes, web tab Home [D S2] |  |  |
| 8 | SlideReset | button |  | Y | ? |  |  |
| 9 | SlideDelete | button |  | Y | ? |  |  |

**GroupFont** (I: "Font")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | Font | combo |  | Y | Yes, web tab Home [D S2] |  |  |
| 2 | FontSize | combo |  | Y | Yes, web tab Home [D S2] |  |  |
| 3 | FontSizeIncrease | button |  | Y | Yes, web tab Home [D S2] |  | Ctrl+Shift+> [D S3]; web Ctrl+Shift+> [D S3 Web] |
| 4 | FontSizeDecrease | button |  | Y | ? |  | Ctrl+Shift+< [D S3]; web Ctrl+Shift+< [D S3 Web] |
| 5 | ClearFormatting | button |  | Y | Yes, web tab Home [D S2] |  | Ctrl+Spacebar [D S3]; web Ctrl+Spacebar [D S3 Web] |
| 6 | Bold | toggle |  | Y | Yes, web tab Home [D S2] |  | Ctrl+B [D S3]; web Ctrl+B [D S3 Web] |
| 7 | Italic | toggle |  | Y | Yes, web tab Home [D S2] |  | Ctrl+I [D S3]; web Ctrl+I [D S3 Web] |
| 8 | Underline | toggle |  | Y | Yes, web tab Home [D S2] |  | Ctrl+U [D S3]; web Ctrl+U [D S3 Web] |
| 9 | Shadow | toggle |  | Y | ? |  |  |
| 10 | Strikethrough | toggle |  | Y | Yes, web tab Home [D S2] |  |  |
| 11 | CharacterSpacingGallery | gallery |  | Y | ? |  |  |
| 12 | ChangeCaseGallery | gallery |  | Y | Yes, web tab Home [D S2] |  | Shift+F3 [D S3] |
| 13 | TextHighlightColorPickerLicensed | gallery |  | Y | ? |  |  |
| 14 | HighlightingStop | button | TextHighlightColorPickerLicensed | Y | Yes, web tab Home [D S2] |  |  |
| 15 | FontColorPicker | gallery |  | Y | Yes, web tab Home [D S2] |  |  |
| 16 | FontColorMoreColorsDialogPowerPoint | button | FontColorPicker | Y | ? |  |  |
| 17 | EyedropperFillText | button | FontColorPicker | Y | ? |  |  |
| 18 | FontDialogPowerPoint | launcher |  | Y | ? |  | Ctrl+T or Ctrl+Shift+F [D S3] |

**GroupParagraph** (I: "Paragraph")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | BulletsGallery | gallery |  | Y | Yes, web tab Home [D S2] |  |  |
| 2 | BulletsAndNumberingBulletsDialog | button | BulletsGallery | Y | ? |  |  |
| 3 | NumberingGallery | gallery |  | Y | Yes, web tab Home [D S2] |  |  |
| 4 | BulletsAndNumberingNumberingDialog | button | NumberingGallery | Y | ? |  |  |
| 5 | IndentDecrease | button |  | Y | Yes, web tab Home [D S2] |  |  |
| 6 | IndentIncrease | button |  | Y | Yes, web tab Home [D S2] |  |  |
| 7 | LineSpacingGalleryPowerPoint | gallery |  | Y | Yes, web tab Home [D S2] |  |  |
| 8 | AlignLeft | toggle |  | Y | Yes, web tab Home [D S2] |  | Ctrl+L [D S3]; web Ctrl+L [D S3 Web] |
| 9 | AlignCenter | toggle |  | Y | Yes, web tab Home [D S2] |  | Ctrl+E [D S3]; web Ctrl+E [D S3 Web] |
| 10 | AlignRight | toggle |  | Y | Yes, web tab Home [D S2] |  | Ctrl+R [D S3]; web Ctrl+R [D S3 Web] |
| 11 | AlignJustify | toggle |  | Y | Yes, web tab Home [D S2] |  | Ctrl+J [D S3]; web Ctrl+J [D S3 Web] |
| 12 | AlignJustifyMenu | dropdown |  | Y | ? |  |  |
| 13 | AlignJustifyWithMixedLanguages | toggle | AlignJustifyMenu | Y | ? |  |  |
| 14 | AlignJustifyLow | toggle | AlignJustifyMenu | Y | ? |  |  |
| 15 | ParagraphDistributed | toggle |  | Y | ? |  |  |
| 16 | AlignJustifyThai | toggle |  | Y | ? |  |  |
| 17 | TextDirectionLeftToRight | toggle |  | Y | Yes, web tab Home [D S2] |  |  |
| 18 | TextDirectionRightToLeft | toggle |  | Y | Yes, web tab Home [D S2] |  |  |
| 19 | TableColumnsGallery | gallery |  | Y | ? |  |  |
| 20 | ParagraphMoreColumnsDialog | button | TableColumnsGallery | Y | ? |  |  |
| 21 | TextDirectionGallery | gallery |  | Y | ? |  |  |
| 22 | TextDirectionMoreOptionsDialog | button | TextDirectionGallery | Y | ? |  |  |
| 23 | TextAlignGallery | gallery |  | Y | Yes, web tab Home [D S2] |  |  |
| 24 | TextAlignMoreOptionsDialog | button | TextAlignGallery | Y | ? |  |  |
| 25 | ConvertToSmartArt | gallery |  | Y | ? |  |  |
| 26 | ConvertToSmartArtMoreSmartArtGraphicsDialog | button | ConvertToSmartArt | Y | ? |  |  |
| 27 | PowerPointParagraphDialog | launcher |  | Y | Yes, web tab Home [D S2] |  |  |

**GroupDrawing** (I: "Drawing")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | GalleryAllShapesAndTextboxes | gallery |  | Y | ? |  |  |
| 2 | ObjectsArrangeMenu | dropdown |  | Y | ? |  |  |
| 3 | ObjectBringToFront | button | ObjectsArrangeMenu | Y | Yes, web tab FormatShape [D S2] |  | web Ctrl+Shift+] [D S3 Web] |
| 4 | ObjectSendToBack | button | ObjectsArrangeMenu | Y | Yes, web tab FormatShape [D S2] |  | web Ctrl+Shift+[ [D S3 Web] |
| 5 | ObjectBringForward | button | ObjectsArrangeMenu | Y | Yes, web tab FormatShape [D S2] |  | Ctrl+Shift+] [D S3]; web Ctrl+] [D S3 Web] |
| 6 | ObjectSendBackward | button | ObjectsArrangeMenu | Y | Yes, web tab FormatShape [D S2] |  | Ctrl+Shift+[ [D S3]; web Ctrl+[ [D S3 Web] |
| 7 | ObjectsGroup | button | ObjectsArrangeMenu | Y | Yes, web tab FormatShape [D S2] |  | Ctrl+G [D S3] |
| 8 | ObjectsUngroup | button | ObjectsArrangeMenu | Y | Yes, web tab FormatShape [D S2] |  | Ctrl+Shift+G [D S3] |
| 9 | ObjectsRegroup | button | ObjectsArrangeMenu | Y | ? |  | Ctrl+Shift+J [D S3] |
| 10 | ObjectAlignMenu | dropdown | ObjectsArrangeMenu | Y | Yes, web tab FormatShape [D S2] |  |  |
| 11 | ObjectsAlignLeftSmart | button | ObjectsArrangeMenu > ObjectAlignMenu | Y | Yes, web tab FormatShape [D S2] |  |  |
| 12 | ObjectsAlignCenterHorizontalSmart | button | ObjectsArrangeMenu > ObjectAlignMenu | Y | Yes, web tab FormatShape [D S2] |  |  |
| 13 | ObjectsAlignRightSmart | button | ObjectsArrangeMenu > ObjectAlignMenu | Y | Yes, web tab FormatShape [D S2] |  |  |
| 14 | ObjectsAlignTopSmart | button | ObjectsArrangeMenu > ObjectAlignMenu | Y | Yes, web tab FormatShape [D S2] |  |  |
| 15 | ObjectsAlignMiddleVerticalSmart | button | ObjectsArrangeMenu > ObjectAlignMenu | Y | Yes, web tab FormatShape [D S2] |  |  |
| 16 | ObjectsAlignBottomSmart | button | ObjectsArrangeMenu > ObjectAlignMenu | Y | Yes, web tab FormatShape [D S2] |  |  |
| 17 | AlignDistributeHorizontally | button | ObjectsArrangeMenu > ObjectAlignMenu | Y | Yes, web tab Home [D S2] |  |  |
| 18 | AlignDistributeVertically | button | ObjectsArrangeMenu > ObjectAlignMenu | Y | Yes, web tab Home [D S2] |  |  |
| 19 | ObjectsAlignRelativeToContainerSmart | toggle | ObjectsArrangeMenu > ObjectAlignMenu | Y | Yes, web tab FormatShape [D S2] |  |  |
| 20 | ObjectsAlignSelectedSmart | toggle | ObjectsArrangeMenu > ObjectAlignMenu | Y | ? |  |  |
| 21 | ObjectRotateGallery | gallery | ObjectsArrangeMenu | Y | Yes, web tab FormatShape [D S2] |  |  |
| 22 | ObjectRotationOptionsDialog | button | ObjectsArrangeMenu > ObjectRotateGallery | Y | Yes, web tab FormatShape [D S2] |  |  |
| 23 | SelectionPane | toggle | ObjectsArrangeMenu | Y | Yes, web tab Home [D S2] |  | Alt+F10 [D S3] |
| 24 | ShapeQuickStylesHome | gallery |  | Y | ? |  |  |
| 25 | ShapeStylesOtherThemeFillsGallery | gallery | ShapeQuickStylesHome | Y | ? |  |  |
| 26 | ShapeFillColorPicker | gallery |  | Y | Yes, web tab FormatShape [D S2] |  |  |
| 27 | ObjectFillMoreColorsDialog | button | ShapeFillColorPicker | Y | ? |  |  |
| 28 | EyedropperFill | button | ShapeFillColorPicker | Y | ? |  |  |
| 29 | ObjectPictureFill | button | ShapeFillColorPicker | Y | ? |  |  |
| 30 | GradientGallery | gallery | ShapeFillColorPicker | Y | ? |  |  |
| 31 | ShapeFillMoreGradientsDialog | button | ShapeFillColorPicker > GradientGallery | Y | ? |  |  |
| 32 | ShapeFillTextureGallery | gallery | ShapeFillColorPicker | Y | ? |  |  |
| 33 | MoreTextureOptions | button | ShapeFillColorPicker > ShapeFillTextureGallery | Y | ? |  |  |
| 34 | ShapeOutlineColorPicker | gallery |  | Y | Yes, web tab FormatShape [D S2] |  |  |
| 35 | ObjectBorderOutlineColorMoreColorsDialog | button | ShapeOutlineColorPicker | Y | ? |  |  |
| 36 | EyedropperOutline | button | ShapeOutlineColorPicker | Y | ? |  |  |
| 37 | OutlineWeightGallery | gallery | ShapeOutlineColorPicker | Y | ? |  |  |
| 38 | LineStylesDialog | button | ShapeOutlineColorPicker > OutlineWeightGallery | Y | ? |  |  |
| 39 | GallerySketchyShapes | gallery | ShapeOutlineColorPicker | Y | ? |  |  |
| 40 | MoreLines2 | button | ShapeOutlineColorPicker > GallerySketchyShapes | Y | ? |  |  |
| 41 | OutlineDashesGallery | gallery | ShapeOutlineColorPicker | Y | ? |  |  |
| 42 | LineStylesDialog | button | ShapeOutlineColorPicker > OutlineDashesGallery | Y | ? |  |  |
| 43 | ArrowStyleGallery | gallery | ShapeOutlineColorPicker | Y | ? |  |  |
| 44 | ArrowsMore | button | ShapeOutlineColorPicker > ArrowStyleGallery | Y | ? |  |  |
| 45 | ShapeEffectsMenu | dropdown |  | Y | ? |  |  |
| 46 | ObjectEffectPresetGallery | gallery | ShapeEffectsMenu | Y | ? |  |  |
| 47 | _3DBevelOptionsDialog | button | ShapeEffectsMenu > ObjectEffectPresetGallery | Y | ? |  |  |
| 48 | ObjectEffectShadowGallery | gallery | ShapeEffectsMenu | Y | ? |  |  |
| 49 | ShadowOptionsDialog | button | ShapeEffectsMenu > ObjectEffectShadowGallery | Y | ? |  |  |
| 50 | ReflectionGallery | gallery | ShapeEffectsMenu | Y | ? |  |  |
| 51 | ReflectionsMoreOptions | button | ShapeEffectsMenu > ReflectionGallery | Y | ? |  |  |
| 52 | ObjectEffectGlowGallery | gallery | ShapeEffectsMenu | Y | ? |  |  |
| 53 | GlowColorPicker | gallery | ShapeEffectsMenu > ObjectEffectGlowGallery | Y | ? |  |  |
| 54 | GlowColorMoreColorsDialog | button | ShapeEffectsMenu > ObjectEffectGlowGallery | Y | ? |  |  |
| 55 | EyedropperGlow | button | ShapeEffectsMenu > ObjectEffectGlowGallery | Y | ? |  |  |
| 56 | GlowsMoreOptions | button | ShapeEffectsMenu > ObjectEffectGlowGallery | Y | ? |  |  |
| 57 | ObjectEffectSoftEdgesGallery | gallery | ShapeEffectsMenu | Y | ? |  |  |
| 58 | SoftEdgesMoreOptions | button | ShapeEffectsMenu > ObjectEffectSoftEdgesGallery | Y | ? |  |  |
| 59 | BevelShapeGallery | gallery | ShapeEffectsMenu | Y | ? |  |  |
| 60 | _3DBevelOptionsDialog | button | ShapeEffectsMenu > BevelShapeGallery | Y | ? |  |  |
| 61 | _3DRotationGallery | gallery | ShapeEffectsMenu | Y | ? |  |  |
| 62 | _3DRotationOptionsDialog | button | ShapeEffectsMenu > _3DRotationGallery | Y | ? |  |  |
| 63 | ObjectFormatDialog | launcher |  | Y | ? |  |  |

**GroupEditing** (I: "Editing")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | FindDialog | button |  | Y | Yes, web tab Home [D S2] |  | Ctrl+F [D S3]; web Ctrl+F [D S3 Web] |
| 2 | ReplaceMenu | split |  | Y | No [D S8]; see C5 |  | Ctrl+H [D S3] |
| 3 | ReplaceDialog | button | ReplaceMenu | Y | No [D S8]; see C5 |  | Ctrl+H [D S3] |
| 4 | FontsReplaceFonts | button | ReplaceMenu | Y | No [D S8]; see C5 |  |  |
| 5 | SelectMenu | dropdown |  | Y | ? |  |  |
| 6 | SelectAll | button | SelectMenu | Y | ? |  | Ctrl+A [D S3]; web Ctrl+A [D S3 Web] |
| 7 | ObjectsSelect | toggle | SelectMenu | Y | ? |  |  |
| 8 | SelectionPane | toggle | SelectMenu | Y | Yes, web tab Home [D S2] |  | Alt+F10 [D S3] |


### Black And White

ID `TabBlackAndWhite`. KeyTip: not sourced. Source S1, checked 2026-11-07.

Groups in source order: `GroupColorModeSetting`, `GroupColorModeClose`.

**GroupColorModeSetting** (I: "Color Mode Setting"). 10 rows identical to `TabGrayscale`.

**GroupColorModeClose** (I: "Color Mode Close"). 1 rows identical to `TabGrayscale`.


### Print Preview

ID `TabPrintPreview`. KeyTip: not sourced. Source S1, checked 2026-11-07.

Groups in source order: `GroupPrintPreviewPrint`, `GroupPrintPreviewPageSetup`, `GroupZoom`, `GroupPrintPreviewPreview`.

**GroupPrintPreviewPrint** (I: "Print Preview Print")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | PrintPreviewAndPrint | button |  | Y | ? |  | Ctrl+P [D S3]; web Ctrl+P [D S3 Web] |
| 2 | PrintOptionsMenu | dropdown |  | Y | ? |  |  |

**GroupPrintPreviewPageSetup** (I: "Print Preview Page Setup")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | PrintWhatLabel | label |  | Y | ? |  |  |
| 2 | PrintWhat | dropdown-list |  | Y | ? |  |  |
| 3 | PageOrientationGallery | dropdown |  | Y | ? |  |  |
| 4 | PageOrientationPortrait | toggle | PageOrientationGallery | Y | ? |  |  |
| 5 | PageOrientationLandscape | toggle | PageOrientationGallery | Y | ? |  |  |

**GroupZoom** (I: "Zoom")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ZoomDialog | button |  | Y | Yes, web tab View [D S2] |  |  |
| 2 | ZoomFitToWindow | button |  | Y | Yes, web tab View [D S2] |  | Ctrl+Alt+O [D S3] |

**GroupPrintPreviewPreview** (I: "Print Preview Preview")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | PageNext | button |  | Y | ? |  |  |
| 2 | PagePrevious | button |  | Y | ? |  |  |
| 3 | PrintPreviewClose | button |  | Y | ? |  |  |


### Merge

ID `TabMerge`. KeyTip: not sourced. Source S1, checked 2026-11-07. Compare and Merge disabled in version 2502 [D S29]; source predates that.

Groups in source order: `GroupResolutions`, `GroupShowHideChanges`, `GroupAnimationPreview`, `GroupMergeViewClose`.

**GroupResolutions** (I: "Resolutions")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ReviewRevisionAcceptMenu | split |  | Y | ? |  |  |
| 2 | ReviewRevisionAccept | button | ReviewRevisionAcceptMenu | Y | ? |  |  |
| 3 | ReviewSlideRevisionAccept | button | ReviewRevisionAcceptMenu | Y | ? |  |  |
| 4 | ReviewPresentationRevisionAccept | button | ReviewRevisionAcceptMenu | Y | ? |  |  |
| 5 | ReviewRevisionRejectMenu | split |  | Y | ? |  |  |
| 6 | ReviewRevisionReject | button | ReviewRevisionRejectMenu | Y | ? |  |  |
| 7 | ReviewSlideRevisionReject | button | ReviewRevisionRejectMenu | Y | ? |  |  |
| 8 | ReviewPresentationRevisionReject | button | ReviewRevisionRejectMenu | Y | ? |  |  |
| 9 | ReviewPreviousChange | button |  | Y | ? |  |  |
| 10 | ReviewNextChange | button |  | Y | ? |  |  |

**GroupShowHideChanges** (I: "Show Hide Changes")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ReviewShowChanges | dropdown |  | Y | ? |  |  |
| 2 | ReviewShowOnlyConflicts | toggle | ReviewShowChanges | Y | ? |  |  |
| 3 | ReviewShowAllChanges | toggle | ReviewShowChanges | Y | ? |  |  |
| 4 | ReviewReviewingPanePowerPoint | toggle |  | Y | ? |  |  |

**GroupAnimationPreview** (I: "Animation Preview")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | AnimationPreview | button |  | Y | ? |  |  |

**GroupMergeViewClose** (I: "Merge View Close")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | MergeViewClose | button |  | Y | ? |  |  |


### Broadcast Presentation

ID `TabBroadcastPresentation`. KeyTip: not sourced. Source S1, checked 2026-11-07.

Groups in source order: `GroupBroadcastResume`, `GroupMonitors`, `GroupBroadcastBroadcast`.

**GroupBroadcastResume** (I: "Broadcast Resume")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | BroadcastFromBeginning | button |  | Y | ? |  |  |
| 2 | BroadcastFromCurrent | button |  | Y | ? |  |  |

**GroupMonitors** (I: "Monitors"). 2 rows identical to `TabSlideShow`.

**GroupBroadcastBroadcast** (I: "Broadcast Broadcast")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | BroadcastNotesSplitButton | split |  | Y | ? |  |  |
| 2 | BroadcastNotes | button | BroadcastNotesSplitButton | Y | ? |  |  |
| 3 | OpenMeetingNotes | button | BroadcastNotesSplitButton | Y | ? |  |  |
| 4 | ShareNotesWithMeeting | button | BroadcastNotesSplitButton | Y | ? |  |  |
| 5 | BroadcastSendInvitation | button |  | Y | ? |  |  |
| 6 | BroadcastEnd | button |  | Y | ? |  |  |


### Background Removal

ID `TabBackgroundRemoval`. KeyTip: not sourced. Source S1, checked 2026-11-07.

Groups in source order: `GroupBackgroundRemovalMode`, `GroupBackgroundRemovalClose`.

**GroupBackgroundRemovalMode** (I: "Background Removal Mode")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | PictureBackgroundRemovalMarkForeground | toggle |  | Y | ? |  |  |
| 2 | PictureBackgroundRemovalMarkBackground | toggle |  | Y | ? |  |  |
| 3 | PictureBackgroundRemovalMarkDelete | toggle |  | Y | ? |  |  |

**GroupBackgroundRemovalClose** (I: "Background Removal Close")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | BackgroundRemovalClose | button |  | Y | ? |  |  |
| 2 | KeepBackgroundRemoval | button |  | Y | ? |  |  |


### Graphics Format (tab set `TabSetSVGTools`)

ID `TabGraphicsToolsFormat`. KeyTip: not sourced. Source S1, checked 2026-11-07.

Groups in source order: `GroupIcons`, `GroupGraphicsStyles`, `GroupAltText`, `GroupArrange`, `GroupGraphicsSize`.

**GroupIcons** (I: "Icons")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | GraphicChangeMenu | dropdown |  | Y | Yes [D S2] |  |  |
| 2 | PictureChangeFromFile | button | GraphicChangeMenu | Y | ? |  |  |
| 3 | PictureChangeFromMobile | button | GraphicChangeMenu | Y | ? |  |  |
| 4 | PictureChangeFromM365 | button | GraphicChangeMenu | Y | ? |  |  |
| 5 | PictureChangeFromOnline | button | GraphicChangeMenu | Y | ? |  |  |
| 6 | ChangeIcon | button | GraphicChangeMenu | Y | ? |  |  |
| 7 | PictureChangeFromClipboard | button | GraphicChangeMenu | Y | ? |  |  |
| 8 | SVGEdit | button |  | Y | ? |  |  |

**GroupGraphicsStyles** (I: "Graphics Styles")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | GraphicsStylesGallery | gallery |  | Y | ? |  |  |
| 2 | GraphicsColorPicker | gallery |  | Y | Yes [D S2] |  |  |
| 3 | ObjectFillMoreColorsDialog | button | GraphicsColorPicker | Y | ? |  |  |
| 4 | EyedropperFill | button | GraphicsColorPicker | Y | ? |  |  |
| 5 | GraphicsOutlineColorPicker | gallery |  | Y | ? |  |  |
| 6 | ObjectBorderOutlineColorMoreColorsDialog | button | GraphicsOutlineColorPicker | Y | ? |  |  |
| 7 | EyedropperOutline | button | GraphicsOutlineColorPicker | Y | ? |  |  |
| 8 | OutlineWeightGallery | gallery | GraphicsOutlineColorPicker | Y | ? |  |  |
| 9 | LineStylesDialog | button | GraphicsOutlineColorPicker > OutlineWeightGallery | Y | ? |  |  |
| 10 | OutlineDashesGallery | gallery | GraphicsOutlineColorPicker | Y | ? |  |  |
| 11 | LineStylesDialog | button | GraphicsOutlineColorPicker > OutlineDashesGallery | Y | ? |  |  |
| 12 | GraphicsEffectsMenu | dropdown |  | Y | ? |  |  |
| 13 | ObjectEffectPresetGallery | gallery | GraphicsEffectsMenu | Y | ? |  |  |
| 14 | _3DBevelOptionsDialog | button | GraphicsEffectsMenu > ObjectEffectPresetGallery | Y | ? |  |  |
| 15 | ObjectEffectShadowGallery | gallery | GraphicsEffectsMenu | Y | ? |  |  |
| 16 | ShadowOptionsDialog | button | GraphicsEffectsMenu > ObjectEffectShadowGallery | Y | ? |  |  |
| 17 | ReflectionGallery | gallery | GraphicsEffectsMenu | Y | ? |  |  |
| 18 | ReflectionsMoreOptions | button | GraphicsEffectsMenu > ReflectionGallery | Y | ? |  |  |
| 19 | ObjectEffectGlowGallery | gallery | GraphicsEffectsMenu | Y | ? |  |  |
| 20 | GlowColorPicker | gallery | GraphicsEffectsMenu > ObjectEffectGlowGallery | Y | ? |  |  |
| 21 | GlowColorMoreColorsDialog | button | GraphicsEffectsMenu > ObjectEffectGlowGallery | Y | ? |  |  |
| 22 | EyedropperGlow | button | GraphicsEffectsMenu > ObjectEffectGlowGallery | Y | ? |  |  |
| 23 | GlowsMoreOptions | button | GraphicsEffectsMenu > ObjectEffectGlowGallery | Y | ? |  |  |
| 24 | ObjectEffectSoftEdgesGallery | gallery | GraphicsEffectsMenu | Y | ? |  |  |
| 25 | SoftEdgesMoreOptions | button | GraphicsEffectsMenu > ObjectEffectSoftEdgesGallery | Y | ? |  |  |
| 26 | BevelShapeGallery | gallery | GraphicsEffectsMenu | Y | ? |  |  |
| 27 | _3DBevelOptionsDialog | button | GraphicsEffectsMenu > BevelShapeGallery | Y | ? |  |  |
| 28 | _3DRotationGallery | gallery | GraphicsEffectsMenu | Y | ? |  |  |
| 29 | _3DRotationOptionsDialog | button | GraphicsEffectsMenu > _3DRotationGallery | Y | ? |  |  |
| 30 | GraphicsFormatDialog | launcher |  | Y | ? |  |  |

**GroupAltText** (I: "Alt Text"). 1 rows identical to `TabDrawingToolsFormat`.

**GroupArrange** (I: "Arrange"). 46 rows identical to `TabTableToolsLayout`.

**GroupGraphicsSize** (I: "Graphics Size")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | PictureCropTools | split |  | Y | Yes, web tab FormatPicture [D S2] |  |  |
| 2 | PictureCrop | toggle | PictureCropTools | Y | ? |  |  |
| 3 | PictureShapeGallery | gallery | PictureCropTools | Y | ? |  |  |
| 4 | PictureCropAspectRatioMenu | dropdown | PictureCropTools | Y | ? |  |  |
| 5 | PictureCropAspectRatio1To1 | button | PictureCropTools > PictureCropAspectRatioMenu | Y | ? |  |  |
| 6 | PictureCropAspectRatio2To3 | button | PictureCropTools > PictureCropAspectRatioMenu | Y | ? |  |  |
| 7 | PictureCropAspectRatio3To4 | button | PictureCropTools > PictureCropAspectRatioMenu | Y | ? |  |  |
| 8 | PictureCropAspectRatio3To5 | button | PictureCropTools > PictureCropAspectRatioMenu | Y | ? |  |  |
| 9 | PictureCropAspectRatio4To5 | button | PictureCropTools > PictureCropAspectRatioMenu | Y | ? |  |  |
| 10 | PictureCropAspectRatio3To2 | button | PictureCropTools > PictureCropAspectRatioMenu | Y | ? |  |  |
| 11 | PictureCropAspectRatio4To3 | button | PictureCropTools > PictureCropAspectRatioMenu | Y | ? |  |  |
| 12 | PictureCropAspectRatio5To3 | button | PictureCropTools > PictureCropAspectRatioMenu | Y | ? |  |  |
| 13 | PictureCropAspectRatio5To4 | button | PictureCropTools > PictureCropAspectRatioMenu | Y | ? |  |  |
| 14 | PictureCropAspectRatio16To9 | button | PictureCropTools > PictureCropAspectRatioMenu | Y | ? |  |  |
| 15 | PictureCropAspectRatio16To10 | button | PictureCropTools > PictureCropAspectRatioMenu | Y | ? |  |  |
| 16 | PictureFillCrop | button | PictureCropTools | Y | ? |  |  |
| 17 | PictureFitCrop | button | PictureCropTools | Y | ? |  |  |
| 18 | ShapeHeight | control |  | Y | ? |  |  |
| 19 | ShapeWidth | control |  | Y | ? |  |  |
| 20 | ObjectSizeAndPositionDialog | launcher |  | Y | ? |  |  |

Web evidence for this tab (S2, web tab `Graphics`). Single-line groups: `GroupArrange`, `GroupGraphicsSize`, `GroupGraphicsStyles`. Multiline groups: `GroupGraphicsStyles`.

Web control IDs with no match on this desktop tab: `GraphicsStyleGallery`.


### 3D Model Format (tab set `TabSet3DModelTools`)

ID `Tab3DModelToolsFormat`. KeyTip: not sourced. Source S1, checked 2026-11-07.

Groups in source order: `Group3DModelPlay`, `GroupAdjust3DModel`, `Group3DModelStyles`, `GroupAltText`, `GroupArrange`, `Group3DModelSize`.

**Group3DModelPlay** (I: "3 DModel Play")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | _3DModelPlay | button |  | Y | ? |  |  |
| 2 | _3DModelSceneGallery | gallery |  | Y | ? |  |  |

**GroupAdjust3DModel** (I: "Adjust3 DModel")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | Insert3DModelFallback | button |  | Y | ? |  |  |
| 2 | Insert3DModelDropdown | split |  | Y | ? |  |  |
| 3 | Insert3DModelDefault | button | Insert3DModelDropdown | Y | ? |  |  |
| 4 | Insert3DModelFromFile | button | Insert3DModelDropdown | Y | ? |  |  |
| 5 | Insert3DModelFromFileConsumer | button | Insert3DModelDropdown | Y | ? |  |  |
| 6 | Insert3DModelFromOnline | button | Insert3DModelDropdown | Y | ? |  |  |
| 7 | Insert3DModelFromOnlineConsumer | button | Insert3DModelDropdown | Y | ? |  |  |
| 8 | Reset3DModelSplitButton | split |  | Y | ? |  |  |
| 9 | Reset3DModel | button | Reset3DModelSplitButton | Y | ? |  |  |
| 10 | Reset3DModelAndSize | button | Reset3DModelSplitButton | Y | ? |  |  |

**Group3DModelStyles** (I: "3 DModel Styles")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | Gallery3DModelViewPresets | gallery |  | Y | ? |  |  |
| 2 | FormatDialog3DModel | launcher |  | Y | ? |  |  |

**GroupAltText** (I: "Alt Text"). 1 rows identical to `TabDrawingToolsFormat`.

**GroupArrange** (I: "Arrange")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ObjectBringForwardMenu | split |  | Y | ? |  |  |
| 2 | ObjectBringForward | button | ObjectBringForwardMenu | Y | Yes, web tab FormatShape [D S2] |  | Ctrl+Shift+] [D S3]; web Ctrl+] [D S3 Web] |
| 3 | ObjectBringToFront | button | ObjectBringForwardMenu | Y | Yes, web tab FormatShape [D S2] |  | web Ctrl+Shift+] [D S3 Web] |
| 4 | ObjectSendBackwardMenu | split |  | Y | ? |  |  |
| 5 | ObjectSendBackward | button | ObjectSendBackwardMenu | Y | Yes, web tab FormatShape [D S2] |  | Ctrl+Shift+[ [D S3]; web Ctrl+[ [D S3 Web] |
| 6 | ObjectSendToBack | button | ObjectSendBackwardMenu | Y | Yes, web tab FormatShape [D S2] |  | web Ctrl+Shift+[ [D S3 Web] |
| 7 | SelectionPane | toggle |  | Y | Yes, web tab Home [D S2] |  | Alt+F10 [D S3] |
| 8 | ObjectAlignMenu | dropdown |  | Y | Yes, web tab FormatShape [D S2] |  |  |
| 9 | ObjectsAlignLeftSmart | button | ObjectAlignMenu | Y | Yes, web tab FormatShape [D S2] |  |  |
| 10 | ObjectsAlignCenterHorizontalSmart | button | ObjectAlignMenu | Y | Yes, web tab FormatShape [D S2] |  |  |
| 11 | ObjectsAlignRightSmart | button | ObjectAlignMenu | Y | Yes, web tab FormatShape [D S2] |  |  |
| 12 | ObjectsAlignTopSmart | button | ObjectAlignMenu | Y | Yes, web tab FormatShape [D S2] |  |  |
| 13 | ObjectsAlignMiddleVerticalSmart | button | ObjectAlignMenu | Y | Yes, web tab FormatShape [D S2] |  |  |
| 14 | ObjectsAlignBottomSmart | button | ObjectAlignMenu | Y | Yes, web tab FormatShape [D S2] |  |  |
| 15 | AlignDistributeHorizontally | button | ObjectAlignMenu | Y | Yes, web tab Home [D S2] |  |  |
| 16 | AlignDistributeVertically | button | ObjectAlignMenu | Y | Yes, web tab Home [D S2] |  |  |
| 17 | ObjectsGroupMenu | dropdown |  | Y | Yes, web tab FormatShape [D S2] |  |  |
| 18 | ObjectsGroup | button | ObjectsGroupMenu | Y | Yes, web tab FormatShape [D S2] |  | Ctrl+G [D S3] |
| 19 | ObjectsRegroup | button | ObjectsGroupMenu | Y | ? |  | Ctrl+Shift+J [D S3] |
| 20 | ObjectsUngroup | button | ObjectsGroupMenu | Y | Yes, web tab FormatShape [D S2] |  | Ctrl+Shift+G [D S3] |

**Group3DModelSize** (I: "3 DModel Size")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | AdvancedEdit3DModel | toggle |  | Y | ? |  |  |
| 2 | ShapeHeight | control |  | Y | ? |  |  |
| 3 | ShapeWidth | control |  | Y | ? |  |  |
| 4 | ObjectSizeAndPositionDialog | launcher |  | Y | ? |  |  |


### Zoom Format (tab set `TabSetInteractiveIndexTools`; label I)

ID `TabInteractiveIndexToolsFormat`. KeyTip: not sourced. Source S1, checked 2026-11-07.

Groups in source order: `GroupIndexOptions`, `GroupIndexStyles`, `GroupAltText`, `GroupArrange`, `GroupSize`.

**GroupIndexOptions** (I: "Index Options")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | UpdateInteractiveIndex | button |  | Y | ? |  |  |
| 2 | ResetLayout | button |  | Y | ? |  |  |
| 3 | SectionLinkResetPictureMenu | split |  | Y | ? |  |  |
| 4 | ChangeImage | button | SectionLinkResetPictureMenu | Y | ? |  |  |
| 5 | SlideLinkReset | button | SectionLinkResetPictureMenu | Y | ? |  |  |
| 6 | ReturnBehavior | check |  | Y | ? |  |  |
| 7 | EnableTransition | check |  | Y | ? |  |  |
| 8 | SectionLinkTransitionDuration | control |  | Y | ? |  |  |

**GroupIndexStyles** (I: "Index Styles")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | PictureStylesGallery | gallery |  | Y | Yes, web tab FormatPicture [D S2] |  |  |
| 2 | InteractiveIndexUxColorPickerLine | gallery |  | Y | ? |  |  |
| 3 | ObjectBorderOutlineColorMoreColorsDialog | button | InteractiveIndexUxColorPickerLine | Y | ? |  |  |
| 4 | EyedropperOutline | button | InteractiveIndexUxColorPickerLine | Y | ? |  |  |
| 5 | OutlineWeightGallery | gallery | InteractiveIndexUxColorPickerLine | Y | ? |  |  |
| 6 | LineStylesDialog | button | InteractiveIndexUxColorPickerLine > OutlineWeightGallery | Y | ? |  |  |
| 7 | OutlineDashesGallery | gallery | InteractiveIndexUxColorPickerLine | Y | ? |  |  |
| 8 | LineStylesDialog | button | InteractiveIndexUxColorPickerLine > OutlineDashesGallery | Y | ? |  |  |
| 9 | InteractiveIndexEffectsMenu | dropdown |  | Y | ? |  |  |
| 10 | PictureEffectsPresetGallery | gallery | InteractiveIndexEffectsMenu | Y | ? |  |  |
| 11 | _3DBevelOptionsDialog | button | InteractiveIndexEffectsMenu > PictureEffectsPresetGallery | Y | ? |  |  |
| 12 | PictureEffectsShadowGallery | gallery | InteractiveIndexEffectsMenu | Y | ? |  |  |
| 13 | ShadowOptionsDialog | button | InteractiveIndexEffectsMenu > PictureEffectsShadowGallery | Y | ? |  |  |
| 14 | PictureReflectionGallery | gallery | InteractiveIndexEffectsMenu | Y | ? |  |  |
| 15 | ReflectionsMoreOptions | button | InteractiveIndexEffectsMenu > PictureReflectionGallery | Y | ? |  |  |
| 16 | PictureEffectsGlowGallery | gallery | InteractiveIndexEffectsMenu | Y | ? |  |  |
| 17 | GlowColorPicker | gallery | InteractiveIndexEffectsMenu > PictureEffectsGlowGallery | Y | ? |  |  |
| 18 | GlowColorMoreColorsDialog | button | InteractiveIndexEffectsMenu > PictureEffectsGlowGallery | Y | ? |  |  |
| 19 | EyedropperGlow | button | InteractiveIndexEffectsMenu > PictureEffectsGlowGallery | Y | ? |  |  |
| 20 | GlowsMoreOptions | button | InteractiveIndexEffectsMenu > PictureEffectsGlowGallery | Y | ? |  |  |
| 21 | PictureEffectsSoftEdgesGallery | gallery | InteractiveIndexEffectsMenu | Y | ? |  |  |
| 22 | SoftEdgesMoreOptions | button | InteractiveIndexEffectsMenu > PictureEffectsSoftEdgesGallery | Y | ? |  |  |
| 23 | _3DBevelPictureTopGallery | gallery | InteractiveIndexEffectsMenu | Y | ? |  |  |
| 24 | _3DBevelOptionsDialog | button | InteractiveIndexEffectsMenu > _3DBevelPictureTopGallery | Y | ? |  |  |
| 25 | PictureRotationGallery | gallery | InteractiveIndexEffectsMenu | Y | ? |  |  |
| 26 | _3DRotationOptionsDialog | button | InteractiveIndexEffectsMenu > PictureRotationGallery | Y | ? |  |  |
| 27 | SlideLinkHideBackgroundBooleanChoice | toggle |  | Y | ? |  |  |
| 28 | ObjectFormatDialog | launcher |  | Y | ? |  |  |

**GroupAltText** (I: "Alt Text"). 1 rows identical to `TabDrawingToolsFormat`.

**GroupArrange** (I: "Arrange"). 46 rows identical to `TabTableToolsLayout`.

**GroupSize** (I: "Size"). 3 rows identical to `TabDrawingToolsFormat`.


### Camera Format (tab set `TabSetCameoTools`; label I)

ID `TabDesignCameo`. KeyTip: not sourced. Source S1, checked 2026-11-07.

Groups in source order: `GroupChunkCameoCamera`, `GroupCameoStyles`, `GroupAltText`, `GroupArrange`, `GroupCameoSize`.

**GroupChunkCameoCamera** (I: "Chunk Cameo Camera")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | DropdownCameoTogglePreviewState | split |  | Y | ? |  |  |
| 2 | CameoEnablePreview | button | DropdownCameoTogglePreviewState | Y | ? |  |  |
| 3 | CameoSourceGallery | gallery | DropdownCameoTogglePreviewState | Y | ? |  |  |

**GroupCameoStyles** (I: "Cameo Styles")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | CameoStylesGallery | gallery |  | Y | ? |  |  |
| 2 | CopyCameoAllSlides | button |  | Y | ? |  |  |
| 3 | CameoShapeGallery | gallery |  | Y | ? |  |  |
| 4 | CameoBorderGallery | gallery |  | Y | ? |  |  |
| 5 | ObjectBorderOutlineColorMoreColorsDialog | button | CameoBorderGallery | Y | ? |  |  |
| 6 | EyedropperOutline | button | CameoBorderGallery | Y | ? |  |  |
| 7 | OutlineWeightGallery | gallery | CameoBorderGallery | Y | ? |  |  |
| 8 | LineStylesDialog | button | CameoBorderGallery > OutlineWeightGallery | Y | ? |  |  |
| 9 | OutlineDashesGallery | gallery | CameoBorderGallery | Y | ? |  |  |
| 10 | LineStylesDialog | button | CameoBorderGallery > OutlineDashesGallery | Y | ? |  |  |
| 11 | CameoEdgeEffectsGallery | dropdown |  | Y | ? |  |  |
| 12 | PictureEffectsPresetGallery | gallery | CameoEdgeEffectsGallery | Y | ? |  |  |
| 13 | _3DBevelOptionsDialog | button | CameoEdgeEffectsGallery > PictureEffectsPresetGallery | Y | ? |  |  |
| 14 | PictureEffectsShadowGallery | gallery | CameoEdgeEffectsGallery | Y | ? |  |  |
| 15 | ShadowOptionsDialog | button | CameoEdgeEffectsGallery > PictureEffectsShadowGallery | Y | ? |  |  |
| 16 | PictureReflectionGallery | gallery | CameoEdgeEffectsGallery | Y | ? |  |  |
| 17 | ReflectionsMoreOptions | button | CameoEdgeEffectsGallery > PictureReflectionGallery | Y | ? |  |  |
| 18 | PictureEffectsGlowGallery | gallery | CameoEdgeEffectsGallery | Y | ? |  |  |
| 19 | GlowColorPicker | gallery | CameoEdgeEffectsGallery > PictureEffectsGlowGallery | Y | ? |  |  |
| 20 | GlowColorMoreColorsDialog | button | CameoEdgeEffectsGallery > PictureEffectsGlowGallery | Y | ? |  |  |
| 21 | EyedropperGlow | button | CameoEdgeEffectsGallery > PictureEffectsGlowGallery | Y | ? |  |  |
| 22 | GlowsMoreOptions | button | CameoEdgeEffectsGallery > PictureEffectsGlowGallery | Y | ? |  |  |
| 23 | PictureEffectsSoftEdgesGallery | gallery | CameoEdgeEffectsGallery | Y | ? |  |  |
| 24 | SoftEdgesMoreOptions | button | CameoEdgeEffectsGallery > PictureEffectsSoftEdgesGallery | Y | ? |  |  |
| 25 | _3DBevelPictureTopGallery | gallery | CameoEdgeEffectsGallery | Y | ? |  |  |
| 26 | _3DBevelOptionsDialog | button | CameoEdgeEffectsGallery > _3DBevelPictureTopGallery | Y | ? |  |  |
| 27 | PictureRotationGallery | gallery | CameoEdgeEffectsGallery | Y | ? |  |  |
| 28 | _3DRotationOptionsDialog | button | CameoEdgeEffectsGallery > PictureRotationGallery | Y | ? |  |  |
| 29 | ObjectFormatDialog | launcher |  | Y | ? |  |  |

**GroupAltText** (I: "Alt Text"). 1 rows identical to `TabDrawingToolsFormat`.

**GroupArrange** (I: "Arrange"). 46 rows identical to `TabTableToolsLayout`.

**GroupCameoSize** (I: "Cameo Size")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | CameoCrop | toggle |  | Y | ? |  |  |
| 2 | SpinnerCameoHeight | control |  | Y | ? |  |  |
| 3 | SpinnerCameoWidth | control |  | Y | ? |  |  |
| 4 | ObjectSizeAndPositionDialog | launcher |  | Y | ? |  |  |


### Accessibility (tab set `TabSetAccessibleAuthoring`)

ID `TabAccessibleAuthoring`. KeyTip: not sourced. Source S1, checked 2026-11-07.

Groups in source order: `GroupAccessibleAuthoringReview`, `GroupColorAndContrast`, `GroupScreenReader`, `GroupCaptionsAndSubtitles`, `GroupAccessibleAuthoringResources`.

**GroupAccessibleAuthoringReview** (I: "Accessible Authoring Review")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | AccessibilityChecker | button |  | Y | Yes, web tab Review [D S2] |  |  |

**GroupColorAndContrast** (I: "Color And Contrast")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | FontColorPicker | gallery |  | Y | Yes, web tab Home [D S2] |  |  |
| 2 | FontColorMoreColorsDialogPowerPoint | button | FontColorPicker | Y | ? |  |  |
| 3 | EyedropperFillText | button | FontColorPicker | Y | ? |  |  |
| 4 | TextHighlightColorPickerLicensed | gallery |  | Y | ? |  |  |
| 5 | HighlightingStop | button | TextHighlightColorPickerLicensed | Y | Yes, web tab Home [D S2] |  |  |
| 6 | ShapeFillColorPicker | gallery |  | Y | Yes, web tab FormatShape [D S2] |  |  |
| 7 | ObjectFillMoreColorsDialog | button | ShapeFillColorPicker | Y | ? |  |  |
| 8 | EyedropperFill | button | ShapeFillColorPicker | Y | ? |  |  |
| 9 | ObjectPictureFill | button | ShapeFillColorPicker | Y | ? |  |  |
| 10 | GradientGallery | gallery | ShapeFillColorPicker | Y | ? |  |  |
| 11 | ShapeFillMoreGradientsDialog | button | ShapeFillColorPicker > GradientGallery | Y | ? |  |  |
| 12 | ShapeFillTextureGallery | gallery | ShapeFillColorPicker | Y | ? |  |  |
| 13 | MoreTextureOptions | button | ShapeFillColorPicker > ShapeFillTextureGallery | Y | ? |  |  |
| 14 | ShapeOutlineColorPicker | gallery |  | Y | Yes, web tab FormatShape [D S2] |  |  |
| 15 | ObjectBorderOutlineColorMoreColorsDialog | button | ShapeOutlineColorPicker | Y | ? |  |  |
| 16 | EyedropperOutline | button | ShapeOutlineColorPicker | Y | ? |  |  |
| 17 | OutlineWeightGallery | gallery | ShapeOutlineColorPicker | Y | ? |  |  |
| 18 | LineStylesDialog | button | ShapeOutlineColorPicker > OutlineWeightGallery | Y | ? |  |  |
| 19 | GallerySketchyShapes | gallery | ShapeOutlineColorPicker | Y | ? |  |  |
| 20 | MoreLines2 | button | ShapeOutlineColorPicker > GallerySketchyShapes | Y | ? |  |  |
| 21 | OutlineDashesGallery | gallery | ShapeOutlineColorPicker | Y | ? |  |  |
| 22 | LineStylesDialog | button | ShapeOutlineColorPicker > OutlineDashesGallery | Y | ? |  |  |
| 23 | ArrowStyleGallery | gallery | ShapeOutlineColorPicker | Y | ? |  |  |
| 24 | ArrowsMore | button | ShapeOutlineColorPicker > ArrowStyleGallery | Y | ? |  |  |
| 25 | InspectWithoutColor | toggle |  | Y | ? |  |  |
| 26 | LauncherAccessibleAuthoringColorAndContrast | launcher |  | Y | ? |  |  |

**GroupScreenReader** (I: "Screen Reader")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | Spelling | button |  | Y | ? |  |  |
| 2 | DropdownSlideTitle | split |  | Y | ? |  |  |
| 3 | SlideTitleDefault | button | DropdownSlideTitle | Y | ? |  |  |
| 4 | EditSlideTitle | button | DropdownSlideTitle | Y | ? |  |  |
| 5 | SetAsSlideTitle | button | DropdownSlideTitle | Y | ? |  |  |
| 6 | AddSlideTitle | button | DropdownSlideTitle | Y | ? |  |  |
| 7 | AddOffSlideTitle | button | DropdownSlideTitle | Y | ? |  |  |
| 8 | ReadingOrderPane | toggle |  | Y | ? |  |  |
| 9 | AltTextPaneRibbon | toggle |  | Y | Yes, web tab FormatPicture [D S2] |  |  |
| 10 | DecorativeToggleButton | toggle |  | Y | ? |  |  |
| 11 | ObjectsGroupMenu | dropdown |  | Y | Yes, web tab FormatShape [D S2] |  |  |
| 12 | ObjectsGroup | button | ObjectsGroupMenu | Y | Yes, web tab FormatShape [D S2] |  | Ctrl+G [D S3] |
| 13 | ObjectsRegroup | button | ObjectsGroupMenu | Y | ? |  | Ctrl+Shift+J [D S3] |
| 14 | ObjectsUngroup | button | ObjectsGroupMenu | Y | Yes, web tab FormatShape [D S2] |  | Ctrl+Shift+G [D S3] |
| 15 | InsertHeaderRowSplitDropdown | split |  | Y | ? |  |  |
| 16 | InsertHeaderRow | button | InsertHeaderRowSplitDropdown | Y | ? |  |  |
| 17 | TableStructureFirstRowToggleButton | toggle | InsertHeaderRowSplitDropdown | Y | ? |  |  |
| 18 | InsertHeaderColumn | button | InsertHeaderRowSplitDropdown | Y | ? |  |  |
| 19 | TableStructureFirstColToggleButton | toggle | InsertHeaderRowSplitDropdown | Y | ? |  |  |
| 20 | EditHyperlinkText | button |  | Y | ? |  |  |
| 21 | LauncherAccessibleAuthoringScreenReader | launcher |  | Y | ? |  |  |

**GroupCaptionsAndSubtitles** (I: "Captions And Subtitles")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | UseSubtitles | check |  | Y | ? |  |  |
| 2 | SubtitleSettingsMenu | dropdown |  | Y | ? |  |  |
| 3 | SubtitlesSpokenLanguage | gallery | SubtitleSettingsMenu | Y | Yes, web tab View [D S2] |  |  |
| 4 | SubtitlesDisplayLanguage | gallery | SubtitleSettingsMenu | Y | Yes, web tab View [D S2] |  |  |
| 5 | SubtitlesMicrophoneSettings | gallery | SubtitleSettingsMenu | Y | ? |  |  |
| 6 | SubtitlePositionBottomOverlayed | toggle | SubtitleSettingsMenu | Y | ? |  |  |
| 7 | SubtitlePositionTopOverlayed | toggle | SubtitleSettingsMenu | Y | ? |  |  |
| 8 | SubtitlePositionBelowSlide | toggle | SubtitleSettingsMenu | Y | ? |  |  |
| 9 | SubtitlePositionAboveSlide | toggle | SubtitleSettingsMenu | Y | ? |  |  |
| 10 | SubtitleMoreSettings | button | SubtitleSettingsMenu | Y | ? |  |  |
| 11 | ClosedCaptionSplitDropdown | split |  | Y | ? |  |  |
| 12 | InsertClosedCaption | button | ClosedCaptionSplitDropdown | Y | ? |  |  |
| 13 | RemoveClosedCaption | button | ClosedCaptionSplitDropdown | Y | ? |  |  |
| 14 | LauncherAccessibleAuthoringCaptionsAndSubtitles | launcher |  | Y | ? |  |  |

**GroupAccessibleAuthoringResources** (I: "Accessible Authoring Resources")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | AccessibilityHelp | button |  | Y | ? |  |  |


### Older chart Design

ID `TabChartToolsDesign`. KeyTip: not sourced. Source S1, checked 2026-11-07. Older form of chart tools in the same source.

Groups in source order: `GroupChartData`, `GroupChartStyles`.

**GroupChartData** (I: "Chart Data"). 6 rows identical to `TabChartToolsDesignNew`.

**GroupChartStyles** (I: "Chart Styles"). 2 rows identical to `TabChartToolsDesignNew`.


### Older chart Layout

ID `TabChartToolsLayout`. KeyTip: not sourced. Source S1, checked 2026-11-07. Older form.

Groups in source order: `GroupChartCurrentSelection`, `GroupChartShapes`, `GroupChartLabels`, `GroupChartAxes`, `GroupChartBackground`, `GroupChartAnalysis`.

**GroupChartCurrentSelection** (I: "Chart Current Selection"). 3 rows identical to `TabChartToolsFormatNew`.

**GroupChartShapes** (I: "Chart Shapes")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | PictureInsertFromFile | button |  | Y | ? |  |  |
| 2 | ShapesInsertGallery | gallery |  | Y | Yes, web tab FormatShape [D S2] |  |  |
| 3 | TextBoxInsert | toggle |  | Y | Yes, web tab Insert [D S2] |  |  |
| 4 | TextBoxInsertMenu | split |  | Y | Yes, web tab FormatShape [D S2] |  |  |
| 5 | TextBoxInsertHorizontal | toggle | TextBoxInsertMenu | Y | ? |  |  |
| 6 | TextBoxInsertVertical | toggle | TextBoxInsertMenu | Y | ? |  |  |

**GroupChartLabels** (I: "Chart Labels")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ChartTitle | gallery |  | Y | ? |  |  |
| 2 | ChartTitleOptionsDialog | button | ChartTitle | Y | ? |  |  |
| 3 | ChartAxisTitles | dropdown |  | Y | ? |  |  |
| 4 | ChartPrimaryHorizontalAxisTitle | gallery | ChartAxisTitles | Y | ? |  |  |
| 5 | ChartPrimaryHorizontalAxisTitleOptionsDialog | button | ChartAxisTitles > ChartPrimaryHorizontalAxisTitle | Y | ? |  |  |
| 6 | ChartPrimaryVerticalAxisTitle | gallery | ChartAxisTitles | Y | ? |  |  |
| 7 | ChartPrimaryVerticalAxisTitleOptionsDialog | button | ChartAxisTitles > ChartPrimaryVerticalAxisTitle | Y | ? |  |  |
| 8 | ChartSecondaryHorizontalAxisTitle | gallery | ChartAxisTitles | Y | ? |  |  |
| 9 | ChartSecondaryHorizontalAxisTitleOptionsDialog | button | ChartAxisTitles > ChartSecondaryHorizontalAxisTitle | Y | ? |  |  |
| 10 | ChartSecondaryVerticalAxisTitle | gallery | ChartAxisTitles | Y | ? |  |  |
| 11 | ChartSecondaryVerticalAxisTitleOptionsDialog | button | ChartAxisTitles > ChartSecondaryVerticalAxisTitle | Y | ? |  |  |
| 12 | ChartDepthAxisTitle | gallery | ChartAxisTitles | Y | ? |  |  |
| 13 | ChartDepthAxisTitleOptionsDialog | button | ChartAxisTitles > ChartDepthAxisTitle | Y | ? |  |  |
| 14 | ChartLegend | gallery |  | Y | ? |  |  |
| 15 | ChartLegendOptionsDialogDialog | button | ChartLegend | Y | ? |  |  |
| 16 | ChartDataLabel | gallery |  | Y | ? |  |  |
| 17 | ChartDataLabelDialog | button | ChartDataLabel | Y | ? |  |  |
| 18 | ChartDataTable | gallery |  | Y | ? |  |  |
| 19 | ChartDataTableOptionsDialog | button | ChartDataTable | Y | ? |  |  |

**GroupChartAxes** (I: "Chart Axes")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ChartAxes | dropdown |  | Y | ? |  |  |
| 2 | ChartPrimaryHorizontalAxis | gallery | ChartAxes | Y | ? |  |  |
| 3 | ChartPrimaryHorizontalAxisOptionsDialog | button | ChartAxes > ChartPrimaryHorizontalAxis | Y | ? |  |  |
| 4 | ChartPrimaryVerticalAxis | gallery | ChartAxes | Y | ? |  |  |
| 5 | ChartPrimaryVerticalAxisOptionsDialog | button | ChartAxes > ChartPrimaryVerticalAxis | Y | ? |  |  |
| 6 | ChartSecondaryHorizontalAxis | gallery | ChartAxes | Y | ? |  |  |
| 7 | ChartSecondaryHorizontalAxisOption | button | ChartAxes > ChartSecondaryHorizontalAxis | Y | ? |  |  |
| 8 | ChartSecondaryVerticalAxis | gallery | ChartAxes | Y | ? |  |  |
| 9 | ChartSecondaryVerticalAxisOptionsDialog | button | ChartAxes > ChartSecondaryVerticalAxis | Y | ? |  |  |
| 10 | ChartDepthAxis | gallery | ChartAxes | Y | ? |  |  |
| 11 | ChartDepthAxisOptionsDialog | button | ChartAxes > ChartDepthAxis | Y | ? |  |  |
| 12 | ChartGridlines | dropdown |  | Y | ? |  |  |
| 13 | ChartPrimaryHorizontalGridlines | gallery | ChartGridlines | Y | ? |  |  |
| 14 | ChartPrimaryHorizontalGridlinesOptionsDialog | button | ChartGridlines > ChartPrimaryHorizontalGridlines | Y | ? |  |  |
| 15 | ChartPrimaryVerticalGridlines | gallery | ChartGridlines | Y | ? |  |  |
| 16 | ChartPrimaryVerticalGridlinesOptionsDialog | button | ChartGridlines > ChartPrimaryVerticalGridlines | Y | ? |  |  |
| 17 | ChartSecondaryHorizontalGridlines | gallery | ChartGridlines | Y | ? |  |  |
| 18 | ChartSecondaryHorizontalGridlinesOptionsDialog | button | ChartGridlines > ChartSecondaryHorizontalGridlines | Y | ? |  |  |
| 19 | ChartSecondaryVerticalGridlines | gallery | ChartGridlines | Y | ? |  |  |
| 20 | ChartSecondaryVerticalGridlinesOptionsDialog | button | ChartGridlines > ChartSecondaryVerticalGridlines | Y | ? |  |  |
| 21 | ChartDepthGridlines | gallery | ChartGridlines | Y | ? |  |  |
| 22 | ChartDepthGridlinesOptionsDialog | button | ChartGridlines > ChartDepthGridlines | Y | ? |  |  |

**GroupChartBackground** (I: "Chart Background")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ChartPlotArea | gallery |  | Y | ? |  |  |
| 2 | ChartPlotAreaOptionsDialog | button | ChartPlotArea | Y | ? |  |  |
| 3 | ChartWall | gallery |  | Y | ? |  |  |
| 4 | ChartWallOptionsDialog | button | ChartWall | Y | ? |  |  |
| 5 | ChartFloor | gallery |  | Y | ? |  |  |
| 6 | ChartFloorOptionsDialog | button | ChartFloor | Y | ? |  |  |
| 7 | Chart3DView | button |  | Y | ? |  |  |

**GroupChartAnalysis** (I: "Chart Analysis")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ChartTrendline | gallery |  | Y | ? |  |  |
| 2 | ChartTrendlineOptionsDialog | button | ChartTrendline | Y | ? |  |  |
| 3 | ChartLines | gallery |  | Y | ? |  |  |
| 4 | ChartUpDownBars | gallery |  | Y | ? |  |  |
| 5 | ChartUpDownBarsOptionsDialog | button | ChartUpDownBars | Y | ? |  |  |
| 6 | ChartErrorBars | gallery |  | Y | ? |  |  |
| 7 | ChartErrorBarsOptionsDialog | button | ChartErrorBars | Y | ? |  |  |


### Older chart Format

ID `TabChartToolsFormat`. KeyTip: not sourced. Source S1, checked 2026-11-07. Older form.

Groups in source order: `GroupChartCurrentSelection`, `GroupShapeStyles`, `GroupWordArtStyles`, `GroupArrange`, `GroupSize`.

**GroupChartCurrentSelection** (I: "Chart Current Selection"). 3 rows identical to `TabChartToolsFormatNew`.

**GroupShapeStyles** (I: "Shape Styles"). 40 rows identical to `TabSmartArtToolsFormat`.

**GroupWordArtStyles** (I: "Word Art Styles"). 33 rows identical to `TabChartToolsFormatNew`.

**GroupArrange** (I: "Arrange"). 46 rows identical to `TabTableToolsLayout`.

**GroupSize** (I: "Size"). 3 rows identical to `TabDrawingToolsFormat`.


### Legacy sound options

ID `TabSoundToolsOptions`. KeyTip: not sourced. Source S1, checked 2026-11-07. Legacy.

Groups in source order: `GroupPlay`, `GroupSoundOptions`, `GroupArrange`, `GroupSize`.

**GroupPlay** (I: "Play")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | MoviePlay | button |  | Y | ? |  |  |

**GroupSoundOptions** (I: "Sound Options")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | SlideShowVolume | gallery |  | Y | ? |  |  |
| 2 | MediaClipToolsHideDuringShow | check |  | Y | ? |  |  |
| 3 | MediaClipLoopUntilStopped | check |  | Y | ? |  |  |
| 4 | SoundPlaySoundGallery | dropdown-list |  | Y | ? |  |  |
| 5 | ObjectEditDialog | launcher |  | Y | ? |  |  |

**GroupArrange** (I: "Arrange"). 46 rows identical to `TabTableToolsLayout`.

**GroupSize** (I: "Size"). 3 rows identical to `TabDrawingToolsFormat`.


### Legacy movie options

ID `TabMovieToolsOptions`. KeyTip: not sourced. Source S1, checked 2026-11-07. Legacy.

Groups in source order: `GroupPlay`, `GroupMovieOptions`, `GroupArrange`, `GroupSize`.

**GroupPlay** (I: "Play"). 1 rows identical to `TabSoundToolsOptions`.

**GroupMovieOptions** (I: "Movie Options")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | SlideShowVolume | gallery |  | Y | ? |  |  |
| 2 | MoviePlayAutomatically | dropdown-list |  | Y | ? |  |  |
| 3 | MediaClipToolsHideDuringShow | check |  | Y | ? |  |  |
| 4 | MoviePlayFullScreen | check |  | Y | ? |  |  |
| 5 | MediaClipLoopUntilStopped | check |  | Y | ? |  |  |
| 6 | MovieRewindAfterPlaying | check |  | Y | ? |  |  |
| 7 | ObjectEditDialog | launcher |  | Y | ? |  |  |

**GroupArrange** (I: "Arrange"). 46 rows identical to `TabTableToolsLayout`.

**GroupSize** (I: "Size"). 3 rows identical to `TabDrawingToolsFormat`.


### Legacy CD audio options

ID `TabCDAudioToolsOptions`. KeyTip: not sourced. Source S1, checked 2026-11-07. Legacy.

Groups in source order: `GroupPlay`, `GroupCDAudioSetup`, `GroupArrange`, `GroupSize`.

**GroupPlay** (I: "Play"). 1 rows identical to `TabSoundToolsOptions`.

**GroupCDAudioSetup** (I: "CDAudio Setup")

| # | Control ID | Type | Parent | Dt | Web | KeyTip | Shortcut |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | SlideShowVolume | gallery |  | Y | ? |  |  |
| 2 | CDAudioStartPlayingLabel | label |  | Y | ? |  |  |
| 3 | CDAudioStartTrack | control |  | Y | ? |  |  |
| 4 | CDAudioStartTime | control |  | Y | ? |  |  |
| 5 | CDAudioStopPlayingLabel | label |  | Y | ? |  |  |
| 6 | CDAudioStopTrack | control |  | Y | ? |  |  |
| 7 | CDAudioStopTime | control |  | Y | ? |  |  |
| 8 | CDAudioPlayTrackAutomatically | dropdown-list |  | Y | ? |  |  |
| 9 | MediaClipLoopUntilStoppedAudio | check |  | Y | ? |  |  |
| 10 | MediaClipToolsHideDuringShowAudio | check |  | Y | ? |  |  |
| 11 | ObjectEditDialog | launcher |  | Y | ? |  |  |

**GroupArrange** (I: "Arrange"). 46 rows identical to `TabTableToolsLayout`.

**GroupSize** (I: "Size"). 3 rows identical to `TabDrawingToolsFormat`.

## Context menus

Source S1, 96 declared menus and 1660 rows in the block, desktop, checked 2026-11-07. Shift+F10 or the Menu key opens one [D S3]. Web context menus: only spell suggestions and slide show navigation are mentioned [D S8]; the rest is unknown. Types: b button, t toggle, g gallery, m dropdown, s split, c control, k check, cm nested menu. Menus are named by object kind in the ID. KeyTips are not sourced.

| Context menu ID | Rows | Items in source order: `ID:type`, children as `Parent>ID:type` |
| --- | --- | --- |
| ContextMenuCommentMarker | 0 | no child rows in source |
| ContextMenuCurve | 0 | no child rows in source |
| ContextMenuDrawnObject | 0 | no child rows in source |
| ContextMenuFrame | 0 | no child rows in source |
|  | 0 | no child rows in source |
| ContextMenuMotionPath | 0 | no child rows in source |
| ContextMenuPathPoint | 0 | no child rows in source |
| ContextMenuPathSegment | 0 | no child rows in source |
| ContextMenuOfficePreviewHandlerPowerPoint | 0 | no child rows in source |
| ContextMenuRevision | 0 | no child rows in source |
| ContextMenuPodiumMonitorSettings | 2 | PodiumSwitchMonitors:b; PodiumCloneSlideShow:b |
| ContextMenuRightDragDrop | 0 | no child rows in source |
| ContextMenuSlideShow | 0 | no child rows in source |
| ContextMenuShowBrowse | 0 | no child rows in source |
| ContextMenuShowHyperlink | 0 | no child rows in source |
| ContextMenuShowPointerOptions | 0 | no child rows in source |
| ContextMenuEndShow | 0 | no child rows in source |
| ContextMenuSlideSync | 0 | no child rows in source |
| ContextMenuSpell | 0 | no child rows in source |
| ContextMenuTextAlternates | 1 | ShowTextAlternates:g |
| ContextMenuSlideGap | 6 | PasteGalleryMini:g; SlideNew:b; ReuseSlides:t; PasteAsSlide:t; SectionAdd:b; SlideMasterInsertLayout:b |
| ContextMenuRPVSettings | 3 | AudioSourceMenu:m; AudioSourceMenu>(unnamed):t; RecordAudio:t |
| ContextMenuRPVDeviceSelection | 6 | AudioSourceMenu:m; AudioSourceMenu>(unnamed):t; VideoSourceMenu:m; VideoSourceMenu>(unnamed):t; VideoEffectSourceMenu:m; VideoEffectSourceMenu>(unnamed):t |
| ContextMenuOverflow | 8 | AudioSourceMenu:m; AudioSourceMenu>(unnamed):t; VideoSourceMenu:m; VideoSourceMenu>(unnamed):t; ClearCurrentSlideRecording:b; ClearAllSlidesRecordings:b; ResetToCameoOnCurrentSlide:b; ResetToCameoOnAllSlides:b |
| ContextMenuRSCameraOptions | 2 | (unnamed):t; (unnamed):t |
| ContextMenuRSMicOptions | 1 | (unnamed):t |
| ContextMenuCameraMode | 1 | (unnamed):t |
| ContextMenuClearRecordings | 2 | ClearCurrentSlideRecording:b; ClearAllSlidesRecordings:b |
| ContextMenuRetake | 6 | PodiumRetakeOnCurrentSlide:b; PodiumRetakeOnAllSlides:b; ClearCurrentSlideRecording:b; ClearAllSlidesRecordings:b; ResetToCameoOnCurrentSlide:b; ResetToCameoOnAllSlides:b |
| ContextMenuHelp | 2 | PodiumRecordSlideshowTeachingTour:b; PodiumRecordSlideshowShortcutGuide:b |
| ContextMenuLabelSection | 8 | SectionRename:b; SectionMergeWithPrevious:b; SectionDelete:b; SectionRemoveAll:b; SectionMoveUp:b; SectionMoveDown:b; SectionCollapseAll:b; SectionExpandAll:b |
| ContextMenuSlideSorter | 17 | Cut:b; Copy:b; PasteGalleryMini:g; SelectAll:b; SlideNew:b; ReuseSlides:t; PasteAsSlide:t; DuplicateSelectedSlides:b; DeleteSlideContextual:b; SectionAdd:b; FilePublishSlides:b; SlideSyncMenu:m; SlideBackgroundFormatDialog:b; SlideHide:t; HideSlide:b; LinkToSlide:b; PeopleNearMe:g |
| ContextMenuThumbnail | 25 | Cut:b; Copy:b; PasteGalleryMini:g; SelectAll:b; SlideNew:b; ReuseSlides:t; PasteAsSlide:t; SlideMasterInsertLayout:b; DuplicateSelectedSlides:b; DeleteSlideContextual:b; SectionAdd:b; SlideMasterPreserveMaster:t; SlideMasterRenameMaster:b; FilePublishSlides:b; SlideSyncMenu:m; SlideLayoutGallery:g; SlideReset:b; SlideMasterMasterLayout:b; SlideBackgroundFormatDialog:b; PhotoAlbumEdit:b; SlideHide:t; HideSlide:b; LinkToSlide:b; ReviewNewComment:b; PeopleNearMe:g |
| ContextMenuPicture | 50 | Cut:b; Copy:b; PasteGalleryMini:g; PictureEdit:b; VideoToolsTrim:b; AudioToolsTrim:b; MoviePlay:b; PictureChange:b; PictureChangeMenu:m; PictureChangeMenu>PictureChangeFromFile:b; PictureChangeMenu>PictureChangeFromMobile:b; PictureChangeMenu>PictureChangeFromM365:b; PictureChangeMenu>PictureChangeFromOnline:b; PictureChangeMenu>ChangeIcon:b; PictureChangeMenu>PictureChangeFromClipboard:b; ObjectsGroupMenu:m; ObjectsGroupMenu>ObjectsGroup:b; ObjectsGroupMenu>ObjectsRegroup:b; ObjectsGroupMenu>ObjectsUngroup:b; ObjectBringToFrontMenu:s; ObjectBringToFrontMenu>ObjectBringToFront:b; ObjectBringToFrontMenu>ObjectBringForward:b; ObjectSendToBackMenu:s; ObjectSendToBackMenu>ObjectSendToBack:b; ObjectSendToBackMenu>ObjectSendBackward:b; LockObject:b; UnlockObject:b; InsertLinkGallery:g; InsertLinkGallery>(unnamed):b; HyperlinkOpenInMenu:m; HyperlinkOpenInMenu>HyperlinkOpenInWeb:b; HyperlinkOpenInMenu>HyperlinkOpenInNative:b; HyperlinkOpenDefaultSettingMenu:m; HyperlinkOpenDefaultSettingMenu>OpenDefaultWeb:b; HyperlinkOpenDefaultSettingMenu>OpenDefaultNative:b; HyperlinkInsertPowerPoint:b; BookshelfDefine:b; HyperlinkEdit:b; HyperlinkOpen:b; HyperlinkCopy:b; HyperlinkRemove:b; PictureAttributionNavigateToSource:b; ObjectSaveAsPicture:b; BlackWhiteSettingMenu:m; AutoAlternativeText:b; ObjectSizeAndPositionDialog:b; ObjectFormatDialog:b; CopyCameoAllSlides:b; ReviewNewComment:b; PeopleNearMe:g |
| ContextMenuGraphic | 44 | Cut:b; Copy:b; PasteGalleryMini:g; SVGEdit:b; GraphicChangeMenu:m; GraphicChangeMenu>PictureChangeFromFile:b; GraphicChangeMenu>PictureChangeFromMobile:b; GraphicChangeMenu>PictureChangeFromM365:b; GraphicChangeMenu>PictureChangeFromOnline:b; GraphicChangeMenu>ChangeIcon:b; GraphicChangeMenu>PictureChangeFromClipboard:b; ObjectsGroupMenu:m; ObjectsGroupMenu>ObjectsGroup:b; ObjectsGroupMenu>ObjectsRegroup:b; ObjectsGroupMenu>ObjectsUngroup:b; ObjectBringToFrontMenu:s; ObjectBringToFrontMenu>ObjectBringToFront:b; ObjectBringToFrontMenu>ObjectBringForward:b; ObjectSendToBackMenu:s; ObjectSendToBackMenu>ObjectSendToBack:b; ObjectSendToBackMenu>ObjectSendBackward:b; LockObject:b; UnlockObject:b; InsertLinkGallery:g; InsertLinkGallery>(unnamed):b; HyperlinkOpenInMenu:m; HyperlinkOpenInMenu>HyperlinkOpenInWeb:b; HyperlinkOpenInMenu>HyperlinkOpenInNative:b; HyperlinkOpenDefaultSettingMenu:m; HyperlinkOpenDefaultSettingMenu>OpenDefaultWeb:b; HyperlinkOpenDefaultSettingMenu>OpenDefaultNative:b; HyperlinkInsertPowerPoint:b; BookshelfDefine:b; HyperlinkEdit:b; HyperlinkOpen:b; HyperlinkCopy:b; HyperlinkRemove:b; ObjectSaveAsPicture:b; BlackWhiteSettingMenu:m; AutoAlternativeText:b; ObjectSizeAndPositionDialog:b; GraphicsFormatDialog:b; ReviewNewComment:b; PeopleNearMe:g |
| ContextMenu3DModel | 36 | Cut:b; Copy:b; PasteGalleryMini:g; ObjectsGroupMenu:m; ObjectsGroupMenu>ObjectsGroup:b; ObjectsGroupMenu>ObjectsRegroup:b; ObjectsGroupMenu>ObjectsUngroup:b; ObjectBringToFrontMenu:s; ObjectBringToFrontMenu>ObjectBringToFront:b; ObjectBringToFrontMenu>ObjectBringForward:b; ObjectSendToBackMenu:s; ObjectSendToBackMenu>ObjectSendToBack:b; ObjectSendToBackMenu>ObjectSendBackward:b; LockObject:b; UnlockObject:b; InsertLinkGallery:g; InsertLinkGallery>(unnamed):b; HyperlinkOpenInMenu:m; HyperlinkOpenInMenu>HyperlinkOpenInWeb:b; HyperlinkOpenInMenu>HyperlinkOpenInNative:b; HyperlinkOpenDefaultSettingMenu:m; HyperlinkOpenDefaultSettingMenu>OpenDefaultWeb:b; HyperlinkOpenDefaultSettingMenu>OpenDefaultNative:b; HyperlinkInsertPowerPoint:b; BookshelfDefine:b; HyperlinkEdit:b; HyperlinkOpen:b; HyperlinkCopy:b; HyperlinkRemove:b; ObjectSaveAsPicture:b; PictureAttributionNavigateToSource:b; AutoAlternativeText:b; ObjectSizeAndPositionDialog:b; FormatDialog3DModel:b; ReviewNewComment:b; PeopleNearMe:g |
| ContextMenuSectionLink | 16 | Cut:b; Copy:b; PasteGalleryMini:g; SectionLinkResetPictureMenu:s; SectionLinkResetPictureMenu>ChangeImage:b; SectionLinkResetPictureMenu>SlideLinkReset:b; MSPPTOpenSectionLink:b; SlideLinkHideBackground:k; LockObject:b; UnlockObject:b; ObjectSaveAsPicture:b; ObjectSizeAndPositionDialog:b; ObjectFormatDialog:b; PictureAttributionNavigateToSource:b; ReviewNewComment:b; PeopleNearMe:g |
| ContextMenuInteractiveIndex | 28 | Cut:b; Copy:b; PasteGalleryMini:g; SectionLinkResetPictureMenu:s; SectionLinkResetPictureMenu>ChangeImage:b; SectionLinkResetPictureMenu>SlideLinkReset:b; MSPPTOpenSectionLink:b; SlideLinkHideBackground:k; UpdateInteractiveIndex:b; ResetLayout:b; ObjectsGroupMenu:m; ObjectsGroupMenu>ObjectsGroup:b; ObjectsGroupMenu>ObjectsRegroup:b; ObjectsGroupMenu>ObjectsUngroup:b; ObjectBringToFrontMenu:s; ObjectBringToFrontMenu>ObjectBringToFront:b; ObjectBringToFrontMenu>ObjectBringForward:b; ObjectSendToBackMenu:s; ObjectSendToBackMenu>ObjectSendToBack:b; ObjectSendToBackMenu>ObjectSendBackward:b; LockObject:b; UnlockObject:b; ObjectSaveAsPicture:b; AutoAlternativeText:b; ObjectSizeAndPositionDialog:b; ObjectFormatDialog:b; ReviewNewComment:b; PeopleNearMe:g |
| ContextMenuAudio | 42 | Cut:b; Copy:b; PasteGalleryMini:g; MoviePlay:b; PictureChange:b; ObjectsGroupMenu:m; ObjectsGroupMenu>ObjectsGroup:b; ObjectsGroupMenu>ObjectsRegroup:b; ObjectsGroupMenu>ObjectsUngroup:b; ObjectBringToFrontMenu:s; ObjectBringToFrontMenu>ObjectBringToFront:b; ObjectBringToFrontMenu>ObjectBringForward:b; ObjectSendToBackMenu:s; ObjectSendToBackMenu>ObjectSendToBack:b; ObjectSendToBackMenu>ObjectSendBackward:b; LockObject:b; UnlockObject:b; InsertLinkGallery:g; InsertLinkGallery>(unnamed):b; HyperlinkOpenInMenu:m; HyperlinkOpenInMenu>HyperlinkOpenInWeb:b; HyperlinkOpenInMenu>HyperlinkOpenInNative:b; HyperlinkOpenDefaultSettingMenu:m; HyperlinkOpenDefaultSettingMenu>OpenDefaultWeb:b; HyperlinkOpenDefaultSettingMenu>OpenDefaultNative:b; HyperlinkInsertPowerPoint:b; BookshelfDefine:b; HyperlinkEdit:b; HyperlinkOpen:b; HyperlinkCopy:b; HyperlinkRemove:b; MediaUpgrade:b; ObjectSaveMedia:b; ObjectSaveAsPicture:b; BlackWhiteSettingMenu:m; AutoAlternativeText:b; ObjectSizeAndPositionDialog:b; ObjectFormatDialog:b; InsertClosedCaption:b; RemoveClosedCaption:b; ReviewNewComment:b; PeopleNearMe:g |
| ContextMenuVideo | 28 | Cut:b; Copy:b; PasteGalleryMini:g; MoviePlay:b; ObjectsGroupMenu:m; ObjectsGroupMenu>ObjectsGroup:b; ObjectsGroupMenu>ObjectsRegroup:b; ObjectsGroupMenu>ObjectsUngroup:b; ObjectBringToFrontMenu:s; ObjectBringToFrontMenu>ObjectBringToFront:b; ObjectBringToFrontMenu>ObjectBringForward:b; ObjectSendToBackMenu:s; ObjectSendToBackMenu>ObjectSendToBack:b; ObjectSendToBackMenu>ObjectSendBackward:b; LockObject:b; UnlockObject:b; MediaUpgrade:b; ObjectSaveMedia:b; ObjectSaveAsPicture:b; BlackWhiteSettingMenu:m; AutoAlternativeText:b; ObjectSizeAndPositionDialog:b; ObjectFormatDialog:b; PictureAttributionNavigateToSource:b; InsertClosedCaption:b; RemoveClosedCaption:b; ReviewNewComment:b; PeopleNearMe:g |
| ContextMenuOnlineVideo | 24 | Cut:b; Copy:b; PasteGalleryMini:g; MoviePlay:b; ObjectsGroupMenu:m; ObjectsGroupMenu>ObjectsGroup:b; ObjectsGroupMenu>ObjectsRegroup:b; ObjectsGroupMenu>ObjectsUngroup:b; ObjectBringToFrontMenu:s; ObjectBringToFrontMenu>ObjectBringToFront:b; ObjectBringToFrontMenu>ObjectBringForward:b; ObjectSendToBackMenu:s; ObjectSendToBackMenu>ObjectSendToBack:b; ObjectSendToBackMenu>ObjectSendBackward:b; LockObject:b; UnlockObject:b; ObjectSaveAsPicture:b; BlackWhiteSettingMenu:m; AutoAlternativeText:b; ObjectSizeAndPositionDialog:b; ObjectFormatDialog:b; PictureAttributionNavigateToSource:b; ReviewNewComment:b; PeopleNearMe:g |
| ContextMenuShape | 49 | Cut:b; Copy:b; PasteGalleryMini:g; ObjectAddText:b; ObjectEditText:b; ObjectEditPoints:t; ObjectsGroupMenu:m; ObjectsGroupMenu>ObjectsGroup:b; ObjectsGroupMenu>ObjectsRegroup:b; ObjectsGroupMenu>ObjectsUngroup:b; ObjectBringToFrontMenu:s; ObjectBringToFrontMenu>ObjectBringToFront:b; ObjectBringToFrontMenu>ObjectBringForward:b; ObjectSendToBackMenu:s; ObjectSendToBackMenu>ObjectSendToBack:b; ObjectSendToBackMenu>ObjectSendBackward:b; LockObject:b; UnlockObject:b; InsertLinkGallery:g; InsertLinkGallery>(unnamed):b; HyperlinkOpenInMenu:m; HyperlinkOpenInMenu>HyperlinkOpenInWeb:b; HyperlinkOpenInMenu>HyperlinkOpenInNative:b; HyperlinkOpenDefaultSettingMenu:m; HyperlinkOpenDefaultSettingMenu>OpenDefaultWeb:b; HyperlinkOpenDefaultSettingMenu>OpenDefaultNative:b; HyperlinkInsertPowerPoint:b; BookshelfDefine:b; HyperlinkEdit:b; HyperlinkOpen:b; HyperlinkCopy:b; HyperlinkRemove:b; Insights:b; PasteAsSlide:b; CollectorFromSelection:c; ReuseSlidesFromSelection:c; ObjectSaveAsPicture:b; BlackWhiteSettingMenu:m; Translator:b; AutoAlternativeText:b; ObjectSetShapeDefaults:b; ObjectSizeAndPositionDialog:b; ObjectFormatDialog:b; PictureAttributionNavigateToSource:b; PlaceholderOrientation:m; PlaceholderOrientation>PlaceholderOrientationHorizontal:t; PlaceholderOrientation>PlaceholderOrientationVertical:t; ReviewNewComment:b; PeopleNearMe:g |
| ContextMenuInk | 27 | Cut:b; Copy:b; InkCopyAsText:b; PasteGalleryMini:g; ObjectsGroupMenu:m; ObjectsGroupMenu>ObjectsGroup:b; ObjectsGroupMenu>ObjectsRegroup:b; ObjectsGroupMenu>ObjectsUngroup:b; ObjectBringToFrontMenu:s; ObjectBringToFrontMenu>ObjectBringToFront:b; ObjectBringToFrontMenu>ObjectBringForward:b; ObjectSendToBackMenu:s; ObjectSendToBackMenu>ObjectSendToBack:b; ObjectSendToBackMenu>ObjectSendBackward:b; LockObject:b; UnlockObject:b; ObjectSaveAsPicture:b; BlackWhiteSettingMenu:m; AutoAlternativeText:b; ObjectSetShapeDefaults:b; ObjectSizeAndPositionDialog:b; ObjectFormatDialog:b; PlaceholderOrientation:m; PlaceholderOrientation>PlaceholderOrientationHorizontal:t; PlaceholderOrientation>PlaceholderOrientationVertical:t; ReviewNewComment:b; PeopleNearMe:g |
| ContextMenuObjectsGroup | 37 | Cut:b; Copy:b; PasteGalleryMini:g; ObjectsGroupMenu:m; ObjectsGroupMenu>ObjectsGroup:b; ObjectsGroupMenu>ObjectsRegroup:b; ObjectsGroupMenu>ObjectsUngroup:b; ObjectBringToFrontMenu:s; ObjectBringToFrontMenu>ObjectBringToFront:b; ObjectBringToFrontMenu>ObjectBringForward:b; ObjectSendToBackMenu:s; ObjectSendToBackMenu>ObjectSendToBack:b; ObjectSendToBackMenu>ObjectSendBackward:b; LockObject:b; UnlockObject:b; InsertLinkGallery:g; InsertLinkGallery>(unnamed):b; HyperlinkOpenInMenu:m; HyperlinkOpenInMenu>HyperlinkOpenInWeb:b; HyperlinkOpenInMenu>HyperlinkOpenInNative:b; HyperlinkOpenDefaultSettingMenu:m; HyperlinkOpenDefaultSettingMenu>OpenDefaultWeb:b; HyperlinkOpenDefaultSettingMenu>OpenDefaultNative:b; HyperlinkInsertPowerPoint:b; BookshelfDefine:b; HyperlinkEdit:b; HyperlinkOpen:b; HyperlinkCopy:b; HyperlinkRemove:b; ObjectSaveAsPicture:b; BlackWhiteSettingMenu:m; AutoAlternativeText:b; ObjectSetShapeDefaults:b; ObjectSizeAndPositionDialog:b; ObjectFormatDialog:b; ReviewNewComment:b; PeopleNearMe:g |
| ContextMenuObjectEditPoint | 8 | AddNode:b; DeleteNode:b; OpenPath:t; ClosePath:t; SmoothVertex:t; StraightVertex:t; CornerVertex:t; ExitEditCurve:b |
| ContextMenuObjectEditSegment | 7 | AddNode:b; DeleteSegment:b; OpenPath:t; ClosePath:t; MakeSegmentStraight:b; MakeSegmentCurved:b; ExitEditCurve:b |
| ContextMenuTextEdit | 52 | CritiqueViewMenu:g; FirstCorrectionAlternative:b; SecondCorrectionAlternative:b; ThirdCorrectionAlternative:b; FourthCorrectionAlternative:b; FifthCorrectionAlternative:b; MoreCorrectionAlternatives:b; (unnamed):b; Cut:b; Copy:b; PasteGalleryMini:g; EquationOptionsGallery:g; CopilotInsertPowerpoint:b; TextEditModeExit:b; FontDialog:b; ParagraphDialog:b; BulletsGallery:g; BulletsGallery>BulletsAndNumberingBulletsDialog:b; NumberingGallery:g; NumberingGallery>BulletsAndNumberingNumberingDialog:b; ConvertToSmartArt:g; ConvertToSmartArt>ConvertToSmartArtMoreSmartArtGraphicsDialog:b; ImeReconvert:b; LockObject:b; UnlockObject:b; InsertLinkGallery:g; InsertLinkGallery>(unnamed):b; HyperlinkOpenInMenu:m; HyperlinkOpenInMenu>HyperlinkOpenInWeb:b; HyperlinkOpenInMenu>HyperlinkOpenInNative:b; HyperlinkOpenDefaultSettingMenu:m; HyperlinkOpenDefaultSettingMenu>OpenDefaultWeb:b; HyperlinkOpenDefaultSettingMenu>OpenDefaultNative:b; HyperlinkInsertPowerPoint:b; BookshelfDefine:b; HyperlinkEdit:b; HyperlinkOpen:b; HyperlinkCopy:b; HyperlinkRemove:b; Insights:b; PasteAsSlide:b; CollectorFromSelection:c; ReuseSlidesFromSelection:c; FixSynonymChangeMenu:m; Translate:b; Translator:b; WordArtFormatDialog:b; ObjectFormatDialog:b; AdditionalActions:g; AdditionalActions>SmartTagOptions:b; ReviewNewComment:b; PeopleNearMe:g |
| ContextMenuShapeConnector | 42 | Cut:b; Copy:b; PasteGalleryMini:g; ShapeConnectorStyleMenu:m; ShapeConnectorStyleMenu>ShapeConnectorStyleStraight:t; ShapeConnectorStyleMenu>ShapeConnectorStyleElbow:t; ShapeConnectorStyleMenu>ShapeConnectorStyleCurved:t; ShapeRerouteConnectors:t; ObjectsGroupMenu:m; ObjectsGroupMenu>ObjectsGroup:b; ObjectsGroupMenu>ObjectsRegroup:b; ObjectsGroupMenu>ObjectsUngroup:b; ObjectBringToFrontMenu:s; ObjectBringToFrontMenu>ObjectBringToFront:b; ObjectBringToFrontMenu>ObjectBringForward:b; ObjectSendToBackMenu:s; ObjectSendToBackMenu>ObjectSendToBack:b; ObjectSendToBackMenu>ObjectSendBackward:b; LockObject:b; UnlockObject:b; InsertLinkGallery:g; InsertLinkGallery>(unnamed):b; HyperlinkOpenInMenu:m; HyperlinkOpenInMenu>HyperlinkOpenInWeb:b; HyperlinkOpenInMenu>HyperlinkOpenInNative:b; HyperlinkOpenDefaultSettingMenu:m; HyperlinkOpenDefaultSettingMenu>OpenDefaultWeb:b; HyperlinkOpenDefaultSettingMenu>OpenDefaultNative:b; HyperlinkInsertPowerPoint:b; BookshelfDefine:b; HyperlinkEdit:b; HyperlinkOpen:b; HyperlinkCopy:b; HyperlinkRemove:b; ObjectSaveAsPicture:b; BlackWhiteSettingMenu:m; AutoAlternativeText:b; ObjectSetShapeDefaults:b; ObjectSizeAndPositionDialog:b; ObjectFormatDialog:b; ReviewNewComment:b; PeopleNearMe:g |
| ContextMenuShapeFreeform | 41 | Cut:b; Copy:b; PasteGalleryMini:g; ObjectAddText:b; ObjectEditText:b; ObjectEditPoints:t; ObjectsGroupMenu:m; ObjectsGroupMenu>ObjectsGroup:b; ObjectsGroupMenu>ObjectsRegroup:b; ObjectsGroupMenu>ObjectsUngroup:b; ObjectBringToFrontMenu:s; ObjectBringToFrontMenu>ObjectBringToFront:b; ObjectBringToFrontMenu>ObjectBringForward:b; ObjectSendToBackMenu:s; ObjectSendToBackMenu>ObjectSendToBack:b; ObjectSendToBackMenu>ObjectSendBackward:b; LockObject:b; UnlockObject:b; InsertLinkGallery:g; InsertLinkGallery>(unnamed):b; HyperlinkOpenInMenu:m; HyperlinkOpenInMenu>HyperlinkOpenInWeb:b; HyperlinkOpenInMenu>HyperlinkOpenInNative:b; HyperlinkOpenDefaultSettingMenu:m; HyperlinkOpenDefaultSettingMenu>OpenDefaultWeb:b; HyperlinkOpenDefaultSettingMenu>OpenDefaultNative:b; HyperlinkInsertPowerPoint:b; BookshelfDefine:b; HyperlinkEdit:b; HyperlinkOpen:b; HyperlinkCopy:b; HyperlinkRemove:b; ObjectSaveAsPicture:b; BlackWhiteSettingMenu:m; AutoAlternativeText:b; ObjectSetShapeDefaults:b; ObjectSizeAndPositionDialog:b; ObjectFormatDialog:b; PictureAttributionNavigateToSource:b; ReviewNewComment:b; PeopleNearMe:g |
| ContextMenuChartArea | 29 | Cut:b; Copy:b; PasteGalleryMini:g; ChartResetToMatchStyle:b; FontDialog:b; ChartChangeType:b; ChartSaveTemplates:b; ChartEditDataMenu:s; ChartEditDataMenu>ChartShowDataGrid:b; ChartEditDataMenu>ChartShowDataContextualMenu:b; Chart3DView:b; ObjectsGroupMenu:m; ObjectsGroupMenu>ObjectsGroup:b; ObjectsGroupMenu>ObjectsRegroup:b; ObjectsGroupMenu>ObjectsUngroup:b; ObjectBringToFrontMenu:s; ObjectBringToFrontMenu>ObjectBringToFront:b; ObjectBringToFrontMenu>ObjectBringForward:b; ObjectSendToBackMenu:s; ObjectSendToBackMenu>ObjectSendToBack:b; ObjectSendToBackMenu>ObjectSendBackward:b; LockObject:b; UnlockObject:b; ObjectSaveAsPicture:b; AutoAlternativeText:b; ChartFormatChartArea:b; PictureAttributionNavigateToSource:b; ReviewNewComment:b; PeopleNearMe:g |
| ContextMenuChartAxis | 18 | ChartDelete:b; ChartResetToMatchStyle:b; FontDialog:b; ChartChangeType:b; ChartSourceDataContextualMenu:b; ChartEditDataMenu:s; ChartEditDataMenu>ChartShowDataGrid:b; ChartEditDataMenu>ChartShowDataContextualMenu:b; Chart3DView:b; ChartMajorGridlinesAdd:b; ChartMinorGridlinesAdd:b; ChartFormatMajorGridlinesAndSelect:b; ChartFormatMinorGridlinesAndSelect:b; ChartFormatAxis:b; PictureAttributionNavigateToSource:b; ReviewNewComment:b; (unnamed):b; PeopleNearMe:g |
| ContextMenuChartAxisTitle | 22 | FirstCorrectionAlternative:b; SecondCorrectionAlternative:b; ThirdCorrectionAlternative:b; FourthCorrectionAlternative:b; FifthCorrectionAlternative:b; MoreCorrectionAlternatives:b; (unnamed):b; ChartDelete:b; ChartResetToMatchStyle:b; ChartEditText:b; ChartExitEditText:b; FontDialog:b; ChartChangeType:b; ChartSourceDataContextualMenu:b; ChartEditDataMenu:s; ChartEditDataMenu>ChartShowDataGrid:b; ChartEditDataMenu>ChartShowDataContextualMenu:b; Chart3DView:b; ChartFormatAxisTitle:b; ReviewNewComment:b; (unnamed):b; PeopleNearMe:g |
| ContextMenuChartBackWall | 13 | ChartDelete:b; ChartResetToMatchStyle:b; ChartChangeType:b; ChartSourceDataContextualMenu:b; ChartEditDataMenu:s; ChartEditDataMenu>ChartShowDataGrid:b; ChartEditDataMenu>ChartShowDataContextualMenu:b; Chart3DView:b; ChartFormatBackWall:b; PictureAttributionNavigateToSource:b; ReviewNewComment:b; (unnamed):b; PeopleNearMe:g |
| ContextMenuChartTitle | 23 | FirstCorrectionAlternative:b; SecondCorrectionAlternative:b; ThirdCorrectionAlternative:b; FourthCorrectionAlternative:b; FifthCorrectionAlternative:b; MoreCorrectionAlternatives:b; (unnamed):b; ChartDelete:b; ChartResetToMatchStyle:b; ChartEditText:b; ChartExitEditText:b; FontDialog:b; ChartChangeType:b; ChartSourceDataContextualMenu:b; ChartEditDataMenu:s; ChartEditDataMenu>ChartShowDataGrid:b; ChartEditDataMenu>ChartShowDataContextualMenu:b; Chart3DView:b; ChartFormatChartTitle:b; PictureAttributionNavigateToSource:b; ReviewNewComment:b; (unnamed):b; PeopleNearMe:g |
| ContextMenuChartDataLabel | 26 | FirstCorrectionAlternative:b; SecondCorrectionAlternative:b; ThirdCorrectionAlternative:b; FourthCorrectionAlternative:b; FifthCorrectionAlternative:b; MoreCorrectionAlternatives:b; (unnamed):b; ChartDelete:b; ChartResetToMatchStyle:b; ChartEditText:b; ChartExitEditText:b; FontDialog:b; ChartInsertDataField:b; ChartChangeType:b; ChartSourceDataContextualMenu:b; ChartEditDataMenu:s; ChartEditDataMenu>ChartShowDataGrid:b; ChartEditDataMenu>ChartShowDataContextualMenu:b; Chart3DView:b; ChartFormatDataPointAndSelect:b; ChartChangeDataLabelShapeGallery:g; ChartFormatDataLabel:b; PictureAttributionNavigateToSource:b; ReviewNewComment:b; (unnamed):b; PeopleNearMe:g |
| ContextMenuChartDataLabels | 15 | ChartDelete:b; ChartResetToMatchStyle:b; FontDialog:b; ChartChangeType:b; ChartSourceDataContextualMenu:b; ChartEditDataMenu:s; ChartEditDataMenu>ChartShowDataGrid:b; ChartEditDataMenu>ChartShowDataContextualMenu:b; Chart3DView:b; ChartFormatDataSeriesAndSelect:b; ChartChangeDataLabelShapesGallery:g; ChartFormatDataLabels:b; ReviewNewComment:b; (unnamed):b; PeopleNearMe:g |
| ContextMenuChartDataPoint | 20 | ChartDeleteSeries:b; ChartResetToMatchStyle:b; ChartSeriesTypeChange:b; ChartSourceDataContextualMenu:b; ChartEditDataMenu:s; ChartEditDataMenu>ChartShowDataGrid:b; ChartEditDataMenu>ChartShowDataContextualMenu:b; Chart3DView:b; AddDataLabelShapesMenu:s; AddDataLabelShapesMenu>ChartDataLabelAdd:b; AddDataLabelShapesMenu>ChartCalloutDataLabelAdd:b; ChartTrendlineAdd:b; ChartFormatDataLabelAndSelect:b; ChartWaterfallSetAsTotal:b; ChartWaterfallUnsetTotal:b; ChartFormatDataPoint:b; PictureAttributionNavigateToSource:b; ReviewNewComment:b; (unnamed):b; PeopleNearMe:g |
| ContextMenuChartDataSeries | 18 | ChartDelete:b; ChartResetToMatchStyle:b; ChartSeriesTypeChange:b; ChartSourceDataContextualMenu:b; ChartEditDataMenu:s; ChartEditDataMenu>ChartShowDataGrid:b; ChartEditDataMenu>ChartShowDataContextualMenu:b; Chart3DView:b; AddDataLabelsShapesMenu:s; AddDataLabelsShapesMenu>ChartDataLabelsAdd:b; AddDataLabelsShapesMenu>ChartCalloutDataLabelsAdd:b; ChartTrendlineAdd:b; ChartFormatDataLabelsAndSelect:b; ChartFormatDataSeries:b; PictureAttributionNavigateToSource:b; ReviewNewComment:b; (unnamed):b; PeopleNearMe:g |
| ContextMenuChartParetoLine | 2 | ChartFormatParetoLine:b; (unnamed):b |
| ContextMenuChartDataTable | 14 | ChartDelete:b; ChartResetToMatchStyle:b; FontDialog:b; ChartChangeType:b; ChartSourceDataContextualMenu:b; ChartEditDataMenu:s; ChartEditDataMenu>ChartShowDataGrid:b; ChartEditDataMenu>ChartShowDataContextualMenu:b; Chart3DView:b; ChartFormatDataTable:b; ChartFormatAxisAndSelect:b; ReviewNewComment:b; (unnamed):b; PeopleNearMe:g |
| ContextMenuChartDisplayUnit | 24 | FirstCorrectionAlternative:b; SecondCorrectionAlternative:b; ThirdCorrectionAlternative:b; FourthCorrectionAlternative:b; FifthCorrectionAlternative:b; MoreCorrectionAlternatives:b; (unnamed):b; ChartDelete:b; ChartResetToMatchStyle:b; ChartEditText:b; ChartExitEditText:b; FontDialog:b; ChartChangeType:b; ChartSourceDataContextualMenu:b; ChartEditDataMenu:s; ChartEditDataMenu>ChartShowDataGrid:b; ChartEditDataMenu>ChartShowDataContextualMenu:b; Chart3DView:b; ChartFormatDisplayUnit:b; ChartFormatAxisAndSelect:b; ChartAddAxis:b; ReviewNewComment:b; (unnamed):b; PeopleNearMe:g |
| ContextMenuChartDownBars | 12 | ChartDelete:b; ChartResetToMatchStyle:b; ChartChangeType:b; ChartSourceDataContextualMenu:b; ChartEditDataMenu:s; ChartEditDataMenu>ChartShowDataGrid:b; ChartEditDataMenu>ChartShowDataContextualMenu:b; Chart3DView:b; ChartFormatDownBars:b; ReviewNewComment:b; (unnamed):b; PeopleNearMe:g |
| ContextMenuChartDropLines | 12 | ChartDelete:b; ChartResetToMatchStyle:b; ChartChangeType:b; ChartSourceDataContextualMenu:b; ChartEditDataMenu:s; ChartEditDataMenu>ChartShowDataGrid:b; ChartEditDataMenu>ChartShowDataContextualMenu:b; Chart3DView:b; ChartFormatDropLines:b; ReviewNewComment:b; (unnamed):b; PeopleNearMe:g |
| ContextMenuChartErrorBars | 12 | ChartDelete:b; ChartResetToMatchStyle:b; ChartChangeType:b; ChartSourceDataContextualMenu:b; ChartEditDataMenu:s; ChartEditDataMenu>ChartShowDataGrid:b; ChartEditDataMenu>ChartShowDataContextualMenu:b; Chart3DView:b; ChartFormatErrorBars:b; ReviewNewComment:b; (unnamed):b; PeopleNearMe:g |
| ContextMenuChartFloor | 12 | ChartDelete:b; ChartResetToMatchStyle:b; ChartChangeType:b; ChartSourceDataContextualMenu:b; ChartEditDataMenu:s; ChartEditDataMenu>ChartShowDataGrid:b; ChartEditDataMenu>ChartShowDataContextualMenu:b; Chart3DView:b; ChartFormatFloor:b; ReviewNewComment:b; (unnamed):b; PeopleNearMe:g |
| ContextMenuChartGridlines | 14 | ChartDelete:b; ChartResetToMatchStyle:b; ChartChangeType:b; ChartSourceDataContextualMenu:b; ChartEditDataMenu:s; ChartEditDataMenu>ChartShowDataGrid:b; ChartEditDataMenu>ChartShowDataContextualMenu:b; Chart3DView:b; ChartFormatGridlines:b; ChartFormatAxisAndSelect:b; ChartAddAxis:b; ReviewNewComment:b; (unnamed):b; PeopleNearMe:g |
| ContextMenuChartHighLowLine | 12 | ChartDelete:b; ChartResetToMatchStyle:b; ChartChangeType:b; ChartSourceDataContextualMenu:b; ChartEditDataMenu:s; ChartEditDataMenu>ChartShowDataGrid:b; ChartEditDataMenu>ChartShowDataContextualMenu:b; Chart3DView:b; ChartFormatHighLowLine:b; ReviewNewComment:b; (unnamed):b; PeopleNearMe:g |
| ContextMenuChartLeaderLines | 12 | ChartDelete:b; ChartResetToMatchStyle:b; ChartChangeType:b; ChartSourceDataContextualMenu:b; ChartEditDataMenu:s; ChartEditDataMenu>ChartShowDataGrid:b; ChartEditDataMenu>ChartShowDataContextualMenu:b; Chart3DView:b; ChartFormatLeaderLines:b; ReviewNewComment:b; (unnamed):b; PeopleNearMe:g |
| ContextMenuChartLegend | 14 | ChartDelete:b; ChartResetToMatchStyle:b; FontDialog:b; ChartChangeType:b; ChartSourceDataContextualMenu:b; ChartEditDataMenu:s; ChartEditDataMenu>ChartShowDataGrid:b; ChartEditDataMenu>ChartShowDataContextualMenu:b; Chart3DView:b; ChartFormatLegend:b; PictureAttributionNavigateToSource:b; ReviewNewComment:b; (unnamed):b; PeopleNearMe:g |
| ContextMenuChartLegendEntry | 15 | ChartDelete:b; ChartDeleteSeries:b; ChartResetToMatchStyle:b; FontDialog:b; ChartSeriesTypeChange:b; ChartSourceDataContextualMenu:b; ChartEditDataMenu:s; ChartEditDataMenu>ChartShowDataGrid:b; ChartEditDataMenu>ChartShowDataContextualMenu:b; Chart3DView:b; ChartFormatDataSeriesAndSelect:b; ChartFormatLegendEntry:b; ReviewNewComment:b; (unnamed):b; PeopleNearMe:g |
| ContextMenuChartPlotArea | 14 | ChartDelete:b; ChartResetToMatchStyle:b; ChartChangeType:b; ChartSaveTemplates:b; ChartSourceDataContextualMenu:b; ChartEditDataMenu:s; ChartEditDataMenu>ChartShowDataGrid:b; ChartEditDataMenu>ChartShowDataContextualMenu:b; Chart3DView:b; ChartFormatPlotArea:b; PictureAttributionNavigateToSource:b; ReviewNewComment:b; (unnamed):b; PeopleNearMe:g |
| ContextMenuChartSeriesLine | 12 | ChartDelete:b; ChartResetToMatchStyle:b; ChartChangeType:b; ChartSourceDataContextualMenu:b; ChartEditDataMenu:s; ChartEditDataMenu>ChartShowDataGrid:b; ChartEditDataMenu>ChartShowDataContextualMenu:b; Chart3DView:b; ChartFormatSeriesLine:b; ReviewNewComment:b; (unnamed):b; PeopleNearMe:g |
| ContextMenuChartSideWall | 13 | ChartDelete:b; ChartResetToMatchStyle:b; ChartChangeType:b; ChartSourceDataContextualMenu:b; ChartEditDataMenu:s; ChartEditDataMenu>ChartShowDataGrid:b; ChartEditDataMenu>ChartShowDataContextualMenu:b; Chart3DView:b; ChartFormatSideWall:b; PictureAttributionNavigateToSource:b; ReviewNewComment:b; (unnamed):b; PeopleNearMe:g |
| ContextMenuChartTrendline | 12 | ChartDelete:b; ChartResetToMatchStyle:b; ChartChangeType:b; ChartSourceDataContextualMenu:b; ChartEditDataMenu:s; ChartEditDataMenu>ChartShowDataGrid:b; ChartEditDataMenu>ChartShowDataContextualMenu:b; Chart3DView:b; ChartFormatTrendline:b; ReviewNewComment:b; (unnamed):b; PeopleNearMe:g |
| ContextMenuChartTrendlineLabel | 24 | FirstCorrectionAlternative:b; SecondCorrectionAlternative:b; ThirdCorrectionAlternative:b; FourthCorrectionAlternative:b; FifthCorrectionAlternative:b; MoreCorrectionAlternatives:b; (unnamed):b; ChartDelete:b; ChartResetToMatchStyle:b; ChartEditText:b; ChartExitEditText:b; FontDialog:b; ChartChangeType:b; ChartSourceDataContextualMenu:b; ChartEditDataMenu:s; ChartEditDataMenu>ChartShowDataGrid:b; ChartEditDataMenu>ChartShowDataContextualMenu:b; Chart3DView:b; ChartFormatTrendlineLabel:b; ChartFormatTrendlineAndSelect:b; ChartFormatDataSeriesAndSelect:b; ReviewNewComment:b; (unnamed):b; PeopleNearMe:g |
| ContextMenuChartUpBars | 12 | ChartDelete:b; ChartResetToMatchStyle:b; ChartChangeType:b; ChartSourceDataContextualMenu:b; ChartEditDataMenu:s; ChartEditDataMenu>ChartShowDataGrid:b; ChartEditDataMenu>ChartShowDataContextualMenu:b; Chart3DView:b; ChartFormatUpBars:b; ReviewNewComment:b; (unnamed):b; PeopleNearMe:g |
| ContextMenuChartWalls | 12 | ChartDelete:b; ChartResetToMatchStyle:b; ChartChangeType:b; ChartSourceDataContextualMenu:b; ChartEditDataMenu:s; ChartEditDataMenu>ChartShowDataGrid:b; ChartEditDataMenu>ChartShowDataContextualMenu:b; Chart3DView:b; ChartFormatWalls:b; ReviewNewComment:b; (unnamed):b; PeopleNearMe:g |
| ContextMenuNotesEditText | 23 | CritiqueViewMenu:g; FirstCorrectionAlternative:b; SecondCorrectionAlternative:b; ThirdCorrectionAlternative:b; FourthCorrectionAlternative:b; FifthCorrectionAlternative:b; MoreCorrectionAlternatives:b; (unnamed):b; Cut:b; Copy:b; PasteGalleryMini:g; EquationOptionsGallery:g; FontDialog:b; ParagraphDialog:b; ImeReconvert:b; Insights:b; PasteAsSlide:b; CollectorFromSelection:c; ReuseSlidesFromSelection:c; FixSynonymChangeMenu:m; Translate:b; Translator:b; ReviewNewComment:b |
| ContextMenuTextEditOutline | 49 | CritiqueViewMenu:g; FirstCorrectionAlternative:b; SecondCorrectionAlternative:b; ThirdCorrectionAlternative:b; FourthCorrectionAlternative:b; FifthCorrectionAlternative:b; MoreCorrectionAlternatives:b; (unnamed):b; Cut:b; Copy:b; PasteGalleryMini:g; EquationOptionsGallery:g; OutlineCollapseMenu:s; OutlineCollapseMenu>OutlineCollapse:b; OutlineCollapseMenu>CollapseAll:b; OutlineExpandMenu:s; OutlineExpandMenu>OutlineExpand:b; OutlineExpandMenu>OutlineExpandAll:b; ImeReconvert:b; SlideNew:b; DeleteSlideContextual:b; OutlinePromote:b; OutlineDemote:b; OutlineMoveUp:b; OutlineMoveDown:b; InsertLinkGallery:g; InsertLinkGallery>(unnamed):b; HyperlinkOpenInMenu:m; HyperlinkOpenInMenu>HyperlinkOpenInWeb:b; HyperlinkOpenInMenu>HyperlinkOpenInNative:b; HyperlinkOpenDefaultSettingMenu:m; HyperlinkOpenDefaultSettingMenu>OpenDefaultWeb:b; HyperlinkOpenDefaultSettingMenu>OpenDefaultNative:b; HyperlinkInsertPowerPoint:b; BookshelfDefine:b; HyperlinkEdit:b; HyperlinkOpen:b; HyperlinkCopy:b; HyperlinkRemove:b; Insights:b; PasteAsSlide:b; CollectorFromSelection:c; ReuseSlidesFromSelection:c; OutlineShowTextFormatting:t; FixSynonymChangeMenu:m; Translate:b; Translator:b; AdditionalActions:g; AdditionalActions>SmartTagOptions:b |
| ContextMenuGraphicsCompatibility | 18 | Cut:b; Copy:b; PasteGalleryMini:g; ObjectsGroupMenu:m; ObjectsGroupMenu>ObjectsGroup:b; ObjectsGroupMenu>ObjectsRegroup:b; ObjectsGroupMenu>ObjectsUngroup:b; ObjectBringToFrontMenu:s; ObjectBringToFrontMenu>ObjectBringToFront:b; ObjectBringToFrontMenu>ObjectBringForward:b; ObjectSendToBackMenu:s; ObjectSendToBackMenu>ObjectSendToBack:b; ObjectSendToBackMenu>ObjectSendBackward:b; LockObject:b; UnlockObject:b; ObjectSaveAsPicture:b; ReviewNewComment:b; PeopleNearMe:g |
| ContextMenuGraphicOleClassic | 37 | Cut:b; Copy:b; PasteGalleryMini:g; EditOleObjectMenu:m; LinksUpdate:b; ObjectsGroupMenu:m; ObjectsGroupMenu>ObjectsGroup:b; ObjectsGroupMenu>ObjectsRegroup:b; ObjectsGroupMenu>ObjectsUngroup:b; ObjectBringToFrontMenu:s; ObjectBringToFrontMenu>ObjectBringToFront:b; ObjectBringToFrontMenu>ObjectBringForward:b; ObjectSendToBackMenu:s; ObjectSendToBackMenu>ObjectSendToBack:b; ObjectSendToBackMenu>ObjectSendBackward:b; LockObject:b; UnlockObject:b; InsertLinkGallery:g; InsertLinkGallery>(unnamed):b; HyperlinkOpenInMenu:m; HyperlinkOpenInMenu>HyperlinkOpenInWeb:b; HyperlinkOpenInMenu>HyperlinkOpenInNative:b; HyperlinkOpenDefaultSettingMenu:m; HyperlinkOpenDefaultSettingMenu>OpenDefaultWeb:b; HyperlinkOpenDefaultSettingMenu>OpenDefaultNative:b; HyperlinkInsertPowerPoint:b; BookshelfDefine:b; HyperlinkEdit:b; HyperlinkOpen:b; HyperlinkCopy:b; HyperlinkRemove:b; ObjectSaveAsPicture:b; BlackWhiteSettingMenu:m; AutoAlternativeText:b; ObjectFormatDialog:b; ReviewNewComment:b; PeopleNearMe:g |
| ContextMenuActiveXControl | 24 | Cut:b; Copy:b; PasteGalleryMini:g; ObjectSaveAsPicture:b; PropertySheet:b; ViewVisualBasicCode:b; EditOleObjectMenu:m; ObjectsGroupMenu:m; ObjectsGroupMenu>ObjectsGroup:b; ObjectsGroupMenu>ObjectsRegroup:b; ObjectsGroupMenu>ObjectsUngroup:b; ObjectBringToFrontMenu:s; ObjectBringToFrontMenu>ObjectBringToFront:b; ObjectBringToFrontMenu>ObjectBringForward:b; ObjectSendToBackMenu:s; ObjectSendToBackMenu>ObjectSendToBack:b; ObjectSendToBackMenu>ObjectSendBackward:b; LockObject:b; UnlockObject:b; AutoAlternativeText:b; ActionInsert:b; ObjectSizeAndPositionDialog:b; ReviewNewComment:b; PeopleNearMe:g |
| ContextMenuTable | 49 | CritiqueViewMenu:g; FirstCorrectionAlternative:b; SecondCorrectionAlternative:b; ThirdCorrectionAlternative:b; FourthCorrectionAlternative:b; FifthCorrectionAlternative:b; MoreCorrectionAlternatives:b; (unnamed):b; Cut:b; Copy:b; PasteGalleryMini:g; FontDialog:b; BulletsGallery:g; BulletsGallery>BulletsAndNumberingBulletsDialog:b; NumberingGallery:g; NumberingGallery>BulletsAndNumberingNumberingDialog:b; LockObject:b; UnlockObject:b; ImeReconvert:b; InsertLinkGallery:g; InsertLinkGallery>(unnamed):b; HyperlinkOpenInMenu:m; HyperlinkOpenInMenu>HyperlinkOpenInWeb:b; HyperlinkOpenInMenu>HyperlinkOpenInNative:b; HyperlinkOpenDefaultSettingMenu:m; HyperlinkOpenDefaultSettingMenu>OpenDefaultWeb:b; HyperlinkOpenDefaultSettingMenu>OpenDefaultNative:b; HyperlinkInsertPowerPoint:b; BookshelfDefine:b; HyperlinkEdit:b; HyperlinkOpen:b; HyperlinkCopy:b; HyperlinkRemove:b; Insights:b; PasteAsSlide:b; CollectorFromSelection:c; ReuseSlidesFromSelection:c; FixSynonymChangeMenu:m; Translate:b; Translator:b; MergeCells:b; SplitCells:b; TableSelect:b; ObjectFormatDialog:b; PictureAttributionNavigateToSource:b; AdditionalActions:g; AdditionalActions>SmartTagOptions:b; ReviewNewComment:b; PeopleNearMe:g |
| ContextMenuTableWhole | 40 | Cut:b; Copy:b; PasteGalleryMini:g; FontDialog:b; BulletsGallery:g; BulletsGallery>BulletsAndNumberingBulletsDialog:b; NumberingGallery:g; NumberingGallery>BulletsAndNumberingNumberingDialog:b; ObjectsGroupMenu:m; ObjectsGroupMenu>ObjectsGroup:b; ObjectsGroupMenu>ObjectsRegroup:b; ObjectsGroupMenu>ObjectsUngroup:b; ObjectBringToFrontMenu:s; ObjectBringToFrontMenu>ObjectBringToFront:b; ObjectBringToFrontMenu>ObjectBringForward:b; ObjectSendToBackMenu:s; ObjectSendToBackMenu>ObjectSendToBack:b; ObjectSendToBackMenu>ObjectSendBackward:b; LockObject:b; UnlockObject:b; InsertLinkGallery:g; InsertLinkGallery>(unnamed):b; HyperlinkOpenInMenu:m; HyperlinkOpenInMenu>HyperlinkOpenInWeb:b; HyperlinkOpenInMenu>HyperlinkOpenInNative:b; HyperlinkOpenDefaultSettingMenu:m; HyperlinkOpenDefaultSettingMenu>OpenDefaultWeb:b; HyperlinkOpenDefaultSettingMenu>OpenDefaultNative:b; HyperlinkInsertPowerPoint:b; BookshelfDefine:b; HyperlinkEdit:b; HyperlinkOpen:b; HyperlinkCopy:b; HyperlinkRemove:b; AutoAlternativeText:b; ObjectSaveAsPicture:b; Translator:b; ObjectFormatDialog:b; ReviewNewComment:b; PeopleNearMe:g |
| ContextMenuSmartArtContentPane | 18 | CritiqueViewMenu:g; FirstCorrectionAlternative:b; SecondCorrectionAlternative:b; ThirdCorrectionAlternative:b; FourthCorrectionAlternative:b; FifthCorrectionAlternative:b; MoreCorrectionAlternatives:b; (unnamed):b; Cut:b; Copy:b; PasteGalleryMini:g; SmartArtPromote:b; SmartArtDemote:b; SmartArtReorderUp:b; SmartArtReorderDown:b; FontDialog:b; ImeReconvert:b; PeopleNearMe:g |
| ContextMenuSmartArtBackground | 41 | Cut:b; Copy:b; PasteGalleryMini:g; SmartArtChangeLayout:b; SmartArtTextPane:t; ObjectsGroupMenu:m; ObjectsGroupMenu>ObjectsGroup:b; ObjectsGroupMenu>ObjectsRegroup:b; ObjectsGroupMenu>ObjectsUngroup:b; ObjectBringToFrontMenu:s; ObjectBringToFrontMenu>ObjectBringToFront:b; ObjectBringToFrontMenu>ObjectBringForward:b; ObjectSendToBackMenu:s; ObjectSendToBackMenu>ObjectSendToBack:b; ObjectSendToBackMenu>ObjectSendBackward:b; LockObject:b; UnlockObject:b; InsertLinkGallery:g; InsertLinkGallery>(unnamed):b; HyperlinkOpenInMenu:m; HyperlinkOpenInMenu>HyperlinkOpenInWeb:b; HyperlinkOpenInMenu>HyperlinkOpenInNative:b; HyperlinkOpenDefaultSettingMenu:m; HyperlinkOpenDefaultSettingMenu>OpenDefaultWeb:b; HyperlinkOpenDefaultSettingMenu>OpenDefaultNative:b; HyperlinkInsertPowerPoint:b; HyperlinkEdit:b; HyperlinkOpen:b; HyperlinkCopy:b; HyperlinkRemove:b; ObjectSaveAsPicture:b; Translator:b; AutoAlternativeText:b; SmartArtResetGraphic:b; SmartArtConvedrtToText:b; SmartArtConvertToShapes:b; ObjectSizeAndPositionDialog:b; ObjectFormatDialog:b; PictureAttributionNavigateToSource:b; ReviewNewComment:b; PeopleNearMe:g |
| ContextMenuSmartArtEditSmartArt | 41 | Cut:b; Copy:b; PasteGalleryMini:g; ObjectAddText:b; ObjectEditText:b; FontDialog:b; ShapeChangeShapeGallery:g; SmartArtAddShapeMenu:m; SmartArtAddShapeMenu>SmartArtAddShapeAfter:b; SmartArtAddShapeMenu>SmartArtAddShapeBefore:b; SmartArtAddShapeMenu>SmartArtAddShapeAbove:b; SmartArtAddShapeMenu>SmartArtAddShapeBelow:b; SmartArtAddShapeMenu>SmartArtAddAssistant:b; GraphicChangeMenu:m; GraphicChangeMenu>PictureChangeFromFile:b; GraphicChangeMenu>PictureChangeFromMobile:b; GraphicChangeMenu>PictureChangeFromM365:b; GraphicChangeMenu>PictureChangeFromOnline:b; GraphicChangeMenu>ChangeIcon:b; GraphicChangeMenu>PictureChangeFromClipboard:b; InsertLinkGallery:g; InsertLinkGallery>(unnamed):b; HyperlinkOpenInMenu:m; HyperlinkOpenInMenu>HyperlinkOpenInWeb:b; HyperlinkOpenInMenu>HyperlinkOpenInNative:b; HyperlinkOpenDefaultSettingMenu:m; HyperlinkOpenDefaultSettingMenu>OpenDefaultWeb:b; HyperlinkOpenDefaultSettingMenu>OpenDefaultNative:b; HyperlinkInsertPowerPoint:b; HyperlinkEdit:b; HyperlinkOpen:b; HyperlinkCopy:b; HyperlinkRemove:b; ObjectSaveAsPicture:b; Translator:b; SmartArtResetShape:b; ObjectSizeAndPositionDialog:b; ObjectFormatDialog:b; PictureAttributionNavigateToSource:b; ReviewNewComment:b; PeopleNearMe:g |
| ContextMenuSmartArtEdit1DShape | 16 | Cut:b; Copy:b; PasteGalleryMini:g; FontDialog:b; SmartArtAddShapeMenu:m; SmartArtAddShapeMenu>SmartArtAddShapeAfter:b; SmartArtAddShapeMenu>SmartArtAddShapeBefore:b; SmartArtAddShapeMenu>SmartArtAddShapeAbove:b; SmartArtAddShapeMenu>SmartArtAddShapeBelow:b; SmartArtAddShapeMenu>SmartArtAddAssistant:b; ObjectSaveAsPicture:b; SmartArtResetShape:b; ObjectSizeAndPositionDialog:b; ObjectFormatDialog:b; ReviewNewComment:b; PeopleNearMe:g |
| ContextMenuSmartArtEditText | 55 | CritiqueViewMenu:g; FirstCorrectionAlternative:b; SecondCorrectionAlternative:b; ThirdCorrectionAlternative:b; FourthCorrectionAlternative:b; FifthCorrectionAlternative:b; MoreCorrectionAlternatives:b; (unnamed):b; Cut:b; Copy:b; PasteGalleryMini:g; EquationOptionsGallery:g; TextEditModeExit:b; FontDialog:b; ParagraphDialog:b; ImeReconvert:b; BulletsGallery:g; BulletsGallery>BulletsAndNumberingBulletsDialog:b; NumberingGallery:g; NumberingGallery>BulletsAndNumberingNumberingDialog:b; ShapeChangeShapeGallery:g; SmartArtAddShapeMenu:m; SmartArtAddShapeMenu>SmartArtAddShapeAfter:b; SmartArtAddShapeMenu>SmartArtAddShapeBefore:b; SmartArtAddShapeMenu>SmartArtAddShapeAbove:b; SmartArtAddShapeMenu>SmartArtAddShapeBelow:b; SmartArtAddShapeMenu>SmartArtAddAssistant:b; InsertLinkGallery:g; InsertLinkGallery>(unnamed):b; HyperlinkOpenInMenu:m; HyperlinkOpenInMenu>HyperlinkOpenInWeb:b; HyperlinkOpenInMenu>HyperlinkOpenInNative:b; HyperlinkOpenDefaultSettingMenu:m; HyperlinkOpenDefaultSettingMenu>OpenDefaultWeb:b; HyperlinkOpenDefaultSettingMenu>OpenDefaultNative:b; HyperlinkInsertPowerPoint:b; HyperlinkEdit:b; HyperlinkOpen:b; HyperlinkCopy:b; HyperlinkRemove:b; Insights:b; PasteAsSlide:b; CollectorFromSelection:c; ReuseSlidesFromSelection:c; FixSynonymChangeMenu:m; Translate:b; Translator:b; SmartArtResetShape:b; WordArtFormatDialog:b; ObjectSizeAndPositionDialog:b; ObjectFormatDialog:b; AdditionalActions:g; AdditionalActions>SmartTagOptions:b; ReviewNewComment:b; PeopleNearMe:g |
| ContextMenuSmartArtEdit1DShapeText | 48 | CritiqueViewMenu:g; FirstCorrectionAlternative:b; SecondCorrectionAlternative:b; ThirdCorrectionAlternative:b; FourthCorrectionAlternative:b; FifthCorrectionAlternative:b; MoreCorrectionAlternatives:b; (unnamed):b; Cut:b; Copy:b; PasteGalleryMini:g; FontDialog:b; ParagraphDialog:b; ImeReconvert:b; SmartArtAddShapeMenu:m; SmartArtAddShapeMenu>SmartArtAddShapeAfter:b; SmartArtAddShapeMenu>SmartArtAddShapeBefore:b; SmartArtAddShapeMenu>SmartArtAddShapeAbove:b; SmartArtAddShapeMenu>SmartArtAddShapeBelow:b; SmartArtAddShapeMenu>SmartArtAddAssistant:b; InsertLinkGallery:g; InsertLinkGallery>(unnamed):b; HyperlinkOpenInMenu:m; HyperlinkOpenInMenu>HyperlinkOpenInWeb:b; HyperlinkOpenInMenu>HyperlinkOpenInNative:b; HyperlinkOpenDefaultSettingMenu:m; HyperlinkOpenDefaultSettingMenu>OpenDefaultWeb:b; HyperlinkOpenDefaultSettingMenu>OpenDefaultNative:b; HyperlinkInsertPowerPoint:b; HyperlinkEdit:b; HyperlinkOpen:b; HyperlinkCopy:b; HyperlinkRemove:b; Insights:b; PasteAsSlide:b; CollectorFromSelection:c; ReuseSlidesFromSelection:c; FixSynonymChangeMenu:m; Translate:b; Translator:b; SmartArtResetShape:b; WordArtFormatDialog:b; ObjectSizeAndPositionDialog:b; ObjectFormatDialog:b; AdditionalActions:g; AdditionalActions>SmartTagOptions:b; ReviewNewComment:b; PeopleNearMe:g |
| ContextMenuGuide | 6 | AddVerticalGuide:b; AddHorizontalGuide:b; GuideColorGallery:g; GuideDelete:b; GridSettings:b; ViewRulerPowerPoint:k |
| ContextMenuOfficeWebExtension | 20 | Cut:b; Copy:b; PasteGalleryMini:g; ObjectsGroupMenu:m; ObjectsGroupMenu>ObjectsGroup:b; ObjectsGroupMenu>ObjectsRegroup:b; ObjectsGroupMenu>ObjectsUngroup:b; ObjectBringToFrontMenu:s; ObjectBringToFrontMenu>ObjectBringToFront:b; ObjectBringToFrontMenu>ObjectBringForward:b; ObjectSendToBackMenu:s; ObjectSendToBackMenu>ObjectSendToBack:b; ObjectSendToBackMenu>ObjectSendBackward:b; LockObject:b; UnlockObject:b; ObjectSetShapeDefaults:b; ObjectSizeAndPropertiesDialog:b; ObjectFormatDialog:b; ReviewNewComment:b; PeopleNearMe:g |
| CtxPodiumNotesModeSelection | 3 | PodiumShowTeleprompterNotesView:t; PodiumShowSidePaneNotesView:t; PodiumHideNotesView:t |
| ContextMenuPresenterViewNotesEditText | 15 | CritiqueViewMenu:g; FirstCorrectionAlternative:b; SecondCorrectionAlternative:b; ThirdCorrectionAlternative:b; FourthCorrectionAlternative:b; FifthCorrectionAlternative:b; MoreCorrectionAlternatives:b; (unnamed):b; Cut:b; Copy:b; PasteGalleryMini:g; FontDialog:b; ParagraphDialog:b; ImeReconvert:b; FixSynonymChangeMenu:m |

Ungrouped rows in the context-menu block: (unnamed):g; (unnamed):g.

## Commands not in the ribbon

Source S1, 383 rows, desktop, checked 2026-11-07. This list is what File, Options offers under Commands Not in the Ribbon (mapping I from S12). Types as above. Web: unknown.

FileNewDefault:b; FileOpenUsingBackstage:b; FileSave:b; SaveAndReview:b; FileSendAsAttachment:b; FilePrintQuick:b; PrintPreviewAndPrint:b; Spelling:b; Undo:g; RedoOrRepeat:b; SlideShowFromBeginning:b; FileOpenRecentFile:b; PointerModeOptions:g; FilePermissionRestrictAs:b; MdiChildSystemMenu:m; MinimizeRibbon:t; WindowMinimize:b; WindowRestore:b; WindowClose:b; HideRibbon:t; PasteAsHyperlink:b; PasteDuplicate:b; PasteBitmap:b; PasteGif:b; PastePng:b; PasteJpeg:b; PasteInk:b; PasteTextOnly:b; PasteAsPicture:b; PasteSourceFormatting:b; PasteDestinationTheme:b; PasteExcelChartSourceFormatting:b; PasteExcelChartDestinationTheme:b; PasteLinkedExcelChartSourceFormatting:b; PasteLinkedExcelChartDestinationTheme:b; PasteAsEmbedded:b; PasteExcelTableSourceFormatting:b; PasteExcelTableDestinationTableStyle:b; FileSaveACopy:b; FileDiscardCachedChanges:b; ReopenFromServer:b; (unnamed):b; RefreshServerFile:b; ReopenServerFile:b; AnimationTransitionSpeedGallery:dl; ViewMessageBar:k; ExportToVideo:b; ExportToAnimatedGif:b; ZoomClassic:g; ShapeRectangle:t; ContrastMore:b; SlideShowPreviousSlide:b; Redo:g; BrightnessMore:b; BordersShadingDialog:b; TableInsertDialog:b; ShapeOval:t; SlideShowNextSlide:b; BrightnessLess:b; ContrastLess:b; ShapeElbowConnectorArrow:t; ObjectRotateLeft90:b; ShapeStraightConnectorArrow:t; ShadowNudgeLeftClassic:b; ShadowNudgeDownClassic:b; ShapeElbowConnector:t; ShadowNudgeRightClassic:b; ShadowNudgeUpClassic:b; OpenOptionsAddIns:b; OpenOptionsAdvanced:b; OpenOptionsPPTTrustCenter:b; OpenOptionsCustomization:b; OpenOptionsEaseOfAcess:b; OpenOptionsCustomizeRibbon:b; OpenOptionsExperimentSettings:b; OpenOptionsLanguage:b; OpenOptionsSaving:b; OpenOptionsSetup:b; OpenOptionsSpelling:b; OpenOptionsTrustCenterSettings:b; FindTellMe:c; ObjectSetShapeDefaults:b; ShapeRoundedRectangle:t; ShapeStraightConnector:t; ObjectFlipHorizontal:b; ShapeRightArrow:t; LinksUpdate:b; OutlineDemote:b; ObjectRotateFree:b; Subscript:t; SlideShowCustom:b; ShapeDownArrow:t; ObjectFlipVertical:b; Superscript:t; ObjectRotateRight90:b; OutlinePromote:b; OutlineShowTextFormatting:t; ShadowOnOrOffClassic:b; ShapeRoundedRectangularCallout:t; Clear:b; PasteApplyStyle:b; ShapeIsoscelesTriangle:t; WordArtVerticalText:b; SlideShowResumeShow:b; ShapesMoreShapes:b; AutoCorrect:b; ViewFullScreenView:b; OutlineMoveUp:b; ShapeLeftBrace:t; ObjectNudgeDown:b; ShapeRightBrace:t; PickUpStyle:b; InsertTab:b; About:b; PropertySheet:b; ObjectNudgeLeft:b; CreateHandoutsInWord:b; ObjectNudgeUp:b; ShapeArc:t; OutlineMoveDown:b; OleConvert:b; ObjectNudgeRight:b; ShapeStar:t; WebPagePreview:b; OutlineCollapse:b; OutlineExpandAll:b; SnapToGrid:t; ObjectsMultiSelect:b; CheckForUpdates:b; NextPane:b; WebGoBack:b; OutlineExpand:b; SnapToShapes:t; ViewSlideView:t; CollapseAll:b; ExchangeFolder:b; WebGoForward:b; DocumentLocation:cb; DrawingCanvasFit:b; CombineCharacters:t; ChangeCaseToggle:b; ViewOutlineView:t; DrawingCanvasScale:b; DrawingCanvasExpand:b; BaselineRaise:b; OutlineThumbnailsShowHide:b; BaselineLower:b; SlideShowInAWindow:b; WindowFitToPage:b; OutlookTaskCreate:b; FindNext:b; GoToProperty:b; LinkBreak:b; LinkChange:b; CharacterFormattingReset:b; WebOptionsDialog:b; DrawingCanvasResize:b; QuickAccessToolbarCustomization:b; RibbonCustomization:b; AlternativeText:b; AdvancedFileProperties:b; HyperlinkOpen:b; HyperlinkCopy:b; HyperlinkEdit:b; HyperlinkRemove:b; SoundMaximumFileSize:c; InkingStart:b; InkToolsClose:b; ShapesUnion:b; ShapesIntersect:b; ShapesFragment:b; ShapesCombine:b; ShapesSubtract:b; ClipArtInsert:t; ReviewEditComment:b; CombineShapesMenu:m; ShapesUnion:b under CombineShapesMenu; ShapesCombine:b under CombineShapesMenu; ShapesFragment:b under CombineShapesMenu; ShapesIntersect:b under CombineShapesMenu; ShapesSubtract:b under CombineShapesMenu; PictureBrightnessGallery:g; PictureCorrectionsDialog:b under PictureBrightnessGallery; PictureContrastGallery:g; PictureCorrectionsDialog:b under PictureContrastGallery; PictureCorrectionsReset:b; PictureColorReset:b; PictureArtisticEffectsReset:b; ShapeConvertToFreeform:b; ViewSlideShowView:b; CDAudioPlayTrack:b; PlaybackTipPlay:b; ObjectBringToFrontMenu:s; ObjectBringToFront:b under ObjectBringToFrontMenu; ObjectBringForward:b under ObjectBringToFrontMenu; ObjectSendToBackMenu:s; ObjectSendToBack:b under ObjectSendToBackMenu; ObjectSendBackward:b under ObjectSendToBackMenu; LockObject:b; UnlockObject:b; MovieInsert:s; VideoInsert:b under MovieInsert; MovieFromFileInsert:b under MovieInsert; VideoInsertFromOnline:b under MovieInsert; MovieFromClipOrganizerInsert:b under MovieInsert; SoundInsertMenu:s; AudioInsert:b under SoundInsertMenu; SoundInsertFromFile:b under SoundInsertMenu; SoundInsertFromClipOrganizer:b under SoundInsertMenu; SoundRecord:b under SoundInsertMenu; SlideShowResolutionGallery:dl; InsertMovieDialog:b; FileOpen:b; Collaborate:t; (unnamed):t; CollaborateActivity:t; CollaborateComments:t; TransitionByObject:b; WhatsNewRecentUpdates:b; GroupOfficeInsider:b; OfficeFeedbackBackstage:b; MarkAsDecorative:b; GenerateAutoAltText:b; SmartLookup:c; ReuseSlidesFromSelection:c; AddSlideTitle:b; AddOffSlideTitle:b; EditSlideTitle:b; SetAsSlideTitle:b; ManagePrivacy:b; ReadingOrderPane:t; SlideHide:t; (unnamed):b; UpgradeDocument:b; OfficeExtensionsAppStore:b; OfficeExtensionsGallery2:g; OfficeExtensionsDialog:b under OfficeExtensionsGallery2; OfficeExtensionsManageOtherAddins:b under OfficeExtensionsGallery2; ExtensibilityExperimentation:b under OfficeExtensionsGallery2; OfficeExtensionsFeaturedApp1:b; OfficeExtensionsFeaturedApp2:b; OfficeExtensionsFeaturedApp3:b; OfficeExtensionsFeaturedApp4:b; OfficeExtensionsFeaturedApp5:b; OfficeExtensionsFeaturedApp6:b; GroupPermission:grp; ReviewProtectPresentationMenu:m in GroupPermission; FilePermissionUnrestricted:t in GroupPermission under ReviewProtectPresentationMenu; FilePermissionDoNotDistribute:t in GroupPermission under ReviewProtectPresentationMenu; GroupInsertBarcode:grp; BarcodeInsert:b in GroupInsertBarcode; LabelInsert:b in GroupInsertBarcode; PictureInsertFromFilePowerPoint:b; TableDrawTable:t; SlideNewGallery:g; DuplicateSelectedSlides:b under SlideNewGallery; SlidesFromOutline:b under SlideNewGallery; SlidesReuseSlides:t under SlideNewGallery; ReuseSlides:t under SlideNewGallery; PasteAsSlide:t under SlideNewGallery; ObjectFormatDialog:b; ObjectSizeAndPositionDialog:b; AnimationCustom:t; TextBoxInsert:t; TextBoxInsertVertical:t; SlideBackgroundFormatDialog:b; SlideLayoutGallery:g; ActionInsert:b; ObjectBringToFront:b; ObjectSendToBack:b; ObjectBringForward:b; ObjectSendBackward:b; ObjectsGroup:b; ObjectsUngroup:b; ObjectsRegroup:b; FileSaveAs:b; MacroPlay:b; FormatPainter:c; FontSize:cb; TextHighlightColorPickerLicensed:g; HighlightingStop:b under TextHighlightColorPickerLicensed; FontColorPicker:g; FontColorMoreColorsDialogPowerPoint:b under FontColorPicker; EyedropperFillText:b under FontColorPicker; Font:cb; FontSizeDecrease:b; FontSizeIncrease:b; Cut:b; Paste:b; Copy:b; ShapesInsertGallery:g; WordArtInsertGallery:g; AnimationTransitionGallery:g; SlideShowFromCurrent:b; ViewSlideMasterView:t; ViewSlideSorterView:t; BulletsGallery:g; BulletsAndNumberingBulletsDialog:b under BulletsGallery; AnimationGallery:g; AnimationCustomEntranceDialog:b under AnimationGallery; AnimationCustomEmphasisDialog:b under AnimationGallery; AnimationCustomExitDialog:b under AnimationGallery; AnimationCustomPathDialog:b under AnimationGallery; AnimationCustomActionVerbDialog:b under AnimationGallery; AnimationAddGallery:g; AnimationCustomAddEntranceDialog:b under AnimationAddGallery; AnimationCustomAddEmphasisDialog:b under AnimationAddGallery; AnimationCustomAddExitDialog:b under AnimationAddGallery; AnimationCustomAddPathDialog:b under AnimationAddGallery; AnimationCustomAddActionVerbDialog:b under AnimationAddGallery; TableInsertGallery:g; TableInsert:b under TableInsertGallery; TableDrawTable:t under TableInsertGallery; ExcelSpreadsheetInsert:b under TableInsertGallery; UpgradePresentation:b; FileExit:b; FileNew:b; ApplicationOptionsDialog:b; (unnamed):b; FileStartWorkflow:b; LeaveReader:b; MenuAccessibility:s; AccessibilityChecker:b under MenuAccessibility; AltTextPaneRibbon:t under MenuAccessibility; ReadingOrderPane:t under MenuAccessibility; OpenOptionsEaseOfAcess:b under MenuAccessibility; AccessibilityHelp:b under MenuAccessibility; FilePrint:b; OfficeFeedback:b; AppGuardTrustPromotion:b; OfficeExtensionsGetAddinsButtonOnBackstageBelowOpenDivider:b; FilePrepareMenu:m; FileProperties:t under FilePrepareMenu; FileDocumentInspect:b under FilePrepareMenu; FileDocumentEncrypt:t under FilePrepareMenu; FilePermissionView:b under FilePrepareMenu; FilePermissionRestrictMenu:m under FilePrepareMenu; FilePermissionUnrestricted:t under FilePrepareMenu; FilePermissionDoNotDistribute:t under FilePrepareMenu; FileAddDigitalSignature:b under FilePrepareMenu; FileViewDigitalSignatures:t under FilePrepareMenu; FileMarkAsFinal:t under FilePrepareMenu; FileMarkAsReadOnlyRecommended:t under FilePrepareMenu; FileCompatibilityCheckerPowerPoint:b under FilePrepareMenu; FileLinksToFiles:b under FilePrepareMenu; MenuPublish:m; FilePackageForCD:b under MenuPublish; FilePublishSlides:b under MenuPublish; CreateHandoutsInWord:b under MenuPublish; FileSaveToDocumentManagementServer:b under MenuPublish; FilePrintMenu:s; PrintPreviewAndPrint:b under FilePrintMenu; FilePrintQuick:b under FilePrintMenu; FilePrintPreview:t under FilePrintMenu; FileSaveAsMenu:s; FileSaveAs:b under FileSaveAsMenu; FileSaveAsPowerPointPptx:b under FileSaveAsMenu; FileSaveAsPowerPointPpsx:b under FileSaveAsMenu; FileSaveAsPowerPoint97_2003:b under FileSaveAsMenu; FileSaveAsPowerPointOpenDocumentPresentation:b under FileSaveAsMenu; AdvertisePublishAs:b under FileSaveAsMenu; FileSaveAsPdfOrXps:b under FileSaveAsMenu; FileSaveAsOtherFormats:b under FileSaveAsMenu; FileSendMenu:m; FileSendAsAttachment:b under FileSendMenu; FileEmailAsPdfEmailAttachment:b under FileSendMenu; FileEmailAsXpsEmailAttachment:b under FileSendMenu; FileInternetFax:b under FileSendMenu; BroadcastSlideShowBackstage:b under FileSendMenu; BroadcastResume:b under FileSendMenu; FileServerTasksMenu:m; FileCheckIn:b under FileServerTasksMenu; FileCheckOut:b under FileServerTasksMenu; FileCheckOutDiscard:b under FileServerTasksMenu; FileVersionHistory:b under FileServerTasksMenu; FileWorkflowTasks:b under FileServerTasksMenu; ReviewProtectPresentationMenu:m; FilePermissionUnrestricted:t under ReviewProtectPresentationMenu; FilePermissionDoNotDistribute:t under ReviewProtectPresentationMenu; ChartShowData:b; (unnamed):grp; Insert3DModelDefaultPlaceholder:b; InsertIconPlaceholder:b.

## Slide show keys

Desktop D S4, page 2026-11-05. Web D S3 Web. Checked 2026-11-07. Slide Show controls are not ribbon commands, and they are not in the tab tables.

| Action | Desktop | Web |
| --- | --- | --- |
| Start from beginning | F5 | S, B path; no shortcut sourced |
| Start from current | Shift+F5 [O O1 2019] | S, C path |
| Presenter View | Alt+F5 | Not supported [D S7] |
| Next, previous | N, Enter, Page down, Right, Down or Spacebar; P, Page up, Left, Up or Backspace | N, Page down, Down, Enter or Space; P, Page up, Left, Up or Backspace |
| Blank black; blank white | B or Period; W or Comma | Not sourced |
| Go to slide | Number, Enter; Ctrl+S for All Slides | G, arrows, Enter |
| Next hidden slide | H | Not sourced |
| First, last slide | Home; End | Not sourced |
| Taskbar | Ctrl+T | Not sourced |
| Context menu | Shift+F10 or Menu key | Not sourced |
| Shortcut help | F1 | Not sourced |
| Hotspots | Tab; Shift+Tab; Enter opens a link | Not sourced |
| Captions on or off | Not sourced | J |
| Exit | Esc | Esc |
| Laser; pen; arrow; eraser | Ctrl+L; Ctrl+P; Ctrl+A; Ctrl+E | Not sourced |
| Hide pointer; ink show or hide; erase | Ctrl+H; Ctrl+M; E | Not sourced |
| Media play; stop; mute | Alt+P or Ctrl+Spacebar; Alt+Q; Alt+U | Not sourced |
| Volume; bookmarks | Alt+Up, Alt+Down; Alt+End, Alt+Home | Not sourced |
| Seek | Alt+Shift+Page down or up, 3 s; Alt+Shift+Right or Left, 0.25 s | Not sourced |
| Audio and subtitle menu | Alt+J | Not sourced |
| Cameo camera source | Alt+X | Not sourced |
| Rehearse: new timing; original; click; re-record | T; O; M; R | Not supported [I from S7] |
| Presenter View regions; tools | F6; Tab | n/a |
| Notes scroll | Ctrl+Down, Ctrl+Up; Ctrl+Page down, Ctrl+Page up | n/a |
| Read notes next, previous; time; next step | Alt+A, Alt+Z; Alt+W; Alt+Q | n/a |

## Web-only key paths

D per row, checked 2026-11-07. These have no matching desktop ID or use a web-only path.

| Action | Web path | Source |
| --- | --- | --- |
| Show the ribbon | Alt+Windows logo key or Alt+Period | S3 |
| Open desktop app | Z, M, O | S17 |
| Rename | F, A, R | S25 |
| Download a copy | F, A, C | S25 |
| Download as PDF | F, S, P | S26 |
| Share with people | F, Z, S | S26 |
| Print slides, notes, handouts | F, P, then R, N, H | S24 |
| Insert picture from device; Bing | N, F, P; N, F, F | S19 |
| Insert online video | N, V, O | S20 |
| Audio Format; Playback | J, P; J, N | S20 |
| Table Layout | J, L | S19 |
| SmartArt Format; Design | J, O; J, S | S19 |

## Keeping this current

The tables were generated from S1 and S2 by script and were not edited by hand; every KeyTip and shortcut overlay was checked to name an ID that exists in S1. Update it when Microsoft publishes a newer workbook, and replace `?` values from `UX-001`.
