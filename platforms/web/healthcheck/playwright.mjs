import { chromium } from 'playwright';
import fs from 'node:fs/promises';
import path from 'node:path';

const url = process.argv[2] || 'http://localhost:8765/healthcheck/';
const ARTIFACT_DIR = process.env.ARTIFACT_DIR || path.join(process.cwd(), 'platforms/web/healthcheck/playwright-artifacts');

(async () => {
  await fs.mkdir(ARTIFACT_DIR, { recursive: true }).catch(() => {});

  const isMac = process.platform === 'darwin';
  const isLinux = process.platform === 'linux';

  const common = ['--no-sandbox'];

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
        '--ozone-platform=x11',
        '--enable-features=Vulkan,VulkanFromANGLE,DefaultANGLEVulkan',
      ]
    : [
        ...common,
        '--enable-unsafe-webgpu',
      ];

  const browser = await chromium.launch({
    // Use new headless mode which has better WebGPU support
    headless: true,
    args,
  });
  const page = await browser.newPage();

  let ok = false;
  const errors = [];

  page.on('console', (msg) => {
    const text = msg.text();
    console.log('[console]', text);
    if (text.includes('Headless JS render completed successfully')) ok = true;
  });
  page.on('pageerror', (err) => {
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

  // Ensure we run in headless/CI mode by appending a query param the page can read
  const gotoUrl = url.includes('?') ? `${url}&mode=headless` : `${url}?mode=headless`;
  await page.goto(gotoUrl, { waitUntil: 'load', timeout: 60000 });

  // Prefer waiting for the success message rather than fixed sleep
  try {
    await page.waitForEvent('console', {
      predicate: (m) => m.text().includes('Headless JS render completed successfully'),
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

