// Static server for the harness. Cross-origin isolated, so performance.now() in the
// page has 5 microsecond resolution; loopback only.
import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import { extname, join, normalize } from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = fileURLToPath(new URL('.', import.meta.url));
const TYPES = {
  '.html': 'text/html', '.js': 'text/javascript', '.mjs': 'text/javascript',
  '.wasm': 'application/wasm', '.ttf': 'font/ttf', '.otf': 'font/otf',
};

export function serve(port = 0) {
  const srv = createServer(async (req, res) => {
    const path = normalize(decodeURIComponent(new URL(req.url, 'http://x').pathname));
    const file = path.startsWith('/fonts/')
      ? join(HERE, '..', path)
      : join(HERE, path === '/' ? 'page.html' : path);
    try {
      const body = await readFile(file);
      res.writeHead(200, {
        'content-type': TYPES[extname(file)] ?? 'application/octet-stream',
        'cross-origin-opener-policy': 'same-origin',
        'cross-origin-embedder-policy': 'require-corp',
      });
      res.end(body);
    } catch {
      res.writeHead(404);
      res.end();
    }
  });
  return new Promise((ok) => srv.listen(port, '127.0.0.1', () => ok(srv)));
}
