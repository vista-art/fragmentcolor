import init from "./pkg/fragmentcolor.js";

function exampleNameFromPath(rel) {
  // platforms.web.examples.core.renderer.render -> core/renderer/render
  try {
    const trimmed = rel.replace(/^\.\.\/examples\//, "");
    return trimmed.replace(/\.js$/, "");
  } catch { return rel; }
}

function createCard({ rel, urlBase }) {
  const name = exampleNameFromPath(rel);
  const card = document.createElement('div');
  card.className = 'card';
  const thumb = document.createElement('div');
  thumb.className = 'thumb';
  const canvas = document.createElement('canvas');
  canvas.width = 256; canvas.height = 160;
  thumb.appendChild(canvas);

  const meta = document.createElement('div');
  meta.className = 'meta';
  const nameEl = document.createElement('div');
  nameEl.className = 'name';
  nameEl.textContent = name;
  const actions = document.createElement('div');
  actions.className = 'actions';
  const openBtn = document.createElement('a');
  openBtn.className = 'btn';
  openBtn.textContent = 'Open';
  openBtn.href = `${urlBase}example.html?rel=${encodeURIComponent(rel)}`;
  const status = document.createElement('span');
  status.className = 'status';
  status.textContent = '…';

  actions.appendChild(openBtn);
  meta.appendChild(nameEl);
  meta.appendChild(actions);

  card.appendChild(thumb);
  card.appendChild(meta);

  return { card, canvas, status, name };
}

async function runIntoCanvas(rel, canvas) {
  // For visual proof, we render a small green triangle using the library
  // and then also invoke the example module to ensure it imports and executes.
  // Many examples are non-visual (create objects), so we don't assert pixels here.
  try {
    // Ensure WASM initialized once
    if (!globalThis.__FC_INITED) {
      const wasmUrl = new URL("./pkg/fragmentcolor_bg.wasm", import.meta.url);
      await init({ module_or_path: wasmUrl.href });
      globalThis.__FC_INITED = true;
    }
    // Draw a default green frame so non-visual examples still show a result
    const { Renderer, Shader } = await import('fragmentcolor');
    const r = new Renderer();
    const target = await r.createTarget(canvas);
    const shader = new Shader(`
struct VOut { @builtin(position) pos: vec4<f32> };
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
  var p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  var out: VOut;
  out.pos = vec4f(p[i], 0., 1.);
  return out;
}
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4f(0., 1., 0., 1.); }
`);
    r.render(shader, target);

    // Dynamically import the example; if it runs without throwing, consider OK.
    await import(rel);
    return true;
  } catch (e) {
    console.error('[gallery] example failed', rel, e);
    return false;
  }
}

(async function start() {
  const grid = document.getElementById('grid');
  const urlBase = '/gallery/';

  let list = [];
  try {
    const res = await fetch('./examples.json');
    list = await res.json();
  } catch (e) {
    console.error('Failed to load examples.json', e);
  }

  for (const rel of list) {
    const { card, canvas } = createCard({ rel, urlBase });
    grid.appendChild(card);
    // Fire and forget; run sequentially to avoid burst CPU if desired
    // eslint-disable-next-line no-await-in-loop
    const ok = await runIntoCanvas(rel, canvas);
    const statusEl = document.createElement('div');
    statusEl.className = ok ? 'status ok' : 'status fail';
    statusEl.textContent = ok ? 'OK' : 'FAILED';
    card.querySelector('.meta')?.appendChild(statusEl);
  }
})();
