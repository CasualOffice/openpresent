/// The ribbon, as data. Copied in shape from OpenCalc's `webapp/ribbon.model.js`
/// at `b23c67c` (webapp/README.md, owner instruction 42): the same `L`, `S`, `R`
/// builders, `fold` ranks and File page model. The content is PowerPoint for the
/// web's, tab by tab and group by group in its order, as docs/53 records it
/// (`UX-003`): File, Home, Insert, Draw, Design, Transitions, Animations, Slide
/// Show, Review, View, Help, and Shape Format while a shape is selected.
///
/// **Where this differs from OpenCalc.** OpenCalc leaves out a verb it has not
/// built (its docs/91 §8: a dead button advertises an absence). The owner asked
/// for the opposite here (2026-11-08): every PowerPoint command is present, and
/// the ones this build cannot run yet are disabled through the command registry,
/// never faked. `editor.core.js` registers each id below that has no working
/// command as `enabled: () => false`.
///
/// Shapes:
///   L(id, icon, label, {menu?})   a large button: 24px icon, its label under it
///   S(id, icon, label, {menu?})   a small button with its label beside, as PowerPoint
///                                 labels Cut, Copy, Layout, Reset, Shape Fill…
///   I(id, icon, label, {menu?})   a small icon-only button (B, I, U, align…)
///   C(id, label, width)           a combo box (Font, Font Size)
///   G(id, label, tiles)           an in-ribbon gallery (Themes, Transitions…)
///   R([[…], […], …])              a column of up to three rows
/// `menu` lists the command ids the button drops. `fold` on a group is its
/// collapse rank, lowest first; `launcher` names the command its corner arrow
/// runs. A tab with `contextual` shows only while that kind of object is
/// selected. Icons are names in `icons.fluent.svg`.

const L = (id, icon, label, o = {}) => ({ kind: "large", id, icon, label, ...o });
const S = (id, icon, label, o = {}) => ({ kind: "small", id, icon, label, text: true, ...o });
const I = (id, icon, label, o = {}) => ({ kind: "small", id, icon, label, ...o });
const C = (id, label, width) => ({ kind: "combo", id, label, width });
const G = (id, label, tiles) => ({ kind: "gallery", id, label, tiles });
const R = (rows) => ({ kind: "rows", rows });

const NEW_SLIDE = ["home.new-slide", "home.new-slide-layout", "home.duplicate-slide", "home.delete-slide", "home.reuse-slides"];
const ZOOM = ["view.zoom.fit", "view.zoom.50", "view.zoom.75", "view.zoom.100", "view.zoom.150", "view.zoom.200"];
const SECTION = ["slide.section.add", "slide.section.rename", "slide.section.remove", "slide.section.collapse-all", "slide.section.expand-all"];
const ARRANGE = ["arrange.bring-to-front", "arrange.bring-forward", "arrange.send-backward", "arrange.send-to-back", "arrange.group", "arrange.ungroup", "arrange.align-left", "arrange.align-center", "arrange.align-right", "arrange.align-top", "arrange.align-middle", "arrange.align-bottom", "arrange.rotate-right", "arrange.flip-horizontal", "arrange.selection-pane"];
const SELECT = ["edit.select-all", "arrange.selection-pane"];
const SHAPES = ["insert.shape.rectangle", "insert.shape.oval", "insert.shape.line", "insert.shape.arrow", "insert.text-box"];
const PRESENT = ["slideshow.from-beginning", "slideshow.from-current", "slideshow.presenter-view"];

// Our own gallery swatches: plain colour pairs, not Microsoft's theme artwork.
const THEMES = [["Office", "#ffffff", "#156082"], ["Ion", "#1e5155", "#6ad7cf"], ["Facet", "#ffffff", "#6f9a3f"], ["Slate", "#3b3b3b", "#d0a54b"], ["Retro", "#f4efe1", "#b34a3c"], ["Dusk", "#2b2d42", "#ef8354"]];
const TRANSITIONS = [["None", "dismiss"], ["Morph", "shape_union"], ["Fade", "slide_transition"], ["Push", "arrow_next"], ["Wipe", "arrow_previous"], ["Split", "layout_row_two_split_top"], ["Reveal", "eye"], ["Cut", "cut"]];
const ANIMATIONS = [["None", "dismiss"], ["Appear", "sparkle"], ["Fade", "text_effects_sparkle"], ["Fly In", "arrow_next"], ["Float In", "arrow_maximize"], ["Split", "layout_row_two_split_top"], ["Wipe", "arrow_previous"], ["Zoom", "zoom_in"]];
const SHAPE_STYLES = [["Colored Outline", "#ffffff", "#156082"], ["Colored Fill", "#156082", "#156082"], ["Light 1 Fill", "#dbe9f4", "#156082"], ["Intense Effect", "#0e4a63", "#0e4a63"]];

export const TABS = [
  { id: "home", label: "Home", key: "H", groups: [
    { label: "Undo", fold: 10, icon: "arrow_undo", items: [
      R([[I("edit.undo", "arrow_undo", "Undo (Ctrl+Z)")], [I("edit.redo", "arrow_redo", "Redo (Ctrl+Y)")]]),
    ] },
    { label: "Clipboard", fold: 6, icon: "clipboard_paste", launcher: "edit.clipboard", items: [
      L("edit.paste", "clipboard_paste", "Paste (Ctrl+V)", { menu: ["edit.paste", "edit.paste-text"] }),
      R([[S("edit.cut", "cut", "Cut (Ctrl+X)")], [S("edit.copy", "copy", "Copy (Ctrl+C)")], [S("home.format-painter", "paint_brush", "Format Painter")]]),
    ] },
    { label: "Slides", fold: 7, icon: "slide_add", items: [
      L("home.new-slide", "slide_add", "New Slide (Ctrl+M)", { menu: NEW_SLIDE }),
      R([[S("home.layout", "slide_layout", "Layout", { menu: ["home.new-slide-layout"] })], [S("home.reset", "arrow_reset", "Reset")], [S("home.section", "slide_multiple", "Section", { menu: SECTION })]]),
    ] },
    { label: "Font", fold: 9, icon: "text_font", launcher: "font.dialog", items: [
      R([
        [C("font.name", "Font", 110), C("font.size", "Font Size", 48), I("font.increase", "font_increase", "Increase Font Size (Ctrl+Shift+>)"), I("font.decrease", "font_decrease", "Decrease Font Size (Ctrl+Shift+<)"), I("font.clear", "text_clear_formatting", "Clear All Formatting (Ctrl+Space)")],
        [I("font.bold", "text_bold", "Bold (Ctrl+B)"), I("font.italic", "text_italic", "Italic (Ctrl+I)"), I("font.underline", "text_underline", "Underline (Ctrl+U)"), I("font.strike", "text_strikethrough", "Strikethrough"),
         I("font.subscript", "text_subscript", "Subscript (Ctrl+=)"), I("font.superscript", "text_superscript", "Superscript (Ctrl+Shift++)"), I("font.case", "text_change_case", "Change Case", { menu: ["font.case.sentence", "font.case.lower", "font.case.upper", "font.case.title"] }),
         I("font.highlight", "highlight", "Text Highlight Color", { menu: ["font.highlight"] }), I("font.color", "text_color", "Font Color", { menu: ["font.color"] })],
      ]),
    ] },
    { label: "Paragraph", fold: 8, icon: "text_align_left", launcher: "para.dialog", items: [
      R([
        [I("para.bullets", "text_bullet_list_ltr", "Bullets", { menu: ["para.bullets"] }), I("para.numbering", "text_number_list_ltr", "Numbering", { menu: ["para.numbering"] }), I("para.indent-less", "text_indent_decrease", "Decrease List Level (Shift+Tab)"), I("para.indent-more", "text_indent_increase", "Increase List Level (Tab)"), I("para.line-spacing", "text_line_spacing", "Line Spacing", { menu: ["para.line-spacing"] })],
        [I("para.left", "text_align_left", "Align Left (Ctrl+L)"), I("para.center", "text_align_center", "Center (Ctrl+E)"), I("para.right", "text_align_right", "Align Right (Ctrl+R)"), I("para.justify", "text_align_justify", "Justify (Ctrl+J)"),
         I("para.columns", "text_column_two", "Columns", { menu: ["para.columns"] }), I("para.direction", "text_direction_rotate_270_right", "Text Direction", { menu: ["para.direction"] }), I("para.align-text", "align_center_vertical", "Align Text", { menu: ["para.align-text"] })],
      ]),
    ] },
    { label: "Drawing", fold: 5, icon: "shapes", launcher: "format.shape-pane", items: [
      L("insert.shapes", "shapes", "Shapes", { menu: SHAPES }),
      L("drawing.arrange", "position_to_front", "Arrange", { menu: ARRANGE }),
      L("drawing.quick-styles", "text_edit_style", "Quick Styles", { menu: ["drawing.quick-styles"] }),
      R([[S("drawing.fill", "color_fill", "Shape Fill", { menu: ["drawing.fill"] })], [S("drawing.outline", "color_line", "Shape Outline", { menu: ["drawing.outline"] })], [S("drawing.effects", "square_shadow", "Shape Effects", { menu: ["drawing.effects"] })]]),
    ] },
    { label: "Editing", fold: 4, icon: "search", items: [
      R([[S("edit.find", "search", "Find (Ctrl+F)")], [S("edit.replace", "text_wrap", "Replace (Ctrl+H)")], [S("edit.select", "select_object", "Select", { menu: SELECT })]]),
    ] },
    { label: "Voice", fold: 2, icon: "mic", items: [L("home.dictate", "mic", "Dictate")] },
    { label: "Designer", fold: 3, icon: "design_ideas", items: [L("design.designer", "design_ideas", "Designer")] },
  ] },
  { id: "insert", label: "Insert", key: "N", groups: [
    { label: "Slides", fold: 9, icon: "slide_add", items: [L("home.new-slide", "slide_add", "New Slide (Ctrl+M)", { menu: NEW_SLIDE })] },
    { label: "Tables", fold: 8, icon: "table_add", items: [L("insert.table", "table_add", "Table")] },
    { label: "Images", fold: 7, icon: "image_add", items: [L("insert.pictures", "image_add", "Pictures")] },
    { label: "Camera", fold: 1, icon: "video", items: [L("insert.cameo", "video", "Cameo")] },
    { label: "Illustrations", fold: 6, icon: "shapes", items: [
      L("insert.shapes", "shapes", "Shapes", { menu: SHAPES }),
      L("insert.icons", "icons", "Icons"),
      L("insert.3d", "cube", "3D Models"),
      L("insert.smartart", "data_area", "SmartArt"),
      L("insert.chart", "data_bar_vertical", "Chart"),
    ] },
    { label: "Forms", fold: 1, icon: "form_new", items: [L("insert.forms", "form_new", "Forms")] },
    { label: "Add-ins", fold: 1, icon: "apps_add_in", items: [L("insert.add-ins", "apps_add_in", "Add-ins")] },
    { label: "Links", fold: 3, icon: "link", items: [L("insert.link", "link", "Link (Ctrl+K)"), L("insert.action", "cursor", "Action")] },
    { label: "Comments", fold: 2, icon: "comment_add", items: [L("review.new-comment", "comment_add", "Comment")] },
    { label: "Text", fold: 5, icon: "textbox", items: [
      L("insert.text-box", "textbox", "Text Box"),
      L("insert.header-footer", "document_header_footer", "Header & Footer"),
      L("insert.wordart", "text_effects", "WordArt"),
    ] },
    { label: "Symbols", fold: 4, icon: "math_symbols", items: [
      L("insert.equation", "math_formula", "Equation"),
      L("insert.symbol", "math_symbols", "Symbol"),
    ] },
    { label: "Media", fold: 2, icon: "video", items: [L("insert.video", "video", "Video"), L("insert.audio", "music_note_2", "Audio")] },
  ] },
  { id: "draw", label: "Draw", key: "JI", groups: [
    { label: "Tools", fold: 3, icon: "pen", items: [
      L("draw.select", "cursor", "Select"),
      L("draw.lasso", "lasso", "Lasso Select"),
      L("draw.eraser", "eraser", "Eraser"),
      L("draw.pen", "pen", "Pen"),
      L("draw.highlighter", "highlight", "Highlighter"),
    ] },
    { label: "Convert", fold: 2, icon: "ink_stroke", items: [L("draw.ink-to-shape", "shapes", "Ink to Shape"), L("draw.ink-to-math", "math_formula", "Ink to Math")] },
    { label: "Replay", fold: 1, icon: "play", items: [L("draw.replay", "play", "Ink Replay")] },
  ] },
  { id: "design", label: "Design", key: "G", groups: [
    { label: "Themes", fold: 3, icon: "style_guide", items: [G("design.themes", "Themes", THEMES)] },
    { label: "Variants", fold: 2, icon: "color_background", items: [G("design.variants", "Variants", THEMES.slice(0, 4))] },
    { label: "Customize", fold: 4, icon: "slide_size", items: [
      L("design.slide-size", "slide_size", "Slide Size", { menu: ["design.slide-size"] }),
      L("design.format-background", "color_background", "Format Background"),
    ] },
    { label: "Designer", fold: 1, icon: "design_ideas", items: [L("design.designer", "design_ideas", "Designer")] },
  ] },
  { id: "transitions", label: "Transitions", key: "K", groups: [
    { label: "Preview", fold: 3, icon: "play", items: [L("transitions.preview", "play", "Preview")] },
    { label: "Transition to This Slide", fold: 2, icon: "slide_transition", items: [
      G("transitions.gallery", "Transitions", TRANSITIONS),
      L("transitions.effect-options", "settings", "Effect Options", { menu: ["transitions.effect-options"] }),
    ] },
    { label: "Timing", fold: 1, icon: "timer", items: [
      R([[S("transitions.timing", "timer", "Duration")], [S("transitions.apply-to-all", "slide_multiple", "Apply To All")]]),
      R([[S("transitions.on-click", "cursor", "On Mouse Click")], [S("transitions.after", "history", "After")]]),
    ] },
  ] },
  { id: "animations", label: "Animations", key: "A", groups: [
    { label: "Preview", fold: 4, icon: "play", items: [L("animations.preview", "play", "Preview")] },
    { label: "Animation", fold: 3, icon: "sparkle", items: [
      G("animations.gallery", "Animation Styles", ANIMATIONS),
      L("animations.effect-options", "settings", "Effect Options", { menu: ["animations.effect-options"] }),
    ] },
    { label: "Advanced Animation", fold: 2, icon: "add", items: [
      L("animations.add", "sparkle", "Add Animation", { menu: ["animations.add"] }),
      R([[S("animations.pane", "panel_right", "Animation Pane")], [S("animations.trigger", "cursor", "Trigger", { menu: ["animations.trigger"] })], [S("animations.painter", "paint_brush", "Animation Painter")]]),
    ] },
    { label: "Timing", fold: 1, icon: "timer", items: [
      R([[S("animations.start", "play", "Start", { menu: ["animations.start"] })], [S("animations.timing", "timer", "Duration")], [S("animations.delay", "history", "Delay")]]),
      R([[S("animations.earlier", "arrow_previous", "Move Earlier")], [S("animations.later", "arrow_next", "Move Later")]]),
    ] },
  ] },
  { id: "slideshow", label: "Slide Show", key: "S", groups: [
    { label: "Start Slide Show", fold: 3, icon: "play", items: [
      L("slideshow.from-beginning", "play", "From Beginning (F5)"),
      L("slideshow.from-current", "slide_text", "From Current Slide (Shift+F5)"),
      L("slideshow.presenter-view", "presenter", "Presenter View (Alt+F5)"),
    ] },
    { label: "Set Up", fold: 2, icon: "settings", items: [
      L("slideshow.set-up", "settings", "Set Up Slide Show"),
      L("slideshow.hide-slide", "slide_hide", "Hide Slide"),
      L("slideshow.rehearse", "timer", "Rehearse with Coach"),
    ] },
    { label: "Captions & Subtitles", fold: 1, icon: "closed_caption", items: [
      R([[S("slideshow.subtitles", "closed_caption", "Always Use Subtitles")], [S("slideshow.subtitle-settings", "settings", "Subtitle Settings", { menu: ["slideshow.subtitle-settings"] })]]),
    ] },
  ] },
  { id: "review", label: "Review", key: "R", groups: [
    { label: "Proofing", fold: 3, icon: "text_proofing_tools", items: [
      L("review.spelling", "text_proofing_tools", "Spelling (F7)", { menu: ["review.spelling"] }),
      L("review.thesaurus", "book_letter", "Thesaurus (Shift+F7)"),
    ] },
    { label: "Accessibility", fold: 2, icon: "accessibility_checkmark", items: [L("review.accessibility", "accessibility_checkmark", "Check Accessibility")] },
    { label: "Language", fold: 1, icon: "translate", items: [L("review.translate", "translate", "Translate"), L("review.language", "local_language", "Language", { menu: ["review.language"] })] },
    { label: "Comments", fold: 4, icon: "comment", items: [
      L("review.new-comment", "comment_add", "New Comment"),
      R([[S("review.delete-comment", "comment_dismiss", "Delete", { menu: ["review.delete-comment"] })], [S("review.previous-comment", "arrow_previous", "Previous")], [S("review.next-comment", "arrow_next", "Next")]]),
      L("review.show-comments", "comment_multiple", "Show Comments"),
    ] },
  ] },
  { id: "view", label: "View", key: "W", groups: [
    { label: "Presentation Views", fold: 5, icon: "slide_grid", items: [
      L("view.normal", "slide_text", "Normal"),
      L("view.outline", "text_bullet_list_ltr", "Outline View"),
      L("view.slide-sorter", "slide_grid", "Slide Sorter"),
      L("view.notes-page", "document_one_page", "Notes Page"),
      L("view.reading", "book_open", "Reading View"),
    ] },
    { label: "Master Views", fold: 2, icon: "slide_layout", items: [L("view.slide-master", "slide_layout", "Slide Master")] },
    { label: "Show", fold: 3, icon: "grid", items: [
      R([[S("view.ruler", "ruler", "Ruler")], [S("view.gridlines", "grid", "Gridlines")], [S("view.guides", "grid_dots", "Guides")]]),
      R([[S("view.notes", "document", "Notes")]]),
    ] },
    { label: "Zoom", fold: 4, icon: "zoom_in", items: [
      L("view.zoom", "zoom_in", "Zoom", { menu: ZOOM }),
      L("view.zoom.fit", "zoom_fit", "Fit to Window"),
    ] },
    { label: "Color", fold: 1, icon: "color_fill", items: [R([[S("view.grayscale", "color_fill", "Grayscale")]])] },
  ] },
  { id: "help", label: "Help", key: "Y", groups: [
    { label: "Help", fold: 1, icon: "question_circle", items: [
      L("help.help", "question_circle", "Help"),
      L("help.support", "headset", "Contact Support"),
      L("help.feedback", "person_feedback", "Feedback"),
      L("help.keyboard-shortcuts", "keyboard", "Keyboard Shortcuts"),
      L("help.about", "info", "About"),
    ] },
  ] },
  // Contextual: PowerPoint adds it after Help, in the accent, while a shape is selected.
  { id: "shape-format", label: "Shape Format", key: "JD", contextual: "shape", groups: [
    { label: "Insert Shapes", fold: 3, icon: "shapes", items: [
      L("insert.shapes", "shapes", "Shapes", { menu: SHAPES }),
      R([[S("shape.edit", "edit", "Edit Shape", { menu: ["shape.edit-points", "shape.change"] })], [S("insert.text-box", "textbox", "Text Box")]]),
    ] },
    { label: "Shape Styles", fold: 4, icon: "text_edit_style", launcher: "format.shape-pane", items: [
      G("drawing.quick-styles", "Shape Styles", SHAPE_STYLES),
      R([[S("drawing.fill", "color_fill", "Shape Fill", { menu: ["drawing.fill"] })], [S("drawing.outline", "color_line", "Shape Outline", { menu: ["drawing.outline"] })], [S("drawing.effects", "square_shadow", "Shape Effects", { menu: ["drawing.effects"] })]]),
    ] },
    { label: "WordArt Styles", fold: 2, icon: "text_effects", launcher: "format.text-pane", items: [
      R([[S("wordart.fill", "text_color", "Text Fill", { menu: ["wordart.fill"] })], [S("wordart.outline", "text_effects", "Text Outline", { menu: ["wordart.outline"] })], [S("wordart.effects", "text_effects_sparkle", "Text Effects", { menu: ["wordart.effects"] })]]),
    ] },
    { label: "Accessibility", fold: 1, icon: "image_alt_text", items: [L("shape.alt-text", "image_alt_text", "Alt Text")] },
    { label: "Arrange", fold: 5, icon: "position_to_front", items: [
      L("arrange.bring-forward", "position_forward", "Bring Forward (Ctrl+])", { menu: ["arrange.bring-forward", "arrange.bring-to-front"] }),
      L("arrange.send-backward", "position_backward", "Send Backward (Ctrl+[)", { menu: ["arrange.send-backward", "arrange.send-to-back"] }),
      L("arrange.selection-pane", "panel_right", "Selection Pane"),
      R([[S("arrange.align", "align_left", "Align", { menu: ARRANGE.slice(6, 12) })], [S("arrange.group", "group", "Group (Ctrl+G)", { menu: ["arrange.group", "arrange.ungroup"] })], [S("arrange.rotate", "arrow_rotate_clockwise", "Rotate", { menu: ["arrange.rotate-right", "arrange.flip-horizontal"] })]]),
    ] },
    { label: "Size", fold: 6, icon: "slide_size", launcher: "format.size-pane", items: [
      R([[S("shape.height", "table_resize_row", "Height")], [S("shape.width", "table_resize_column", "Width")]]),
    ] },
  ] },
];

/// The File page, in PowerPoint's order. As OpenCalc's: each page is a list of
/// sections, and an item names a command, its icon, and one line of what it does.
const F = (id, icon, label, desc) => ({ id, icon, label, desc });
export const FILE_RAIL = [
  { id: "new", label: "New", icon: "document_add", sections: [
    { tiles: true, items: [F("file.new", "slide_add", "Blank Presentation", "Start a new presentation with one title slide")] },
  ] },
  { id: "open", label: "Open", icon: "folder_open", sections: [
    { items: [F("file.open", "folder_open", "Browse", "Open a PowerPoint presentation (.pptx) from this device. It stays in this browser")] },
  ] },
  { id: "info", label: "Info", icon: "info", sections: [
    { items: [
      F("file.render-report", "warning", "What this slide leaves out", "Features on the current slide this build does not draw yet, by name and count"),
      F("file.properties", "info", "Properties", "Title, author and other properties saved with the presentation"),
      F("file.version-history", "history", "Version History", "See and restore earlier versions of this presentation"),
    ] },
  ] },
  { id: "save", label: "Save a Copy", icon: "save", sections: [
    { items: [
      F("file.save", "save", "Save", "Download this presentation (.pptx). Unedited, it is the file you opened, byte for byte (Ctrl+S)"),
      F("file.save-copy", "save", "Save a Copy", "Download a copy of this presentation (.pptx)"),
    ] },
  ] },
  { id: "export", label: "Export", icon: "arrow_download", sections: [
    { items: [
      F("file.export-pdf", "document_pdf", "Download as PDF", "Save every slide as one PDF document, with text you can search and copy"),
      F("file.export-png", "image", "Download as Image", "Save the current slide as a PNG picture, 1920 pixels wide"),
    ] },
  ] },
  { id: "print", label: "Print", icon: "print", sections: [
    { items: [F("file.print", "print", "Print", "Print the slides (Ctrl+P)")] },
  ] },
  { id: "share", label: "Share", icon: "share", sections: [
    { items: [F("file.share", "people", "Share with people", "Invite others to view or edit this presentation")] },
  ] },
];

/// The right-click menus (docs/53), each a list of command ids with `null` for
/// a separator. Run through the registry, so a verb this build lacks shows
/// disabled, as on the ribbon.
export const CONTEXT_MENUS = {
  slide: ["edit.cut", "edit.copy", "edit.paste", null, "home.new-slide-layout", "home.reset", null, "design.format-background", null, "review.new-comment"],
  thumb: ["edit.cut", "edit.copy", "edit.paste", null, "home.new-slide", "home.duplicate-slide", "home.delete-slide", null, "slide.section.add", null, "home.new-slide-layout", "home.reset", "design.format-background", null, "slideshow.hide-slide", null, "review.new-comment"],
  shape: ["edit.cut", "edit.copy", "edit.paste", null, "shape.edit-text", "shape.edit-points", null, "arrange.group", "arrange.bring-to-front", "arrange.send-to-back", null, "insert.link", "shape.alt-text", "format.size-pane", "format.shape-pane", null, "shape.delete", null, "review.new-comment"],
  text: ["edit.cut", "edit.copy", "edit.paste", null, "font.dialog", "para.dialog", "para.bullets", "para.numbering", null, "insert.link", "format.shape-pane", null, "review.new-comment"],
};

/// Every command id the model names, for registering the ones not built yet
/// as disabled, and for the registry-completeness check OpenCalc runs.
export function modelCommandIds() {
  const ids = new Set();
  const walk = (it) => {
    if (it.kind === "rows") { for (const row of it.rows) row.forEach(walk); return; }
    ids.add(it.id);
    for (const m of it.menu || []) ids.add(m);
  };
  for (const tab of TABS) for (const g of tab.groups) { g.items.forEach(walk); if (g.launcher) ids.add(g.launcher); }
  for (const r of FILE_RAIL) for (const sec of r.sections || []) for (const it of sec.items) ids.add(it.id);
  for (const list of Object.values(CONTEXT_MENUS)) for (const id of list) if (id) ids.add(id);
  for (const id of PRESENT) ids.add(id);
  return [...ids];
}

/// Names for the ids that appear only in a menu or a launcher, in PowerPoint's
/// words, so a disabled one reads as the command it is, not as its id.
export const LABELS = {
  "edit.paste-text": "Keep Text Only (Ctrl+Shift+V)",
  "edit.clipboard": "Clipboard",
  "edit.select-all": "Select All (Ctrl+A)",
  "slide.section.add": "Add Section",
  "slide.section.rename": "Rename Section",
  "slide.section.remove": "Remove Section",
  "slide.section.collapse-all": "Collapse All",
  "slide.section.expand-all": "Expand All",
  "font.dialog": "Font…",
  "para.dialog": "Paragraph…",
  "font.case.sentence": "Sentence case.",
  "font.case.lower": "lowercase",
  "font.case.upper": "UPPERCASE",
  "font.case.title": "Capitalize Each Word",
  "arrange.bring-to-front": "Bring to Front (Ctrl+Shift+])",
  "arrange.bring-forward": "Bring Forward (Ctrl+])",
  "arrange.send-backward": "Send Backward (Ctrl+[)",
  "arrange.send-to-back": "Send to Back (Ctrl+Shift+[)",
  "arrange.group": "Group (Ctrl+G)",
  "arrange.ungroup": "Ungroup (Ctrl+Shift+G)",
  "arrange.align-left": "Align Left",
  "arrange.align-center": "Align Center",
  "arrange.align-right": "Align Right",
  "arrange.align-top": "Align Top",
  "arrange.align-middle": "Align Middle",
  "arrange.align-bottom": "Align Bottom",
  "arrange.rotate-right": "Rotate Right 90°",
  "arrange.flip-horizontal": "Flip Horizontal",
  "arrange.selection-pane": "Selection Pane",
  "shape.edit-text": "Edit Text",
  "shape.edit-points": "Edit Points",
  "shape.change": "Change Shape",
  "format.shape-pane": "Format Shape…",
  "format.text-pane": "Format Text Effects…",
  "format.size-pane": "Size and Position…",
  "home.reset": "Reset Slide",
  "home.new-slide-layout": "Layout",
};

/// The Present button's drop, at the right of the tab row.
export const PRESENT_MENU = PRESENT;
