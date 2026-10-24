// The read-only index.js tab: the page code that runs main.js. It follows what
// host.js does each build and each frame, without the playground's own editor,
// controls, and error box. Every demo, the compute ones included, runs on it.

export const HARNESS = /* js */ `// Runs main.js the way the playground does. In a Vite project, keep this file next
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
