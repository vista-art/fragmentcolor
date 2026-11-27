// A glTF fish loaded with the scene layer, lit by two lights, in three styles.
//
// The fish draws in PBR as the file describes it, and each part of it gets a toon
// and an unlit Model over the same mesh, of which only the current style's show.
// A drag turns the fish, and the wheel or a pinch moves the camera closer. The scene
// draws into a float image with a depth buffer, and a present pass turns its light
// into screen colors over a water gradient, paints and inks the toon fish, and names
// the style in a corner.

import { formatValue } from "../codegen.js";

const PRESENT = /* wgsl */ `// Turns the light values the fish pass drew into colors for the screen, over
// water that brightens toward the surface. Each style maps light to color its
// own way, the toon fish takes flat paint and ink outlines, and the style's name
// sits in the top right corner.

struct Look {
  exposure: f32, // 0.25..4
  surface_color: vec3<f32>,
  deep_color: vec3<f32>,
  // The style the fish draws in, which main.js sets every frame: 0 is PBR, 1 toon and 2 unlit.
  style: u32,
}

@group(0) @binding(0) var<uniform> look: Look;
@group(0) @binding(1) var image: texture_2d<f32>;
@group(0) @binding(2) var image_sampler: sampler;
// The fish pass's depth buffer, which the ink reads one texel at a time. WebGL2
// reads it as zeros, so there the ink follows the silhouette alone.
@group(0) @binding(3) var depth: texture_2d<f32>;

// The camera's near and far planes in main.js, which turn a depth back into a distance.
const NEAR = 0.01;
const FAR = 10.0;
// The color of the toon fish's ink, and how much farther than the surface
// around it a pixel's neighbors must be, as a fraction of its distance, for an
// edge, as where a fin passes in front of the body.
const INK = vec3<f32>(0.02, 0.03, 0.07);
const JUMP = 0.002;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  // The fish pass clears to transparent, so alpha marks where the fish is.
  let uv = vec2<f32>(in.uv.x, 1.0 - in.uv.y);
  let fish = textureSample(image, image_sampler, uv);
  let size = vec2<f32>(textureDimensions(image));

  // Light falls in from above and fades with depth.
  let above = in.uv - vec2<f32>(0.5, 1.15);
  let glow = exp(-6.0 * dot(above, above)) * 0.35;
  let water = mix(look.deep_color, look.surface_color, smoothstep(0.0, 1.0, in.uv.y)) + look.surface_color * glow;

  var color = mix(water, shade(fish.rgb), fish.a);
  if (look.style == 1u) {
    // The toon fish's strokes and lines widen with the canvas.
    let pixel = vec2<i32>(uv * size);
    let reach = i32(max(round(min(size.x, size.y) / 320.0), 1.0));
    let last = vec2<i32>(size) - 1;
    if (fish.a > 0.0) {
      color = mix(water, shade(flatten(pixel, reach, last)), fish.a);
    }
    color = mix(color, INK, ink(pixel, reach, last));
  }
  return vec4<f32>(label(in.position.xy, size, color), 1.0);
}

// The screen color for a light value. PBR runs it through ACES, a filmic curve
// that rolls highlights off and fades strong colors toward white. Toon scales
// its flat colors straight to the screen, a third brighter to match the PBR
// fish's mid tones, with half again the saturation. Unlit shows the texture as it is.
fn shade(light: vec3<f32>) -> vec3<f32> {
  if (look.style == 1u) {
    let color = clamp(light * look.exposure * 1.3, vec3<f32>(0.0), vec3<f32>(1.0));
    return linear_to_srgb(clamp(mix(vec3<f32>(brightness(color)), color, 1.5), vec3<f32>(0.0), vec3<f32>(1.0)));
  }
  if (look.style == 2u) {
    return linear_to_srgb(clamp(light, vec3<f32>(0.0), vec3<f32>(1.0)));
  }
  return linear_to_srgb(tonemap_aces(light * look.exposure));
}

// How bright a linear color looks, with each channel weighed as the eye weighs it.
fn brightness(color: vec3<f32>) -> f32 {
  return dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
}

// The toon fish's light at \`pixel\`, with the texture's fine grain painted flat.
//
// A Kuwahara filter splits the square around the pixel into four quarters that
// share the pixel, and takes the mean color of the quarter whose colors vary least.
// A quarter that crosses an edge varies most, so the band edges and the glint stay sharp.
fn flatten(pixel: vec2<i32>, reach: i32, last: vec2<i32>) -> vec3<f32> {
  var sums = array<vec3<f32>, 4>(vec3<f32>(0.0), vec3<f32>(0.0), vec3<f32>(0.0), vec3<f32>(0.0));
  var squared = array<vec3<f32>, 4>(vec3<f32>(0.0), vec3<f32>(0.0), vec3<f32>(0.0), vec3<f32>(0.0));
  var counts = array<f32, 4>(0.0, 0.0, 0.0, 0.0);
  for (var y = -3; y <= 3; y++) {
    for (var x = -3; x <= 3; x++) {
      let texel = textureLoad(image, clamp(pixel + vec2<i32>(x, y) * reach, vec2<i32>(0), last), 0);
      if (texel.a < 0.5) {
        continue;
      }
      // Quarters 0 to 3 lie up and left, up and right, down and left, down and right.
      for (var quarter = 0; quarter < 4; quarter++) {
        let right = (quarter & 1) == 1;
        let down = quarter >= 2;
        if ((x <= 0 || right) && (x >= 0 || !right) && (y <= 0 || down) && (y >= 0 || !down)) {
          sums[quarter] += texel.rgb;
          squared[quarter] += texel.rgb * texel.rgb;
          counts[quarter] += 1.0;
        }
      }
    }
  }
  var best = textureLoad(image, pixel, 0).rgb;
  var least = 1e9;
  for (var quarter = 0; quarter < 4; quarter++) {
    if (counts[quarter] < 1.0) {
      continue;
    }
    let mean = sums[quarter] / counts[quarter];
    let spread = squared[quarter] / counts[quarter] - mean * mean;
    let variance = spread.r + spread.g + spread.b;
    if (variance < least) {
      least = variance;
      best = mean;
    }
  }
  return best;
}

// How much ink the toon fish takes at \`pixel\`, 0 or 1: along its silhouette,
// where its coverage changes, and on the near side of an edge, where the
// neighbors on a line through the pixel lie farther than a flat surface would put them.
fn ink(pixel: vec2<i32>, reach: i32, last: vec2<i32>) -> f32 {
  let cover = textureLoad(image, pixel, 0).a;
  let here = distance_at(pixel);
  // Across, down and both diagonals.
  var lines = array<vec2<i32>, 4>(vec2<i32>(1, 0), vec2<i32>(0, 1), vec2<i32>(1, 1), vec2<i32>(1, -1));
  var amount = 0.0;
  for (var index = 0; index < 4; index++) {
    let ahead = clamp(pixel + lines[index] * reach, vec2<i32>(0), last);
    let behind = clamp(pixel - lines[index] * reach, vec2<i32>(0), last);
    amount = max(amount, abs(textureLoad(image, ahead, 0).a - cover));
    amount = max(amount, abs(textureLoad(image, behind, 0).a - cover));
    if (distance_at(ahead) + distance_at(behind) - 2.0 * here > JUMP * here) {
      amount = 1.0;
    }
  }
  return amount;
}

// The distance along the view axis from the camera to the fish at \`pixel\`.
// A pixel the fish doesn't cover keeps the cleared depth, 1, and reads as the far plane.
fn distance_at(pixel: vec2<i32>) -> f32 {
  let stored = textureLoad(depth, pixel, 0).r;
  return NEAR * FAR / (FAR - stored * (FAR - NEAR));
}

// The letters of the label, five font pixels square, a bit for each font pixel,
// row after row from the top left.
const B = 0x0f8be2fu;
const I = 0x1f2109fu;
const L = 0x1f08421u;
const N = 0x11cd671u;
const O = 0x0e8c62eu;
const P = 0x010be2fu;
const R = 0x114be2fu;
const T = 0x042109fu;
const U = 0x0e8c631u;

// \`under\` with the style's name over a dark box in the top right corner of a
// canvas \`size\` pixels across, at the canvas pixel \`position\`. A font pixel
// covers a few canvas pixels.
fn label(position: vec2<f32>, size: vec2<f32>, under: vec3<f32>) -> vec3<f32> {
  // Each style's name in five letters, with 0 past its last letter.
  var names = array<u32, 15>(
    P, B, R, 0u, 0u,
    T, O, O, N, 0u,
    U, N, L, I, T,
  );
  let first = min(look.style, 2u) * 5u;
  var count = 0u;
  for (var index = 0u; index < 5u; index++) {
    count += select(0u, 1u, names[first + index] != 0u);
  }

  // The text in font pixels: five for each letter and one between letters, four
  // in from the corner. The box reaches two past it, with rounded corners.
  let pixel = max(floor(min(size.x, size.y) / 160.0), 2.0);
  let text = vec2<f32>(f32(count * 6u - 1u), 5.0);
  let spot = position / pixel - vec2<f32>(size.x / pixel - text.x - 4.0, 4.0);
  let half = text * 0.5;
  let outside = abs(spot - half) - half - vec2<f32>(1.0);
  let edge = (length(max(outside, vec2<f32>(0.0))) - 1.0) * pixel;
  if (edge > 1.0) {
    return under;
  }
  let box = mix(under, vec3<f32>(0.02, 0.025, 0.035), 0.6 * clamp(0.5 - edge, 0.0, 1.0));

  let cell = vec2<i32>(floor(spot));
  if (any(cell < vec2<i32>(0)) || cell.y > 4 || cell.x >= i32(text.x) || cell.x % 6 == 5) {
    return box;
  }
  let bit = (names[first + u32(cell.x / 6)] >> u32(cell.y * 5 + cell.x % 6)) & 1u;
  return select(box, vec3<f32>(0.92, 0.93, 0.95), bit == 1u);
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

const PROGRAM = /* js */ `import { Camera, Light, Model, Pass, PbrMaterial, Scene, Shader, TextureFormat, TextureUsage, ToonMaterial, UnlitMaterial } from "fragmentcolor";
import PRESENT_SOURCE from "./present.wgsl?raw";

const LIBRARY = ["color/tonemap_aces", "color/linear_to_srgb"];

// Barramundi Fish by Microsoft, CC0, from the Khronos glTF sample assets.
const MODEL = "/playground/models/BarramundiFish.glb";
// The middle of the fish and how far the camera starts from it, in meters.
const CENTER = [0, 0.143, 0];
const DISTANCE = 1.3;
// The key light shines on the fish from over the camera's right shoulder, the rim
// light from behind the fish. The toon fish takes its key light from overhead, which
// lays its shadow band along the belly, and more ambient light, since a cartoon
// shades with a darker tone of the same color. Unlit takes no light at all. KEY and
// AMBIENT list the styles in turn: PBR, toon, unlit.
const KEY = [[-0.5, -0.6, -1], [-0.3, -1, -0.35], [-0.5, -0.6, -1]];
const RIM = [0.6, 0.2, 1];
const AMBIENT = [[0.1, 0.1, 0.12], [0.3, 0.32, 0.36], [0.1, 0.1, 0.12]];
// How fast the fish turns before the first drag, in radians per second.
const SPIN = 0.4;
// How long each style shows before the next one takes over, in seconds, until the first M.
const HOLD = 4;

export async function setup(renderer, target) {
  const response = await fetch(MODEL);
  const scene = Scene.load(new Uint8Array(await response.arrayBuffer()));
  // Upload the textures now, so the first frame only draws.
  await renderer.load(scene);

  // Each part of the fish draws in PBR as the file describes it, and gets a toon and
  // an unlit Model over its mesh. Both sample its base color texture, which stays one
  // upload. The lists follow the styles in turn, and so does present.wgsl's look.style.
  const styles = [[], [], []];
  for (const model of scene.models()) {
    const pbr = model.material();
    if (!(pbr instanceof PbrMaterial)) continue;
    // Light in three flat steps, a hard white glint, and a pale rim along the edges.
    const toon = new ToonMaterial()
      .bands(3)
      .highlight([1, 1, 1])
      .highlightSize(0.012)
      .rim([0.2, 0.35, 0.4])
      .rimWidth(0.3);
    toon.baseColorMap().texture(pbr.baseColorMap());
    const unlit = new UnlitMaterial();
    unlit.baseColorMap().texture(pbr.baseColorMap());
    styles[0].push(model);
    styles[1].push(restyle(scene, model, toon));
    styles[2].push(restyle(scene, model, unlit));
  }

  const camera = Camera.perspective(0.6, 1, 0.01, 10);
  const key = Light.directional(KEY[0], [1, 0.96, 0.9]).setIntensity(3);
  const rim = Light.directional(RIM, [0.6, 0.7, 1]).setIntensity(1);
  scene.add(camera).add(key).add(rim).ambient(AMBIENT[0]);

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
    styles,
    style: 0,
    shown: 0,
    cycling: true,
    held: false,
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

export function frame(state, renderer, target, time, size, pointer, gesture = { zoom: 0, pan: [0, 0] }, keys = { down: new Set() }) {
  if (size[0] !== state.wanted[0] || size[1] !== state.wanted[1]) fit(state, renderer, size);
  const delta = state.last === null ? 0 : Math.min(Math.max(time - state.last, 0), 0.1);
  state.last = time;

  // M, or the Style button on a touch screen, shows the next style. Until the
  // first press, the styles take turns by themselves.
  const held = keys.down.has("KeyM");
  const pressed = held && !state.held;
  state.held = held;
  if (pressed) state.cycling = false;
  state.shown += delta;
  if (pressed || (state.cycling && state.shown >= HOLD)) show(state, (state.style + 1) % state.styles.length);
  state.present.set("look.style", state.style);

  // The fish turns by itself until the first drag, scroll, or pinch.
  if (gesture.zoom || gesture.pan[0] || gesture.pan[1]) state.turning = false;
  if (state.turning) state.yaw += delta * SPIN;
  // A drag the height of the canvas turns the camera half a circle.
  const height = Math.max(size[1], 1);
  state.yaw -= (gesture.pan[0] / height) * Math.PI;
  state.pitch = clamp(state.pitch + (gesture.pan[1] / height) * Math.PI, -1.4, 1.4);
  state.distance = clamp(state.distance / 2 ** gesture.zoom, DISTANCE / 3, DISTANCE * 3);

  const reach = state.distance * Math.cos(state.pitch);
  const eye = [
    CENTER[0] + reach * Math.sin(state.yaw),
    CENTER[1] + state.distance * Math.sin(state.pitch),
    CENTER[2] + reach * Math.cos(state.yaw),
  ];
  state.camera.setAspect(state.size[0] / state.size[1]).lookAt(eye, CENTER, [0, 1, 0]);
  // Both lights turn with the camera, so the side in view stays lit.
  state.key.setDirection(turn(KEY[state.style], state.yaw));
  state.rim.setDirection(turn(RIM, state.yaw));
  renderer.render(state.scene, target);
}

// A hidden Model that draws the mesh of \`model\` with \`material\`, where \`model\` is.
function restyle(scene, model, material) {
  const copy = new Model(model.mesh(), material);
  copy.setTransform(model.transform());
  copy.setVisible(false);
  scene.add(copy);
  return copy;
}

// Shows the Models of one style and hides the rest, which then cost no draw call,
// and sets the style's ambient light.
function show(state, style) {
  state.styles.forEach((models, index) => {
    for (const model of models) model.setVisible(index === style);
  });
  state.scene.ambient(AMBIENT[style]);
  state.style = style;
  state.shown = 0;
}

// The fish draws into a 16-bit float image, so light brighter than white reaches
// the tone map, and a depth texture keeps its far side behind its near side.
// present.wgsl reads both. A new canvas size gets a new pair, and the fish keeps
// drawing at the old size until they are ready.
async function fit(state, renderer, size) {
  state.wanted = size;
  const usage = TextureUsage.RENDER_ATTACHMENT | TextureUsage.TEXTURE_BINDING;
  const image = await renderer.createStorageTexture(size, TextureFormat.Rgba16Float, null, usage);
  const depth = await renderer.createDepthTexture(size);
  // When a newer size came in during the wait, these two are already out of date.
  // Unregistering frees them now, where a dropped handle waits for the collector.
  if (state.wanted !== size) {
    renderer.unregisterTexture(image.id());
    renderer.unregisterTexture(depth.id());
    return;
  }
  for (const old of [state.image, state.depth]) if (old) renderer.unregisterTexture(old.id());
  state.fish.setTarget(image);
  state.fish.setDepthTarget(depth);
  state.present.set("image", image);
  state.present.set("depth", depth);
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

// The toon and unlit fish need ToonMaterial, UnlitMaterial and a material map that takes
// another map's texture, which arrive in 0.12.8. The gate keeps the demo out of the picker
// on an older site build, and each tab's first line names it.
const RELEASE = "0.12.8";
const NEEDS = `Needs FragmentColor ${RELEASE} or later.`;

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
    `// ${NEEDS}`,
    "",
    "use std::collections::HashSet;",
    "use std::error::Error;",
    "use std::f32::consts::PI;",
    "",
    "use fragmentcolor::{",
    "    Camera, Light, Material, Model, Pass, Renderer, Scene, Shader, Target, Texture, TextureData,",
    "    TextureFormat, TextureInput, TextureOptions, TextureUsage, ToonMaterial, UnlitMaterial,",
    "};",
    "",
    'const PRESENT_SOURCE: &str = include_str!("present.wgsl");',
    "",
    `/// ${CREDIT}`,
    'const MODEL: &str = "BarramundiFish.glb";',
    "/// The middle of the fish and how far the camera starts from it, in meters.",
    "const CENTER: [f32; 3] = [0.0, 0.143, 0.0];",
    "const DISTANCE: f32 = 1.3;",
    "/// The key light shines on the fish from over the camera's right shoulder, the rim",
    "/// light from behind the fish. The toon fish takes its key light from overhead, which",
    "/// lays its shadow band along the belly, and more ambient light, since a cartoon",
    "/// shades with a darker tone of the same color. Unlit takes no light at all. `KEY` and",
    "/// `AMBIENT` list the styles in turn: PBR, toon, unlit.",
    "const KEY: [[f32; 3]; 3] = [[-0.5, -0.6, -1.0], [-0.3, -1.0, -0.35], [-0.5, -0.6, -1.0]];",
    "const RIM: [f32; 3] = [0.6, 0.2, 1.0];",
    "const AMBIENT: [[f32; 3]; 3] = [[0.1, 0.1, 0.12], [0.3, 0.32, 0.36], [0.1, 0.1, 0.12]];",
    "/// How fast the fish turns before the first drag, in radians per second.",
    "const SPIN: f32 = 0.4;",
    "/// How long each style shows before the next one takes over, in seconds, until the first M.",
    "const HOLD: f32 = 4.0;",
    "",
    "/// What the wheel, a drag and a pinch did since the last frame: zoom in doublings",
    "/// (positive zooms in) and pan in window pixels.",
    "#[derive(Clone, Copy, Default)]",
    "struct Gesture {",
    "    zoom: f32,",
    "    pan: [f32; 2],",
    "}",
    "",
    "/// The keys held down, named the way `KeyboardEvent.code` names them, such as \"KeyM\".",
    "#[derive(Default)]",
    "struct Keys {",
    "    down: HashSet<String>,",
    "}",
    "",
    "struct State {",
    "    scene: Scene,",
    "    present: Shader,",
    "    camera: Camera,",
    "    key: Light,",
    "    rim: Light,",
    "    fish: Pass,",
    "    /// The Models of each style, of which only `style`'s show.",
    "    styles: [Vec<Model>; 3],",
    "    style: usize,",
    "    /// How long the style has shown, in seconds.",
    "    shown: f32,",
    "    /// Whether the styles still take turns by themselves.",
    "    cycling: bool,",
    "    /// Whether M was down on the last frame.",
    "    held: bool,",
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
    "    // Each part of the fish draws in PBR as the file describes it, and gets a toon and",
    "    // an unlit Model over its mesh. Both sample its base color texture, which stays one",
    "    // upload. The lists follow the styles in turn, and so does present.wgsl's look.style.",
    "    let mut styles: [Vec<Model>; 3] = Default::default();",
    "    for model in scene.models() {",
    "        let Material::Pbr(pbr) = model.material() else {",
    "            continue;",
    "        };",
    "        // Light in three flat steps, a hard white glint, and a pale rim along the edges.",
    "        let toon = ToonMaterial::new()",
    "            .bands(3)?",
    "            .highlight([1.0, 1.0, 1.0])?",
    "            .highlight_size(0.012)?",
    "            .rim([0.2, 0.35, 0.4])?",
    "            .rim_width(0.3)?;",
    "        toon.base_color_map().texture(&pbr.base_color_map())?;",
    "        let unlit = UnlitMaterial::new();",
    "        unlit.base_color_map().texture(&pbr.base_color_map())?;",
    "        styles[1].push(restyle(&scene, &model, toon)?);",
    "        styles[2].push(restyle(&scene, &model, unlit)?);",
    "        styles[0].push(model);",
    "    }",
    "",
    "    let camera = Camera::perspective(0.6, 1.0, 0.01, 10.0)?;",
    "    let key = Light::directional(KEY[0], [1.0, 0.96, 0.9])?.set_intensity(3.0)?;",
    "    let rim = Light::directional(RIM, [0.6, 0.7, 1.0])?.set_intensity(1.0)?;",
    "    scene",
    "        .add(&camera)?",
    "        .add(&key)?",
    "        .add(&rim)?",
    "        .ambient(AMBIENT[0])?;",
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
    "        styles,",
    "        style: 0,",
    "        shown: 0.0,",
    "        cycling: true,",
    "        held: false,",
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
    "#[allow(clippy::too_many_arguments)]",
    "async fn frame(",
    "    state: &mut State,",
    "    renderer: &Renderer,",
    "    target: &impl Target,",
    "    time: f32,",
    "    size: [f32; 2],",
    "    _pointer: [f32; 2],",
    "    gesture: Gesture,",
    "    keys: &Keys,",
    ") -> Result<(), Box<dyn Error>> {",
    "    if state.size != Some(size) {",
    "        fit(state, renderer, size).await?;",
    "    }",
    "    let delta = state.last.map_or(0.0, |last| (time - last).clamp(0.0, 0.1));",
    "    state.last = Some(time);",
    "",
    "    // M, or the Style button on a touch screen, shows the next style. Until the",
    "    // first press, the styles take turns by themselves.",
    '    let held = keys.down.contains("KeyM");',
    "    let pressed = held && !state.held;",
    "    state.held = held;",
    "    if pressed {",
    "        state.cycling = false;",
    "    }",
    "    state.shown += delta;",
    "    if pressed || (state.cycling && state.shown >= HOLD) {",
    "        show(state, (state.style + 1) % state.styles.len())?;",
    "    }",
    '    state.present.set("look.style", state.style as u32)?;',
    "",
    "    // The fish turns by itself until the first drag, scroll, or pinch.",
    "    if gesture.zoom != 0.0 || gesture.pan != [0.0, 0.0] {",
    "        state.turning = false;",
    "    }",
    "    if state.turning {",
    "        state.yaw += delta * SPIN;",
    "    }",
    "    // A drag the height of the window turns the camera half a circle.",
    "    let height = size[1].max(1.0);",
    "    state.yaw -= gesture.pan[0] / height * PI;",
    "    state.pitch = (state.pitch + gesture.pan[1] / height * PI).clamp(-1.4, 1.4);",
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
    "        .set_aspect(size[0].max(1.0) / height)?",
    "        .look_at(eye, CENTER, [0.0, 1.0, 0.0])?;",
    "    // Both lights turn with the camera, so the side in view stays lit.",
    "    state.key.set_direction(turn(KEY[state.style], state.yaw))?;",
    "    state.rim.set_direction(turn(RIM, state.yaw))?;",
    "    renderer.render(&state.scene, target)?;",
    "    Ok(())",
    "}",
    "",
    "/// A hidden Model that draws the mesh of `model` with `material`, where `model` is.",
    "fn restyle(",
    "    scene: &Scene,",
    "    model: &Model,",
    "    material: impl Into<Material>,",
    ") -> Result<Model, Box<dyn Error>> {",
    "    let copy = Model::new(model.mesh().clone(), material);",
    "    copy.set_transform(model.transform())?;",
    "    copy.set_visible(false);",
    "    scene.add(&copy)?;",
    "    Ok(copy)",
    "}",
    "",
    "/// Shows the Models of one style and hides the rest, which then cost no draw call,",
    "/// and sets the style's ambient light.",
    "fn show(state: &mut State, style: usize) -> Result<(), Box<dyn Error>> {",
    "    for (index, models) in state.styles.iter().enumerate() {",
    "        for model in models {",
    "            model.set_visible(index == style);",
    "        }",
    "    }",
    "    state.scene.ambient(AMBIENT[style])?;",
    "    state.style = style;",
    "    state.shown = 0.0;",
    "    Ok(())",
    "}",
    "",
    "/// The fish draws into a 16-bit float image, so light brighter than white reaches",
    "/// the tone map, and a depth texture keeps its far side behind its near side.",
    "/// present.wgsl reads both. A new window size gets a new pair.",
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
    "    // Unregistering frees the old pair now, before its handles go.",
    "    let stale = [state.image.take(), state.depth.take()];",
    "    for old in stale.into_iter().flatten() {",
    "        renderer.unregister_texture(old.id())?;",
    "    }",
    "    state.fish.set_target(&image)?;",
    "    state.fish.set_depth_target(&depth)?;",
    '    state.present.set("image", &image)?;',
    '    state.present.set("depth", &depth)?;',
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
    "    Model,",
    "    Pass,",
    "    PbrMaterial,",
    "    Scene,",
    "    Shader,",
    "    TextureFormat,",
    "    TextureUsage,",
    "    ToonMaterial,",
    "    UnlitMaterial,",
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
    "# The key light shines on the fish from over the camera's right shoulder, the rim",
    "# light from behind the fish. The toon fish takes its key light from overhead, which",
    "# lays its shadow band along the belly, and more ambient light, since a cartoon",
    "# shades with a darker tone of the same color. Unlit takes no light at all. KEY and",
    "# AMBIENT list the styles in turn: PBR, toon, unlit.",
    "KEY = [[-0.5, -0.6, -1.0], [-0.3, -1.0, -0.35], [-0.5, -0.6, -1.0]]",
    "RIM = [0.6, 0.2, 1.0]",
    "AMBIENT = [[0.1, 0.1, 0.12], [0.3, 0.32, 0.36], [0.1, 0.1, 0.12]]",
    "# How fast the fish turns before the first drag, in radians per second.",
    "SPIN = 0.4",
    "# How long each style shows before the next one takes over, in seconds, until the first M.",
    "HOLD = 4.0",
    "",
    "",
    "# Call setup() once, then frame() on every redraw of your canvas. frame's gesture",
    "# is what the wheel, a drag and a pinch did since the last frame: zoom in doublings",
    "# (positive zooms in) and pan in canvas pixels. Its keys hold the keys held down,",
    '# named the way KeyboardEvent.code names them, such as "KeyM".',
    "def setup(renderer):",
    "    scene = Scene.load(str(MODEL))",
    "    # Upload the textures now, so the first frame only draws.",
    "    renderer.load(scene)",
    "",
    "    # Each part of the fish draws in PBR as the file describes it, and gets a toon and",
    "    # an unlit Model over its mesh. Both sample its base color texture, which stays one",
    "    # upload. The lists follow the styles in turn, and so does present.wgsl's look.style.",
    "    styles = [[], [], []]",
    "    for model in scene.models():",
    "        pbr = model.material()",
    "        if not isinstance(pbr, PbrMaterial):",
    "            continue",
    "        # Light in three flat steps, a hard white glint, and a pale rim along the edges.",
    "        toon = (",
    "            ToonMaterial()",
    "            .bands(3)",
    "            .highlight([1.0, 1.0, 1.0])",
    "            .highlight_size(0.012)",
    "            .rim([0.2, 0.35, 0.4])",
    "            .rim_width(0.3)",
    "        )",
    "        toon.base_color_map().texture(pbr.base_color_map())",
    "        unlit = UnlitMaterial()",
    "        unlit.base_color_map().texture(pbr.base_color_map())",
    "        styles[0].append(model)",
    "        styles[1].append(restyle(scene, model, toon))",
    "        styles[2].append(restyle(scene, model, unlit))",
    "",
    "    camera = Camera.perspective(0.6, 1.0, 0.01, 10.0)",
    "    key = Light.directional(KEY[0], [1.0, 0.96, 0.9]).set_intensity(3.0)",
    "    rim = Light.directional(RIM, [0.6, 0.7, 1.0]).set_intensity(1.0)",
    "    scene.add(camera).add(key).add(rim).ambient(AMBIENT[0])",
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
    "        styles=styles,",
    "        style=0,",
    "        shown=0.0,",
    "        cycling=True,",
    "        held=False,",
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
    "def frame(state, renderer, target, time, size, pointer, gesture=None, keys=None):",
    "    gesture = gesture or SimpleNamespace(zoom=0.0, pan=[0.0, 0.0])",
    "    keys = keys or SimpleNamespace(down=set())",
    "    if state.size != list(size):",
    "        fit(state, renderer, size)",
    "    delta = 0.0 if state.last is None else min(max(time - state.last, 0.0), 0.1)",
    "    state.last = time",
    "",
    "    # M, or the Style button on a touch screen, shows the next style. Until the",
    "    # first press, the styles take turns by themselves.",
    '    held = "KeyM" in keys.down',
    "    pressed = held and not state.held",
    "    state.held = held",
    "    if pressed:",
    "        state.cycling = False",
    "    state.shown += delta",
    "    if pressed or (state.cycling and state.shown >= HOLD):",
    "        show(state, (state.style + 1) % len(state.styles))",
    '    state.present.set("look.style", state.style)',
    "",
    "    # The fish turns by itself until the first drag, scroll, or pinch.",
    "    if gesture.zoom or gesture.pan[0] or gesture.pan[1]:",
    "        state.turning = False",
    "    if state.turning:",
    "        state.yaw += delta * SPIN",
    "    # A drag the height of the canvas turns the camera half a circle.",
    "    height = max(size[1], 1)",
    "    state.yaw -= gesture.pan[0] / height * math.pi",
    "    state.pitch = min(max(state.pitch + gesture.pan[1] / height * math.pi, -1.4), 1.4)",
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
    "    aspect = max(size[0], 1) / height",
    "    state.camera.set_aspect(aspect).look_at(eye, CENTER, [0.0, 1.0, 0.0])",
    "    # Both lights turn with the camera, so the side in view stays lit.",
    "    state.key.set_direction(turn(KEY[state.style], state.yaw))",
    "    state.rim.set_direction(turn(RIM, state.yaw))",
    "    renderer.render(state.scene, target)",
    "",
    "",
    "# A hidden Model that draws the mesh of model with material, where model is.",
    "def restyle(scene, model, material):",
    "    copy = Model(model.mesh(), material)",
    "    copy.set_transform(model.transform())",
    "    copy.set_visible(False)",
    "    scene.add(copy)",
    "    return copy",
    "",
    "",
    "# Shows the Models of one style and hides the rest, which then cost no draw call,",
    "# and sets the style's ambient light.",
    "def show(state, style):",
    "    for index, models in enumerate(state.styles):",
    "        for model in models:",
    "            model.set_visible(index == style)",
    "    state.scene.ambient(AMBIENT[style])",
    "    state.style = style",
    "    state.shown = 0.0",
    "",
    "",
    "# The fish draws into a 16-bit float image, so light brighter than white reaches",
    "# the tone map, and a depth texture keeps its far side behind its near side.",
    "# present.wgsl reads both. A new canvas size gets a new pair.",
    "def fit(state, renderer, size):",
    "    pixels = [max(int(size[0]), 1), max(int(size[1]), 1)]",
    "    usage = TextureUsage.RENDER_ATTACHMENT | TextureUsage.TEXTURE_BINDING",
    "    image = renderer.create_storage_texture(",
    "        pixels, TextureFormat.Rgba16Float, usage=usage",
    "    )",
    "    depth = renderer.create_depth_texture(pixels)",
    "",
    "    # Unregistering frees the old pair now, before its handles go.",
    "    for old in (state.image, state.depth):",
    "        if old is not None:",
    "            renderer.unregister_texture(old.id())",
    "    state.fish.set_target(image)",
    "    state.fish.set_depth_target(depth)",
    '    state.present.set("image", image)',
    '    state.present.set("depth", depth)',
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
    "/// The key light shines on the fish from over the camera's right shoulder, the rim",
    "/// light from behind the fish. The toon fish takes its key light from overhead, which",
    "/// lays its shadow band along the belly, and more ambient light, since a cartoon",
    "/// shades with a darker tone of the same color. Unlit takes no light at all. Both",
    "/// lists hold the styles in turn: PBR, toon, unlit.",
    "let keyDirections: [[Float]] = [[-0.5, -0.6, -1], [-0.3, -1, -0.35], [-0.5, -0.6, -1]]",
    "let rimDirection: [Float] = [0.6, 0.2, 1]",
    "let ambientColors: [[Float]] = [[0.1, 0.1, 0.12], [0.3, 0.32, 0.36], [0.1, 0.1, 0.12]]",
    "/// How fast the fish turns before the first drag, in radians per second.",
    "let spin: Float = 0.4",
    "/// How long each style shows before the next one takes over, in seconds, until the first M.",
    "let hold: Float = 4",
    "",
    "/// What the wheel, a drag and a pinch did since the last frame: zoom in doublings",
    "/// (positive zooms in) and pan in view pixels.",
    "struct Gesture {",
    "    var zoom: Float = 0",
    "    var pan: [Float] = [0, 0]",
    "}",
    "",
    "/// The keys held down, named the way `KeyboardEvent.code` names them, such as \"KeyM\".",
    "struct Keys {",
    "    var down: Set<String> = []",
    "}",
    "",
    "struct State {",
    "    let scene: Scene",
    "    let present: Shader",
    "    let camera: Camera",
    "    let key: Light",
    "    let rim: Light",
    "    let fish: Pass",
    "    /// The Models of each style, of which only `style`'s show.",
    "    let styles: [[Model]]",
    "    var style = 0",
    "    /// How long the style has shown, in seconds.",
    "    var shown: Float = 0",
    "    /// Whether the styles still take turns by themselves.",
    "    var cycling = true",
    "    /// Whether M was down on the last frame.",
    "    var held = false",
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
    "    // Each part of the fish draws in PBR as the file describes it, and gets a toon and",
    "    // an unlit Model over its mesh. Both sample its base color texture, which stays one",
    "    // upload. The lists follow the styles in turn, and so does present.wgsl's look.style.",
    "    var styles: [[Model]] = [[], [], []]",
    "    for model in scene.models() {",
    "        guard let pbr = model.material() as? PbrMaterial else {",
    "            continue",
    "        }",
    "        // Light in three flat steps, a hard white glint, and a pale rim along the edges.",
    "        let toon = try ToonMaterial()",
    "            .bands(3)",
    "            .highlight([1, 1, 1])",
    "            .highlightSize(0.012)",
    "            .rim([0.2, 0.35, 0.4])",
    "            .rimWidth(0.3)",
    "        try toon.baseColorMap().texture(pbr.baseColorMap())",
    "        let unlit = UnlitMaterial()",
    "        try unlit.baseColorMap().texture(pbr.baseColorMap())",
    "        styles[0].append(model)",
    "        styles[1].append(try restyle(scene, model, toon))",
    "        styles[2].append(try restyle(scene, model, unlit))",
    "    }",
    "",
    "    let camera = try Camera.perspective(0.6, 1, 0.01, 10)",
    "    let key = try Light.directional(keyDirections[0], [1, 0.96, 0.9]).setIntensity(3.0)",
    "    let rim = try Light.directional(rimDirection, [0.6, 0.7, 1]).setIntensity(1.0)",
    "    try scene.add(camera).add(key).add(rim).ambient(ambientColors[0])",
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
    "    return State(scene: scene, present: present, camera: camera, key: key, rim: rim, fish: fish, styles: styles)",
    "}",
    "",
    "func frame(",
    "    state: inout State,",
    "    renderer: Renderer,",
    "    target: WindowTarget,",
    "    time: Float,",
    "    size: [Float],",
    "    pointer: [Float],",
    "    gesture: Gesture = Gesture(),",
    "    keys: Keys = Keys()",
    ") async throws {",
    "    if state.size != size {",
    "        try await fit(&state, renderer, size)",
    "    }",
    "    let delta = state.last.map { min(max(time - $0, 0), 0.1) } ?? 0",
    "    state.last = time",
    "",
    "    // M, or the Style button on a touch screen, shows the next style. Until the",
    "    // first press, the styles take turns by themselves.",
    '    let held = keys.down.contains("KeyM")',
    "    let pressed = held && !state.held",
    "    state.held = held",
    "    if pressed {",
    "        state.cycling = false",
    "    }",
    "    state.shown += delta",
    "    if pressed || (state.cycling && state.shown >= hold) {",
    "        try show(&state, (state.style + 1) % state.styles.count)",
    "    }",
    '    try state.present.set("look.style", UInt32(state.style))',
    "",
    "    // The fish turns by itself until the first drag, scroll, or pinch.",
    "    if gesture.zoom != 0 || gesture.pan != [0, 0] {",
    "        state.turning = false",
    "    }",
    "    if state.turning {",
    "        state.yaw += delta * spin",
    "    }",
    "    // A drag the height of the view turns the camera half a circle.",
    "    let height = max(size[1], 1)",
    "    state.yaw -= gesture.pan[0] / height * .pi",
    "    state.pitch = min(max(state.pitch + gesture.pan[1] / height * .pi, -1.4), 1.4)",
    "    let zoomed = state.distance / pow(2, gesture.zoom)",
    "    state.distance = min(max(zoomed, startDistance / 3), startDistance * 3)",
    "",
    "    let reach = state.distance * cos(state.pitch)",
    "    let eye = [",
    "        center[0] + reach * sin(state.yaw),",
    "        center[1] + state.distance * sin(state.pitch),",
    "        center[2] + reach * cos(state.yaw),",
    "    ]",
    "    let aspect = max(size[0], 1) / height",
    "    try state.camera.setAspect(aspect).lookAt(eye, center, [0, 1, 0])",
    "    // Both lights turn with the camera, so the side in view stays lit.",
    "    try state.key.setDirection(turn(keyDirections[state.style], state.yaw))",
    "    try state.rim.setDirection(turn(rimDirection, state.yaw))",
    "    try renderer.render(state.scene, target)",
    "}",
    "",
    "/// A hidden Model that draws the mesh of `model` with `material`, where `model` is.",
    "func restyle(_ scene: Scene, _ model: Model, _ material: any Material) throws -> Model {",
    "    let copy = Model(model.mesh(), material)",
    "    try copy.setTransform(model.transform())",
    "    copy.setVisible(false)",
    "    try scene.add(copy)",
    "    return copy",
    "}",
    "",
    "/// Shows the Models of one style and hides the rest, which then cost no draw call,",
    "/// and sets the style's ambient light.",
    "func show(_ state: inout State, _ style: Int) throws {",
    "    for (index, models) in state.styles.enumerated() {",
    "        for model in models {",
    "            model.setVisible(index == style)",
    "        }",
    "    }",
    "    try state.scene.ambient(ambientColors[style])",
    "    state.style = style",
    "    state.shown = 0",
    "}",
    "",
    "/// The fish draws into a 16-bit float image, so light brighter than white reaches",
    "/// the tone map, and a depth texture keeps its far side behind its near side.",
    "/// present.wgsl reads both. A new view size gets a new pair.",
    "func fit(_ state: inout State, _ renderer: Renderer, _ size: [Float]) async throws {",
    "    let pixels = [UInt32(max(size[0], 1)), UInt32(max(size[1], 1))]",
    "    let usage = TextureUsage.RENDER_ATTACHMENT | TextureUsage.TEXTURE_BINDING",
    "    let image = try await renderer.createStorageTexture(pixels, .rgba16Float, nil, usage)",
    "    let depth = try await renderer.createDepthTexture(pixels)",
    "",
    "    // Unregistering frees the old pair now, before its handles go.",
    "    for old in [state.image, state.depth].compactMap({ $0 }) {",
    "        try renderer.unregisterTexture(old.id())",
    "    }",
    "    try state.fish.setTarget(image)",
    "    try state.fish.setDepthTarget(depth)",
    '    try state.present.set("image", image)',
    '    try state.present.set("depth", depth)',
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
    "/**",
    " * The key light shines on the fish from over the camera's right shoulder, the rim",
    " * light from behind the fish. The toon fish takes its key light from overhead, which",
    " * lays its shadow band along the belly, and more ambient light, since a cartoon",
    " * shades with a darker tone of the same color. Unlit takes no light at all. KEY and",
    " * AMBIENT list the styles in turn: PBR, toon, unlit.",
    " */",
    "private val KEY = listOf(listOf(-0.5, -0.6, -1.0), listOf(-0.3, -1.0, -0.35), listOf(-0.5, -0.6, -1.0))",
    "private val RIM = listOf(0.6, 0.2, 1.0)",
    "private val AMBIENT = listOf(listOf(0.1, 0.1, 0.12), listOf(0.3, 0.32, 0.36), listOf(0.1, 0.1, 0.12))",
    "/** How fast the fish turns before the first drag, in radians per second. */",
    "private const val SPIN = 0.4",
    "/** How long each style shows before the next one takes over, in seconds, until the first M. */",
    "private const val HOLD = 4.0",
    "",
    "/**",
    " * What the wheel, a drag and a pinch did since the last frame: zoom in doublings",
    " * (positive zooms in) and pan in view pixels.",
    " */",
    "data class Gesture(val zoom: Double = 0.0, val pan: List<Double> = listOf(0.0, 0.0))",
    "",
    '/** The keys held down, named the way KeyboardEvent.code names them, such as "KeyM". */',
    "data class Keys(val down: Set<String> = emptySet())",
    "",
    "class State(",
    "    val scene: Scene,",
    "    val present: Shader,",
    "    val camera: Camera,",
    "    val key: Light,",
    "    val rim: Light,",
    "    val fish: Pass,",
    "    /** The Models of each style, of which only [style]'s show. */",
    "    val styles: List<List<Model>>,",
    ") {",
    "    var style = 0",
    "    /** How long the style has shown, in seconds. */",
    "    var shown = 0.0",
    "    /** Whether the styles still take turns by themselves. */",
    "    var cycling = true",
    "    /** Whether M was down on the last frame. */",
    "    var held = false",
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
    "    // Each part of the fish draws in PBR as the file describes it, and gets a toon and",
    "    // an unlit Model over its mesh. Both sample its base color texture, which stays one",
    "    // upload. The lists follow the styles in turn, and so does present.wgsl's look.style.",
    "    val styles = List(3) { mutableListOf<Model>() }",
    "    for (model in scene.models()) {",
    "        val pbr = model.material() as? PbrMaterial ?: continue",
    "        // Light in three flat steps, a hard white glint, and a pale rim along the edges.",
    "        val toon = ToonMaterial()",
    "            .bands(3u)",
    "            .highlight(listOf(1.0, 1.0, 1.0))",
    "            .highlightSize(0.012)",
    "            .rim(listOf(0.2, 0.35, 0.4))",
    "            .rimWidth(0.3)",
    "        toon.baseColorMap().texture(pbr.baseColorMap())",
    "        val unlit = UnlitMaterial()",
    "        unlit.baseColorMap().texture(pbr.baseColorMap())",
    "        styles[0].add(model)",
    "        styles[1].add(restyle(scene, model, toon))",
    "        styles[2].add(restyle(scene, model, unlit))",
    "    }",
    "",
    "    val camera = Camera.perspective(0.6, 1.0, 0.01, 10.0)",
    "    val key = Light.directional(KEY[0], listOf(1.0, 0.96, 0.9)).setIntensity(3.0)",
    "    val rim = Light.directional(RIM, listOf(0.6, 0.7, 1.0)).setIntensity(1.0)",
    "    scene.add(camera).add(key).add(rim).ambient(AMBIENT[0])",
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
    "    return State(scene, present, camera, key, rim, fish, styles)",
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
    "    keys: Keys = Keys(),",
    ") {",
    "    if (state.size != size) fit(state, renderer, size)",
    "    val delta = state.last?.let { (time - it).coerceIn(0f, 0.1f).toDouble() } ?: 0.0",
    "    state.last = time",
    "",
    "    // M, or the Style button on a touch screen, shows the next style. Until the",
    "    // first press, the styles take turns by themselves.",
    '    val held = "KeyM" in keys.down',
    "    val pressed = held && !state.held",
    "    state.held = held",
    "    if (pressed) state.cycling = false",
    "    state.shown += delta",
    "    if (pressed || (state.cycling && state.shown >= HOLD)) show(state, (state.style + 1) % state.styles.size)",
    '    state.present.set("look.style", state.style.toUInt())',
    "",
    "    // The fish turns by itself until the first drag, scroll, or pinch.",
    "    if (gesture.zoom != 0.0 || gesture.pan != listOf(0.0, 0.0)) state.turning = false",
    "    if (state.turning) state.yaw += delta * SPIN",
    "    // A drag the height of the view turns the camera half a circle.",
    "    val height = maxOf(size[1].toDouble(), 1.0)",
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
    "    val aspect = maxOf(size[0].toDouble(), 1.0) / height",
    "    state.camera.setAspect(aspect).lookAt(eye, CENTER, listOf(0.0, 1.0, 0.0))",
    "    // Both lights turn with the camera, so the side in view stays lit.",
    "    state.key.setDirection(turn(KEY[state.style], state.yaw))",
    "    state.rim.setDirection(turn(RIM, state.yaw))",
    "    renderer.render(state.scene, target)",
    "}",
    "",
    "/** A hidden Model that draws the mesh of [model] with [material], where [model] is. */",
    "fun restyle(scene: Scene, model: Model, material: Any): Model {",
    "    val copy = Model(model.mesh(), material)",
    "    copy.setTransform(model.transform())",
    "    copy.setVisible(false)",
    "    scene.add(copy)",
    "    return copy",
    "}",
    "",
    "/**",
    " * Shows the Models of one style and hides the rest, which then cost no draw call,",
    " * and sets the style's ambient light.",
    " */",
    "fun show(state: State, style: Int) {",
    "    state.styles.forEachIndexed { index, models ->",
    "        for (model in models) model.setVisible(index == style)",
    "    }",
    "    state.scene.ambient(AMBIENT[style])",
    "    state.style = style",
    "    state.shown = 0.0",
    "}",
    "",
    "/**",
    " * The fish draws into a 16-bit float image, so light brighter than white reaches",
    " * the tone map, and a depth texture keeps its far side behind its near side.",
    " * present.wgsl reads both. A new view size gets a new pair.",
    " */",
    "suspend fun fit(state: State, renderer: Renderer, size: List<Float>) {",
    "    val pixels = listOf(maxOf(size[0].toUInt(), 1u), maxOf(size[1].toUInt(), 1u))",
    "    val usage = TextureUsage.RENDER_ATTACHMENT or TextureUsage.TEXTURE_BINDING",
    "    val image = renderer.createStorageTexture(pixels, TextureFormat.RGBA16_FLOAT, usage = usage)",
    "    val depth = renderer.createDepthTexture(pixels)",
    "",
    "    // Unregistering frees the old pair now, before its handles go.",
    "    listOfNotNull(state.image, state.depth).forEach { renderer.unregisterTexture(it.id()) }",
    "    state.fish.setTarget(image)",
    "    state.fish.setDepthTarget(depth)",
    '    state.present.set("image", image)',
    '    state.present.set("depth", depth)',
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
    "Barramundi Fish by Microsoft (CC0), loaded from a glTF file and lit by two lights, in three styles: PBR as the file describes it, toon with ink outlines, and unlit. The styles take turns until you click the canvas and press M, or tap Style on a touch screen, to switch yourself. Drag to turn the fish, and scroll or pinch to move closer.",
  category: "3D",
  thumbnail: "/playground/thumbs/model.webp",
  chip: { x: 0.12, y: 0.6, zoom: 2.2 },
  kind: "scene",
  needs: RELEASE,
  gestures: true,
  keys: true,
  // A touch screen gets a Style button that holds M, and no stick, since nothing here walks.
  buttons: [{ code: "KeyM", label: "Style" }],
  stick: false,
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
