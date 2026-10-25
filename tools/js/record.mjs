// Plays a scripted interaction on a docs page and writes it as an animated WebP.
//
// A recording script is a module whose default export describes the take:
//
//   export default {
//     page: "/welcome/playground/#cells",  // path on the docs server
//     element: "fc-playground",            // what the frames show
//     width: 928,                          // the element's width in pixels
//     out: "playground.webp",              // next to the script unless absolute
//     async play(r) {
//       await r.click(".fcp-menu-button");
//       await r.caret("2.5", { at: "end" });
//       await r.type("0");
//     },
//   };
//
// `play` drives the page through the recorder `r`, which moves a visible cursor
// the way a person would. Frames come from screenshots taken while `play` runs,
// each shown for as long as it took to capture the next.

import { createRequire } from "node:module";
import path from "node:path";
import { pathToFileURL } from "node:url";

// A cursor drawn into the page, since headless screenshots leave the real one out.
const CURSOR = `
  const cursor = document.createElement("div");
  cursor.id = "fc-record-cursor";
  cursor.innerHTML = '<svg width="22" height="22" viewBox="0 0 22 22"><path d="M3 2 L3 18 L7.5 13.8 L10.6 20.4 L13.4 19.1 L10.4 12.6 L16.4 12.4 Z" fill="#fff" stroke="#111" stroke-width="1.4" stroke-linejoin="round"/></svg>';
  Object.assign(cursor.style, {
    position: "fixed", left: "0", top: "0", zIndex: "2147483647", pointerEvents: "none",
    transform: "translate(-100px, -100px)", transition: "scale 120ms ease", transformOrigin: "3px 2px",
  });
  document.body.append(cursor);
  addEventListener("mousemove", (event) => {
    cursor.style.transform = \`translate(\${event.clientX - 3}px, \${event.clientY - 2}px)\`;
  }, true);
  addEventListener("mousedown", () => (cursor.style.scale = "0.85"), true);
  addEventListener("mouseup", () => (cursor.style.scale = "1"), true);
`;

// Dev-only overlays that shouldn't appear in a recording.
const HIDDEN = "astro-dev-toolbar, .dev-editor-trigger { display: none !important; }";

/**
 * Records the take a script describes.
 * @param {import("playwright").Browser} browser
 * @param {string} origin the docs server
 * @param {string} scriptPath the recording script
 * @param {{ root: string, site: string, fps?: string, quality?: string }} options
 */
export async function record(browser, origin, scriptPath, options) {
  const sharp = createRequire(path.join(options.site, "package.json"))("sharp");
  const file = path.resolve(scriptPath);
  const take = (await import(pathToFileURL(file).href)).default;
  const fps = Number(options.fps ?? take.fps ?? 10);
  const quality = Number(options.quality ?? take.quality ?? 72);
  const out = path.resolve(path.dirname(file), take.out ?? `${path.basename(file).split(".")[0]}.webp`);

  // The page's side margins are 16 pixels, so this width gives the element the requested one.
  const page = await browser.newPage({
    viewport: { width: (take.width ?? 928) + 32, height: take.viewportHeight ?? 1000 },
    colorScheme: take.theme ?? "dark",
  });
  try {
    await page.goto(`${origin}${take.page}`, { waitUntil: "load" });
    await page.addStyleTag({ content: HIDDEN });
    await page.evaluate(CURSOR);
    const element = page.locator(take.element ?? "fc-playground").first();
    await element.waitFor();
    if (take.ready) await take.ready(page);
    else await sleep(2500);

    const recorder = new Recorder(page);
    const frames = [];
    let running = true;
    // One box for the whole take, so every frame has the same size even when panels open.
    const box = await element.boundingBox();
    const capture = (async () => {
      while (running) {
        const started = Date.now();
        const png = await page.screenshot({ type: "png", clip: box });
        frames.push({ at: started, png });
        await sleep(Math.max(0, 1000 / fps - (Date.now() - started)));
      }
    })();
    await take.play(recorder);
    running = false;
    await capture;

    await writeAnimation(sharp, frames, out, { fps, quality });
  } finally {
    await page.close();
  }
}

async function writeAnimation(sharp, frames, out, { fps, quality }) {
  // Each frame stays up until the next one was taken; the last holds for one frame time.
  const delay = frames.map((frame, i) => Math.round((frames[i + 1]?.at ?? frame.at + 1000 / fps) - frame.at));
  const { width, height } = await sharp(frames[0].png).metadata();
  const info = await sharp(frames.map((frame) => frame.png), { join: { animated: true } })
    .webp({ quality, effort: 5, loop: 0, delay, smartSubsample: true })
    .toFile(out);
  const seconds = delay.reduce((sum, value) => sum + value, 0) / 1000;
  console.log(
    `record: ${path.basename(out)} ${width}x${height}, ${frames.length} frames over ${seconds.toFixed(1)} s, ${(info.size / 1024 / 1024).toFixed(2)} MB`,
  );
}

// The actions a recording script can take, each moving the visible cursor.
class Recorder {
  constructor(page) {
    this.page = page;
    this.mouse = [0, 0];
  }

  /** Waits this many seconds. */
  wait(seconds) {
    return sleep(seconds * 1000);
  }

  /** Glides the cursor to a selector, a `{ text }` in the editor, or an `[x, y]` point. */
  async move(target, { duration = 0.6 } = {}) {
    const [x, y] = await this.locate(target);
    const [fromX, fromY] = this.mouse;
    const steps = Math.max(2, Math.round(duration * 30));
    for (let i = 1; i <= steps; i++) {
      const t = i / steps;
      const eased = t < 0.5 ? 2 * t * t : 1 - (-2 * t + 2) ** 2 / 2;
      await this.page.mouse.move(fromX + (x - fromX) * eased, fromY + (y - fromY) * eased);
      await sleep((duration * 1000) / steps);
    }
    this.mouse = [x, y];
  }

  /** Moves to a target and clicks it. */
  async click(target, options = {}) {
    await this.move(target, options);
    await this.page.mouse.down();
    await sleep(90);
    await this.page.mouse.up();
    await sleep(250);
  }

  /** Clicks a code tab by its label, such as "shader.wgsl" or "Rust". */
  tab(label) {
    return this.click(`fc-playground [role=tab]:text-is("${label}"), fc-playground button:text-is("${label}")`);
  }

  /** Puts the editor caret at the start or end of the first visible `text`. */
  caret(text, { at = "end" } = {}) {
    return this.click({ text, at });
  }

  /** Selects the first visible `text` in the editor, as a person would with shift and click. */
  async select(text) {
    await this.click({ text, at: "start" });
    await this.page.keyboard.down("Shift");
    await this.click({ text, at: "end" }, { duration: 0.35 });
    await this.page.keyboard.up("Shift");
  }

  /** Types text at the caret, one key at a time. */
  async type(text, { delay = 75 } = {}) {
    await this.page.keyboard.type(text, { delay });
    await sleep(300);
  }

  /** Presses a key, such as "Backspace" or "Escape", `times` times. */
  async press(key, { times = 1, delay = 90 } = {}) {
    for (let i = 0; i < times; i++) {
      await this.page.keyboard.press(key);
      await sleep(delay);
    }
  }

  /** Drags the slider of the control named `label`, such as "u.scale", to `value`. */
  async slide(label, value, { duration = 1.2 } = {}) {
    const slider = this.page.locator(".fcp-control", { hasText: label }).locator("input[type=range]").first();
    const range = await slider.evaluate((input) => ({
      min: Number(input.min),
      max: Number(input.max),
      value: Number(input.value),
    }));
    const box = await slider.boundingBox();
    // Browsers inset the thumb by half its width at each end; 8 pixels covers the playground's.
    const inset = 8;
    const at = (v) => box.x + inset + ((v - range.min) / (range.max - range.min)) * (box.width - 2 * inset);
    const y = box.y + box.height / 2;
    await this.move([at(range.value), y]);
    await this.page.mouse.down();
    await this.move([at(value), y], { duration });
    await this.page.mouse.up();
    await sleep(250);
  }

  async locate(target) {
    if (Array.isArray(target)) return target;
    if (typeof target === "string") {
      const box = await this.page.locator(target).first().boundingBox();
      if (!box) throw new Error(`nothing visible matches ${target}`);
      return [box.x + box.width / 2, box.y + box.height / 2];
    }
    // A position in the editor: the start or end of the first visible line that holds `text`.
    const point = await this.page.evaluate(({ text, at }) => {
      for (const line of document.querySelectorAll("fc-playground .cm-line")) {
        const walker = document.createTreeWalker(line, NodeFilter.SHOW_TEXT);
        const nodes = [];
        while (walker.nextNode()) nodes.push(walker.currentNode);
        const whole = nodes.map((node) => node.data).join("");
        const index = whole.indexOf(text);
        if (index < 0) continue;
        const offset = at === "start" ? index : index + text.length;
        let seen = 0;
        for (const node of nodes) {
          if (offset <= seen + node.data.length) {
            const range = document.createRange();
            range.setStart(node, offset - seen);
            range.setEnd(node, offset - seen);
            const rect = range.getBoundingClientRect();
            return [rect.left, rect.top + rect.height / 2];
          }
          seen += node.data.length;
        }
      }
      return null;
    }, target);
    if (!point) throw new Error(`the editor shows no "${target.text}"`);
    return point;
  }
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
