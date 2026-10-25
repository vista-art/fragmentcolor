// Captures canvases from the docs site as small WebP files: the still frames
// that tutorial figures annotate, the playground's demo thumbnails, and the
// animated recordings of the playground that the README shows.
//
// Usage:
//   ./fc capture figures <tutorial>     every capture in docs/tutorials/<tutorial>/figures.json
//   ./fc capture demo <step file> --out <file.webp> [--size 720x420] [--time 1.5] [--pointer 360,210]
//   ./fc capture thumbs [demo ids...]   playground thumbnails, all demos by default
//   ./fc capture record <script>        a scripted interaction as an animated WebP (see record.mjs)
//
// Options:
//   --url <origin>   a running docs server; without it the tool starts one and stops it after
//   --scale <n>      device pixels per figure unit (2 by default, 1 for thumbnails)
//   --wait <s>       seconds a thumbnail demo runs before its capture (2.5 by default)
//   --dir <folder>   where thumbnails go (docs/website/public/playground/thumbs by default)
//   --fps <n>        frames a recording tries to take each second (10 by default)
//   --quality <n>    WebP quality of a recording (72 by default)
//
// Sizes, pointers and annotations share one coordinate system: the figure's own
// units, 720 wide by default. A capture at --scale 2 is twice that in pixels.
//
// Playwright comes from platforms/web/healthcheck and sharp from docs/website,
// so `pnpm install` in those two folders is the only setup. WebGPU canvases read
// back as zeros, so every capture is a screenshot of the canvas element.

import { execFile as execFileCallback, execFileSync } from "node:child_process";
import { mkdir, readFile } from "node:fs/promises";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";
import { record } from "./record.mjs";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const SITE = path.join(ROOT, "docs/website");
const { chromium } = createRequire(path.join(ROOT, "platforms/web/healthcheck/package.json"))("playwright");
const sharp = createRequire(path.join(SITE, "package.json"))("sharp");
const execFile = promisify(execFileCallback);

const THUMB_SIZE = [384, 240];
const PORT = 4399;

main().catch((error) => {
  console.error(`capture: ${error.message}`);
  process.exit(1);
});

async function main() {
  const { command, positional, options } = parseArgs(process.argv.slice(2));
  if (!command || command === "help") return printHelp();

  const server = await startServer(options.url);
  const browser = await launchBrowser();
  try {
    if (command === "figures") await captureFigures(browser, server.origin, positional[0], options);
    else if (command === "demo") await captureDemoCommand(browser, server.origin, positional[0], options);
    else if (command === "thumbs") await captureThumbs(browser, server.origin, positional, options);
    else if (command === "record") await recordCommand(browser, server.origin, positional, options);
    else throw new Error(`unknown command "${command}"`);
  } finally {
    await browser.close();
    server.stop();
  }
}

function printHelp() {
  const text = [
    "Usage: ./fc capture <command> [args] [options]",
    "",
    "  figures <tutorial>   every capture in docs/tutorials/<tutorial>/figures.json",
    "  demo <step file>     one tutorial step: --out <file.webp> [--size WxH] [--time s] [--pointer x,y]",
    "  thumbs [ids...]      playground thumbnails in docs/website/public/playground/thumbs/",
    "  record <scripts...>  scripted playground interactions as animated WebP, such as docs/media/*.record.mjs",
    "",
    "  --url <origin>       use a running docs server instead of starting one",
    "  --scale <n>          device pixels per figure unit (default 2, thumbnails 1)",
    "  --wait <s>           seconds a thumbnail demo runs first (default 2.5)",
    "  --dir <folder>       where thumbnails go (default docs/website/public/playground/thumbs)",
    "  --fps <n>            frames per second a recording aims for (default 10)",
    "  --quality <n>        WebP quality of a recording (default 72)",
  ];
  console.log(text.join("\n"));
}

function parseArgs(args) {
  const [command, ...rest] = args;
  const positional = [];
  const options = {};
  for (let i = 0; i < rest.length; i++) {
    const arg = rest[i];
    if (arg.startsWith("--")) options[arg.slice(2)] = rest[++i];
    else positional.push(arg);
  }
  return { command, positional, options };
}

// The docs server: the one at --url, the project's running dev server, or one this
// run starts in the background and stops at the end. Astro runs one dev server per
// project, so a server someone already started is reused and left running.
async function startServer(url) {
  if (url) {
    if (!(await reachable(url))) throw new Error(`no docs server answers at ${url}`);
    return { origin: url.replace(/\/$/, ""), stop() {} };
  }
  const running = await astro(["dev", "status"]);
  const found = running.match(/running at (https?:\/\/[^\s()"\\]+)/);
  if (found) return { origin: found[1].replace(/\/$/, ""), stop() {} };

  console.log(`capture: starting the docs dev server on port ${PORT}`);
  await astro(["dev", "--background", "--port", String(PORT)]);
  const origin = `http://localhost:${PORT}`;
  const stop = () => execFileSync("pnpm", ["exec", "astro", "dev", "stop"], { cwd: SITE, stdio: "ignore" });
  const deadline = Date.now() + 180_000;
  while (Date.now() < deadline) {
    if (await reachable(origin)) return { origin, stop };
    await sleep(1000);
  }
  stop();
  throw new Error("the docs dev server did not start within three minutes");
}

async function astro(args) {
  const { stdout, stderr } = await execFile("pnpm", ["exec", "astro", ...args], { cwd: SITE });
  return `${stdout}${stderr}`;
}

async function reachable(url) {
  try {
    const response = await fetch(url, { signal: AbortSignal.timeout(5000) });
    return response.ok;
  } catch {
    return false;
  }
}

// Chromium with WebGPU, using the flags the web healthcheck runs with.
function launchBrowser() {
  const flags = {
    darwin: ["--enable-webgl", "--ignore-gpu-blocklist", "--use-gl=angle", "--use-angle=metal", "--enable-unsafe-webgpu"],
    linux: [
      "--ignore-gpu-blocklist",
      "--enable-unsafe-webgpu",
      "--use-angle=vulkan",
      "--enable-features=Vulkan",
      "--disable-vulkan-surface",
    ],
  };
  return chromium.launch({
    headless: true,
    ignoreDefaultArgs: ["--disable-gpu"],
    args: ["--no-sandbox", ...(flags[process.platform] ?? ["--enable-unsafe-webgpu"])],
  });
}

async function captureFigures(browser, origin, tutorial, options) {
  if (!tutorial) throw new Error("figures needs a tutorial folder name, such as drawing-with-distance");
  const manifest = path.join(ROOT, "docs/tutorials", tutorial, "figures.json");
  const figures = JSON.parse(await readFile(manifest, "utf8"));
  for (const figure of figures) {
    const out = path.join(SITE, "public/tutorials", tutorial, `${figure.name}.webp`);
    await captureDemo(browser, origin, {
      file: `${tutorial}/${figure.step}/example.rs`,
      size: figure.size ?? [720, 420],
      time: figure.time,
      pointer: figure.pointer,
      scale: Number(options.scale ?? figure.scale ?? 2),
      out,
    });
  }
}

async function recordCommand(browser, origin, scripts, options) {
  if (!scripts.length) throw new Error("record needs a recording script, such as docs/media/playground.record.mjs");
  for (const script of scripts) await record(browser, origin, script, { ...options, root: ROOT, site: SITE });
}

async function captureDemoCommand(browser, origin, file, options) {
  if (!file || !options.out) throw new Error("demo needs a step file and --out <file.webp>");
  await captureDemo(browser, origin, {
    file,
    size: options.size ? options.size.split("x").map(Number) : [720, 420],
    time: options.time === undefined ? undefined : Number(options.time),
    pointer: options.pointer ? options.pointer.split(",").map(Number) : undefined,
    scale: Number(options.scale ?? 2),
    out: path.resolve(options.out),
  });
}

// One tutorial step, rendered by its <Demo> on the tutorial's page.
async function captureDemo(browser, origin, { file, size, time, pointer, scale, out }) {
  const [width, height] = size;
  const tutorial = file.split("/")[0];
  const address = await findAddress(tutorial);
  const step = path.posix.dirname(file);
  const page = await browser.newPage({ viewport: { width: Math.max(1280, width + 400), height: 1000 }, deviceScaleFactor: scale });
  const errors = watchErrors(page, (url) => isCaptureOf(tutorial, url));
  try {
    await page.goto(`${origin}${address}`, { waitUntil: "load" });
    await page.waitForFunction(() => customElements.get("tutorial-demo"));
    const found = await page.evaluate(
      ({ step, width, height, time, pointer, scale }) => {
        const demo = [...document.querySelectorAll("tutorial-demo")].find((element) =>
          element.dataset.moduleKey?.endsWith(`/${step}/example.js`),
        );
        if (!demo) return false;
        demo.id = "fc-capture";
        demo.style.width = `${width}px`;
        const frame = demo.shadowRoot.querySelector(".frame");
        frame.style.height = `${height}px`;
        frame.style.borderRadius = "0";
        if (time !== undefined && time !== null) demo.dataset.time = String(time);
        if (pointer) demo.dataset.pointer = pointer.map((value) => value * scale).join(",");
        demo.scrollIntoView({ block: "center" });
        demo.shadowRoot.querySelector(".play").click();
        return true;
      },
      { step, width, height, time, pointer, scale },
    );
    if (!found) throw new Error(`no <Demo> for ${file} on ${address}`);

    // Wait for the canvas to reach the requested size, then for a few more frames.
    await page.waitForFunction(
      ({ width, height, scale }) => {
        const canvas = document.querySelector("#fc-capture").shadowRoot.querySelector("canvas");
        return canvas.width === Math.floor(width * scale) && canvas.height === Math.floor(height * scale);
      },
      { width, height, scale },
      { timeout: 60_000 },
    );
    await sleep(1500);
    const failure = await page.evaluate(() => document.querySelector("#fc-capture").shadowRoot.querySelector(".err").textContent);
    if (failure || errors.length) throw new Error(`${file}: ${failure || errors.join("; ")}`);

    const canvas = page.locator("#fc-capture").locator("canvas");
    const png = await canvas.screenshot({ type: "png" });
    await writeWebp(png, out, { quality: 80, resize: [Math.round(width * scale), Math.round(height * scale)] });
  } finally {
    await page.close();
  }
}

// A tutorial's page sits at the `slug` in its frontmatter, or else at tutorials/<folder>/.
async function findAddress(tutorial) {
  const source = await readFile(path.join(SITE, "src/content/docs/tutorials", `${tutorial}.mdx`), "utf8");
  const frontmatter = source.split(/^---$/m)[1] ?? "";
  const slug = frontmatter.match(/^slug:\s*["']?([^"'\s]+)["']?\s*$/m)?.[1] ?? `tutorials/${tutorial}`;
  return `/${slug.replace(/^\/+|\/+$/g, "")}/`;
}

// Playground demos, each running for --wait seconds before a 384x240 capture.
async function captureThumbs(browser, origin, ids, options) {
  const scale = Number(options.scale ?? 1);
  const wait = Number(options.wait ?? 2.5) * 1000;
  const dir = options.dir ? path.resolve(options.dir) : path.join(SITE, "public/playground/thumbs");
  const all = await listDemos(browser, origin);
  const chosen = ids.length ? ids : all;
  for (const id of chosen) {
    if (!all.includes(id)) throw new Error(`the playground has no demo "${id}"`);
    // A fresh page per demo, so no navigation cuts off a page that is still loading.
    const page = await browser.newPage({ viewport: { width: 1280, height: 900 }, deviceScaleFactor: scale * 2 });
    const errors = watchErrors(page);
    try {
      await page.goto(`${origin}/welcome/playground/#${id}`, { waitUntil: "load" });
      await page.waitForFunction((id) => document.querySelector("fc-playground")?.dataset.active === id, id, {
        timeout: 60_000,
      });
      // Buttons and notes drawn over the canvas stay out of the picture.
      await page.addStyleTag({ content: ".fcp-canvas > :not(canvas) { visibility: hidden !important; }" });
      await sleep(wait);
      if (errors.length) throw new Error(`${id}: ${errors.join("; ")}`);
      const png = await page.locator("fc-playground canvas").first().screenshot({ type: "png" });
      const out = path.join(dir, `${id}.webp`);
      await writeWebp(png, out, { quality: 80, resize: THUMB_SIZE.map((value) => value * scale) });
    } finally {
      await page.close();
    }
  }
}

// The playground's demo ids, read from the page's markup before any demo needs to load.
async function listDemos(browser, origin) {
  const page = await browser.newPage();
  try {
    await page.goto(`${origin}/welcome/playground/`, { waitUntil: "domcontentloaded" });
    return await page.evaluate(() => document.querySelector("fc-playground")?.dataset.demos?.split(",") ?? []);
  } finally {
    await page.close();
  }
}

// Collects page errors. `allow` passes a console error by the URL it came from.
function watchErrors(page, allow = () => false) {
  const errors = [];
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("console", (message) => {
    if (message.type() !== "error") return;
    const url = message.location()?.url ?? "";
    if (!allow(url)) errors.push(url ? `${message.text()} (${url})` : message.text());
  });
  return errors;
}

// A tutorial page shows the captures this tool writes, so they 404 until the first run makes them.
function isCaptureOf(tutorial, url) {
  try {
    const { pathname } = new URL(url);
    return pathname.startsWith(`/tutorials/${tutorial}/`) && /\.(webp|png|jpe?g)$/.test(pathname);
  } catch {
    return false;
  }
}

async function writeWebp(png, out, { quality, resize }) {
  await mkdir(path.dirname(out), { recursive: true });
  let image = sharp(png);
  if (resize) image = image.resize(resize[0], resize[1], { fit: "cover" });
  const info = await image.webp({ quality, effort: 6 }).toFile(out);
  console.log(`capture: ${path.relative(ROOT, out)} ${info.width}x${info.height} ${(info.size / 1024).toFixed(1)} KB`);
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
