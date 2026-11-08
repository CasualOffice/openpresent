/// The ribbon, as data. Copied in shape from OpenCalc's `webapp/ribbon.model.js`
/// at `b23c67c` (webapp/README.md, owner instruction 42): the same `L`, `S`, `R`
/// builders, `fold` ranks and File page model. The content is PowerPoint's: its
/// tabs in its order (Home, Insert, Design, Transitions, Animations, Slide Show,
/// Review, View), its groups, and within a group its large buttons and stacks
/// (docs/47, docs/49).
///
/// **Where this differs from OpenCalc.** OpenCalc leaves out a verb it has not
/// built (its docs/91 §8: a dead button advertises an absence). The owner asked
/// for the opposite here (2026-11-08): every PowerPoint command is present, and
/// the ones this build cannot run yet are disabled through the command registry,
/// never faked. `editor.core.js` registers each id below that has no working
/// command as `enabled: () => false`.
///
/// Shapes:
///   L(id, icon, label, {menu?})  a large button: 24px icon, 72px column
///   S(id, icon, label, {menu?})  a small button: 16px icon, 24px row
///   R([[S…], [S…], …])           a column of up to three rows
/// `menu` lists the command ids the button drops. `fold` on a group is its
/// collapse rank, lowest first. Icons are names in `icons.fluent.svg`.

const L = (id, icon, label, o = {}) => ({ kind: "large", id, icon, label, ...o });
const S = (id, icon, label, o = {}) => ({ kind: "small", id, icon, label, ...o });
const R = (rows) => ({ kind: "rows", rows });

const NEW_SLIDE = ["home.new-slide", "home.new-slide-layout", "home.duplicate-slide", "home.delete-slide", "home.reuse-slides"];
const ZOOM = ["view.zoom.fit", "view.zoom.50", "view.zoom.75", "view.zoom.100", "view.zoom.150", "view.zoom.200"];

export const TABS = [
  { id: "home", label: "Home", key: "H", groups: [
    { label: "Clipboard", fold: 6, icon: "clipboard_paste", items: [
      L("edit.paste", "clipboard_paste", "Paste (Ctrl+V)"),
      R([[S("edit.cut", "cut", "Cut (Ctrl+X)")], [S("edit.copy", "copy", "Copy (Ctrl+C)")], [S("home.format-painter", "paint_brush", "Format Painter")]]),
    ] },
    { label: "Slides", fold: 7, icon: "document_add", items: [
      L("home.new-slide", "document_add", "New Slide (Ctrl+M)", { menu: NEW_SLIDE }),
      R([[S("home.layout", "table_settings", "Layout")], [S("home.reset", "arrow_clockwise", "Reset")], [S("home.section", "tab_desktop_multiple", "Section")]]),
    ] },
    { label: "Font", fold: 9, icon: "text_color", items: [
      R([
        [S("font.increase", "font_increase", "Increase Font Size (Ctrl+Shift+>)"), S("font.decrease", "font_decrease", "Decrease Font Size (Ctrl+Shift+<)"), S("font.clear", "eraser", "Clear All Formatting")],
        [S("font.bold", "text_bold", "Bold (Ctrl+B)"), S("font.italic", "text_italic", "Italic (Ctrl+I)"), S("font.underline", "text_underline", "Underline (Ctrl+U)"), S("font.strike", "text_strikethrough", "Strikethrough"),
         S("font.subscript", "text_subscript", "Subscript"), S("font.superscript", "text_superscript", "Superscript"), S("font.case", "text_case_title", "Change Case"), S("font.color", "text_color", "Font Color")],
      ]),
    ] },
    { label: "Paragraph", fold: 8, icon: "text_align_left", items: [
      R([
        [S("para.indent-less", "text_indent_decrease", "Decrease List Level"), S("para.indent-more", "text_indent_increase", "Increase List Level"), S("para.direction", "text_direction_rotate_270_right", "Text Direction")],
        [S("para.left", "text_align_left", "Align Left (Ctrl+L)"), S("para.center", "text_align_center", "Center (Ctrl+E)"), S("para.right", "text_align_right", "Align Right (Ctrl+R)"),
         S("para.align-text", "align_center_vertical", "Align Text")],
      ]),
    ] },
    { label: "Drawing", fold: 5, icon: "shapes", items: [
      L("insert.shapes", "shapes", "Shapes"),
      L("drawing.arrange", "table_cells_merge", "Arrange"),
      L("drawing.quick-styles", "style_guide", "Quick Styles"),
      R([[S("drawing.fill", "paint_bucket", "Shape Fill")], [S("drawing.outline", "border_outside", "Shape Outline")]]),
    ] },
    { label: "Editing", fold: 4, icon: "search", items: [
      R([[S("edit.find", "search", "Find (Ctrl+F)")], [S("edit.replace", "text_wrap", "Replace (Ctrl+H)")], [S("edit.select", "checkmark", "Select")]]),
    ] },
  ] },
  { id: "insert", label: "Insert", key: "N", groups: [
    { label: "Slides", fold: 7, icon: "document_add", items: [L("home.new-slide", "document_add", "New Slide (Ctrl+M)", { menu: NEW_SLIDE })] },
    { label: "Tables", fold: 6, icon: "table", items: [L("insert.table", "table", "Table")] },
    { label: "Images", fold: 5, icon: "image", items: [L("insert.pictures", "image", "Pictures")] },
    { label: "Illustrations", fold: 4, icon: "shapes", items: [
      L("insert.shapes", "shapes", "Shapes"),
      L("insert.smartart", "data_area", "SmartArt"),
      L("insert.chart", "data_bar_vertical", "Chart"),
    ] },
    { label: "Links", fold: 3, icon: "link", items: [L("insert.link", "link", "Link (Ctrl+K)")] },
    { label: "Comments", fold: 2, icon: "comment_add", items: [L("review.new-comment", "comment_add", "Comment")] },
    { label: "Text", fold: 3, icon: "textbox", items: [
      L("insert.text-box", "textbox", "Text Box"),
      L("insert.header-footer", "document_one_page", "Header & Footer"),
    ] },
    { label: "Symbols", fold: 1, icon: "math_formula", items: [
      L("insert.equation", "math_formula", "Equation"),
      L("insert.symbol", "asterisk", "Symbol"),
    ] },
  ] },
  { id: "design", label: "Design", key: "G", groups: [
    { label: "Themes", fold: 3, icon: "style_guide", items: [L("design.themes", "style_guide", "Themes")] },
    { label: "Variants", fold: 2, icon: "paint_bucket", items: [L("design.variants", "paint_bucket", "Variants")] },
    { label: "Customize", fold: 4, icon: "table_resize_column", items: [
      L("design.slide-size", "table_resize_column", "Slide Size"),
      L("design.format-background", "paint_bucket", "Format Background"),
    ] },
  ] },
  { id: "transitions", label: "Transitions", key: "K", groups: [
    { label: "Preview", fold: 3, icon: "play", items: [L("transitions.preview", "play", "Preview")] },
    { label: "Transition to This Slide", fold: 2, icon: "tab_desktop_multiple", items: [
      L("transitions.gallery", "tab_desktop_multiple", "Transitions"),
      L("transitions.effect-options", "settings", "Effect Options"),
    ] },
    { label: "Timing", fold: 1, icon: "history", items: [
      R([[S("transitions.apply-to-all", "checkmark", "Apply To All")], [S("transitions.timing", "history", "Duration")]]),
    ] },
  ] },
  { id: "animations", label: "Animations", key: "A", groups: [
    { label: "Preview", fold: 4, icon: "play", items: [L("animations.preview", "play", "Preview")] },
    { label: "Animation", fold: 3, icon: "asterisk", items: [
      L("animations.gallery", "asterisk", "Animation Styles"),
      L("animations.effect-options", "settings", "Effect Options"),
    ] },
    { label: "Advanced Animation", fold: 2, icon: "add", items: [
      L("animations.add", "add", "Add Animation"),
      R([[S("animations.pane", "tab_desktop_multiple", "Animation Pane")], [S("animations.trigger", "play", "Trigger")], [S("animations.painter", "paint_brush", "Animation Painter")]]),
    ] },
    { label: "Timing", fold: 1, icon: "history", items: [R([[S("animations.timing", "history", "Duration")]])] },
  ] },
  { id: "slideshow", label: "Slide Show", key: "S", groups: [
    { label: "Start Slide Show", fold: 3, icon: "play", items: [
      L("slideshow.from-beginning", "play", "From Beginning (F5)"),
      L("slideshow.from-current", "play", "From Current Slide (Shift+F5)"),
    ] },
    { label: "Set Up", fold: 2, icon: "settings", items: [
      L("slideshow.set-up", "settings", "Set Up Slide Show"),
      L("slideshow.hide-slide", "eye", "Hide Slide"),
      R([[S("slideshow.rehearse", "history", "Rehearse Timings")]]),
    ] },
  ] },
  { id: "review", label: "Review", key: "R", groups: [
    { label: "Proofing", fold: 2, icon: "checkmark", items: [L("review.spelling", "checkmark", "Spelling (F7)")] },
    { label: "Accessibility", fold: 1, icon: "eye", items: [L("review.accessibility", "eye", "Check Accessibility")] },
    { label: "Comments", fold: 3, icon: "comment", items: [
      L("review.new-comment", "comment_add", "New Comment"),
      L("review.show-comments", "comment", "Show Comments"),
    ] },
    { label: "Compare", fold: 1, icon: "tab_desktop_multiple", items: [L("review.compare", "tab_desktop_multiple", "Compare")] },
  ] },
  { id: "view", label: "View", key: "W", groups: [
    { label: "Presentation Views", fold: 5, icon: "grid", items: [
      L("view.normal", "tab_desktop_multiple", "Normal"),
      L("view.outline", "text_align_left", "Outline View"),
      L("view.slide-sorter", "grid", "Slide Sorter"),
      L("view.notes-page", "document_one_page", "Notes Page"),
      L("view.reading", "book_open", "Reading View"),
    ] },
    { label: "Master Views", fold: 2, icon: "table_settings", items: [L("view.slide-master", "table_settings", "Slide Master")] },
    { label: "Show", fold: 3, icon: "grid", items: [
      R([[S("view.ruler", "table_resize_row", "Ruler")], [S("view.gridlines", "grid", "Gridlines")], [S("view.notes", "document", "Notes")]]),
    ] },
    { label: "Zoom", fold: 4, icon: "zoom_in", items: [
      L("view.zoom", "zoom_in", "Zoom", { menu: ZOOM }),
      L("view.zoom.fit", "search", "Fit to Window"),
    ] },
  ] },
  { id: "help", label: "Help", key: "Y", groups: [
    { label: "Help", fold: 1, icon: "question_circle", items: [
      L("help.keyboard-shortcuts", "keyboard", "Keyboard Shortcuts"),
      L("help.about", "info", "About"),
    ] },
  ] },
];

/// The File page, in PowerPoint's order. As OpenCalc's: each page is a list of
/// sections, and an item names a command, its icon, and one line of what it does.
const I = (id, icon, label, desc) => ({ id, icon, label, desc });
export const FILE_RAIL = [
  { id: "new", label: "New", icon: "document_add", sections: [
    { items: [I("file.new", "document_add", "Blank Presentation", "Start a new presentation with one title slide")] },
  ] },
  { id: "open", label: "Open", icon: "folder_open", sections: [
    { items: [I("file.open", "folder_open", "Browse", "Open a PowerPoint presentation (.pptx) from this device. It stays in this browser")] },
  ] },
  { id: "info", label: "Info", icon: "info", sections: [
    { items: [
      I("file.render-report", "warning", "What this slide leaves out", "Features on the current slide this build does not draw yet, by name and count"),
      I("file.properties", "info", "Properties", "Title, author and other properties saved with the presentation"),
    ] },
  ] },
  { id: "save", label: "Save", icon: "save", sections: [
    { items: [
      I("file.save", "save", "Save", "Download this presentation (.pptx). Unedited, it is the file you opened, byte for byte (Ctrl+S)"),
      I("file.save-copy", "save", "Save a copy", "Download a copy of this presentation (.pptx)"),
    ] },
  ] },
  { id: "export", label: "Export", icon: "document_pdf", sections: [
    { items: [
      I("file.export-pdf", "document_pdf", "Create PDF", "Save every slide as one PDF document, with text you can search and copy"),
      I("file.export-png", "image", "Export slide as PNG", "Save the current slide as a PNG picture, 1920 pixels wide"),
    ] },
  ] },
  { id: "print", label: "Print", icon: "print", sections: [
    { items: [I("file.print", "print", "Print", "Print the slides (Ctrl+P)")] },
  ] },
  { id: "share", label: "Share", icon: "share", sections: [
    { items: [I("file.share", "people", "Share with people", "Invite others to view or edit this presentation")] },
  ] },
];

/// Every command id the model names, for registering the ones not built yet
/// as disabled, and for the registry-completeness check OpenCalc runs.
export function modelCommandIds() {
  const ids = new Set();
  const walk = (it) => {
    if (it.kind === "rows") { for (const row of it.rows) row.forEach(walk); return; }
    ids.add(it.id);
    for (const m of it.menu || []) ids.add(m);
  };
  for (const tab of TABS) for (const g of tab.groups) g.items.forEach(walk);
  for (const r of FILE_RAIL) for (const sec of r.sections || []) for (const it of sec.items) ids.add(it.id);
  return [...ids];
}
