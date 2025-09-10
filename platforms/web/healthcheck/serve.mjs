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
  // Static assets from same origin will satisfy COEP.
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
  console.log(`[server] serving ${WEB_ROOT} at http://localhost:${PORT}/`);
  console.log(`[server] try http://localhost:${PORT}/healthcheck/visual.html`);
});

function shutdown() {
  server.close(() => process.exit(0));
}
process.on('SIGINT', shutdown);
process.on('SIGTERM', shutdown);

