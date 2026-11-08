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
  } finally {
    await browser?.close();
    child.kill();
  }
}

main().catch((e) => {
  console.error(e);
  process.exitCode = 1;
});
