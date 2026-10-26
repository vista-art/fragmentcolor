import init, {
  Renderer, Shader, Pass, Mesh, Vertex, Instance, Scene, Model, Material, Camera, Mipmap,
  TextureFormat, TextureUsage, set_log_level,
} from "fragmentcolor";
import * as fc from "fragmentcolor";

// import { installInstrumentation } from './instrument.mjs';
// Install JS-level instrumentation before running any examples or docs coverage.
// installInstrumentation({ Renderer, Shader, Pass, TextureTarget, WindowTarget });

const wasmUrl = new URL("./pkg/fragmentcolor_bg.wasm", import.meta.url);
await init({ module_or_path: wasmUrl.href });

// Raise log level to see Naga internals in JS console
try {
  const params = new URLSearchParams(globalThis.location?.search || "");
  const verbose = params.get('verbose') || params.get('v') || '';
  if (verbose === '1' || verbose.toLowerCase() === 'true') {
    set_log_level('debug');
    console.log('[hc] verbose=1: log level set to debug');
  }
} catch {}

// Helper to scope module name for instrumentation and proceed on error
async function withModule(moduleName, fn) {
  globalThis.__HC = globalThis.__HC || { currentModule: null };
  const prev = globalThis.__HC.currentModule;
  globalThis.__HC.currentModule = moduleName;
  try {
    await fn();
  } catch (e) {
    throw e; // propagate failure so healthcheck can fail
  } finally {
    globalThis.__HC.currentModule = prev || null;
  }
}

// Inputs smoke: validate various JS inputs to texture creation before other tests
await withModule('platforms.web.healthcheck.texture.inputs.smoke', async () => {
  const r = new Renderer();

  // 1) Plain Array (1x1 RGBA)
  const texA = await r.createTexture([255,255,255,255], { size: [1,1] });
  console.log('created from plain array 1x1');

  // 2) Uint8ClampedArray from ImageData
  const id = new ImageData(new Uint8ClampedArray([255,0,0,255]), 1, 1);
  const texB = await r.createTexture(id.data, { size: [1,1] });
  console.log('created from ImageData.data 1x1');

  // 3) HTMLCanvasElement
  const c = document.createElement('canvas'); c.width = 1; c.height = 1;
  const ctx = c.getContext('2d');
  if (ctx) { ctx.fillStyle = 'white'; ctx.fillRect(0,0,1,1); }
  const texC = await r.createTexture(c, { size: [1,1] });
  console.log('created from HTMLCanvasElement via data URL');

  // also via createTexture (encoded data URL)
  const texC2 = await r.createTexture(c.toDataURL());
  console.log('created from HTMLCanvasElement (createTexture)');

  // 4) OffscreenCanvas (if available)
  if (typeof OffscreenCanvas !== 'undefined') {
    const oc = new OffscreenCanvas(1,1);
    const oc2d = oc.getContext('2d');
    if (oc2d) { oc2d.fillStyle = 'white'; oc2d.fillRect(0,0,1,1); }
    const texD = await r.createTexture(oc, { size: [1,1] }); // raw bytes path
    console.log('created from OffscreenCanvas (raw bytes)');

    const blob = await oc.convertToBlob();
    const buf = await blob.arrayBuffer();
    const texD2 = await r.createTexture(new Uint8Array(buf)); // encoded bytes path
    console.log('created from OffscreenCanvas (encoded PNG bytes)');
  }

  // 5) Data URL treated as image URL
  const dataUrl = c.toDataURL();
  const texE = await r.createTexture(dataUrl);
  console.log('created from data URL');
});

// Run a smoke render but do not abort the page if it fails; continue to run generated examples
await withModule('platforms.web.healthcheck.shader|pass|frame|render.smoke', async () => {
  const renderer = new Renderer();
  const canvas = document.createElement('canvas');
  canvas.width = 64; canvas.height = 64;
  const target = await renderer.createTarget(canvas);
  const textureTarget = await renderer.createTextureTarget([64, 64]);

  const shader = new Shader(`
struct VertexOutput {
    @builtin(position) coords: vec4<f32>,
}
@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    const vertices = array(
        vec2( -1., -1.),
        vec2(  3., -1.),
        vec2( -1.,  3.)
    );
    return VertexOutput(vec4<f32>(vertices[in_vertex_index], 0.0, 1.0));
}
@group(0) @binding(1) var<uniform> resolution: vec2<f32>;
@fragment
fn main(_v: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
`);

  shader.set("resolution", [64.0, 64.0]);

  console.log("shader before call:", shader);
  console.log("target before call:", target);

  console.log("shader, target");
  renderer.render(shader, target);
  console.log("shader after call:", shader);
  console.log("target after call:", target);

  console.log("shader, target 2");
  renderer.render(shader, target);
  console.log("shader after call 2:", shader);
  console.log("target after call 2:", target);

  console.log("shader, target 3");
  renderer.render(shader, target);
  console.log("shader after call 3:", shader);
  console.log("target after call 3:", target);

  console.log("shader, textureTarget");
  renderer.render(shader, textureTarget);

  const rpass = new Pass("single pass");
  rpass.addShader(shader);
  renderer.render(rpass, target);
  renderer.render(rpass, textureTarget);

  const rpass2 = new Pass("second pass");
  rpass2.addShader(shader);
  renderer.render([rpass, rpass2], target);
  renderer.render([rpass, rpass2], textureTarget);

  const res = shader.get("resolution");
  console.log(res);

  let image = await textureTarget.getImage();
  console.log(image);

  console.log(shader.listUniforms());
  console.log(shader.listKeys());
});

// Test texture creation and shader.set parity
await withModule('platforms.web.healthcheck.texture.smoke', async () => {
    const renderer = new Renderer();
    const target = await renderer.createTextureTarget([64, 64]);
    
    // Create 2x2 red-green-blue-white texture from raw RGBA bytes
    const pixels = new Uint8Array([
      255,0,0,255,    0,255,0,255,
      0,0,255,255,    255,255,255,255,
    ]);
    const tex = await renderer.createTexture(pixels, { size: [2, 2] });
    console.log('Created texture from raw RGBA bytes with explicit size');
    
    // Test setting texture on shader
    const shader = new Shader(`
@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;
@group(0) @binding(2) var<uniform> resolution: vec2<f32>;

struct VOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
    var p = array<vec2<f32>, 3>(vec2<f32>(-1.,-1.), vec2<f32>(3.,-1.), vec2<f32>(-1.,3.));
    var uv = array<vec2<f32>, 3>(vec2<f32>(0.,1.), vec2<f32>(2.,1.), vec2<f32>(0.,-1.));
    var out: VOut;
    out.pos = vec4<f32>(p[i], 0., 1.);
    out.uv = uv[i];
    return out;
}
@fragment
fn main(v: VOut) -> @location(0) vec4<f32> {
    return textureSample(tex, samp, v.uv);
}
`);
    
    shader.set("tex", tex);
    shader.set("resolution", [64.0, 64.0]);
    console.log('Set texture on shader successfully');
    
    renderer.render(shader, target);
    const img = await target.getImage();
    console.log('Rendered textured shader:', img);

    // Other exported classes, and plain objects that copy their pointer, are
    // rejected instead of being read as a Texture. The bound texture survives.
    const pass = new Pass('not a texture');
    const notTextures = [
      ['a Shader', shader],
      ['a Pass', pass],
      ['a Mesh', new Mesh()],
      ['a plain object with a pointer', { __wbg_ptr: pass.__wbg_ptr }],
    ];
    for (const [label, value] of notTextures) {
      let threw = false;
      try {
        shader.set("tex", value);
      } catch (e) {
        threw = true;
        console.log(`shader.set rejected ${label}:`, e?.message ?? String(e));
      }
      if (!threw) {
        throw new Error(`shader.set("tex", ${label}) should throw`);
      }
    }
    renderer.render(shader, target);
    console.log('Texture binding intact after rejected sets');
});

// Mesh smoke: positions-only triangle
await withModule('platforms.web.healthcheck.mesh.smoke', async () => {
    const renderer = new Renderer();
    const target = await renderer.createTextureTarget([32, 32]);
    const shader = new Shader(`
struct VOut { @builtin(position) pos: vec4<f32> };
@vertex
fn vs_main(@location(0) pos: vec3<f32>) -> VOut {
  var out: VOut;
  out.pos = vec4<f32>(pos, 1.0);
  return out;
}
@fragment
fn main(_v: VOut) -> @location(0) vec4<f32> { return vec4<f32>(1.,0.,0.,1.); }
`);
    const pass = new Pass('mesh-basic');
    pass.addShader(shader);
    const mesh = new Mesh();
    mesh.addVertex(new Vertex([-0.5, -0.5, 0.0]));
    mesh.addVertex(new Vertex([ 0.5, -0.5, 0.0]));
    mesh.addVertex(new Vertex([ 0.0,  0.5, 0.0]));
    pass.addMesh(mesh);
    renderer.render(pass, target);
    const img = await target.getImage();
    console.log('mesh.basic image bytes:', img?.length || 0);
});

// Mesh smoke: two instances with offsets
// Instance attributes share the location namespace with vertex attributes.
// Vertex position occupies @location(0), so instance attribute "offset" must be at @location(1).
// Use Vertex.set to allocate location 1 for "offset", then createInstance to copy that layout.
await withModule('platforms.web.healthcheck.mesh.instances.smoke', async () => {
    const renderer = new Renderer();
    const target = await renderer.createTextureTarget([32, 32]);
    const shader = new Shader(`
struct VOut { @builtin(position) pos: vec4<f32> };
@vertex
fn vs_main(@location(0) pos: vec3<f32>, @location(1) offset: vec2<f32>) -> VOut {
  var out: VOut;
  let p = vec3<f32>(pos.xy + offset, pos.z);
  out.pos = vec4<f32>(p, 1.0);
  return out;
}
@fragment
fn main(_v: VOut) -> @location(0) vec4<f32> { return vec4<f32>(0.,1.,0.,1.); }
`);
    const mesh = new Mesh();
    mesh.addVertices([
      new Vertex([-0.5, -0.5, 0.0]),
      new Vertex([ 0.5, -0.5, 0.0]),
      new Vertex([ 0.0,  0.5, 0.0]),
    ]);
    // Use a template vertex with the "offset" property at location 1 (position = location 0).
    // createInstance() preserves the prop_locations so instance "offset" stays at location 1.
    const template = new Vertex([0.0, 0.0, 0.0]).set("offset", [0.0, 0.0]);
    mesh.addInstances([
      template.createInstance().set("offset", [0.0, 0.0]),
      template.createInstance().set("offset", [0.25, 0.0]),
    ]);
    shader.addMesh(mesh);
    const pass = new Pass('mesh-inst');
    pass.addShader(shader);
    renderer.render(pass, target);
    const img = await target.getImage();
    console.log('mesh.instances image bytes:', img?.length || 0);
});

// A plain number on a Vertex or Instance takes the type the shader declares at its location.
// The vertex stage compares the values, so every pixel is exactly green or red.
await withModule('platforms.web.healthcheck.vertex.types', async () => {
    const renderer = new Renderer();
    const typed = `
struct VOut {
  @builtin(position) pos: vec4<f32>,
  @location(0) @interpolate(flat) ok: u32,
};
@vertex fn vs_main(
  @location(0) position: vec2<f32>,
  @location(1) id: u32,
  @location(2) cell: vec2<i32>,
) -> VOut {
  var out: VOut;
  out.pos = vec4f(position, 0., 1.);
  out.ok = select(0u, 1u, id == 16777217u && all(cell == vec2<i32>(-1, 2)));
  return out;
}
@fragment fn fs_main(v: VOut) -> @location(0) vec4<f32> {
  if (v.ok == 1u) { return vec4f(0., 1., 0., 1.); }
  return vec4f(1., 0., 0., 1.);
}
`;
    const corners = [[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]];
    const green = async (label, shader) => {
      const target = await renderer.createTextureTarget([8, 8]);
      renderer.render(Pass.fromShader(label, shader), target);
      const image = await target.getImage();
      const pixel = Array.from(image.slice(0, 4));
      if (pixel.join() !== '0,255,0,255') throw new Error(`${label}: pixel ${pixel}, want green`);
    };

    // 2^24 + 1 only survives as an integer, so a float transport would draw red.
    const vertices = new Mesh();
    for (const corner of corners) {
      vertices.addVertex(new Vertex(corner).set('id', 16777217).set('cell', new Int32Array([-1, 2])));
    }
    const vertexShader = new Shader(typed);
    vertexShader.addMesh(vertices);
    await green('vertex numbers', vertexShader);

    const instanced = new Mesh();
    for (const corner of corners) instanced.addVertex(new Vertex(corner).set('cell', [-1, 2]));
    instanced.addInstance({ id: 16777217 });
    const instanceShader = new Shader(typed);
    instanceShader.addMesh(instanced);
    await green('instance numbers', instanceShader);

    // A value that doesn't fit the declared type fails the attach and names it.
    const rejects = (label, mesh, needle) => {
      try {
        new Shader(typed).addMesh(mesh);
      } catch (e) {
        if (!(e instanceof Error) || !e.message.includes(needle)) {
          throw new Error(`${label}: expected an Error naming "${needle}", got ${e?.message ?? e}`);
        }
        console.log(`${label} rejected:`, e.message);
        return;
      }
      throw new Error(`${label} should fail to attach`);
    };
    const negative = new Mesh();
    negative.addVertex(new Vertex([0.0, 0.0]).set('id', 7).set('cell', [0, 0]));
    negative.addVertex(new Vertex([1.0, 0.0]).set('id', -1).set('cell', [0, 0]));
    rejects('-1 for a u32', negative, 'expected u32, got -1 at vertex 1');
    const fraction = new Mesh();
    fraction.addVertex(new Vertex([0.0, 0.0]).set('id', 7).set('cell', [2.5, 0]));
    rejects('2.5 for an i32', fraction, 'expected vec2<i32>, got 2.5');
    console.log('Vertex and Instance numbers take the shader types');
});

// Signal success for Playwright harness if we reached this point
// Push constants smoke: solid color via var<immediate>
await withModule('platforms.web.healthcheck.push_constant.smoke', async () => {
    const renderer = new Renderer();
    const target = await renderer.createTextureTarget([8, 8]);
    const shader = new Shader(`
struct PC { color: vec4<f32> };
var<immediate> pc: PC;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn fs_main() -> @location(0) vec4<f32> { return pc.color; }
`);
    shader.set("pc.color", [1.0, 0.0, 0.0, 1.0]);
    renderer.render(shader, target);
    const img = await target.getImage();
    if (!(img && img.length >= 4)) throw new Error('push-constant image empty');
    const px = [img[0], img[1], img[2], img[3]];
    const expect = [255, 0, 0, 255];
    if (px[0] !== expect[0] || px[1] !== expect[1] || px[2] !== expect[2] || px[3] !== expect[3]) {
      throw new Error(`unexpected pixel ${px} != ${expect}`);
    }
});

// Shader.set shapes plain JS values to each uniform's declared WGSL type.
await withModule('platforms.web.healthcheck.shader.set.values', async () => {
    const shader = new Shader(`
struct Light { color: vec4<f32>, power: f32 };
struct Particles { items: array<vec4<f32>, 2> };
@group(0) @binding(0) var<uniform> count: u32;
@group(0) @binding(1) var<uniform> cell: vec2<i32>;
@group(0) @binding(2) var<uniform> rot: mat2x2<f32>;
@group(0) @binding(3) var<uniform> light: Light;
@group(0) @binding(4) var<uniform> offset: vec2<f32>;
@group(0) @binding(5) var<storage, read> particles: Particles;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn main() -> @location(0) vec4<f32> {
  let x = f32(count) + f32(cell.x) + rot[0].x + light.power + offset.x + particles.items[0].x;
  return vec4f(x, 0., 0., 1.);
}
`);
    const same = (label, got, want) => {
      const a = Array.from(got), b = Array.from(want);
      if (a.length !== b.length || a.some((v, i) => v !== b[i])) {
        throw new Error(`${label}: got [${a}], want [${b}]`);
      }
    };
    const rejects = (label, key, value) => {
      try {
        shader.set(key, value);
      } catch (e) {
        console.log(`shader.set rejected ${label}:`, e?.message ?? String(e));
        return;
      }
      throw new Error(`shader.set("${key}", ${label}) should throw`);
    };

    shader.set("count", 3);
    if (shader.get("count") !== 3) throw new Error(`u32 from a number: got ${shader.get("count")}`);
    shader.set("cell", [2, -1]);
    same("vec2<i32> from an array", shader.get("cell"), [2, -1]);
    shader.set("rot", [1, 2, 3, 4]);
    same("mat2x2 from four numbers", shader.get("rot"), [1, 2, 3, 4]);
    shader.set("offset", new Int32Array([5, 6]));
    same("vec2<f32> from an Int32Array", shader.get("offset"), [5, 6]);

    rejects("three numbers for a vec2", "offset", [1, 2, 3]);
    same("vec2 kept after a rejected set", shader.get("offset"), [5, 6]);
    rejects("3.5 for a u32", "count", 3.5);
    rejects("-1 for a u32", "count", -1);
    rejects("bytes for a vec2", "offset", new Uint8Array(8));

    const blob = Uint8Array.from({ length: 32 }, (_, i) => i);
    shader.set("particles", blob);
    same("storage root from a Uint8Array", shader.get("particles"), blob);

    shader.set("light.power", 0.5);
    const light = shader.get("light");
    if (!(light instanceof Uint8Array) || light.length !== 32) {
      throw new Error(`struct root should read as 32 bytes, got ${light}`);
    }
    same("struct root reads current bytes", new Float32Array(light.buffer, 16, 1), [0.5]);
});

// The JS shapes 0.12.6 brings in line with Rust, Python, Swift and Kotlin.
await withModule('platforms.web.healthcheck.parity', async () => {
    const renderer = new Renderer();
    const fail = (message) => { throw new Error(message); };
    const caught = (label, error) => {
      if (!(error instanceof Error)) fail(`${label} threw a ${typeof error}, not an Error`);
      console.log(`${label} threw:`, error.message);
    };
    const throws = (label, run) => {
      try { run(); } catch (e) { return caught(label, e); }
      fail(`${label} should throw`);
    };
    const rejects = async (label, run) => {
      try { await run(); } catch (e) { return caught(label, e); }
      fail(`${label} should reject`);
    };
    const same = (label, got, want) => {
      const a = Array.from(got), b = Array.from(want);
      if (a.length !== b.length || a.some((v, i) => v !== b[i])) fail(`${label}: got [${a}], want [${b}]`);
    };

    // Failures are Error objects, and a bad shader throws instead of falling back.
    throws('new Shader with invalid WGSL', () => new Shader('not wgsl'));
    await rejects('Shader.fetch with a number', () => Shader.fetch(42));
    throws('Shader.set on a missing key', () => Shader.default().set('missing', 1.0));

    // Scene.setPasses leaves the caller's passes usable.
    const first = new Pass('first');
    const second = new Pass('second');
    const scene = new Scene();
    scene.setPasses([first, second]);
    if (first.name() !== 'first' || second.name() !== 'second') fail('setPasses consumed its passes');
    if (scene.listPasses().length !== 2) fail('setPasses did not keep two passes');
    throws('Scene.setPasses with a number item', () => scene.setPasses([first, 1]));

    // Builder methods return the receiver.
    const mesh = new Mesh();
    mesh.addVertex(Vertex.pbr([0.0, 0.5, 0.0]));
    const model = new Model(mesh, Material.pbr());
    const camera = Camera.perspective(1.047, 1.0, 0.1, 100.0);
    const built = new Scene().add(model).add(camera).ambient([0.1, 0.1, 0.1]).noDefaultLight();
    if (built.models().length !== 1) fail('chained Scene.add lost the model');
    if (!(new Pass('chain').add(model) instanceof Pass)) fail('Pass.add should return the pass');

    // Indices, counts and dispatch sizes are whole numbers.
    throws('Scene.addTo(-1)', () => built.addTo(-1, model));
    throws('Scene.getPass(1.5)', () => built.getPass(1.5));
    throws('Mesh.setInstanceCount(-1)', () => mesh.setInstanceCount(-1));
    throws('Mesh.setIndices([0.5])', () => mesh.setIndices([0.5]));
    throws('Pass.setComputeDispatch(-1, 1, 1)', () => first.setComputeDispatch(-1, 1, 1));

    // Pass.require takes a Scene, and throws on anything it can't render.
    const compose = new Pass('compose');
    compose.require(built);
    throws('Pass.require with a string', () => compose.require('nope'));
    throws('Pass.require with a string item', () => compose.require([first, 'nope']));

    // An array renders as one frame, and load takes an array.
    const target = await renderer.createTextureTarget([8, 8]);
    await renderer.load([Shader.default(), built]);
    renderer.render([Shader.default(), Shader.default()], target);
    throws('render with a string item', () => renderer.render([Shader.default(), 'x'], target));

    // The window target is WindowTarget, and CanvasTarget names the same class.
    const canvas = document.createElement('canvas');
    canvas.width = 8; canvas.height = 8;
    const windowTarget = await renderer.createTarget(canvas);
    if (!(windowTarget instanceof fc.WindowTarget)) fail('createTarget should return a WindowTarget');
    if (fc.CanvasTarget !== fc.WindowTarget) fail('CanvasTarget should alias WindowTarget');
    // A frame on screen can't be read back, so getImage rejects with an Error.
    await rejects('WindowTarget.getImage', () => windowTarget.getImage());

    // createTexture takes an ImageData's size, and reads sampler and usage.
    const image = new ImageData(new Uint8ClampedArray([255, 0, 0, 255, 0, 255, 0, 255]), 2, 1);
    const fromImage = await renderer.createTexture(image);
    const size = fromImage.size();
    if (size.width !== 2 || size.height !== 1) fail(`ImageData texture is ${size.width}x${size.height}`);
    await renderer.createTexture(new Uint8Array([255, 255, 255, 255]), {
      size: [1, 1], sampler: { repeatX: true, smooth: false },
    });
    await rejects('createTexture with mipmaps: "yes"', () =>
      renderer.createTexture(new Uint8Array(4), { size: [1, 1], mipmaps: 'yes' }));
    const readable = await renderer.createTexture(new Uint8Array([1, 2, 3, 255]), {
      size: [1, 1], format: TextureFormat.Rgba8Unorm, mipmaps: false, usage: TextureUsage.COPY_SRC,
    });
    same('readTexture after usage: COPY_SRC', (await renderer.readTexture(readable.id())).slice(0, 4), [1, 2, 3, 255]);

    // An ImageData keeps its size and RGBA pixels, and Bgra8Unorm stores them blue first.
    await rejects('createTexture(ImageData) with another size', () =>
      renderer.createTexture(image, { size: [1, 1] }));
    await rejects('createTexture(ImageData) as R8Unorm', () =>
      renderer.createTexture(image, { format: TextureFormat.R8Unorm }));
    const red = new ImageData(new Uint8ClampedArray([255, 0, 0, 255]), 1, 1);
    const bgra = await renderer.createTexture(red, {
      format: TextureFormat.Bgra8Unorm, mipmaps: false, usage: TextureUsage.COPY_SRC,
    });
    same('ImageData into Bgra8Unorm', await renderer.readTexture(bgra.id()), [0, 0, 255, 255]);
    // A region object needs its extent, and its bounds in order.
    const canvasTexture = await renderer.createStorageTexture([16, 16], TextureFormat.Rgba);
    const patch = new Uint8Array(4 * 4 * 4);
    canvasTexture.writeRegion(patch, { minX: 4, maxX: 8, minY: 0, maxY: 4 });
    throws('writeRegion with reversed bounds', () =>
      canvasTexture.writeRegion(patch, { minX: 8, maxX: 4, minY: 0, maxY: 4 }));
    throws('writeRegion with misspelled keys', () => canvasTexture.writeRegion(patch, { x: 0, y: 0, w: 4, h: 4 }));
    throws('writeRegion with minZ 2**32 - 1', () =>
      canvasTexture.writeRegion(patch, { minX: 0, maxX: 4, minY: 0, maxY: 4, minZ: 4294967295 }));
    throws('setSamplerOptions with compare 1.5', () => canvasTexture.setSamplerOptions({ compare: 1.5 }));
    // An options object keeps its size under `size`, so a width beside format is an error.
    await rejects('createTexture options with width and format', () =>
      renderer.createTexture(new Uint8Array(4), { width: 1, height: 1, format: TextureFormat.Rgba8Unorm }));

    // Material texture setters take any texture source.
    Material.pbr().baseColorTexture(new Uint8Array([255, 0, 0, 255]), { size: [1, 1] });
    Material.pbr().baseColorTexture(fromImage);

    // Scene.load takes an ArrayBuffer or a view, and can skip the file's cameras and lights.
    const glb = await (await fetch('/healthcheck/public/model.glb')).arrayBuffer();
    const loaded = Scene.load(glb);
    const bare = Scene.load(new DataView(glb), { cameras: false, lights: false });
    if (bare.cameras().length !== 0 || bare.lights().length !== 0) fail('Scene.load kept filtered nodes');
    if (bare.models().length !== loaded.models().length) fail('Scene.load options dropped models');
    throws('Scene.load with cameras: "no"', () => Scene.load(glb, { cameras: 'no' }));

    // Matrices as columns, and bytes from an ArrayBuffer.
    const columns = [[2, 0, 0, 0], [0, 2, 0, 0], [0, 0, 2, 0], [3, 0, 0, 1]];
    model.setTransform(columns);
    same('Model.transform after columns', model.transform(), columns.flat());
    const shader = new Shader(`
struct Data { items: array<vec4<f32>, 2> };
@group(0) @binding(0) var<storage, read> data: Data;
@group(0) @binding(1) var<uniform> m: mat2x2<f32>;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  return vec4f(0., 0., 0., 1.);
}
@fragment fn main() -> @location(0) vec4<f32> { return data.items[0] + vec4f(m[0], 0., 0.); }
`);
    shader.set('m', [[1, 2], [3, 4]]);
    same('mat2x2 from columns', shader.get('m'), [1, 2, 3, 4]);
    const blob = Uint8Array.from({ length: 32 }, (_, i) => i);
    shader.set('data', blob.buffer);
    same('storage root from an ArrayBuffer', shader.get('data'), blob);

    // Vertex constants, attribute values, and vertices given in parts.
    if (Vertex.UV0 !== 'uv0' || Vertex.NORMAL !== 'normal') fail('Vertex constants are missing');
    throws('Vertex.set with a string item', () => new Vertex([0.0, 0.0]).set('uv', [1, 'x']));
    throws('Vertex.set with five numbers', () => new Vertex([0.0, 0.0]).set('uv', [1, 2, 3, 4, 5]));
    throws('Instance.set with an empty list', () => new Instance().set('offset', []));
    new Mesh().addVertex([[0.0, 0.0, 0.0], [0.0, 1.0]]);
    new Mesh().addVertices([[[1.0, 0.0, 0.0], [1.0, 1.0], [1.0, 1.0, 1.0, 1.0]]]);

    // Mipmap.levels returns every level.
    const chain = Mipmap.build(new Uint8Array(4 * 4 * 4), TextureFormat.Rgba8Unorm, [4, 4]);
    const levels = chain.levels();
    if (levels.length !== chain.count() || !(levels[0] instanceof Uint8Array)) fail('Mipmap.levels is wrong');
});

// Auto-generated: run all extracted examples (after init)
await withModule('platforms.web.healthcheck.generated_examples', async () => {
  const mod = await import('./generated_examples.mjs');
  const { passed, failed } = await mod.runExamples();
  if (failed === 0) {
    console.log(`\n✅ test result: ok. ${passed} passed; ${failed} failed`);
  } else {
    throw new Error(`${failed} JS examples failed`);
  }
});
