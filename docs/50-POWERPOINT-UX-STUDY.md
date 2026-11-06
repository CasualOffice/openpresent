# 50 — PowerPoint UX study: how it behaves, web and desktop

**Status: research note, not accepted by the owner.** Nothing here is implemented, and no code was
changed to write it. It feeds `UX-001`. It does not replace the live session that `UX-001` needs, and
§40 lists what that session must measure. The rules in §41 are proposals.

## Outcome

A person who knows PowerPoint should be able to use our shell without being taught. This note records
what that person expects, step by step, and separates what Microsoft states from what is only believed.

Our product replaces Microsoft's name, logo, colours, icon artwork, fonts and templates. Behaviour and
structure are what we study. The text here is our own summary, not Microsoft's prose.

## 0. How this was done, and what it is not

**No live session was run.** No copy of PowerPoint, web or desktop, was opened for this note. **Every
pixel figure in this note is an estimate.** Microsoft publishes no device-independent size for any
PowerPoint region, and none was found in this round. OpenCalc's docs/91 §0 records the same finding for
Excel.

**Evidence marks.** Every fact carries one:

| Mark | Meaning | Trust |
| --- | --- | --- |
| **[D]** | Documented: Microsoft states it (Support, Learn, Fluent 2, Insider blog) and the page was read | Can still be stale, see §0.1 |
| **[D\*]** | Documented, but read only through a search excerpt or a summarising fetch, so the wording needs checking | Lower |
| **[O]** | Observed: a named third-party tutorial reports it. **We did not observe it ourselves** | Often an old version |
| **[I]** | Inferred: reasoned from documented facts, and the reason is given | Hypothesis |
| **[I-recall]** | Inferred from the author's memory of the product, with no source found | **Weakest. A question for the live session, not a spec** |
| **[est]** | An estimated number. Never a measurement | Do not build from it |
| **[LIVE]** | Needs a live session to settle | Open |

**Reading method.** About 55 Microsoft pages were downloaded and read as extracted text. Tab labels and
pictures are lost in extraction, so a platform is attributed by the page's tab order, and shortcut icons
in tables are missing. Further pages were read through a summarising fetch, marked [D\*]. After about 60
requests support.microsoft.com began answering 403, so the rest were read that way. Two office-watch.com
pages returned a bot check and are cited from search excerpts only.

All sources were checked on **2026-11-07**. Most Microsoft pages show no revision date.

### 0.1 Microsoft's pages contradict each other

These are real conflicts. Each one needs a live check before anyone builds on it.

| # | Topic | One page says | Another says | Likely reading |
| --- | --- | --- | --- | --- |
| C1 | Find and Replace on the web | Feature-behaviour page: Replace is not on the web | Find-and-replace page has a web tab with Replace on the right of Home | The behaviour page is stale ([I]: it still mentions Silverlight and Flash) |
| C2 | Charts on the web | Behaviour page and compare table: cannot insert | docs/48 records a conflict with the service description | **[LIVE]**. Do not assume either way |
| C3 | Video on the web | Behaviour page: online video only, none from the PC | Compare table marks adding video as supported on the web | **[LIVE]** |
| C4 | Ctrl+D on Windows | "Duplicate selected objects" | The same table also lists it as "Close a presentation" | The second is a documentation error ([I]: Duplicate is repeated by other pages) |
| C5 | Ctrl+F1 on Windows | "Expand or collapse the ribbon" | "Switch to full screen (hide menus)" | Two descriptions of one command ([I]) |
| C6 | Search box location | Simplified-ribbon page: near the right end of the ribbon | Other pages: at the top of the app window | The ribbon wording is old ([I]) |
| C7 | Web Simplified toggle | Simplified-ribbon page: a slide toggle near the top right | Shortcut page: Alt+Period, Z, R | **[LIVE]**, both may exist |
| C8 | Sections on the web | Sections page: add from Slide Sorter | Nothing states the rail's menu lacks it | **[LIVE]** |

Sources for §0.1: [behaviour](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint), [compare](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b), [find](https://support.microsoft.com/en-us/office/find-and-replace-text-b64c77c2-f5a1-4b78-9f03-50ec050b2174), [shortcuts](https://support.microsoft.com/en-us/accessibility/powerpoint/use-keyboard-shortcuts-to-create-powerpoint-presentations), [Simplified](https://support.microsoft.com/en-us/office/use-the-simplified-ribbon-in-powerpoint-for-the-web-f05d2d91-ad15-44b0-b401-e85735b409cb), [sections](https://support.microsoft.com/en-us/powerpoint/training/organize-your-powerpoint-slides-into-sections). Checked 2026-11-07.

### 0.2 Corrections this note makes to docs/47

docs/47 warns that it was written from recall. These lines disagree with what Microsoft documents. The
file is not edited here; the owner of `UX-001` should carry them over.

| docs/47 says | Documented | Effect |
| --- | --- | --- |
| "Ctrl+[ and ] font size" | The Microsoft tables give Ctrl+Shift+> and < for font size on both platforms. On the **web**, Ctrl+[ and ] move an object one layer ([D]) | **Binding Ctrl+[ to font size breaks web z-order** |
| "Ctrl+Shift+C and V format painter" | Desktop only. On the web table Ctrl+Shift+V is **paste as plain text** ([D]) | One key, two meanings: choose per §41 rule 14 |
| "number + Enter jump" in show | Desktop. The web table gives G, arrows, Enter ([D]) | Support both |
| Silent on the region-cycle key | Desktop F6; web Ctrl+F6 ([D]) | Add it. See §37 |
| Silent on how a shape is placed | Desktop drags; web inserts a default-size shape in the middle of the view, without a drag ([D]) | Add it. See §13 |

## 1. Layout regions and proportions

### Regions

Normal view has three working areas: thumbnail pane, slide pane, notes pane ([D]). Notes sit under the
slide and toggle from a status-bar button ([D]). Thumbnails sit on the left ([D]).

```
┌ Title bar: AutoSave · QAT · file name · Search · presence · Comments · Share ┐
├ Tab strip ───────────────────────────────────────────────────────────────────┤
├ Ribbon body (classic: three rows plus group captions; Simplified: one row) ──┤
│ Thumbnail │                                                       │ Task     │
│ rail      │            Slide canvas, fit to window                │ pane     │
│           ├───────────────────────────────────────────────────────┤ (right)  │
│           │ Notes pane (hidden or shown)                          │          │
├ Status: slide count · language · Notes · views · zoom · fit ─────────────────┤
└──────────────────────────────────────────────────────────────────────────────┘
```

The diagram's arrangement is [D] for rail, slide, notes, status views and task panes on the right, and
[I-recall] for the exact left-to-right order in the title bar and the status bar.

| Region | Facts | Mark |
| --- | --- | --- |
| Title bar, desktop | AutoSave at upper left, for subscribers | [D] |
| Title bar, desktop | Clicking the file name opens a menu containing Version History | [D] |
| Title bar, desktop | Presence avatars at top right; Share at top right of the ribbon area | [D] |
| Title bar, desktop | QAT hidden by default when it holds only default commands such as Undo and Save, per the 2021 refresh post (updated 2024) | [D\*] |
| Title bar, web | File name in the centre of the title bar; click to rename | [D] |
| Title bar, web | Search field at the top; Comments button near the top right | [D] |
| Ribbon, web | Simplified and classic both exist; Alt+Period, Z, R switches | [D] |
| Ribbon, desktop | Ribbon Display Options includes Always show Ribbon | [D\*] |
| Tabs, desktop | File, Home, Insert, Draw, Design, Transitions, Animations, Slide Show, Review, View, Recording, Help (access keys in §37) | [D] |
| Status bar | Holds the Notes button and the view buttons; the web sorter button is here too | [D] |
| Task panes | Open on the right: Accessibility, Design Suggestions, Copilot, Animation | [D] |
| Default ribbon on the web | Simplified is the default | [I-recall], [LIVE] (docs/47 repeats this from recall) |

### Proportions: all estimates

**Not one number in this table is measured.** Heights are copied in shape from OpenCalc's Excel
estimates (docs/91 §1.2) and from memory. Nothing was sourced for PowerPoint. Use these only to decide
what to measure.

| Region | [est] size at 1440 × 900 | Basis |
| --- | --- | --- |
| Title bar | 32–48 px | docs/91 Excel [est] and recall |
| Tab strip | 28–36 px | docs/91 Excel [est] |
| Classic ribbon body | 88–100 px | docs/91 Excel [est] |
| Simplified ribbon body | 36–44 px | recall |
| **Above the canvas, classic** | 148–184 px | sum of the above [est] |
| **Above the canvas, Simplified** | 96–128 px | sum of the above [est] |
| Status bar | 22–28 px | docs/91 Excel [est] |
| Thumbnail rail | 12–20 % of width, user-resizable | recall |
| Notes pane | 0 when hidden, else 10–20 % of working height, user-resizable | recall |
| Task pane | 280–360 px | recall |

**Fluent 2 tokens, which are real but are not PowerPoint measurements** ([D], design system only):

- Spacing is built on a 4 px base with 2, 6 and 10 added for icon padding. The ramp is 0, 2, 4, 6, 8,
  10, 12, 16, 20, 24, 28, 32, 36, 40, 48, 52, 56.
- Corner radius: None 0 (nav and tab bars), Small 2, Medium 4 (buttons, dropdowns), Large 8, X-Large 12
  (popovers), Circle 50 % (personas).
- Stroke on web: 1, 2, 3, 4 px.
- Breakpoints: 320–479, 480–639, 640–1023, 1024–1365, 1366–1919, 1920+.
- Minimum touch target on web: 44 × 44.

Fluent does not publish a ribbon or control-height spec on these pages. Whether PowerPoint uses these
values exactly is [LIVE].

**Near-term change.** The roadmap lists a slide-size task pane in November 2026 (573149, 573150), so
some dialogs are turning into panes. Read through a third-party roadmap digest ([D\*]).

Sources: [views](https://support.microsoft.com/en-us/powerpoint/training/choose-the-right-view-for-the-task-in-powerpoint), [AutoSave](https://support.microsoft.com/en-us/office/collab-files/what-is-autosave), [versions](https://support.microsoft.com/en-us/office/view-previous-versions-of-office-files-5c1e076f-a9c9-41b8-8ace-f77b9642e2c2), [co-authoring](https://support.microsoft.com/en-us/office/work-together-on-powerpoint-presentations-0c30ee3f-8674-4f0e-97be-89cf2892a34d), [shortcuts](https://support.microsoft.com/en-us/accessibility/powerpoint/use-keyboard-shortcuts-to-create-powerpoint-presentations), [new look](https://support.microsoft.com/en-us/office/foundations-experiences/the-new-look-of-office), [visual refresh](https://techcommunity.microsoft.com/blog/microsoft365insiderblog/visual-refresh-of-office-apps-for-windows/4213124), [Fluent layout](https://fluent2.microsoft.design/layout), [Fluent shapes](https://fluent2.microsoft.design/shapes), [roadmap digest](https://go-planet.com/in-the-know/october-29-2026/). Checked 2026-11-07.

## 2. Start page and templates

**Web** ([D])

1. Go to the PowerPoint web address, or use the Microsoft 365 app launcher.
2. Choose among: New blank presentation, a Recent file, a theme, or a presentation template.
3. Typing `ppt.new` or `PowerPoint.new` in the address bar creates a deck directly.

A Copilot "create" option was added to the web home page from mid-May 2026, on by default for licensed
users ([D\*], message-center mirror). Its position on the page is not stated.

**Desktop.** File (Backstage) has a home area with New, including a Blank Presentation tile ([D\*],
Microsoft Q&A). The File page is a separate full window state; Alt+F opens it and Esc returns ([D]).

**Not documented:** tile sizes, how many themes appear, the search box, pinning, whether Recent has
tabs. All [LIVE]. "Microsoft Create" as a template source was searched for and nothing usable was found.

**For us:** templates are our own authored content, so only the structure of this page is studied.

Sources: [create on web](https://support.microsoft.com/en-au/office/21360025-7eef-4173-9d7c-08281d55f64a), [get started](https://support.microsoft.com/en-gb/office/get-started-with-powerpoint-for-the-web-6303da7a-402a-4300-8b1c-160e8940cc34), [shortcuts](https://support.microsoft.com/en-us/accessibility/powerpoint/use-keyboard-shortcuts-to-create-powerpoint-presentations), [MC1311972 mirror](https://mc.merill.net/message/MC1311972), [Q&A](https://learn.microsoft.com/en-us/answers/questions/1536882/replace-powerpoint-blank-presentation-with-custom), [screen reader](https://support.microsoft.com/en-us/office/use-a-screen-reader-to-explore-and-navigate-powerpoint-a11115c7-6038-44a9-b355-8e133a4e9594). Checked 2026-11-07.

## 3. New deck and first run

**Web** ([D]):

1. Choosing blank creates and saves the file at once, to OneDrive, under a default name like
   "Presentation 1". There is no Save dialog.
2. Rename by clicking the title and typing.
3. Saving then runs every few seconds with no action. There is no Save button.

**Desktop:** AutoSave is off until the file lives in cloud storage ([D]). Ctrl+N makes a new deck ([D]).

**First slide:** a title slide with two text boxes, title and subtitle ([O], OpenStax/LibreTexts,
excerpt). The prompt wording "Click to add title" is [I-recall].

**Design Suggestions on a new deck.** Designer can open automatically on a new presentation, controlled
by a File > Options setting ([D]). Adding text to a blank deck's title slide prompts photo suggestions
([D]). We exclude this, per `DEC-05`.

**Not documented:** whether coach marks or a tour appear. [LIVE].

Sources: [get started](https://support.microsoft.com/en-gb/office/get-started-with-powerpoint-for-the-web-6303da7a-402a-4300-8b1c-160e8940cc34), [create on web](https://support.microsoft.com/en-au/office/21360025-7eef-4173-9d7c-08281d55f64a), [AutoSave](https://support.microsoft.com/en-us/office/collab-files/what-is-autosave), [Designer](https://support.microsoft.com/en-us/powerpoint/create-professional-slide-layouts-with-designer), [LibreTexts](https://workforce.libretexts.org/Bookshelves/Information_Technology/Computer_Applications/Workplace_Software_and_Skills_(OpenStax)/06%3A_Preparing_Presentations/6.02%3A_Designing_a_Presentation_in_Microsoft_PowerPoint). Checked 2026-11-07.

## 4. Thumbnail rail

**Select and move through** ([D])

- Click selects. Page Down and Page Up move through slides when focus is in the rail or slide pane.
- Up and Down arrows move between thumbnails when the rail has focus.
- Ctrl+A selects all slides in the rail or sorter (desktop).

**Multi-select** ([D]): Ctrl+click for separate slides, Shift+click for a run, on Windows and web. Cmd on
Mac.

**Reorder** ([D])

1. Drag one thumbnail, or Ctrl-select several and drag them as a group.
2. Keyboard: Ctrl+Up and Ctrl+Down move one place. Ctrl+Shift+Up and Ctrl+Shift+Down move to the start
   or end. This is the same on both platforms.

The drop indicator's look, and auto-scroll while dragging, are [LIVE].

**Add, duplicate, delete** ([D])

| Action | How |
| --- | --- |
| New slide | Select the slide it should follow, then Home > New Slide. The arrow opens the layout gallery. Ctrl+M with focus in rail, slide or notes |
| Duplicate | Right-click > Duplicate Slide; the copy lands right after the original. Ctrl+Shift+D on desktop |
| Delete | Right-click > Delete Slide, with a selection of one or many. Delete key on the web |
| Link | Right-click > Link to this Slide opens a sharing card with Copy (desktop). The link opens the whole deck at that slide |

Enter in the rail adding a slide is [I-recall].

**Sections**

| Step | Windows | Web |
| --- | --- | --- |
| Add | Right-click between slides > Add Section; an Untitled Section appears and a Rename dialog opens ([D]) | Sorter view: select the first slide, right-click, Add Section, type, Enter ([D]). See C8 |
| Collapse | Triangle beside the name; a numeral shows the slide count ([D]) | [LIVE] |
| Move | Sorter: drag the section title ([D]). Mac also in Normal ([D]) | [LIVE] |
| Rename, remove | Delete by right-click ([D]). Mac: double-click the name ([D]) | Select, right-click ([D]) |

Ctrl+Up and Ctrl+Down also move a selected **section** on desktop ([D]). Section names show in the rail
and sorter ([D\*]) and not in the show ([D\*]).

**Hide** ([D])

1. Right-click the slide > Hide Slide. Right-click > Unhide Slide reverses it.
2. Windows and web mark the slide with a slash through its number. Mac uses a "no" symbol.
3. The slide stays in the file and is skipped in the show.
4. To show one while presenting: Windows, right-click > See All Slides. Web, the grid icon in the
   lower-left control bar. Mac, H from the previous slide, or the Presenter navigation pane.

**Indicators on a thumbnail**

- A transition icon marks a slide with a transition (read on one platform tab; others [LIVE]) ([D]).
- A co-author's identity icon; hover to see who ([D]).
- A blue dot in the margin marks slides changed by others since you last looked ([D]).
- Comment markers on the thumbnail are [LIVE].

**Context menu.** Documented items are Hide or Unhide, Duplicate, Delete, Add Section, Link to this
Slide. The rest of the menu (New Slide, Layout, Reset, Format Background, Paste Options) is [I-recall].
**The full menu and its order need a live capture on each platform.**

Sources: [add and move slides](https://support.microsoft.com/en-us/powerpoint/training/add-rearrange-duplicate-and-delete-slides-in-powerpoint), [sections](https://support.microsoft.com/en-us/powerpoint/training/organize-your-powerpoint-slides-into-sections), [hide](https://support.microsoft.com/en-us/office/hide-or-show-a-slide-8313e1ec-3e20-4464-952f-387931554d69), [link to slide](https://support.microsoft.com/en-gb/powerpoint/share-a-link-to-a-specific-slide), [shortcuts](https://support.microsoft.com/en-us/accessibility/powerpoint/use-keyboard-shortcuts-to-create-powerpoint-presentations), [transitions](https://support.microsoft.com/en-us/powerpoint/training/add-change-or-remove-transitions-between-slides), [co-authoring](https://support.microsoft.com/en-us/office/work-together-on-powerpoint-presentations-0c30ee3f-8674-4f0e-97be-89cf2892a34d). Checked 2026-11-07.

## 5. Canvas: fit, zoom, pan

| Action | Desktop | Web |
| --- | --- | --- |
| Zoom in, out | Ctrl+Plus, Ctrl+Minus ([D]) | View > Zoom ([D]). Shortcut not listed |
| Fit | Ctrl+Alt+O ([D]) | Not listed; [LIVE] |
| Zoom dialog | Alt+W, Q ([D]) | Not listed |
| Status-bar slider and fit button | Yes ([I-recall]) | [LIVE] |

**Fit is the default state and re-fits on window or pane resize** ([I-recall]). The slide is on a neutral
workspace ([I-recall]).

**Pan and scroll.** Microsoft documents nothing here. All of this is [I-recall] and [LIVE]:

- At fit, the wheel or scrollbar moves to the next or previous slide.
- Above fit, the wheel pans, and only at the slide's edge does it change slide.
- Ctrl+wheel zooms about the pointer.
- Hand or space-drag panning exists or does not (unknown).

**Touch zoom** is documented: spread and pinch on Android, iPhone, iPad ([D]).

Sources: [shortcuts](https://support.microsoft.com/en-us/accessibility/powerpoint/use-keyboard-shortcuts-to-create-powerpoint-presentations), [behaviour](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint), [Android](https://support.microsoft.com/en-us/office/powerpoint-for-android-touch-guide-7f2b68df-608f-428b-958b-5a165fb109ce). Checked 2026-11-07.

## 6. Canvas: selection, handles, rotation, adjust

**Handles**

- Rotation handle at the top of the object ([D]).
- Sizing handles at corners and sides; eight is [I-recall], and handle shape and size are [LIVE].
- Some shapes add yellow adjustment handles. A hexagon has one and a U-turn arrow five; rectangles and
  lines may have none ([O], Indezine, 2023). Dragging one changes the shape's geometry, not its box.
- A pie's dot alters how complete the circle looks ([O]).

**Drag modifiers**

| Gesture | Effect | Mark |
| --- | --- | --- |
| Shift + corner handle | Keep proportions; also on web | [D] |
| Ctrl + handle | Resize from centre | [O], old AutoShape-era sources |
| Ctrl+Shift + handle | Both | [O] |
| Shift + rotation handle | Snap to 15° steps | [D] |
| Shift + drag object | Constrain to axis | [I-recall] |
| Alt + drag | Override snapping; desktop | [D] |
| Cmd + drag (Mac) | Fine placement; hides dynamic guides | [D] |

**Rotation** ([D])

- Exact angle: Format > Arrange > Rotate > More Rotation Options.
- Rotate Left 90°, Rotate Right 90°, Flip Horizontal, Flip Vertical are in the same menu.
- Flipping does not mirror text.
- **With several objects selected, each rotates about its own centre, not as a block.**
- Alt+Right and Alt+Left step 15° on Windows and web; Option on Mac.
- Web keyboard route: Ctrl+F6 to the slide, Tab to the object, Shift+F10, Format options, enter the angle.
- Shift+arrow resize and Ctrl+Alt fine rotate ([D\*], office-watch excerpt only; [LIVE]).

**Selected versus editing border**

- Object selected: a solid outline ([O], Indezine, 2015; The PowerPoint Blog, 2009).
- Text being edited: a dashed outline with handles still present ([O], same).

The current appearance is [LIVE]. Both sources are old, and the visual refresh may have changed the
drawing.

Sources: [rotate](https://support.microsoft.com/en-us/office/graphics-visuals/rotate-or-flip-a-text-box-shape-wordart-or-picture), [add shapes](https://support.microsoft.com/en-us/powerpoint/add-shapes?ui=en-us&rs=en-us&ad=us), [grid](https://support.microsoft.com/en-us/powerpoint/work-with-gridlines-and-use-snap-to-grid-in-powerpoint), [align](https://support.microsoft.com/en-us/office/graphics-visuals/align-or-arrange-objects), [Indezine yellow handles](https://www.indezine.com/products/powerpoint/learn/shapes/365/manipulate-shapes-with-yellow-round.html), [Indezine insert shapes](https://www.indezine.com/products/powerpoint/learn/shapes/2016/insert-shapes.html), [Indezine selecting text](https://www.indezine.com/products/powerpoint/learn/textandfonts/2013/selecting-changing-text.html), [PowerPoint Blog](https://thepowerpointblog.com/?p=2477), [office-watch](https://office-watch.com/2023/powerpoint-rotate-resize-arrow-key/). Checked 2026-11-07.

## 7. Canvas: guides, snapping, gridlines

**Smart guides** ([D])

- They appear only during a move, as red dashed lines showing alignment with other objects.
- They also appear between objects and near slide edges to show even spacing.
- On Mac the lines run through other objects' centres and the slide centre, and View > Grid and Guides >
  Smart Guides toggles them.
- The web has them too, appearing automatically ([D]). A web off switch is [LIVE].

**Grid** ([D])

- View > Gridlines shows it. The major squares are one inch and cannot be changed.
- Spacing changes the distance between the grid's dots.
- Grid and Guides dialog, from the launcher on View > Show: Snap objects to grid, Display grid on screen,
  Display drawing guides, Display smart guides, Set as Default.
- Shift+F9 toggles grid and Alt+F9 toggles guides.

**Drawing guides** ([D])

- A dotted pair crossing at slide centre. Objects snap when dragged within a few pixels.
- While dragging a guide, its distance from centre shows by the pointer.
- Guides can be added, deleted and recoloured (right-click > Color).
- They do not print or show in the slide show.

**Web:** rulers and gridlines are stated to be desktop-only ([D]; the page is partly stale, so [LIVE]).

**Align and distribute** ([D])

- Align Left, Center, Right, Top, Middle, Bottom. Greyed when one item is selected.
- Distribute needs at least three objects.
- Align to Slide and Align Selected Objects are modes in the menu, on Mac and web.
- Desktop: on the object's Format tab. Web: on the Shape or Picture tab.

**Not documented:** snap distance in pixels, snap priority between guides and objects, equal-spacing
marker design, colour on modern builds. All [LIVE].

Sources: [grid](https://support.microsoft.com/en-us/powerpoint/work-with-gridlines-and-use-snap-to-grid-in-powerpoint), [guides](https://support.microsoft.com/en-us/office/guides-for-arranging-things-on-a-slide-33854dfa-e0d1-43ff-8971-667b19512de3), [align](https://support.microsoft.com/en-us/office/graphics-visuals/align-or-arrange-objects), [behaviour](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint), [shortcuts](https://support.microsoft.com/en-us/accessibility/powerpoint/use-keyboard-shortcuts-to-create-powerpoint-presentations). Checked 2026-11-07.

## 8. Canvas: marquee, tab order, nudge, duplicate, group

**Marquee and multi-select**

- Web: drag, Shift+drag or Ctrl+drag selects several; Ctrl+click or Shift+click adds ([D]).
- Desktop: Ctrl-click to add ([D]). Home > Select > Select Objects then draw a box reaches objects that
  are hidden, stacked or behind text ([D]).
- **Whether a marquee must wholly enclose an object** is [I-recall] (yes) and [LIVE].

**Keyboard selection and tab order** ([D])

1. Ctrl+Alt+5 moves focus to the first floating shape (desktop). Esc returns to normal navigation.
2. Tab and Shift+Tab move between objects; Shift+Tab after the top object cycles backward down the stack.
3. Web: Tab moves through placeholders left to right.
4. Ctrl+A selects all objects, or all text if editing.
5. For keyboard multi-select, Microsoft says to use the Selection pane.

**Whether Tab order follows z-order** is [I], reasoned from two facts: the Selection pane lists stack
order, and changing Reading Order can change how overlaps look. [LIVE].

**Selection pane** ([D])

- Open: desktop Home > Editing > Select > Selection Pane, or Alt+F10. Web: Home or Shape > Arrange.
- Top of the list is top of the stack; the newest object is at the top.
- Drag rows or use Bring Forward and Send Backward. An object in a group stays in that group.
- A row icon hides or shows (the icon was lost in extraction; an eye is [I-recall]). Lock is on desktop and Mac; web documents only show, hide, reorder and group.
- Rename: click a selected row again, or F2. Double-click also renames, per the page's tips.
- Keys: Ctrl+Shift+S visibility, Ctrl+Shift+F forward, Ctrl+Shift+B back, Ctrl+Shift+L lock, Alt+Shift+1
  and 9 collapse and expand. Multi-add: Shift+Enter or Shift+Space on desktop, Ctrl+Enter or Ctrl+Space
  on web.
- Web limit: objects inside groups cannot be selected together with objects outside groups.

**Nudge**

- Arrow keys move a selected shape, on both platforms ([D]). Ctrl+arrow is finer, and snap state affects
  the step ([D], Microsoft employee blog, **2004**).
- 6 pt with snap on and 1.25 pt with snap off or with Ctrl ([D\*]: office-watch excerpts, 2023; the page
  itself was not readable). [LIVE]: verify this and the web step.

**Duplicate** ([D])

- Ctrl+D, or Ctrl+drag on desktop. Ctrl+D on web, Cmd+D on Mac, Option+drag on Mac.
- See C4. Browsers normally reserve Ctrl+D, so whether the web app wins it is [LIVE].
- Duplicate offset, and repeated Ctrl+D continuing the last offset, are [I-recall] and [LIVE].

**Layer order** ([D]). Platforms differ, and this is a trap:

| | Windows desktop | Web (Windows) |
| --- | --- | --- |
| Back one | Ctrl+Shift+[ | Ctrl+[ |
| Forward one | Ctrl+Shift+] | Ctrl+] |
| To back | not listed | Ctrl+Shift+[ |
| To front | not listed | Ctrl+Shift+] |

**Group** ([D])

- Ctrl+G, Ctrl+Shift+G ungroup, Ctrl+Shift+J regroup on desktop. Web: Arrange > Group. The web table
  lists no group shortcut.
- A grouped set moves and formats as one.
- **Grouping loses animations on the members** (reading-order page caution).
- Web: Ctrl+click to select members for grouping.

**Entering a group.** Selecting a group then clicking a child to select just it is [I-recall]. It is not
in any page read; the only related fact is that multi-selection inside a group is allowed. [LIVE],
including whether Esc steps from child to group.

Sources: [shortcuts](https://support.microsoft.com/en-us/accessibility/powerpoint/use-keyboard-shortcuts-to-create-powerpoint-presentations), [shape shortcuts](https://support.microsoft.com/en-us/office/keyboard-shortcuts-for-working-with-shapes-text-boxes-and-wordart), [selection pane](https://support.microsoft.com/en-us/powerpoint/use-the-selection-pane-to-manage-objects-in-documents), [align](https://support.microsoft.com/en-us/office/graphics-visuals/align-or-arrange-objects), [reading order](https://support.microsoft.com/en-us/office/make-slides-easier-to-read-by-using-the-reading-order-pane-863b5c1c-4f19-45ec-96e6-93a6457f5e1c), [nudging blog](https://learn.microsoft.com/en-us/archive/blogs/waynekao/nudging), [office-watch](https://office-watch.com/2023/powerpoint-positioning-nudge), [screen reader](https://support.microsoft.com/en-us/office/use-a-screen-reader-to-explore-and-navigate-powerpoint-a11115c7-6038-44a9-b355-8e133a4e9594). Checked 2026-11-07.

## 9. Placeholders, prompts and layouts

**What a placeholder is** ([D])

- A dotted box on the slide placed by the layout.
- A Content placeholder accepts text, table, chart, SmartArt, picture or video, shown as icons in its centre.
- A title placeholder takes the theme's heading font, size and colour.
- Prompt text is edited in Slide Master view, on the layout (desktop).
- Title and Footers check boxes on a layout control whether those exist. Footers still need Insert >
  Header and Footer to appear on slides.

**Layouts** ([D])

1. Home > Layout (or Slide Layout), or the arrow on New Slide.
2. The set comes from the theme and includes Title, Title and Content, Comparison, Picture with Caption.
3. Home > Reset restores the layout's positions and formatting without deleting content.
4. Web: choose a layout when adding a slide or change it afterwards. Masters cannot be edited on the web.

**Empty-state behaviour.** All [I-recall], all [LIVE]:

- Prompts do not appear in the show, in thumbnails, or in print.
- Clicking prompt text enters editing and the prompt disappears.
- Clicking a content icon opens that insert flow, and the new object takes the placeholder's box.
- Deleting a filled placeholder empties it first; a second delete removes the empty box.
- Changing a layout keeps content and moves it to matching placeholders.
- A new slide inherits the current slide's layout, and after a title slide it becomes Title and Content.

**Ctrl+Enter** moves to the next title or body placeholder, and from the last one creates a slide with
the same layout ([D], desktop). It is not on the web table.

Sources: [placeholders](https://support.microsoft.com/en-us/powerpoint/add-edit-or-remove-a-placeholder-on-a-slide-layout), [AutoFit and split](https://support.microsoft.com/en-us/powerpoint/split-body-text-between-two-slides), [apply layout](https://support.microsoft.com/en-us/office/apply-a-slide-layout-158e6dba-e53e-479b-a6fc-caab72609689), [add slides](https://support.microsoft.com/en-us/powerpoint/training/add-rearrange-duplicate-and-delete-slides-in-powerpoint), [behaviour](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint), [shortcuts](https://support.microsoft.com/en-us/accessibility/powerpoint/use-keyboard-shortcuts-to-create-powerpoint-presentations). Checked 2026-11-07.

## 10. Text editing

**Two selection levels** ([O] for the click rules; [D] for the keys)

| Gesture | Result | Mark |
| --- | --- | --- |
| Click in text | Caret in text; dashed border | [O] |
| Click the object's edge | Whole object; solid border | [O] |
| Shift+click anywhere on it | Whole object, never text | [O], 2009 |
| Drag across text | Selects text | [O] |
| Double-click a word | Selects the word | [O] |
| Triple-click | Sentence or bulleted level per Indezine; paragraph per the iPhone guide | [O], [D] |
| Enter, object selected | Into the text | [D] |
| **Esc, text selected** | **Back to the object** | [D] |
| F2 on web | Enter editing, and F2 again ends it | [D] |
| Second Esc | Deselects | [I-recall] |
| Double-click a shape | Enters text, or opens formatting | [I-recall], [LIVE] |
| Type, shape selected | Starts its text ("just start typing") | [D] |
| Type, object already has text | Replaces or appends? | [LIVE] |

Tab inside text does not move to another object while editing ([I]; Tab indents, below).

**Object click in a large filled shape.** Whether clicking a filled body selects text or the object is
[LIVE]. Our canvas hit-test depends on it.

**Autofit** ([D])

1. An AutoFit Options control appears at the lower left of a **placeholder**, only once it starts to
   overflow. It does not appear on drawn text boxes.
2. Menu: AutoFit Text (the default; shrinks font), Stop Fitting Text, Split Text Between Two Slides,
   Continue on a New Slide, Change to Two Columns.
3. Split and Continue make a new slide with the same title right after. Split divides the text roughly in half.
4. Format pane, Text Box section (page is 2010-era, [D]): Do not Autofit, Shrink text on overflow,
   Resize shape to fit text, internal margins, Wrap text, Columns.
5. **Web has no automated split** ([D]). Other web AutoFit controls are [LIVE].

Default fit on a drawn text box being "resize shape to fit text" is [I-recall].

**Mini toolbar**

- A floating toolbar on right-click, used for table Insert and Delete on both platforms ([D\*]).
- A floating toolbar on an object offers New Comment ([D\*]).
- Appearing on text selection is [I-recall], as is a File > Options toggle for it. [LIVE] on each platform,
  with exact contents.

**Lists** ([D])

- Bullets and Numbering are on Home > Paragraph. Enter makes the next item.
- **Tab and Shift+Tab change level only with the caret at the start of the paragraph** (web page).
- Increase List Level does the same. Desktop also has Alt+Shift+Right and Left, and Alt+Shift+Up and
  Down to move paragraphs.
- Auto-format triggers on desktop: `*` then Space for round bullets; `-`, `>`, `<>`, `->`, `=>` for other
  styles.

**Other keys** ([D]): Ctrl+B, I, U; Ctrl+L, E, R, J on Windows (Mac differs on L and R); Ctrl+Shift+> and <
size; Ctrl+Space clear formatting; Ctrl+T or Ctrl+Shift+F Font dialog (desktop); Shift+F3 case (desktop);
Ctrl+= subscript and Ctrl+Shift++ superscript (desktop).

**Paste.** On the web, pasted text keeps its source formatting and Ctrl+Shift+V pastes it plain ([D]).
Ctrl+Alt+V opens Paste Special on desktop ([D]).

**Spell check**

- Red squiggle and right-click suggestions on the web ([D]). Proofing language can be set ([D]).
- F7 for the spelling pane on desktop ([O], third-party lists, and listed in tutorials not read in full).
- The ribbon's Review > Spelling entry on each platform is [LIVE].

Sources: [AutoFit and split](https://support.microsoft.com/en-us/powerpoint/split-body-text-between-two-slides), [Text Box pane](https://support.microsoft.com/en-us/topic/format-object-text-box-pane-e5574edd-71b5-4ea6-88b7-b74b513b33ae), [bullets](https://support.microsoft.com/en-us/powerpoint/add-text-bullets-and-sub-bullets), [shortcuts](https://support.microsoft.com/en-us/accessibility/powerpoint/use-keyboard-shortcuts-to-create-powerpoint-presentations), [add shapes](https://support.microsoft.com/en-us/powerpoint/add-shapes?ui=en-us&rs=en-us&ad=us), [table video](https://support.microsoft.com/en-us/office/video-add-a-table-to-a-slide-33b3491e-f0ec-48c4-b9fb-3a5bd8797608), [modern comments](https://support.microsoft.com/en-us/powerpoint/modern-comments-in-powerpoint), [behaviour](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint), [iPhone](https://support.microsoft.com/en-us/office/powerpoint-for-iphone-touch-guide-39499f20-6f6b-4de3-8cf0-173923c8bf7f), [Indezine](https://www.indezine.com/products/powerpoint/learn/textandfonts/2013/selecting-changing-text.html), [PowerPoint Blog](https://thepowerpointblog.com/?p=2477), [Computer Hope](https://www.computerhope.com/shortcut/powerpoint.htm). Checked 2026-11-07.

## 11. Format Painter

**Mouse** ([D]; the page covers Office generally)

1. Select the source text or object.
2. Home > Clipboard > Format Painter. The pointer becomes a brush.
3. Paint a target. One use, then it turns off.
4. **Double-click the button to keep it on**, and Esc ends it.

Selecting part of a paragraph copies text formatting. Selecting the whole paragraph, with its mark, also
copies paragraph formatting ([D]; the page's example is Word, and PowerPoint is [I]).

**Keyboard**: Ctrl+Shift+C and Ctrl+Shift+V (desktop) ([D]). The same pair on Word-family apps may be
Ctrl+Alt+C and V, per the general page.

**Web:** Format Painter is available per the behaviour page ([D]). Its keyboard route is not in the web
table, where Ctrl+Shift+V is plain paste.

**Animation Painter:** Alt+Shift+C and Alt+Shift+V (desktop) ([D]).

**Which properties are copied** (fill, line, effect, font) for PowerPoint specifically is [LIVE].

Sources: [Format Painter](https://support.microsoft.com/en-us/office/use-the-format-painter-4bb415a9-d4e4-42b7-b579-170adc594e40), [shortcuts](https://support.microsoft.com/en-us/accessibility/powerpoint/use-keyboard-shortcuts-to-create-powerpoint-presentations), [behaviour](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint). Checked 2026-11-07.

## 12. Ribbon behaviour relevant to objects

**Contextual tabs differ by name** ([D]):

| Selected | Desktop | Web |
| --- | --- | --- |
| Shape, text box | Shape Format | **Shape** (Format, with Simplified off, per the add-shapes page) |
| Picture | Picture Format | Picture |
| SmartArt | SmartArt Design (a second Format tab is [I-recall]) | SmartArt |
| Cameo | Camera Format | [LIVE] |
| Video | Video Format, Playback | [LIVE] |

Merge Shapes is on Shape Format > Insert Shapes and greys out when nothing is selected ([D]).

**Web Simplified ribbon** ([D]): shows common commands; more live behind ellipsis buttons, some tabs have
several; a down arrow by a button lists variants; search finds anything missing.

Sources: [add shapes](https://support.microsoft.com/en-us/powerpoint/add-shapes?ui=en-us&rs=en-us&ad=us), [Simplified](https://support.microsoft.com/en-us/office/use-the-simplified-ribbon-in-powerpoint-for-the-web-f05d2d91-ad15-44b0-b401-e85735b409cb), [merge](https://support.microsoft.com/en-us/office/draw-a-picture-by-combining-and-merging-shapes-2c9ce121-7b28-46ed-aec9-eae20cdddb15), [cameo](https://support.microsoft.com/en-us/powerpoint/presenting-with-cameo), [video](https://support.microsoft.com/en-us/powerpoint/training/add-format-and-record-video-in-powerpoint). Checked 2026-11-07.

## 13. Shape insertion

**The largest flow difference between web and desktop**

| Step | Desktop | Web |
| --- | --- | --- |
| Choose | Insert > Shapes (Alt+N, S, H), pick from the gallery ([D]) | Insert > Shapes ([D]) |
| Place | **Drag on the slide** ([D]). A single click gives a default-size shape, about 1 in × 1 in ([O], Indezine) | **A default-size shape lands in the middle of the view, with no drag** ([D]) |
| Constrain | Shift while dragging: square, circle, or fixed proportion ([D]) | Shift on a handle afterwards ([D]) |
| From centre | Ctrl while dragging ([O]) | [LIVE] |
| Repeat | Right-click the gallery item > Lock Drawing Mode, then Esc ends ([D]) | [LIVE] |
| Add text | Right-click > Add or Edit Text, or just type ([D]) | Right-click > Edit Text; caret at centre ([D]) |
| Style | Shape Format ([D]) | Shape tab: Fill, Outline with Weight and Dashes, Styles gallery ([D]) |
| Change type | Edit Shape > Change Shape ([D]) | [LIVE] |

Text in a shape rotates and flips with it ([D]). Insert > Text Box is Alt+N, X on desktop ([D]). Its
click-versus-drag behaviour is [LIVE]. Pointer shape while placing is [I-recall] (crosshair).

**Whether the web has since added drag-to-draw** is [LIVE]. The page may be stale.

Sources: [add shapes](https://support.microsoft.com/en-us/powerpoint/add-shapes?ui=en-us&rs=en-us&ad=us), [Indezine](https://www.indezine.com/products/powerpoint/learn/shapes/2016/insert-shapes.html), [shortcuts](https://support.microsoft.com/en-us/accessibility/powerpoint/use-keyboard-shortcuts-to-create-powerpoint-presentations). Checked 2026-11-07.

## 14. Connectors

No Microsoft end-user article was found in this round. These come from a developer reference and one
recent tutorial.

1. Insert > Shapes, Lines section: straight, elbow, curved ([O], SlideModel, 2025).
2. Hovering a shape shows small connection points ([O]).
3. Drag from one point to a point on another shape ([O]).
4. Moving either shape reflows the connector and keeps it attached ([O]; [D\*] in the Office VBA docs).
5. Right-click to change the connector type ([O]).
6. Connection sites are numbered from the top, counter-clockwise ([D\*], VBA docs).

Attached-end colour, an elbow's yellow handle, and Reroute Connectors are [I-recall]. Web support and
behaviour are [LIVE].

Sources: [SlideModel](https://slidemodel.com/connect-lines-powerpoint/), [ConnectorFormat](https://learn.microsoft.com/lv-lv/office/VBA/api/powerpoint.connectorformat). Checked 2026-11-07.

## 15. Merge shapes

1. Ctrl-select two or more shapes (Shift on Mac) ([D]).
2. Shape Format > Insert Shapes > Merge Shapes ([D]).
3. Choose Union, Combine, Fragment, Intersect or Subtract ([D]).

Selection order may affect the options offered ([D]). Subtract removes the overlap from the first shape
selected ([D\*]). Edit Shape > Edit Points gives vertex editing: black points move, white squares bend
([D]).

**Web:** the page read has no web instructions. [LIVE].

Sources: [merge](https://support.microsoft.com/en-us/office/draw-a-picture-by-combining-and-merging-shapes-2c9ce121-7b28-46ed-aec9-eae20cdddb15). Checked 2026-11-07.

## 16. Crop

1. Select the picture, then Picture Format > Crop ([D]).
2. Black crop handles appear on edges and corners ([D]).
3. Side handle crops one side. Corner handle crops two. **Ctrl+side handle crops both parallel sides** ([D]).
4. Dragging outward adds a margin ([D]).
5. Drag the picture or crop rectangle to reframe ([D]).
6. **Esc or a click outside finishes** ([D]).

Variants: Crop to Shape and Aspect Ratio, under the arrow on the Crop button ([D]). Cropped areas stay in
the file until deleted, which is a privacy note ([D]). Shapes have no crop; Edit Points is the substitute
([D]). The web can crop, but advanced picture formatting is not there ([D]).

Sources: [crop](https://support.microsoft.com/en-us/office/graphics-visuals/crop-a-picture-in-office), [behaviour](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint). Checked 2026-11-07.

## 17. Alt text

1. Right-click > View Alt Text, or the object's Format tab > Alt Text ([D]).
2. The Alt Text pane takes one or two sentences ([D]).
3. Mark as decorative greys out the field ([D]).
4. For a whole chart or SmartArt, click its border, not a piece ([D]).
5. AI-generated text has an Approve check box, and File > Options > Accessibility controls it ([D]). Out
   of scope for us.

Sources: [alt text](https://support.microsoft.com/en-us/office/add-alternative-text-to-a-shape-picture-chart-smartart-graphic-or-other-object-44989b2a-903c-4d9a-b742-6a75b451c669). Checked 2026-11-07.

## 18. Format panes

- Format Shape is a right-hand pane, with Fill and Line, Effects, Size and Properties, and Text Box areas
  ([D\*] for the areas named; the full tab list is [I-recall]).
- Shape Effects opens it on a tutorial's account ([O], GCF).
- **Edits apply immediately and there is no Cancel; Undo reverts** ([D], a 2010-era page, still likely true [I]).
- Web: right-click > Format options reaches an angle field; other fields are [LIVE] ([D]).
- Mac: "Format Pane" ([D]).
- Keyboard ([D]): F6 reaches a pane, Tab moves inside it, and Ctrl+Space opens the pane menu with Move,
  Size and Close (Ctrl+Space, C closes).
- Web keyboard: Ctrl+F6 reaches regions; Tab moves inside.

**Can more than one pane be open at once?** [LIVE]. It decides our pane model.

Sources: [Text Box pane](https://support.microsoft.com/en-us/topic/format-object-text-box-pane-e5574edd-71b5-4ea6-88b7-b74b513b33ae), [rotate](https://support.microsoft.com/en-us/office/graphics-visuals/rotate-or-flip-a-text-box-shape-wordart-or-picture), [shortcuts](https://support.microsoft.com/en-us/accessibility/powerpoint/use-keyboard-shortcuts-to-create-powerpoint-presentations), [GCF](https://edu.gcfglobal.org/en/powerpoint/shapes/1). Checked 2026-11-07.

## 19. Tables

| Step | Desktop | Web |
| --- | --- | --- |
| Insert | Insert > Table; hover the grid and click, or Insert Table dialog with row and column counts ([D]) | Grid in the drop-down only ([D]) |
| Dialog default | 5 columns, 2 rows; caret lands in the top-left cell ([D\*]) | no dialog |
| Add, delete | Right-click a cell, Insert or Delete on the mini toolbar ([D]) | same ([D]) |
| Excel | Insert > Table > Excel Spreadsheet: embedded, double-click to edit, ignores theme ([D]) | not mentioned |
| Finish | Click outside ([D]) | same ([D]) |

Keys ([D], both): Tab next cell, Shift+Tab previous, arrows by row, Enter new paragraph. **Tab in the last
cell adds a row.** Ctrl+Tab inserts a real tab (desktop). Column header rows are set on Table Design ([D\*]).
Slide text cannot be converted into a table ([D]).

Contextual tabs: Table Design and Layout on desktop ([D\*]); on web [LIVE]. Resize and border-drag are [LIVE].

Sources: [table video](https://support.microsoft.com/en-us/office/video-add-a-table-to-a-slide-33b3491e-f0ec-48c4-b9fb-3a5bd8797608), [screen reader tables](https://support.microsoft.com/en-us/office/use-a-screen-reader-to-insert-and-edit-pictures-and-tables-in-powerpoint-d14b4772-af53-4bb1-9111-00f1e239b029), [shortcuts](https://support.microsoft.com/en-us/accessibility/powerpoint/use-keyboard-shortcuts-to-create-powerpoint-presentations), [accessibility](https://support.microsoft.com/en-us/accessibility/powerpoint/make-your-powerpoint-presentations-accessible-to-people-with-disabilities). Checked 2026-11-07.

## 20. Charts

**Desktop** ([D\*])

1. Insert > Chart.
2. Pick a type, double-click a variant.
3. A worksheet opens with sample data; replace it.
4. Small buttons appear at the chart's upper right: Elements, Styles, Filters.
5. Close the worksheet.

A chart can also be pasted from Excel, optionally linked ([D\*]).

**Web:** see C2. Both Microsoft pages read say it cannot insert charts, and existing charts display.
docs/48 notes a contradicting source, and the author's recollection is that a web flow exists
([I-recall]). **[LIVE] before scope is set**.

The worksheet surface is a natural OpenCalc integration point ([I]). Not decided here.

Sources: [charts](https://support.microsoft.com/en-us/office/use-charts-and-graphs-in-your-presentation-c74616f1-a5b2-4a37-8695-fbcc043bf526), [compare](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b), [behaviour](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint). Checked 2026-11-07.

## 21. SmartArt

| Step | Desktop | Web |
| --- | --- | --- |
| Insert | Insert > SmartArt opens a dialog (Mac a menu) ([D\*]) | Dropdown of layouts, no dialog ([D\*]) |
| Text | Text pane, or click a box. An arrow on the graphic's left opens the pane ([D\*]) | A text editor opens on the left automatically; each bullet is a shape ([D\*]) |
| Add shape | SmartArt Design > Add Shape, Before or After ([D\*]) | Enter in the editor ([D\*]) |
| Remove | Delete on the shape ([D\*]) | Backspace over a bullet ([D\*]) |
| Levels | Demote; Shift+Tab promotes ([D\*]) | Promote and Demote on the SmartArt tab ([D\*]) |
| Style | Change Colors, SmartArt Styles ([D\*]) | same ([D\*]) |
| Delete all | Select the border, Delete ([D\*]) | [LIVE] |
| Reverse | [LIVE] | Right to Left ([D\*]) |

The compare table rates the web as Limited ([D]). Our position per docs/47: render first, edit later.

Sources: [SmartArt](https://support.microsoft.com/en-us/office/graphics-visuals/create-a-smartart-graphic-from-scratch), [compare](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b). Checked 2026-11-07.

## 22. Media

**Video on desktop** ([D\*])

1. Insert > Video, then This Device, Stock Videos, or Online Videos (YouTube or embed code).
2. Drag to size and place.
3. Video Format: frames, effects, Shape, Border, Corrections, Poster Frame.
4. Playback: captions from a file. Online video cannot take captions.
5. Insert > Screen Recording. Slide Show > Record. File > Export > Create a Video.

**Web:** see C3. Trim, bookmark and fade are not on the web ([D]). Media controls can be moved and sized
([D]). Size limits on the behaviour page look old, and the page should not be relied on ([I]).

**Audio:** compare table lists adding audio as desktop and Mac only (docs/48). Playing works on the web
([D]).

**Show keys** (desktop) ([D]): Alt+P or Ctrl+Space play or pause, Alt+Q stop, Alt+Up and Down volume, Alt+U
mute, Alt+Home and End bookmarks, Alt+Shift+PgDn and PgUp three-second skip. They do not apply to online
video.

Sources: [video](https://support.microsoft.com/en-us/powerpoint/training/add-format-and-record-video-in-powerpoint), [compare](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b), [behaviour](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint), [show shortcuts](https://support.microsoft.com/en-us/accessibility/powerpoint/use-keyboard-shortcuts-to-deliver-powerpoint-presentations). Checked 2026-11-07.

## 23. Ink

- The Draw tab is automatic on touch devices. Elsewhere, File > Options > Customize Ribbon ([D]).
- Choose a pen; the arrow beside it sets thickness (.25 to 3.5 mm in five settings) and colour ([D]).
- Pen sets carry across Word, Excel and PowerPoint ([D]).
- Eraser with several sizes, and Segment Eraser in PowerPoint ([D]).
- A Draw tool switches between inking and selection ([D]).
- Lasso selects ink only, never shapes or pictures; the selection is a faded dashed region ([D]).
- Some pens have an eraser end; a pen button can lasso ([D]).
- Ink to Shape converts drawn shapes (via the linked PowerPoint page) ([D\*]).
- A setting makes the pen select by default ([D]).

Web ink support is [LIVE]. In-show annotation is §26.

Sources: [ink](https://support.microsoft.com/en-us/office/draw-and-write-with-ink-in-office). Checked 2026-11-07.

## 24. Transitions

**Step by step** ([D])

1. Select a slide in the rail. Sorter works too ([I]).
2. Transitions tab: click an effect. **Selecting one previews it.**
3. Effect Options (web: Options) chooses direction.
4. Timing: Duration, Advance on click or after a time, Sound.
5. Preview replays. Apply To All copies to every slide.

**Semantics** ([D]): one transition per slide. It governs how that slide enters and the previous one exits.
To remove it, choose None; None then Apply To All clears the deck.

**Web** ([D]): a limited gallery. Desktop-only transitions are kept in the file and play only on desktop.
The web page has you see results with the Slide Show button in the status bar.

Morph is in the transitions gallery, and renaming objects in the Selection pane helps it ([D]). How it matches objects, and web support: [LIVE].

Sources: [transitions](https://support.microsoft.com/en-us/powerpoint/training/add-change-or-remove-transitions-between-slides), [web video](https://support.microsoft.com/en-us/powerpoint/training/video-animations-and-transitions-for-powerpoint-web), [behaviour](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint), [selection pane](https://support.microsoft.com/en-us/powerpoint/use-the-selection-pane-to-manage-objects-in-documents). Checked 2026-11-07.

## 25. Animations and the Animation Pane

**Add** ([D])

1. Select the object or text.
2. Animations tab: pick from the gallery. Web groups colour entrance green, emphasis yellow, exit red.
3. Effect Options sets direction or sequence.
4. A number appears beside the object, in play order. Effects at the same moment show a stack (read on the Mac tab).

**Second effect** ([D]): the gallery replaces; **Add Animation** adds. Web: Animation Pane > Add appends one
with a default such as Fade, named after the object, to then be changed.

**Timing** ([D]): Start is On Click, With Previous, or After Previous. Duration and Delay are in the same
group.

**Reorder** ([D]): Move Earlier and Move Later on the ribbon, or drag in the pane. In the pane **a thin red
line marks the drop point** (web tab). Ctrl+Z undoes. Arrow and delete controls are at the pane's top
([D\*]).

**Trigger** ([D]): Windows and Mac documented; select in the pane, then Trigger > On Click of > an object.
A trigger during media needs a bookmark in the clip first. Compare table: not on the web (docs/48).

**Web** ([D]): many effects; desktop-only ones are kept and play, but are not editable.

**Animation Painter:** §11. **Grouping loses animations:** §8. Mac's pane has Play From ([D]).

Pane row design and timeline bars are [LIVE].

Sources: [animate](https://support.microsoft.com/en-us/powerpoint/training/animate-text-or-objects), [multiple](https://support.microsoft.com/en-us/powerpoint/apply-multiple-animation-effects-to-one-object), [trigger](https://support.microsoft.com/en-us/powerpoint/trigger-an-animation-effect), [web video](https://support.microsoft.com/en-us/powerpoint/training/video-animations-and-transitions-for-powerpoint-web), [behaviour](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint). Checked 2026-11-07.

## 26. Slide show controls

**Start** ([D]): F5 from the start, Shift+F5 from current, Alt+F5 in Presenter on desktop; Slide Show > From
Beginning or From Current Slide; the status-bar Slide Show button. Web: Slide Show tab, or status bar.

**Keys**

| Action | Desktop ([D]) | Web ([D]) |
| --- | --- | --- |
| Next | N, Enter, PgDn, Right, Down, Space | N, PgDn, Down, Enter, Space (Right on Mac listed) |
| Previous | P, PgUp, Left, Up, Backspace | P, PgUp, Left, Up, Backspace |
| Jump | Number then Enter; Ctrl+S All Slides | G, arrows, Enter |
| First, last | Home, End | [LIVE] |
| Black, white | B or `.`, W or `,` | [LIVE] (Mobile documents B) |
| Hidden next | H, not in Presenter | [LIVE] |
| Hotspots | Tab, Shift+Tab, Enter | [LIVE] |
| Pointer | Ctrl+L laser, Ctrl+P pen, Ctrl+A arrow, Ctrl+E eraser, Ctrl+H hide, E erase, Ctrl+M ink on or off | [LIVE] |
| Captions | J | J |
| Help | F1 | [LIVE] |
| End | Esc | Esc |
| Menu | Shift+F10 or right-click | Right-click: back, go to, end |

Holding both mouse buttons for two seconds returns to the first slide ([D], desktop).

**Web control bar** ([D]): at lower left, for the presenter only. It may fade; move to the corner or press T.
Buttons: previous, next, all slides, drawing tools, subtitles, Present Live, Rehearse with Coach, camera, end,
options. Looping is not supported on the web.

**Near-term change.** A roadmap item for single-screen controls is listed for November 2026 (573151) ([D\*]).

Sources: [show shortcuts](https://support.microsoft.com/en-us/accessibility/powerpoint/use-keyboard-shortcuts-to-deliver-powerpoint-presentations), [shortcuts](https://support.microsoft.com/en-us/accessibility/powerpoint/use-keyboard-shortcuts-to-create-powerpoint-presentations), [present](https://support.microsoft.com/en-us/powerpoint/present-your-slide-show), [behaviour](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint), [roadmap digest](https://go-planet.com/in-the-know/october-29-2026/). Checked 2026-11-07.

## 27. Presenter view

**Availability:** desktop and Mac. Not on the web ([D], behaviour page and compare table). **This is the
largest gap in web parity, and `DEC-06` has it on our feature bar.**

**Start** ([D]): automatic with a second display; Slide Show > Use Presenter View with a monitor choice;
Alt+F5 on one display; or Show Presenter View from the show's lower-left menu. Mac has Swap Displays and
Mirror Show.

**Content** ([D]): current slide, next slide, notes; slide navigation arrows; timer with pause and reset;
clock; notes font larger and smaller; pen and laser; all-slides grid; zoom into a part of a slide; black
screen; End Slideshow. The audience sees none of it, including the steps to reach a hidden slide.

**Keys** ([D]): F6 cycles regions, Tab within, Ctrl+Up and Down scroll notes a line, Ctrl+PgDn and PgUp a
screen, Esc closes. Alt+A, Z, W, Q read notes and time aloud.

**This surface is changing now.** Redesigned Presenter View items for Windows (577178) and Mac (577179) are
listed for November 2026: tools, navigation and timer in one bar, thumbnails of upcoming slides, and
collapsible panes ([D\*]). **Measure the new version**, not screenshots of the old one.

**In a browser** ([I]): showing two windows requires the Window Management permission or a popup. That limit
falls under the docs/95-style exception, and it is ours to design.

Sources: [presenter](https://support.microsoft.com/en-us/office/use-presenter-view-in-powerpoint-fe7638e4-76fb-4349-8d81-5eb6679f49d7), [views](https://support.microsoft.com/en-us/powerpoint/training/choose-the-right-view-for-the-task-in-powerpoint), [show shortcuts](https://support.microsoft.com/en-us/accessibility/powerpoint/use-keyboard-shortcuts-to-deliver-powerpoint-presentations), [present](https://support.microsoft.com/en-us/powerpoint/present-your-slide-show), [behaviour](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint), [roadmap digest](https://go-planet.com/in-the-know/october-29-2026/). Checked 2026-11-07.

## 28. Cameo and captions

**Cameo** ([D]; subscribers on Windows, Mac and web)

1. Insert > Cameo places a camera placeholder.
2. The Preview icon turns the feed on. The web asks for browser camera permission first.
3. Camera Format tab: camera choice, Styles, Shape, Border, Effects.
4. It arranges, animates and transitions like a picture. One feed per slide.
5. In a shared file each person sees only their own feed.
6. A Cameo placeholder can be added to a layout.

Alt+X changes the camera in a show ([D\*]).

**Live captions** ([D])

- Slide Show > Subtitle Settings: spoken language, subtitle language, microphone, position.
- Positions: overlaid at top or bottom, or docked above or below the slide. **The default is Below Slide.**
- Appearance options: colour, size, transparency, font and background (Windows).
- Toggle: button in the show or Presenter toolbar, right-click, or J.
- Always Use Subtitles is off by default.
- It needs a connection, and a headset is advised. The web tab lists compatible browsers.

Speech recognition and translation would be a host seam for us (docs/47).

Sources: [cameo](https://support.microsoft.com/en-us/powerpoint/presenting-with-cameo), [captions](https://support.microsoft.com/en-us/office/present-with-real-time-automatic-captions-or-subtitles-in-powerpoint-68d20e49-aec3-456a-939d-34a79e8ddd5f), [show shortcuts](https://support.microsoft.com/en-us/accessibility/powerpoint/use-keyboard-shortcuts-to-deliver-powerpoint-presentations). Checked 2026-11-07.

## 29. Slide Sorter and Reading view

**Sorter** ([D])

- View tab, or status bar. Alt+W, I on desktop ([D\*]).
- A thumbnail grid. Drag to reorder.
- Move sections and add them here. On the web this is the documented route for sections.
- Hide and unhide by the same right-click.
- The web has it now, per the behaviour page.
- Ctrl+A selects all.

**Reading** ([D])

- From the status bar. Alt+W, D on desktop ([D\*]).
- Full-window show with simple flip controls, for someone reading alone.
- Web: flip through, and show or hide notes.
- Audio and video play ([D], from a page with stale media notes).

**Other views:** Notes Page, Outline and Masters are on desktop and Mac View. Outline and Master are not on the
web ([D]). Ctrl+Shift+Tab switches rail between thumbnails and outline on desktop ([D]).

Sorter zoom, thumbnail size, and Reading view's exit gesture are [LIVE].

Sources: [views](https://support.microsoft.com/en-us/powerpoint/training/choose-the-right-view-for-the-task-in-powerpoint), [views (other URL)](https://support.microsoft.com/en-us/office/choose-the-right-view-for-the-task-in-powerpoint-21332d8d-adbc-4717-a2c6-e25a697b40e9), [sections](https://support.microsoft.com/en-us/powerpoint/training/organize-your-powerpoint-slides-into-sections), [behaviour](https://support.microsoft.com/en-us/powerpoint/how-certain-features-behave-in-web-based-powerpoint), [shortcuts](https://support.microsoft.com/en-us/accessibility/powerpoint/use-keyboard-shortcuts-to-create-powerpoint-presentations). Checked 2026-11-07.

## 30. Comments

**Add** ([D\*])

1. Select an object or text.
2. Review > New Comment, Insert > Comment, right-click, or the floating toolbar. Ctrl+Alt+M (Mac Cmd+Shift+M).
3. Type. Post, or Ctrl+Enter.

**Anchors** ([D\*])

- Object comments move with the object. The bubble can be dragged anywhere within it.
- Text comments highlight a range. Typing inside grows the range, deleting shrinks it, and typing at its ends
  does not grow it.
- Otherwise a comment attaches to the slide.

**View** ([D\*])

- Web: Comments button near the top right opens the pane. **Margin comments are gone from the web.**
- Desktop: Review > Show Comments.
- Only the selected comment's marker shows by default. Show Advanced Markup shows all, for the session.
- A blue dot marks an unread item.
- Thread actions: edit (pencil on Windows, More actions elsewhere), Delete thread, Like, Resolve.
- Resolved threads leave the margin and stay in the pane.
- @mention sends email and works for cloud files. "Assign to" makes a task.

**Authorship conflict in the docs.** One page says only the creator can edit. Another warns that people with
edit access can alter comments since they live in the file. [LIVE].

**Keys** ([D]): F6 (web Ctrl+F6) to the pane; Alt, Z, C opens it (web Alt+Period, Z, C); Down and Up move threads;
Right and Left expand and collapse; Tab reaches the reply box; Alt+F12 jumps between comment and anchor.

Web has **no Track Changes** ([D\*]), and classic-versus-modern choice is desktop-only ([D\*]).

Sources: [modern comments](https://support.microsoft.com/en-us/powerpoint/modern-comments-in-powerpoint), [comments](https://support.microsoft.com/en-us/powerpoint/training/add-change-hide-or-delete-comments-in-a-presentation), [comment shortcuts](https://support.microsoft.com/en-us/accessibility/powerpoint/use-keyboard-shortcuts-to-navigate-modern-comments-in-powerpoint), [web review](https://support.microsoft.com/en-us/powerpoint/track-changes-in-powerpoint-for-the-web-during-a-peer-review). Checked 2026-11-07.

## 31. Co-authoring presence

- Avatars (image or initials) sit at the top right. Clicking one offers Join Group Chat ([D]).
- A co-author's icon on a thumbnail shows who is on that slide; hover names them ([D]).
- On the slide, a border surrounds the area they are in ([D]). Coloured flags show location ([D]).
- A setting shows full names as presence flags: File > Options > Advanced > Display ([D]).
- A Design Suggestions button greys out when someone else is editing the slide ([D]).

**Recent changes** ([D]): a blue dot on the thumbnail, a coloured outline on the slide. It skips notes, deleted
shapes, new comments, and animation changes.

**Conflicts** ([D]): non-conflicting edits merge on save. On desktop, real conflicts open a resolve screen with
My Changes and Changes by Others, Next and Previous, and "Do this for remaining conflicts". Web conflict behaviour
is [LIVE] and probably absent ([I], since it is continuous).

**Rename while co-authoring** on the web syncs after a moment ([D]).

**Editing the same object at once:** locking, last write wins, or per-character merge, is not documented.
**[LIVE] with two sessions.** Relevant to docs/24.

Sources: [co-authoring](https://support.microsoft.com/en-us/office/work-together-on-powerpoint-presentations-0c30ee3f-8674-4f0e-97be-89cf2892a34d), [share and collaborate](https://support.microsoft.com/en-us/office/share-and-collaborate-in-powerpoint-07fcfb59-f410-4e52-9c60-3b069a805ee5), [Designer](https://support.microsoft.com/en-us/powerpoint/create-professional-slide-layouts-with-designer). Checked 2026-11-07.

## 32. Share

**Steps** ([D])

1. Share at the top right. If the deck is local, a message directs you to save it to cloud storage first.
2. Enter people or addresses.
3. Permission drop-down: can edit or view. Older page: Allow editing checked by default, uncheck for view only.
4. Optional note. Send.
5. Or Copy Link: anyone holding the link gets the default permissions.

Share a copy is on the same panel ([D]). Slide link: §4. Web sharing specifics and the current dialog layout are
[LIVE].

Sources: [co-authoring](https://support.microsoft.com/en-us/office/work-together-on-powerpoint-presentations-0c30ee3f-8674-4f0e-97be-89cf2892a34d), [share and collaborate](https://support.microsoft.com/en-us/office/share-and-collaborate-in-powerpoint-07fcfb59-f410-4e52-9c60-3b069a805ee5), [link to slide](https://support.microsoft.com/en-gb/powerpoint/share-a-link-to-a-specific-slide). Checked 2026-11-07.

## 33. AutoSave, version history, undo

**AutoSave** ([D])

- Web: always on, saves every few seconds, no Save button. Pending edits sync when back online.
- Desktop: a toggle at upper left; on by default for cloud files; disabled for local files, old formats, or paused sync.
- With cloud files **Save As is replaced by Save a Copy**, and Microsoft advises taking the copy before editing.
- AutoRecover and a Document Recovery pane cover crashes.
- Ctrl+S on desktop. Web table says saving is automatic.

**Version history** ([D])

1. Click the file name > Version History (Mac: Browse Version History; web also File > Info).
2. Choose a version; it opens read-only in a separate window.
3. Restore. Or copy content out, or Save a Copy.

New versions are added about every 10 minutes during a session, after the first edit. Personal accounts keep the
last 25; work accounts depend on the library.

**Undo**

- Ctrl+Z and Ctrl+Y, on both ([D]).
- Undo reverses a Designer choice and a mis-move in the Animation Pane ([D]).
- QAT has Undo on desktop; on the web, Undo is on Home ([I-recall], matching docs/47).
- Depth of 20 by default with a setting up to 150 is [I-recall], not in any page read. [LIVE].
- Whether undo is per-user in co-authoring is **not documented**. [LIVE]. Relevant to docs/24.

Sources: [AutoSave](https://support.microsoft.com/en-us/office/collab-files/what-is-autosave), [versions](https://support.microsoft.com/en-us/office/view-previous-versions-of-office-files-5c1e076f-a9c9-41b8-8ace-f77b9642e2c2), [co-authoring](https://support.microsoft.com/en-us/office/work-together-on-powerpoint-presentations-0c30ee3f-8674-4f0e-97be-89cf2892a34d), [create on web](https://support.microsoft.com/en-au/office/21360025-7eef-4173-9d7c-08281d55f64a), [shortcuts](https://support.microsoft.com/en-us/accessibility/powerpoint/use-keyboard-shortcuts-to-create-powerpoint-presentations). Checked 2026-11-07.

## 34. Find, and the Search box

**Find and Replace** ([D])

- Ctrl+F opens Find, Ctrl+H opens Replace, Shift+F4 repeats on desktop. Web table lists Ctrl+F.
- Desktop: Home > Editing. Fields are Find what and Replace with, then Find Next, Replace, Replace All. Esc cancels.
- Mac: the arrow by Search in Presentation, upper right.
- Web: on the right end of Home, per one page. See C1.

Match case, whole words, and scope (notes and masters) are [LIVE].

**Search box** ([D])

1. Alt+Q focuses it, on both platforms.
2. Type an action. Down arrow moves through results.
3. Enter runs a command directly. Help results are mixed in.

It may be labelled Tell Me on older versions. With Simplified, Microsoft sends you here when a command is
missing. Result grouping is [LIVE].

Sources: [find](https://support.microsoft.com/en-us/office/find-and-replace-text-b64c77c2-f5a1-4b78-9f03-50ec050b2174), [shortcuts](https://support.microsoft.com/en-us/accessibility/powerpoint/use-keyboard-shortcuts-to-create-powerpoint-presentations), [screen reader](https://support.microsoft.com/en-us/office/use-a-screen-reader-to-explore-and-navigate-powerpoint-a11115c7-6038-44a9-b355-8e133a4e9594), [Simplified](https://support.microsoft.com/en-us/office/use-the-simplified-ribbon-in-powerpoint-for-the-web-f05d2d91-ad15-44b0-b401-e85735b409cb). Checked 2026-11-07.

## 35. Accessibility checker and Reading Order

**Checker, steps** ([D])

1. Review > Check Accessibility opens a right-hand pane. The lower half of the button has more options on desktop.
2. Web: choose among Errors, Warnings, Tips near the top of the pane.
3. Expand an issue to list affected objects. **Selecting an item highlights it on the slide.**
4. Recommended Actions on the item fixes it.
5. Web: Recheck.

It also runs in the background ([D\*]).

**Rule classes** ([D\*]): Errors include missing alt text and tables without column headers. Warnings include
non-rectangular tables, low contrast, missing media captions and illogical reading order.

**Reading Order pane** ([D])

1. From the checker: Check reading order > a slide > Verify object order.
2. Rows are numbered in read order. Unnumbered means decorative, and is skipped.
3. Drag or Move Up and Move Down. **Order changes can alter overlap**, and Ctrl+Z reverts.
4. Ctrl-select and group to treat a set as one, then describe it. A tip suggests when.

Sources: [checker](https://support.microsoft.com/en-us/accessibility/office-accessibility/improve-accessibility-with-the-accessibility-checker), [rules](https://support.microsoft.com/en-us/office/rules-for-the-accessibility-checker-651e08f2-0fc3-4e10-aaca-74b4a67101c1), [reading order](https://support.microsoft.com/en-us/office/make-slides-easier-to-read-by-using-the-reading-order-pane-863b5c1c-4f19-45ec-96e6-93a6457f5e1c). Checked 2026-11-07.

## 36. Designer and Copilot entry points (scope awareness only)

**We do not build these, pending `DEC-05`.** Recorded so nothing we build collides with their positions.

**Designer** ([D]): Design > Design Suggestions opens a right-hand pane; picking applies at once and Ctrl+Z reverses;
can auto-open; File > Options > General turns it off; the setting depends on connected experiences.

**Copilot**

- **A Dynamic Action Button in a corner of the document**, on Windows, Mac and web (page updated June 2026). It opens a chat pane on
  the right. It only appears with a licence. It can be docked by right-click or by dragging to the side ([D]).
- Keys: Alt+C on Windows and web, Cmd+Control+I on Mac. **F6 also lands on that button**, on all platforms ([D]).
- Shift+F8 rewrites text on desktop ([D]).
- Web home page: a create option from May 2026 ([D\*]).
- Ribbon one-click skills: Review Presentation on Review, Visualize Slide on Design, from mid-2026 ([D\*]).

**Why it matters for us:** F6 and the Alt+C slot, and the canvas's corner, have meaning in PowerPoint. Our F6 ring has
no such stop unless we add one.

Sources: [Designer](https://support.microsoft.com/en-us/powerpoint/create-professional-slide-layouts-with-designer), [Copilot button](https://support.microsoft.com/en-us/office/foundations-experiences/copilot-dab/the-copilot-dynamic-action-button-in-word-excel-and-powerpoint), [MC1311972](https://mc.merill.net/message/MC1311972), [MC1419795](https://mc.merill.net/message/MC1419795), [shortcuts](https://support.microsoft.com/en-us/accessibility/powerpoint/use-keyboard-shortcuts-to-create-powerpoint-presentations). Checked 2026-11-07.

## 37. Keyboard-only navigation

### Region cycle

| | Desktop ([D]) | Web ([D]) |
| --- | --- | --- |
| Forward | **F6** | **Ctrl+F6** (Cmd+F6) |
| Back | Shift+F6 | Ctrl+Shift+F6 |
| Order | Slide, status bar, ribbon tabs, Notes if open, Thumbnails | Slide, Comments if open, Notes if open, status bar, top toolbar, ribbon, Thumbnails |

Microsoft states the reason: browser hosting changes keys ([D]). The short table for the web omits Comments and
calls the top toolbar "title bar". The screen-reader page gives the longer list. Whether a task pane joins the desktop
ring is documented only as "may take several presses" ([D]).

### Inside a region ([D])

Tab and Shift+Tab move. Enter selects. Arrows browse lists. Esc leaves a menu or mode. In the rail, Up and Down.

### Ribbon

| Step | Desktop ([D]) | Web ([D]) |
| --- | --- | --- |
| Reveal KeyTips | Alt, or F10 | Alt+Period (Mac Ctrl+Period) |
| Focus active tab | Alt | Alt+Windows logo key |
| Change tab | Letter, or Left and Right | Left and Right; screen-reader page says Tab and Enter |
| Enter the commands | Tab | Tab once |
| Move | Tab, arrows | Left and Right |
| Next group | Ctrl+Left, Ctrl+Right | Same, on the Single Line Ribbon |
| Activate | Space, Enter | Enter or Space |
| List | Down arrow, or Alt+Down | Alt+Down, then Up and Down |
| Gallery | Arrows | Tab to More, Enter, then Tab through |
| Tooltip | Ctrl+Shift+F10 | [LIVE] |
| Context menu | Shift+F10, Menu key | [LIVE] (rotate page uses Shift+F10) |
| Collapse | Ctrl+F1 | Not in the web table; one search excerpt says Ctrl+F1, [LIVE] |
| Help | F1 | [LIVE] |
| Header | Shift+Tab from the tabs | Region cycle |

**KeyTip rules** ([D]): one or two letters per step; two to four keys for most commands; Alt cancels; **Esc steps
back a level**, and at the first level returns to the document.

**Desktop access keys** ([D]): File F, Home H, Insert N, Draw J then I, Design G, Transitions K, Animations A, Slide Show S,
Review R, View W, Recording E, Help Y then 2, Search Q.

**Common chains** ([D]): Layout Alt+H, L. Font size Alt+H, F, S. Selection pane Alt+H, S, L, P. Text box Alt+N, X.
Picture Alt+N, P, D. Shape Alt+N, S, H. Theme Alt+G, H. Notes Alt+W, P, N. Zoom Alt+W, Q.

**Web KeyTip letters** other than Z (ribbon) and C (comments) are not documented. [LIVE]. Note the screen-reader guide
advises turning **Simplified off** for effective ribbon keys ([D]). That is a real caveat on a Simplified default.

**Browser reservation.** Microsoft recommends Edge and warns that shortcuts differ ([D]). Which keys the app
actually wins in Chrome, Firefox, Safari (Ctrl+D, Ctrl+P, Ctrl+N, Ctrl+T, Ctrl+W, F5, Ctrl+F6) is [LIVE]. The web table
**omits Ctrl+N and F5** and says saving is automatic, consistent with these keys being browser-owned ([I]).

Sources: [shortcuts](https://support.microsoft.com/en-us/accessibility/powerpoint/use-keyboard-shortcuts-to-create-powerpoint-presentations), [ribbon keys](https://support.microsoft.com/en-gb/office/use-the-keyboard-to-work-with-the-ribbon-954cd3f7-2f77-4983-978d-c09b20e31f0e), [screen reader](https://support.microsoft.com/en-us/office/use-a-screen-reader-to-explore-and-navigate-powerpoint-a11115c7-6038-44a9-b355-8e133a4e9594). Checked 2026-11-07.

## 38. Mobile

**Native apps** ([D])

- Ribbon is collapsible. iPhone: Edit icon shows, down arrow hides. Android phone: Edit icon or up arrow; down arrow.
  iPad: tap a tab. Android tablet: up and down arrows.
- Slides change by tapping the navigation pane.
- Layout: Home > Slide Layout.
- Comment button at the bottom on Android, and above the slide sorter on both ([D\*]).
- The transitions page has Android and iOS tabs.
- Compare table marks charts as not addable on mobile.

The **exact phone command layout** is not in any page read. OpenCalc's docs/95 records Excel's as a bottom bar with a
tab picker expanding to a sheet; that PowerPoint phones work the same way is [I-recall]. [LIVE].

**Web on a phone** is not documented; behaviour on phone browsers is [LIVE]. The page says the web app is reachable from a
phone or tablet ([D]).

Mac and iPad keyboards have their own tables ([D]). iOS can make portrait slides, per an Insider blog title, 2022 ([D\*]).

Sources: [mobile ribbon](https://support.microsoft.com/en-us/office/show-or-hide-the-ribbon-in-powerpoint-on-a-mobile-device-ae985bbb-c273-413b-addc-872dcec6e7b8), [apply layout](https://support.microsoft.com/en-us/office/apply-a-slide-layout-158e6dba-e53e-479b-a6fc-caab72609689), [comments](https://support.microsoft.com/en-us/powerpoint/training/add-change-hide-or-delete-comments-in-a-presentation), [compare](https://support.microsoft.com/en-us/office/compare-powerpoint-features-on-different-platforms-90986850-227c-4b25-938e-1c5838166b8b), [create on web](https://support.microsoft.com/en-au/office/21360025-7eef-4173-9d7c-08281d55f64a), [iOS portrait](https://techcommunity.microsoft.com/blog/microsoft365insiderblog/create-slides-in-portrait-mode-on-powerpoint-for-ios/4217188). Checked 2026-11-07.

## 39. Touch

**Gestures** ([D]: Android, iPhone, iPad guides. These guides are undated, and may be old.)

| Do | Gesture |
| --- | --- |
| Change slide | Tap in the navigation pane |
| Zoom | Spread; pinch |
| Open keyboard | **Double-tap the text box** |
| Place caret | With keyboard open, tap (iPad, Android); tap (iPhone) |
| Word | Double-tap; also shows the Edit menu |
| Paragraph | Triple-tap (iPhone, iPad); drag indicators (Android) |
| Range | Double-tap and drag the handles |
| Clear text selection | Tap elsewhere |
| Table | Tap anywhere on it |
| Move | Tap the object, hold, drag. On iPhone drag a **move handle** |
| Resize | Tap, hold a resize handle, drag |
| Rotate | Tap, hold the rotation handle, drag |

**So: one tap selects, a second opens text.** The difference from the mouse is that a single tap never places a caret in
unselected text ([I], from double-tap being what opens the keyboard). [LIVE].

**Desktop Touch Mode** ([D]): a hand icon in the QAT, added from its customise menu, switches Touch and Mouse. It only
widens ribbon button spacing. The page's "Applies to" lists perpetual versions 2016–2024, and whether it persists in
Microsoft 365 is [LIVE].

**Ink on touch** ([D]): §23. Draw tab appears automatically. Fingers ink when a pen is active; otherwise tap selects
([D\*]). Lasso and Draw tool switch modes.

Sources: [Android](https://support.microsoft.com/en-us/office/powerpoint-for-android-touch-guide-7f2b68df-608f-428b-958b-5a165fb109ce), [iPhone](https://support.microsoft.com/en-us/office/powerpoint-for-iphone-touch-guide-39499f20-6f6b-4de3-8cf0-173923c8bf7f), [iPad](https://support.microsoft.com/en-us/office/powerpoint-for-ipad-touch-guide-bbc7cfc1-6fb0-4579-a19a-de7b4ff10478), [touch mode](https://support.microsoft.com/en-us/office/turn-touch-mode-on-or-off-90e162b1-44f9-434f-bc2b-9321c989ea6e), [ink](https://support.microsoft.com/en-us/office/draw-and-write-with-ink-in-office). Checked 2026-11-07.

## 40. What a live session must measure

Nothing below is known. `UX-001` needs `DEC-04` first.

**Setup.** PowerPoint for the web in Edge and Chrome; PowerPoint for Microsoft 365 on Windows and Mac. Viewports 1280 × 720,
1366 × 768, 1440 × 900, 1920 × 1080, at 100 % display scaling. Web: read `getBoundingClientRect()` and computed styles.
Desktop: 100 % scale screenshots, and state that they are screenshots. Record the build number with each figure.

| # | Measure | Why |
| --- | --- | --- |
| M1 | Height of title bar, tab strip, ribbon body (both layouts), status bar | Region stack |
| M2 | Rail default, min and max width; thumbnail size; gap; number position | Rail |
| M3 | Notes pane default, min, max; persisted or not | Layout |
| M4 | Task pane width, resizable or not, single or several | Pane model |
| M5 | Canvas margin around a fit slide; whether fit is recomputed on pane toggle | Canvas |
| M6 | Handle size and hit area; rotation handle offset; colours at each zoom | Selection |
| M7 | Snap threshold; smart-guide colour, dash, and spacing markers; priority | Guides |
| M8 | Nudge steps with snap on and off, with Ctrl, on both platforms; duplicate offset | Precision |
| M9 | Ribbon control sizes; group collapse order and widths | Ribbon |
| M10 | Zoom steps, slider range; wheel behaviour at and above fit | Canvas |
| M11 | Every context menu on rail, slide background, shape, text, picture, table | Menus |
| M12 | Mini toolbar trigger and contents; delay | Text |
| M13 | Esc ladder from child in group, text in child, crop, painter, drawing mode | Modes |
| M14 | Typing with object selected, and empty-placeholder delete | Text |
| M15 | Keys actually won by the web app in each browser | Keyboard |
| M16 | Web KeyTip letters for every tab and common command | Keyboard |
| M17 | Web charts, video insert, Replace, merge, connectors, gridlines, ink | Conflicts C1–C3 |
| M18 | Two sessions on one object; undo scope; offline banner | Collaboration |
| M19 | Presenter View after the November 2026 redesign | Presenter |
| M20 | Phone browser and native app command layout | Mobile |
| M21 | Motion durations for pane open, tab switch, thumbnail drag | Motion |

## 41. Rules for our shell (proposed)

In the style of OpenCalc docs/95. Adopted from PowerPoint: structure, flows, keys, wording. Ours: name, logo, colours, icon
artwork, fonts, themes, templates, help. Adopted under owner instruction 12 ([03](03-OWNER-INSTRUCTIONS.md)); the decisions on §42's owner questions are `DEC-10` to `DEC-15` in [14](14-EXECUTION-TRACKER.md). Unverified points stay open under `UX-002`.

1. **PowerPoint is the specification.** A difference is allowed only where a browser cannot do what PowerPoint does, or for
   our own assets. Each difference names its limit.
2. **Web sets flows, desktop sets the feature bar** (docs/48, `DEC-06`). Where the web has no UI for a desktop feature, use
   desktop behaviour in the web's visual language.
3. **Where web and desktop differ in a key, follow the web first and add the desktop key when it does not collide.** So
   Ctrl+F6 is primary, F6 also works when the browser passes it.
4. **Icons, not text glyphs; group names on, button labels off, as in OpenCalc rule 1.** Every icon has a tooltip with name
   and shortcut. Measure against M9.
5. **One selection model with two levels, always visible.** Object selected is a solid outline; text being edited is dashed.
   Enter or F2 goes in. **Esc always steps out one level:** text → object → nothing. Same for child → group, crop → picture,
   painter → off, drawing mode → off, KeyTips → previous level, menu → closed.
6. **One Esc, one level, no exceptions.** It never closes a pane holding focus elsewhere, and it never loses text.
7. **The canvas is fit by default, and re-fits when panes change,** until the user zooms. Ctrl+Alt+O restores fit (with a web
   fallback if the browser claims it).
8. **Guides appear only during a drag,** in one accent, and never print or show. Alt suspends snapping.
9. **Snap and nudge respect the same state.** Arrows nudge, Ctrl+arrows nudge finer. Numbers come from M8; do not hard-code
   docs/47's.
10. **Rotation handle on top. Shift gives 15°. Alt+arrows step 15°.** Several selected rotate about their own centres.
11. **Shape insertion follows the web by default, a tap or click drops a default-size shape; a drag draws one.** Shift
    constrains. This accepts both habits, since the two platforms differ. Verify M17 first.
12. **Placeholder prompts never reach the show, thumbnail or export.** Icons in an empty content placeholder start the same
    flow as the ribbon.
13. **AutoFit control appears only on overflow, only on placeholders,** at the lower left. Text boxes use the pane.
14. **A key has one meaning at each selection level.** With a caret in text, Ctrl+Shift+V is plain paste, as on the web. With an
    object selected, Ctrl+Shift+C and V copy and apply formatting, as on desktop (open question 3). Ctrl+[ and ] change layer. Font size is Ctrl+Shift+> and <. Do not copy docs/47's
    Ctrl+[ binding.
15. **Contextual tabs appear with the selection and are named as the web names them** where it differs: Shape, Picture, and
    so on. Tab names are one of the wordings to confirm at M9.
16. **One place per command, plus the right-click menu and search,** as OpenCalc rule 3. Search runs commands.
17. **A pane edit applies live, with no OK and no Cancel; Undo reverts.** Panes open on the right.
18. **Everything has a keyboard route.** F6 ring, Tab inside, arrows in lists, Shift+F10 menus, Alt+Period KeyTips. Selection
    pane is the keyboard path to multi-select. A test walks the ring.
19. **Save is not a command.** Saving is automatic and visible as state. Version history lives on the file name.
20. **Web-only limits are stated in place.** A deck feature we preserve but cannot edit stays preserved and plays, as Microsoft
    does with unsupported animations, and the control says so instead of disappearing without trace.
21. **Touch: one tap selects, a second tap opens text.** Targets meet Fluent's 44 × 44 floor on phones. Pinch zooms.
22. **Show keys match desktop and web together.** N, Enter, Space, PgDn, Down, Right go on. P, Backspace, PgUp, Up, Left go back.
    Number + Enter and G both jump. B and W blank. J captions. T toggles the bar. Esc ends.
23. **Nothing that is for us** in the chrome, and defaults need no decision: a new deck is blank with a title slide, as OpenCalc
    rules 6 and 7.
24. **No Microsoft asset, ever.** Themes, templates, stock, icons and wording of help are ours. docs/47 already holds this.

## 42. Open questions

Decided 2026-11-07 under owner instruction 12: `DEC-10` to `DEC-15` in [14](14-EXECUTION-TRACKER.md). The questions are kept for the record:

1. **Simplified or classic as default?** OpenCalc chose classic. Our docs/47 says the web defaults to Simplified, from recall.
   And the screen-reader guide advises turning Simplified off for keyboard work. Recommendation: classic, matching docs/95, unless M9 shows otherwise.
2. **Shape insertion (rule 11):** is click-drops-and-drag-draws acceptable, given the two platforms differ?
3. **`Ctrl+Shift+V` (rule 14):** plain paste or format paste?
4. **`DEC-05`:** with Copilot's button occupying a canvas corner and F6 stop in PowerPoint, does our shell leave a slot, or none?
5. **Presenter View:** which of old or redesigned (November 2026) is the target? This changes the build.
6. **Charts:** does editing use OpenCalc in a pane or a separate window?

For the live session, beyond §40:

7. C1, C2, C3, C7 and C8 in §0.1.
8. Is a marquee enclosing or touching? How is a group entered? What does Esc do from a child?
9. How many undo levels, and is undo per-user during co-authoring?
10. What is the same-object conflict rule during co-authoring?
11. Does Touch Mode still exist in Microsoft 365 builds?
12. Do the phone native app and phone browser share a command layout?
13. Can more than one task pane be open at once?
14. Which window does a browser Presenter view need, and which permission? (Ours to design; PowerPoint's web has none to copy.)

**Pages that could not be used:** two office-watch.com pages (bot check); a guessed Microsoft URL for connectors (404); several
guessed URLs for mini toolbar and Format Painter (404, 410, 403). No Microsoft end-user article on connectors, the mini toolbar or
undo depth was found in this round; that absence is a finding about this search, not proof that none exists.
