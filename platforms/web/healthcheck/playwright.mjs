import { chromium } from 'playwright';

const url = process.argv[2] || 'http://localhost:8765/healthcheck/';

(async () => {
const browser = await chromium.launch({ headless: true, args: [
  '--use-angle=swiftshader',
  '--use-gl=swiftshader-webgl',
  '--disable-gpu-sandbox',
  '--disable-software-rasterizer=false'
] });
  const page = await browser.newPage();

  let ok = false;
  page.on('console', (msg) => {
    const text = msg.text();
    console.log('[console]', text);
    if (text.includes('Headless JS render completed successfully')) ok = true;
  });

  await page.goto(url, { waitUntil: 'load', timeout: 60000 });
  // Allow some time for GPU init and rendering
  await page.waitForTimeout(2000);
  await browser.close();

  if (!ok) {
    console.error('Healthcheck marker not found in browser console');
    process.exit(1);
  }
  console.log('Web healthcheck PASS');
})();

