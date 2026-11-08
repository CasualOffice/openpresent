// Co-editing in two browser pages (`COL-04`): CI's `collab-browser` job.
//
// Starts the collaboration server on 127.0.0.1, serving this checkout's `webapp/`
// (with its freshly built `pkg/`), and drives two headless Chrome pages as two
// people, as OpenCalc's `collab.spec.mjs` does with Playwright: Ada opens the
// editor and shares, Grace opens the link, both type into the title at once, Ada
// makes the title bold, and Grace undoes her own typing. Both pages must end with
// the same deck, byte for byte, and neither may raise an alert or confirm dialog.
//
// Usage: node tests/browser/collab-two-pages.mjs <server binary>
// Needs `playwright-core` (installed by the job, pinned) and Chrome on the runner.

import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { chromium } from "playwright-core";

const ROOT = path.resolve(fileURLToPath(new URL("../..", import.meta.url)));
const PORT = Number(process.env.OPENPRESENT_BROWSER_PORT ?? 8793);
const ORIGIN = `http://127.0.0.1:${PORT}`;
const server = process.argv[2];

function fail(message) {
  console.error(`collab-two-pages: ${message}`);
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

/// Poll `fn` in the page until it returns true. Not `page.waitForFunction`: that
/// rebuilds the predicate from a string inside the page, which the editor's CSP
/// (`script-src 'self' 'wasm-unsafe-eval'`, no `unsafe-eval`) rightly refuses (`CI-23`).
/// `page.evaluate` sends the function over the DevTools protocol instead, which CSP
/// does not govern, so the editor's policy stays exactly as it ships.
async function until(page, fn, what, timeout = 20000) {
  const end = Date.now() + timeout;
  while (Date.now() < end) {
    if (await page.evaluate(fn)) return;
    await new Promise((r) => setTimeout(r, 200));
  }
  fail(`timed out after ${timeout} ms waiting for ${what}`);
}

/// Open the editor and wait for its engine and first slide.
async function open(page, url) {
  page.on("dialog", (d) => fail(`a ${d.type()} dialog appeared: ${d.message()}`));
  page.on("pageerror", (e) => fail(`a page error: ${e.message}`));
  await page.goto(url);
  await page.locator("#thumbs .thumb").first().waitFor({ state: "attached", timeout: 30000 });
}

/// The title placeholder's centre on the blank deck's first slide.
async function clickTitle(page) {
  const frame = await page.locator("#slide-frame").boundingBox();
  await page.mouse.click(frame.x + frame.width / 2, frame.y + frame.height * 0.36);
}

/// Wait until this page has nothing unacknowledged and nothing arrives for a while.
async function settle(page) {
  await until(page, async () => {
    const core = await import("/editor.core.js");
    const r = await core.request({ op: "collab", fn: "collab_unacknowledged", args: [] }, { priority: 9 });
    return r.value === false;
  }, "every edit to be acknowledged");
}

async function bytesOf(page) {
  const b64 = await page.evaluate(async () => {
    const core = await import("/editor.core.js");
    const bytes = await core.deckBytes();
    let s = "";
    for (const b of bytes) s += String.fromCharCode(b);
    return btoa(s);
  });
  return Buffer.from(b64, "base64");
}

async function main() {
  if (!server) fail("usage: collab-two-pages.mjs <server binary>");
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
    const ada = await (await browser.newContext()).newPage();
    const grace = await (await browser.newContext()).newPage();
    await open(ada, `${ORIGIN}/editor.html`);
    // Share, as a person does: the Share button, a name, Start sharing.
    await ada.click(".rb-share");
    await ada.fill("#share-name", "Ada");
    await ada.click("#share-start");
    await ada.waitForURL(/\?room=/);
    const link = ada.url();
    await ada.keyboard.press("Escape");
    await grace.addInitScript(() => sessionStorage.setItem("op.name", "Grace"));
    await open(grace, link);
    await until(ada, () => document.querySelectorAll("#presence-people .people-badge").length === 2, "two people present");
    // Both type into the title at once.
    await clickTitle(ada);
    await clickTitle(grace);
    await Promise.all([ada.keyboard.type("Hello", { delay: 30 }), grace.keyboard.type("World", { delay: 30 })]);
    await settle(ada);
    await settle(grace);
    // Ada makes the title bold; Grace undoes her own typing.
    await ada.keyboard.press("Control+a");
    await ada.keyboard.press("Control+b");
    await grace.keyboard.press("Control+z");
    await settle(ada);
    await settle(grace);
    await new Promise((r) => setTimeout(r, 1000));
    await settle(ada);
    await settle(grace);
    const [a, g] = [await bytesOf(ada), await bytesOf(grace)];
    const sha = (b) => createHash("sha256").update(b).digest("hex");
    console.log(`Ada ${a.length} bytes ${sha(a)}`);
    console.log(`Grace ${g.length} bytes ${sha(g)}`);
    if (sha(a) !== sha(g)) fail("the two pages hold different decks");
    const text = await grace.evaluate(() => document.querySelector("#thumbs .thumb")?.getAttribute("aria-label"));
    console.log(`collab-two-pages: both pages saved the same deck (${text})`);
  } finally {
    await browser?.close();
    child.kill();
  }
}

main().catch((e) => {
  console.error(e);
  process.exitCode = 1;
});
