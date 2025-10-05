import init, { Renderer, Shader } from '../pkg/fragmentcolor.js';
import { EditorView, basicSetup } from 'codemirror';
import { EditorState } from '@codemirror/state';

function logError(text: string) {
  const el = document.getElementById('log') as HTMLDivElement;
  if (!el) return;
  el.textContent = String(text);
  el.style.display = 'block';
}
function clearError() {
  const el = document.getElementById('log') as HTMLDivElement;
  if (!el) return;
  el.textContent = '';
  el.style.display = 'none';
}

const DEFAULT_WGSL = `
struct VertexOutput {
  @builtin(position) coords: vec4<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VertexOutput {
  let v = array(
    vec2(-1., -1.),
    vec2( 3., -1.),
    vec2(-1.,  3.)
  );
  return VertexOutput(vec4<f32>(v[i], 0.0, 1.0));
}

struct Circle {
  position: vec2<f32>,
  radius: f32,
  border: f32,
  color: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> circle: Circle;

@group(0) @binding(1)
var<uniform> resolution: vec2<f32>;

@fragment
fn fs_main(pixel: VertexOutput) -> @location(0) vec4<f32> {
  let normalized = pixel.coords.xy / resolution;
  var uv = -1.0 + 2.0 * normalized;
  if (resolution.x > resolution.y) {
    uv.x *= resolution.x / resolution.y;
  } else {
    uv.y *= resolution.y / resolution.x;
  }

  let circle_pos = circle.position / resolution;
  let dist = distance(uv, circle_pos);
  let r = circle.radius / min(resolution.x, resolution.y);
  let aa = 2.0 / min(resolution.x, resolution.y);
  let border = circle.border / min(resolution.x, resolution.y);

  if (dist > r + (border + aa)) { discard; }

  let circle_sdf = 1.0 - smoothstep(border - aa, border + aa, abs(dist - r));
  let a = circle.color.a * circle_sdf;
  return vec4<f32>(circle.color.rgb * a, a);
}
`;

async function start() {
  await init();

  const canvas = document.getElementById('preview') as HTMLCanvasElement;
  const editorHost = document.getElementById('editor') as HTMLDivElement;
  const divider = document.getElementById('divider') as HTMLDivElement;
  const leftPane = document.getElementById('leftPane') as HTMLDivElement;

  // Fit canvas to its container size
  function fitCanvas() {
    const { clientWidth, clientHeight } = canvas;
    canvas.width = clientWidth;
    canvas.height = clientHeight;
  }
  fitCanvas();
  window.addEventListener('resize', () => {
    fitCanvas();
    if (target && (target as any).resize) {
      (target as any).resize([canvas.width, canvas.height]);
    }
    shaderResolutionUpdate();
    renderOnce();
  });

  const editor = new EditorView({
    state: EditorState.create({
      doc: DEFAULT_WGSL,
      extensions: [
        basicSetup,
        EditorView.theme({ '&': { height: '100%' } }),
        EditorView.updateListener.of((update) => {
          if (update.docChanged) debounceRender();
        }),
      ],
    }),
    parent: editorHost,
  });

  const renderer = new Renderer();
  let target: any = await renderer.createTarget(canvas);

  let timeout: any;
  function debounceRender() {
    clearTimeout(timeout);
    timeout = setTimeout(renderOnce, 400);
  }

  function currentSource(): string {
    return editor.state.doc.toString();
  }

  function shaderResolutionUpdate() {
    try {
      // No-op helper to keep resolution updates in one place
      // In this simple REPL we create a fresh Shader in renderOnce, so this is
      // primarily used around resizing to keep behavior consistent.
    } catch { }
  }

  async function renderOnce() {
    clearError();
    try {
      const shader = new Shader(currentSource());
      shader.set('resolution', [canvas.width, canvas.height]);
      shader.set('circle.radius', 200.0);
      shader.set('circle.color', [1.0, 0.0, 0.0, 0.9]);
      shader.set('circle.border', 4.0);
      shader.set('circle.position', [0.0, 0.0]);
      renderer.render(shader, target);
      console.log('REPL_READY');
    } catch (e: any) {
      console.error('REPL error:', e);
      logError(e?.message || String(e));
    }
  }

  // Draggable divider to resize panes
  (() => {
    let dragging = false;
    let startX = 0;
    let startWidth = 0;

    divider.addEventListener('mousedown', (e) => {
      dragging = true;
      startX = e.clientX;
      startWidth = leftPane.getBoundingClientRect().width;
      document.body.style.userSelect = 'none';
    });

    window.addEventListener('mousemove', (e) => {
      if (!dragging) return;
      const delta = e.clientX - startX;
      const newWidth = Math.max(200, startWidth + delta);
      leftPane.style.flex = `0 0 ${newWidth}px`;
      fitCanvas();
      if (target && (target as any).resize) {
        (target as any).resize([canvas.width, canvas.height]);
      }
      renderOnce();
    });

    window.addEventListener('mouseup', () => {
      if (!dragging) return;
      dragging = false;
      document.body.style.userSelect = '';
    });
  })();

  await renderOnce();
}

start().catch((e) => {
  console.error('Failed to start REPL:', e);
  logError(e?.message || String(e));
});

