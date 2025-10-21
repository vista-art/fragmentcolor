import init, { Renderer, Shader } from "./pkg/fragmentcolor.js";

const statusEl = document.getElementById('status');
const canvas = document.getElementById('cv');

function setStatus(text, ok = false) {
  statusEl.textContent = text;
  statusEl.className = ok ? 'ok' : 'fail';
}

(async () => {
  try {
    const wasmUrl = new URL("./pkg/fragmentcolor_bg.wasm", import.meta.url);
    await init({ module_or_path: wasmUrl.href });

    const renderer = new Renderer();

    // Try a canvas-backed target first
    let target;
    try {
      target = await renderer.createTarget(canvas);
    } catch (e) {
      console.warn('Canvas target failed, falling back to texture target', e);
      target = await renderer.createTextureTarget([256, 256]);
    }

    const shader = new Shader(`
struct VertexOutput { @builtin(position) coords: vec4<f32> }
@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VertexOutput {
  let v = array(vec2(-1.0,-1.0), vec2(3.0,-1.0), vec2(-1.0,3.0));
  return VertexOutput(vec4<f32>(v[i], 0.0, 1.0));
}
@fragment
fn main() -> @location(0) vec4<f32> { return vec4<f32>(0.2, 0.8, 0.3, 1.0); }
`);

    renderer.render(shader, target);

    // If we got here without throwing, consider it OK
    setStatus('✓ Web renderer OK', true);
  } catch (err) {
    console.error(err);
    setStatus('✗ Web renderer FAILED: ' + (err?.message || String(err)));
  }
})();

