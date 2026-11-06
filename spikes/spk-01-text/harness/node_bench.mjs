// Same throughput bench as the browser page, in Node, as a cross-check on V8 alone.
import { readFile, writeFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { CASES, FONTS, coldLayout, registerFaces, warmThroughput } from './bench_core.js';

const HERE = new URL('.', import.meta.url);
const fontBytes = {};
for (const [id, file] of FONTS) fontBytes[id] = new Uint8Array(await readFile(new URL(`../fonts/${file}`, HERE)));

const out = { node: process.version, warm: {}, cold: {} };
for (const tag of ['o3', 'small']) {
  const wasm = await import(new URL(`pkg/${tag}/spk_text_wasm.js`, HERE).href);
  const bytes = await readFile(new URL(`pkg/${tag}/spk_text_wasm_bg.wasm`, HERE));
  await wasm.default({ module_or_path: bytes });
  const engine = new wasm.Engine();
  const { ids } = registerFaces(engine, fontBytes);
  // The first layout per case in one engine; later cases share no plan, except Latin.
  out.cold[tag] = CASES.map((c) => ({ name: c.name, ms: coldLayout(wasm, engine, ids, c) }));
  out.warm[tag] = warmThroughput(wasm, engine, ids);
}
await writeFile(fileURLToPath(new URL('../results/node-throughput.json', HERE)), JSON.stringify(out, null, 1));
console.log(JSON.stringify(out.warm.o3.map((r) => [r.name, r.full_ms.median]), null, 0));
