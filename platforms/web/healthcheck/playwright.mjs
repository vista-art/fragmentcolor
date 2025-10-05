import { chromium } from 'playwright';
import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const WEB_ROOT = path.resolve(path.join(__dirname, '..'));

const url = process.argv[2] || 'http://localhost:8765/healthcheck/';
const ARTIFACT_DIR = process.env.ARTIFACT_DIR || path.join(process.cwd(), 'platforms/web/healthcheck/playwright-artifacts');

(async () => {
  await fs.mkdir(ARTIFACT_DIR, { recursive: true }).catch(() => {});

  const isMac = process.platform === 'darwin';
  const isLinux = process.platform === 'linux';

  const common = ['--no-sandbox'];

  // Chromium flags tuned for GPU/WebGPU in headless based on latest guidance
  const args = isMac
    ? [
        ...common,
        '--enable-webgl',
        '--ignore-gpu-blocklist',
        '--use-gl=angle',
        '--use-angle=metal',
        '--enable-unsafe-webgpu',
      ]
    : isLinux
    ? [
        ...common,
        '--ignore-gpu-blocklist',
        '--enable-unsafe-webgpu',
        '--use-angle=vulkan',
        '--enable-features=Vulkan',
        '--disable-vulkan-surface',
      ]
    : [
        ...common,
        '--enable-unsafe-webgpu',
      ];

  const browser = await chromium.launch({
    // Use new headless mode which has better WebGPU support
    headless: true,
    // Prevent Playwright from injecting legacy GPU-disabling defaults
    ignoreDefaultArgs: ['--disable-gpu'],
    args,
  });
  const page = await browser.newPage();

  let ok = false;
  const errors = [];

  page.on('console', (msg) => {
    const text = msg.text();
    let localLink = '';
    try {
      // Prefer stack frames inside the console text over message location (which often points to the runner)
      const frames = Array.from(text.matchAll(/(http[s]?:[^\s\)]+):(\d+):(\d+)/g));
      let chosen = null;
      for (const m of frames) {
        const urlStr = m[1];
        if (urlStr.includes('/healthcheck/generated_examples.mjs') || urlStr.includes('/healthcheck/main.js')) continue;
        // Prefer example files if present
        chosen = m;
        if (urlStr.includes('/examples/')) { chosen = m; break; }
      }
      if (chosen) {
        const urlStr = chosen[1];
        const ln = Number(chosen[2] || 0);
        const col = Number(chosen[3] || 0);
        let absLocal = '';
        if (urlStr.startsWith('http://') || urlStr.startsWith('https://')) {
          const u = new URL(urlStr);
          const relPath = decodeURIComponent(u.pathname);
          const safeRel = path.normalize(relPath).replace(/^\/+/, '');
          absLocal = path.join(WEB_ROOT, safeRel);
        } else if (urlStr.startsWith('file://')) {
          absLocal = fileURLToPath(urlStr);
        }
        if (absLocal) {
          localLink = `${absLocal}:${ln}:${col}`;
        }
      } else {
        // Fallback to the message location, but avoid printing runner files
        const loc = msg.location();
        const urlStr = loc?.url || '';
        if (urlStr && !urlStr.includes('/healthcheck/generated_examples.mjs') && !urlStr.includes('/healthcheck/main.js')) {
          let absLocal = '';
          if (urlStr.startsWith('http://') || urlStr.startsWith('https://')) {
            const u = new URL(urlStr);
            const relPath = decodeURIComponent(u.pathname);
            const safeRel = path.normalize(relPath).replace(/^\/+/, '');
            absLocal = path.join(WEB_ROOT, safeRel);
          } else if (urlStr.startsWith('file://')) {
            absLocal = fileURLToPath(urlStr);
          }
          if (absLocal) {
            const ln = loc.lineNumber || 0;
            const col = loc.columnNumber || 0;
            localLink = `${absLocal}:${ln}:${col}`;
          }
        }
      }
    } catch {}
    if (msg.type() === 'error' && localLink) {
      console.error(`\n${localLink}\n`);
    }
    console.log('[console]', text);
    if (
      text.includes('Headless JS render completed successfully') ||
      text.includes('✅ test result: ok')
    ) ok = true;
  });
  page.on('pageerror', (err) => {
    // Try to extract a source URL:line:col from the stack and map to local file
    try {
      const stack = err?.stack || '';
      const frames = Array.from(stack.matchAll(/(http[s]?:[^\s\)]+):(\d+):(\d+)/g));
      let chosen = null;
      for (const m of frames) {
        const urlStr = m[1];
        if (urlStr.includes('/healthcheck/generated_examples.mjs') || urlStr.includes('/healthcheck/main.js')) continue;
        chosen = m;
        if (urlStr.includes('/examples/')) { chosen = m; break; }
      }
      if (chosen) {
        const urlStr = chosen[1];
        const ln = Number(chosen[2] || 0);
        const col = Number(chosen[3] || 0);
        let absLocal = '';
        if (urlStr.startsWith('http://') || urlStr.startsWith('https://')) {
          const u = new URL(urlStr);
          const relPath = decodeURIComponent(u.pathname);
          const safeRel = path.normalize(relPath).replace(/^\/+/, '');
          absLocal = path.join(WEB_ROOT, safeRel);
        }
        if (absLocal) {
          console.error(`\n${absLocal}:${ln}:${col}\n`);
        }
      }
    } catch {}
    console.error('[pageerror]', err?.message || String(err));
    if (err?.stack) console.error(err.stack);
    errors.push({ type: 'pageerror', message: err?.message || String(err) });
  });
  page.on('requestfailed', (req) => {
    const item = `[requestfailed] ${req.url()} ${req.failure()?.errorText || ''}`;
    console.error(item);
    errors.push({ type: 'requestfailed', message: item });
  });
  page.on('response', (resp) => {
    if (!resp.ok()) {
      const item = `[response] ${resp.status()} ${resp.url()}`;
      console.error(item);
      errors.push({ type: 'response', message: item });
    }
  });

  await page.goto(url, { waitUntil: 'load', timeout: 60000 });

  // Optional: probe WebGPU to report adapter details if available
  try {
    const probe = await page.evaluate(async () => {
      if (!('gpu' in navigator)) return { supported: false };
      const adapter = await navigator.gpu.requestAdapter();
      if (!adapter) return { supported: true, gotAdapter: false };
      const info = { name: adapter.name, features: Array.from(adapter.features || []) };
      // Some implementations expose isFallbackAdapter
      if ('isFallbackAdapter' in adapter) info.isFallbackAdapter = adapter.isFallbackAdapter;
      return { supported: true, gotAdapter: true, info };
    });
    console.log('[gpu-probe]', JSON.stringify(probe));
  } catch {}

  // Prefer waiting for the success message rather than fixed sleep
  try {
    await page.waitForEvent('console', {
      predicate: (m) => {
        const t = m.text();
        return t.includes('Headless JS render completed successfully') || t.includes('✅ test result: ok');
      },
      timeout: 20000,
    });
    ok = true;
  } catch (e) {
    // fall through, we'll handle as failure below
  }

  if (!ok) {
    try { await page.screenshot({ path: path.join(ARTIFACT_DIR, 'failure.png'), fullPage: true }); } catch {}
    try { await fs.writeFile(path.join(ARTIFACT_DIR, 'page.html'), await page.content()); } catch {}
  }

  await browser.close();

  if (!ok || errors.length) {
    if (!ok) {
      console.error('Healthcheck marker not found in browser console');
    }
    if (errors.length) {
      console.error('Collected errors:', JSON.stringify(errors, null, 2));
    }
    process.exit(1);
  }
  console.log('Web healthcheck PASS');
})();

