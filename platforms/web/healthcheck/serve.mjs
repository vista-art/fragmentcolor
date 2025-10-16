#!/usr/bin/env node
import http from 'node:http';
import fs from 'node:fs/promises';
import fssync from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Serve the platforms/web directory as the root
const WEB_ROOT = path.resolve(path.join(__dirname, '..'));
const PORT = Number(process.env.PORT || 8765);

function contentType(filePath) {
  const ext = path.extname(filePath).toLowerCase();
  switch (ext) {
    case '.html': return 'text/html; charset=utf-8';
    case '.js':
    case '.mjs': return 'text/javascript; charset=utf-8';
    case '.css': return 'text/css; charset=utf-8';
    case '.json': return 'application/json; charset=utf-8';
    case '.wasm': return 'application/wasm';
    case '.png': return 'image/png';
    case '.jpg':
    case '.jpeg': return 'image/jpeg';
    case '.svg': return 'image/svg+xml; charset=utf-8';
    default: return 'application/octet-stream';
  }
}

function withHeaders(res) {
  // Enable cross-origin isolation for SharedArrayBuffer and WebGPU readbacks
  res.setHeader('Cross-Origin-Opener-Policy', 'same-origin');
  res.setHeader('Cross-Origin-Embedder-Policy', 'require-corp');
  res.setHeader('Cache-Control', 'no-store, max-age=0, must-revalidate');
  res.setHeader('Pragma', 'no-cache');
  res.setHeader('Expires', '0');
}

async function serveFile(req, res, filePath) {
  try {
    const stat = await fs.stat(filePath);
    if (stat.isDirectory()) {
      // Directory: serve index.html if present
      const indexPath = path.join(filePath, 'index.html');
      if (fssync.existsSync(indexPath)) {
        return serveFile(req, res, indexPath);
      }
      res.statusCode = 404;
      res.end('Not Found');
      return;
    }

    const data = await fs.readFile(filePath);
    withHeaders(res);
    res.setHeader('Content-Type', contentType(filePath));
    res.statusCode = 200;
    res.end(data);
  } catch (err) {
    res.statusCode = 404;
    res.end('Not Found');
  }
}

const server = http.createServer(async (req, res) => {
  try {
    const url = new URL(req.url, `http://localhost:${PORT}`);
    let rel = decodeURIComponent(url.pathname);
    if (rel === '/') rel = '/index.html';

    // Special route: runtime examples list for gallery
    if (rel === '/healthcheck/examples.json' || rel === '/gallery/examples.json') {
      try {
        const base = path.join(WEB_ROOT, 'examples');
        async function walk(dir) {
          const items = await fs.readdir(dir, { withFileTypes: true });
          let out = [];
          for (const ent of items) {
            const full = path.join(dir, ent.name);
            if (ent.isDirectory()) {
              const sub = await walk(full);
              out = out.concat(sub);
            } else if (ent.isFile() && full.toLowerCase().endsWith('.js')) {
              const relFromBase = path.relative(base, full).split(path.sep).join('/');
              out.push('../examples/' + relFromBase);
            }
          }
          return out;
        }
        const list = fssync.existsSync(base) ? await walk(base) : [];
        withHeaders(res);
        res.setHeader('Content-Type', 'application/json; charset=utf-8');
        res.statusCode = 200;
        res.end(JSON.stringify(list));
      } catch (err) {
        res.statusCode = 500;
        res.end('[]');
      }
      return;
    }

    // Map /gallery to the visual gallery page
    if (rel === '/gallery' || rel === '/gallery/') rel = '/healthcheck/gallery.html';
    else if (rel.startsWith('/gallery/')) rel = '/healthcheck' + rel.slice('/gallery'.length);

    // Security: prevent path traversal
    const safePath = path.normalize(rel).replace(/^\/+/, '/');
    const abs = path.join(WEB_ROOT, safePath);

    // Ensure abs is within WEB_ROOT
    const absResolved = path.resolve(abs);
    if (!absResolved.startsWith(WEB_ROOT)) {
      res.statusCode = 403;
      res.end('Forbidden');
      return;
    }

    await serveFile(req, res, absResolved);
  } catch (e) {
    res.statusCode = 500;
    res.end('Internal Server Error');
  }
});

server.listen(PORT, () => {
  const addr = server.address();
  const port = (addr && typeof addr === 'object') ? addr.port : PORT;
  console.log(`[server] serving ${WEB_ROOT} at http://localhost:${port}/`);
  console.log(`[server] open http://localhost:${port}/gallery/`);
  console.log(`[server] visual http://localhost:${port}/gallery/visual.html`);
});

function shutdown() {
  server.close(() => process.exit(0));
}
process.on('SIGINT', shutdown);
process.on('SIGTERM', shutdown);

