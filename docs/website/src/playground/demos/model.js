// A glTF fish loaded with the scene layer, lit by two lights, in three styles.
//
// The fish draws in PBR, wetter than its file describes it, and each part of it gets
// a toon and an unlit Model over the same mesh, of which only the current style's show.
// A drag turns the fish, and the wheel or a pinch moves the camera closer. The scene
// draws into a float image with a depth buffer. A camera along each light draws a map
// of the PBR fish, and a shade pass reads both maps from the camera's view: where the
// fish lies in its own shadow, where ripples of light fall, and where the rim light
// comes through its fins. A present pass turns the light into screen colors over a
// water gradient, applies all three to the PBR fish, paints and inks the toon fish,
// and names the style in a corner.

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
  // How much of its light the PBR fish loses where its fins and body keep the key light off it.
  shadow: f32, // 0..1
  // How much brighter the ripples of light on the PBR fish make it.
  ripples: f32, // 0..2
  // How bright the rim light shines through the PBR fish's fins.
  glow: f32, // 0..4
}

@group(0) @binding(0) var<uniform> look: Look;
@group(0) @binding(1) var image: texture_2d<f32>;
@group(0) @binding(2) var image_sampler: sampler;
// The fish pass's depth buffer, which the ink reads one texel at a time. WebGL2
// reads it as zeros, so there the ink follows the silhouette alone.
@group(0) @binding(3) var depth: texture_2d<f32>;
// What shade.wgsl drew for each pixel of the fish: in red, how far it lies in its own
// shadow, in green, the ripples of light on it, and in blue, the rim light that comes through it.
@group(0) @binding(4) var lighting: texture_2d<f32>;

// The camera's near and far planes in main.js, which turn a depth back into a distance.
const NEAR = 0.01;
const FAR = 10.0;
// The color of the toon fish's ink, and how much farther than a flat surface
// through a pixel its neighbors must lie together, as a fraction of its distance,
// for an edge, as where a fin passes in front of the body.
const INK = vec3<f32>(0.02, 0.03, 0.07);
const JUMP = 0.004;
// The color the rim light takes through the thin, dark skin of a fin.
const FIN = vec3<f32>(0.5, 0.3, 0.13);

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  // The fish pass clears to transparent, so alpha marks where the fish is.
  let uv = vec2<f32>(in.uv.x, 1.0 - in.uv.y);
  let fish = textureSample(image, image_sampler, uv);
  let size = vec2<f32>(textureDimensions(image));

  // Light falls in from above and fades with depth.
  let above = in.uv - vec2<f32>(0.5, 1.15);
  let halo = exp(-6.0 * dot(above, above)) * 0.35;
  let water = mix(look.deep_color, look.surface_color, smoothstep(0.0, 1.0, in.uv.y)) + look.surface_color * halo;

  // The PBR fish's own shadows, ripples and glowing fins. The toon and unlit fish have none.
  let mask = textureSample(lighting, image_sampler, uv).rgb * f32(look.style == 0u);
  let lit = fish.rgb * (1.0 - mask.r * look.shadow) * (1.0 + mask.g * look.ripples);
  let light = lit + FIN * mask.b * look.glow;
  var color = mix(water, shade(light), fish.a);
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
  let own = textureLoad(image, pixel, 0).rgb;
  var best = own;
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
  // A dark spot much smaller than the square, such as the eye, keeps its own color.
  if (brightness(own) < 0.3 * brightness(best)) {
    return own;
  }
  return best;
}

// How much ink the toon fish takes at \`pixel\`, 0 or 1: along its silhouette,
// where its coverage changes, and on the near side of an edge, where the
// neighbors on a line through the pixel lie farther than a flat surface would put them.
fn ink(pixel: vec2<i32>, reach: i32, last: vec2<i32>) -> f32 {
  let cover = textureLoad(image, pixel, 0).a;
  let here = nearness_at(pixel);
  // Across, down and both diagonals.
  var lines = array<vec2<i32>, 4>(vec2<i32>(1, 0), vec2<i32>(0, 1), vec2<i32>(1, 1), vec2<i32>(1, -1));
  var amount = 0.0;
  for (var index = 0; index < 4; index++) {
    let ahead = clamp(pixel + lines[index] * reach, vec2<i32>(0), last);
    let behind = clamp(pixel - lines[index] * reach, vec2<i32>(0), last);
    amount = max(amount, abs(textureLoad(image, ahead, 0).a - cover));
    amount = max(amount, abs(textureLoad(image, behind, 0).a - cover));
    // Two neighbors on a flat surface average to the pixel's own nearness, however
    // steep the surface, and two that lie behind an edge average to less.
    if (2.0 * here - nearness_at(ahead) - nearness_at(behind) > JUMP * here) {
      amount = 1.0;
    }
  }
  return amount;
}

// One over the distance along the view axis from the camera to the fish at \`pixel\`,
// which changes at an even rate from pixel to pixel across a flat surface. A pixel
// the fish doesn't cover keeps the cleared depth, 1, and reads as the far plane.
fn nearness_at(pixel: vec2<i32>) -> f32 {
  let stored = textureLoad(depth, pixel, 0).r;
  return (FAR - stored * (FAR - NEAR)) / (NEAR * FAR);
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

const DEPTH = /* wgsl */ `// Draws the fish as a light sees it. Each pixel keeps how far along the light the
// nearest point of the fish lies, which shade.wgsl compares against.

struct Camera {
  view_proj: mat4x4<f32>,
  position: vec3<f32>,
}

// The light's camera, which main.js points along the light every frame.
@group(0) @binding(0) var<uniform> camera: Camera;

struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) depth: f32,
}

@vertex
fn vs_main(
  @location(0) position: vec3<f32>,
  @location(3) model_0: vec4<f32>,
  @location(4) model_1: vec4<f32>,
  @location(5) model_2: vec4<f32>,
  @location(6) model_3: vec4<f32>,
) -> VertexOutput {
  let model = mat4x4<f32>(model_0, model_1, model_2, model_3);
  var out: VertexOutput;
  out.position = camera.view_proj * model * vec4<f32>(position, 1.0);
  out.depth = out.position.z;
  return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  return vec4<f32>(in.depth, 0.0, 0.0, 1.0);
}
`;

const SHADE = /* wgsl */ `// Draws the fish as the camera sees it, with what the two lights do at each point:
// how far it lies in the shadow its own fins and body cast from the key light, the
// ripples the key light lays on it, and the rim light that comes through where it is thin.

struct Camera {
  view_proj: mat4x4<f32>,
  position: vec3<f32>,
}

// A light's camera and the direction the light travels, which main.js sets every frame.
struct LightView {
  view_proj: mat4x4<f32>,
  direction: vec3<f32>,
}

@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var<uniform> key: LightView;
@group(0) @binding(2) var<uniform> rim: LightView;
// What depth.wgsl drew from each light.
@group(0) @binding(3) var key_map: texture_2d<f32>;
@group(0) @binding(4) var rim_map: texture_2d<f32>;
// The time in seconds, which main.js sets every frame.
@group(0) @binding(5) var<uniform> time: f32;

// How far a point moves out along its normal before it looks itself up, in meters,
// and how much nearer the light a surface must lie to cast a shadow on it, as a
// fraction of the depth a light's camera sees, so no surface shadows itself.
const LIFT = 0.003;
const BIAS = 0.002;
// How far apart the 25 lookups lie, in shadow map texels, which softens the shadow's edge.
const SPREAD = 1.5;
// How thick the fish is along the rim light where a third of that light still comes
// through, as a fraction of the depth the rim light's camera sees: 3 mm, as in a fin.
const THIN = 0.004;

struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) world: vec3<f32>,
  @location(1) normal: vec3<f32>,
}

@vertex
fn vs_main(
  @location(0) position: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(3) model_0: vec4<f32>,
  @location(4) model_1: vec4<f32>,
  @location(5) model_2: vec4<f32>,
  @location(6) model_3: vec4<f32>,
) -> VertexOutput {
  let model = mat4x4<f32>(model_0, model_1, model_2, model_3);
  let world = model * vec4<f32>(position, 1.0);
  var out: VertexOutput;
  out.position = camera.view_proj * world;
  out.world = world.xyz;
  out.normal = (model * vec4<f32>(normal, 0.0)).xyz;
  return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  let normal = normalize(in.normal);
  // The key light reaches a side that faces it, and the rim light comes out of a
  // side that faces away from it, having gone in on the other side.
  let lit = smoothstep(0.0, 0.2, dot(normal, -normalize(key.direction)));
  let behind = smoothstep(0.0, 0.2, dot(normal, normalize(rim.direction)));
  let spot = key.view_proj * vec4<f32>(in.world + normal * LIFT, 1.0);
  let shade = hidden(spot) * lit;
  let through = exp(-thickness(rim.view_proj * vec4<f32>(in.world, 1.0)) / THIN) * behind;
  return vec4<f32>(shade, ripples(spot.xy, time) * (lit - shade), through, 1.0);
}

// How much of the key light the fish keeps from the point \`spot\` in the key light's
// view, from 0 to 1, out of 25 lookups around it in the key light's shadow map.
fn hidden(spot: vec4<f32>) -> f32 {
  let size = vec2<f32>(textureDimensions(key_map));
  let middle = texel(spot) * size;
  let last = vec2<i32>(size) - 1;
  var count = 0.0;
  for (var y = -2; y <= 2; y++) {
    for (var x = -2; x <= 2; x++) {
      let at = clamp(vec2<i32>(middle + vec2<f32>(f32(x), f32(y)) * SPREAD), vec2<i32>(0), last);
      count += select(0.0, 1.0, textureLoad(key_map, at, 0).r < spot.z - BIAS);
    }
  }
  return count / 25.0;
}

// How far the rim light travels through the fish to reach the point \`spot\` in its view,
// as a fraction of the depth its camera sees.
fn thickness(spot: vec4<f32>) -> f32 {
  let size = vec2<f32>(textureDimensions(rim_map));
  let at = clamp(vec2<i32>(texel(spot) * size), vec2<i32>(0), vec2<i32>(size) - 1);
  return max(spot.z - textureLoad(rim_map, at, 0).r, 0.0);
}

// Where a point a light's camera sees lands on its map, from 0 to 1 across and down.
fn texel(spot: vec4<f32>) -> vec2<f32> {
  return vec2<f32>(spot.x * 0.5 + 0.5, 0.5 - spot.y * 0.5);
}

// Light through a rippled water surface gathers in bright lines that drift. Two
// layers of cells slide past each other, and the light gathers along the edges of both.
fn ripples(spot: vec2<f32>, time: f32) -> f32 {
  let point = spot * 6.0;
  let one = worley2(point + vec2<f32>(0.31, 0.17) * time);
  let two = worley2(point * 1.4 - vec2<f32>(0.23, 0.29) * time + vec2<f32>(7.0));
  let edge = min(one.y - one.x, two.y - two.x);
  return pow(1.0 - smoothstep(0.0, 0.3, edge), 3.0);
}
`;

const PROGRAM = /* js */ `import { Camera, Light, Model, Pass, PbrMaterial, Scene, Shader, TextureFormat, TextureUsage, ToonMaterial, UnlitMaterial } from "fragmentcolor";
import DEPTH_SOURCE from "./depth.wgsl?raw";
import PRESENT_SOURCE from "./present.wgsl?raw";
import SHADE_SOURCE from "./shade.wgsl?raw";

const LIBRARY = ["color/tonemap_aces", "color/linear_to_srgb"];

// Barramundi Fish by Microsoft, CC0, from the Khronos glTF sample assets.
const MODEL = "/playground/models/BarramundiFish.glb";
// The middle of the fish and how far the camera starts from it, in meters.
const CENTER = [0, 0.143, 0];
const DISTANCE = 1.3;
// The key light shines down on the fish as daylight through the water would,
// from above and to the camera's right, and the rim light from behind the fish. The
// toon fish takes its key light from nearer overhead, which lays its shadow band along
// the belly, and more ambient light, since a cartoon shades with a darker tone of the same
// color. Unlit takes no light at all. KEY and AMBIENT list the styles in turn: PBR, toon, unlit.
const KEY = [[-0.6, -1, -0.5], [-0.3, -1, -0.35], [-0.5, -0.6, -1]];
const RIM = [0.6, 0.2, 1];
const AMBIENT = [[0.1, 0.1, 0.12], [0.3, 0.32, 0.36], [0.1, 0.1, 0.12]];
// How fast the fish turns before the first drag, in radians per second. It turns its
// nose toward the camera first, so the first toon turn shows its side.
const SPIN = -0.4;
// How long each style shows before the next one takes over, in seconds, until the first M.
const HOLD = 4;
// The PBR fish looks wetter than its file describes it: its scales keep a third of
// their roughness, and their bumps stand out twice as far.
const GLOSS = 0.35;
const BUMPS = 2;
// Each light's camera sees a box around the fish, this far each way from its
// middle in meters, and the map it draws is this many texels across.
const REACH = 0.4;
const MAP_SIZE = 1024;

export async function setup(renderer, target) {
  const response = await fetch(MODEL);
  const scene = Scene.load(new Uint8Array(await response.arrayBuffer()));
  // Upload the textures now, so the first frame only draws.
  await renderer.load(scene);

  // The fish pass clears to transparent, so present.wgsl can lay the fish over the water.
  const fish = scene.getPass(0);
  fish.setClearColor([0, 0, 0, 0]);

  // A camera along each light draws the fish into a map in a pass of its own: the key
  // light's for the shadows, and the rim light's for the light that comes through the
  // fins. The shade pass draws the fish again from the camera, with what both maps say
  // of each point, for present.wgsl to use.
  const keyView = await createView(renderer, scene, "key");
  const rimView = await createView(renderer, scene, "rim");
  const receiver = await Shader.fetch(["noise/worley2", SHADE_SOURCE]);
  receiver.set("key_map", keyView.map);
  receiver.set("rim_map", rimView.map);
  const shadePass = new Pass("shade");
  shadePass.setClearColor([0, 0, 0, 0]);
  scene.addPass(shadePass);

  // Each part of the fish draws in PBR, wetter than the file describes it, and in the
  // key, rim and shade passes. It gets a toon and an unlit Model over its mesh too, which
  // both sample its base color texture, so that stays one upload. The lists follow the
  // styles in turn, and so does present.wgsl's look.style.
  const styles = [[], [], []];
  for (const model of scene.models()) {
    const pbr = model.material();
    if (!(pbr instanceof PbrMaterial)) continue;
    pbr.roughness(GLOSS).normalScale(BUMPS);
    // Light in three flat steps, a hard white glint, and a pale rim along the edges.
    const toon = new ToonMaterial()
      .bands(3)
      .highlight([1, 1, 1])
      .highlightSize(0.012)
      .rim([0.2, 0.35, 0.4])
      .rimWidth(0.12);
    toon.baseColorMap().texture(pbr.baseColorMap());
    const unlit = new UnlitMaterial();
    unlit.baseColorMap().texture(pbr.baseColorMap());
    styles[0].push(
      model,
      restyle(scene, "key", model, keyView.caster),
      restyle(scene, "rim", model, rimView.caster),
      restyle(scene, "shade", model, receiver),
    );
    styles[1].push(restyle(scene, fish.name(), model, toon));
    styles[2].push(restyle(scene, fish.name(), model, unlit));
  }

  const camera = Camera.perspective(0.6, 1, 0.01, 10);
  const key = Light.directional(KEY[0], [1, 0.96, 0.9]).setIntensity(3);
  const rim = Light.directional(RIM, [0.6, 0.7, 1]).setIntensity(1);
  scene.add(camera).add(key).add(rim).ambient(AMBIENT[0]);
  // The camera draws the shade pass too.
  scene.addTo("shade", camera);

  const present = await Shader.fetch([...LIBRARY, PRESENT_SOURCE]);
  present.set("look.exposure", 1.5);
  present.set("look.surface_color", [0.25, 0.58, 0.68]);
  present.set("look.deep_color", [0.02, 0.12, 0.2]);
  present.set("look.shadow", 0.7);
  present.set("look.ripples", 0.8);
  present.set("look.glow", 1);
  scene.addPass(Pass.fromShader("present", present));

  // The playground gives a control to each value setup sets on a shader it returns.
  const state = {
    scene,
    present,
    camera,
    key,
    rim,
    keyView,
    rimView,
    receiver,
    fish,
    shadePass,
    styles,
    style: 0,
    shown: 0,
    cycling: true,
    held: false,
    textures: null,
    size: null,
    wanted: null,
    yaw: 1,
    pitch: 0.25,
    distance: DISTANCE,
    turning: true,
    last: null,
  };
  show(state, 0);
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
  // Both lights turn with the camera, so the side in view stays lit, and each
  // light's camera follows its light.
  const keyDirection = turn(KEY[state.style], state.yaw);
  const rimDirection = turn(RIM, state.yaw);
  state.key.setDirection(keyDirection);
  state.rim.setDirection(rimDirection);
  aim(state.receiver, "key", state.keyView, keyDirection);
  aim(state.receiver, "rim", state.rimView, rimDirection);
  state.receiver.set("time", time);
  renderer.render(state.scene, target);
}

// A camera that looks along a light, with the pass \`name\` in which it draws the fish
// into a map of how far along the light each point lies: the camera, the shader the
// fish draws with in that pass, and the map.
async function createView(renderer, scene, name) {
  const camera = Camera.orthographic(-REACH, REACH, -REACH, REACH, 1 - REACH, 1 + REACH);
  const caster = new Shader(DEPTH_SOURCE).doubleSided(true);
  const pass = new Pass(name);
  pass.setClearColor([1, 1, 1, 1]);
  scene.addPass(pass).addTo(name, camera);
  const usage = TextureUsage.RENDER_ATTACHMENT | TextureUsage.TEXTURE_BINDING;
  const size = [MAP_SIZE, MAP_SIZE];
  const map = await renderer.createStorageTexture(size, TextureFormat.Rgba16Float, null, usage);
  pass.setTarget(map);
  pass.setDepthTarget(await renderer.createDepthTexture(size));
  return { camera, caster, map };
}

// Points the camera of \`view\`, the light \`name\`'s, at the fish along \`direction\`
// from a meter away, and tells the shade pass's \`receiver\` how that camera sees.
function aim(receiver, name, view, direction) {
  const length = Math.hypot(...direction);
  view.camera.lookAt(CENTER.map((center, axis) => center - direction[axis] / length), CENTER, [0, 1, 0]);
  receiver.set(\`\${name}.view_proj\`, view.camera.viewProj());
  receiver.set(\`\${name}.direction\`, direction);
}

// A Model in \`pass\` that draws the mesh of \`model\` with \`material\`, where \`model\` is.
function restyle(scene, pass, model, material) {
  const copy = new Model(model.mesh(), material);
  copy.setTransform(model.transform());
  scene.addTo(pass, copy);
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
// the tone map, and a depth texture keeps its far side behind its near side. The
// shade pass draws into an image and a depth texture of its own. present.wgsl
// reads all but the last. A new canvas size gets a new set, and the fish keeps
// drawing at the old size until it is ready.
async function fit(state, renderer, size) {
  state.wanted = size;
  const usage = TextureUsage.RENDER_ATTACHMENT | TextureUsage.TEXTURE_BINDING;
  const textures = {
    image: await renderer.createStorageTexture(size, TextureFormat.Rgba16Float, null, usage),
    depth: await renderer.createDepthTexture(size),
    lighting: await renderer.createStorageTexture(size, TextureFormat.Rgba8Unorm, null, usage),
    lightingDepth: await renderer.createDepthTexture(size),
  };
  // When a newer size came in during the wait, these are already out of date.
  // Unregistering frees them now, where a dropped handle waits for the collector.
  const stale = state.wanted !== size ? textures : state.textures;
  for (const texture of Object.values(stale ?? {})) renderer.unregisterTexture(texture.id());
  if (stale === textures) return;
  state.fish.setTarget(textures.image);
  state.fish.setDepthTarget(textures.depth);
  state.shadePass.setTarget(textures.lighting);
  state.shadePass.setDepthTarget(textures.lightingDepth);
  state.present.set("image", textures.image);
  state.present.set("depth", textures.depth);
  state.present.set("lighting", textures.lighting);
  Object.assign(state, { textures, size });
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
  "look.shadow": 0.7,
  "look.ripples": 0.8,
  "look.glow": 1,
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
    'const DEPTH_SOURCE: &str = include_str!("depth.wgsl");',
    'const PRESENT_SOURCE: &str = include_str!("present.wgsl");',
    'const SHADE_SOURCE: &str = include_str!("shade.wgsl");',
    "",
    `/// ${CREDIT}`,
    'const MODEL: &str = "BarramundiFish.glb";',
    "/// The middle of the fish and how far the camera starts from it, in meters.",
    "const CENTER: [f32; 3] = [0.0, 0.143, 0.0];",
    "const DISTANCE: f32 = 1.3;",
    "/// The key light shines down on the fish as daylight through the water would, from",
    "/// above and to the camera's right, and the rim light from behind the fish. The toon",
    "/// fish takes its key light from nearer overhead, which lays its shadow band along the",
    "/// belly, and more ambient light, since a cartoon shades with a darker tone of the same color.",
    "/// Unlit takes no light at all. `KEY` and `AMBIENT` list the styles in turn: PBR, toon, unlit.",
    "const KEY: [[f32; 3]; 3] = [[-0.6, -1.0, -0.5], [-0.3, -1.0, -0.35], [-0.5, -0.6, -1.0]];",
    "const RIM: [f32; 3] = [0.6, 0.2, 1.0];",
    "const AMBIENT: [[f32; 3]; 3] = [[0.1, 0.1, 0.12], [0.3, 0.32, 0.36], [0.1, 0.1, 0.12]];",
    "/// How fast the fish turns before the first drag, in radians per second. It turns its",
    "/// nose toward the camera first, so the first toon turn shows its side.",
    "const SPIN: f32 = -0.4;",
    "/// How long each style shows before the next one takes over, in seconds, until the first M.",
    "const HOLD: f32 = 4.0;",
    "/// The PBR fish looks wetter than its file describes it: its scales keep a third of",
    "/// their roughness, and their bumps stand out twice as far.",
    "const GLOSS: f32 = 0.35;",
    "const BUMPS: f32 = 2.0;",
    "/// Each light's camera sees a box around the fish, this far each way from its",
    "/// middle in meters, and the map it draws is this many texels across.",
    "const REACH: f32 = 0.4;",
    "const MAP_SIZE: u32 = 1024;",
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
    "/// A camera that looks along a light, the shader the fish draws with in that",
    "/// light's pass, and the map the pass draws: how far along the light each point lies.",
    "struct LightView {",
    "    camera: Camera,",
    "    caster: Shader,",
    "    map: Texture,",
    "}",
    "",
    "struct State {",
    "    scene: Scene,",
    "    present: Shader,",
    "    camera: Camera,",
    "    key: Light,",
    "    rim: Light,",
    "    key_view: LightView,",
    "    rim_view: LightView,",
    "    /// The shade pass's shader, which reads the maps of both light views.",
    "    receiver: Shader,",
    "    fish: Pass,",
    "    shade: Pass,",
    "    /// The Models of each style, of which only `style`'s show.",
    "    styles: [Vec<Model>; 3],",
    "    style: usize,",
    "    /// How long the style has shown, in seconds.",
    "    shown: f32,",
    "    /// Whether the styles still take turns by themselves.",
    "    cycling: bool,",
    "    /// Whether M was down on the last frame.",
    "    held: bool,",
    "    /// The textures `fit` made for the window's size.",
    "    textures: Vec<Texture>,",
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
    "    // The fish pass clears to transparent, so present.wgsl can lay the fish over the water.",
    "    // The first frame gives it an image and a depth buffer the size of the window.",
    '    let fish = scene.get_pass(0).ok_or("the model loaded no pass")?;',
    "    fish.set_clear_color([0.0, 0.0, 0.0, 0.0])?;",
    "",
    "    // A camera along each light draws the fish into a map in a pass of its own: the key",
    "    // light's for the shadows, and the rim light's for the light that comes through the",
    "    // fins. The shade pass draws the fish again from the camera, with what both maps say",
    "    // of each point, for present.wgsl to use.",
    '    let key_view = create_view(renderer, &scene, "key").await?;',
    '    let rim_view = create_view(renderer, &scene, "rim").await?;',
    '    let receiver = Shader::new(["noise/worley2", SHADE_SOURCE])?;',
    '    receiver.set("key_map", &key_view.map)?;',
    '    receiver.set("rim_map", &rim_view.map)?;',
    '    let shade = Pass::new("shade");',
    "    shade.set_clear_color([0.0, 0.0, 0.0, 0.0])?;",
    "    scene.add_pass(&shade);",
    "",
    "    // Each part of the fish draws in PBR, wetter than the file describes it, and in the",
    "    // key, rim and shade passes. It gets a toon and an unlit Model over its mesh too, which",
    "    // both sample its base color texture, so that stays one upload. The lists follow the",
    "    // styles in turn, and so does present.wgsl's look.style.",
    "    let mut styles: [Vec<Model>; 3] = Default::default();",
    "    for model in scene.models() {",
    "        let Material::Pbr(pbr) = model.material() else {",
    "            continue;",
    "        };",
    "        pbr.roughness(GLOSS)?.normal_scale(BUMPS)?;",
    "        // Light in three flat steps, a hard white glint, and a pale rim along the edges.",
    "        let toon = ToonMaterial::new()",
    "            .bands(3)?",
    "            .highlight([1.0, 1.0, 1.0])?",
    "            .highlight_size(0.012)?",
    "            .rim([0.2, 0.35, 0.4])?",
    "            .rim_width(0.12)?;",
    "        toon.base_color_map().texture(&pbr.base_color_map())?;",
    "        let unlit = UnlitMaterial::new();",
    "        unlit.base_color_map().texture(&pbr.base_color_map())?;",
    '        styles[0].push(restyle(&scene, "key", &model, key_view.caster.clone())?);',
    '        styles[0].push(restyle(&scene, "rim", &model, rim_view.caster.clone())?);',
    '        styles[0].push(restyle(&scene, "shade", &model, receiver.clone())?);',
    "        styles[1].push(restyle(&scene, &fish.name(), &model, toon)?);",
    "        styles[2].push(restyle(&scene, &fish.name(), &model, unlit)?);",
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
    "    // The camera draws the shade pass too.",
    '    scene.add_to("shade", &camera)?;',
    "",
    ...wrap("    ", "let present = Shader::new([", [...library.map(quote), "PRESENT_SOURCE"], "])?;", 100),
    ...setCalls("rust", values, { after: "?;" }),
    '    scene.add_pass(&Pass::from_shader("present", &present));',
    "",
    "    let mut state = State {",
    "        scene,",
    "        present,",
    "        camera,",
    "        key,",
    "        rim,",
    "        key_view,",
    "        rim_view,",
    "        receiver,",
    "        fish,",
    "        shade,",
    "        styles,",
    "        style: 0,",
    "        shown: 0.0,",
    "        cycling: true,",
    "        held: false,",
    "        textures: Vec::new(),",
    "        size: None,",
    "        yaw: 1.0,",
    "        pitch: 0.25,",
    "        distance: DISTANCE,",
    "        turning: true,",
    "        last: None,",
    "    };",
    "    show(&mut state, 0)?;",
    "    Ok(state)",
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
    "    // Both lights turn with the camera, so the side in view stays lit, and each",
    "    // light's camera follows its light.",
    "    let key_direction = turn(KEY[state.style], state.yaw);",
    "    let rim_direction = turn(RIM, state.yaw);",
    "    state.key.set_direction(key_direction)?;",
    "    state.rim.set_direction(rim_direction)?;",
    '    aim(&state.receiver, "key", &state.key_view, key_direction)?;',
    '    aim(&state.receiver, "rim", &state.rim_view, rim_direction)?;',
    '    state.receiver.set("time", time)?;',
    "    renderer.render(&state.scene, target)?;",
    "    Ok(())",
    "}",
    "",
    "/// A camera that looks along a light, with the pass `name` in which it draws the fish",
    "/// into a map of how far along the light each point lies.",
    "async fn create_view(",
    "    renderer: &Renderer,",
    "    scene: &Scene,",
    "    name: &str,",
    ") -> Result<LightView, Box<dyn Error>> {",
    "    let camera = Camera::orthographic(-REACH, REACH, -REACH, REACH, 1.0 - REACH, 1.0 + REACH)?;",
    "    let caster = Shader::new(DEPTH_SOURCE)?.double_sided(true);",
    "    let pass = Pass::new(name);",
    "    pass.set_clear_color([1.0, 1.0, 1.0, 1.0])?;",
    "    scene.add_pass(&pass).add_to(name, &camera)?;",
    "    let map = create_image(renderer, [MAP_SIZE; 2], TextureFormat::Rgba16Float).await?;",
    "    pass.set_target(&map)?;",
    "    pass.set_depth_target(&renderer.create_depth_texture([MAP_SIZE; 2]).await?)?;",
    "    Ok(LightView {",
    "        camera,",
    "        caster,",
    "        map,",
    "    })",
    "}",
    "",
    "/// Points the camera of `view`, the light `name`'s, at the fish along `direction`",
    "/// from a meter away, and tells the shade pass's `receiver` how that camera sees.",
    "fn aim(",
    "    receiver: &Shader,",
    "    name: &str,",
    "    view: &LightView,",
    "    direction: [f32; 3],",
    ") -> Result<(), Box<dyn Error>> {",
    "    let length = direction.iter().map(|axis| axis * axis).sum::<f32>().sqrt();",
    "    let position = [0, 1, 2].map(|axis| CENTER[axis] - direction[axis] / length);",
    "    view.camera.look_at(position, CENTER, [0.0, 1.0, 0.0])?;",
    '    receiver.set(&format!("{name}.view_proj"), view.camera.view_proj())?;',
    '    receiver.set(&format!("{name}.direction"), direction)?;',
    "    Ok(())",
    "}",
    "",
    "/// A Model in `pass` that draws the mesh of `model` with `material`, where `model` is.",
    "fn restyle(",
    "    scene: &Scene,",
    "    pass: &str,",
    "    model: &Model,",
    "    material: impl Into<Material>,",
    ") -> Result<Model, Box<dyn Error>> {",
    "    let copy = Model::new(model.mesh().clone(), material);",
    "    copy.set_transform(model.transform())?;",
    "    scene.add_to(pass, &copy)?;",
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
    "/// the tone map, and a depth texture keeps its far side behind its near side. The",
    "/// shade pass draws into an image and a depth texture of its own. present.wgsl",
    "/// reads all but the last. A new window size gets a new set.",
    "async fn fit(state: &mut State, renderer: &Renderer, size: [f32; 2]) -> Result<(), Box<dyn Error>> {",
    "    let pixels = [size[0].max(1.0) as u32, size[1].max(1.0) as u32];",
    "    let image = create_image(renderer, pixels, TextureFormat::Rgba16Float).await?;",
    "    let depth = renderer.create_depth_texture(pixels).await?;",
    "    let lighting = create_image(renderer, pixels, TextureFormat::Rgba8Unorm).await?;",
    "    let lighting_depth = renderer.create_depth_texture(pixels).await?;",
    "",
    "    // Unregistering frees the old set now, before its handles go.",
    "    for old in state.textures.drain(..) {",
    "        renderer.unregister_texture(old.id())?;",
    "    }",
    "    state.fish.set_target(&image)?;",
    "    state.fish.set_depth_target(&depth)?;",
    "    state.shade.set_target(&lighting)?;",
    "    state.shade.set_depth_target(&lighting_depth)?;",
    '    state.present.set("image", &image)?;',
    '    state.present.set("depth", &depth)?;',
    '    state.present.set("lighting", &lighting)?;',
    "    state.textures = vec![image, depth, lighting, lighting_depth];",
    "    state.size = Some(size);",
    "    Ok(())",
    "}",
    "",
    "/// An empty texture of `pixels` and `format` that a pass draws into and a shader reads.",
    "async fn create_image(",
    "    renderer: &Renderer,",
    "    pixels: [u32; 2],",
    "    format: TextureFormat,",
    ") -> Result<Texture, Box<dyn Error>> {",
    "    let options = TextureOptions {",
    "        size: Some(pixels.into()),",
    "        format,",
    "        ..Default::default()",
    "    }",
    "    .with_usage(TextureUsage::RENDER_ATTACHMENT | TextureUsage::TEXTURE_BINDING);",
    "    let input = TextureInput {",
    "        data: TextureData::Empty,",
    "        options,",
    "    };",
    "    Ok(renderer.create_storage_texture(input).await?)",
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
    'DEPTH_SOURCE = Path(__file__).with_name("depth.wgsl").read_text()',
    'PRESENT_SOURCE = Path(__file__).with_name("present.wgsl").read_text()',
    'SHADE_SOURCE = Path(__file__).with_name("shade.wgsl").read_text()',
    "",
    `# ${CREDIT}`,
    'MODEL = Path(__file__).with_name("BarramundiFish.glb")',
    "# The middle of the fish and how far the camera starts from it, in meters.",
    "CENTER = [0.0, 0.143, 0.0]",
    "DISTANCE = 1.3",
    "# The key light shines down on the fish as daylight through the water",
    "# would, from above and to the camera's right, and the rim light from",
    "# behind the fish. The toon fish takes its key light from nearer overhead,",
    "# which lays its shadow band along the belly, and more ambient light, since",
    "# a cartoon shades with a darker tone of the same color. Unlit takes no light",
    "# at all. KEY and AMBIENT list the styles in turn: PBR, toon, unlit.",
    "KEY = [[-0.6, -1.0, -0.5], [-0.3, -1.0, -0.35], [-0.5, -0.6, -1.0]]",
    "RIM = [0.6, 0.2, 1.0]",
    "AMBIENT = [[0.1, 0.1, 0.12], [0.3, 0.32, 0.36], [0.1, 0.1, 0.12]]",
    "# How fast the fish turns before the first drag, in radians per second. It turns its",
    "# nose toward the camera first, so the first toon turn shows its side.",
    "SPIN = -0.4",
    "# How long each style shows before the next one takes over, in seconds, until the first M.",
    "HOLD = 4.0",
    "# The PBR fish looks wetter than its file describes it: its scales keep a third of",
    "# their roughness, and their bumps stand out twice as far.",
    "GLOSS = 0.35",
    "BUMPS = 2.0",
    "# Each light's camera sees a box around the fish, this far each way from its",
    "# middle in meters, and the map it draws is this many texels across.",
    "REACH = 0.4",
    "MAP_SIZE = 1024",
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
    "    # The fish pass clears to transparent, so present.wgsl can lay the fish",
    "    # over the water. The first frame gives it an image and a depth buffer",
    "    # the size of the canvas.",
    "    fish = scene.get_pass(0)",
    "    fish.set_clear_color([0.0, 0.0, 0.0, 0.0])",
    "",
    "    # A camera along each light draws the fish into a map in a pass of its own: the",
    "    # key light's for the shadows, and the rim light's for the light that comes",
    "    # through the fins. The shade pass draws the fish again from the camera, with",
    "    # what both maps say of each point, for present.wgsl to use.",
    '    key_view = create_view(renderer, scene, "key")',
    '    rim_view = create_view(renderer, scene, "rim")',
    '    receiver = Shader(["noise/worley2", SHADE_SOURCE])',
    '    receiver.set("key_map", key_view.map)',
    '    receiver.set("rim_map", rim_view.map)',
    '    shade = Pass("shade")',
    "    shade.set_clear_color([0.0, 0.0, 0.0, 0.0])",
    "    scene.add_pass(shade)",
    "",
    "    # Each part of the fish draws in PBR, wetter than the file describes it, and in",
    "    # the key, rim and shade passes. It gets a toon and an unlit Model over its mesh",
    "    # too, which both sample its base color texture, so that stays one upload. The",
    "    # lists follow the styles in turn, and so does present.wgsl's look.style.",
    "    styles = [[], [], []]",
    "    for model in scene.models():",
    "        pbr = model.material()",
    "        if not isinstance(pbr, PbrMaterial):",
    "            continue",
    "        pbr.roughness(GLOSS).normal_scale(BUMPS)",
    "        # Light in three flat steps, a hard white glint, and a pale rim along the edges.",
    "        toon = (",
    "            ToonMaterial()",
    "            .bands(3)",
    "            .highlight([1.0, 1.0, 1.0])",
    "            .highlight_size(0.012)",
    "            .rim([0.2, 0.35, 0.4])",
    "            .rim_width(0.12)",
    "        )",
    "        toon.base_color_map().texture(pbr.base_color_map())",
    "        unlit = UnlitMaterial()",
    "        unlit.base_color_map().texture(pbr.base_color_map())",
    "        styles[0].append(model)",
    '        styles[0].append(restyle(scene, "key", model, key_view.caster))',
    '        styles[0].append(restyle(scene, "rim", model, rim_view.caster))',
    '        styles[0].append(restyle(scene, "shade", model, receiver))',
    "        styles[1].append(restyle(scene, fish.name(), model, toon))",
    "        styles[2].append(restyle(scene, fish.name(), model, unlit))",
    "",
    "    camera = Camera.perspective(0.6, 1.0, 0.01, 10.0)",
    "    key = Light.directional(KEY[0], [1.0, 0.96, 0.9]).set_intensity(3.0)",
    "    rim = Light.directional(RIM, [0.6, 0.7, 1.0]).set_intensity(1.0)",
    "    scene.add(camera).add(key).add(rim).ambient(AMBIENT[0])",
    "    # The camera draws the shade pass too.",
    '    scene.add_to("shade", camera)',
    "",
    "    present = Shader([*LIBRARY, PRESENT_SOURCE])",
    ...setCalls("python", values),
    '    scene.add_pass(Pass.from_shader("present", present))',
    "",
    "    state = SimpleNamespace(",
    "        scene=scene,",
    "        present=present,",
    "        camera=camera,",
    "        key=key,",
    "        rim=rim,",
    "        key_view=key_view,",
    "        rim_view=rim_view,",
    "        receiver=receiver,",
    "        fish=fish,",
    "        shade=shade,",
    "        styles=styles,",
    "        style=0,",
    "        shown=0.0,",
    "        cycling=True,",
    "        held=False,",
    "        textures=[],",
    "        size=None,",
    "        yaw=1.0,",
    "        pitch=0.25,",
    "        distance=DISTANCE,",
    "        turning=True,",
    "        last=None,",
    "    )",
    "    show(state, 0)",
    "    return state",
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
    "    # Both lights turn with the camera, so the side in view stays lit, and each",
    "    # light's camera follows its light.",
    "    key_direction = turn(KEY[state.style], state.yaw)",
    "    rim_direction = turn(RIM, state.yaw)",
    "    state.key.set_direction(key_direction)",
    "    state.rim.set_direction(rim_direction)",
    '    aim(state.receiver, "key", state.key_view, key_direction)',
    '    aim(state.receiver, "rim", state.rim_view, rim_direction)',
    '    state.receiver.set("time", time)',
    "    renderer.render(state.scene, target)",
    "",
    "",
    "# A camera that looks along a light, with the pass name in which it draws the fish",
    "# into a map of how far along the light each point lies.",
    "def create_view(renderer, scene, name):",
    "    camera = Camera.orthographic(-REACH, REACH, -REACH, REACH, 1 - REACH, 1 + REACH)",
    "    caster = Shader(DEPTH_SOURCE).double_sided(True)",
    "    light_pass = Pass(name)",
    "    light_pass.set_clear_color([1.0, 1.0, 1.0, 1.0])",
    "    scene.add_pass(light_pass).add_to(name, camera)",
    "    usage = TextureUsage.RENDER_ATTACHMENT | TextureUsage.TEXTURE_BINDING",
    "    size = [MAP_SIZE, MAP_SIZE]",
    "    depth_map = renderer.create_storage_texture(",
    "        size, TextureFormat.Rgba16Float, usage=usage",
    "    )",
    "    light_pass.set_target(depth_map)",
    "    light_pass.set_depth_target(renderer.create_depth_texture(size))",
    "    return SimpleNamespace(camera=camera, caster=caster, map=depth_map)",
    "",
    "",
    "# Points the camera of view, the light name's, at the fish along direction from",
    "# a meter away, and tells the shade pass's receiver how that camera sees.",
    "def aim(receiver, name, view, direction):",
    "    length = math.hypot(*direction)",
    "    position = [center - axis / length for center, axis in zip(CENTER, direction)]",
    "    view.camera.look_at(position, CENTER, [0.0, 1.0, 0.0])",
    '    receiver.set(f"{name}.view_proj", view.camera.view_proj())',
    '    receiver.set(f"{name}.direction", direction)',
    "",
    "",
    "# A Model in the pass named pass_name that draws the mesh of model with material,",
    "# where model is.",
    "def restyle(scene, pass_name, model, material):",
    "    copy = Model(model.mesh(), material)",
    "    copy.set_transform(model.transform())",
    "    scene.add_to(pass_name, copy)",
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
    "# the tone map, and a depth texture keeps its far side behind its near side. The",
    "# shade pass draws into an image and a depth texture of its own. present.wgsl",
    "# reads all but the last. A new canvas size gets a new set.",
    "def fit(state, renderer, size):",
    "    pixels = [max(int(size[0]), 1), max(int(size[1]), 1)]",
    "    usage = TextureUsage.RENDER_ATTACHMENT | TextureUsage.TEXTURE_BINDING",
    "    image = renderer.create_storage_texture(",
    "        pixels, TextureFormat.Rgba16Float, usage=usage",
    "    )",
    "    depth = renderer.create_depth_texture(pixels)",
    "    lighting = renderer.create_storage_texture(",
    "        pixels, TextureFormat.Rgba8Unorm, usage=usage",
    "    )",
    "    lighting_depth = renderer.create_depth_texture(pixels)",
    "",
    "    # Unregistering frees the old set now, before its handles go.",
    "    for old in state.textures:",
    "        renderer.unregister_texture(old.id())",
    "    state.fish.set_target(image)",
    "    state.fish.set_depth_target(depth)",
    "    state.shade.set_target(lighting)",
    "    state.shade.set_depth_target(lighting_depth)",
    '    state.present.set("image", image)',
    '    state.present.set("depth", depth)',
    '    state.present.set("lighting", lighting)',
    "    state.textures = [image, depth, lighting, lighting_depth]",
    "    state.size = list(size)",
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
    "/// The key light shines down on the fish as daylight through the water would, from",
    "/// above and to the camera's right, and the rim light from behind the fish. The toon",
    "/// fish takes its key light from nearer overhead, which lays its shadow band along the",
    "/// belly, and more ambient light, since a cartoon shades with a darker tone of the same",
    "/// color. Unlit takes no light at all. Both lists hold the styles in turn: PBR, toon, unlit.",
    "let keyDirections: [[Float]] = [[-0.6, -1, -0.5], [-0.3, -1, -0.35], [-0.5, -0.6, -1]]",
    "let rimDirection: [Float] = [0.6, 0.2, 1]",
    "let ambientColors: [[Float]] = [[0.1, 0.1, 0.12], [0.3, 0.32, 0.36], [0.1, 0.1, 0.12]]",
    "/// How fast the fish turns before the first drag, in radians per second. It turns its",
    "/// nose toward the camera first, so the first toon turn shows its side.",
    "let spin: Float = -0.4",
    "/// How long each style shows before the next one takes over, in seconds, until the first M.",
    "let hold: Float = 4",
    "/// The PBR fish looks wetter than its file describes it: its scales keep a third of",
    "/// their roughness, and their bumps stand out twice as far.",
    "let gloss: Float = 0.35",
    "let bumps: Float = 2",
    "/// Each light's camera sees a box around the fish, this far each way from its",
    "/// middle in meters, and the map it draws is this many texels across.",
    "let viewReach: Float = 0.4",
    "let mapSize: UInt32 = 1024",
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
    "/// A camera that looks along a light, the shader the fish draws with in that",
    "/// light's pass, and the map the pass draws: how far along the light each point lies.",
    "struct LightView {",
    "    let camera: Camera",
    "    let caster: Shader",
    "    let map: Texture",
    "}",
    "",
    "struct State {",
    "    let scene: Scene",
    "    let present: Shader",
    "    let camera: Camera",
    "    let key: Light",
    "    let rim: Light",
    "    let keyView: LightView",
    "    let rimView: LightView",
    "    /// The shade pass's shader, which reads the maps of both light views.",
    "    let receiver: Shader",
    "    let fish: Pass",
    "    let shade: Pass",
    "    /// The Models of each style, of which only `style`'s show.",
    "    let styles: [[Model]]",
    "    var style = 0",
    "    /// How long the style has shown, in seconds.",
    "    var shown: Float = 0",
    "    /// Whether the styles still take turns by themselves.",
    "    var cycling = true",
    "    /// Whether M was down on the last frame.",
    "    var held = false",
    "    /// The textures `fit` made for the view's size.",
    "    var textures: [Texture] = []",
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
    "    // The fish pass clears to transparent, so present.wgsl can lay the fish over the water.",
    "    // The first frame gives it an image and a depth buffer the size of the view.",
    "    let fish = scene.getPass(0)!",
    "    try fish.setClearColor([0, 0, 0, 0])",
    "",
    "    // A camera along each light draws the fish into a map in a pass of its own: the key",
    "    // light's for the shadows, and the rim light's for the light that comes through the",
    "    // fins. The shade pass draws the fish again from the camera, with what both maps say",
    "    // of each point, for present.wgsl to use.",
    '    let keyView = try await createView(renderer, scene, "key")',
    '    let rimView = try await createView(renderer, scene, "rim")',
    '    let receiver = try Shader(["noise/worley2", readSource("shade")])',
    '    try receiver.set("key_map", keyView.map)',
    '    try receiver.set("rim_map", rimView.map)',
    '    let shade = Pass("shade")',
    "    try shade.setClearColor([0, 0, 0, 0])",
    "    scene.addPass(shade)",
    "",
    "    // Each part of the fish draws in PBR, wetter than the file describes it, and in the",
    "    // key, rim and shade passes. It gets a toon and an unlit Model over its mesh too, which",
    "    // both sample its base color texture, so that stays one upload. The lists follow the",
    "    // styles in turn, and so does present.wgsl's look.style.",
    "    var styles: [[Model]] = [[], [], []]",
    "    for model in scene.models() {",
    "        guard let pbr = model.material() as? PbrMaterial else {",
    "            continue",
    "        }",
    "        try pbr.roughness(gloss).normalScale(bumps)",
    "        // Light in three flat steps, a hard white glint, and a pale rim along the edges.",
    "        let toon = try ToonMaterial()",
    "            .bands(3)",
    "            .highlight([1, 1, 1])",
    "            .highlightSize(0.012)",
    "            .rim([0.2, 0.35, 0.4])",
    "            .rimWidth(0.12)",
    "        try toon.baseColorMap().texture(pbr.baseColorMap())",
    "        let unlit = UnlitMaterial()",
    "        try unlit.baseColorMap().texture(pbr.baseColorMap())",
    "        styles[0].append(model)",
    '        styles[0].append(try restyle(scene, "key", model, keyView.caster))',
    '        styles[0].append(try restyle(scene, "rim", model, rimView.caster))',
    '        styles[0].append(try restyle(scene, "shade", model, receiver))',
    "        styles[1].append(try restyle(scene, fish.name(), model, toon))",
    "        styles[2].append(try restyle(scene, fish.name(), model, unlit))",
    "    }",
    "",
    "    let camera = try Camera.perspective(0.6, 1, 0.01, 10)",
    "    let key = try Light.directional(keyDirections[0], [1, 0.96, 0.9]).setIntensity(3.0)",
    "    let rim = try Light.directional(rimDirection, [0.6, 0.7, 1]).setIntensity(1.0)",
    "    try scene.add(camera).add(key).add(rim).ambient(ambientColors[0])",
    "    // The camera draws the shade pass too.",
    '    try scene.addTo("shade", camera)',
    "",
    '    let present = try Shader(library + [readSource("present")])',
    ...setCalls("swift", values, { before: "try " }),
    '    scene.addPass(Pass.fromShader("present", present))',
    "",
    "    var state = State(",
    "        scene: scene,",
    "        present: present,",
    "        camera: camera,",
    "        key: key,",
    "        rim: rim,",
    "        keyView: keyView,",
    "        rimView: rimView,",
    "        receiver: receiver,",
    "        fish: fish,",
    "        shade: shade,",
    "        styles: styles",
    "    )",
    "    try show(&state, 0)",
    "    return state",
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
    "    // Both lights turn with the camera, so the side in view stays lit, and each",
    "    // light's camera follows its light.",
    "    let turnedKey = turn(keyDirections[state.style], state.yaw)",
    "    let turnedRim = turn(rimDirection, state.yaw)",
    "    try state.key.setDirection(turnedKey)",
    "    try state.rim.setDirection(turnedRim)",
    '    try aim(state.receiver, "key", state.keyView, turnedKey)',
    '    try aim(state.receiver, "rim", state.rimView, turnedRim)',
    '    try state.receiver.set("time", time)',
    "    try renderer.render(state.scene, target)",
    "}",
    "",
    "/// A camera that looks along a light, with the pass `name` in which it draws the fish",
    "/// into a map of how far along the light each point lies.",
    "func createView(_ renderer: Renderer, _ scene: Scene, _ name: String) async throws -> LightView {",
    "    let camera = try Camera.orthographic(-viewReach, viewReach, -viewReach, viewReach, 1 - viewReach, 1 + viewReach)",
    '    let caster = try Shader(readSource("depth")).doubleSided(true)',
    "    let pass = Pass(name)",
    "    try pass.setClearColor([1, 1, 1, 1])",
    "    try scene.addPass(pass).addTo(name, camera)",
    "    let usage = TextureUsage.RENDER_ATTACHMENT | TextureUsage.TEXTURE_BINDING",
    "    let map = try await renderer.createStorageTexture([mapSize, mapSize], .rgba16Float, nil, usage)",
    "    let depth = try await renderer.createDepthTexture([mapSize, mapSize])",
    "    try pass.setTarget(map)",
    "    try pass.setDepthTarget(depth)",
    "    return LightView(camera: camera, caster: caster, map: map)",
    "}",
    "",
    "/// Points the camera of `view`, the light `name`'s, at the fish along `direction`",
    "/// from a meter away, and tells the shade pass's `receiver` how that camera sees.",
    "func aim(_ receiver: Shader, _ name: String, _ view: LightView, _ direction: [Float]) throws {",
    "    let length = sqrt(direction.map { $0 * $0 }.reduce(0, +))",
    "    let position = zip(center, direction).map { $0 - $1 / length }",
    "    try view.camera.lookAt(position, center, [0, 1, 0])",
    '    try receiver.set("\\(name).view_proj", view.camera.viewProj())',
    '    try receiver.set("\\(name).direction", direction)',
    "}",
    "",
    "/// A Model in the pass named `pass` that draws the mesh of `model` with `material`, where `model` is.",
    "func restyle(_ scene: Scene, _ pass: String, _ model: Model, _ material: any Material) throws -> Model {",
    "    let copy = Model(model.mesh(), material)",
    "    try copy.setTransform(model.transform())",
    "    try scene.addTo(pass, copy)",
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
    "/// the tone map, and a depth texture keeps its far side behind its near side. The",
    "/// shade pass draws into an image and a depth texture of its own. present.wgsl",
    "/// reads all but the last. A new view size gets a new set.",
    "func fit(_ state: inout State, _ renderer: Renderer, _ size: [Float]) async throws {",
    "    let pixels = [UInt32(max(size[0], 1)), UInt32(max(size[1], 1))]",
    "    let usage = TextureUsage.RENDER_ATTACHMENT | TextureUsage.TEXTURE_BINDING",
    "    let image = try await renderer.createStorageTexture(pixels, .rgba16Float, nil, usage)",
    "    let depth = try await renderer.createDepthTexture(pixels)",
    "    let lighting = try await renderer.createStorageTexture(pixels, .rgba8Unorm, nil, usage)",
    "    let lightingDepth = try await renderer.createDepthTexture(pixels)",
    "",
    "    // Unregistering frees the old set now, before its handles go.",
    "    for old in state.textures {",
    "        try renderer.unregisterTexture(old.id())",
    "    }",
    "    try state.fish.setTarget(image)",
    "    try state.fish.setDepthTarget(depth)",
    "    try state.shade.setTarget(lighting)",
    "    try state.shade.setDepthTarget(lightingDepth)",
    '    try state.present.set("image", image)',
    '    try state.present.set("depth", depth)',
    '    try state.present.set("lighting", lighting)',
    "    state.textures = [image, depth, lighting, lightingDepth]",
    "    state.size = size",
    "}",
    "",
    "/// The WGSL file `name` from the app's bundle.",
    "func readSource(_ name: String) throws -> String {",
    '    let url = Bundle.main.url(forResource: name, withExtension: "wgsl")!',
    "    return try String(contentsOf: url, encoding: .utf8)",
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
    "import kotlin.math.sqrt",
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
    " * The key light shines down on the fish as daylight through the water would,",
    " * from above and to the camera's right, and the rim light from behind the fish. The",
    " * toon fish takes its key light from nearer overhead, which lays its shadow band along",
    " * the belly, and more ambient light, since a cartoon shades with a darker tone of the same",
    " * color. Unlit takes no light at all. KEY and AMBIENT list the styles in turn: PBR, toon, unlit.",
    " */",
    "private val KEY = listOf(listOf(-0.6, -1.0, -0.5), listOf(-0.3, -1.0, -0.35), listOf(-0.5, -0.6, -1.0))",
    "private val RIM = listOf(0.6, 0.2, 1.0)",
    "private val AMBIENT = listOf(listOf(0.1, 0.1, 0.12), listOf(0.3, 0.32, 0.36), listOf(0.1, 0.1, 0.12))",
    "/**",
    " * How fast the fish turns before the first drag, in radians per second. It turns its",
    " * nose toward the camera first, so the first toon turn shows its side.",
    " */",
    "private const val SPIN = -0.4",
    "/** How long each style shows before the next one takes over, in seconds, until the first M. */",
    "private const val HOLD = 4.0",
    "/**",
    " * The PBR fish looks wetter than its file describes it: its scales keep a third of",
    " * their roughness, and their bumps stand out twice as far.",
    " */",
    "private const val GLOSS = 0.35",
    "private const val BUMPS = 2.0",
    "/**",
    " * Each light's camera sees a box around the fish, this far each way from its",
    " * middle in meters, and the map it draws is this many texels across.",
    " */",
    "private const val REACH = 0.4",
    "private const val MAP_SIZE = 1024u",
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
    "/**",
    " * A camera that looks along a light, the shader the fish draws with in that light's",
    " * pass, and the map the pass draws: how far along the light each point lies.",
    " */",
    "class LightView(val camera: Camera, val caster: Shader, val map: Texture)",
    "",
    "class State(",
    "    val scene: Scene,",
    "    val present: Shader,",
    "    val camera: Camera,",
    "    val key: Light,",
    "    val rim: Light,",
    "    val keyView: LightView,",
    "    val rimView: LightView,",
    "    /** The shade pass's shader, which reads the maps of both light views. */",
    "    val receiver: Shader,",
    "    val fish: Pass,",
    "    val shade: Pass,",
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
    "    /** The textures [fit] made for the view's size. */",
    "    var textures: List<Texture> = emptyList()",
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
    "    // The fish pass clears to transparent, so present.wgsl can lay the fish over the water.",
    "    // The first frame gives it an image and a depth buffer the size of the view.",
    "    val fish = scene.getPass(0u)!!",
    "    fish.setClearColor(listOf(0f, 0f, 0f, 0f))",
    "",
    "    // A camera along each light draws the fish into a map in a pass of its own: the key",
    "    // light's for the shadows, and the rim light's for the light that comes through the",
    "    // fins. The shade pass draws the fish again from the camera, with what both maps say",
    "    // of each point, for present.wgsl to use.",
    '    val depthSource = readText(assets, "depth.wgsl")',
    '    val keyView = createView(renderer, scene, "key", depthSource)',
    '    val rimView = createView(renderer, scene, "rim", depthSource)',
    '    val receiver = Shader(listOf("noise/worley2", readText(assets, "shade.wgsl")))',
    '    receiver.set("key_map", keyView.map)',
    '    receiver.set("rim_map", rimView.map)',
    '    val shade = Pass("shade")',
    "    shade.setClearColor(listOf(0f, 0f, 0f, 0f))",
    "    scene.addPass(shade)",
    "",
    "    // Each part of the fish draws in PBR, wetter than the file describes it, and in the",
    "    // key, rim and shade passes. It gets a toon and an unlit Model over its mesh too, which",
    "    // both sample its base color texture, so that stays one upload. The lists follow the",
    "    // styles in turn, and so does present.wgsl's look.style.",
    "    val styles = List(3) { mutableListOf<Model>() }",
    "    for (model in scene.models()) {",
    "        val pbr = model.material() as? PbrMaterial ?: continue",
    "        pbr.roughness(GLOSS).normalScale(BUMPS)",
    "        // Light in three flat steps, a hard white glint, and a pale rim along the edges.",
    "        val toon = ToonMaterial()",
    "            .bands(3u)",
    "            .highlight(listOf(1.0, 1.0, 1.0))",
    "            .highlightSize(0.012)",
    "            .rim(listOf(0.2, 0.35, 0.4))",
    "            .rimWidth(0.12)",
    "        toon.baseColorMap().texture(pbr.baseColorMap())",
    "        val unlit = UnlitMaterial()",
    "        unlit.baseColorMap().texture(pbr.baseColorMap())",
    "        styles[0].add(model)",
    '        styles[0].add(restyle(scene, "key", model, keyView.caster))',
    '        styles[0].add(restyle(scene, "rim", model, rimView.caster))',
    '        styles[0].add(restyle(scene, "shade", model, receiver))',
    "        styles[1].add(restyle(scene, fish.name(), model, toon))",
    "        styles[2].add(restyle(scene, fish.name(), model, unlit))",
    "    }",
    "",
    "    val camera = Camera.perspective(0.6, 1.0, 0.01, 10.0)",
    "    val key = Light.directional(KEY[0], listOf(1.0, 0.96, 0.9)).setIntensity(3.0)",
    "    val rim = Light.directional(RIM, listOf(0.6, 0.7, 1.0)).setIntensity(1.0)",
    "    scene.add(camera).add(key).add(rim).ambient(AMBIENT[0])",
    "    // The camera draws the shade pass too.",
    '    scene.addTo("shade", camera)',
    "",
    '    val present = Shader(LIBRARY + readText(assets, "present.wgsl"))',
    ...setCalls("kotlin", values),
    '    scene.addPass(Pass.fromShader("present", present))',
    "",
    "    val state = State(scene, present, camera, key, rim, keyView, rimView, receiver, fish, shade, styles)",
    "    show(state, 0)",
    "    return state",
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
    "    // Both lights turn with the camera, so the side in view stays lit, and each",
    "    // light's camera follows its light.",
    "    val keyDirection = turn(KEY[state.style], state.yaw)",
    "    val rimDirection = turn(RIM, state.yaw)",
    "    state.key.setDirection(keyDirection)",
    "    state.rim.setDirection(rimDirection)",
    '    aim(state.receiver, "key", state.keyView, keyDirection)',
    '    aim(state.receiver, "rim", state.rimView, rimDirection)',
    '    state.receiver.set("time", time)',
    "    renderer.render(state.scene, target)",
    "}",
    "",
    "/**",
    " * A camera that looks along a light, with the pass [name] in which it draws the fish",
    " * into a map of how far along the light each point lies, with the shader [source].",
    " */",
    "suspend fun createView(renderer: Renderer, scene: Scene, name: String, source: String): LightView {",
    "    val camera = Camera.orthographic(-REACH, REACH, -REACH, REACH, 1 - REACH, 1 + REACH)",
    "    val caster = Shader(source).doubleSided(true)",
    "    val pass = Pass(name)",
    "    pass.setClearColor(listOf(1f, 1f, 1f, 1f))",
    "    scene.addPass(pass).addTo(name, camera)",
    "    val usage = TextureUsage.RENDER_ATTACHMENT or TextureUsage.TEXTURE_BINDING",
    "    val size = listOf(MAP_SIZE, MAP_SIZE)",
    "    val map = renderer.createStorageTexture(size, TextureFormat.RGBA16_FLOAT, usage = usage)",
    "    pass.setTarget(map)",
    "    pass.setDepthTarget(renderer.createDepthTexture(size))",
    "    return LightView(camera, caster, map)",
    "}",
    "",
    "/**",
    " * Points the camera of [view], the light [name]'s, at the fish along [direction]",
    " * from a meter away, and tells the shade pass's [receiver] how that camera sees.",
    " */",
    "fun aim(receiver: Shader, name: String, view: LightView, direction: List<Double>) {",
    "    val length = sqrt(direction.sumOf { it * it })",
    "    val position = CENTER.zip(direction) { center, axis -> center - axis / length }",
    "    view.camera.lookAt(position, CENTER, listOf(0.0, 1.0, 0.0))",
    '    receiver.set("$name.view_proj", view.camera.viewProj())',
    '    receiver.set("$name.direction", direction)',
    "}",
    "",
    "/** A Model in the pass named [pass] that draws the mesh of [model] with [material], where [model] is. */",
    "fun restyle(scene: Scene, pass: String, model: Model, material: Any): Model {",
    "    val copy = Model(model.mesh(), material)",
    "    copy.setTransform(model.transform())",
    "    scene.addTo(pass, copy)",
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
    " * the tone map, and a depth texture keeps its far side behind its near side. The",
    " * shade pass draws into an image and a depth texture of its own. present.wgsl",
    " * reads all but the last. A new view size gets a new set.",
    " */",
    "suspend fun fit(state: State, renderer: Renderer, size: List<Float>) {",
    "    val pixels = listOf(maxOf(size[0].toUInt(), 1u), maxOf(size[1].toUInt(), 1u))",
    "    val usage = TextureUsage.RENDER_ATTACHMENT or TextureUsage.TEXTURE_BINDING",
    "    val image = renderer.createStorageTexture(pixels, TextureFormat.RGBA16_FLOAT, usage = usage)",
    "    val depth = renderer.createDepthTexture(pixels)",
    "    val lighting = renderer.createStorageTexture(pixels, TextureFormat.RGBA8_UNORM, usage = usage)",
    "    val lightingDepth = renderer.createDepthTexture(pixels)",
    "",
    "    // Unregistering frees the old set now, before its handles go.",
    "    state.textures.forEach { renderer.unregisterTexture(it.id()) }",
    "    state.fish.setTarget(image)",
    "    state.fish.setDepthTarget(depth)",
    "    state.shade.setTarget(lighting)",
    "    state.shade.setDepthTarget(lightingDepth)",
    '    state.present.set("image", image)',
    '    state.present.set("depth", depth)',
    '    state.present.set("lighting", lighting)',
    "    state.textures = listOf(image, depth, lighting, lightingDepth)",
    "    state.size = size",
    "}",
    "",
    "/** The text of the asset [name], such as a WGSL file. */",
    "fun readText(assets: AssetManager, name: String): String =",
    "    assets.open(name).bufferedReader().use { it.readText() }",
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
    "Barramundi Fish by Microsoft (CC0), loaded from a glTF file and lit by two lights, in three styles: wet PBR with the shadows of its own fins, ripples of light over it and light glowing through its fins, toon with ink outlines, and unlit. The styles take turns until you click the canvas and press M, or tap Style on a touch screen, to switch yourself. Drag to turn the fish, and scroll or pinch to move closer.",
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
  files: { "present.wgsl": PRESENT, "depth.wgsl": DEPTH, "shade.wgsl": SHADE },
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
