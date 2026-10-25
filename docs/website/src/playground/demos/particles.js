// One million particles: a compute pass moves them and an instanced render
// pass draws them. Both shaders share one storage buffer by its name.

import { formatValue } from "../codegen.js";

const COMPUTE = /* wgsl */ `// Moves every particle one step. main.js declares COUNT, so this file and
// render.wgsl size the shared particles buffer the same way.

struct Particle {
  position: vec2<f32>,
  velocity: vec2<f32>,
}

struct Sim {
  resolution: vec2<f32>,
  pointer: vec2<f32>,
  time: f32,
  delta: f32,
  idle: f32,
  strength: f32, // 0..4
  swirl: f32, // -4..4
  drag: f32, // 0..4
  respawn: f32, // 0..0.5
  orbit: f32, // 0..1
}

@group(0) @binding(0) var<storage, read_write> particles: array<Particle, COUNT>;
@group(0) @binding(1) var<uniform> sim: Sim;

// PCG hash: a well mixed u32 for every input.
fn hash(value: u32) -> u32 {
  let state = value * 747796405u + 2891336453u;
  let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
  return (word >> 22u) ^ word;
}

fn random(seed: u32) -> f32 {
  return f32(hash(seed)) / 4294967295.0;
}

// A random spot on screen, already turning around the middle.
fn spawn(seed: u32, aspect: f32) -> Particle {
  let spot = (vec2<f32>(random(seed), random(seed + 1u)) * 2.0 - 1.0) * vec2<f32>(aspect, 1.0);
  let jitter = vec2<f32>(random(seed + 2u), random(seed + 3u)) - 0.5;
  return Particle(spot, vec2<f32>(-spot.y, spot.x) * 0.4 + jitter * 0.1);
}

@compute @workgroup_size(256)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
  let index = id.x;
  if (index >= COUNT) {
    return;
  }
  let aspect = sim.resolution.x / max(sim.resolution.y, 1.0);
  let seed = hash(index ^ hash(bitcast<u32>(sim.time)));
  var p = particles[index];

  // A new buffer is all zeros, so the first dispatch seeds it. After that each
  // particle respawns with probability respawn per second, as does any that flies far away.
  let fresh = all(p.velocity == vec2<f32>(0.0));
  let lost = any(abs(p.position) > vec2<f32>(aspect, 1.0) * 1.5);
  if (fresh || lost || random(seed) < sim.respawn * sim.delta) {
    particles[index] = spawn(seed + 1u, aspect);
    return;
  }

  // The pointer pulls while it moves. When it rests, the pull drifts onto an orbit.
  let mouse = sim.pointer / max(sim.resolution, vec2<f32>(1.0));
  let pointer = (vec2<f32>(mouse.x, 1.0 - mouse.y) * 2.0 - 1.0) * vec2<f32>(aspect, 1.0);
  let orbit = vec2<f32>(cos(sim.time * 0.37) * aspect, sin(sim.time * 0.61)) * sim.orbit;
  let center = mix(pointer, orbit, smoothstep(1.0, 3.0, sim.idle));

  // Pull toward the center plus a sideways swirl, both weaker with distance.
  let offset = center - p.position;
  let falloff = 1.0 / (dot(offset, offset) + 0.01);
  let force = (offset * sim.strength + vec2<f32>(-offset.y, offset.x) * sim.swirl) * falloff;
  p.velocity = (p.velocity + force * sim.delta) * exp(-sim.drag * sim.delta);
  p.position += p.velocity * sim.delta;
  particles[index] = p;
}
`;

const RENDER = /* wgsl */ `// Draws one small triangle per particle, colored by its speed. It reads the
// particles buffer that compute.wgsl writes, because both use that name.

struct Particle {
  position: vec2<f32>,
  velocity: vec2<f32>,
}

struct Look {
  resolution: vec2<f32>,
  size: f32, // 0.5..6
  top_speed: f32, // 0.1..4
  slow_color: vec3<f32>,
  fast_color: vec3<f32>,
}

@group(0) @binding(0) var<storage, read> particles: array<Particle, COUNT>;
@group(0) @binding(1) var<uniform> look: Look;

struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) color: vec3<f32>,
}

@vertex
fn vs_main(@location(0) corner: vec2<f32>, @builtin(instance_index) index: u32) -> VertexOutput {
  let particle = particles[index];
  let aspect = look.resolution.x / max(look.resolution.y, 1.0);
  let pixel = 2.0 / max(look.resolution, vec2<f32>(1.0));
  let heat = smoothstep(0.0, look.top_speed, length(particle.velocity));
  var out: VertexOutput;
  out.position = vec4<f32>(particle.position / vec2<f32>(aspect, 1.0) + corner * look.size * pixel, 0.0, 1.0);
  out.color = mix(look.slow_color, look.fast_color, heat);
  return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  return vec4<f32>(in.color, 1.0);
}
`;

const PROGRAM = /* js */ `import { Mesh, Pass, Shader } from "fragmentcolor";
import COMPUTE from "./compute.wgsl?raw";
import RENDER from "./render.wgsl?raw";

const LIBRARY = [];

// Both shaders size the shared particles buffer with this count.
const COUNT = 1_000_000;

export async function setup(renderer, target) {
  const header = \`const COUNT: u32 = \${COUNT}u;\`;

  const simulate = await prepare("simulate", [...LIBRARY, header, COMPUTE]);
  const compute = simulate.shader;
  compute.set("sim.strength", 1.2);
  compute.set("sim.swirl", 0.4);
  compute.set("sim.drag", 0.8);
  compute.set("sim.respawn", 0.03);
  compute.set("sim.orbit", 0.55);
  // 256 matches @workgroup_size in compute.wgsl.
  simulate.pass.setComputeDispatch(Math.ceil(COUNT / 256), 1, 1);

  const draw = await prepare("draw", [...LIBRARY, header, RENDER]);
  const render = draw.shader;
  render.set("look.size", 1.5);
  render.set("look.top_speed", 2.5);
  render.set("look.slow_color", [0.07, 0.2, 0.55]);
  render.set("look.fast_color", [1, 0.35, 0.5]);
  draw.pass.setClearColor([0.02, 0.02, 0.04, 1]);

  // One small triangle, drawn once per particle. A reused shader still holds
  // the last build's triangle, so that one goes first.
  const triangle = Mesh.fromVertices([[0, 1], [-0.866, -0.5], [0.866, -0.5]]);
  triangle.setInstanceCount(COUNT);
  render.clearMeshes();
  render.addMesh(triangle);

  return {
    compute,
    render,
    stages: [simulate, draw],
    passes: [simulate.pass, draw.pass],
    last: null,
    pointer: null,
    moved: -Infinity,
  };
}

export function frame(state, renderer, target, time, size, pointer) {
  // This program draws, so later builds may reuse its shaders.
  for (const stage of state.stages) kept.set(stage.key, stage);

  // The pointer counts as idle until it moves.
  const [x, y] = pointer;
  if (state.pointer && (x !== state.pointer[0] || y !== state.pointer[1])) state.moved = time;
  state.pointer = [x, y];
  const delta = state.last === null ? 0 : Math.min(time - state.last, 1 / 30);
  const idle = Math.min(time - state.moved, 60);
  state.last = time;

  state.compute.set("sim.resolution", size);
  state.compute.set("sim.pointer", pointer);
  state.compute.set("sim.time", time);
  state.compute.set("sim.delta", delta);
  state.compute.set("sim.idle", idle);
  state.render.set("look.resolution", size);
  renderer.render(state.passes, target);
}

// A Pass keeps its shaders alive for good (a known FragmentColor bug), and each
// shader here holds a 16 MB copy of the particles. So a build reuses the shader
// and pass that an earlier program made from the same sources, if that one drew.
const kept = (globalThis.particleShaders ??= new Map());

async function prepare(name, sources) {
  const key = JSON.stringify([name, ...sources]);
  if (kept.has(key)) return kept.get(key);
  const shader = await Shader.fetch(sources);
  return { key, shader, pass: Pass.fromShader(name, shader) };
}
`;

// The values main.js sets in setup, in order. Keys under look. belong to the render shader.
const VALUES = {
  "sim.strength": 1.2,
  "sim.swirl": 0.4,
  "sim.drag": 0.8,
  "sim.respawn": 0.03,
  "sim.orbit": 0.55,
  "look.size": 1.5,
  "look.top_speed": 2.5,
  "look.slow_color": [0.07, 0.2, 0.55],
  "look.fast_color": [1, 0.35, 0.5],
};

// On 0.12.5 two passes in one render call share their uniforms, and Swift lacks
// some calls used here, so every generated program names the release it needs.
const NEEDS = "Needs FragmentColor 0.12.6 or later.";

function quote(text) {
  return JSON.stringify(String(text));
}

function kotlinQuote(text) {
  return quote(text).replace(/\$/g, "\\$");
}

// Splits the live values between the two shaders by their uniform root.
function describe(model) {
  const values = model.values ?? Object.entries(VALUES);
  const pick = (render) => values.filter(([key]) => key.startsWith("look.") === render);
  return { library: model.library ?? [], compute: pick(false), render: pick(true) };
}

// `head(a, b)tail` on one line when it fits in `width`, otherwise one part per line.
function wrap(indent, head, parts, tail, width) {
  const line = `${indent}${head}${parts.join(", ")}${tail}`;
  if (line.length <= width) return [line];
  return [`${indent}${head}`, ...parts.map((part) => `${indent}    ${part},`), `${indent}${tail}`];
}

// One `set` call per value, spelled the way `lang` writes it.
function setCalls(lang, name, values, { before = "", after = "", text = quote } = {}) {
  return values.map(
    ([key, value, scalar]) => `    ${before}${name}.set(${text(key)}, ${formatValue(lang, value, scalar)})${after}`,
  );
}

function writeRust({ library, compute, render }) {
  const shader = (name, source) =>
    wrap("    ", `let ${name} = Shader::new([`, [...library.map(quote), "header.as_str()", source], "])?;", 100);
  const sets = (name, values) => setCalls("rust", name, values, { after: "?;" });
  return [
    `// ${NEEDS}`,
    "",
    "use std::error::Error;",
    "",
    "use fragmentcolor::{Mesh, Pass, Renderer, Shader, Target};",
    "",
    'const COMPUTE_SOURCE: &str = include_str!("compute.wgsl");',
    'const RENDER_SOURCE: &str = include_str!("render.wgsl");',
    "",
    "// Both shaders size the shared particles buffer with this count.",
    "const COUNT: u32 = 1_000_000;",
    "",
    "struct State {",
    "    compute: Shader,",
    "    render: Shader,",
    "    passes: Vec<Pass>,",
    "    last: Option<f32>,",
    "    pointer: Option<[f32; 2]>,",
    "    moved: f32,",
    "}",
    "",
    "// Call setup once, then frame on every redraw of your window.",
    "fn setup() -> Result<State, Box<dyn Error>> {",
    '    let header = format!("const COUNT: u32 = {COUNT}u;");',
    "",
    ...shader("compute", "COMPUTE_SOURCE"),
    ...sets("compute", compute),
    "",
    ...shader("render", "RENDER_SOURCE"),
    ...sets("render", render),
    "",
    "    // One small triangle, drawn once per particle.",
    "    let triangle = Mesh::from_vertices([[0.0, 1.0], [-0.866, -0.5], [0.866, -0.5]]);",
    "    triangle.set_instance_count(COUNT);",
    "    render.add_mesh(&triangle)?;",
    "",
    "    // 256 matches @workgroup_size in compute.wgsl.",
    '    let simulate = Pass::from_shader("simulate", &compute);',
    "    simulate.set_compute_dispatch(COUNT.div_ceil(256), 1, 1);",
    '    let draw = Pass::from_shader("draw", &render);',
    "    draw.set_clear_color([0.02, 0.02, 0.04, 1.0]);",
    "",
    "    Ok(State {",
    "        compute,",
    "        render,",
    "        passes: vec![simulate, draw],",
    "        last: None,",
    "        pointer: None,",
    "        moved: f32::NEG_INFINITY,",
    "    })",
    "}",
    "",
    "fn frame(",
    "    state: &mut State,",
    "    renderer: &Renderer,",
    "    target: &impl Target,",
    "    time: f32,",
    "    size: [f32; 2],",
    "    pointer: [f32; 2],",
    ") -> Result<(), Box<dyn Error>> {",
    "    // The pointer counts as idle until it moves.",
    "    if state.pointer.is_some_and(|last| last != pointer) {",
    "        state.moved = time;",
    "    }",
    "    state.pointer = Some(pointer);",
    "    let delta = state.last.map_or(0.0, |last| (time - last).min(1.0 / 30.0));",
    "    let idle = (time - state.moved).min(60.0);",
    "    state.last = Some(time);",
    "",
    '    state.compute.set("sim.resolution", size)?;',
    '    state.compute.set("sim.pointer", pointer)?;',
    '    state.compute.set("sim.time", time)?;',
    '    state.compute.set("sim.delta", delta)?;',
    '    state.compute.set("sim.idle", idle)?;',
    '    state.render.set("look.resolution", size)?;',
    "    renderer.render(&state.passes, target)?;",
    "    Ok(())",
    "}",
    "",
  ].join("\n");
}

function writePython({ library, compute, render }) {
  const sets = (name, values) => setCalls("python", name, values);
  return [
    `# ${NEEDS}`,
    "",
    "import math",
    "from pathlib import Path",
    "from types import SimpleNamespace",
    "",
    "from fragmentcolor import Mesh, Pass, Shader",
    "",
    ...wrap("", "LIBRARY = [", library.map(quote), "]", 88),
    'COMPUTE_SOURCE = Path(__file__).with_name("compute.wgsl").read_text()',
    'RENDER_SOURCE = Path(__file__).with_name("render.wgsl").read_text()',
    "",
    "# Both shaders size the shared particles buffer with this count.",
    "COUNT = 1_000_000",
    "",
    "",
    "# Call setup() once, then frame() on every redraw of your canvas.",
    "def setup():",
    '    header = f"const COUNT: u32 = {COUNT}u;"',
    "",
    "    compute = Shader([*LIBRARY, header, COMPUTE_SOURCE])",
    ...sets("compute", compute),
    "",
    "    render = Shader([*LIBRARY, header, RENDER_SOURCE])",
    ...sets("render", render),
    "",
    "    # One small triangle, drawn once per particle.",
    "    triangle = Mesh.from_vertices([[0.0, 1.0], [-0.866, -0.5], [0.866, -0.5]])",
    "    triangle.set_instance_count(COUNT)",
    "    render.add_mesh(triangle)",
    "",
    "    # 256 matches @workgroup_size in compute.wgsl.",
    '    simulate = Pass.from_shader("simulate", compute)',
    "    simulate.set_compute_dispatch(math.ceil(COUNT / 256), 1, 1)",
    '    draw = Pass.from_shader("draw", render)',
    "    draw.set_clear_color([0.02, 0.02, 0.04, 1.0])",
    "",
    "    return SimpleNamespace(",
    "        compute=compute,",
    "        render=render,",
    "        passes=[simulate, draw],",
    "        last=None,",
    "        pointer=None,",
    "        moved=-math.inf,",
    "    )",
    "",
    "",
    "def frame(state, renderer, target, time, size, pointer):",
    "    # The pointer counts as idle until it moves.",
    "    if state.pointer is not None and list(pointer) != state.pointer:",
    "        state.moved = time",
    "    state.pointer = list(pointer)",
    "    delta = 0.0 if state.last is None else min(time - state.last, 1 / 30)",
    "    idle = min(time - state.moved, 60.0)",
    "    state.last = time",
    "",
    '    state.compute.set("sim.resolution", size)',
    '    state.compute.set("sim.pointer", pointer)',
    '    state.compute.set("sim.time", time)',
    '    state.compute.set("sim.delta", delta)',
    '    state.compute.set("sim.idle", idle)',
    '    state.render.set("look.resolution", size)',
    "    renderer.render(state.passes, target)",
    "",
  ].join("\n");
}

function writeSwift({ library, compute, render }) {
  const sets = (name, values) => setCalls("swift", name, values, { before: "try " });
  const libraryLine = library.length
    ? wrap("", "let library = [", library.map(quote), "]", 100)
    : ["let library: [String] = []"];
  return [
    `// ${NEEDS}`,
    "",
    "import Foundation",
    "import FragmentColor",
    "",
    ...libraryLine,
    "",
    "// Both shaders size the shared particles buffer with this count.",
    "let count: UInt32 = 1_000_000",
    "",
    "struct State {",
    "    let compute: Shader",
    "    let render: Shader",
    "    let passes: [Pass]",
    "    var last: Float?",
    "    var pointer: [Float]?",
    "    var moved = -Float.infinity",
    "}",
    "",
    "// Call setup once, then frame from your MTKView or CADisplayLink callback.",
    "func setup() throws -> State {",
    '    let computeURL = Bundle.main.url(forResource: "compute", withExtension: "wgsl")!',
    "    let computeSource = try String(contentsOf: computeURL, encoding: .utf8)",
    '    let renderURL = Bundle.main.url(forResource: "render", withExtension: "wgsl")!',
    "    let renderSource = try String(contentsOf: renderURL, encoding: .utf8)",
    '    let header = "const COUNT: u32 = \\(count)u;"',
    "",
    "    let compute = try Shader(library + [header, computeSource])",
    ...sets("compute", compute),
    "",
    "    let render = try Shader(library + [header, renderSource])",
    ...sets("render", render),
    "",
    "    // One small triangle, drawn once per particle.",
    "    let triangle = try Mesh.fromVertices([[0.0, 1.0], [-0.866, -0.5], [0.866, -0.5]])",
    "    triangle.setInstanceCount(count)",
    "    try render.addMesh(triangle)",
    "",
    "    // 256 matches @workgroup_size in compute.wgsl.",
    '    let simulate = Pass.fromShader("simulate", compute)',
    "    simulate.setComputeDispatch((count + 255) / 256, 1, 1)",
    '    let draw = Pass.fromShader("draw", render)',
    "    try draw.setClearColor([0.02, 0.02, 0.04, 1.0])",
    "",
    "    return State(compute: compute, render: render, passes: [simulate, draw])",
    "}",
    "",
    "func frame(",
    "    state: inout State,",
    "    renderer: Renderer,",
    "    target: WindowTarget,",
    "    time: Float,",
    "    size: [Float],",
    "    pointer: [Float]",
    ") throws {",
    "    // The pointer counts as idle until it moves.",
    "    if let last = state.pointer, last != pointer {",
    "        state.moved = time",
    "    }",
    "    state.pointer = pointer",
    "    let delta = state.last.map { min(time - $0, 1 / 30) } ?? 0",
    "    let idle = min(time - state.moved, 60)",
    "    state.last = time",
    "",
    '    try state.compute.set("sim.resolution", size)',
    '    try state.compute.set("sim.pointer", pointer)',
    '    try state.compute.set("sim.time", time)',
    '    try state.compute.set("sim.delta", delta)',
    '    try state.compute.set("sim.idle", idle)',
    '    try state.render.set("look.resolution", size)',
    "    try renderer.render(state.passes, target)",
    "}",
    "",
  ].join("\n");
}

function writeKotlin({ library, compute, render }) {
  const sets = (name, values) => setCalls("kotlin", name, values, { text: kotlinQuote });
  const libraryLine = library.length
    ? wrap("", "private val LIBRARY = listOf(", library.map(kotlinQuote), ")", 100)
    : ["private val LIBRARY = emptyList<String>()"];
  return [
    `// ${NEEDS}`,
    "",
    "import android.content.res.AssetManager",
    "import org.fragmentcolor.*",
    "",
    ...libraryLine,
    "",
    "// Both shaders size the shared particles buffer with this count.",
    "private const val COUNT = 1_000_000u",
    "",
    "class State(val compute: Shader, val render: Shader, val passes: List<Pass>) {",
    "    var last: Float? = null",
    "    var pointer: List<Float>? = null",
    "    var moved = Float.NEGATIVE_INFINITY",
    "}",
    "",
    "// Call setup once, then frame from your Choreographer or render thread callback.",
    "fun setup(assets: AssetManager): State {",
    '    val computeSource = assets.open("compute.wgsl").bufferedReader().use { it.readText() }',
    '    val renderSource = assets.open("render.wgsl").bufferedReader().use { it.readText() }',
    '    val header = "const COUNT: u32 = ${COUNT}u;"',
    "",
    "    val compute = Shader(LIBRARY + listOf(header, computeSource))",
    ...sets("compute", compute),
    "",
    "    val render = Shader(LIBRARY + listOf(header, renderSource))",
    ...sets("render", render),
    "",
    "    // One small triangle, drawn once per particle.",
    "    val triangle = Mesh.fromVertices(",
    "        listOf(listOf(0.0f, 1.0f), listOf(-0.866f, -0.5f), listOf(0.866f, -0.5f)),",
    "    )",
    "    triangle.setInstanceCount(COUNT)",
    "    render.addMesh(triangle)",
    "",
    "    // 256 matches @workgroup_size in compute.wgsl.",
    '    val simulate = Pass.fromShader("simulate", compute)',
    "    simulate.setComputeDispatch((COUNT + 255u) / 256u, 1u, 1u)",
    '    val draw = Pass.fromShader("draw", render)',
    "    draw.setClearColor(listOf(0.02f, 0.02f, 0.04f, 1.0f))",
    "",
    "    return State(compute, render, listOf(simulate, draw))",
    "}",
    "",
    "fun frame(",
    "    state: State,",
    "    renderer: Renderer,",
    "    target: WindowTarget,",
    "    time: Float,",
    "    size: List<Float>,",
    "    pointer: List<Float>,",
    ") {",
    "    // The pointer counts as idle until it moves.",
    "    if (state.pointer != null && state.pointer != pointer) state.moved = time",
    "    state.pointer = pointer",
    "    val delta = state.last?.let { minOf(time - it, 1f / 30f) } ?: 0f",
    "    val idle = minOf(time - state.moved, 60f)",
    "    state.last = time",
    "",
    '    state.compute.set("sim.resolution", size)',
    '    state.compute.set("sim.pointer", pointer)',
    '    state.compute.set("sim.time", time)',
    '    state.compute.set("sim.delta", delta)',
    '    state.compute.set("sim.idle", idle)',
    '    state.render.set("look.resolution", size)',
    "    renderer.render(state.passes, target)",
    "}",
    "",
  ].join("\n");
}

export default {
  id: "particles",
  title: "Particles",
  summary:
    "One million particles, moved by a compute shader and drawn by an instanced render pass. The pointer pulls them in; when it rests, an orbit takes over.",
  category: "Compute",
  thumbnail: "/playground/thumbs/particles.webp",
  kind: "compute",
  requires: "webgpu",
  poster: "/playground/particles.webp",
  library: [],
  values: VALUES,
  files: { "compute.wgsl": COMPUTE, "render.wgsl": RENDER },
  js: PROGRAM,
  code(model) {
    const spec = describe(model);
    return {
      rust: writeRust(spec),
      python: writePython(spec),
      swift: writeSwift(spec),
      kotlin: writeKotlin(spec),
    };
  },
};
