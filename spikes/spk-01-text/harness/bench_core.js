// Shared by the browser page and the Node run: shaping and line-breaking throughput.
// Timer is performance.now(); the page is served cross-origin isolated, so its
// resolution in Chrome is 5 microseconds rather than 100.

export const FONTS = [
  ['carlito', 'Carlito-Regular.ttf', 'SpkCarlito'],
  ['liberation', 'LiberationSans-Regular.ttf', 'SpkLiberation'],
  ['arabic', 'NotoSansArabic.ttf', 'SpkArabic'],
  ['cjk', 'NotoSansSC.otf', 'SpkCJK'],
];

export const CASES = [
  { name: 'Latin (Carlito)', kind: 'latin', seed: 1, stack: ['carlito'] },
  { name: 'Latin (Liberation Sans)', kind: 'latin', seed: 1, stack: ['liberation'] },
  { name: 'Arabic (Noto Sans Arabic)', kind: 'arabic', seed: 3, stack: ['arabic', 'carlito'] },
  { name: 'CJK (Noto Sans SC, CFF)', kind: 'cjk', seed: 5, stack: ['cjk', 'carlito'] },
  { name: 'Mixed bidi + fallback', kind: 'mixed', seed: 7, stack: ['carlito', 'arabic', 'cjk'] },
];

export const SIZE = 16;
export const WIDTH = 600;

export function stats(a) {
  const s = [...a].sort((x, y) => x - y);
  const q = (p) => s[Math.min(s.length - 1, Math.floor(p * s.length))];
  return { n: s.length, median: q(0.5), p95: q(0.95), min: s[0], max: s[s.length - 1] };
}

function batches(fn, iters, n) {
  const out = [];
  for (let i = 0; i < n; i++) {
    const t = performance.now();
    fn(iters);
    out.push((performance.now() - t) / iters);
  }
  return out;
}

/** Register all faces; returns name -> index and per-face parse time. */
export function registerFaces(engine, bytes) {
  const ids = {};
  const ms = {};
  for (const [id] of FONTS) {
    const t = performance.now();
    const face = engine.register_font(bytes[id]);
    ms[id] = performance.now() - t;
    if (face < 0) throw new Error('unreadable face ' + id);
    ids[id] = face;
  }
  return { ids, ms };
}

/** The first layout of one case in this engine: fonts parsed, no shape plan yet. */
export function coldLayout(wasm, engine, ids, c) {
  const text = wasm.corpus(c.kind, 2000, c.seed);
  const stack = Uint32Array.from(c.stack.map((s) => ids[s]));
  const t = performance.now();
  const r = engine.layout(text, stack, SIZE, WIDTH);
  const ms = performance.now() - t;
  r.free();
  return ms;
}

export function warmThroughput(wasm, engine, ids) {
  const rows = [];
  for (const c of CASES) {
    const text = wasm.corpus(c.kind, 2000, c.seed);
    const stack = Uint32Array.from(c.stack.map((s) => ids[s]));
    engine.bench_layout(text, stack, SIZE, WIDTH, 20); // warm-up, discarded
    const n = 30;
    const full = stats(batches((it) => engine.bench_layout(text, stack, SIZE, WIDTH, it), 20, n));
    const itemize = stats(batches((it) => engine.bench_itemize(text, stack, it), 20, n));
    const shape = stats(batches((it) => engine.bench_shape(text, stack, SIZE, it), 20, n));
    const breaks = stats(batches((it) => wasm.bench_breaks(text, it), 20, n));
    // Layout plus copying every array a painter needs across the boundary.
    const cross = [];
    let glyphs = 0;
    let lines = 0;
    for (let i = 0; i < 200; i++) {
      const t = performance.now();
      const r = engine.layout(text, stack, SIZE, WIDTH);
      const g = r.gids();
      r.faces(); r.xs(); r.ys();
      r.line_y();
      cross.push(performance.now() - t);
      glyphs = g.length;
      lines = r.line_y().length;
      r.free();
    }
    rows.push({
      name: c.name,
      chars: [...text].length,
      utf8_bytes: new TextEncoder().encode(text).length,
      glyphs,
      lines,
      full_ms: full,
      itemize_ms: itemize,
      shape_incl_itemize_ms: shape,
      breaks_ms: breaks,
      full_plus_js_copy_ms: stats(cross),
      chars_per_ms: [...text].length / full.median,
    });
  }
  return rows;
}
