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

const start = performance.now();
requestAnimationFrame(function tick(now) {
  requestAnimationFrame(tick);
  if (fit(canvas)) target.resize([canvas.width, canvas.height]);
  const size = [canvas.width, canvas.height];
  const time = (now - start) / 1000;
  frame(state, renderer, target, time, size, pointer ?? [size[0] / 2, size[1] / 2]);
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
