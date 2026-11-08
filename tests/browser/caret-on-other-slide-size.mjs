// The caret lands on the text on a deck of another slide size (`UX-013`): CI's
// `collab-browser` job, as a second step.
//
// The owner's bug: on a Google Slides deck (10 by 5.63 inches) opened after the
// blank deck (13.33 by 7.5 inches), the caret and selection were drawn about 25%
// short of the text, because the text overlay kept the blank deck's `viewBox`. Both
// are 16:9, so the slide frame does not resize and nothing told the overlay.
//
// This opens the editor (the blank deck), opens `fixtures/real/gs-shapecrawler-074-ru.pptx`
// (9144000 by 5143500 EMU, one centred title, "Title") through the file input, finds
// the title's drawn ink in the stage picture itself, clicks its centre, and asserts the
// `.text-caret` on screen is inside that ink's line: within its left and right edges,
// and spanning its middle. The ink is read from the rendered pixels, not from the
// engine's geometry, so the check does not share the mapping it tests. With the
// overlay left at 12192000 EMU the caret is drawn at 0.75 of its true distance from
// the frame's left edge: about 90 px left of the word on a 730 px frame.
//
// Then two steps of `UX-015`, found editing real decks:
// - Keys typed at once after a click into text go where the click put the caret. On
//   `gs-z11091145-curation-en.pptx` a click in "Curator is assigned..." and an
//   immediate "Q" typed it at the text's end: the shape selection took the key while
//   the box was still opening.
// - A grouped shape's text is entered as PowerPoint's: on `z21464709-souzou-ja.pptx`,
//   a click on a callout inside a group selects the group, a second puts the caret in
//   the callout, and typing goes into it.
// - One click in a table's cell types into that cell, as PowerPoint. On
//   `z17310490-vierfelder-de.pptx` a click selected the table only, the cell calls
//   were refused by the worker, and the cell was found by the rows' stated heights,
//   a row below the one drawn. The cell is found here from the stage picture: the
//   first orange cell, "falsch-positiv", row 2, column 2.
//
// Usage: node tests/browser/caret-on-other-slide-size.mjs <server binary>
// Needs `playwright-core` (installed by the job, pinned) and Chrome on the runner.

import { spawn } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { chromium } from "playwright-core";

const ROOT = path.resolve(fileURLToPath(new URL("../..", import.meta.url)));
const PORT = Number(process.env.OPENPRESENT_BROWSER_PORT ?? 8794);
const ORIGIN = `http://127.0.0.1:${PORT}`;
const DECK = path.join(ROOT, "fixtures/real/gs-shapecrawler-074-ru.pptx");
const FLOW = path.join(ROOT, "fixtures/real/gs-z11091145-curation-en.pptx");
const GROUPED = path.join(ROOT, "fixtures/real/z21464709-souzou-ja.pptx");
const TABLE = path.join(ROOT, "fixtures/real/z17310490-vierfelder-de.pptx");
const server = process.argv[2];

function fail(message) {
  console.error(`caret-on-other-slide-size: ${message}`);
  process.exitCode = 1;
  throw new Error(message);
}

async function waitForServer() {
  for (let i = 0; i < 100; i++) {
    try {
      const r = await fetch(`${ORIGIN}/editor.html`);
      if (r.ok) return;
    } catch { /* not listening yet */ }
    await new Promise((r) => setTimeout(r, 100));
  }
  fail(`the server did not answer on ${ORIGIN}`);
}

/// Poll `fn` in the page until it returns a truthy value, which is returned. Not
/// `page.waitForFunction`, which the editor's CSP refuses (`CI-23`).
async function until(page, fn, what, arg, timeout = 20000) {
  const end = Date.now() + timeout;
  while (Date.now() < end) {
    const v = await page.evaluate(fn, arg);
    if (v) return v;
    await new Promise((r) => setTimeout(r, 200));
  }
  fail(`timed out after ${timeout} ms waiting for ${what}`);
}

/// Open `file` and wait for its first slide's shapes to be read; `name` is its file name.
async function openDeck(page, file, name) {
  await page.setInputFiles("#file-input", file);
  await until(page, (n) => document.title.startsWith(n), `${name} to open`, name);
  await until(page, async () => {
    const sel = await import("/editor.selection.js");
    return sel.allShapes().length > 0;
  }, `the shapes of ${name}`);
}

/// The plain text of shape `node` in part `part`, paragraphs joined by line feeds.
function bodyText(page, part, node) {
  return page.evaluate(async ([part, node]) => {
    const core = await import("/editor.core.js");
    const r = await core.request({ op: "review", fn: "text_body_runs", args: [part, node] });
    return JSON.parse(r.value).map((p) => p.map((run) => run[0]).join("")).join("\n");
  }, [part, node]);
}

/// The page point at the centre of a box in slide EMU.
function centre(page, b) {
  return page.evaluate(async (b) => {
    const core = await import("/editor.core.js");
    const r = document.getElementById("slide-frame").getBoundingClientRect();
    const k = r.width / core.state.deck.width;
    return [r.left + (b.x + b.cx / 2) * k, r.top + (b.y + b.cy / 2) * k];
  }, b);
}

/// The page box of the topmost orange region in the stage picture, by flood fill from
/// its first pixel: on the table deck, the "falsch-positiv" cell.
function orangeCell() {
  const img = document.getElementById("slide-img");
  if (!img.complete || !img.naturalWidth) return null;
  const c = document.createElement("canvas");
  c.width = img.naturalWidth;
  c.height = img.naturalHeight;
  const g = c.getContext("2d");
  g.drawImage(img, 0, 0);
  const d = g.getImageData(0, 0, c.width, c.height).data;
  const orange = (i) => d[i] > 230 && d[i + 1] > 170 && d[i + 1] < 215 && d[i + 2] < 60;
  let start = -1;
  for (let i = 0; i < d.length && start < 0; i += 4) if (orange(i)) start = i / 4;
  if (start < 0) return null;
  const seen = new Uint8Array(c.width * c.height);
  const stack = [start];
  let x0 = Infinity, y0 = Infinity, x1 = -1, y1 = -1;
  seen[start] = 1;
  while (stack.length) {
    const p = stack.pop();
    const x = p % c.width, y = (p - x) / c.width;
    x0 = Math.min(x0, x); x1 = Math.max(x1, x); y0 = Math.min(y0, y); y1 = Math.max(y1, y);
    for (const q of [p - 1, p + 1, p - c.width, p + c.width]) {
      if (q >= 0 && q < seen.length && !seen[q] && orange(q * 4)) { seen[q] = 1; stack.push(q); }
    }
  }
  const r = img.getBoundingClientRect();
  const k = r.width / c.width;
  return { left: r.left + x0 * k, right: r.left + (x1 + 1) * k, top: r.top + y0 * k, bottom: r.top + (y1 + 1) * k };
}

/// The bounding box, in CSS pixels on the page, of everything in the stage picture
/// that differs from its top-left pixel: on this slide, the title's glyphs.
function ink() {
  const img = document.getElementById("slide-img");
  if (!img.complete || !img.naturalWidth) return null;
  const c = document.createElement("canvas");
  c.width = img.naturalWidth;
  c.height = img.naturalHeight;
  const g = c.getContext("2d");
  g.drawImage(img, 0, 0);
  const d = g.getImageData(0, 0, c.width, c.height).data;
  const bg = [d[0], d[1], d[2]];
  let x0 = Infinity, y0 = Infinity, x1 = -1, y1 = -1;
  for (let y = 0; y < c.height; y++) {
    for (let x = 0; x < c.width; x++) {
      const i = (y * c.width + x) * 4;
      if (Math.abs(d[i] - bg[0]) + Math.abs(d[i + 1] - bg[1]) + Math.abs(d[i + 2] - bg[2]) > 96) {
        x0 = Math.min(x0, x); x1 = Math.max(x1, x);
        y0 = Math.min(y0, y); y1 = Math.max(y1, y);
      }
    }
  }
  if (x1 < 0) return null;
  const r = img.getBoundingClientRect();
  const k = r.width / c.width;
  return { left: r.left + x0 * k, right: r.left + (x1 + 1) * k, top: r.top + y0 * k, bottom: r.top + (y1 + 1) * k, frameWidth: r.width };
}

async function main() {
  if (!server) fail("usage: caret-on-other-slide-size.mjs <server binary>");
  const child = spawn(server, [], {
    env: {
      ...process.env,
      OPENPRESENT_COLLAB_BIND: `127.0.0.1:${PORT}`,
      OPENPRESENT_COLLAB_FILES: path.join(ROOT, "webapp"),
    },
    stdio: ["ignore", "inherit", "inherit"],
  });
  let browser;
  try {
    await waitForServer();
    browser = await chromium.launch({ channel: process.env.OPENPRESENT_CHROME_CHANNEL || "chrome", headless: true });
    const page = await (await browser.newContext({ viewport: { width: 1400, height: 900 } })).newPage();
    page.on("dialog", (d) => fail(`a ${d.type()} dialog appeared: ${d.message()}`));
    page.on("pageerror", (e) => fail(`a page error: ${e.message}`));
    await page.goto(`${ORIGIN}/editor.html`);
    await page.locator("#thumbs .thumb").first().waitFor({ state: "attached", timeout: 30000 });
    // The blank deck first, at 12192000 by 6858000, as the owner had it.
    await until(page, async () => {
      const core = await import("/editor.core.js");
      return core.state.deck?.width === 12192000;
    }, "the blank deck");
    const blankSrc = await page.evaluate(() => document.getElementById("slide-img").src);
    await page.setInputFiles("#file-input", DECK);
    await until(page, async () => {
      const core = await import("/editor.core.js");
      return core.state.deck?.width === 9144000 && core.state.deck?.height === 5143500;
    }, "the 10-inch 16:9 deck to open");
    // The stage picture of the new deck, holding only the title's glyphs.
    await until(page, (old) => document.getElementById("slide-img").src !== old, "the new deck on the stage", blankSrc);
    const box = await until(page, ink, "the title drawn on the stage");
    if (box.right - box.left > box.frameWidth / 2) fail(`the stage ink is ${Math.round(box.right - box.left)} px wide: not just the title`);
    const cx = (box.left + box.right) / 2, cy = (box.top + box.bottom) / 2;
    await page.mouse.click(cx, cy);
    const caret = await until(page, () => {
      const c = document.querySelector("#slide-frame .text-caret");
      if (!c) return null;
      const r = c.getBoundingClientRect();
      return { x: r.left + r.width / 2, top: r.top, bottom: r.bottom };
    }, "a caret after clicking the title");
    const fmt = (n) => n.toFixed(1);
    console.log(`ink x ${fmt(box.left)}..${fmt(box.right)} y ${fmt(box.top)}..${fmt(box.bottom)}; caret x ${fmt(caret.x)} y ${fmt(caret.top)}..${fmt(caret.bottom)}`);
    const SLOP = 3;
    if (caret.x < box.left - SLOP || caret.x > box.right + SLOP) {
      fail(`the caret is at x ${fmt(caret.x)}, outside the title's line (${fmt(box.left)} to ${fmt(box.right)})`);
    }
    if (caret.top > cy || caret.bottom < cy) {
      fail(`the caret spans y ${fmt(caret.top)} to ${fmt(caret.bottom)}, missing the title's line at ${fmt(cy)}`);
    }
    console.log("caret-on-other-slide-size: the caret is on the title's drawn line");

    // UX-015: a key typed at once after a click into text goes in at the click.
    await openDeck(page, FLOW, "gs-z11091145-curation-en.pptx");
    const flow = await page.evaluate(async () => {
      const sel = await import("/editor.selection.js");
      const core = await import("/editor.core.js");
      for (const s of sel.allShapes()) {
        if (s.part == null) continue;
        const r = await core.request({ op: "review", fn: "text_body_runs", args: [s.part, s.node] });
        if (r.value.includes("Curator is assigned")) return s;
      }
      return null;
    });
    if (!flow) fail("no shape holds \"Curator is assigned\"");
    const before = await bodyText(page, flow.part, flow.node);
    const [fx, fy] = await centre(page, flow);
    await page.mouse.click(fx, fy);
    await page.keyboard.type("Q");
    const after = await until(page, async ([part, node, n]) => {
      const core = await import("/editor.core.js");
      const r = await core.request({ op: "review", fn: "text_body_runs", args: [part, node] });
      const t = JSON.parse(r.value).map((p) => p.map((run) => run[0]).join("")).join("\n");
      return [...t].length > n ? t : null;
    }, "the typed Q", [flow.part, flow.node, [...before].length]);
    if (after.endsWith("Q") || !after.includes("documented)")) {
      fail(`a key typed just after the click went to the text's end: ${JSON.stringify(after)}`);
    }
    console.log("caret-on-other-slide-size: a key typed at once after a click goes in at the click");

    // UX-015: a second click in a group enters the shape inside, and typing goes there.
    await openDeck(page, GROUPED, "z21464709-souzou-ja.pptx");
    const callout = await page.evaluate(async () => {
      const sel = await import("/editor.selection.js");
      const core = await import("/editor.core.js");
      for (const g of sel.allShapes().filter((s) => s.flags & 64)) {
        const r = await core.request({ op: "build", fn: "group_shapes", args: [0, g.node] });
        for (let i = 0; i + 6 < r.value.length; i += 7) {
          const [node, x, y, cx, cy] = r.value.slice(i, i + 5);
          const h = await core.request({ op: "text", fn: "text_hit", args: [0, x + cx / 2, y + cy / 2] });
          if (h.value[1] === node) return { group: g.node, node, part: h.value[0], x, y, cx, cy };
        }
      }
      return null;
    });
    if (!callout) fail("no shape with text inside a group on slide 1");
    const [gx, gy] = await centre(page, callout);
    await page.mouse.click(gx, gy);
    const first = await until(page, async () => (await import("/editor.selection.js")).selectedNode(), "a selection after the first click");
    if (first !== callout.group) fail(`the first click selected node ${first}, not the group ${callout.group}`);
    await page.mouse.click(gx, gy);
    await page.keyboard.type("XY");
    await until(page, async ([part, node]) => {
      const core = await import("/editor.core.js");
      const r = await core.request({ op: "review", fn: "text_body_runs", args: [part, node] });
      return r.value.includes("XY");
    }, "XY typed into the grouped callout", [callout.part, callout.node]);
    console.log("caret-on-other-slide-size: a second click in a group types into the shape inside");

    // UX-015: one click in a table cell, then typing, types into that cell.
    const groupedSrc = await page.evaluate(() => document.getElementById("slide-img").src);
    await openDeck(page, TABLE, "z17310490-vierfelder-de.pptx");
    await until(page, (old) => document.getElementById("slide-img").src !== old, "the table deck on the stage", groupedSrc);
    const cell = await until(page, orangeCell, "the orange cell drawn on the stage");
    const table = await page.evaluate(async () => (await import("/editor.selection.js")).allShapes().find((s) => s.flags & 256)?.node);
    if (table == null) fail("no table on the slide");
    await page.mouse.click((cell.left + cell.right) / 2, (cell.top + cell.bottom) / 2);
    await page.keyboard.type("Q");
    await until(page, () => document.querySelector(".op-cell-editor"), "the cell editor");
    await page.mouse.click(cell.left - 2, cell.bottom + 200);   // off the table: commits
    const typed = await until(page, async (node) => {
      const core = await import("/editor.core.js");
      const r = await core.request({ op: "build", fn: "table_cell_text", args: [0, node, 1, 1] });
      return r.value.includes("Q") ? r.value : null;
    }, "Q in the falsch-positiv cell", table);
    if (!typed.startsWith("falsch-positiv")) fail(`the cell holds ${JSON.stringify(typed)}`);
    console.log("caret-on-other-slide-size: one click in a table cell types into that cell");
  } finally {
    await browser?.close();
    child.kill();
  }
}

main().catch((e) => {
  console.error(e);
  process.exitCode = 1;
});
