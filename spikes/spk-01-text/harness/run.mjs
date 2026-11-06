// Driver: node run.mjs <env> [part...]. env: shell | chrome | chrome-gpu | headed
// Parts: info throughput frames gpu raf appearance agreement. Writes results/<env>-*.json and PNGs.
import { chromium } from 'playwright';
import { mkdir, writeFile, readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { serve } from './server.mjs';

const [envName = 'shell', ...parts] = process.argv.slice(2);
const want = (p) => parts.length === 0 || parts.includes(p);
const OUT = fileURLToPath(new URL('../results/', import.meta.url));
await mkdir(OUT, { recursive: true });

const UNCAPPED = ['--disable-frame-rate-limit', '--disable-gpu-vsync'];
const ENVS = {
  // Playwright's chromium-headless-shell: the old headless, software raster by default.
  shell: { args: [] },
  // Installed Google Chrome, new headless mode.
  chrome: { channel: 'chrome', args: [] },
  // Installed Google Chrome, new headless, with the Metal/ANGLE GPU asked for.
  'chrome-gpu': { channel: 'chrome', args: ['--enable-gpu-rasterization', '--ignore-gpu-blocklist', '--use-angle=metal'] },
  // Linux container, see docs/64.
  linux: { args: [] },
};
const cfg = ENVS[envName];
const srv = await serve();
const URL_ = `http://127.0.0.1:${srv.address().port}/`;

async function fresh(args = [], extra = {}) {
  const browser = await chromium.launch({
    headless: true, channel: cfg.channel, args: [...cfg.args, ...args],
  });
  const page = await browser.newPage({ viewport: { width: 1400, height: 900 }, ...extra });
  page.on('pageerror', (e) => console.error('pageerror', e.message));
  await page.goto(URL_);
  await page.waitForFunction('window.spkReady === true');
  return { browser, page };
}
const save = (name, data) => writeFile(`${OUT}${envName}-${name}.json`, JSON.stringify(data, null, 1));
const savePng = (name, url) => writeFile(`${OUT}${envName}-${name}.png`, Buffer.from(url.split(',')[1], 'base64'));

if (want('info')) {
  const { browser, page } = await fresh();
  const version = browser.version();
  const isolated = await page.evaluate(() => window.crossOriginIsolated);
  const gl = await page.evaluate(() => {
    const c = document.createElement('canvas').getContext('webgl');
    if (!c) return null;
    const e = c.getExtension('WEBGL_debug_renderer_info');
    return e ? c.getParameter(e.UNMASKED_RENDERER_WEBGL) : 'no debug info';
  });
  let gpu = null;
  try {
    const p2 = await browser.newPage();
    await p2.goto('chrome://gpu');
    await p2.waitForTimeout(1500);
    gpu = await p2.evaluate(() => {
      const text = document.querySelector('info-view')?.shadowRoot?.textContent ?? document.body.innerText;
      const m = text.match(/Canvas:\s*[^\n]{0,80}/);
      const r = text.match(/Rasterization:\s*[^\n]{0,80}/);
      return { canvas: m?.[0] ?? null, raster: r?.[0] ?? null };
    });
    await p2.close();
  } catch (e) {
    gpu = { error: String(e).slice(0, 200) };
  }
  const idle = await page.evaluate(() => window.spk.idleRaf());
  const data = { version, isolated, webgl_renderer: gl, chrome_gpu: gpu, idle_raf_ms: idle };
  await save('info', data);
  console.log(data);
  await browser.close();
}

if (want('throughput')) {
  // Cold: one fresh browser per sample per case.
  const cold = {};
  let instantiate = [];
  let register = [];
  const names = [];
  for (const tag of ['o3', 'small']) {
    for (let caseIndex = 0; caseIndex < 5; caseIndex++) {
      for (let rep = 0; rep < 5; rep++) {
        const { browser, page } = await fresh();
        const rep0 = await page.evaluate((t) => window.spk.init(t), tag);
        const ms = await page.evaluate((i) => window.spk.cold(i), caseIndex);
        const name = await page.evaluate((i) => window.spk.CASES[i].name, caseIndex);
        (cold[`${tag}:${name}`] ??= []).push(ms);
        if (!names.includes(name)) names.push(name);
        instantiate.push({ tag, ms: rep0.instantiate_ms });
        register.push({ tag, ...rep0.register_ms });
        await browser.close();
      }
    }
  }
  const warm = {};
  for (const tag of ['o3', 'small']) {
    const { browser, page } = await fresh();
    await page.evaluate((t) => window.spk.init(t), tag);
    warm[tag] = await page.evaluate(() => window.spk.warm());
    await browser.close();
  }
  await save('throughput', { cold, instantiate, register, warm });
  console.log('throughput done');
}

const KINDS = ['latin', 'arabic', 'cjk', 'han'];
const MODES = ['glyph_path', 'shape_path', 'cluster_filltext', 'line_filltext'];

if (want('frames')) {
  const rows = [];
  const REPS = 5; // Cold samples are fresh browsers: Skia's strike cache is per process.
  for (const dpr of [1, 2]) {
    for (const kind of KINDS) {
      for (const mode of MODES) {
        const samples = [];
        for (let rep = 0; rep < REPS; rep++) {
          const { browser, page } = await fresh();
          await page.evaluate(() => window.spk.init('o3'));
          samples.push(await page.evaluate((a) => window.spk.denseFrames(a), { kind, mode, dpr }));
          await browser.close();
        }
        rows.push({ kind, mode, dpr, samples });
        const s = samples[samples.length - 1];
        console.log(kind, mode, dpr, 'first', samples.map((x) => x.first_frame_ms.toFixed(1)).join(' '), 'cached', s.cached_total_ms.median.toFixed(2));
      }
    }
  }
  await save('frames', rows);
}

if (want('gpu')) {
  const rows = [];
  for (const dpr of [1, 2]) {
    for (const kind of KINDS) {
      for (const mode of MODES) {
        const samples = [];
        for (let rep = 0; rep < 3; rep++) {
          const { browser, page } = await fresh();
          await page.evaluate(() => window.spk.init('o3'));
          samples.push(await page.evaluate((a) => window.spk.denseGpu(a), { kind, mode, dpr }));
          await browser.close();
        }
        rows.push({ kind, mode, dpr, samples });
        console.log(kind, mode, dpr, 'cold', samples.map((x) => x.first_frame_ms.toFixed(1)).join(' '), 'cached', samples.map((x) => x.cached_ms.median.toFixed(2)).join(' '), 'empty', samples[0].empty_ms.median.toFixed(2));
      }
    }
  }
  await save('gpu', rows);
}

if (want('raf')) {
  const rows = [];
  for (const dpr of [1, 2]) {
    for (const kind of ['latin', 'cjk']) {
      for (const mode of MODES) {
        const { browser, page } = await fresh(UNCAPPED);
        await page.evaluate(() => window.spk.init('o3'));
        const idle = await page.evaluate(() => window.spk.idleRaf());
        const r = await page.evaluate((a) => window.spk.denseRaf(a), { kind, mode, dpr });
        rows.push({ ...r, idle_raf_ms: idle });
        console.log(kind, mode, dpr, 'raf', r.raf_interval_ms.median.toFixed(2), 'submit', r.submit_ms.median.toFixed(2), 'idle', idle.median.toFixed(2));
        await browser.close();
      }
    }
  }
  await save('raf', rows);
}

if (want('appearance')) {
  const { browser, page } = await fresh();
  await page.evaluate(() => window.spk.init('o3'));
  const metrics = {};
  for (const [name, opaque] of [['opaque', true], ['alpha', false]]) {
    const r = await page.evaluate((o) => window.spk.appearance({ opaque: o }), opaque);
    await savePng(`appearance-${name}`, r.png);
    metrics[name] = r.metrics;
  }
  await savePng('appearance-mag', await page.evaluate(() => window.spk.magnified()));
  await savePng('sidebyside', await page.evaluate(() => window.spk.sideBySide()));
  for (const [tag, spec] of [
    ['crop-carlito-10pt-1x', { family: 'SpkCarlito', stack: ['carlito'], pt: 10, s: 1 }],
    ['crop-carlito-12pt-1x', { family: 'SpkCarlito', stack: ['carlito'], pt: 12, s: 1 }],
    ['crop-carlito-18pt-1x', { family: 'SpkCarlito', stack: ['carlito'], pt: 18, s: 1 }],
    ['crop-carlito-10pt-2x', { family: 'SpkCarlito', stack: ['carlito'], pt: 10, s: 2 }],
    ['crop-carlito-10pt-half', { family: 'SpkCarlito', stack: ['carlito'], pt: 10, s: 0.5 }],
    ['crop-liberation-10pt-1x', { family: 'SpkLiberation', stack: ['liberation'], pt: 10, s: 1 }],
  ]) {
    await savePng(tag, await page.evaluate((a) => window.spk.pixelCrop(a), spec));
  }
  const probe = await page.evaluate(() => window.spk.subpixelProbe());
  await save('appearance', { metrics, probe });
  await savePng('dense-glyph-path', await page.evaluate(() => window.spk.renderDense({ kind: 'latin', mode: 'glyph_path', dpr: 1 })));
  await savePng('dense-cluster', await page.evaluate(() => window.spk.renderDense({ kind: 'latin', mode: 'cluster_filltext', dpr: 1 })));
  await browser.close();
  console.log('appearance done');
}

if (want('agreement')) {
  const { browser, page } = await fresh();
  await page.evaluate(() => window.spk.init('o3'));
  const data = await page.evaluate(() => window.spk.agreement());
  await save('agreement', data);
  await savePng('fallback', await page.evaluate(() => window.spk.fallbackPicture()));
  console.log(JSON.stringify(data, null, 1));
  await browser.close();
}

srv.close();
