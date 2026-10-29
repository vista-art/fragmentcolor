import init, {
  Renderer, Shader, Pass, Mesh, Vertex, Instance, Scene, Model, Material, Camera, Light, Mipmap,
  TextureFormat, TextureUsage, ErrorKind, set_log_level,
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
    // Propagate the failure so the healthcheck fails, named after its module,
    // since an error the library threw doesn't say which check it broke.
    throw new Error(`${moduleName} failed: ${e?.message ?? e}`, { cause: e });
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
    negative.addVertex(new Vertex([0.0, 1.0]).set('id', 7).set('cell', [0, 0]));
    rejects('-1 for a u32', negative, 'expected u32, got -1 at vertex 1');
    const fraction = new Mesh();
    fraction.addVertex(new Vertex([0.0, 0.0]).set('id', 7).set('cell', [2.5, 0]));
    fraction.addVertex(new Vertex([1.0, 0.0]).set('id', 7).set('cell', [0, 0]));
    fraction.addVertex(new Vertex([0.0, 1.0]).set('id', 7).set('cell', [0, 0]));
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
    mesh.addVertices([Vertex.pbr([0.0, 0.5, 0.0]), Vertex.pbr([-0.5, -0.5, 0.0]), Vertex.pbr([0.5, -0.5, 0.0])]);
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
    const occlusion = fc.TextureSlot.Occlusion;
    throws('Material.textureUv(Occlusion, 1.5)', () => Material.pbr().textureUv(occlusion, 1.5));
    throws('Material.textureUv(Occlusion, -1)', () => Material.pbr().textureUv(occlusion, -1));
    // A UV set past 1 is a whole number, but no texture can sample it.
    throws('Material.textureUv(Occlusion, 2)', () => Material.pbr().textureUv(occlusion, 2));
    if (!(Material.pbr().textureUv(occlusion, 1) instanceof Material)) fail('textureUv should return the material');

    // Size fields take only the numbers the constructor takes, and ScreenRegion has no setters.
    const extent = new fc.Size(64, 64);
    throws('Size.width = 2 ** 32 + 64', () => { extent.width = 2 ** 32 + 64; });
    throws('Size.height = -1', () => { extent.height = -1; });
    throws('Size.depth = NaN', () => { extent.depth = NaN; });
    if (extent.width !== 64 || extent.height !== 64 || extent.depth !== undefined) fail('a refused write changed the Size');
    extent.width = 32;
    extent.depth = 2;
    if (extent.width !== 32 || extent.depth !== 2) fail('Size fields did not take valid numbers');
    extent.depth = undefined;
    if (extent.depth !== undefined) fail('Size.depth = undefined did not clear the depth');

    // A size or region takes whole numbers, as counts do, and a fraction throws.
    throws('new Size(1.5, 2)', () => new fc.Size(1.5, 2));
    throws('Size.width = 1.5', () => { extent.width = 1.5; });
    throws('new ScreenRegion([0, 0], [1.5, 2])', () => new fc.ScreenRegion([0, 0], [1.5, 2]));
    throws('Pass.setViewport([0, 0, 1.5, 2])', () => first.setViewport([0, 0, 1.5, 2]));
    await rejects('createTextureTarget([1.5, 2])', () => renderer.createTextureTarget([1.5, 2]));
    // An ImageData brings its own size, so only a size that fails to read can reject.
    const pixels = new ImageData(new Uint8ClampedArray(8), 2, 1);
    await rejects('createTexture(ImageData, [1.5, 2])', () => renderer.createTexture(pixels, [1.5, 2]));
    await rejects('createTexture(ImageData, { width: 1.5, height: 2 })',
      () => renderer.createTexture(pixels, { width: 1.5, height: 2 }));

    // A color or vector item that is NaN, an infinity or past the 32-bit float range
    // throws, and the message names the call, as in every binding.
    const named = (label, run) => {
      try { run(); } catch (e) {
        caught(label, e);
        if (!e.message.includes(label)) fail(`${label} threw "${e.message}", which doesn't name the call`);
        return;
      }
      fail(`${label} should throw`);
    };
    named('Light.point: position', () => Light.point([1e39, 0, 0], [1, 1, 1]));
    named('Light.setColor', () => Light.point([0, 1, 0], [1, 1, 1]).setColor([NaN, 0, 0]));
    named('Pass.setClearColor', () => first.setClearColor([0, Infinity, 0]));
    named('Scene.ambient', () => new Scene().ambient(new Float32Array([0, -Infinity, 0])));
    named('Model.translate', () => model.translate([0, 1e39, 0]));
    named('Model.rotate: axis', () => model.rotate([0, NaN, 0], 1));
    named('Camera.lookAt: up', () => camera.lookAt([0, 0, 5], [0, 0, 0], [0, 1e39, 0]));
    named('Material.baseColor', () => Material.pbr().baseColor([1, 1, NaN, 1]));
    named('Material.uvTransform: offset', () => Material.pbr().uvTransform([1e39, 0], [1, 1], 0));
    named('Material.textureTransform: scale',
      () => Material.pbr().textureTransform(occlusion, [0, 0], [Infinity, 1], 0));
    // A float vector is an array or a Float32Array, as a color is.
    named('Light.point: color', () => Light.point([0, 1, 0], new Float64Array([1, 1, 1])));
    // A vertex position and a Quad corner follow the same rule.
    named('Vertex: position', () => new Vertex([1e39, 0, 0]));
    named('Vertex: position', () => new Vertex(NaN));
    named('Vertex: position', () => new Vertex(new Float64Array([0, 0, 0])));
    named('Vertex.pbr: position', () => Vertex.pbr([0, Infinity, 0]));
    named('Quad: min', () => new fc.Quad([NaN, 0], [1, 1]));
    named('Quad: max', () => new fc.Quad([0, 0], new Float64Array([1, 1])));
    named('Mesh.addVertex: vertex', () => new Mesh().addVertex([0, 1e39, 0]));
    named('Mesh.addVertex: vertex', () => new Mesh().addVertex([[0, NaN, 0], [0, 0]]));
    named('Mesh.addVertices: vertex 1', () => new Mesh().addVertices([[0, 0, 0], [-Infinity, 0, 0]]));
    // A position given in parts is a float vector too, so it takes an array or a Float32Array.
    named('Mesh.addVertex: vertex: position', () => new Mesh().addVertex([new Float64Array([0, 0, 0]), [0, 0]]));
    named('Mesh.addVertices: vertex 0: position', () => new Mesh().addVertices([[new Int32Array([0, 0, 0]), [0, 0]]]));
    named('Mesh.fromVertices: vertex 0: position',
      () => Mesh.fromVertices([[new Uint8Array([0, 0, 0]), [0, 0], [1, 1, 1, 1]]]));
    // Parts take 3, 2 and 4 numbers under the float vector rule,
    // and each error names the call, the vertex and the part.
    named('Mesh.addVertex: vertex: position', () => new Mesh().addVertex([[0, 0], [0, 0]]));
    named('Mesh.addVertex: vertex: uv', () => new Mesh().addVertex([[0, 0, 0], [0, 0, 0]]));
    named('Mesh.addVertices: vertex 0: color', () => new Mesh().addVertices([[[0, 0, 0], [0, 0], [1, 1, 1]]]));
    named('Mesh.fromVertices: vertex 0: uv', () => Mesh.fromVertices([[[0, 0, 0], {}]]));
    named('Mesh.addVertex: vertex', () => new Mesh().addVertex([[0, 0, 0], [0, 0], [1, 1, 1, 1], [0]]));
    named('Mesh.addVertex: vertex: uv: item 0', () => new Mesh().addVertex([[0, 0, 0], [1e39, 0]]));
    named('Mesh.addVertex: vertex: uv: item 1', () => new Mesh().addVertex([[0, 0, 0], [0, NaN], [1, 1, 1, 1]]));
    named('Mesh.addVertices: vertex 1: color: item 3',
      () => new Mesh().addVertices([[0, 0, 0], [[0, 0, 0], [0, 0], [1, 1, 1, Infinity]]]));
    named('Mesh.fromVertices: vertex 0: uv', () => Mesh.fromVertices([[[0, 0, 0], new Float64Array([0, 0])]]));
    named('Mesh.addVertex: vertex: color', () => new Mesh().addVertex([[0, 0, 0], [0, 0], new Uint8Array(4)]));
    new Mesh().addVertex([new Float32Array([0, 0, 0]), [0, 1]]);
    new Mesh().addVertex([[0, 0, 0], new Float32Array([0, 1]), new Float32Array([1, 1, 1, 1])]);
    // A matrix takes the float vector forms too, flat or as four columns.
    const identity = [[1, 0, 0, 0], [0, 1, 0, 0], [0, 0, 1, 0], [0, 0, 0, 1]];
    named('Model.setTransform', () => model.setTransform(new Float64Array(16)));
    named('Model.setTransform: column 2', () => model.setTransform([identity[0], identity[1], new Int32Array(4), identity[3]]));
    named('Model.setTransform', () => model.setTransform(new Array(15).fill(0)));
    named('Model.setTransform', () => model.setTransform(identity.slice(0, 3)));
    named('Model.setTransform: column 1', () => model.setTransform([identity[0], [0, 1, 0], identity[2], identity[3]]));
    model.setTransform(new Float32Array([1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1]));
    model.setTransform(identity.map((column) => new Float32Array(column)));
    model.setTransform(identity);
    if (!(new fc.Quad(new Float32Array([0, 0]), [1, 1]) instanceof fc.Quad)) fail('Quad took no Float32Array');
    if (!(new Vertex(0.5) instanceof Vertex)) fail('Vertex took no single number');
    if (!(Light.point([0, 1, 0], new Float32Array([1, 1, 1])) instanceof Light)) fail('Light.point took no Float32Array');

    // A region or viewport with a width or height of 0 throws an InvalidInputError, as Python
    // raises, and an argument of the wrong type throws a TypeError, as Python raises one.
    throws('new ScreenRegion([0, 0], [0, 4])', () => new fc.ScreenRegion([0, 0], [0, 4]));
    for (const [label, run, name] of [
      ['new ScreenRegion([0, 0], [0, 4])', () => new fc.ScreenRegion([0, 0], [0, 4]), 'InvalidInputError'],
      ["Pass.add('nope')", () => first.add('nope'), 'TypeError'],
    ]) {
      try { run(); } catch (e) {
        if (e.name !== name) fail(`${label} threw ${e.name}, not ${name}`);
        if (name === 'TypeError' && !(e instanceof TypeError)) fail(`${label} threw no TypeError`);
        continue;
      }
      fail(`${label} should throw`);
    }
    throws('Pass.setViewport([0, 0, 0, 4])', () => first.setViewport([0, 0, 0, 4]));
    throws('Pass.setViewport([4, 0])', () => first.setViewport([4, 0]));
    throws('Pass.setViewport({ minX: 2, minY: 2, maxX: 2, maxY: 6 })',
      () => first.setViewport({ minX: 2, minY: 2, maxX: 2, maxY: 6 }));
    // A region whose max lies before its min throws, never swapped back.
    throws('Pass.setViewport({ minX: 8, minY: 0, maxX: 2, maxY: 4 })',
      () => first.setViewport({ minX: 8, minY: 0, maxX: 2, maxY: 4 }));
    const region = new fc.ScreenRegion([0, 0], [64, 64]);
    region.max_x = -1;
    if (region.maxX !== 64) fail(`ScreenRegion.max_x wrote ${region.maxX} past the checks`);

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

    // A clear color takes numbers as given, as in every other language, and refuses anything else.
    throws('Pass.setClearColor([undefined, 0, 0])', () => first.setClearColor([undefined, 0, 0]));
    throws("Pass.setClearColor(['1', 0, 0])", () => first.setClearColor(['1', 0, 0]));
    throws('Pass.setClearColor([NaN, 0, 0])', () => first.setClearColor([NaN, 0, 0]));
    throws("Pass.setClearColor({ r: 1, g: 0, b: 0 })", () => first.setClearColor({ r: 1, g: 0, b: 0 }));
    throws("Pass.setClearColor('#ff8800')", () => first.setClearColor('#ff8800'));
    throws('Pass.setClearColor([0, 0])', () => first.setClearColor([0, 0]));
    const clear = Pass.fromShader('clear', Shader.default());
    clear.setViewport([0, 0, 1, 1]);
    clear.setClearColor([1.2, 0.5, 0.25]);
    renderer.render(clear, target);
    const corner = Array.from((await target.getImage()).slice(252, 255));
    if (Math.max(...corner) !== 255 || Math.min(...corner) < 50) fail(`[1.2, 0.5, 0.25] cleared to ${corner}`);

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
    // Every path that takes an ImageData reads it as RGBA, as createTexture does.
    bgra.write(red);
    same('Texture.write(ImageData) into Bgra8Unorm', await renderer.readTexture(bgra.id()), [0, 0, 255, 255]);
    bgra.writeRegion(red, [0, 0, 1, 1]);
    same('writeRegion(ImageData) into Bgra8Unorm', await renderer.readTexture(bgra.id()), [0, 0, 255, 255]);
    await rejects('createTexture(ImageData) with a fractional size', () => renderer.createTexture(image, [1.5, 2]));
    same('Mipmap.build(ImageData) as Bgra8Unorm', Mipmap.build(red, TextureFormat.Bgra8Unorm).level(0), [0, 0, 255, 255]);
    const seeded = await renderer.createStorageTexture([1, 1], TextureFormat.Bgra8Unorm, red,
      TextureUsage.TEXTURE_BINDING | TextureUsage.COPY_DST | TextureUsage.COPY_SRC);
    same('createStorageTexture seeded with an ImageData', await renderer.readTexture(seeded.id()), [0, 0, 255, 255]);
    // A region object needs its extent, and its bounds in order.
    const canvasTexture = await renderer.createStorageTexture([16, 16], TextureFormat.Rgba);
    const patch = new Uint8Array(4 * 4 * 4);
    canvasTexture.writeRegion(patch, { minX: 4, maxX: 8, minY: 0, maxY: 4 });
    canvasTexture.writeRegion(patch, new fc.ScreenRegion([4, 0], [4, 4]));
    throws('writeRegion with reversed bounds', () =>
      canvasTexture.writeRegion(patch, { minX: 8, maxX: 4, minY: 0, maxY: 4 }));
    throws('writeRegion with misspelled keys', () => canvasTexture.writeRegion(patch, { x: 0, y: 0, w: 4, h: 4 }));
    // 4 rows of 16 texels, the bytes a 16x4 region holds, so only the zero width is wrong.
    throws('writeRegion with a zero width', () => canvasTexture.writeRegion(new Uint8Array(16 * 4 * 4), [0, 0, 0, 4]));
    // The same 64 bytes as the 4x4 region, in another shape.
    const wide = new ImageData(new Uint8ClampedArray(8 * 2 * 4), 8, 2);
    throws('writeRegion with an ImageData of another shape', () => canvasTexture.writeRegion(wide, [0, 0, 4, 4]));
    throws('writeRegion with a function for x', () =>
      canvasTexture.writeRegion(patch, { x: () => 4, y: 0, width: 4, height: 4 }));
    throws('writeRegion with a function stride', () =>
      canvasTexture.writeRegion(patch, { x: 0, y: 0, width: 4, height: 4, bytesPerRow: () => 16 }));
    canvasTexture.writeRegion(new Uint8Array(16 * 16 * 4), undefined);
    throws('writeRegion with minZ 2**32 - 1', () =>
      canvasTexture.writeRegion(patch, { minX: 0, maxX: 4, minY: 0, maxY: 4, minZ: 4294967295 }));
    throws('setSamplerOptions with compare 1.5', () => canvasTexture.setSamplerOptions({ compare: 1.5 }));
    // An options object keeps its size under `size`, so a width beside format is an error.
    await rejects('createTexture options with width and format', () =>
      renderer.createTexture(new Uint8Array(4), { width: 1, height: 1, format: TextureFormat.Rgba8Unorm }));

    // Material texture setters take what createTexture takes, and raw pixels reach the upload.
    Material.pbr().baseColorTexture(fromImage);
    const glow = async (setTexture) => {
      // A black surface lit only by its emissive map, over the whole target.
      const surface = new Mesh();
      surface.addVertices([[-1, -1, 0], [3, -1, 0], [-1, 3, 0]]
        .map((corner) => Vertex.pbr(corner).set(Vertex.NORMAL, [0, 0, 1])));
      const material = Material.pbr().baseColor([0, 0, 0, 1]).emissive([1, 1, 1]);
      const glowing = new Pass('glow');
      glowing.add(new Model(surface, setTexture(material)));
      const frame = await renderer.createTextureTarget([4, 4]);
      await renderer.load(glowing);
      renderer.render(glowing, frame);
      return Array.from((await frame.getImage()).slice(0, 4));
    };
    const redPixel = [255, 0, 0, 255];
    const forms = {
      'a bare size': (m) => m.emissiveTexture(new Uint8Array(redPixel), [1, 1]),
      '{ size }': (m) => m.emissiveTexture(redPixel, { size: [1, 1] }),
      '{ size, format: R8Unorm }': (m) =>
        m.emissiveTexture(new Uint8Array([255]), { size: [1, 1], format: TextureFormat.R8Unorm }),
      'an ImageData': (m) => m.emissiveTexture(new ImageData(new Uint8ClampedArray(redPixel), 1, 1)),
      'every option': (m) => m.emissiveTexture(new Uint8Array(redPixel), {
        size: [1, 1], format: TextureFormat.Rgba, mipmaps: false, sampler: { repeatX: true }, usage: TextureUsage.COPY_SRC,
      }),
    };
    for (const [label, form] of Object.entries(forms)) {
      const [r, g, b] = await glow(form);
      if (r < 200 || g > 40 || b > 40) fail(`emissiveTexture with ${label} drew [${r}, ${g}, ${b}]`);
    }
    for (const name of ['baseColorTexture', 'metallicRoughnessTexture', 'normalTexture', 'occlusionTexture', 'emissiveTexture']) {
      const material = Material.pbr();
      material[name](new Uint8Array(redPixel), { size: [1, 1], format: TextureFormat.Rgba8Unorm, mipmaps: false });
      throws(`Material.${name} with a fractional size`, () => material[name](new Uint8Array(redPixel), [1.5, 1]));
      throws(`Material.${name} with mipmaps: "yes"`, () => material[name](new Uint8Array(redPixel), { size: [1, 1], mipmaps: 'yes' }));
    }

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

    // Enum and index arguments are checked, where undefined, NaN, strings and fractions read as a code.
    throws('Mipmap.level(1.5)', () => chain.level(1.5));
    throws('Mipmap.level(NaN)', () => chain.level(NaN));
    throws('Mipmap.build without a format', () => Mipmap.build(new Uint8Array(4 * 4 * 4), undefined, [4, 4]));
    throws('Material.alphaMode(undefined)', () => Material.pbr().alphaMode(undefined));
    throws('Material.alphaMode(0.5)', () => Material.pbr().alphaMode(0.5));
    throws('Material.textureUv(undefined, 0)', () => Material.pbr().textureUv(undefined, 0));
    throws('Material.textureTransform(5, ...)', () =>
      Material.pbr().textureTransform(5, [0, 0], [1, 1], 0));
    await rejects('createStorageTexture with a format string', () =>
      renderer.createStorageTexture([4, 4], 'rgba8unorm'));
    await rejects('createStorageTexture with usage 32', () =>
      renderer.createStorageTexture([4, 4], TextureFormat.Rgba, null, 32));
});

// The checks render runs before any GPU work throw from the render call that broke
// the rule, where WebGPU would report the error after render returned. A readback
// right after finds no late error, and a late one would fail the run through console.error.
await withModule('platforms.web.healthcheck.pass_checks', async () => {
    const fail = (message) => { throw new Error(message); };
    const refuses = (label, words, run) => {
      try { run(); } catch (e) {
        if (!(e instanceof Error)) fail(`${label} threw a ${typeof e}, not an Error`);
        if (!e.message.includes(words)) fail(`${label} threw: ${e.message}`);
        return console.log(`${label} threw:`, e.message);
      }
      fail(`${label} should throw`);
    };
    const solid = () => new Shader(`
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>, 3>(vec2f(-1.0, -1.0), vec2f(3.0, -1.0), vec2f(-1.0, 3.0));
  return vec4<f32>(p[i], 0.0, 1.0);
}
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0, 0.0, 0.0, 1.0); }
`);
    const renderer = new Renderer();
    const target = await renderer.createTextureTarget([8, 8]);

    const feedback = new Shader(`
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var soft: sampler;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>, 3>(vec2f(-1.0, -1.0), vec2f(3.0, -1.0), vec2f(-1.0, 3.0));
  return vec4<f32>(p[i], 0.0, 1.0);
}
@fragment fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
  return textureSample(source, soft, pos.xy / 8.0);
}
`);
    feedback.set('source', target.texture());
    refuses('sampling the render target', 'Texture conflict', () => renderer.render(feedback, target));

    const depth = await renderer.createDepthTexture([4, 4]);
    const deep = new Pass('depth');
    deep.addShader(solid());
    deep.setDepthTarget(depth);
    refuses('a depth target of another size', 'Depth size mismatch', () => renderer.render(deep, target));

    const foreign = await new Renderer().createTextureTarget([8, 8]);
    refuses('a target from another Renderer', 'another Renderer', () => renderer.render(solid(), foreign));

    // WebGL2 runs no compute shaders, so only WebGPU checks the dispatch.
    const adapter = navigator.gpu ? await navigator.gpu.requestAdapter() : null;
    if (adapter) {
      const compute = Pass.compute('big');
      compute.addShader(new Shader('@compute @workgroup_size(1)\nfn cs_main() {}\n'));
      compute.setComputeDispatch(4294967295, 1, 1);
      refuses('a dispatch past the device limit', 'dispatches', () => renderer.render(compute, target));
    }

    refuses('a Renderer with no context', 'another Renderer', () => new Renderer().render(solid(), target));

    // Blending reads the source alpha, which the browser requires the output to have:
    // a one-channel output blended into R8Unorm throws from render itself.
    const usage = TextureUsage.RENDER_ATTACHMENT | TextureUsage.TEXTURE_BINDING;
    const red = await renderer.createStorageTexture([8, 8], TextureFormat.R8Unorm, null, usage);
    const single = new Shader(`
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>, 3>(vec2f(-1.0, -1.0), vec2f(3.0, -1.0), vec2f(-1.0, 3.0));
  return vec4<f32>(p[i], 0.0, 1.0);
}
@fragment fn fs_main() -> @location(0) f32 { return 0.5; }
`);
    Material.custom(single).alphaMode(fc.AlphaMode.Blend);
    const blend = Pass.fromShader('blend', single);
    blend.setTarget(red);
    refuses('a one-channel output blended', 'reads an alpha channel', () => renderer.render(blend, target));

    // A shader that writes no color draws with an empty write mask, which the browser
    // requires, so it renders without a late error and keeps the clear color.
    const silent = new Shader(`
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>, 3>(vec2f(-1.0, -1.0), vec2f(3.0, -1.0), vec2f(-1.0, 3.0));
  return vec4<f32>(p[i], 0.0, 1.0);
}
@fragment fn fs_main() {}
`);
    const quiet = Pass.fromShader('no color', silent);
    quiet.setClearColor([1.0, 0.0, 0.0, 1.0]);
    renderer.render(quiet, target);
    const kept = await target.getImage();
    if (kept[0] !== 255 || kept[1] !== 0) fail(`a shader without color drew over the clear: ${kept.slice(0, 4)}`);

    renderer.render(solid(), target);
    await target.getImage();
});

// One shader draws meshes with other vertex and instance layouts, each with a pipeline of
// its own that shares the shader's bind groups, so the browser takes them with no late error,
// and a second Renderer draws the same meshes. The mesh checks throw from the call that breaks them.
await withModule('platforms.web.healthcheck.mesh_checks', async () => {
    const fail = (message) => { throw new Error(message); };
    const refuses = (label, words, run) => {
      try { run(); } catch (e) {
        if (!(e instanceof Error)) fail(`${label} threw a ${typeof e}, not an Error`);
        if (!e.message.includes(words)) fail(`${label} threw: ${e.message}`);
        return console.log(`${label} threw:`, e.message);
      }
      fail(`${label} should throw`);
    };
    const source = `
struct VOut { @builtin(position) pos: vec4<f32>, @location(0) color: vec4<f32> };
@group(0) @binding(0) var<uniform> gain: f32;
@vertex fn vs_main(@location(0) position: vec2<f32>, @location(1) color: vec4<f32>) -> VOut {
  return VOut(vec4<f32>(position, 0.0, 1.0), color * gain);
}
@fragment fn fs_main(v: VOut) -> @location(0) vec4<f32> { return v.color; }
`;
    const column = (left, right) => [[left, -1], [right, -1], [right, 1], [left, -1], [right, 1], [left, 1]];
    const shader = new Shader(source);
    shader.set('gain', 1.0);
    const painted = new Mesh();
    painted.addVertices(column(-1, 0).map((corner) => new Vertex(corner).set('color', [1, 0, 0, 1])));
    const tinted = new Mesh();
    tinted.addVertices(column(0, 1).map((corner) => new Vertex(corner)));
    tinted.addInstance(new Instance().set('color', [0, 0, 1, 1]));
    shader.addMesh(painted);
    shader.addMesh(tinted);

    const draw = async (renderer) => {
      const target = await renderer.createTextureTarget([8, 2]);
      renderer.render(shader, target);
      const image = await target.getImage();
      return (x) => Array.from(image.slice(x * 4, x * 4 + 4)).join(',');
    };
    const renderer = new Renderer();
    for (const [label, pixel] of [['one Renderer', await draw(renderer)], ['a second Renderer', await draw(new Renderer())]]) {
      if (pixel(1) !== '255,0,0,255') fail(`${label}: the vertex color column drew ${pixel(1)}`);
      if (pixel(6) !== '0,0,255,255') fail(`${label}: the instance color column drew ${pixel(6)}`);
    }

    const target = await renderer.createTextureTarget([8, 2]);
    painted.setIndices([0, 1, 7]);
    refuses('an index past the last vertex', 'points past the last of 6 vertices', () => renderer.render(shader, target));
    painted.setIndices([0, 1, 2, 0]);
    refuses('a partial triangle', 'multiple of 3 indices', () => renderer.render(shader, target));
    painted.clearIndices();
    tinted.addInstance(new Instance());
    refuses('an instance without the color the shader reads', 'got no value at instance 1', () => renderer.render(shader, target));
    tinted.clearInstances();
    tinted.addInstance(new Instance().set('color', [0, 0, 1, 1]));
    const named = new Mesh();
    named.addVertices([[-1, -1], [1, -1], [-1, 1]].map((corner) => new Vertex(corner).set('position', [5, 5]).set('color', [1, 1, 1, 1])));
    refuses('a position property', "property named 'position'", () => new Shader(source).addMesh(named));
    refuses('Shader.fromMesh of an empty mesh', 'Mesh has no vertices', () => Shader.fromMesh(new Mesh()));
    refuses('Shader.fromMesh of a mesh its shader refuses', "property named 'position'", () => Shader.fromMesh(named));

    // No device takes 65 vertex attributes, so an input at @location(64) is past every limit.
    const far = new Shader(`
@vertex fn vs_main(@location(0) position: vec2<f32>, @location(64) extra: f32) -> @builtin(position) vec4<f32> { return vec4<f32>(position * extra, 0.0, 1.0); }
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
`);
    const reaching = new Mesh();
    reaching.addVertices([[-1, -1], [3, -1], [-1, 3]].map((corner) => new Vertex(corner).set('extra', 1)));
    far.addMesh(reaching);
    let past = null;
    try { renderer.render(far, target); } catch (e) { past = e; }
    if (!(past instanceof Error) || past.limit !== 'max_vertex_attributes' || past.value !== 65) {
      fail(`a vertex input past the attribute limit threw ${past?.message ?? 'nothing'}`);
    }

    renderer.render(shader, target);
    await target.getImage();
});

// One Material map draws with every Renderer that awaits load, each from a copy of its own.
// A set right after a load starts wins, a custom slot asks for load before it, and
// a freed Renderer leaves the others drawing.
await withModule('platforms.web.healthcheck.material_maps', async () => {
    const fail = (message) => { throw new Error(message); };
    const mapped = () => Material.custom(new Shader(`
@group(0) @binding(0) var base_color_map: texture_2d<f32>;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>, 3>(vec2f(-1.0, -1.0), vec2f(3.0, -1.0), vec2f(-1.0, 3.0));
  return vec4<f32>(p[i], 0.0, 1.0);
}
@fragment fn fs_main() -> @location(0) vec4<f32> { return textureLoad(base_color_map, vec2<i32>(0), 0); }
`));
    const open = async () => {
      const renderer = new Renderer();
      return [renderer, await renderer.createTextureTarget([4, 4])];
    };
    const reads = async (label, target, expected) => {
      const pixel = Array.from((await target.getImage()).slice(0, 4));
      if (pixel.join() !== expected.join()) fail(`${label} drew [${pixel}], not [${expected}]`);
    };
    const red = [255, 0, 0, 255];
    const blue = [0, 0, 255, 255];

    const material = mapped().baseColorTexture(new Uint8Array(red), [1, 1]);
    const shader = material.shader();
    const [first, firstTarget] = await open();
    const [second, secondTarget] = await open();
    await first.load(shader);
    await second.load(shader);
    first.render(shader, firstTarget);
    second.render(shader, secondTarget);
    await reads('the first Renderer', firstTarget, red);
    await reads('the second Renderer', secondTarget, red);

    const loading = first.load(shader);
    material.baseColorTexture(new Uint8Array(blue), [1, 1]);
    await loading;
    await second.load(shader);
    first.render(shader, firstTarget);
    second.render(shader, secondTarget);
    await reads('the first Renderer after a set during its load', firstTarget, blue);
    await reads('the second Renderer after the set', secondTarget, blue);

    const [third, thirdTarget] = await open();
    let early = null;
    try { third.render(shader, thirdTarget); } catch (e) { early = e; }
    if (!(early instanceof Error) || !early.message.includes('load')) {
      fail(`a custom map before load threw ${early?.message ?? 'nothing'}`);
    }

    first.free();
    firstTarget.free();
    second.render(shader, secondTarget);
    await reads('the second Renderer after the first was freed', secondTarget, blue);
});

// A shader the device can't run, or a texture its slot can't take, throws from render
// itself, before any GPU work, so no late error or console.error follows.
await withModule('platforms.web.healthcheck.device_checks', async () => {
    const renderer = new Renderer();
    const target = await renderer.createTextureTarget([8, 8]);
    const fail = (message) => { throw new Error(message); };
    // Checks that `run` throws an Error whose message holds `text`, and returns it.
    const refuses = (label, text, run) => {
      try { run(); } catch (e) {
        if (!(e instanceof Error)) fail(`${label} threw a ${typeof e}, not an Error`);
        if (!text.test(e.message)) fail(`${label} threw: ${e.message}`);
        console.log(`${label} threw:`, e.message);
        return e;
      }
      fail(`${label} should throw`);
    };
    const vertex = `
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>, 3>(vec2f(-1.0, -1.0), vec2f(3.0, -1.0), vec2f(-1.0, 3.0));
  return vec4<f32>(p[i], 0.0, 1.0);
}`;

    const sampled = new Shader(`${vertex}
@group(0) @binding(0) var image: texture_2d<f32>;
@group(0) @binding(1) var linear: sampler;
@fragment fn fs_main() -> @location(0) vec4<f32> { return textureSample(image, linear, vec2<f32>(0.5)); }`);
    const unset = refuses('render with an unset texture', /'image' takes a texture, not none/, () => renderer.render(sampled, target));
    // A typed refusal carries its fields as properties, and its name is the class Python raises.
    if (unset.uniform !== 'image' || unset.expected !== 'a texture' || unset.found !== 'none, since it was never set' || unset.name !== 'BindingError') {
      fail(`an unset texture threw uniform ${unset.uniform}, expected ${unset.expected}, found ${unset.found}, name ${unset.name}`);
    }
    const depth = await renderer.createDepthTexture([8, 8]);
    sampled.set('image', depth);
    refuses('render with a depth texture in a texture_2d<f32> slot', /'image' takes a texture that samples as Float \{ filterable: true \}, not Depth32Float/,
      () => renderer.render(sampled, target));
    const plain = await renderer.createTexture(new Uint8Array(4 * 4 * 4), { size: [4, 4] });
    sampled.set('image', plain);
    renderer.unregisterTexture(plain.id());
    refuses('render with a texture unregisterTexture removed', /not found/, () => renderer.render(sampled, target));

    const marks = new Shader(`${vertex}
@group(0) @binding(0) var marks: texture_storage_2d<rgba8unorm, write>;
@fragment fn fs_main() -> @location(0) vec4<f32> {
  textureStore(marks, vec2<u32>(0u), vec4<f32>(1.0));
  return vec4<f32>(1.0);
}`);
    const sampledOnly = await renderer.createTexture(new Uint8Array(4 * 4 * 4), { size: [4, 4] });
    marks.set('marks', sampledOnly);
    // WebGL2 writes no storage from the fragment stage, and the shader check refuses that before any texture check.
    refuses('render with a sampled texture in a storage slot',
      /takes a texture with STORAGE_BINDING usage|'marks' at @group\(0\) @binding\(0\) needs FRAGMENT_WRITABLE_STORAGE/,
      () => renderer.render(marks, target));

    const far = new Shader(`${vertex}
@group(100) @binding(0) var<uniform> tint: vec4<f32>;
@fragment fn fs_main() -> @location(0) vec4<f32> { return tint; }`);
    refuses('render with a group past the device', /needs max_bind_groups of at least 101/,
      () => renderer.render(far, target));
    try { renderer.render(far, target); } catch (e) {
      if (e.name !== 'LimitError' || e.limit !== 'max_bind_groups' || e.value !== 101 || !(e.max < 101) || !e.item.startsWith("'tint'")) {
        fail(`a group past the device threw item ${e.item}, limit ${e.limit}, value ${e.value}, max ${e.max}`);
      }
    }
    const lone = new Shader(`@fragment fn main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }`);
    const entry = refuses('render without a vertex entry point', /no @vertex entry point/, () => renderer.render(lone, target));
    if (entry.stage !== '@vertex' || entry.name !== 'EntryError') fail(`a shader without a vertex stage threw stage ${entry.stage}, name ${entry.name}`);
    const storage = new Shader(`@group(0) @binding(0) var img: texture_storage_2d<bgra8unorm, write>;
@compute @workgroup_size(1) fn cs_main() { textureStore(img, vec2<i32>(0), vec4<f32>(1.0)); }`);
    // Storage slots are checked before the compute stage, so WebGL2 refuses the format too.
    const feature = refuses('render with a storage format the device lacks',
      /'img' at @group\(0\) @binding\(0\) needs WriteOnly storage access to Bgra8Unorm/,
      () => renderer.render(storage, target));
    if (feature.message !== `${feature.item} needs ${feature.feature}, which the device lacks` || feature.name !== 'FeatureError') {
      fail(`a storage format the device lacks threw item ${feature.item}, feature ${feature.feature}, name ${feature.name}`);
    }
    const wide = new Shader(`@compute @workgroup_size(4096) fn cs_main() {}`);
    refuses('dispatch of a workgroup past the device', /max_compute_workgroup_size_x|needs COMPUTE_SHADERS/,
      () => renderer.render(wide, target));
    const integer = new Shader(`${vertex}
@fragment fn fs_main() -> @location(0) vec4<u32> { return vec4<u32>(1u); }`);
    refuses('render of integer output into a float target', /writes vec4<u32> at @location\(0\)/,
      () => renderer.render(integer, target));

    // Each refusal ran no GPU work, so the next frame draws and reads back with no late error.
    renderer.render(Shader.default(), target);
    const image = await target.getImage();
    if (!(image instanceof Uint8Array) || image.length !== 8 * 8 * 4) fail(`getImage returned ${image}`);
});

// The WGSL wgpu writes for the browser drops @size and @align, so FragmentColor hands it
// WGSL that keeps them: each member reads and writes the bytes its explicit layout gives it.
await withModule('platforms.web.healthcheck.explicit_layouts', async () => {
    if (!navigator.gpu || !(await navigator.gpu.requestAdapter())) {
      return console.log('explicit layouts: no WebGPU adapter, and WebGL2 refuses them, as webgl2.js checks');
    }
    const renderer = new Renderer();
    const target = await renderer.createTextureTarget([8, 8]);
    const shader = new Shader(`
struct Params { scale: f32, @align(16) offset: vec2<f32>, @size(20) tag: f32, last: f32 };
@group(0) @binding(0) var<uniform> params: Params;
struct Out { a: f32, @align(16) b: vec2<f32>, c: f32 };
@group(0) @binding(1) var<storage, read_write> explicit_layout_out: Out;
@compute @workgroup_size(1) fn cs_main() {
  explicit_layout_out.a = params.scale;
  explicit_layout_out.b = params.offset;
  explicit_layout_out.c = params.last;
}`);
    shader.set('params.scale', 2);
    shader.set('params.offset', [3, 4]);
    shader.set('params.last', 5);

    renderer.render(shader, target);
    const bytes = await renderer.readStorage(shader, 'explicit_layout_out');

    const floats = new Float32Array(bytes.buffer, bytes.byteOffset, bytes.byteLength / 4);
    // `a` at byte 0, `b` at byte 16 after its @align(16), and `c` right after `b`, at byte 24.
    const read = [floats[0], floats[4], floats[5], floats[6]];
    const want = [2, 3, 4, 5];
    if (read.some((value, index) => value !== want[index])) {
      throw new Error(`explicit layouts read [${read}], want [${want}]`);
    }
});

// WebGPU reports a GPU error after render returns. Each one logs a console.error as it
// lands, and the next render throws it, once. A readback or load waits for it and throws it.
await withModule('platforms.web.healthcheck.late_errors', async () => {
    const renderer = new Renderer();
    const target = await renderer.createTextureTarget([8, 8]);
    const fail = (message) => { throw new Error(message); };
    const pause = () => new Promise((resolve) => setTimeout(resolve, 10));
    // Checks that `run` throws the frame's error as an Error, with the text render throws
    // and the error's kind and label as properties.
    const throwsLate = async (label, run) => {
      try { await run(); } catch (e) {
        if (!(e instanceof Error)) fail(`${label} threw a ${typeof e}, not an Error`);
        if (!e.message.startsWith('Device error (validation)')) fail(`${label} threw: ${e.message}`);
        if (e.kind !== ErrorKind.Validation) fail(`${label} threw kind ${e.kind}`);
        if (typeof e.label !== 'string' || !e.message.includes(`'${e.label}'`)) {
          fail(`${label} threw label ${e.label} for: ${e.message}`);
        }
        if (typeof e.detail !== 'string' || !e.detail || !e.message.endsWith(`: ${e.detail}`)) {
          fail(`${label} threw detail ${e.detail} for: ${e.message}`);
        }
        return console.log(`${label} threw:`, e.message);
      }
      fail(`${label} never reported the failed frame`);
    };
    // Runs `run`, which must not throw, and names the step when it does.
    const succeeds = async (label, run) => {
      try { return await run(); } catch (e) { fail(`${label} threw: ${e?.message ?? e}`); }
    };
    // Calls `run` until the error lands and it throws, then checks what it threw.
    const surfaces = async (label, run) => {
      for (let attempt = 0; attempt < 200; attempt++) {
        await pause();
        try { await run(); } catch (e) {
          return throwsLate(label, () => { throw e; });
        }
      }
      fail(`${label} never reported the failed frame`);
    };

    // A late error fails the run anywhere else, through its console.error.
    // This module expects them, so it collects them instead.
    const logged = [];
    const consoleError = console.error;
    console.error = (...args) => {
      const text = args.map(String).join(' ');
      if (text.includes('Device error (')) logged.push(text);
      else consoleError.apply(console, args);
    };
    // Waits until a new error has logged, and then until no more arrive.
    const settleLog = async (label, count) => {
      for (let attempt = 0; attempt < 200 && logged.length === count; attempt++) await pause();
      if (logged.length === count) fail(`${label} logged nothing`);
      let seen = -1;
      while (seen !== logged.length) { seen = logged.length; for (let i = 0; i < 10; i++) await pause(); }
    };
    const pipelineErrors = () => logged.filter((text) => text.includes("'render pipeline'")).length;

    try {
      // The fragment stage reads @location(0), which the vertex stage never writes,
      // so the shader parses and only its pipeline fails.
      const unfed = new Shader(`
struct VOut { @builtin(position) pos: vec4<f32> };
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
  let p = array<vec2<f32>, 3>(vec2f(-1.0, -1.0), vec2f(3.0, -1.0), vec2f(-1.0, 3.0));
  return VOut(vec4<f32>(p[i], 0.0, 1.0));
}
@fragment fn fs_main(@location(0) tint: vec4<f32>) -> @location(0) vec4<f32> { return tint; }
`);
      const valid = Shader.default();
      const storage = new Shader(`
struct Data { values: array<f32, 4> };
@group(0) @binding(0) var<storage, read_write> data: Data;
@compute @workgroup_size(1) fn cs_main() { data.values[0] = 7.0; }
`);

      renderer.render(unfed, target);
      await surfaces('render after a failed frame', () => renderer.render(valid, target));
      await succeeds('render after the error was thrown', () => renderer.render(valid, target));

      // A readback or load in the same task as the failed frame waits for its error.
      renderer.render(unfed, target);
      await throwsLate('getImage right after a failed frame', () => target.getImage());
      await succeeds('getImage after the error was thrown', () => target.getImage());

      renderer.render(unfed, target);
      await throwsLate('readTexture right after a failed frame', () => renderer.readTexture(target.texture().id()));
      await succeeds('readTexture after the error was thrown', () => renderer.readTexture(target.texture().id()));

      // A call that waits throws only the error of a frame rendered before it began.
      // A frame rendered right after the call, in the same task, keeps its error for the next call.
      const keepsLaterErrors = async (label, call) => {
        await succeeds(`render before ${label}`, () => renderer.render(valid, target));
        const waiting = call();
        renderer.render(unfed, target);
        const result = await succeeds(`${label} during a failed frame`, () => waiting);
        await throwsLate(`getImage after a frame that failed during ${label}`, () => target.getImage());
        return result;
      };
      const bytes = await keepsLaterErrors('readTexture', () => renderer.readTexture(target.texture().id()));
      if (!(bytes instanceof Uint8Array) || bytes.length !== 8 * 8 * 4) {
        fail(`readTexture during a failed frame returned ${bytes}`);
      }
      const image = await keepsLaterErrors('getImage', () => target.getImage());
      if (!(image instanceof Uint8Array) || image.length !== 8 * 8 * 4) {
        fail(`getImage during a failed frame returned ${image}`);
      }
      await keepsLaterErrors('load', () => renderer.load(valid));

      await succeeds('render of a storage shader', () => renderer.render(storage, target));
      await succeeds('readStorage', () => renderer.readStorage(storage, 'data'));
      await keepsLaterErrors('readStorage', () => renderer.readStorage(storage, 'data'));
      renderer.render(unfed, target);
      await throwsLate('readStorage right after a failed frame', () => renderer.readStorage(storage, 'data'));
      await succeeds('readStorage after the error was thrown', () => renderer.readStorage(storage, 'data'));

      renderer.render(unfed, target);
      await throwsLate('load right after a failed frame', () => renderer.load(valid));
      await succeeds('load after the error was thrown', () => renderer.load(valid));

      // A pipeline whose error landed left the cache, so the next
      // failed frame builds it again and logs its cause again.
      const reported = pipelineErrors();
      renderer.render(unfed, target);
      await surfaces('getImage after the pipeline failed again', () => target.getImage());
      if (pipelineErrors() !== reported + 1) fail('a failed pipeline stayed in the cache');

      // A frame no later call follows still reports its error, through the console.
      const lone = new Renderer();
      const loneTarget = await lone.createTextureTarget([8, 8]);
      const count = logged.length;
      lone.render(unfed, loneTarget);
      await settleLog('a failed frame that no call follows', count);
    } finally {
      console.error = consoleError;
    }
});

// A wasm frame opens error scopes only for the frame and for each pipeline it builds.
// Its texture views and bind groups, placeholders included, run under the frame's scopes.
await withModule('platforms.web.healthcheck.error_scopes', async () => {
    const device = globalThis.GPUDevice;
    if (!device) return console.log('error_scopes: no WebGPU, skipped');
    const push = device.prototype.pushErrorScope;
    let pushed = 0;
    device.prototype.pushErrorScope = function (filter) {
      pushed++;
      return push.call(this, filter);
    };
    try {
      const renderer = new Renderer();
      const target = await renderer.createTextureTarget([8, 8]);
      // Group 0 holds nothing, so the pipeline binds an empty placeholder group there.
      const shader = new Shader(`
@group(1) @binding(0) var image: texture_2d<f32>;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>, 3>(vec2f(-1.0, -1.0), vec2f(3.0, -1.0), vec2f(-1.0, 3.0));
  return vec4<f32>(p[i], 0.0, 1.0);
}
@fragment fn fs_main() -> @location(0) vec4<f32> { return textureLoad(image, vec2<i32>(0, 0), 0); }
`);
      const texture = await renderer.createTexture(new Uint8Array([255, 0, 0, 255]), { size: [1, 1] });
      shader.set('image', texture);

      renderer.render(shader, target);
      await target.getImage();
      if (pushed === 0) return console.log('error_scopes: the device opens no WebGPU scopes, skipped');
      pushed = 0;
      renderer.render(shader, target);
      const frame = pushed;
      await target.getImage();

      if (frame !== 3) throw new Error(`a frame with a cached pipeline opened ${frame} error scopes, not 3`);
    } finally {
      device.prototype.pushErrorScope = push;
    }
});

// WebGPU bins lights into clusters with a compute pass, so a PBR shader takes
// more lights than any uniform block holds: the largest block takes 1,021, so
// 1,100 lights fit only on the clustered path. They render with no GPU error,
// captured or not. The runner requires a WebGPU adapter on CI.
await withModule('platforms.web.healthcheck.lights.clustered', async () => {
    const adapter = navigator.gpu ? await navigator.gpu.requestAdapter() : null;
    if (!adapter) {
      console.log('clustered lights: no WebGPU adapter, so the WebGL2 pass covers lights');
      return;
    }
    const gpuErrors = [];
    const original = console.error;
    console.error = (...parts) => {
      const text = parts.map(String).join(' ');
      if (text.includes('GPU error')) gpuErrors.push(text);
      original.apply(console, parts);
    };
    try {
      const renderer = new Renderer();
      const target = await renderer.createTextureTarget([64, 64]);
      const floor = new Mesh();
      const corners = [[-6, -6], [-6, 6], [6, 6], [-6, -6], [6, 6], [6, -6]];
      for (const [x, z] of corners) {
        floor.addVertex(Vertex.pbr([x, 0.0, z]).set('normal', [0.0, 1.0, 0.0]));
      }
      const pass = new Pass('1100 lights');
      pass.add(new Model(floor, Material.pbr()));
      pass.add(Camera.perspective(1.0, 1.0, 0.1, 100.0)
        .lookAt([0.0, 6.0, 8.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]));
      for (let index = 0; index < 1100; index++) {
        const x = (index % 44) * 0.26 - 5.6;
        const z = Math.floor(index / 44) * 0.46 - 5.5;
        const hue = index / 1100 * Math.PI * 2;
        const color = [0.5 + 0.5 * Math.cos(hue), 0.5 + 0.5 * Math.cos(hue + 2.1), 0.5 + 0.5 * Math.cos(hue + 4.2)];
        const light = Light.point([x, 0.3, z], color).setIntensity(0.15);
        light.setRange(1.0);
        pass.add(light);
      }
      renderer.render(pass, target);
      const image = await target.getImage();
      // A scope that catches an error resolves after render returns, so the
      // next render or readback throws it.
      await new Promise((resolve) => setTimeout(resolve, 50));
      renderer.render(pass, target);
      await target.getImage();
      let lit = 0;
      for (let pixel = 0; pixel < image.length; pixel += 4) {
        if (image[pixel] > 100 || image[pixel + 1] > 100 || image[pixel + 2] > 100) lit++;
      }
      if (lit < 64) throw new Error(`1100 lights lit only ${lit} pixels`);
      console.log(`clustered lights: 1100 lights lit ${lit} pixels`);
    } finally {
      console.error = original;
    }
    if (gpuErrors.length) {
      throw new Error(`1100 lights raised ${gpuErrors.length} uncaptured GPU error(s): ${gpuErrors[0]}`);
    }
});

// Auto-generated: run all extracted examples (after init)
await withModule('platforms.web.healthcheck.generated_examples', async () => {
  const mod = await import('./generated_examples.mjs');
  // The runner waits for each example's GPU errors to land before the next one starts.
  const { passed, failed } = await mod.runExamples();
  if (failed === 0) {
    console.log(`\n✅ test result: ok. ${passed} passed; ${failed} failed`);
  } else {
    throw new Error(`${failed} JS examples failed`);
  }
});
