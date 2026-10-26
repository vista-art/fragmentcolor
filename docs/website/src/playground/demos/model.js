// A glTF model loaded with the scene layer and lit by two lights. A drag turns it,
// and the wheel or a pinch moves the camera closer. The scene draws into a float
// image with a depth buffer, and a present pass turns its light values into
// screen colors over a water gradient.

import { formatValue } from "../codegen.js";

const PRESENT = /* wgsl */ `// Turns the light values the scene drew into colors for the screen,
// over water that brightens toward the surface.

struct Look {
  exposure: f32, // 0.25..4
  surface_color: vec3<f32>,
  deep_color: vec3<f32>,
}

@group(0) @binding(0) var<uniform> look: Look;
@group(0) @binding(1) var image: texture_2d<f32>;
@group(0) @binding(2) var image_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  // The fish pass clears to transparent, so alpha marks where the fish is.
  let fish = textureSample(image, image_sampler, vec2<f32>(in.uv.x, 1.0 - in.uv.y));
  let color = linear_to_srgb(tonemap_aces(fish.rgb * look.exposure));

  // Light falls in from above and fades with depth.
  let above = in.uv - vec2<f32>(0.5, 1.15);
  let glow = exp(-6.0 * dot(above, above)) * 0.35;
  let water = mix(look.deep_color, look.surface_color, smoothstep(0.0, 1.0, in.uv.y)) + look.surface_color * glow;
  return vec4<f32>(mix(water, color, fish.a), 1.0);
}

// Fullscreen triangle: three vertices that cover the canvas.
struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VertexOutput {
  var corners = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(3.0, -1.0),
    vec2<f32>(-1.0, 3.0),
  );
  var out: VertexOutput;
  out.position = vec4<f32>(corners[i], 0.0, 1.0);
  out.uv = corners[i] * 0.5 + vec2<f32>(0.5);
  return out;
}
`;

const PROGRAM = /* js */ `import { Camera, Light, Pass, Scene, Shader, TextureFormat, TextureUsage } from "fragmentcolor";
import PRESENT_SOURCE from "./present.wgsl?raw";

const LIBRARY = ["color/tonemap_aces", "color/linear_to_srgb"];

// Barramundi Fish by Microsoft, CC0, from the Khronos glTF sample assets.
const MODEL = "/playground/models/BarramundiFish.glb";
// The middle of the fish and how far the camera starts from it, in meters.
const CENTER = [0, 0.143, 0];
const DISTANCE = 1.3;
// The key light shines from above and left of the camera, the rim light from behind the fish.
const KEY = [-0.5, -0.6, -1];
const RIM = [0.6, 0.2, 1];
// How fast the fish turns before the first drag, in radians per second.
const SPIN = 0.4;

export async function setup(renderer, target) {
  const response = await fetch(MODEL);
  const scene = Scene.load(new Uint8Array(await response.arrayBuffer()));
  // Upload the textures now, so the first frame only draws.
  await renderer.load(scene);

  const camera = Camera.perspective(0.6, 1, 0.01, 10);
  const key = Light.directional(KEY, [1, 0.96, 0.9]).setIntensity(3);
  const rim = Light.directional(RIM, [0.6, 0.7, 1]).setIntensity(1);
  scene.add(camera).add(key).add(rim).ambient([0.1, 0.1, 0.12]);

  // The fish pass clears to transparent, so present.wgsl can lay the fish over the water.
  const fish = scene.getPass(0);
  fish.setClearColor([0, 0, 0, 0]);

  const present = await Shader.fetch([...LIBRARY, PRESENT_SOURCE]);
  present.set("look.exposure", 1.5);
  present.set("look.surface_color", [0.25, 0.58, 0.68]);
  present.set("look.deep_color", [0.02, 0.12, 0.2]);
  scene.addPass(Pass.fromShader("present", present));

  // The playground gives a control to each value setup sets on a shader it returns.
  const state = {
    scene,
    present,
    camera,
    key,
    rim,
    fish,
    image: null,
    depth: null,
    size: null,
    wanted: null,
    yaw: 1,
    pitch: 0.25,
    distance: DISTANCE,
    turning: true,
    last: null,
  };
  const { width, height } = target.size();
  await fit(state, renderer, [width, height]);
  return state;
}

export function frame(state, renderer, target, time, size, pointer, gesture = { zoom: 0, pan: [0, 0] }) {
  if (size[0] !== state.wanted[0] || size[1] !== state.wanted[1]) fit(state, renderer, size);
  const delta = state.last === null ? 0 : Math.min(Math.max(time - state.last, 0), 0.1);
  state.last = time;

  // The fish turns by itself until the first drag, scroll, or pinch.
  if (gesture.zoom || gesture.pan[0] || gesture.pan[1]) state.turning = false;
  if (state.turning) state.yaw += delta * SPIN;
  // A drag the height of the canvas turns the camera half a circle.
  state.yaw -= (gesture.pan[0] / size[1]) * Math.PI;
  state.pitch = clamp(state.pitch + (gesture.pan[1] / size[1]) * Math.PI, -1.4, 1.4);
  state.distance = clamp(state.distance / 2 ** gesture.zoom, DISTANCE / 3, DISTANCE * 3);

  const reach = state.distance * Math.cos(state.pitch);
  const eye = [
    CENTER[0] + reach * Math.sin(state.yaw),
    CENTER[1] + state.distance * Math.sin(state.pitch),
    CENTER[2] + reach * Math.cos(state.yaw),
  ];
  state.camera.setAspect(state.size[0] / state.size[1]).lookAt(eye, CENTER, [0, 1, 0]);
  // Both lights turn with the camera, so the side in view stays lit.
  state.key.setDirection(turn(KEY, state.yaw));
  state.rim.setDirection(turn(RIM, state.yaw));
  renderer.render(state.scene, target);
}

// The fish draws into a 16-bit float image, so light brighter than white reaches
// the tone map, and a depth buffer keeps its far side behind its near side.
// Neither can change size, so a new canvas size gets new ones, and the fish
// keeps drawing at the old size until they are ready.
async function fit(state, renderer, size) {
  state.wanted = size;
  const usage = TextureUsage.RENDER_ATTACHMENT | TextureUsage.TEXTURE_BINDING;
  const image = await renderer.createStorageTexture(size, TextureFormat.Rgba16Float, null, usage);
  const depth = await renderer.createDepthTexture(size);
  // The renderer keeps every texture until it is unregistered. When a newer size
  // came in during the wait, these two are already out of date.
  if (state.wanted !== size) {
    renderer.unregisterTexture(image.id());
    renderer.unregisterTexture(depth.id());
    return;
  }
  for (const old of [state.image, state.depth]) if (old) renderer.unregisterTexture(old.id());
  state.fish.setTarget(image);
  state.fish.setDepthTarget(depth);
  state.present.set("image", image);
  Object.assign(state, { image, depth, size });
}

// Turns a direction around the vertical axis by yaw radians.
function turn([x, y, z], yaw) {
  const [c, s] = [Math.cos(yaw), Math.sin(yaw)];
  return [x * c + z * s, y, z * c - x * s];
}

function clamp(value, low, high) {
  return Math.min(Math.max(value, low), high);
}
`;

// The values main.js sets in setup.
const VALUES = {
  "look.exposure": 1.5,
  "look.surface_color": [0.25, 0.58, 0.68],
  "look.deep_color": [0.02, 0.12, 0.2],
};

// The site loads FragmentColor 0.12.6, where Shader.set takes a texture on the web.
const RELEASE = "0.12.6";
const NEEDS = `Needs FragmentColor ${RELEASE} or later.`;
// The Rust tab handles the camera errors 0.12.7 returns with `?`, which 0.12.6 can't compile.
const RUST_NEEDS = "Needs FragmentColor 0.12.7 or later.";

const CREDIT = "Barramundi Fish by Microsoft, CC0, from the Khronos glTF sample assets.";

function quote(text) {
  return JSON.stringify(String(text));
}

function describe(model) {
  return { library: model.library ?? [], values: model.values ?? Object.entries(VALUES) };
}

// `head(a, b)tail` on one line when it fits in `width`, otherwise one part per line.
function wrap(indent, head, parts, tail, width) {
  const line = `${indent}${head}${parts.join(", ")}${tail}`;
  if (line.length <= width) return [line];
  return [`${indent}${head}`, ...parts.map((part) => `${indent}    ${part},`), `${indent}${tail}`];
}

// One `set` call per value, spelled the way `lang` writes it.
function setCalls(lang, values, { before = "", after = "" } = {}) {
  return values.map(
    ([key, value, scalar]) => `    ${before}present.set(${quote(key)}, ${formatValue(lang, value, scalar)})${after}`,
  );
}

function writeRust({ library, values }) {
  return [
    `// ${RUST_NEEDS}`,
    "",
    "use std::error::Error;",
    "use std::f32::consts::PI;",
    "",
    "use fragmentcolor::{",
    "    Camera, Light, Pass, Renderer, Scene, Shader, Target, Texture, TextureData, TextureFormat,",
    "    TextureInput, TextureOptions, TextureUsage,",
    "};",
    "",
    'const PRESENT_SOURCE: &str = include_str!("present.wgsl");',
    "",
    `/// ${CREDIT}`,
    'const MODEL: &str = "BarramundiFish.glb";',
    "/// The middle of the fish and how far the camera starts from it, in meters.",
    "const CENTER: [f32; 3] = [0.0, 0.143, 0.0];",
    "const DISTANCE: f32 = 1.3;",
    "/// The key light shines from above and left of the camera, the rim light from behind the fish.",
    "const KEY: [f32; 3] = [-0.5, -0.6, -1.0];",
    "const RIM: [f32; 3] = [0.6, 0.2, 1.0];",
    "/// How fast the fish turns before the first drag, in radians per second.",
    "const SPIN: f32 = 0.4;",
    "",
    "/// What the wheel, a drag and a pinch did since the last frame: zoom in doublings",
    "/// (positive zooms in) and pan in window pixels.",
    "#[derive(Clone, Copy, Default)]",
    "struct Gesture {",
    "    zoom: f32,",
    "    pan: [f32; 2],",
    "}",
    "",
    "struct State {",
    "    scene: Scene,",
    "    present: Shader,",
    "    camera: Camera,",
    "    key: Light,",
    "    rim: Light,",
    "    fish: Pass,",
    "    image: Option<Texture>,",
    "    depth: Option<Texture>,",
    "    size: Option<[f32; 2]>,",
    "    yaw: f32,",
    "    pitch: f32,",
    "    distance: f32,",
    "    turning: bool,",
    "    last: Option<f32>,",
    "}",
    "",
    "// Await setup once, then await frame on every redraw of your window.",
    "async fn setup(renderer: &Renderer) -> Result<State, Box<dyn Error>> {",
    "    let scene = Scene::load(MODEL)?;",
    "    // Upload the textures now, so the first frame only draws.",
    "    renderer.load(&scene).await?;",
    "",
    "    let camera = Camera::perspective(0.6, 1.0, 0.01, 10.0)?;",
    "    let key = Light::directional(KEY, [1.0, 0.96, 0.9]).set_intensity(3.0);",
    "    let rim = Light::directional(RIM, [0.6, 0.7, 1.0]).set_intensity(1.0);",
    "    scene",
    "        .add(&camera)?",
    "        .add(&key)?",
    "        .add(&rim)?",
    "        .ambient([0.1, 0.1, 0.12]);",
    "",
    "    // The fish pass clears to transparent, so present.wgsl can lay the fish over the water.",
    "    // The first frame gives it an image and a depth buffer the size of the window.",
    '    let fish = scene.get_pass(0).ok_or("the model loaded no pass")?;',
    "    fish.set_clear_color([0.0, 0.0, 0.0, 0.0])?;",
    "",
    ...wrap("    ", "let present = Shader::new([", [...library.map(quote), "PRESENT_SOURCE"], "])?;", 100),
    ...setCalls("rust", values, { after: "?;" }),
    '    scene.add_pass(&Pass::from_shader("present", &present));',
    "",
    "    Ok(State {",
    "        scene,",
    "        present,",
    "        camera,",
    "        key,",
    "        rim,",
    "        fish,",
    "        image: None,",
    "        depth: None,",
    "        size: None,",
    "        yaw: 1.0,",
    "        pitch: 0.25,",
    "        distance: DISTANCE,",
    "        turning: true,",
    "        last: None,",
    "    })",
    "}",
    "",
    "async fn frame(",
    "    state: &mut State,",
    "    renderer: &Renderer,",
    "    target: &impl Target,",
    "    time: f32,",
    "    size: [f32; 2],",
    "    _pointer: [f32; 2],",
    "    gesture: Gesture,",
    ") -> Result<(), Box<dyn Error>> {",
    "    if state.size != Some(size) {",
    "        fit(state, renderer, size).await?;",
    "    }",
    "    let delta = state.last.map_or(0.0, |last| (time - last).clamp(0.0, 0.1));",
    "    state.last = Some(time);",
    "",
    "    // The fish turns by itself until the first drag, scroll, or pinch.",
    "    if gesture.zoom != 0.0 || gesture.pan != [0.0, 0.0] {",
    "        state.turning = false;",
    "    }",
    "    if state.turning {",
    "        state.yaw += delta * SPIN;",
    "    }",
    "    // A drag the height of the window turns the camera half a circle.",
    "    state.yaw -= gesture.pan[0] / size[1] * PI;",
    "    state.pitch = (state.pitch + gesture.pan[1] / size[1] * PI).clamp(-1.4, 1.4);",
    "    state.distance = (state.distance / gesture.zoom.exp2()).clamp(DISTANCE / 3.0, DISTANCE * 3.0);",
    "",
    "    let reach = state.distance * state.pitch.cos();",
    "    let eye = [",
    "        CENTER[0] + reach * state.yaw.sin(),",
    "        CENTER[1] + state.distance * state.pitch.sin(),",
    "        CENTER[2] + reach * state.yaw.cos(),",
    "    ];",
    "    state",
    "        .camera",
    "        .set_aspect(size[0].max(1.0) / size[1].max(1.0))?",
    "        .look_at(eye, CENTER, [0.0, 1.0, 0.0])?;",
    "    // Both lights turn with the camera, so the side in view stays lit.",
    "    state.key.set_direction(turn(KEY, state.yaw))?;",
    "    state.rim.set_direction(turn(RIM, state.yaw))?;",
    "    renderer.render(&state.scene, target)?;",
    "    Ok(())",
    "}",
    "",
    "/// The fish draws into a 16-bit float image, so light brighter than white reaches",
    "/// the tone map, and a depth buffer keeps its far side behind its near side.",
    "/// Neither can change size, so a new window size gets new ones.",
    "async fn fit(state: &mut State, renderer: &Renderer, size: [f32; 2]) -> Result<(), Box<dyn Error>> {",
    "    let pixels = [size[0].max(1.0) as u32, size[1].max(1.0) as u32];",
    "    let options = TextureOptions {",
    "        size: Some(pixels.into()),",
    "        format: TextureFormat::Rgba16Float,",
    "        ..Default::default()",
    "    }",
    "    .with_usage(TextureUsage::RENDER_ATTACHMENT | TextureUsage::TEXTURE_BINDING);",
    "    let input = TextureInput {",
    "        data: TextureData::Empty,",
    "        options,",
    "    };",
    "    let image = renderer.create_storage_texture(input).await?;",
    "    let depth = renderer.create_depth_texture(pixels).await?;",
    "",
    "    // The renderer keeps every texture until it is unregistered.",
    "    let stale = [state.image.take(), state.depth.take()];",
    "    for old in stale.into_iter().flatten() {",
    "        renderer.unregister_texture(old.id())?;",
    "    }",
    "    state.fish.set_target(&image)?;",
    "    state.fish.set_depth_target(&depth)?;",
    '    state.present.set("image", &image)?;',
    "    state.image = Some(image);",
    "    state.depth = Some(depth);",
    "    state.size = Some(size);",
    "    Ok(())",
    "}",
    "",
    "/// Turns a direction around the vertical axis by `yaw` radians.",
    "fn turn([x, y, z]: [f32; 3], yaw: f32) -> [f32; 3] {",
    "    let (sin, cos) = yaw.sin_cos();",
    "    [x * cos + z * sin, y, z * cos - x * sin]",
    "}",
    "",
  ].join("\n");
}

function writePython({ library, values }) {
  return [
    `# ${NEEDS}`,
    "",
    "import math",
    "from pathlib import Path",
    "from types import SimpleNamespace",
    "",
    "from fragmentcolor import (",
    "    Camera,",
    "    Light,",
    "    Pass,",
    "    Scene,",
    "    Shader,",
    "    TextureFormat,",
    "    TextureUsage,",
    ")",
    "",
    ...wrap("", "LIBRARY = [", library.map(quote), "]", 88),
    'PRESENT_SOURCE = Path(__file__).with_name("present.wgsl").read_text()',
    "",
    `# ${CREDIT}`,
    'MODEL = Path(__file__).with_name("BarramundiFish.glb")',
    "# The middle of the fish and how far the camera starts from it, in meters.",
    "CENTER = [0.0, 0.143, 0.0]",
    "DISTANCE = 1.3",
    "# The key light shines from above and left of the camera,",
    "# the rim light from behind the fish.",
    "KEY = [-0.5, -0.6, -1.0]",
    "RIM = [0.6, 0.2, 1.0]",
    "# How fast the fish turns before the first drag, in radians per second.",
    "SPIN = 0.4",
    "",
    "",
    "# Call setup() once, then frame() on every redraw of your canvas. frame's gesture",
    "# is what the wheel, a drag and a pinch did since the last frame: zoom in doublings",
    "# (positive zooms in) and pan in canvas pixels.",
    "def setup(renderer):",
    "    scene = Scene.load(str(MODEL))",
    "    # Upload the textures now, so the first frame only draws.",
    "    renderer.load(scene)",
    "",
    "    camera = Camera.perspective(0.6, 1.0, 0.01, 10.0)",
    "    key = Light.directional(KEY, [1.0, 0.96, 0.9]).set_intensity(3.0)",
    "    rim = Light.directional(RIM, [0.6, 0.7, 1.0]).set_intensity(1.0)",
    "    scene.add(camera).add(key).add(rim).ambient([0.1, 0.1, 0.12])",
    "",
    "    # The fish pass clears to transparent, so present.wgsl can lay the fish",
    "    # over the water. The first frame gives it an image and a depth buffer",
    "    # the size of the canvas.",
    "    fish = scene.get_pass(0)",
    "    fish.set_clear_color([0.0, 0.0, 0.0, 0.0])",
    "",
    "    present = Shader([*LIBRARY, PRESENT_SOURCE])",
    ...setCalls("python", values),
    '    scene.add_pass(Pass.from_shader("present", present))',
    "",
    "    return SimpleNamespace(",
    "        scene=scene,",
    "        present=present,",
    "        camera=camera,",
    "        key=key,",
    "        rim=rim,",
    "        fish=fish,",
    "        image=None,",
    "        depth=None,",
    "        size=None,",
    "        yaw=1.0,",
    "        pitch=0.25,",
    "        distance=DISTANCE,",
    "        turning=True,",
    "        last=None,",
    "    )",
    "",
    "",
    "def frame(state, renderer, target, time, size, pointer, gesture=None):",
    "    gesture = gesture or SimpleNamespace(zoom=0.0, pan=[0.0, 0.0])",
    "    if state.size != list(size):",
    "        fit(state, renderer, size)",
    "    delta = 0.0 if state.last is None else min(max(time - state.last, 0.0), 0.1)",
    "    state.last = time",
    "",
    "    # The fish turns by itself until the first drag, scroll, or pinch.",
    "    if gesture.zoom or gesture.pan[0] or gesture.pan[1]:",
    "        state.turning = False",
    "    if state.turning:",
    "        state.yaw += delta * SPIN",
    "    # A drag the height of the canvas turns the camera half a circle.",
    "    state.yaw -= gesture.pan[0] / size[1] * math.pi",
    "    state.pitch = min(max(state.pitch + gesture.pan[1] / size[1] * math.pi, -1.4), 1.4)",
    "    state.distance = min(",
    "        max(state.distance / 2**gesture.zoom, DISTANCE / 3), DISTANCE * 3",
    "    )",
    "",
    "    reach = state.distance * math.cos(state.pitch)",
    "    eye = [",
    "        CENTER[0] + reach * math.sin(state.yaw),",
    "        CENTER[1] + state.distance * math.sin(state.pitch),",
    "        CENTER[2] + reach * math.cos(state.yaw),",
    "    ]",
    "    aspect = max(size[0], 1) / max(size[1], 1)",
    "    state.camera.set_aspect(aspect).look_at(eye, CENTER, [0.0, 1.0, 0.0])",
    "    # Both lights turn with the camera, so the side in view stays lit.",
    "    state.key.set_direction(turn(KEY, state.yaw))",
    "    state.rim.set_direction(turn(RIM, state.yaw))",
    "    renderer.render(state.scene, target)",
    "",
    "",
    "# The fish draws into a 16-bit float image, so light brighter than white reaches",
    "# the tone map, and a depth buffer keeps its far side behind its near side.",
    "# Neither can change size, so a new canvas size gets new ones.",
    "def fit(state, renderer, size):",
    "    pixels = [max(int(size[0]), 1), max(int(size[1]), 1)]",
    "    usage = TextureUsage.RENDER_ATTACHMENT | TextureUsage.TEXTURE_BINDING",
    "    image = renderer.create_storage_texture(",
    "        pixels, TextureFormat.Rgba16Float, usage=usage",
    "    )",
    "    depth = renderer.create_depth_texture(pixels)",
    "",
    "    # The renderer keeps every texture until it is unregistered.",
    "    for old in (state.image, state.depth):",
    "        if old is not None:",
    "            renderer.unregister_texture(old.id())",
    "    state.fish.set_target(image)",
    "    state.fish.set_depth_target(depth)",
    '    state.present.set("image", image)',
    "    state.image, state.depth, state.size = image, depth, list(size)",
    "",
    "",
    "# Turns a direction around the vertical axis by yaw radians.",
    "def turn(direction, yaw):",
    "    x, y, z = direction",
    "    c, s = math.cos(yaw), math.sin(yaw)",
    "    return [x * c + z * s, y, z * c - x * s]",
    "",
  ].join("\n");
}

function writeSwift({ library, values }) {
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
    `/// ${CREDIT}`,
    'let modelName = "BarramundiFish"',
    "/// The middle of the fish and how far the camera starts from it, in meters.",
    "let center: [Float] = [0, 0.143, 0]",
    "let startDistance: Float = 1.3",
    "/// The key light shines from above and left of the camera, the rim light from behind the fish.",
    "let keyDirection: [Float] = [-0.5, -0.6, -1]",
    "let rimDirection: [Float] = [0.6, 0.2, 1]",
    "/// How fast the fish turns before the first drag, in radians per second.",
    "let spin: Float = 0.4",
    "",
    "/// What the wheel, a drag and a pinch did since the last frame: zoom in doublings",
    "/// (positive zooms in) and pan in view pixels.",
    "struct Gesture {",
    "    var zoom: Float = 0",
    "    var pan: [Float] = [0, 0]",
    "}",
    "",
    "struct State {",
    "    let scene: Scene",
    "    let present: Shader",
    "    let camera: Camera",
    "    let key: Light",
    "    let rim: Light",
    "    let fish: Pass",
    "    var image: Texture?",
    "    var depth: Texture?",
    "    var size: [Float]?",
    "    var yaw: Float = 1",
    "    var pitch: Float = 0.25",
    "    var distance = startDistance",
    "    var turning = true",
    "    var last: Float?",
    "}",
    "",
    "// Await setup once, then await frame from a task your MTKView or CADisplayLink callback starts.",
    "func setup(renderer: Renderer) async throws -> State {",
    '    let modelURL = Bundle.main.url(forResource: modelName, withExtension: "glb")!',
    "    let scene = try Scene.load(modelURL.path)",
    "    // Upload the textures now, so the first frame only draws.",
    "    try await renderer.load(scene)",
    "",
    "    let camera = try Camera.perspective(0.6, 1, 0.01, 10)",
    "    let key = try Light.directional(keyDirection, [1, 0.96, 0.9]).setIntensity(3)",
    "    let rim = try Light.directional(rimDirection, [0.6, 0.7, 1]).setIntensity(1)",
    "    try scene.add(camera).add(key).add(rim).ambient([0.1, 0.1, 0.12])",
    "",
    "    // The fish pass clears to transparent, so present.wgsl can lay the fish over the water.",
    "    // The first frame gives it an image and a depth buffer the size of the view.",
    "    let fish = scene.getPass(0)!",
    "    try fish.setClearColor([0, 0, 0, 0])",
    "",
    '    let presentURL = Bundle.main.url(forResource: "present", withExtension: "wgsl")!',
    "    let present = try Shader(library + [String(contentsOf: presentURL, encoding: .utf8)])",
    ...setCalls("swift", values, { before: "try " }),
    '    scene.addPass(Pass.fromShader("present", present))',
    "",
    "    return State(scene: scene, present: present, camera: camera, key: key, rim: rim, fish: fish)",
    "}",
    "",
    "func frame(",
    "    state: inout State,",
    "    renderer: Renderer,",
    "    target: WindowTarget,",
    "    time: Float,",
    "    size: [Float],",
    "    pointer: [Float],",
    "    gesture: Gesture = Gesture()",
    ") async throws {",
    "    if state.size != size {",
    "        try await fit(&state, renderer, size)",
    "    }",
    "    let delta = state.last.map { min(max(time - $0, 0), 0.1) } ?? 0",
    "    state.last = time",
    "",
    "    // The fish turns by itself until the first drag, scroll, or pinch.",
    "    if gesture.zoom != 0 || gesture.pan != [0, 0] {",
    "        state.turning = false",
    "    }",
    "    if state.turning {",
    "        state.yaw += delta * spin",
    "    }",
    "    // A drag the height of the view turns the camera half a circle.",
    "    state.yaw -= gesture.pan[0] / size[1] * .pi",
    "    state.pitch = min(max(state.pitch + gesture.pan[1] / size[1] * .pi, -1.4), 1.4)",
    "    let zoomed = state.distance / pow(2, gesture.zoom)",
    "    state.distance = min(max(zoomed, startDistance / 3), startDistance * 3)",
    "",
    "    let reach = state.distance * cos(state.pitch)",
    "    let eye = [",
    "        center[0] + reach * sin(state.yaw),",
    "        center[1] + state.distance * sin(state.pitch),",
    "        center[2] + reach * cos(state.yaw),",
    "    ]",
    "    let aspect = max(size[0], 1) / max(size[1], 1)",
    "    _ = try state.camera.setAspect(aspect).lookAt(eye, center, [0, 1, 0])",
    "    // Both lights turn with the camera, so the side in view stays lit.",
    "    _ = try state.key.setDirection(turn(keyDirection, state.yaw))",
    "    _ = try state.rim.setDirection(turn(rimDirection, state.yaw))",
    "    try renderer.render(state.scene, target)",
    "}",
    "",
    "/// The fish draws into a 16-bit float image, so light brighter than white reaches",
    "/// the tone map, and a depth buffer keeps its far side behind its near side.",
    "/// Neither can change size, so a new view size gets new ones.",
    "func fit(_ state: inout State, _ renderer: Renderer, _ size: [Float]) async throws {",
    "    let pixels = [UInt32(max(size[0], 1)), UInt32(max(size[1], 1))]",
    "    let usage = TextureUsage.RENDER_ATTACHMENT | TextureUsage.TEXTURE_BINDING",
    "    let image = try await renderer.createStorageTexture(pixels, .rgba16Float, nil, usage)",
    "    let depth = try await renderer.createDepthTexture(pixels)",
    "",
    "    // The renderer keeps every texture until it is unregistered.",
    "    for old in [state.image, state.depth].compactMap({ $0 }) {",
    "        try renderer.unregisterTexture(old.id())",
    "    }",
    "    try state.fish.setTarget(image)",
    "    try state.fish.setDepthTarget(depth)",
    '    try state.present.set("image", image)',
    "    state.image = image",
    "    state.depth = depth",
    "    state.size = size",
    "}",
    "",
    "/// Turns a direction around the vertical axis by `yaw` radians.",
    "func turn(_ direction: [Float], _ yaw: Float) -> [Float] {",
    "    let (x, y, z) = (direction[0], direction[1], direction[2])",
    "    return [x * cos(yaw) + z * sin(yaw), y, z * cos(yaw) - x * sin(yaw)]",
    "}",
    "",
  ].join("\n");
}

function writeKotlin({ library, values }) {
  const kotlinQuote = (text) => quote(text).replace(/\$/g, "\\$");
  const libraryLine = library.length
    ? wrap("", "private val LIBRARY = listOf(", library.map(kotlinQuote), ")", 100)
    : ["private val LIBRARY = emptyList<String>()"];
  return [
    `// ${NEEDS}`,
    "",
    "import android.content.res.AssetManager",
    "import kotlin.math.PI",
    "import kotlin.math.cos",
    "import kotlin.math.pow",
    "import kotlin.math.sin",
    "import org.fragmentcolor.*",
    "",
    ...libraryLine,
    "",
    `/** ${CREDIT} */`,
    'private const val MODEL = "BarramundiFish.glb"',
    "/** The middle of the fish and how far the camera starts from it, in meters. */",
    "private val CENTER = listOf(0.0, 0.143, 0.0)",
    "private const val DISTANCE = 1.3",
    "/** The key light shines from above and left of the camera, the rim light from behind the fish. */",
    "private val KEY = listOf(-0.5, -0.6, -1.0)",
    "private val RIM = listOf(0.6, 0.2, 1.0)",
    "/** How fast the fish turns before the first drag, in radians per second. */",
    "private const val SPIN = 0.4",
    "",
    "/**",
    " * What the wheel, a drag and a pinch did since the last frame: zoom in doublings",
    " * (positive zooms in) and pan in view pixels.",
    " */",
    "data class Gesture(val zoom: Double = 0.0, val pan: List<Double> = listOf(0.0, 0.0))",
    "",
    "class State(",
    "    val scene: Scene,",
    "    val present: Shader,",
    "    val camera: Camera,",
    "    val key: Light,",
    "    val rim: Light,",
    "    val fish: Pass,",
    ") {",
    "    var image: Texture? = null",
    "    var depth: Texture? = null",
    "    var size: List<Float>? = null",
    "    var yaw = 1.0",
    "    var pitch = 0.25",
    "    var distance = DISTANCE",
    "    var turning = true",
    "    var last: Float? = null",
    "}",
    "",
    "// Call setup once, then frame on every frame, from a coroutine on your render thread.",
    "suspend fun setup(renderer: Renderer, assets: AssetManager): State {",
    "    val scene = Scene.load(assets.open(MODEL).use { it.readBytes() })",
    "    // Upload the textures now, so the first frame only draws.",
    "    renderer.load(scene)",
    "",
    "    val camera = Camera.perspective(0.6, 1.0, 0.01, 10.0)",
    "    val key = Light.directional(KEY, listOf(1.0, 0.96, 0.9)).setIntensity(3.0)",
    "    val rim = Light.directional(RIM, listOf(0.6, 0.7, 1.0)).setIntensity(1.0)",
    "    scene.add(camera).add(key).add(rim).ambient(listOf(0.1, 0.1, 0.12))",
    "",
    "    // The fish pass clears to transparent, so present.wgsl can lay the fish over the water.",
    "    // The first frame gives it an image and a depth buffer the size of the view.",
    "    val fish = scene.getPass(0u)!!",
    "    fish.setClearColor(listOf(0f, 0f, 0f, 0f))",
    "",
    '    val presentSource = assets.open("present.wgsl").bufferedReader().use { it.readText() }',
    "    val present = Shader(LIBRARY + presentSource)",
    ...setCalls("kotlin", values),
    '    scene.addPass(Pass.fromShader("present", present))',
    "",
    "    return State(scene, present, camera, key, rim, fish)",
    "}",
    "",
    "suspend fun frame(",
    "    state: State,",
    "    renderer: Renderer,",
    "    target: WindowTarget,",
    "    time: Float,",
    "    size: List<Float>,",
    "    pointer: List<Float>,",
    "    gesture: Gesture = Gesture(),",
    ") {",
    "    if (state.size != size) fit(state, renderer, size)",
    "    val delta = state.last?.let { (time - it).coerceIn(0f, 0.1f).toDouble() } ?: 0.0",
    "    state.last = time",
    "",
    "    // The fish turns by itself until the first drag, scroll, or pinch.",
    "    if (gesture.zoom != 0.0 || gesture.pan != listOf(0.0, 0.0)) state.turning = false",
    "    if (state.turning) state.yaw += delta * SPIN",
    "    // A drag the height of the view turns the camera half a circle.",
    "    val height = size[1].toDouble()",
    "    state.yaw -= gesture.pan[0] / height * PI",
    "    state.pitch = (state.pitch + gesture.pan[1] / height * PI).coerceIn(-1.4, 1.4)",
    "    state.distance = (state.distance / 2.0.pow(gesture.zoom)).coerceIn(DISTANCE / 3, DISTANCE * 3)",
    "",
    "    val reach = state.distance * cos(state.pitch)",
    "    val eye = listOf(",
    "        CENTER[0] + reach * sin(state.yaw),",
    "        CENTER[1] + state.distance * sin(state.pitch),",
    "        CENTER[2] + reach * cos(state.yaw),",
    "    )",
    "    val aspect = maxOf(size[0].toDouble(), 1.0) / maxOf(height, 1.0)",
    "    state.camera.setAspect(aspect).lookAt(eye, CENTER, listOf(0.0, 1.0, 0.0))",
    "    // Both lights turn with the camera, so the side in view stays lit.",
    "    state.key.setDirection(turn(KEY, state.yaw))",
    "    state.rim.setDirection(turn(RIM, state.yaw))",
    "    renderer.render(state.scene, target)",
    "}",
    "",
    "/**",
    " * The fish draws into a 16-bit float image, so light brighter than white reaches",
    " * the tone map, and a depth buffer keeps its far side behind its near side.",
    " * Neither can change size, so a new view size gets new ones.",
    " */",
    "suspend fun fit(state: State, renderer: Renderer, size: List<Float>) {",
    "    val pixels = listOf(maxOf(size[0].toUInt(), 1u), maxOf(size[1].toUInt(), 1u))",
    "    val usage = TextureUsage.RENDER_ATTACHMENT or TextureUsage.TEXTURE_BINDING",
    "    val image = renderer.createStorageTexture(pixels, TextureFormat.RGBA16_FLOAT, usage = usage)",
    "    val depth = renderer.createDepthTexture(pixels)",
    "",
    "    // The renderer keeps every texture until it is unregistered.",
    "    listOfNotNull(state.image, state.depth).forEach { renderer.unregisterTexture(it.id()) }",
    "    state.fish.setTarget(image)",
    "    state.fish.setDepthTarget(depth)",
    '    state.present.set("image", image)',
    "    state.image = image",
    "    state.depth = depth",
    "    state.size = size",
    "}",
    "",
    "/** Turns a direction around the vertical axis by [yaw] radians. */",
    "fun turn(direction: List<Double>, yaw: Double): List<Double> {",
    "    val (x, y, z) = direction",
    "    return listOf(x * cos(yaw) + z * sin(yaw), y, z * cos(yaw) - x * sin(yaw))",
    "}",
    "",
  ].join("\n");
}

export default {
  id: "model",
  title: "glTF Model",
  summary:
    "Barramundi Fish by Microsoft (CC0), loaded from a glTF file and lit by two lights. Drag to turn it, and scroll or pinch to move closer.",
  category: "3D",
  thumbnail: "/playground/thumbs/model.webp",
  kind: "scene",
  needs: RELEASE,
  gestures: true,
  library: ["color/tonemap_aces", "color/linear_to_srgb"],
  values: VALUES,
  files: { "present.wgsl": PRESENT },
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
