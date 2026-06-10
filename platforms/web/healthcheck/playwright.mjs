import { chromium } from 'playwright';
import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const WEB_ROOT = path.resolve(path.join(__dirname, '..'));

const url = process.argv[2] || 'http://localhost:8765/healthcheck/';
const ARTIFACT_DIR = process.env.ARTIFACT_DIR || path.join(process.cwd(), 'platforms/web/healthcheck/playwright-artifacts');

// Hard watchdog: if any of the awaits below hang, this hammer fires and
// forces process.exit so CI logs a failure instead of stalling for the
// 6-hour job ceiling. 5 minutes is plenty for the healthy run (~30 s
// total) and short enough that a stuck Playwright / wgpu adapter shows
// up promptly. Independent of the workflow `timeout-minutes` knob.
const WATCHDOG_MS = 5 * 60 * 1000;
const WATCHDOG = setTimeout(() => {
  console.error(`[watchdog] script exceeded ${WATCHDOG_MS / 1000}s; forcing exit(2)`);
  process.exit(2);
}, WATCHDOG_MS);
WATCHDOG.unref?.();

(async () => {
  console.log('[playwright] starting; target:', url);
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

  console.log('[playwright] launching chromium…');
  const browser = await chromium.launch({
    // Use new headless mode which has better WebGPU support
    headless: true,
    // Prevent Playwright from injecting legacy GPU-disabling defaults
    ignoreDefaultArgs: ['--disable-gpu'],
    args,
  });
  console.log('[playwright] chromium launched; opening newPage…');
  const page = await browser.newPage();
  console.log('[playwright] newPage ready');

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

  console.log('[playwright] page.goto…');
  await page.goto(url, { waitUntil: 'load', timeout: 60000 });
  console.log('[playwright] page loaded');

  // Optional: probe WebGPU to report adapter details if available.
  // Race against a 10-second deadline so a stuck adapter never blocks
  // the script — `requestAdapter` is known to hang on misconfigured
  // headless GPUs.
  try {
    const probe = await Promise.race([
      page.evaluate(async () => {
        if (!('gpu' in navigator)) return { supported: false };
        const adapter = await navigator.gpu.requestAdapter();
        if (!adapter) return { supported: true, gotAdapter: false };
        const info = { name: adapter.name, features: Array.from(adapter.features || []) };
        if ('isFallbackAdapter' in adapter) info.isFallbackAdapter = adapter.isFallbackAdapter;
        return { supported: true, gotAdapter: true, info };
      }),
      new Promise((resolve) => setTimeout(() => resolve({ probeTimedOut: true }), 10_000)),
    ]);
    console.log('[gpu-probe]', JSON.stringify(probe));
  } catch (e) {
    console.log('[gpu-probe] threw:', e?.message || String(e));
  }

  console.log('[playwright] waiting for healthcheck console marker (90 s)…');
  // Prefer waiting for the success message rather than fixed sleep. The
  // headroom covers verbose runs (FC_HEALTHCHECK_VERBOSE=1), where every
  // example's WGSL parse logs per-expression resolution lines that Playwright
  // captures one by one — the wall-clock grows with the example suite, not
  // because anything hung.
  try {
    await page.waitForEvent('console', {
      predicate: (m) => {
        const t = m.text();
        return t.includes('Headless JS render completed successfully') || t.includes('✅ test result: ok');
      },
      timeout: 90000,
    });
    ok = true;
    console.log('[playwright] marker seen; ok=true');
  } catch (e) {
    console.log('[playwright] marker not seen within 90 s');
    // fall through, we'll handle as failure below
  }

  if (!ok) {
    try { await page.screenshot({ path: path.join(ARTIFACT_DIR, 'failure.png'), fullPage: true }); } catch {}
    try { await fs.writeFile(path.join(ARTIFACT_DIR, 'page.html'), await page.content()); } catch {}
  }

  // browser.close() can block indefinitely if the wasm renderer keeps the
  // GPU surface alive (a busy raymarcher loop or a dangling wgpu submission).
  // Race it against a 5 s ceiling, then force-exit either way so the CI
  // step terminates promptly. The watchdog above is the final safety net.
  console.log('[playwright] closing browser…');
  await Promise.race([
    browser.close(),
    new Promise((resolve) => setTimeout(resolve, 5_000)),
  ]).catch(() => {});
  console.log('[playwright] browser close returned');

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
  process.exit(0);
})().catch((err) => {
  // Promote a top-level rejection into an explicit error so the watchdog
  // isn't the only line of defence.
  console.error('[playwright] top-level rejection:', err?.message || String(err));
  if (err?.stack) console.error(err.stack);
  process.exit(3);
});

