// The read-only index.js tab: the page code that runs main.js. It follows what
// host.js does each build and each frame, without the playground's own editor,
// controls, and error box. Every demo, the compute ones included, runs on it,
// and a demo with `keys: true` also gets the keyboard and the touch stick.

export function harness(demo) {
  const [before, after] = HARNESS.split(TICK);
  return demo?.keys ? before + KEYS + TICK.replace(GESTURE_CALL, KEYS_CALL) + after : HARNESS;
}

const GESTURE_CALL = "pointer ?? [size[0] / 2, size[1] / 2], input);";
const KEYS_CALL = "pointer ?? [size[0] / 2, size[1] / 2], input, held());";
const TICK = `const start = performance.now();
requestAnimationFrame(function tick(now) {
  requestAnimationFrame(tick);
  if (fit(canvas)) target.resize([canvas.width, canvas.height]);
  const size = [canvas.width, canvas.height];
  const time = (now - start) / 1000;
  const input = gesture;
  gesture = { zoom: 0, pan: [0, 0] };
  frame(state, renderer, target, time, size, ${GESTURE_CALL}
});
`;

// Keys held while the canvas has focus, and a stick for touch screens.
const KEYS = /* js */ `// Every key held down while the canvas has focus, by KeyboardEvent.code, and
// the touch stick as [x, y] from -1 to 1 with y up. A click or Tab gives the
// canvas focus. Shortcuts with a modifier, Tab and Escape stay with the page.
const down = new Set();
let stick = [0, 0];
const held = () => ({ down: new Set(down), stick: [...stick] });
const scrolling = new Set(["Space", "ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight", "PageUp", "PageDown", "Home", "End"]);
canvas.tabIndex = 0;
canvas.addEventListener("pointerdown", () => canvas.focus({ preventScroll: true }));
canvas.addEventListener("keydown", (event) => {
  if (event.code === "Tab") return;
  if (event.code === "Escape") return canvas.blur();
  // macOS sends no keyup for a key released while Cmd is down, so a modifier lifts every key.
  if (event.metaKey || event.ctrlKey || event.altKey) return down.clear();
  down.add(event.code);
  if (scrolling.has(event.code)) event.preventDefault();
});
canvas.addEventListener("keyup", (event) => down.delete(event.code));
canvas.addEventListener("blur", () => down.clear());

// The stick shows on touch screens, over the canvas's lower left corner.
if (matchMedia("(any-pointer: coarse)").matches) {
  const base = document.createElement("div");
  const knob = document.createElement("div");
  base.style.cssText =
    "position: fixed; left: 16px; bottom: 16px; width: 112px; height: 112px; display: grid; place-items: center; " +
    "border-radius: 50%; background: rgba(255, 255, 255, 0.08); border: 1px solid rgba(255, 255, 255, 0.28); touch-action: none";
  knob.style.cssText = "width: 44px; height: 44px; border-radius: 50%; background: rgba(255, 255, 255, 0.45); pointer-events: none";
  base.append(knob);
  document.body.append(base);
  let finger = null;
  const push = (event) => {
    const rect = base.getBoundingClientRect();
    const radius = rect.width / 2;
    let x = (event.clientX - rect.left - radius) / radius;
    let y = (rect.top + radius - event.clientY) / radius;
    const length = Math.hypot(x, y);
    if (length > 1) [x, y] = [x / length, y / length];
    stick = [x, y];
    knob.style.transform = \`translate(\${x * radius * 0.6}px, \${-y * radius * 0.6}px)\`;
  };
  const release = (event) => {
    if (event.pointerId !== finger) return;
    finger = null;
    stick = [0, 0];
    knob.style.transform = "";
  };
  base.addEventListener("pointerdown", (event) => {
    event.preventDefault();
    base.setPointerCapture(event.pointerId);
    finger = event.pointerId;
    push(event);
  });
  base.addEventListener("pointermove", (event) => event.pointerId === finger && push(event));
  base.addEventListener("pointerup", release);
  base.addEventListener("pointercancel", release);
}

`;

const HARNESS = /* js */ `// Runs main.js the way the playground does. In a Vite project, keep this file next
// to main.js and the .wgsl files, with an index.html that loads it and holds a
// canvas sized by CSS:
//
//   <canvas style="position: fixed; inset: 0; width: 100%; height: 100%"></canvas>
//   <script type="module" src="./index.js"></script>

import init, { Renderer } from "fragmentcolor";
import wasm from "fragmentcolor/fragmentcolor_bg.wasm?url";
import { setup, frame } from "./main.js";

// At most 2 device pixels per CSS pixel, which keeps 3x phone screens fast.
const DPR_CAP = 2;

await init({ module_or_path: wasm });

const canvas = document.querySelector("canvas");
fit(canvas);
const renderer = new Renderer();
const target = await renderer.createTarget(canvas);
const state = await setup(renderer, target);

// The pointer in canvas pixels, with y pointing down. Until it moves over
// the canvas, frame gets the center.
let pointer = null;
function track(event) {
  const rect = canvas.getBoundingClientRect();
  pointer = [
    ((event.clientX - rect.left) / rect.width) * canvas.width,
    ((event.clientY - rect.top) / rect.height) * canvas.height,
  ];
}
canvas.addEventListener("pointermove", track);
canvas.addEventListener("pointerdown", track);

// The wheel, a drag and a two-finger pinch since the last frame: zoom in
// doublings (positive zooms in) and pan in canvas pixels.
let gesture = { zoom: 0, pan: [0, 0] };
const touches = new Map();
const scale = () => canvas.width / Math.max(canvas.getBoundingClientRect().width, 1);
const spread = () => {
  const [a, b] = [...touches.values()];
  return Math.hypot(a[0] - b[0], a[1] - b[1]);
};
const middle = () => {
  const points = [...touches.values()];
  return points.reduce(([x, y], [px, py]) => [x + px / points.length, y + py / points.length], [0, 0]);
};
canvas.style.touchAction = "none";
canvas.addEventListener(
  "wheel",
  (event) => {
    event.preventDefault();
    const lines = event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? canvas.clientHeight : 1;
    // A trackpad pinch arrives as a wheel with ctrlKey and small deltas.
    gesture.zoom -= (event.deltaY * lines) / (event.ctrlKey ? 100 : 400);
  },
  { passive: false },
);
canvas.addEventListener("pointerdown", (event) => {
  if (event.pointerType === "mouse" && event.button !== 0) return;
  canvas.setPointerCapture(event.pointerId);
  touches.set(event.pointerId, [event.clientX, event.clientY]);
});
canvas.addEventListener("pointermove", (event) => {
  const last = touches.get(event.pointerId);
  if (!last) return;
  const before = touches.size === 2 ? { spread: spread(), middle: middle() } : null;
  touches.set(event.pointerId, [event.clientX, event.clientY]);
  const ratio = scale();
  if (before) {
    // A pinch zooms around the point between the two fingers.
    const now = middle();
    const rect = canvas.getBoundingClientRect();
    pointer = [(now[0] - rect.left) * ratio, (now[1] - rect.top) * ratio];
    gesture.zoom += Math.log2(Math.max(spread(), 1) / Math.max(before.spread, 1));
    gesture.pan[0] += (now[0] - before.middle[0]) * ratio;
    gesture.pan[1] += (now[1] - before.middle[1]) * ratio;
  } else if (touches.size === 1) {
    gesture.pan[0] += (event.clientX - last[0]) * ratio;
    gesture.pan[1] += (event.clientY - last[1]) * ratio;
  }
});
canvas.addEventListener("pointerup", (event) => touches.delete(event.pointerId));
canvas.addEventListener("pointercancel", (event) => touches.delete(event.pointerId));

const start = performance.now();
requestAnimationFrame(function tick(now) {
  requestAnimationFrame(tick);
  if (fit(canvas)) target.resize([canvas.width, canvas.height]);
  const size = [canvas.width, canvas.height];
  const time = (now - start) / 1000;
  const input = gesture;
  gesture = { zoom: 0, pan: [0, 0] };
  frame(state, renderer, target, time, size, pointer ?? [size[0] / 2, size[1] / 2], input);
});

// Sizes the canvas to its CSS box in device pixels. True when the size changed.
function fit(canvas) {
  const dpr = Math.min(window.devicePixelRatio || 1, DPR_CAP);
  const width = Math.max(1, Math.round(canvas.clientWidth * dpr));
  const height = Math.max(1, Math.round(canvas.clientHeight * dpr));
  if (width === canvas.width && height === canvas.height) return false;
  canvas.width = width;
  canvas.height = height;
  return true;
}
`;
