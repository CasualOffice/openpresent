// Browser side of the spike. The driver (run.mjs) calls window.spk.* from Playwright.
import { CASES, FONTS, coldLayout, registerFaces, stats, warmThroughput } from './bench_core.js';

const now = () => performance.now();
const flush = (ctx) => ctx.getImageData(0, 0, 1, 1); // Forces the deferred canvas queue.
const raf = () => new Promise(requestAnimationFrame);

const st = { wasm: null, engine: null, ids: null, family: {}, bytes: {} };

async function init(tag = 'o3') {
  const report = {};
  let t = now();
  st.wasm = await import(`/pkg/${tag}/spk_text_wasm.js`);
  const exports_ = await st.wasm.default({ module_or_path: `/pkg/${tag}/spk_text_wasm_bg.wasm` });
  report.instantiate_ms = now() - t;
  st.exports = exports_;
  for (const [id, file, family] of FONTS) {
    const buf = await (await fetch('/fonts/' + file)).arrayBuffer();
    st.bytes[id] = new Uint8Array(buf);
    st.family[id] = family;
    const face = new FontFace(family, buf);
    await face.load();
    document.fonts.add(face);
  }
  st.engine = new st.wasm.Engine();
  const r = registerFaces(st.engine, st.bytes);
  st.ids = r.ids;
  report.register_ms = r.ms;
  st.nameByFace = {};
  for (const [id] of FONTS) st.nameByFace[st.ids[id]] = id;
  report.memory_after_fonts_bytes = exports_.memory.buffer.byteLength;
  st.memory = report.memory_after_fonts_bytes;
  return report;
}

// (2) throughput ----------------------------------------------------------------
function cold(caseIndex) {
  return coldLayout(st.wasm, st.engine, st.ids, CASES[caseIndex]);
}
function warm() {
  return warmThroughput(st.wasm, st.engine, st.ids);
}

// Scene --------------------------------------------------------------------------
const COLORS = ['#1f2937', '#7c2d12', '#14532d', '#1e3a8a', '#581c87', '#0f172a', '#3f3f46', '#134e4a'];
const FILLS = ['#f8fafc', '#fff7ed', '#f0fdf4', '#eff6ff', '#faf5ff', '#f1f5f9', '#fafafa', '#f0fdfa'];
const SIZES = [10, 11, 12, 13, 14];

const STACKS = {
  latin: ['carlito'], latin2: ['liberation'], arabic: ['arabic', 'carlito'],
  cjk: ['cjk', 'carlito'], han: ['cjk', 'carlito'],
};

function buildScene(kind) {
  const corpusKind = kind === 'latin2' ? 'latin' : kind;
  const body = [...st.wasm.corpus(corpusKind, 5000, 11)];
  const stack = Uint32Array.from(STACKS[kind].map((s) => st.ids[s]));
  const shapes = [];
  const W = 62;
  const H = 70;
  const t0 = now();
  for (let i = 0; i < 200; i++) {
    const col = i % 20;
    const row = Math.floor(i / 20);
    const text = body.slice(i * 25, (i + 1) * 25).join('');
    const size = SIZES[i % SIZES.length];
    const r = st.engine.layout(text, stack, size, W - 6);
    const gids = r.gids();
    const faces = r.faces();
    const xs = r.xs();
    const ys = r.ys();
    const cs = r.cl_start16();
    const ce = r.cl_end16();
    const cx = r.cl_x();
    const cy = r.cl_y();
    const cf = r.cl_face();
    const crtl = r.cl_rtl();
    const clusters = [];
    for (let c = 0; c < cs.length; c++) {
      clusters.push({ text: text.slice(cs[c], ce[c]), x: cx[c], y: cy[c], face: cf[c], rtl: crtl[c] });
    }
    const ls = r.line_start16();
    const le = r.line_end16();
    const lx = r.line_x();
    const ly = r.line_y();
    const lrtl = r.line_rtl();
    const lines = [];
    for (let l = 0; l < ls.length; l++) {
      lines.push({ text: text.slice(ls[l], le[l]), x: lx[l], y: ly[l], rtl: lrtl[l] });
    }
    r.free();
    shapes.push({
      x: col * 64 + 1, y: row * 72 + 1, w: W, h: H, pad: 3, size,
      color: COLORS[i % COLORS.length], fill: FILLS[i % FILLS.length],
      gids, faces, xs, ys, clusters, lines,
      family: st.family[STACKS[kind][0]], compound: null,
    });
  }
  const layout_ms = now() - t0;
  const uniq = new Set();
  let glyphs = 0;
  for (const s of shapes) {
    glyphs += s.gids.length;
    for (let g = 0; g < s.gids.length; g++) uniq.add(s.faces[g] * 1e6 + s.gids[g]);
  }
  return { kind, shapes, layout_ms, glyphs, unique_glyphs: uniq.size };
}

// Painters -----------------------------------------------------------------------
class PathCache {
  constructor() { this.map = new Map(); this.load_ms = 0; this.misses = 0; this.svg_bytes = 0; }
  get(face, gid) {
    const key = face * 1e6 + gid;
    let p = this.map.get(key);
    if (p === undefined) {
      const t = now();
      const svg = st.engine.glyph_svg(face, gid);
      this.svg_bytes += svg.length;
      p = new Path2D(svg);
      this.map.set(key, p);
      this.misses++;
      this.load_ms += now() - t;
    }
    return p;
  }
}

function scaleOf(face, size) {
  return size / st.engine.units_per_em(face);
}

function boxes(ctx, sh, dpr) {
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.fillStyle = sh.fill;
  ctx.fillRect(sh.x, sh.y, sh.w, sh.h);
  ctx.strokeStyle = sh.color;
  ctx.lineWidth = 1;
  ctx.strokeRect(sh.x + 0.5, sh.y + 0.5, sh.w - 1, sh.h - 1);
}

const painters = {
  // ADR-005 as written: one cached Path2D per glyph, placed by transform.
  glyph_path(ctx, scene, dpr, cache) {
    for (const sh of scene.shapes) {
      boxes(ctx, sh, dpr);
      ctx.fillStyle = sh.color;
      const ox = sh.x + sh.pad;
      const oy = sh.y + sh.pad;
      for (let g = 0; g < sh.gids.length; g++) {
        const face = sh.faces[g];
        const k = scaleOf(face, sh.size) * dpr;
        ctx.setTransform(k, 0, 0, -k, (ox + sh.xs[g]) * dpr, (oy + sh.ys[g]) * dpr);
        ctx.fill(cache.get(face, sh.gids[g]));
      }
    }
  },
  // One compound Path2D per shape, built from the glyph cache and kept with the shape.
  shape_path(ctx, scene, dpr, cache) {
    for (const sh of scene.shapes) {
      boxes(ctx, sh, dpr);
      if (!sh.compound) {
        const p = new Path2D();
        for (let g = 0; g < sh.gids.length; g++) {
          const face = sh.faces[g];
          const k = scaleOf(face, sh.size);
          p.addPath(cache.get(face, sh.gids[g]), new DOMMatrix([k, 0, 0, -k, sh.xs[g], sh.ys[g]]));
        }
        sh.compound = p;
      }
      ctx.setTransform(dpr, 0, 0, dpr, (sh.x + sh.pad) * dpr, (sh.y + sh.pad) * dpr);
      ctx.fillStyle = sh.color;
      ctx.fill(sh.compound);
    }
  },
  // The ADR-005 fallback: engine positions, browser rasterizes (and re-shapes) each cluster.
  cluster_filltext(ctx, scene, dpr) {
    ctx.textAlign = 'left';
    ctx.textBaseline = 'alphabetic';
    for (const sh of scene.shapes) {
      boxes(ctx, sh, dpr);
      ctx.fillStyle = sh.color;
      const ox = sh.x + sh.pad;
      const oy = sh.y + sh.pad;
      let last = -1;
      for (const c of sh.clusters) {
        if (c.face !== last) {
          ctx.font = `${sh.size}px ${st.family[st.nameByFace[c.face]]}`;
          last = c.face;
        }
        ctx.fillText(c.text, ox + c.x, oy + c.y);
      }
    }
  },
  // Reference: the browser shapes each engine line. Not an ADR-004 option; shows the floor.
  line_filltext(ctx, scene, dpr) {
    ctx.textAlign = 'left';
    ctx.textBaseline = 'alphabetic';
    for (const sh of scene.shapes) {
      boxes(ctx, sh, dpr);
      ctx.fillStyle = sh.color;
      ctx.font = `${sh.size}px ${sh.family}, SpkCarlito`;
      const ox = sh.x + sh.pad;
      const oy = sh.y + sh.pad;
      for (const l of sh.lines) ctx.fillText(l.text, ox + l.x, oy + l.y);
    }
  },
};

function canvasFor(dpr) {
  const old = document.getElementById('c');
  const c = document.createElement('canvas');
  c.id = 'c';
  c.width = 1280 * dpr;
  c.height = 720 * dpr;
  c.style.width = '1280px';
  c.style.height = '720px';
  old.replaceWith(c);
  return c;
}

function clear(ctx) {
  ctx.setTransform(1, 0, 0, 1, 0, 0);
  ctx.fillStyle = '#fff';
  ctx.fillRect(0, 0, ctx.canvas.width, ctx.canvas.height);
}

// (3) frame time, the flush-by-readback method: paint, then read one pixel. -------
function frame(ctx, paint, scene, dpr, cache) {
  const t0 = now();
  clear(ctx);
  paint(ctx, scene, dpr, cache);
  const t1 = now();
  flush(ctx);
  const t2 = now();
  return { submit: t1 - t0, total: t2 - t0 };
}

function denseFrames({ kind, mode, dpr, warmFrames = 40 }) {
  const c = canvasFor(dpr);
  const ctx = c.getContext('2d');
  clear(ctx);
  flush(ctx);
  const scene = buildScene(kind);
  const cache = new PathCache();
  const paint = painters[mode];
  const first = frame(ctx, paint, scene, dpr, cache);
  const coldStats = {
    first_frame_ms: first.total, first_submit_ms: first.submit,
    path_load_ms: cache.load_ms, path_misses: cache.misses, svg_bytes: cache.svg_bytes,
  };
  for (let i = 0; i < 5; i++) frame(ctx, paint, scene, dpr, cache);
  const totals = [];
  const submits = [];
  for (let i = 0; i < warmFrames; i++) {
    const f = frame(ctx, paint, scene, dpr, cache);
    totals.push(f.total);
    submits.push(f.submit);
  }
  return {
    kind, mode, dpr, glyphs: scene.glyphs, unique_glyphs: scene.unique_glyphs,
    scene_layout_ms: scene.layout_ms, ...coldStats,
    cached_total_ms: stats(totals), cached_submit_ms: stats(submits),
  };
}

// (3) frame time, the rAF method: no readback, so the canvas keeps whatever
// acceleration it has. Interval between animation frames while repainting each one.
async function denseRaf({ kind, mode, dpr, frames = 90 }) {
  const c = canvasFor(dpr);
  const ctx = c.getContext('2d');
  const scene = buildScene(kind);
  const cache = new PathCache();
  const paint = painters[mode];
  await raf();
  const firstStart = now();
  clear(ctx);
  paint(ctx, scene, dpr, cache);
  const firstSubmit = now() - firstStart;
  await raf();
  await raf();
  const firstPresented = now() - firstStart;
  for (let i = 0; i < 10; i++) {
    await raf();
    clear(ctx);
    paint(ctx, scene, dpr, cache);
  }
  const intervals = [];
  const submits = [];
  let last = await raf();
  for (let i = 0; i < frames; i++) {
    const t = now();
    clear(ctx);
    paint(ctx, scene, dpr, cache);
    submits.push(now() - t);
    const ts = await raf();
    intervals.push(ts - last);
    last = ts;
  }
  return {
    kind, mode, dpr, glyphs: scene.glyphs,
    first_submit_ms: firstSubmit, first_two_rafs_ms: firstPresented,
    raf_interval_ms: stats(intervals), submit_ms: stats(submits),
  };
}

// (3) frame time on the GPU. Chrome moves a canvas off the GPU once it has been read
// back, which is why the readback table above matches the software browser. So each
// sample here is a NEW canvas whose first and only readback ends the sample: everything
// painted must have finished on the GPU first. Fresh-canvas cost is reported as `empty`.
async function denseGpu({ kind, mode, dpr, n = 25 }) {
  const scene = buildScene(kind);
  const cache = new PathCache();
  const paint = painters[mode];
  const once = (fn) => {
    const c = canvasFor(dpr);
    const ctx = c.getContext('2d');
    const t = now();
    clear(ctx);
    fn(ctx);
    flush(ctx);
    return now() - t;
  };
  const empty = [];
  for (let i = 0; i < 8; i++) {
    empty.push(once(() => {}));
    await raf();
  }
  const first = once((ctx) => paint(ctx, scene, dpr, cache));
  const load = cache.load_ms;
  await raf();
  const cached = [];
  for (let i = 0; i < n; i++) {
    cached.push(once((ctx) => paint(ctx, scene, dpr, cache)));
    await raf();
  }
  return {
    kind, mode, dpr, glyphs: scene.glyphs, unique_glyphs: scene.unique_glyphs,
    first_frame_ms: first, path_load_ms: load, empty_ms: stats(empty), cached_ms: stats(cached),
    memory_bytes: st.memory,
  };
}

// Idle rAF cadence, so a capped measurement can be read as capped.
async function idleRaf() {
  const out = [];
  let last = await raf();
  for (let i = 0; i < 60; i++) {
    const ts = await raf();
    out.push(ts - last);
    last = ts;
  }
  return stats(out);
}

function renderDense({ kind, mode, dpr }) {
  const c = canvasFor(dpr);
  const ctx = c.getContext('2d');
  const scene = buildScene(kind);
  clear(ctx);
  painters[mode](ctx, scene, dpr, new PathCache());
  flush(ctx);
  return c.toDataURL('image/png');
}

// (4) appearance ------------------------------------------------------------------
const SAMPLE = 'Quarterly affinity: fiscal AVATAR waffle 2026, Typography office.';

function singleLine(text, stackNames, size) {
  const stack = Uint32Array.from(stackNames.map((s) => st.ids[s]));
  const r = st.engine.layout(text, stack, size, 2e5);
  const o = {
    gids: r.gids(), faces: r.faces(), xs: r.xs(), ys: r.ys(), lineY: r.line_y()[0], lineX: r.line_x()[0],
    w: r.line_w()[0], cl: [],
  };
  const cs = r.cl_start16();
  const ce = r.cl_end16();
  const cx = r.cl_x();
  const cy = r.cl_y();
  const adv = r.cl_adv();
  const cn = r.cl_n();
  const rtl = r.cl_rtl();
  const cf = r.cl_face();
  let g0 = 0;
  for (let i = 0; i < cs.length; i++) {
    o.cl.push({ start: cs[i], end: ce[i], text: text.slice(cs[i], ce[i]), x: cx[i], y: cy[i], adv: adv[i], g0, gn: cn[i], rtl: rtl[i], face: cf[i] });
    g0 += cn[i];
  }
  r.free();
  return o;
}

function drawPathLine(ctx, l, size, ox, oy, scale, cache) {
  ctx.save();
  for (let g = 0; g < l.gids.length; g++) {
    const face = l.faces[g];
    const k = scaleOf(face, size) * scale;
    ctx.setTransform(k, 0, 0, -k, ox + (l.xs[g] - l.lineX) * scale, oy + (l.ys[g] - l.lineY) * scale);
    ctx.fill(cache.get(face, l.gids[g]));
  }
  ctx.restore();
}

function drawClusterLine(ctx, l, size, family, ox, oy, scale, wrap = (t) => t) {
  ctx.save();
  ctx.setTransform(scale, 0, 0, scale, ox, oy);
  ctx.font = `${size}px ${family}`;
  ctx.textAlign = 'left';
  ctx.textBaseline = 'alphabetic';
  for (const c of l.cl) ctx.fillText(wrap(c.text, c), c.x - l.lineX, c.y - l.lineY);
  ctx.restore();
}

function drawWholeLine(ctx, text, size, family, ox, oy, scale, rtlWidth) {
  ctx.save();
  ctx.setTransform(scale, 0, 0, scale, ox, oy);
  ctx.font = `${size}px ${family}`;
  ctx.textAlign = 'left';
  ctx.textBaseline = 'alphabetic';
  if (rtlWidth !== undefined) ctx.direction = 'rtl';
  ctx.fillText(text, 0, 0);
  ctx.restore();
}

function regionMetrics(ctx, x, y, w, h, bgLum, glyphs) {
  const d = ctx.getImageData(x, y, w, h).data;
  let ink = 0;
  let extreme = 0;
  let chroma = 0;
  let touched = 0;
  const lum = new Float32Array(w * h);
  for (let i = 0; i < w * h; i++) {
    const [r, g, b] = [d[i * 4], d[i * 4 + 1], d[i * 4 + 2]];
    const l = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    lum[i] = l;
    const delta = Math.abs(l - bgLum) / 255;
    ink += delta;
    if (delta > extreme) extreme = delta;
    if (delta > 0.02) touched++;
    if (Math.max(r, g, b) - Math.min(r, g, b) > 12) chroma++;
  }
  return { ink_per_glyph: ink / glyphs, peak_contrast: extreme, touched_px_per_glyph: touched / glyphs, chroma_px: chroma, lum };
}

function appearance({ opaque }) {
  const rows = [[10, 1], [12, 1], [18, 1], [10, 2], [12, 2], [18, 2], [10, 0.5]];
  const fonts = [['Carlito', ['carlito'], 'SpkCarlito'], ['Liberation Sans', ['liberation'], 'SpkLiberation']];
  const COLW = 1500;
  const LABEL = 30;
  let H = 40;
  const rowH = (pt, s) => Math.ceil(pt * (96 / 72) * s * 1.9) + 18;
  for (const _ of fonts) for (const [pt, s] of rows) H += rowH(pt, s);
  H = H * 2 + 40;
  const c = document.getElementById('c');
  const canvas = document.createElement('canvas');
  canvas.id = 'c';
  canvas.width = COLW * 3 + LABEL;
  canvas.height = H;
  c.replaceWith(canvas);
  const ctx = canvas.getContext('2d', { alpha: !opaque });
  ctx.fillStyle = '#fff';
  ctx.fillRect(0, 0, canvas.width, H / 2);
  ctx.fillStyle = '#111111';
  ctx.fillRect(0, H / 2, canvas.width, H / 2);
  const cache = new PathCache();
  const metrics = [];
  const label = (t, x, y, col) => {
    ctx.save();
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.font = '11px Helvetica, Arial, sans-serif';
    ctx.fillStyle = col;
    ctx.fillText(t, x, y);
    ctx.restore();
  };
  const panel = (y0, bg, fg, bgLum, labelCol) => {
    label('Path2D (unhinted skrifa outlines at engine x)', LABEL, y0 + 14, labelCol);
    label('fillText per cluster at engine x', LABEL + COLW, y0 + 14, labelCol);
    label('fillText whole line (browser shapes)', LABEL + 2 * COLW, y0 + 14, labelCol);
    let y = y0 + 22;
    for (const [fname, stack, family] of fonts) {
      for (const [pt, s] of rows) {
        const size = pt * 96 / 72;
        const line = singleLine(SAMPLE, stack, size);
        const h = rowH(pt, s);
        const base = y + 12 + Math.ceil(size * s * 1.15);
        label(`${fname} ${pt}pt ${s}x (${(size * s).toFixed(1)} device px)`, LABEL, y + 10, '#9ca3af');
        ctx.fillStyle = fg;
        drawPathLine(ctx, line, size, LABEL, base, s, cache);
        ctx.fillStyle = fg;
        drawClusterLine(ctx, line, size, family, LABEL + COLW, base, s);
        ctx.fillStyle = fg;
        drawWholeLine(ctx, SAMPLE, size, family, LABEL + 2 * COLW, base, s);
        // Metrics over the glyph band only, excluding the row's label.
        const band = [Math.floor(base - size * s * 1.05), Math.ceil(size * s * 1.4)];
        const w = Math.min(COLW - 4, Math.ceil(line.w * s) + 4);
        const glyphs = line.gids.length;
        const a = regionMetrics(ctx, LABEL, band[0], w, band[1], bgLum, glyphs);
        const b = regionMetrics(ctx, LABEL + COLW, band[0], w, band[1], bgLum, glyphs);
        const l = regionMetrics(ctx, LABEL + 2 * COLW, band[0], w, band[1], bgLum, glyphs);
        let diffAB = 0;
        let diffAL = 0;
        for (let i = 0; i < a.lum.length; i++) {
          diffAB += Math.abs(a.lum[i] - b.lum[i]);
          diffAL += Math.abs(a.lum[i] - l.lum[i]);
        }
        const strip = (m) => ({
          ink_per_glyph: m.ink_per_glyph, peak_contrast: m.peak_contrast,
          touched_px_per_glyph: m.touched_px_per_glyph, chroma_px: m.chroma_px,
        });
        metrics.push({
          panel: bg, font: fname, pt, scale: s, device_px: size * s, engine_w: line.w,
          path: strip(a), cluster: strip(b), line: strip(l),
          ink_ratio_cluster_over_path: b.ink_per_glyph / a.ink_per_glyph,
          ink_ratio_line_over_path: l.ink_per_glyph / a.ink_per_glyph,
          mean_abs_lum_diff_cluster: diffAB / a.lum.length,
          mean_abs_lum_diff_line: diffAL / a.lum.length,
        });
        y += h;
      }
    }
  };
  panel(0, 'light', '#000000', 255, '#000000');
  panel(H / 2, 'dark', '#ffffff', 17, '#ffffff');
  return { png: canvas.toDataURL('image/png'), metrics };
}

/** Nearest-neighbour magnification of the small rows, for looking at pixels. */
function magnified() {
  const mag = 4;
  const wCol = 300;
  const fonts = [['Carlito', ['carlito'], 'SpkCarlito'], ['Liberation Sans', ['liberation'], 'SpkLiberation']];
  const rows = [[10, 1], [12, 1], [10, 2], [10, 0.5]];
  const off = document.createElement('canvas');
  const rowPx = 48;
  off.width = wCol * 2 + 20;
  off.height = rowPx * rows.length * fonts.length * 2;
  const octx = off.getContext('2d');
  octx.fillStyle = '#fff';
  octx.fillRect(0, 0, off.width, off.height);
  const cache = new PathCache();
  let y = 0;
  for (const [, stack, family] of fonts) {
    for (const [pt, s] of rows) {
      const size = pt * 96 / 72;
      const line = singleLine(SAMPLE, stack, size);
      octx.fillStyle = '#111827';
      drawPathLine(octx, line, size, 4, y + 24, s, cache);
      y += rowPx;
      octx.fillStyle = '#111827';
      drawClusterLine(octx, line, size, family, 4, y + 24, s);
      y += rowPx;
    }
  }
  const out = document.createElement('canvas');
  out.width = 1500;
  out.height = off.height * mag;
  const c2 = out.getContext('2d');
  c2.imageSmoothingEnabled = false;
  c2.drawImage(off, 0, 0, 375, off.height, 0, 0, 1500, off.height * mag);
  return out.toDataURL('image/png');
}

// Does fillText honour fractional x? Centroid of one glyph drawn at sub-pixel offsets.
function subpixelProbe() {
  const size = 13.333;
  const results = [];
  for (const [name, stack, family] of [['Carlito', ['carlito'], 'SpkCarlito'], ['Liberation Sans', ['liberation'], 'SpkLiberation']]) {
    const line = singleLine('l', stack, size);
    for (const method of ['path', 'fillText', 'fillText geometricPrecision']) {
      const centroids = [];
      for (const frac of [0, 0.125, 0.25, 0.375, 0.5, 0.625, 0.75, 0.875]) {
        const cv = document.createElement('canvas');
        cv.width = 32;
        cv.height = 32;
        const ctx = cv.getContext('2d', { willReadFrequently: true });
        ctx.fillStyle = '#fff';
        ctx.fillRect(0, 0, 32, 32);
        ctx.fillStyle = '#000';
        if (method === 'path') {
          drawPathLine(ctx, line, size, 10 + frac, 24, 1, new PathCache());
        } else {
          if (method.endsWith('Precision')) ctx.textRendering = 'geometricPrecision';
          ctx.font = `${size}px ${family}`;
          ctx.fillText('l', 10 + frac, 24);
        }
        const d = ctx.getImageData(0, 0, 32, 32).data;
        let m = 0;
        let sum = 0;
        for (let y = 0; y < 32; y++) {
          for (let x = 0; x < 32; x++) {
            const a = 255 - d[(y * 32 + x) * 4 + 1];
            m += a;
            sum += a * (x + 0.5);
          }
        }
        centroids.push(sum / m);
      }
      const shifts = centroids.map((c) => c - centroids[0]);
      results.push({ font: name, method, axis: 'x', shifts });
      const ys = [];
      for (const frac of [0, 0.25, 0.5, 0.75]) {
        const cv = document.createElement('canvas');
        cv.width = 32;
        cv.height = 32;
        const ctx = cv.getContext('2d', { willReadFrequently: true });
        ctx.fillStyle = '#fff';
        ctx.fillRect(0, 0, 32, 32);
        ctx.fillStyle = '#000';
        if (method === 'path') {
          drawPathLine(ctx, line, size, 10, 24 + frac, 1, new PathCache());
        } else {
          if (method.endsWith('Precision')) ctx.textRendering = 'geometricPrecision';
          ctx.font = `${size}px ${family}`;
          ctx.fillText('l', 10, 24 + frac);
        }
        const d = ctx.getImageData(0, 0, 32, 32).data;
        let m = 0;
        let sum = 0;
        for (let y = 0; y < 32; y++) {
          for (let x = 0; x < 32; x++) {
            const a = 255 - d[(y * 32 + x) * 4 + 1];
            m += a;
            sum += a * (y + 0.5);
          }
        }
        ys.push(sum / m);
      }
      results.push({ font: name, method, axis: 'y', shifts: ys.map((c) => c - ys[0]) });
    }
  }
  return results;
}

// (5) agreement between the engine and fillText --------------------------------------
// Same-form test, tolerant of raster weight. macOS text is dilated, so plain IoU punishes
// the correct glyph. At 96px, the share of each mask within 3px of the other: a wrong
// form leaves whole strokes uncovered, a heavier rendering of the right form does not.
function dilate(mask, W, H, r) {
  const out = new Uint8Array(W * H);
  for (let y = 0; y < H; y++) {
    for (let x = 0; x < W; x++) {
      if (!mask[y * W + x]) continue;
      for (let dy = -r; dy <= r; dy++) {
        for (let dx = -r; dx <= r; dx++) {
          const yy = y + dy;
          const xx = x + dx;
          if (yy >= 0 && yy < H && xx >= 0 && xx < W && dx * dx + dy * dy <= r * r) out[yy * W + xx] = 1;
        }
      }
    }
  }
  return out;
}

function maskOf(ctx, W, H) {
  const d = ctx.getImageData(0, 0, W, H).data;
  const m = new Uint8Array(W * H);
  let n = 0;
  for (let i = 0; i < W * H; i++) {
    if (d[i * 4] < 128) {
      m[i] = 1;
      n++;
    }
  }
  return { m, n };
}

function coverage(a, bDil) {
  let hit = 0;
  for (let i = 0; i < a.m.length; i++) if (a.m[i] && bDil[i]) hit++;
  return a.n === 0 ? 1 : hit / a.n;
}

const CV_W = 520;
const CV_H = 300;

function pathMask(c, l, cvP, cache, size) {
  const px = cvP.getContext('2d', { willReadFrequently: true });
  px.setTransform(1, 0, 0, 1, 0, 0);
  px.fillStyle = '#fff';
  px.fillRect(0, 0, CV_W, CV_H);
  px.fillStyle = '#000';
  for (let g = c.g0; g < c.g0 + c.gn; g++) {
    const face = l.faces[g];
    const k = scaleOf(face, size);
    px.setTransform(k, 0, 0, -k, 80 + l.xs[g] - c.x, 200 + l.ys[g] - c.y);
    px.fill(cache.get(face, l.gids[g]));
  }
  return maskOf(px, CV_W, CV_H);
}

function textMask(str, family, cvT, size) {
  const tx = cvT.getContext('2d', { willReadFrequently: true });
  tx.setTransform(1, 0, 0, 1, 0, 0);
  tx.fillStyle = '#fff';
  tx.fillRect(0, 0, CV_W, CV_H);
  tx.fillStyle = '#000';
  tx.font = `${size}px ${family}`;
  tx.textAlign = 'left';
  tx.textBaseline = 'alphabetic';
  tx.fillText(str, 80, 200);
  return maskOf(tx, CV_W, CV_H);
}

function sameForm(p, pDil, t) {
  const tDil = dilate(t.m, CV_W, CV_H, 3);
  return Math.min(coverage(p, tDil), coverage(t, pDil));
}

// Smallest prefix where browser and engine advance disagree, to name the cause.
function firstDivergence(ctx, text, widthOf) {
  const chars = [...text];
  let lo = 1;
  let hi = chars.length;
  const differs = (n) => {
    const t = chars.slice(0, n).join('');
    return Math.abs(ctx.measureText(t).width - widthOf(t)) > 0.02;
  };
  if (!differs(hi)) return null;
  while (lo < hi) {
    const mid = (lo + hi) >> 1;
    if (differs(mid)) hi = mid;
    else lo = mid + 1;
  }
  return { at_char: lo, context: chars.slice(Math.max(0, lo - 6), lo + 2).join('') };
}

function wordsOf(l, text) {
  // Whitespace-separated words in logical order, from the engine's clusters.
  const cl = [...l.cl].sort((a, b) => a.start - b.start);
  const out = [];
  let cur = null;
  for (const c of cl) {
    if (c.text.trim() === '') {
      if (cur) out.push(cur);
      cur = null;
      continue;
    }
    if (!cur) cur = { start: c.start, end: c.end, adv: 0 };
    cur.end = c.end;
    cur.adv += c.adv;
  }
  if (cur) out.push(cur);
  return out.map((w) => ({ text: text.slice(w.start, w.end), adv: w.adv }));
}

function agreement() {
  const out = [];
  const mk = () => {
    const c = document.createElement('canvas');
    c.width = CV_W;
    c.height = CV_H;
    return c;
  };
  const cvP = mk();
  const cvT = mk();
  const measure = document.createElement('canvas').getContext('2d');
  const cases = [
    ['Latin (Carlito)', 'latin', 1, ['carlito'], 'SpkCarlito'],
    ['Latin (Liberation Sans)', 'latin', 1, ['liberation'], 'SpkLiberation'],
    ['Arabic (Noto Sans Arabic)', 'arabic', 3, ['arabic', 'carlito'], 'SpkArabic, SpkCarlito'],
    ['CJK (Noto Sans SC)', 'cjk', 5, ['cjk', 'carlito'], 'SpkCJK, SpkCarlito'],
  ];
  const size = 16;
  for (const [name, kind, seed, stack, family] of cases) {
    const text = st.wasm.corpus(kind, 2000, seed);
    const line = singleLine(text, stack, size);
    const setFont = (kern, rendering) => {
      measure.fontKerning = kern;
      measure.textRendering = rendering;
      measure.font = `${size}px ${family}`;
    };
    setFont('auto', 'auto');
    const engineWidth = (t) => singleLine(t, stack, size).w;
    const whole = {};
    for (const [k, r] of [['auto', 'auto'], ['normal', 'auto'], ['auto', 'optimizeLegibility'], ['auto', 'geometricPrecision']]) {
      setFont(k, r);
      whole[`kerning=${k} textRendering=${r}`] = measure.measureText(text).width - line.w;
    }
    setFont('auto', 'auto');
    const divergence = firstDivergence(measure, text, engineWidth);
    let advBad = 0;
    let advSum = 0;
    let advMax = 0;
    for (const c of line.cl) {
      const d = Math.abs(measure.measureText(c.text).width - c.adv);
      advSum += d;
      advMax = Math.max(advMax, d);
      if (d > 0.01) advBad++;
    }
    let wordBad = 0;
    let wordMax = 0;
    const words = wordsOf(line, text);
    for (const w of words) {
      const d = Math.abs(measure.measureText(w.text).width - w.adv);
      wordMax = Math.max(wordMax, d);
      if (d > 0.05) wordBad++;
    }
    // Per-cluster form, per distinct (text, glyph ids).
    const big = singleLine(text, stack, 96);
    const seen = new Map();
    const cache = new PathCache();
    for (const c of big.cl) {
      const key = c.text + '|' + Array.from(big.gids.subarray(c.g0, c.g0 + c.gn)).join(',');
      const e = seen.get(key);
      if (e) e.count++;
      else seen.set(key, { c, count: 1 });
    }
    const Z = '‍';
    const wraps = kind === 'arabic'
      ? { plain: [(t) => t], zwj_both: [(t) => Z + t + Z], best_of_4_zwj: [(t) => t, (t) => Z + t, (t) => t + Z, (t) => Z + t + Z] }
      : { plain: [(t) => t] };
    const results = {};
    for (const [w, fns] of Object.entries(wraps)) {
      let bad = 0;
      let badOcc = 0;
      let occ = 0;
      let distinct = 0;
      const vals = [];
      const examples = [];
      for (const { c, count } of seen.values()) {
        if (c.text.trim() === '') continue;
        const p = pathMask(c, big, cvP, cache, 96);
        const pDil = dilate(p.m, CV_W, CV_H, 3);
        let best = 0;
        for (const fn of fns) best = Math.max(best, sameForm(p, pDil, textMask(fn(c.text), family, cvT, 96)));
        vals.push(best);
        distinct++;
        occ += count;
        if (best < 0.97) {
          bad++;
          badOcc += count;
          if (examples.length < 12) examples.push(c.text);
        }
      }
      results[w] = {
        distinct, distinct_wrong_form: bad, occurrences: occ, occurrences_wrong_form: badOcc,
        min_coverage: stats(vals), examples,
      };
    }
    out.push({
      name, clusters: line.cl.length, engine_width: line.w, whole_text_width_diff_px: whole,
      first_divergence: divergence,
      cluster_adv_differs_gt_0_01px: advBad, cluster_adv_mean_abs: advSum / line.cl.length, cluster_adv_max: advMax,
      words: words.length, word_adv_differs_gt_0_05px: wordBad, word_adv_max: wordMax,
      shape: results,
    });
  }
  return out;
}

// Picture of the Arabic disagreement.
function fallbackPicture() {
  const arab = 'السلام عليكم، هذا التقرير يوضح النمو المستمر للأرباح والمبيعات لا الله كِتَابٌ';
  const latin = 'Office affinity AVATAR Typography fjord waffle';
  const cjk = '演示文稿的第一季度财务报告，「重要」收入增长。';
  const canvas = document.createElement('canvas');
  canvas.width = 1400;
  canvas.height = 690;
  document.getElementById('c').replaceWith(canvas);
  canvas.id = 'c';
  const ctx = canvas.getContext('2d');
  ctx.fillStyle = '#fff';
  ctx.fillRect(0, 0, 1400, 690);
  const cache = new PathCache();
  const size = 32;
  const lab = (t, y) => {
    ctx.save();
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.fillStyle = '#6b7280';
    ctx.font = '12px Helvetica, Arial, sans-serif';
    ctx.textAlign = 'left';
    ctx.direction = 'ltr';
    ctx.fillText(t, 10, y);
    ctx.restore();
  };
  let y = 20;
  const row = (t, draw) => {
    lab(t, y);
    ctx.fillStyle = '#000';
    draw(y + 42);
    y += 66;
  };
  const al = singleLine(arab, ['arabic', 'carlito'], size);
  row('Arabic: Path2D at engine positions (correct, joined)', (b) => drawPathLine(ctx, al, size, 10, b, 1, cache));
  row('Arabic: fillText per cluster at engine positions (letters isolated)', (b) => drawClusterLine(ctx, al, size, 'SpkArabic, SpkCarlito', 10, b, 1));
  row('Arabic: fillText per cluster with ZWJ on both sides ', (b) => drawClusterLine(ctx, al, size, 'SpkArabic, SpkCarlito', 10, b, 1, (t) => '‍' + t + '‍'));
  row('Arabic: fillText per word at engine word positions', (b) => {
    ctx.save();
    ctx.setTransform(1, 0, 0, 1, 10, b);
    ctx.font = `${size}px SpkArabic, SpkCarlito`;
    ctx.direction = 'rtl';
    ctx.textAlign = 'left';
    const visual = al.cl;
    let i = 0;
    while (i < visual.length) {
      if (visual[i].text.trim() === '') { i++; continue; }
      let j = i;
      while (j + 1 < visual.length && visual[j + 1].text.trim() !== '') j++;
      // Visual order: cluster i is the word's left edge.
      const start = Math.min(...visual.slice(i, j + 1).map((c) => c.start));
      const end = Math.max(...visual.slice(i, j + 1).map((c) => c.end));
      ctx.fillText(arab.slice(start, end), visual[i].x - al.lineX, 0);
      i = j + 1;
    }
    ctx.restore();
  });
  row('Arabic: fillText whole line (browser shapes; correct but not engine-placed)', (b) => drawWholeLine(ctx, arab, size, 'SpkArabic, SpkCarlito', 10, b, 1, al.w));
  const ll = singleLine(latin, ['carlito'], size);
  row('Latin: Path2D at engine positions', (b) => drawPathLine(ctx, ll, size, 10, b, 1, cache));
  row('Latin: fillText per cluster at engine positions', (b) => drawClusterLine(ctx, ll, size, 'SpkCarlito', 10, b, 1));
  const cl = singleLine(cjk, ['cjk', 'carlito'], size);
  row('CJK: Path2D at engine positions', (b) => drawPathLine(ctx, cl, size, 10, b, 1, cache));
  row('CJK: fillText per cluster at engine positions', (b) => drawClusterLine(ctx, cl, size, 'SpkCJK, SpkCarlito', 10, b, 1));
  return canvas.toDataURL('image/png');
}

/** 6x nearest-neighbour: path above fillText, for one size. */
function pixelCrop({ family, stack, pt, s }) {
  const size = pt * 96 / 72;
  const text = 'Quarterly affinity: fiscal AVATAR';
  const line = singleLine(text, stack, size);
  const off = document.createElement('canvas');
  off.width = Math.ceil(line.w * s) + 8;
  off.height = Math.ceil(size * s * 1.5) * 2 + 8;
  const o = off.getContext('2d');
  o.fillStyle = '#fff';
  o.fillRect(0, 0, off.width, off.height);
  o.fillStyle = '#000';
  const h = Math.ceil(size * s * 1.5);
  drawPathLine(o, line, size, 4, Math.ceil(size * s * 1.1), s, new PathCache());
  o.fillStyle = '#000';
  drawClusterLine(o, line, size, family, 4, h + Math.ceil(size * s * 1.1), s);
  const mag = Math.max(2, Math.min(6, Math.floor(1400 / off.width)));
  const out = document.createElement('canvas');
  out.width = off.width * mag;
  out.height = off.height * mag;
  const c2 = out.getContext('2d');
  c2.imageSmoothingEnabled = false;
  c2.drawImage(off, 0, 0, out.width, out.height);
  return out.toDataURL('image/png');
}

/** The reviewable picture: Path2D left, fillText per cluster right, device pixels 1:1. */
function sideBySide() {
  const text = 'Quarterly affinity: fiscal AVATAR 2026';
  const rows = [[10, 1], [12, 1], [18, 1], [10, 2], [12, 2], [18, 2], [10, 0.5]];
  const fonts = [['Carlito', ['carlito'], 'SpkCarlito'], ['Liberation Sans', ['liberation'], 'SpkLiberation']];
  const COL = 840;
  const rowH = (pt, s) => Math.ceil((pt * 96 / 72) * s * 1.6) + 16;
  let panelH = 26;
  for (const _ of fonts) for (const [pt, s] of rows) panelH += rowH(pt, s);
  const canvas = document.createElement('canvas');
  canvas.width = COL * 2;
  canvas.height = panelH * 2;
  document.getElementById('c').replaceWith(canvas);
  canvas.id = 'c';
  const ctx = canvas.getContext('2d', { alpha: false });
  const cache = new PathCache();
  const panel = (y0, bg, fg) => {
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.fillStyle = bg;
    ctx.fillRect(0, y0, COL * 2, panelH);
    const lab = (t, x, y) => {
      ctx.save();
      ctx.setTransform(1, 0, 0, 1, 0, 0);
      ctx.font = '11px Helvetica, Arial, sans-serif';
      ctx.fillStyle = '#888888';
      ctx.fillText(t, x, y);
      ctx.restore();
    };
    lab('Path2D at engine positions', 8, y0 + 14);
    lab('fillText per cluster at engine positions', COL + 8, y0 + 14);
    let y = y0 + 22;
    for (const [fname, stack, family] of fonts) {
      for (const [pt, s] of rows) {
        const size = pt * 96 / 72;
        const line = singleLine(text, stack, size);
        lab(`${fname} ${pt}pt ${s}x (${(size * s).toFixed(1)} device px)`, 8, y + 10);
        lab(`${fname} ${pt}pt ${s}x`, COL + 8, y + 10);
        const base = y + 12 + Math.ceil(size * s * 1.05);
        ctx.fillStyle = fg;
        drawPathLine(ctx, line, size, 8, base, s, cache);
        ctx.fillStyle = fg;
        drawClusterLine(ctx, line, size, family, COL + 8, base, s);
        y += rowH(pt, s);
      }
    }
  };
  panel(0, '#ffffff', '#000000');
  panel(panelH, '#111111', '#ffffff');
  return canvas.toDataURL('image/png');
}

window.spk = {
  sideBySide,
  pixelCrop,
  init, cold, warm, denseFrames, denseRaf, denseGpu, idleRaf, renderDense, appearance, magnified,
  subpixelProbe, agreement, fallbackPicture, CASES,
};
window.spkReady = true;
