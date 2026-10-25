// The host program and code tabs of the swarm demos: a swarm in a storage
// buffer, counted into a grid that field.wgsl blurs into slopes, and added up
// into an image that present.wgsl tone-maps. Each demo brings its own
// field.wgsl and update.wgsl, its swarm size, and its starting values.

import { formatValue } from "../codegen.js";

export const CLEAR = /* wgsl */ `// Empties the counts and the image before update.wgsl fills them again.

@group(0) @binding(0) var<storage, read_write> counts: array<vec4<u32>, CELLS>;
@group(0) @binding(1) var<storage, read_write> image: array<u32, PIXELS * 3u>;

@compute @workgroup_size(256)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
  let i = gid.x;
  if (i < CELLS) {
    counts[i] = vec4<u32>(0u);
  }
  if (i < PIXELS) {
    image[i * 3u] = 0u;
    image[i * 3u + 1u] = 0u;
    image[i * 3u + 2u] = 0u;
  }
}
`;

export const PRESENT = /* wgsl */ `// Tone-maps the image that update.wgsl adds up and draws it onto the canvas without stretching.

@group(0) @binding(0) var<uniform> image_size: vec2<u32>;
@group(0) @binding(1) var<storage, read> image: array<u32, PIXELS * 3u>;

struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) uv: vec2<f32>,
}

// Fullscreen triangle: three vertices that cover the canvas, with uv.y down.
@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VertexOutput {
  var corners = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
  var uv = array<vec2<f32>, 3>(vec2<f32>(0.0, 1.0), vec2<f32>(2.0, 1.0), vec2<f32>(0.0, -1.0));
  return VertexOutput(vec4<f32>(corners[i], 0.0, 1.0), uv[i]);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  let pixel = min(vec2<u32>(in.uv * vec2<f32>(image_size)), image_size - 1u);
  let slot = (pixel.y * image_size.x + pixel.x) * 3u;
  let light = vec3<f32>(f32(image[slot]), f32(image[slot + 1u]), f32(image[slot + 2u])) / 255.0;
  // Exposure falls as COUNT per pixel grows and keeps any swarm size looking alike.
  let exposure = 1.5 / (1.0 + f32(COUNT) / f32(image_size.x * image_size.y));
  // Tone-map the brightest channel and scale the others with it, so crowds keep their hue.
  let peak = max(max(light.r, light.g), max(light.b, 1e-6));
  return vec4<f32>(light * (1.0 - exp(-peak * exposure)) / peak, 1.0);
}
`;

/**
 * A swarm demo's main.js and its Rust, Python, Swift, and Kotlin tabs.
 * @param {object} options
 * @param {string} options.id the demo's id, which names its map of kept shaders
 * @param {number} options.count slots in the swarm buffer
 * @param {Record<string, number>} options.values the values main.js sets on update.wgsl, in order
 * @param {Set<string>} options.unsigned the keys among them that update.wgsl declares as u32
 */
export function swarm({ id, count, values, unsigned }) {
  return {
    js: writeProgram(id, count, Object.entries(values)),
    code(model) {
      const spec = describe(model, { count, values, unsigned });
      return {
        rust: writeRust(spec),
        python: writePython(spec),
        swift: writeSwift(spec),
        kotlin: writeKotlin(spec),
      };
    },
  };
}

// main.js: setup builds the four passes, and frame feeds them the pointer.
function writeProgram(id, count, values) {
  return /* js */ `import { Pass, Shader } from "fragmentcolor";
import FIELD from "./field.wgsl?raw";
import CLEAR from "./clear.wgsl?raw";
import UPDATE from "./update.wgsl?raw";
import PRESENT from "./present.wgsl?raw";

const LIBRARY = ["gradient/palette_iq", "gradient/turbo", "hash/pcg2d"];

// Every shader sizes the shared buffers with these.
const COUNT = ${digits(count)};
const ROWS = 96;
// Grid columns for the widest canvas, 4:1. A narrower one uses fewer.
const COLUMNS = ROWS * 4;
// Image pixels. A larger canvas renders the image at a lower resolution.
const PIXELS = 1 << 21;

export async function setup(renderer, target) {
  const header = [
    \`const COUNT: u32 = \${COUNT}u;\`,
    \`const ROWS: u32 = \${ROWS}u;\`,
    \`const COLUMNS: u32 = \${COLUMNS}u;\`,
    "const CELLS: u32 = ROWS * COLUMNS;",
    \`const PIXELS: u32 = \${PIXELS}u;\`,
  ].join("\\n");

  const field = await prepare("field", [header, FIELD]);
  // 8 matches @workgroup_size in field.wgsl.
  field.pass.setComputeDispatch(COLUMNS / 8, ROWS / 8, 1);

  const clear = await prepare("clear", [header, CLEAR]);

  // The library brings the color ramps and the hash that update.wgsl calls.
  const update = await prepare("update", [...LIBRARY, header, UPDATE]);
${values.map(([key, value]) => `  update.shader.set(${quote(key)}, ${formatValue("js", value)});
`).join("")}  // 256 matches @workgroup_size in update.wgsl. A row holds at most 65535 groups.
  const groups = Math.ceil(COUNT / 256);
  update.pass.setComputeDispatch(Math.min(groups, 65535), Math.ceil(groups / 65535), 1);

  const present = await prepare("present", [header, PRESENT]);

  const stages = [field, clear, update, present];
  return {
    field: field.shader,
    update: update.shader,
    present: present.shader,
    clear: clear.pass,
    stages,
    passes: stages.map((stage) => stage.pass),
    size: null,
    frame: 0,
    pointer: null,
    moved: -Infinity,
  };
}

export function frame(state, renderer, target, time, size, pointer) {
  // This program draws, so later builds may reuse its shaders.
  for (const stage of state.stages) kept.set(stage.name, stage);
  const resized = size[0] !== state.size?.[0] || size[1] !== state.size?.[1];
  if (resized) fit(state, size);

  // The pointer pulls while it moves and lets go over 1.5 seconds once it rests.
  // A resize can shift its pixel position while the pointer stays put.
  const [x, y] = pointer;
  const moved = state.pointer && (x !== state.pointer[0] || y !== state.pointer[1]);
  if (moved && !resized) state.moved = time;
  state.pointer = [x, y];

  state.update.set("params.mouse", [x / size[0], y / size[1]]);
  state.update.set("params.grip", Math.max(0, 1 - (time - state.moved) / 1.5));
  state.update.set("params.frame", state.frame++);
  renderer.render(state.passes, target);
}

// Fits the grid and the image to the canvas, so nothing stretches.
function fit(state, [width, height]) {
  const columns = Math.min(Math.max(Math.floor((ROWS * width) / height), 16), COLUMNS);
  let scale = 1;
  while (Math.floor(width / scale) * Math.floor(height / scale) > PIXELS) scale++;
  const image = [Math.max(Math.floor(width / scale), 1), Math.max(Math.floor(height / scale), 1)];

  state.field.set("columns", columns);
  state.update.set("params.columns", columns);
  state.update.set("params.image", image);
  state.present.set("image_size", image);
  // 256 matches @workgroup_size in clear.wgsl.
  const threads = Math.max(image[0] * image[1], ROWS * COLUMNS);
  state.clear.setComputeDispatch(Math.ceil(threads / 256), 1, 1);
  state.size = [width, height];
}

// Each shader holds a copy of every buffer it declares, tens of megabytes for
// update.wgsl. A build reuses the shader and pass of the last program that drew,
// stage by stage, where its sources match. The map belongs to this demo and keys
// a stage by its name alone: a build with new sources or another COUNT replaces
// the kept stage once its program draws, and never adds one more.
const kept = (globalThis.${id}Shaders ??= new Map());

async function prepare(name, sources) {
  const source = JSON.stringify(sources);
  const last = kept.get(name);
  if (last?.source === source) return last;
  const shader = await Shader.fetch(sources);
  return { name, source, shader, pass: Pass.fromShader(name, shader) };
}
`;
}

// FragmentColor 0.12.5 rejects atomic<u32> storage buffers and shares uniforms
// across passes in one render call. The playground runs these demos from 0.12.6 on,
// and every generated program names that release.
export const RELEASE = "0.12.6";
const NEEDS = `Needs FragmentColor ${RELEASE} or later.`;

function quote(text) {
  return JSON.stringify(String(text));
}

function kotlinQuote(text) {
  return quote(text).replace(/\$/g, "\\$");
}

// Keys under params. belong to update.wgsl. Any other key is taken as present.wgsl's.
function describe(model, { count, values: defaults, unsigned }) {
  const values = (model.values ?? Object.entries(defaults)).map(([key, value, scalar]) => [
    key,
    value,
    scalar ?? (unsigned.has(key) ? "uint" : "float"),
  ]);
  const pick = (update) => values.filter(([key]) => key.startsWith("params.") === update);
  return { library: model.library ?? [], update: pick(true), present: pick(false), count };
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

function writeRust({ library, update, present, count }) {
  const sets = (name, values) => setCalls("rust", name, values, { after: "?;" });
  return [
    `// ${NEEDS}`,
    "",
    "use std::error::Error;",
    "",
    "use fragmentcolor::{Pass, Renderer, Shader, Target};",
    "",
    'const FIELD_SOURCE: &str = include_str!("field.wgsl");',
    'const CLEAR_SOURCE: &str = include_str!("clear.wgsl");',
    'const UPDATE_SOURCE: &str = include_str!("update.wgsl");',
    'const PRESENT_SOURCE: &str = include_str!("present.wgsl");',
    "",
    "// Every shader sizes the shared buffers with these.",
    `const COUNT: u32 = ${digits(count)};`,
    "const ROWS: u32 = 96;",
    "// Grid columns for the widest window, 4:1. A narrower one uses fewer.",
    "const COLUMNS: u32 = ROWS * 4;",
    "// Image pixels. A larger window renders the image at a lower resolution.",
    "const PIXELS: u32 = 1 << 21;",
    "",
    "struct State {",
    "    field: Shader,",
    "    update: Shader,",
    "    present: Shader,",
    "    clear: Pass,",
    "    passes: Vec<Pass>,",
    "    size: Option<[f32; 2]>,",
    "    frame: u32,",
    "    pointer: Option<[f32; 2]>,",
    "    moved: f32,",
    "}",
    "",
    "// Call setup once, then frame on every redraw of your window.",
    "fn setup() -> Result<State, Box<dyn Error>> {",
    "    let header = [",
    '        format!("const COUNT: u32 = {COUNT}u;"),',
    '        format!("const ROWS: u32 = {ROWS}u;"),',
    '        format!("const COLUMNS: u32 = {COLUMNS}u;"),',
    '        "const CELLS: u32 = ROWS * COLUMNS;".to_string(),',
    '        format!("const PIXELS: u32 = {PIXELS}u;"),',
    "    ]",
    '    .join("\\n");',
    "",
    "    let field = Shader::new([header.as_str(), FIELD_SOURCE])?;",
    "    let clear = Shader::new([header.as_str(), CLEAR_SOURCE])?;",
    "    // The library brings the color ramps and the hash that update.wgsl calls.",
    ...wrap("    ", "let update = Shader::new([", [...library.map(quote), "header.as_str()", "UPDATE_SOURCE"], "])?;", 100),
    ...sets("update", update),
    "    let present = Shader::new([header.as_str(), PRESENT_SOURCE])?;",
    ...sets("present", present),
    "",
    "    // 8 matches @workgroup_size in field.wgsl.",
    '    let field_pass = Pass::from_shader("field", &field);',
    "    field_pass.set_compute_dispatch(COLUMNS / 8, ROWS / 8, 1);",
    '    let clear_pass = Pass::from_shader("clear", &clear);',
    "    // 256 matches @workgroup_size in update.wgsl. A row holds at most 65535 groups.",
    '    let update_pass = Pass::from_shader("update", &update);',
    "    let groups = COUNT.div_ceil(256);",
    "    update_pass.set_compute_dispatch(groups.min(65535), groups.div_ceil(65535), 1);",
    '    let present_pass = Pass::from_shader("present", &present);',
    "",
    "    Ok(State {",
    "        passes: vec![field_pass, clear_pass.clone(), update_pass, present_pass],",
    "        field,",
    "        update,",
    "        present,",
    "        clear: clear_pass,",
    "        size: None,",
    "        frame: 0,",
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
    "    let resized = state.size != Some(size);",
    "    if resized {",
    "        fit(state, size)?;",
    "    }",
    "",
    "    // The pointer pulls while it moves and lets go over 1.5 seconds once it rests.",
    "    // A resize can shift its pixel position while the pointer stays put.",
    "    if !resized && state.pointer.is_some_and(|last| last != pointer) {",
    "        state.moved = time;",
    "    }",
    "    state.pointer = Some(pointer);",
    "",
    "    let mouse = [pointer[0] / size[0], pointer[1] / size[1]];",
    '    state.update.set("params.mouse", mouse)?;',
    "    let grip = (1.0 - (time - state.moved) / 1.5).max(0.0);",
    '    state.update.set("params.grip", grip)?;',
    '    state.update.set("params.frame", state.frame)?;',
    "    state.frame += 1;",
    "    renderer.render(&state.passes, target)?;",
    "    Ok(())",
    "}",
    "",
    "// Fits the grid and the image to the window, so nothing stretches.",
    "fn fit(state: &mut State, size: [f32; 2]) -> Result<(), Box<dyn Error>> {",
    "    let width = (size[0] as u32).max(1);",
    "    let height = (size[1] as u32).max(1);",
    "    let columns = (ROWS * width / height).clamp(16, COLUMNS);",
    "    let scale = (1..)",
    "        .find(|scale| (width / scale) * (height / scale) <= PIXELS)",
    "        .unwrap_or(1);",
    "    let image = [(width / scale).max(1), (height / scale).max(1)];",
    "",
    '    state.field.set("columns", columns)?;',
    '    state.update.set("params.columns", columns)?;',
    '    state.update.set("params.image", image)?;',
    '    state.present.set("image_size", image)?;',
    "    // 256 matches @workgroup_size in clear.wgsl.",
    "    let threads = (image[0] * image[1]).max(ROWS * COLUMNS);",
    "    let groups = threads.div_ceil(256);",
    "    state.clear.set_compute_dispatch(groups, 1, 1);",
    "    state.size = Some(size);",
    "    Ok(())",
    "}",
    "",
  ].join("\n");
}

function writePython({ library, update, present, count }) {
  const sets = (name, values) => setCalls("python", name, values);
  return [
    `# ${NEEDS}`,
    "",
    "import math",
    "from pathlib import Path",
    "from types import SimpleNamespace",
    "",
    "from fragmentcolor import Pass, Shader",
    "",
    ...wrap("", "LIBRARY = [", library.map(quote), "]", 88),
    'FIELD_SOURCE = Path(__file__).with_name("field.wgsl").read_text()',
    'CLEAR_SOURCE = Path(__file__).with_name("clear.wgsl").read_text()',
    'UPDATE_SOURCE = Path(__file__).with_name("update.wgsl").read_text()',
    'PRESENT_SOURCE = Path(__file__).with_name("present.wgsl").read_text()',
    "",
    "# Every shader sizes the shared buffers with these.",
    `COUNT = ${digits(count)}`,
    "ROWS = 96",
    "# Grid columns for the widest canvas, 4:1. A narrower one uses fewer.",
    "COLUMNS = ROWS * 4",
    "# Image pixels. A larger canvas renders the image at a lower resolution.",
    "PIXELS = 1 << 21",
    "",
    "",
    "# Call setup() once, then frame() on every redraw of your canvas.",
    "def setup():",
    '    header = "\\n".join(',
    "        [",
    '            f"const COUNT: u32 = {COUNT}u;",',
    '            f"const ROWS: u32 = {ROWS}u;",',
    '            f"const COLUMNS: u32 = {COLUMNS}u;",',
    '            "const CELLS: u32 = ROWS * COLUMNS;",',
    '            f"const PIXELS: u32 = {PIXELS}u;",',
    "        ]",
    "    )",
    "",
    "    field = Shader([header, FIELD_SOURCE])",
    "    clear = Shader([header, CLEAR_SOURCE])",
    "    # The library brings the color ramps and the hash that update.wgsl calls.",
    "    update = Shader([*LIBRARY, header, UPDATE_SOURCE])",
    ...sets("update", update),
    "    present = Shader([header, PRESENT_SOURCE])",
    ...sets("present", present),
    "",
    "    # 8 matches @workgroup_size in field.wgsl.",
    '    field_pass = Pass.from_shader("field", field)',
    "    field_pass.set_compute_dispatch(COLUMNS // 8, ROWS // 8, 1)",
    '    clear_pass = Pass.from_shader("clear", clear)',
    "    # 256 matches @workgroup_size in update.wgsl. A row holds at most 65535 groups.",
    '    update_pass = Pass.from_shader("update", update)',
    "    groups = math.ceil(COUNT / 256)",
    "    update_pass.set_compute_dispatch(min(groups, 65535), math.ceil(groups / 65535), 1)",
    '    present_pass = Pass.from_shader("present", present)',
    "",
    "    return SimpleNamespace(",
    "        field=field,",
    "        update=update,",
    "        present=present,",
    "        clear=clear_pass,",
    "        passes=[field_pass, clear_pass, update_pass, present_pass],",
    "        size=None,",
    "        frame=0,",
    "        pointer=None,",
    "        moved=-math.inf,",
    "    )",
    "",
    "",
    "def frame(state, renderer, target, time, size, pointer):",
    "    resized = state.size != list(size)",
    "    if resized:",
    "        fit(state, size)",
    "",
    "    # The pointer pulls while it moves and lets go over 1.5 seconds once it rests.",
    "    # A resize can shift its pixel position while the pointer stays put.",
    "    if not resized and state.pointer is not None and list(pointer) != state.pointer:",
    "        state.moved = time",
    "    state.pointer = list(pointer)",
    "",
    '    state.update.set("params.mouse", [pointer[0] / size[0], pointer[1] / size[1]])',
    '    state.update.set("params.grip", max(0.0, 1 - (time - state.moved) / 1.5))',
    '    state.update.set("params.frame", state.frame)',
    "    state.frame += 1",
    "    renderer.render(state.passes, target)",
    "",
    "",
    "# Fits the grid and the image to the canvas, so nothing stretches.",
    "def fit(state, size):",
    "    width = max(int(size[0]), 1)",
    "    height = max(int(size[1]), 1)",
    "    columns = min(max(ROWS * width // height, 16), COLUMNS)",
    "    scale = 1",
    "    while (width // scale) * (height // scale) > PIXELS:",
    "        scale += 1",
    "    image = [max(width // scale, 1), max(height // scale, 1)]",
    "",
    '    state.field.set("columns", columns)',
    '    state.update.set("params.columns", columns)',
    '    state.update.set("params.image", image)',
    '    state.present.set("image_size", image)',
    "    # 256 matches @workgroup_size in clear.wgsl.",
    "    threads = max(image[0] * image[1], ROWS * COLUMNS)",
    "    state.clear.set_compute_dispatch(math.ceil(threads / 256), 1, 1)",
    "    state.size = list(size)",
    "",
  ].join("\n");
}

function writeSwift({ library, update, present, count }) {
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
    "// Every shader sizes the shared buffers with these.",
    `let swarmSize: UInt32 = ${digits(count)}`,
    "let gridRows: UInt32 = 96",
    "// Grid columns for the widest window, 4:1. A narrower one uses fewer.",
    "let gridColumns: UInt32 = gridRows * 4",
    "// Image pixels. A larger window renders the image at a lower resolution.",
    "let imagePixels: UInt32 = 1 << 21",
    "",
    "struct State {",
    "    let field: Shader",
    "    let update: Shader",
    "    let present: Shader",
    "    let clear: Pass",
    "    let passes: [Pass]",
    "    var size: [Float]?",
    "    var frame: UInt32 = 0",
    "    var pointer: [Float]?",
    "    var moved = -Float.infinity",
    "}",
    "",
    "// Reads a WGSL file bundled with the app.",
    "func readSource(_ name: String) throws -> String {",
    '    let url = Bundle.main.url(forResource: name, withExtension: "wgsl")!',
    "    return try String(contentsOf: url, encoding: .utf8)",
    "}",
    "",
    "// Call setup once, then frame from your MTKView or CADisplayLink callback.",
    "func setup() throws -> State {",
    '    let header = """',
    "        const COUNT: u32 = \\(swarmSize)u;",
    "        const ROWS: u32 = \\(gridRows)u;",
    "        const COLUMNS: u32 = \\(gridColumns)u;",
    "        const CELLS: u32 = ROWS * COLUMNS;",
    "        const PIXELS: u32 = \\(imagePixels)u;",
    '        """',
    "",
    '    let field = try Shader([header, readSource("field")])',
    '    let clear = try Shader([header, readSource("clear")])',
    "    // The library brings the color ramps and the hash that update.wgsl calls.",
    '    let update = try Shader(library + [header, readSource("update")])',
    ...sets("update", update),
    '    let present = try Shader([header, readSource("present")])',
    ...sets("present", present),
    "",
    "    // 8 matches @workgroup_size in field.wgsl.",
    '    let fieldPass = Pass.fromShader("field", field)',
    "    fieldPass.setComputeDispatch(gridColumns / 8, gridRows / 8, 1)",
    '    let clearPass = Pass.fromShader("clear", clear)',
    "    // 256 matches @workgroup_size in update.wgsl. A row holds at most 65535 groups.",
    '    let updatePass = Pass.fromShader("update", update)',
    "    let groups = (swarmSize + 255) / 256",
    "    updatePass.setComputeDispatch(min(groups, 65535), (groups + 65534) / 65535, 1)",
    '    let presentPass = Pass.fromShader("present", present)',
    "",
    "    return State(",
    "        field: field,",
    "        update: update,",
    "        present: present,",
    "        clear: clearPass,",
    "        passes: [fieldPass, clearPass, updatePass, presentPass]",
    "    )",
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
    "    let resized = state.size != size",
    "    if resized {",
    "        try fit(&state, size)",
    "    }",
    "",
    "    // The pointer pulls while it moves and lets go over 1.5 seconds once it rests.",
    "    // A resize can shift its pixel position while the pointer stays put.",
    "    if !resized, let last = state.pointer, last != pointer {",
    "        state.moved = time",
    "    }",
    "    state.pointer = pointer",
    "",
    '    try state.update.set("params.mouse", [pointer[0] / size[0], pointer[1] / size[1]])',
    '    try state.update.set("params.grip", max(0, 1 - (time - state.moved) / 1.5))',
    '    try state.update.set("params.frame", state.frame)',
    "    state.frame += 1",
    "    try renderer.render(state.passes, target)",
    "}",
    "",
    "// Fits the grid and the image to the window, so nothing stretches.",
    "func fit(_ state: inout State, _ size: [Float]) throws {",
    "    let width = max(UInt32(size[0]), 1)",
    "    let height = max(UInt32(size[1]), 1)",
    "    let columns = min(max(gridRows * width / height, 16), gridColumns)",
    "    var scale: UInt32 = 1",
    "    while (width / scale) * (height / scale) > imagePixels {",
    "        scale += 1",
    "    }",
    "    let image = [max(width / scale, 1), max(height / scale, 1)]",
    "",
    '    try state.field.set("columns", columns)',
    '    try state.update.set("params.columns", columns)',
    '    try state.update.set("params.image", image)',
    '    try state.present.set("image_size", image)',
    "    // 256 matches @workgroup_size in clear.wgsl.",
    "    let threads = max(image[0] * image[1], gridRows * gridColumns)",
    "    state.clear.setComputeDispatch((threads + 255) / 256, 1, 1)",
    "    state.size = size",
    "}",
    "",
  ].join("\n");
}

function writeKotlin({ library, update, present, count }) {
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
    "// Every shader sizes the shared buffers with these.",
    `private const val COUNT = ${digits(count)}u`,
    "private const val ROWS = 96u",
    "",
    "// Grid columns for the widest window, 4:1. A narrower one uses fewer.",
    "private val COLUMNS = ROWS * 4u",
    "",
    "// Image pixels. A larger window renders the image at a lower resolution.",
    "private val PIXELS = 1u shl 21",
    "",
    "class State(",
    "    val field: Shader,",
    "    val update: Shader,",
    "    val present: Shader,",
    "    val clear: Pass,",
    "    val passes: List<Pass>,",
    ") {",
    "    var size: List<Float>? = null",
    "    var frame = 0u",
    "    var pointer: List<Float>? = null",
    "    var moved = Float.NEGATIVE_INFINITY",
    "}",
    "",
    "// Call setup once, then frame from your Choreographer or render thread callback.",
    "fun setup(assets: AssetManager): State {",
    "    fun read(name: String) = assets.open(name).bufferedReader().use { it.readText() }",
    "    val header =",
    "        listOf(",
    '            "const COUNT: u32 = ${COUNT}u;",',
    '            "const ROWS: u32 = ${ROWS}u;",',
    '            "const COLUMNS: u32 = ${COLUMNS}u;",',
    '            "const CELLS: u32 = ROWS * COLUMNS;",',
    '            "const PIXELS: u32 = ${PIXELS}u;",',
    '        ).joinToString("\\n")',
    "",
    '    val field = Shader(listOf(header, read("field.wgsl")))',
    '    val clear = Shader(listOf(header, read("clear.wgsl")))',
    "    // The library brings the color ramps and the hash that update.wgsl calls.",
    '    val update = Shader(LIBRARY + listOf(header, read("update.wgsl")))',
    ...sets("update", update),
    '    val present = Shader(listOf(header, read("present.wgsl")))',
    ...sets("present", present),
    "",
    "    // 8 matches @workgroup_size in field.wgsl.",
    '    val fieldPass = Pass.fromShader("field", field)',
    "    fieldPass.setComputeDispatch(COLUMNS / 8u, ROWS / 8u, 1u)",
    '    val clearPass = Pass.fromShader("clear", clear)',
    "    // 256 matches @workgroup_size in update.wgsl. A row holds at most 65535 groups.",
    '    val updatePass = Pass.fromShader("update", update)',
    "    val groups = (COUNT + 255u) / 256u",
    "    updatePass.setComputeDispatch(minOf(groups, 65535u), (groups + 65534u) / 65535u, 1u)",
    '    val presentPass = Pass.fromShader("present", present)',
    "",
    "    val passes = listOf(fieldPass, clearPass, updatePass, presentPass)",
    "    return State(field, update, present, clearPass, passes)",
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
    "    val resized = state.size != size",
    "    if (resized) fit(state, size)",
    "",
    "    // The pointer pulls while it moves and lets go over 1.5 seconds once it rests.",
    "    // A resize can shift its pixel position while the pointer stays put.",
    "    if (!resized && state.pointer != null && state.pointer != pointer) state.moved = time",
    "    state.pointer = pointer",
    "",
    '    state.update.set("params.mouse", listOf(pointer[0] / size[0], pointer[1] / size[1]))',
    '    state.update.set("params.grip", maxOf(0f, 1f - (time - state.moved) / 1.5f))',
    '    state.update.set("params.frame", state.frame)',
    "    state.frame++",
    "    renderer.render(state.passes, target)",
    "}",
    "",
    "// Fits the grid and the image to the window, so nothing stretches.",
    "fun fit(",
    "    state: State,",
    "    size: List<Float>,",
    ") {",
    "    val width = maxOf(size[0].toUInt(), 1u)",
    "    val height = maxOf(size[1].toUInt(), 1u)",
    "    val columns = (ROWS * width / height).coerceIn(16u, COLUMNS)",
    "    var scale = 1u",
    "    while ((width / scale) * (height / scale) > PIXELS) scale++",
    "    val image = listOf(maxOf(width / scale, 1u), maxOf(height / scale, 1u))",
    "",
    '    state.field.set("columns", columns)',
    '    state.update.set("params.columns", columns)',
    '    state.update.set("params.image", image)',
    '    state.present.set("image_size", image)',
    "    // 256 matches @workgroup_size in clear.wgsl.",
    "    val threads = maxOf(image[0] * image[1], ROWS * COLUMNS)",
    "    state.clear.setComputeDispatch((threads + 255u) / 256u, 1u, 1u)",
    "    state.size = size",
    "}",
    "",
  ].join("\n");
}


// A whole number with an underscore between each group of three digits, as every language writes it.
function digits(count) {
  return String(count).replace(/\B(?=(\d{3})+(?!\d))/g, "_");
}
