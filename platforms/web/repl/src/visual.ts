import init, { Renderer, Shader } from "../pkg/fragmentcolor.js";

function setStatus(text: string, ok = false) {
  const statusEl = document.getElementById('status') as HTMLDivElement;
  if (!statusEl) return;
  statusEl.textContent = text;
  statusEl.className = ok ? 'ok' : 'fail';
}

(async () => {
  try {
    await init();

    const canvas = document.getElementById('cv') as HTMLCanvasElement;
    const renderer = new Renderer();

    // Try canvas target first, fall back to texture target
    let target: any;
    try {
      target = await (renderer as any).createTarget(canvas);
    } catch (e) {
      console.warn('Canvas target failed, using texture target', e);
      target = await (renderer as any).createTextureTarget([256, 256]);
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
    setStatus('✓ Web renderer OK', true);
  } catch (err: any) {
    console.error(err);
    setStatus('✗ Web renderer FAILED: ' + (err?.message || String(err)));
  }
})();

