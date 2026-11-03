// A small valley you walk through, built from the scene layer: Scene, Model,
// Mesh, Material, Camera and Light. A sky pass draws first, and the
// valley draws over it.

import { formatValue } from "../codegen.js";

const SKY = /* wgsl */ `// The sky behind the valley: a gradient from the horizon up, a glow around the
// sun, and stars at night. main.js points the camera vectors at the view.

struct Sky {
  forward: vec3<f32>,
  right: vec3<f32>,
  up: vec3<f32>,
  sun: vec3<f32>,
  resolution: vec2<f32>,
  lens: f32,
  day: f32,
  zenith: vec3<f32>,
  horizon: vec3<f32>,
  sunset: vec3<f32>,
  night: vec3<f32>,
}

@group(0) @binding(0) var<uniform> sky: Sky;

struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) uv: vec2<f32>,
}

// Fullscreen triangle: three vertices that cover the canvas.
@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VertexOutput {
  var corners = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
  var out: VertexOutput;
  out.position = vec4<f32>(corners[i], 0.0, 1.0);
  out.uv = corners[i];
  return out;
}

fn hash(cell: vec3<f32>) -> f32 {
  return fract(sin(dot(cell, vec3<f32>(127.1, 311.7, 74.7))) * 43758.5453);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  let aspect = sky.resolution.x / max(sky.resolution.y, 1.0);
  let ray = normalize(sky.forward + (sky.right * in.uv.x * aspect + sky.up * in.uv.y) * sky.lens);
  let height = max(ray.y, 0.0);

  // Low sun turns the horizon warm around it.
  let low = 1.0 - smoothstep(0.0, 0.35, abs(sky.sun.y));
  let toward = max(dot(ray, sky.sun), 0.0);
  let day_sky = mix(sky.horizon, sky.zenith, pow(height, 0.6));
  var color = mix(sky.night, day_sky, sky.day);
  color = mix(color, sky.sunset, low * pow(toward, 6.0) * (1.0 - height) * 0.9);
  color += sky.sunset * low * pow(toward, 64.0) * 0.6;

  // The sun's disc, and stars once it is dark.
  color += vec3<f32>(20.0, 17.0, 12.0) * smoothstep(0.9995, 0.9998, toward) * smoothstep(-0.02, 0.02, sky.sun.y);
  let cell = floor(ray * 180.0);
  let star = step(0.997, hash(cell)) * smoothstep(0.05, 0.3, height) * (1.0 - sky.day);
  color += vec3<f32>(star * 0.6);
  // The colors above are linear light. The screen takes them encoded, as PBR writes them.
  return vec4<f32>(linear_to_srgb(color), 1.0);
}
`;

const PROGRAM = /* js */ `import { AlphaMode, Camera, Light, Material, Mesh, Model, Pass, Scene, Shader, Vertex } from "fragmentcolor";
import SKY_SOURCE from "./sky.wgsl?raw";

const LIBRARY = ["color/linear_to_srgb"];

// The valley is SIZE meters across, and a day lasts DAY seconds. Hold T to hurry it.
const SIZE = 72;
const DAY = 180;
const TREES = 140;
const ROCKS = 36;
const LAMPS = 8;
const GRASS = 8000;
const WATER = -1.2;

// Blade colors from the soil to the tip, in three steps, for three kinds of tuft.
const LUSH = [[0.04, 0.12, 0.02, 1], [0.07, 0.2, 0.03, 1], [0.13, 0.29, 0.05, 1]];
const DRY = [[0.05, 0.12, 0.02, 1], [0.13, 0.21, 0.04, 1], [0.26, 0.3, 0.09, 1]];
const SHORT = [[0.035, 0.1, 0.02, 1], [0.055, 0.16, 0.025, 1], [0.09, 0.23, 0.04, 1]];

// Meters per second (Shift runs), eye height in meters, and radians of turn per pixel dragged.
const SPEED = 3.5;
const EYE = 1.6;
const TURN = 0.005;
const FOV = 1.1;
const TAU = Math.PI * 2;

export async function setup(renderer, target) {
  const sky = await Shader.fetch([...LIBRARY, SKY_SOURCE]);
  sky.set("sky.zenith", [0.06, 0.2, 0.6]);
  sky.set("sky.horizon", [0.5, 0.65, 0.85]);
  sky.set("sky.sunset", [1.6, 0.55, 0.15]);
  sky.set("sky.night", [0.002, 0.004, 0.012]);

  // The valley draws over the sky. A pass that draws models gets a depth buffer,
  // so the nearest surface wins at each pixel.
  const world = new Pass("world");
  world.loadPrevious();

  const scene = new Scene();
  scene.addPass(Pass.fromShader("sky", sky)).addPass(world);
  const add = (object) => scene.addTo("world", object);

  add(new Model(ground(), Material.pbr().roughness(0.95)));
  const water = Material.pbr().baseColor([0.02, 0.08, 0.1, 0.8]).roughness(0.08).alphaMode(AlphaMode.Blend);
  add(new Model(square(SIZE, WATER), water));

  // Trees and rocks share one mesh each, so each kind draws in one call.
  const obstacles = [];
  const tree = pine();
  const bark = Material.pbr().roughness(0.8);
  for (let i = 0; obstacles.length < TREES && i < TREES * 8; i++) {
    const [x, z] = spot(i, SIZE * 0.42);
    if (height(x, z) < WATER + 0.5 || height(x, z) > 5 || Math.hypot(x, z) < 6) continue;
    const scale = 0.7 + random(i, 2) * 0.7;
    add(place(tree, bark, [x, height(x, z) - 0.1, z], random(i, 3) * TAU, [scale, scale, scale]));
    obstacles.push([x, z, 0.35 * scale]);
  }
  const rock = boulder();
  const stone = Material.pbr().roughness(0.7);
  for (let i = 0; i < ROCKS; i++) {
    const [x, z] = spot(i + 5000, SIZE * 0.45);
    const scale = 0.4 + random(i, 4) * 0.9;
    const stretch = [scale * (1 + random(i, 5)), scale * 0.7, scale];
    add(place(rock, stone, [x, height(x, z) - 0.15 * scale, z], random(i, 6) * TAU, stretch));
    obstacles.push([x, z, scale]);
  }

  // Grass grows in meadows on the gentle slopes between the water and the rocks,
  // thicker where two slow waves meet. Its three kinds of tuft share one material.
  const kinds = [tuft(9, 0.55, LUSH), tuft(6, 0.85, DRY), tuft(12, 0.3, SHORT)];
  const grass = Material.pbr().roughness(0.85);
  for (let i = 0; i < GRASS; i++) {
    const [x, z] = spot(i + 9000, SIZE * 0.4);
    const floor = height(x, z);
    const slope = Math.hypot(height(x + 0.5, z) - floor, height(x, z + 0.5) - floor);
    const waves = Math.sin(x * 0.23 + 1.7) * Math.cos(z * 0.19 - 0.6);
    const meadow = 0.55 + 0.35 * waves + 0.2 * Math.sin((x + z) * 0.51);
    if (floor < WATER + 0.4 || floor > 5 || slope > 0.25 || random(i, 10) > meadow) continue;
    const kind = random(i, 13) < 0.12 ? 1 : meadow > 0.7 ? 0 : 2;
    const size = 0.7 + random(i, 11) * 0.6;
    const stretch = [size, size * (0.8 + random(i, 14) * 0.5), size];
    add(place(kinds[kind], grass, [x, floor - 0.02, z], random(i, 12) * TAU, stretch));
  }

  // Lamps ring the clearing where the walk starts. Each one carries a point light.
  const post = column([0.02, 0.018, 0.016, 1]);
  const metal = Material.pbr().metallic(0.6).roughness(0.4);
  const bulb = Material.pbr().baseColor([1, 0.85, 0.6, 1]);
  const lamps = [];
  for (let i = 0; i < LAMPS; i++) {
    const angle = (i / LAMPS) * TAU;
    const [x, z] = [Math.cos(angle) * 5, Math.sin(angle) * 5];
    const y = height(x, z);
    add(place(post, metal, [x, y, z], 0, [0.12, 2.6, 0.12]));
    add(place(post, bulb, [x, y + 2.6, z], 0.785, [0.32, 0.32, 0.32]));
    const light = Light.point([x, y + 2.75, z], [1, 0.72, 0.42]).setRange(14);
    lamps.push(light);
    add(light);
    obstacles.push([x, z, 0.3]);
  }

  const sun = Light.directional([0, -1, 0], [1, 1, 1]);
  const moon = Light.directional([0.3, -1, 0.5], [0.45, 0.55, 0.85]);
  // The first frame sets the aspect from the canvas, which may have no size yet.
  const camera = Camera.perspective(FOV, 1, 0.05, 200);
  add(sun);
  add(moon);
  add(camera);

  return {
    scene, sky, sun, moon, lamps, bulb, camera, obstacles, size: [0, 0],
    walker: { x: 0, z: 3, y: height(0, 3) + EYE, rise: 0, yaw: -0.7, pitch: -0.05, walked: 0 },
    clock: 0.28 * DAY,
    time: 0,
  };
}

export function frame(state, renderer, target, time, size, pointer, gesture = { zoom: 0, pan: [0, 0] }, keys = { down: new Set(), stick: [0, 0] }) {
  const delta = Math.min(Math.max(time - state.time, 0), 0.1);
  state.time = time;
  state.clock += delta * (keys.down.has("KeyT") ? 20 : 1);
  walk(state, delta, gesture, keys);
  light(state);
  fit(state, size);
  state.sky.set("sky.resolution", size);
  renderer.render(state.scene, target);
}

// A new canvas size changes the camera's aspect.
function fit(state, size) {
  if (size[0] === state.size[0] && size[1] === state.size[1]) return;
  state.camera.setAspect(Math.max(size[0], 1) / Math.max(size[1], 1));
  state.size = size;
}

function walk(state, delta, gesture, keys) {
  const walker = state.walker;
  const held = (...codes) => (codes.some((code) => keys.down.has(code)) ? 1 : 0);
  walker.yaw -= gesture.pan[0] * TURN + (held("ArrowRight") - held("ArrowLeft")) * 2 * delta;
  walker.pitch = Math.min(Math.max(walker.pitch - gesture.pan[1] * TURN, -1.4), 1.4);

  // Forward and right on the ground, from the keys, the stick and the wheel.
  let ahead = held("KeyW", "ArrowUp") - held("KeyS", "ArrowDown") + keys.stick[1];
  let side = held("KeyD") - held("KeyA") + keys.stick[0];
  const length = Math.hypot(ahead, side);
  if (length > 1) [ahead, side] = [ahead / length, side / length];
  const speed = SPEED * (1 + held("ShiftLeft", "ShiftRight"));
  const [sin, cos] = [Math.sin(walker.yaw), Math.cos(walker.yaw)];
  const step = ahead * speed * delta + gesture.zoom * 2;
  let x = walker.x - sin * step + cos * side * speed * delta;
  let z = walker.z - cos * step - sin * side * speed * delta;

  // Trees, rocks and lamps push back, the water stops you, and the hills keep you in.
  for (const [stopX, stopZ, radius] of state.obstacles) {
    const [awayX, awayZ] = [x - stopX, z - stopZ];
    const gap = Math.hypot(awayX, awayZ);
    const reach = radius + 0.3;
    if (gap > 0 && gap < reach) [x, z] = [stopX + (awayX / gap) * reach, stopZ + (awayZ / gap) * reach];
  }
  const far = Math.hypot(x, z) / (SIZE * 0.42);
  if (far > 1) [x, z] = [x / far, z / far];
  if (height(x, z) > WATER + 0.1) {
    walker.walked += Math.hypot(x - walker.x, z - walker.z);
    [walker.x, walker.z] = [x, z];
  }

  // Space jumps. Feet land on the ground, and the head bobs while walking.
  const floor = height(walker.x, walker.z) + EYE;
  if (walker.y <= floor + 0.01 && keys.down.has("Space")) walker.rise = 4.5;
  walker.rise -= 12 * delta;
  walker.y = Math.max(floor, walker.y + walker.rise * delta);
  if (walker.y <= floor) walker.rise = 0;
  const bob = Math.sin(walker.walked * 2.4) * 0.05;

  const look = [-sin * Math.cos(walker.pitch), Math.sin(walker.pitch), -cos * Math.cos(walker.pitch)];
  const eye = [walker.x, walker.y + bob, walker.z];
  const toward = [eye[0] + look[0], eye[1] + look[1], eye[2] + look[2]];
  state.camera.lookAt(eye, toward, [0, 1, 0]);
  const right = normalize(cross(look, [0, 1, 0]));
  state.sky.set("sky.forward", look);
  state.sky.set("sky.right", right);
  state.sky.set("sky.up", cross(right, look));
  state.sky.set("sky.lens", Math.tan(FOV / 2));
}

// The sun turns once a day, above the horizon for two thirds of it. Lamps glow from dusk to dawn.
function light(state) {
  const angle = (state.clock / DAY) * TAU;
  const sun = normalize([Math.cos(angle), Math.sin(angle) + 0.45, 0.35]);
  const day = smoothstep(-0.08, 0.12, sun[1]);
  const low = 1 - smoothstep(0, 0.35, Math.abs(sun[1]));
  const warm = [1, 0.95 - low * 0.4, 0.85 - low * 0.6];
  state.sun.setDirection(sun.map((v) => -v)).setColor(warm).setIntensity(3.2 * day);
  state.moon.setIntensity(0.35 * (1 - day));
  state.scene.ambient(mix([0.025, 0.03, 0.06], [0.28, 0.32, 0.4], day));
  for (const lamp of state.lamps) lamp.setIntensity(5 * (1 - day));
  state.bulb.emissive(mix([0.3, 0.24, 0.14], [1.2, 0.75, 0.3], 1 - day));
  state.sky.set("sky.sun", sun);
  state.sky.set("sky.day", day);
}

// The ground: a flat clearing in rolling hills, a pond, and a rim that rises toward the edge.
function height(x, z) {
  const middle = Math.hypot(x, z);
  const angle = Math.atan2(z, x);
  const hills = Math.sin(x * 0.15) * Math.cos(z * 0.12) * 1.2 + Math.sin(x * 0.31 + z * 0.23) * 0.45;
  const edge = 0.6 + Math.sin(angle * 3) * 0.06 + Math.sin(angle * 7 + 1) * 0.04;
  const rim = Math.max(0, middle / (SIZE / 2) - edge);
  const peaks = 1 + Math.sin(angle * 5 + 2) * 0.35;
  const pond = Math.exp(-((x - 11) ** 2 + (z + 9) ** 2) / 45);
  return hills * smoothstep(3, 12, middle) + rim * rim * 60 * peaks - pond * 3;
}

function ground() {
  const cells = 72;
  const cell = SIZE / cells;
  const triangles = [];
  for (let i = 0; i < cells; i++) {
    for (let j = 0; j < cells; j++) {
      const [x, z] = [-SIZE / 2 + i * cell, -SIZE / 2 + j * cell];
      const corner = (alongX, alongZ) => [x + alongX, height(x + alongX, z + alongZ), z + alongZ];
      const [here, ahead] = [corner(0, 0), corner(0, cell)];
      const [beside, across] = [corner(cell, 0), corner(cell, cell)];
      triangles.push([here, ahead, beside], [beside, ahead, across]);
    }
  }
  return facets(triangles, (triangle, normal, index) => {
    const y = (triangle[0][1] + triangle[1][1] + triangle[2][1]) / 3;
    if (y < WATER + 0.3) return [0.42, 0.33, 0.18, 1];
    if (y > 10) return [0.7, 0.72, 0.76, 1];
    if (normal[1] < 0.6 || y > 6) return [0.13, 0.12, 0.11, 1];
    if (normal[1] < 0.85) return [0.035, 0.1, 0.02, 1];
    return random(index, 1) < 0.5 ? [0.05, 0.16, 0.025, 1] : [0.07, 0.19, 0.03, 1];
  });
}

function square(size, y) {
  const [low, high] = [-size / 2, size / 2];
  const white = [1, 1, 1, 1];
  return facets([[[low, y, low], [low, y, high], [high, y, low], white], [[high, y, low], [low, y, high], [high, y, high], white]]);
}

// A pine: a five-sided trunk under two stacked cones.
function pine() {
  const trunk = cone(0.18, 0, 1.3, 5, true).map((triangle) => [...triangle, [0.12, 0.06, 0.03, 1]]);
  const crown = [...cone(1.2, 1, 3.2, 7), ...cone(0.85, 2.2, 4.3, 7)].map((triangle) => [...triangle, [0.02, 0.09, 0.035, 1]]);
  return facets([...trunk, ...crown]);
}

// A tuft: blades fanned around one root, each bending outward in three steps that
// taper to a tip and lighten from the soil up. A blade has a face on each side,
// since the material draws front faces only, and its normals lean toward the sky,
// so it takes the light the way the ground does.
function tuft(blades, tall, colors) {
  const mesh = new Mesh();
  for (let blade = 0; blade < blades; blade++) {
    const [cos, sin] = [Math.cos(blade * 2.4), Math.sin(blade * 2.4)];
    const top = tall * (0.7 + random(blade, 20) * 0.6);
    const lean = 0.12 + random(blade, 21) * 0.25;
    // From the root up: how far out, how high, and half the width.
    const rings = [[0, 0, 0.035], [lean * 0.25, top * 0.45, 0.028], [lean * 0.65, top * 0.8, 0.016], [lean, top, 0]];
    const corner = ([out, y, half], side) => [cos * out + sin * half * side, y, -sin * out + cos * half * side];
    for (let step = 0; step < 3; step++) {
      const [low, high] = [rings[step], rings[step + 1]];
      const faces = [[corner(low, -1), corner(low, 1), corner(high, 1)]];
      if (step < 2) faces.push([corner(low, -1), corner(high, 1), corner(high, -1)]);
      for (const face of faces) {
        for (const side of [face, [...face].reverse()]) {
          const normal = normalize(cross(minus(side[1], side[0]), minus(side[2], side[0])));
          const up = normalize([normal[0] * 0.4, normal[1] * 0.4 + 1, normal[2] * 0.4]);
          mesh.addVertices(side.map((point) => Vertex.pbr(point).set(Vertex.NORMAL, up).set(Vertex.COLOR0, colors[step])));
        }
      }
    }
  }
  return mesh;
}

// A cone, or with \`prism\` set, a prism with the same sides.
function cone(radius, bottom, top, sides, prism = false) {
  const rim = (angle, y) => [Math.cos(angle) * radius, y, -Math.sin(angle) * radius];
  const triangles = [];
  for (let face = 0; face < sides; face++) {
    const [start, end] = [(face / sides) * TAU, ((face + 1) / sides) * TAU];
    if (prism) triangles.push([rim(start, bottom), rim(end, bottom), rim(end, top)], [rim(start, bottom), rim(end, top), rim(start, top)]);
    else triangles.push([rim(start, bottom), rim(end, bottom), [0, top, 0]]);
    triangles.push([rim(end, bottom), rim(start, bottom), [0, bottom, 0]]);
  }
  return triangles;
}

function column(color) {
  return facets(cone(0.7, 0, 1, 4, true).map((triangle) => [...triangle, color]));
}

// An icosahedron: twenty faces, shaded flat.
function boulder() {
  const golden = 1.618;
  const corners = [
    [-1, golden, 0], [1, golden, 0], [-1, -golden, 0], [1, -golden, 0],
    [0, -1, golden], [0, 1, golden], [0, -1, -golden], [0, 1, -golden],
    [golden, 0, -1], [golden, 0, 1], [-golden, 0, -1], [-golden, 0, 1],
  ].map(normalize);
  const faces = [[0, 11, 5], [0, 5, 1], [0, 1, 7], [0, 7, 10], [0, 10, 11], [1, 5, 9], [5, 11, 4], [11, 10, 2], [10, 7, 6], [7, 1, 8],
    [3, 9, 4], [3, 4, 2], [3, 2, 6], [3, 6, 8], [3, 8, 9], [4, 9, 5], [2, 4, 11], [6, 2, 10], [8, 6, 7], [9, 8, 1]];
  return facets(faces.map((face) => face.map((corner) => corners[corner])), (triangle, normal, index) => {
    const shade = 0.16 + random(index, 7) * 0.08;
    return [shade, shade * 0.97, shade * 0.92, 1];
  });
}

// A flat-shaded mesh: every triangle gets three vertices of its own, its face
// normal and a color, which \`paint\` picks or the triangle carries fourth.
function facets(triangles, paint = (triangle) => triangle[3]) {
  const mesh = new Mesh();
  triangles.forEach((triangle, index) => {
    const [first, second, third] = triangle;
    const normal = normalize(cross(minus(second, first), minus(third, first)));
    const color = paint(triangle, normal, index);
    mesh.addVertices([first, second, third].map((point) => Vertex.pbr(point).set(Vertex.NORMAL, normal).set(Vertex.COLOR0, color)));
  });
  return mesh;
}

function place(mesh, material, position, turn, scale) {
  const model = new Model(mesh, material);
  model.translate(position);
  model.rotate([0, 1, 0], turn);
  model.scale(scale);
  return model;
}

// The same pseudo-random numbers in every language: a 32-bit integer hash.
function random(index, salt) {
  let hash = Math.imul(index ^ Math.imul(salt, 0x9e3779b9), 0x85ebca6b) >>> 0;
  hash = Math.imul(hash ^ (hash >>> 13), 0xc2b2ae35) >>> 0;
  return ((hash ^ (hash >>> 16)) >>> 0) / 4294967296;
}

function spot(index, radius) {
  const distance = Math.sqrt(random(index, 8)) * radius;
  const angle = random(index, 9) * TAU;
  return [Math.cos(angle) * distance, Math.sin(angle) * distance];
}

const minus = (first, second) => [first[0] - second[0], first[1] - second[1], first[2] - second[2]];
const cross = (first, second) => [
  first[1] * second[2] - first[2] * second[1],
  first[2] * second[0] - first[0] * second[2],
  first[0] * second[1] - first[1] * second[0],
];
const normalize = (vector) => vector.map((value) => value / Math.hypot(...vector));
const mix = (start, end, amount) => start.map((low, axis) => low + (end[axis] - low) * amount);
const smoothstep = (low, high, value) => {
  const amount = Math.min(Math.max((value - low) / (high - low), 0), 1);
  return amount * amount * (3 - 2 * amount);
};
`;

// The values main.js sets in setup, in order.
const VALUES = {
  "sky.zenith": [0.06, 0.2, 0.6],
  "sky.horizon": [0.5, 0.65, 0.85],
  "sky.sunset": [1.6, 0.55, 0.15],
  "sky.night": [0.002, 0.004, 0.012],
};

// From 0.12.7, Material::pbr encodes its colors for display and lights the unevenly scaled rocks the way they face,
// and a pass that draws models gets a depth buffer of its own.
const NEEDS = "Needs FragmentColor 0.12.7 or later.";

function quote(text) {
  return JSON.stringify(String(text));
}

function describe(model) {
  return { library: model.library ?? [], sky: model.values ?? Object.entries(VALUES) };
}

// `head(a, b)tail` on one line when it fits in `width`, otherwise one part per line.
function wrap(indent, head, parts, tail, width) {
  const line = `${indent}${head}${parts.join(", ")}${tail}`;
  if (line.length <= width) return line;
  return [`${indent}${head}`, ...parts.map((part) => `${indent}    ${part},`), `${indent}${tail}`].join("\n");
}

// One `set` call per value, spelled the way `lang` writes it.
function setCalls(lang, name, values, { before = "", after = "" } = {}) {
  return values
    .map(([key, value, scalar]) => `    ${before}${name}.set(${quote(key)}, ${formatValue(lang, value, scalar)})${after}`)
    .join("\n");
}

function writeRust({ library, sky }) {
  const shader = wrap("    ", "let sky = Shader::new([", [...library.map(quote), "SKY_SOURCE"], "])?;", 100);
  return `// ${NEEDS}

use std::collections::HashSet;
use std::error::Error;
use std::f32::consts::TAU;

use fragmentcolor::{
    AlphaMode, Camera, Light, Material, Mesh, Model, Pass, Renderer, Scene, Shader, Target, Vertex,
};

const SKY_SOURCE: &str = include_str!("sky.wgsl");

/// The valley is \`SIZE\` meters across, and a day lasts \`DAY\` seconds. Hold T to hurry it.
const SIZE: f32 = 72.0;
const DAY: f32 = 180.0;
const TREES: usize = 140;
const ROCKS: u32 = 36;
const LAMPS: u32 = 8;
const GRASS: u32 = 8000;
const WATER: f32 = -1.2;

/// Blade colors from the soil to the tip, in three steps, for three kinds of tuft.
const LUSH: [Color; 3] = [[0.04, 0.12, 0.02, 1.0], [0.07, 0.2, 0.03, 1.0], [0.13, 0.29, 0.05, 1.0]];
const DRY: [Color; 3] = [[0.05, 0.12, 0.02, 1.0], [0.13, 0.21, 0.04, 1.0], [0.26, 0.3, 0.09, 1.0]];
const SHORT: [Color; 3] = [[0.035, 0.1, 0.02, 1.0], [0.055, 0.16, 0.025, 1.0], [0.09, 0.23, 0.04, 1.0]];

/// Meters per second (Shift runs), eye height in meters, and radians of turn per pixel dragged.
const SPEED: f32 = 3.5;
const EYE: f32 = 1.6;
const TURN: f32 = 0.005;
const FOV: f32 = 1.1;

type Point = [f32; 3];
type Color = [f32; 4];

/// What the wheel, a drag and a pinch did since the last frame: zoom in doublings
/// (positive zooms in) and pan in window pixels.
#[derive(Clone, Copy, Default)]
struct Gesture {
    zoom: f32,
    pan: [f32; 2],
}

/// The keys held down, named the way \`KeyboardEvent.code\` names them
/// ("KeyW", "ArrowUp", "Space"), and a touch stick as \`[x, y]\` from -1 to 1, y up.
#[derive(Default)]
struct Keys {
    down: HashSet<String>,
    stick: [f32; 2],
}

impl Keys {
    /// 1 while any of \`codes\` is held down, 0 otherwise.
    fn held(&self, codes: &[&str]) -> f32 {
        if codes.iter().any(|code| self.down.contains(*code)) {
            1.0
        } else {
            0.0
        }
    }
}

struct Walker {
    x: f32,
    y: f32,
    z: f32,
    rise: f32,
    yaw: f32,
    pitch: f32,
    walked: f32,
}

struct State {
    scene: Scene,
    sky: Shader,
    sun: Light,
    moon: Light,
    lamps: Vec<Light>,
    bulb: Material,
    camera: Camera,
    obstacles: Vec<[f32; 3]>,
    size: [u32; 2],
    walker: Walker,
    clock: f32,
    last: Option<f32>,
}

// Call setup once, then frame on every redraw of your window.
fn setup() -> Result<State, Box<dyn Error>> {
${shader}
${setCalls("rust", "sky", sky, { after: "?;" })}

    // The valley draws over the sky. A pass that draws models gets a depth buffer,
    // so the nearest surface wins at each pixel.
    let world = Pass::new("world");
    world.load_previous();

    let scene = Scene::new();
    scene.add_pass(&Pass::from_shader("sky", &sky)).add_pass(&world);

    scene.add_to("world", &Model::new(ground(), Material::pbr().roughness(0.95)))?;
    let water = Material::pbr()
        .base_color([0.02, 0.08, 0.1, 0.8])
        .roughness(0.08)
        .alpha_mode(AlphaMode::Blend);
    scene.add_to("world", &Model::new(square(SIZE, WATER), water))?;

    // Trees and rocks share one mesh each, so each kind draws in one call.
    let mut obstacles = Vec::new();
    let tree = pine();
    let bark = Material::pbr().roughness(0.8);
    for index in 0..(TREES * 8) as u32 {
        if obstacles.len() == TREES {
            break;
        }
        let [x, z] = spot(index, SIZE * 0.42);
        let floor = height(x, z);
        if floor < WATER + 0.5 || floor > 5.0 || x.hypot(z) < 6.0 {
            continue;
        }
        let scale = 0.7 + random(index, 2) * 0.7;
        let turn = random(index, 3) * TAU;
        scene.add_to("world", &place(&tree, &bark, [x, floor - 0.1, z], turn, [scale; 3]))?;
        obstacles.push([x, z, 0.35 * scale]);
    }
    let rock = boulder();
    let stone = Material::pbr().roughness(0.7);
    for index in 0..ROCKS {
        let [x, z] = spot(index + 5000, SIZE * 0.45);
        let scale = 0.4 + random(index, 4) * 0.9;
        let stretch = [scale * (1.0 + random(index, 5)), scale * 0.7, scale];
        let position = [x, height(x, z) - 0.15 * scale, z];
        let turn = random(index, 6) * TAU;
        scene.add_to("world", &place(&rock, &stone, position, turn, stretch))?;
        obstacles.push([x, z, scale]);
    }

    // Grass grows in meadows on the gentle slopes between the water and the rocks,
    // thicker where two slow waves meet. Its three kinds of tuft share one material.
    let kinds = [tuft(9, 0.55, LUSH), tuft(6, 0.85, DRY), tuft(12, 0.3, SHORT)];
    let grass = Material::pbr().roughness(0.85);
    for index in 0..GRASS {
        let [x, z] = spot(index + 9000, SIZE * 0.4);
        let floor = height(x, z);
        let slope = (height(x + 0.5, z) - floor).hypot(height(x, z + 0.5) - floor);
        let waves = (x * 0.23 + 1.7).sin() * (z * 0.19 - 0.6).cos();
        let meadow = 0.55 + 0.35 * waves + 0.2 * ((x + z) * 0.51).sin();
        if floor < WATER + 0.4 || floor > 5.0 || slope > 0.25 || random(index, 10) > meadow {
            continue;
        }
        let kind = if random(index, 13) < 0.12 {
            1
        } else if meadow > 0.7 {
            0
        } else {
            2
        };
        let size = 0.7 + random(index, 11) * 0.6;
        let stretch = [size, size * (0.8 + random(index, 14) * 0.5), size];
        let turn = random(index, 12) * TAU;
        scene.add_to("world", &place(&kinds[kind], &grass, [x, floor - 0.02, z], turn, stretch))?;
    }

    // Lamps ring the clearing where the walk starts. Each one carries a point light.
    let post = column([0.02, 0.018, 0.016, 1.0]);
    let metal = Material::pbr().metallic(0.6).roughness(0.4);
    let bulb = Material::pbr().base_color([1.0, 0.85, 0.6, 1.0]);
    let mut lamps = Vec::new();
    for index in 0..LAMPS {
        let angle = index as f32 / LAMPS as f32 * TAU;
        let (x, z) = (angle.cos() * 5.0, angle.sin() * 5.0);
        let y = height(x, z);
        scene.add_to("world", &place(&post, &metal, [x, y, z], 0.0, [0.12, 2.6, 0.12]))?;
        scene.add_to("world", &place(&post, &bulb, [x, y + 2.6, z], 0.785, [0.32; 3]))?;
        let lamp = Light::point([x, y + 2.75, z], [1.0, 0.72, 0.42]).set_range(14.0)?;
        scene.add_to("world", &lamp)?;
        lamps.push(lamp);
        obstacles.push([x, z, 0.3]);
    }

    let sun = Light::directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0]);
    let moon = Light::directional([0.3, -1.0, 0.5], [0.45, 0.55, 0.85]);
    let camera = Camera::perspective(FOV, 1.0, 0.05, 200.0)?;
    scene
        .add_to("world", &sun)?
        .add_to("world", &moon)?
        .add_to("world", &camera)?;

    Ok(State {
        scene,
        sky,
        sun,
        moon,
        lamps,
        bulb,
        camera,
        obstacles,
        size: [0, 0],
        walker: Walker {
            x: 0.0,
            y: height(0.0, 3.0) + EYE,
            z: 3.0,
            rise: 0.0,
            yaw: -0.7,
            pitch: -0.05,
            walked: 0.0,
        },
        clock: 0.28 * DAY,
        last: None,
    })
}

#[allow(clippy::too_many_arguments)]
fn frame(
    state: &mut State,
    renderer: &Renderer,
    target: &impl Target,
    time: f32,
    size: [f32; 2],
    _pointer: [f32; 2],
    gesture: Gesture,
    keys: &Keys,
) -> Result<(), Box<dyn Error>> {
    let delta = state.last.map_or(0.0, |last| (time - last).clamp(0.0, 0.1));
    state.last = Some(time);
    state.clock += delta * if keys.down.contains("KeyT") { 20.0 } else { 1.0 };

    // A new window size changes the camera's aspect.
    let size = [size[0].max(1.0) as u32, size[1].max(1.0) as u32];
    if size != state.size {
        state.camera.set_aspect(size[0] as f32 / size[1] as f32)?;
        state.size = size;
    }
    walk(state, delta, gesture, keys)?;
    light(state)?;
    let resolution = [state.size[0] as f32, state.size[1] as f32];
    state.sky.set("sky.resolution", resolution)?;
    renderer.render(&state.scene, target)?;
    Ok(())
}

fn walk(state: &mut State, delta: f32, gesture: Gesture, keys: &Keys) -> Result<(), Box<dyn Error>> {
    let walker = &mut state.walker;
    let turn = keys.held(&["ArrowRight"]) - keys.held(&["ArrowLeft"]);
    walker.yaw -= gesture.pan[0] * TURN + turn * 2.0 * delta;
    walker.pitch = (walker.pitch - gesture.pan[1] * TURN).clamp(-1.4, 1.4);

    // Forward and right on the ground, from the keys, the stick and the wheel.
    let mut ahead = keys.held(&["KeyW", "ArrowUp"]) - keys.held(&["KeyS", "ArrowDown"]) + keys.stick[1];
    let mut side = keys.held(&["KeyD"]) - keys.held(&["KeyA"]) + keys.stick[0];
    let length = ahead.hypot(side);
    if length > 1.0 {
        ahead /= length;
        side /= length;
    }
    let speed = SPEED * (1.0 + keys.held(&["ShiftLeft", "ShiftRight"]));
    let (sin, cos) = walker.yaw.sin_cos();
    let step = ahead * speed * delta + gesture.zoom * 2.0;
    let mut x = walker.x - sin * step + cos * side * speed * delta;
    let mut z = walker.z - cos * step - sin * side * speed * delta;

    // Trees, rocks and lamps push back, the water stops you, and the hills keep you in.
    for &[stop_x, stop_z, radius] in &state.obstacles {
        let (away_x, away_z) = (x - stop_x, z - stop_z);
        let gap = away_x.hypot(away_z);
        let reach = radius + 0.3;
        if gap > 0.0 && gap < reach {
            x = stop_x + away_x / gap * reach;
            z = stop_z + away_z / gap * reach;
        }
    }
    let far = x.hypot(z) / (SIZE * 0.42);
    if far > 1.0 {
        x /= far;
        z /= far;
    }
    if height(x, z) > WATER + 0.1 {
        walker.walked += (x - walker.x).hypot(z - walker.z);
        walker.x = x;
        walker.z = z;
    }

    // Space jumps. Feet land on the ground, and the head bobs while walking.
    let floor = height(walker.x, walker.z) + EYE;
    if walker.y <= floor + 0.01 && keys.down.contains("Space") {
        walker.rise = 4.5;
    }
    walker.rise -= 12.0 * delta;
    walker.y = floor.max(walker.y + walker.rise * delta);
    if walker.y <= floor {
        walker.rise = 0.0;
    }
    let bob = (walker.walked * 2.4).sin() * 0.05;

    let look = [-sin * walker.pitch.cos(), walker.pitch.sin(), -cos * walker.pitch.cos()];
    let eye = [walker.x, walker.y + bob, walker.z];
    let toward = [eye[0] + look[0], eye[1] + look[1], eye[2] + look[2]];
    state.camera.look_at(eye, toward, [0.0, 1.0, 0.0])?;
    let right = normalize(cross(look, [0.0, 1.0, 0.0]));
    state.sky.set("sky.forward", look)?;
    state.sky.set("sky.right", right)?;
    state.sky.set("sky.up", cross(right, look))?;
    state.sky.set("sky.lens", (FOV / 2.0).tan())?;
    Ok(())
}

/// The sun turns once a day, above the horizon for two thirds of it. Lamps glow from dusk to dawn.
fn light(state: &State) -> Result<(), Box<dyn Error>> {
    let angle = state.clock / DAY * TAU;
    let sun = normalize([angle.cos(), angle.sin() + 0.45, 0.35]);
    let day = smoothstep(-0.08, 0.12, sun[1]);
    let low = 1.0 - smoothstep(0.0, 0.35, sun[1].abs());
    let warm = [1.0, 0.95 - low * 0.4, 0.85 - low * 0.6];
    state
        .sun
        .set_direction(sun.map(|value| -value))?
        .set_color(warm)
        .set_intensity(3.2 * day);
    state.moon.set_intensity(0.35 * (1.0 - day));
    state.scene.ambient(mix([0.025, 0.03, 0.06], [0.28, 0.32, 0.4], day));
    for lamp in &state.lamps {
        lamp.set_intensity(5.0 * (1.0 - day));
    }
    state.bulb.emissive(mix([0.3, 0.24, 0.14], [1.2, 0.75, 0.3], 1.0 - day));
    state.sky.set("sky.sun", sun)?;
    state.sky.set("sky.day", day)?;
    Ok(())
}

/// The ground: a flat clearing in rolling hills, a pond, and a rim that rises toward the edge.
fn height(x: f32, z: f32) -> f32 {
    let middle = x.hypot(z);
    let angle = z.atan2(x);
    let hills = (x * 0.15).sin() * (z * 0.12).cos() * 1.2 + (x * 0.31 + z * 0.23).sin() * 0.45;
    let edge = 0.6 + (angle * 3.0).sin() * 0.06 + (angle * 7.0 + 1.0).sin() * 0.04;
    let rim = (middle / (SIZE / 2.0) - edge).max(0.0);
    let peaks = 1.0 + (angle * 5.0 + 2.0).sin() * 0.35;
    let pond = (-((x - 11.0).powi(2) + (z + 9.0).powi(2)) / 45.0).exp();
    hills * smoothstep(3.0, 12.0, middle) + rim * rim * 60.0 * peaks - pond * 3.0
}

fn ground() -> Mesh {
    const CELLS: u32 = 72;
    let cell = SIZE / CELLS as f32;
    let mut triangles = Vec::new();
    for row in 0..CELLS {
        for slot in 0..CELLS {
            let (x, z) = (-SIZE / 2.0 + row as f32 * cell, -SIZE / 2.0 + slot as f32 * cell);
            let corner = |along_x: f32, along_z: f32| {
                [x + along_x, height(x + along_x, z + along_z), z + along_z]
            };
            let (here, ahead) = (corner(0.0, 0.0), corner(0.0, cell));
            let (beside, across) = (corner(cell, 0.0), corner(cell, cell));
            triangles.push([here, ahead, beside]);
            triangles.push([beside, ahead, across]);
        }
    }
    let painted: Vec<([Point; 3], Color)> = triangles
        .into_iter()
        .enumerate()
        .map(|(index, corners)| {
            let y = (corners[0][1] + corners[1][1] + corners[2][1]) / 3.0;
            let up = face_normal(&corners)[1];
            let color = if y < WATER + 0.3 {
                [0.42, 0.33, 0.18, 1.0]
            } else if y > 10.0 {
                [0.7, 0.72, 0.76, 1.0]
            } else if up < 0.6 || y > 6.0 {
                [0.13, 0.12, 0.11, 1.0]
            } else if up < 0.85 {
                [0.035, 0.1, 0.02, 1.0]
            } else if random(index as u32, 1) < 0.5 {
                [0.05, 0.16, 0.025, 1.0]
            } else {
                [0.07, 0.19, 0.03, 1.0]
            };
            (corners, color)
        })
        .collect();
    facets(&painted)
}

fn square(size: f32, y: f32) -> Mesh {
    let (low, high) = (-size / 2.0, size / 2.0);
    let white = [1.0; 4];
    facets(&[
        ([[low, y, low], [low, y, high], [high, y, low]], white),
        ([[high, y, low], [low, y, high], [high, y, high]], white),
    ])
}

/// A pine: a five-sided trunk under two stacked cones.
fn pine() -> Mesh {
    let bark = [0.12, 0.06, 0.03, 1.0];
    let needles = [0.02, 0.09, 0.035, 1.0];
    let trunk = cone(0.18, 0.0, 1.3, 5, true).into_iter().map(|corners| (corners, bark));
    let crown = cone(1.2, 1.0, 3.2, 7, false)
        .into_iter()
        .chain(cone(0.85, 2.2, 4.3, 7, false))
        .map(|corners| (corners, needles));
    facets(&trunk.chain(crown).collect::<Vec<_>>())
}

/// A tuft: blades fanned around one root, each bending outward in three steps that
/// taper to a tip and lighten from the soil up. A blade has a face on each side,
/// since the material draws front faces only, and its normals lean toward the sky,
/// so it takes the light the way the ground does.
fn tuft(blades: u32, tall: f32, colors: [Color; 3]) -> Mesh {
    let mesh = Mesh::new();
    for blade in 0..blades {
        let (sin, cos) = (blade as f32 * 2.4).sin_cos();
        let top = tall * (0.7 + random(blade, 20) * 0.6);
        let lean = 0.12 + random(blade, 21) * 0.25;
        // From the root up: how far out, how high, and half the width.
        let rings = [
            [0.0, 0.0, 0.035],
            [lean * 0.25, top * 0.45, 0.028],
            [lean * 0.65, top * 0.8, 0.016],
            [lean, top, 0.0],
        ];
        let corner = |[out, y, half]: Point, side: f32| {
            [cos * out + sin * half * side, y, -sin * out + cos * half * side]
        };
        for step in 0..3 {
            let (low, high) = (rings[step], rings[step + 1]);
            let mut faces = vec![[corner(low, -1.0), corner(low, 1.0), corner(high, 1.0)]];
            if step < 2 {
                faces.push([corner(low, -1.0), corner(high, 1.0), corner(high, -1.0)]);
            }
            for [first, second, third] in faces {
                for side in [[first, second, third], [third, second, first]] {
                    let [x, y, z] = face_normal(&side);
                    let up = normalize([x * 0.4, y * 0.4 + 1.0, z * 0.4]);
                    mesh.add_vertices(side.map(|point| {
                        Vertex::pbr(point)
                            .set(Vertex::NORMAL, up)
                            .set(Vertex::COLOR0, colors[step])
                    }));
                }
            }
        }
    }
    mesh
}

/// A cone, or with \`prism\` set, a prism with the same sides.
fn cone(radius: f32, bottom: f32, top: f32, sides: u32, prism: bool) -> Vec<[Point; 3]> {
    let rim = |angle: f32, y: f32| [angle.cos() * radius, y, -angle.sin() * radius];
    let mut triangles = Vec::new();
    for face in 0..sides {
        let start = face as f32 / sides as f32 * TAU;
        let end = (face + 1) as f32 / sides as f32 * TAU;
        if prism {
            triangles.push([rim(start, bottom), rim(end, bottom), rim(end, top)]);
            triangles.push([rim(start, bottom), rim(end, top), rim(start, top)]);
        } else {
            triangles.push([rim(start, bottom), rim(end, bottom), [0.0, top, 0.0]]);
        }
        triangles.push([rim(end, bottom), rim(start, bottom), [0.0, bottom, 0.0]]);
    }
    triangles
}

fn column(color: Color) -> Mesh {
    let sides = cone(0.7, 0.0, 1.0, 4, true);
    facets(&sides.into_iter().map(|corners| (corners, color)).collect::<Vec<_>>())
}

/// An icosahedron: twenty faces, shaded flat.
fn boulder() -> Mesh {
    const GOLDEN: f32 = 1.618;
    let corners = [
        [-1.0, GOLDEN, 0.0],
        [1.0, GOLDEN, 0.0],
        [-1.0, -GOLDEN, 0.0],
        [1.0, -GOLDEN, 0.0],
        [0.0, -1.0, GOLDEN],
        [0.0, 1.0, GOLDEN],
        [0.0, -1.0, -GOLDEN],
        [0.0, 1.0, -GOLDEN],
        [GOLDEN, 0.0, -1.0],
        [GOLDEN, 0.0, 1.0],
        [-GOLDEN, 0.0, -1.0],
        [-GOLDEN, 0.0, 1.0],
    ]
    .map(normalize);
    let faces: [[usize; 3]; 20] = [
        [0, 11, 5], [0, 5, 1], [0, 1, 7], [0, 7, 10], [0, 10, 11],
        [1, 5, 9], [5, 11, 4], [11, 10, 2], [10, 7, 6], [7, 1, 8],
        [3, 9, 4], [3, 4, 2], [3, 2, 6], [3, 6, 8], [3, 8, 9],
        [4, 9, 5], [2, 4, 11], [6, 2, 10], [8, 6, 7], [9, 8, 1],
    ];
    let triangles: Vec<([Point; 3], Color)> = faces
        .iter()
        .enumerate()
        .map(|(index, face)| {
            let shade = 0.16 + random(index as u32, 7) * 0.08;
            (face.map(|corner| corners[corner]), [shade, shade * 0.97, shade * 0.92, 1.0])
        })
        .collect();
    facets(&triangles)
}

/// A flat-shaded mesh: every triangle gets three vertices of its own, its face normal and its color.
fn facets(triangles: &[([Point; 3], Color)]) -> Mesh {
    let mesh = Mesh::new();
    for (corners, color) in triangles {
        let normal = face_normal(corners);
        mesh.add_vertices(corners.map(|corner| {
            Vertex::pbr(corner)
                .set(Vertex::NORMAL, normal)
                .set(Vertex::COLOR0, *color)
        }));
    }
    mesh
}

fn face_normal([first, second, third]: &[Point; 3]) -> Point {
    normalize(cross(minus(*second, *first), minus(*third, *first)))
}

fn place(mesh: &Mesh, material: &Material, position: Point, turn: f32, scale: Point) -> Model {
    let model = Model::new(mesh.clone(), material.clone());
    model.translate(position);
    model.rotate([0.0, 1.0, 0.0], turn);
    model.scale(scale);
    model
}

/// The same pseudo-random numbers in every language: a 32-bit integer hash.
fn random(index: u32, salt: u32) -> f32 {
    let mut hash = (index ^ salt.wrapping_mul(0x9e37_79b9)).wrapping_mul(0x85eb_ca6b);
    hash = (hash ^ (hash >> 13)).wrapping_mul(0xc2b2_ae35);
    (f64::from(hash ^ (hash >> 16)) / 4_294_967_296.0) as f32
}

fn spot(index: u32, radius: f32) -> [f32; 2] {
    let distance = random(index, 8).sqrt() * radius;
    let angle = random(index, 9) * TAU;
    [angle.cos() * distance, angle.sin() * distance]
}

fn minus(first: Point, second: Point) -> Point {
    [first[0] - second[0], first[1] - second[1], first[2] - second[2]]
}

fn cross(first: Point, second: Point) -> Point {
    [
        first[1] * second[2] - first[2] * second[1],
        first[2] * second[0] - first[0] * second[2],
        first[0] * second[1] - first[1] * second[0],
    ]
}

fn normalize(vector: Point) -> Point {
    let length = (vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2]).sqrt();
    vector.map(|value| value / length)
}

fn mix(from: Point, to: Point, amount: f32) -> Point {
    [0, 1, 2].map(|axis| from[axis] + (to[axis] - from[axis]) * amount)
}

fn smoothstep(low: f32, high: f32, value: f32) -> f32 {
    let amount = ((value - low) / (high - low)).clamp(0.0, 1.0);
    amount * amount * (3.0 - 2.0 * amount)
}
`;
}

function writePython({ library, sky }) {
  return `# ${NEEDS}

import math
from pathlib import Path
from types import SimpleNamespace

from fragmentcolor import AlphaMode, Camera, Light, Material, Mesh, Model, Pass, Scene, Shader, Vertex

${wrap("", "LIBRARY = [", library.map(quote), "]", 88)}
SKY_SOURCE = Path(__file__).with_name("sky.wgsl").read_text()

# The valley is SIZE meters across, and a day lasts DAY seconds. Hold T to hurry it.
SIZE = 72.0
DAY = 180.0
TREES = 140
ROCKS = 36
LAMPS = 8
GRASS = 8000
WATER = -1.2

# Blade colors from the soil to the tip, in three steps, for three kinds of tuft.
LUSH = [[0.04, 0.12, 0.02, 1.0], [0.07, 0.2, 0.03, 1.0], [0.13, 0.29, 0.05, 1.0]]
DRY = [[0.05, 0.12, 0.02, 1.0], [0.13, 0.21, 0.04, 1.0], [0.26, 0.3, 0.09, 1.0]]
SHORT = [[0.035, 0.1, 0.02, 1.0], [0.055, 0.16, 0.025, 1.0], [0.09, 0.23, 0.04, 1.0]]

# Meters per second (Shift runs), eye height in meters, and radians of turn per pixel dragged.
SPEED = 3.5
EYE = 1.6
TURN = 0.005
FOV = 1.1


# Call setup() once, then frame() on every redraw of your canvas. frame's gesture is
# what the wheel, a drag and a pinch did since the last frame: zoom in doublings
# (positive zooms in) and pan in canvas pixels. Its keys hold the keys held down,
# named the way KeyboardEvent.code names them ("KeyW", "ArrowUp", "Space"), and a
# touch stick as [x, y] from -1 to 1 with y up.
def setup():
    sky = Shader([*LIBRARY, SKY_SOURCE])
${setCalls("python", "sky", sky)}

    # The valley draws over the sky. A pass that draws models gets a depth buffer,
    # so the nearest surface wins at each pixel.
    world = Pass("world")
    world.load_previous()

    scene = Scene()
    scene.add_pass(Pass.from_shader("sky", sky)).add_pass(world)

    scene.add_to("world", Model(ground(), Material.pbr().roughness(0.95)))
    water = Material.pbr().base_color([0.02, 0.08, 0.1, 0.8]).roughness(0.08).alpha_mode(AlphaMode.Blend)
    scene.add_to("world", Model(square(SIZE, WATER), water))

    # Trees and rocks share one mesh each, so each kind draws in one call.
    obstacles = []
    tree = pine()
    bark = Material.pbr().roughness(0.8)
    for index in range(TREES * 8):
        if len(obstacles) == TREES:
            break
        x, z = spot(index, SIZE * 0.42)
        floor = height(x, z)
        if floor < WATER + 0.5 or floor > 5 or math.hypot(x, z) < 6:
            continue
        scale = 0.7 + random(index, 2) * 0.7
        turn = random(index, 3) * math.tau
        scene.add_to("world", place(tree, bark, [x, floor - 0.1, z], turn, [scale, scale, scale]))
        obstacles.append([x, z, 0.35 * scale])
    rock = boulder()
    stone = Material.pbr().roughness(0.7)
    for index in range(ROCKS):
        x, z = spot(index + 5000, SIZE * 0.45)
        scale = 0.4 + random(index, 4) * 0.9
        stretch = [scale * (1 + random(index, 5)), scale * 0.7, scale]
        position = [x, height(x, z) - 0.15 * scale, z]
        scene.add_to("world", place(rock, stone, position, random(index, 6) * math.tau, stretch))
        obstacles.append([x, z, scale])

    # Grass grows in meadows on the gentle slopes between the water and the rocks,
    # thicker where two slow waves meet. Its three kinds of tuft share one material.
    kinds = [tuft(9, 0.55, LUSH), tuft(6, 0.85, DRY), tuft(12, 0.3, SHORT)]
    grass = Material.pbr().roughness(0.85)
    for index in range(GRASS):
        x, z = spot(index + 9000, SIZE * 0.4)
        floor = height(x, z)
        slope = math.hypot(height(x + 0.5, z) - floor, height(x, z + 0.5) - floor)
        waves = math.sin(x * 0.23 + 1.7) * math.cos(z * 0.19 - 0.6)
        meadow = 0.55 + 0.35 * waves + 0.2 * math.sin((x + z) * 0.51)
        if floor < WATER + 0.4 or floor > 5 or slope > 0.25 or random(index, 10) > meadow:
            continue
        kind = 1 if random(index, 13) < 0.12 else 0 if meadow > 0.7 else 2
        size = 0.7 + random(index, 11) * 0.6
        stretch = [size, size * (0.8 + random(index, 14) * 0.5), size]
        turn = random(index, 12) * math.tau
        scene.add_to("world", place(kinds[kind], grass, [x, floor - 0.02, z], turn, stretch))

    # Lamps ring the clearing where the walk starts. Each one carries a point light.
    post = column([0.02, 0.018, 0.016, 1.0])
    metal = Material.pbr().metallic(0.6).roughness(0.4)
    bulb = Material.pbr().base_color([1.0, 0.85, 0.6, 1.0])
    lamps = []
    for index in range(LAMPS):
        angle = index / LAMPS * math.tau
        x, z = math.cos(angle) * 5, math.sin(angle) * 5
        y = height(x, z)
        scene.add_to("world", place(post, metal, [x, y, z], 0.0, [0.12, 2.6, 0.12]))
        scene.add_to("world", place(post, bulb, [x, y + 2.6, z], 0.785, [0.32, 0.32, 0.32]))
        lamp = Light.point([x, y + 2.75, z], [1.0, 0.72, 0.42]).set_range(14.0)
        scene.add_to("world", lamp)
        lamps.append(lamp)
        obstacles.append([x, z, 0.3])

    sun = Light.directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0])
    moon = Light.directional([0.3, -1.0, 0.5], [0.45, 0.55, 0.85])
    camera = Camera.perspective(FOV, 1.0, 0.05, 200.0)
    scene.add_to("world", sun).add_to("world", moon).add_to("world", camera)

    walker = SimpleNamespace(x=0.0, y=height(0.0, 3.0) + EYE, z=3.0, rise=0.0, yaw=-0.7, pitch=-0.05, walked=0.0)
    return SimpleNamespace(
        scene=scene,
        sky=sky,
        sun=sun,
        moon=moon,
        lamps=lamps,
        bulb=bulb,
        camera=camera,
        obstacles=obstacles,
        size=[0, 0],
        walker=walker,
        clock=0.28 * DAY,
        last=None,
    )


def frame(state, renderer, target, time, size, pointer, gesture=None, keys=None):
    gesture = gesture or SimpleNamespace(zoom=0.0, pan=[0.0, 0.0])
    keys = keys or SimpleNamespace(down=set(), stick=[0.0, 0.0])
    delta = 0.0 if state.last is None else min(max(time - state.last, 0.0), 0.1)
    state.last = time
    state.clock += delta * (20 if "KeyT" in keys.down else 1)

    # A new canvas size changes the camera's aspect.
    size = [max(int(size[0]), 1), max(int(size[1]), 1)]
    if size != state.size:
        state.camera.set_aspect(size[0] / size[1])
        state.size = size
    walk(state, delta, gesture, keys)
    light(state)
    state.sky.set("sky.resolution", [float(state.size[0]), float(state.size[1])])
    renderer.render(state.scene, target)


def walk(state, delta, gesture, keys):
    walker = state.walker

    def held(*codes):
        return 1.0 if any(code in keys.down for code in codes) else 0.0

    walker.yaw -= gesture.pan[0] * TURN + (held("ArrowRight") - held("ArrowLeft")) * 2 * delta
    walker.pitch = min(max(walker.pitch - gesture.pan[1] * TURN, -1.4), 1.4)

    # Forward and right on the ground, from the keys, the stick and the wheel.
    ahead = held("KeyW", "ArrowUp") - held("KeyS", "ArrowDown") + keys.stick[1]
    side = held("KeyD") - held("KeyA") + keys.stick[0]
    length = math.hypot(ahead, side)
    if length > 1:
        ahead, side = ahead / length, side / length
    speed = SPEED * (1 + held("ShiftLeft", "ShiftRight"))
    sin, cos = math.sin(walker.yaw), math.cos(walker.yaw)
    step = ahead * speed * delta + gesture.zoom * 2
    x = walker.x - sin * step + cos * side * speed * delta
    z = walker.z - cos * step - sin * side * speed * delta

    # Trees, rocks and lamps push back, the water stops you, and the hills keep you in.
    for stop_x, stop_z, radius in state.obstacles:
        away_x, away_z = x - stop_x, z - stop_z
        gap = math.hypot(away_x, away_z)
        reach = radius + 0.3
        if 0 < gap < reach:
            x, z = stop_x + away_x / gap * reach, stop_z + away_z / gap * reach
    far = math.hypot(x, z) / (SIZE * 0.42)
    if far > 1:
        x, z = x / far, z / far
    if height(x, z) > WATER + 0.1:
        walker.walked += math.hypot(x - walker.x, z - walker.z)
        walker.x, walker.z = x, z

    # Space jumps. Feet land on the ground, and the head bobs while walking.
    floor = height(walker.x, walker.z) + EYE
    if walker.y <= floor + 0.01 and "Space" in keys.down:
        walker.rise = 4.5
    walker.rise -= 12 * delta
    walker.y = max(floor, walker.y + walker.rise * delta)
    if walker.y <= floor:
        walker.rise = 0.0
    bob = math.sin(walker.walked * 2.4) * 0.05

    look = [-sin * math.cos(walker.pitch), math.sin(walker.pitch), -cos * math.cos(walker.pitch)]
    eye = [walker.x, walker.y + bob, walker.z]
    toward = [eye[0] + look[0], eye[1] + look[1], eye[2] + look[2]]
    state.camera.look_at(eye, toward, [0.0, 1.0, 0.0])
    right = normalize(cross(look, [0.0, 1.0, 0.0]))
    state.sky.set("sky.forward", look)
    state.sky.set("sky.right", right)
    state.sky.set("sky.up", cross(right, look))
    state.sky.set("sky.lens", math.tan(FOV / 2))


# The sun turns once a day, above the horizon for two thirds of it. Lamps glow from dusk to dawn.
def light(state):
    angle = state.clock / DAY * math.tau
    sun = normalize([math.cos(angle), math.sin(angle) + 0.45, 0.35])
    day = smoothstep(-0.08, 0.12, sun[1])
    low = 1 - smoothstep(0.0, 0.35, abs(sun[1]))
    warm = [1.0, 0.95 - low * 0.4, 0.85 - low * 0.6]
    state.sun.set_direction([-value for value in sun]).set_color(warm).set_intensity(3.2 * day)
    state.moon.set_intensity(0.35 * (1 - day))
    state.scene.ambient(mix([0.025, 0.03, 0.06], [0.28, 0.32, 0.4], day))
    for lamp in state.lamps:
        lamp.set_intensity(5 * (1 - day))
    state.bulb.emissive(mix([0.3, 0.24, 0.14], [1.2, 0.75, 0.3], 1 - day))
    state.sky.set("sky.sun", sun)
    state.sky.set("sky.day", day)


# The ground: a flat clearing in rolling hills, a pond, and a rim that rises toward the edge.
def height(x, z):
    middle = math.hypot(x, z)
    angle = math.atan2(z, x)
    hills = math.sin(x * 0.15) * math.cos(z * 0.12) * 1.2 + math.sin(x * 0.31 + z * 0.23) * 0.45
    edge = 0.6 + math.sin(angle * 3) * 0.06 + math.sin(angle * 7 + 1) * 0.04
    rim = max(0.0, middle / (SIZE / 2) - edge)
    peaks = 1 + math.sin(angle * 5 + 2) * 0.35
    pond = math.exp(-((x - 11) ** 2 + (z + 9) ** 2) / 45)
    return hills * smoothstep(3.0, 12.0, middle) + rim * rim * 60 * peaks - pond * 3


def ground():
    cells = 72
    cell = SIZE / cells
    triangles = []
    for row in range(cells):
        for slot in range(cells):
            x, z = -SIZE / 2 + row * cell, -SIZE / 2 + slot * cell
            here, ahead = corner(x, z, 0.0, 0.0), corner(x, z, 0.0, cell)
            beside, across = corner(x, z, cell, 0.0), corner(x, z, cell, cell)
            triangles += [[here, ahead, beside], [beside, ahead, across]]

    def paint(triangle, normal, index):
        y = (triangle[0][1] + triangle[1][1] + triangle[2][1]) / 3
        if y < WATER + 0.3:
            return [0.42, 0.33, 0.18, 1.0]
        if y > 10:
            return [0.7, 0.72, 0.76, 1.0]
        if normal[1] < 0.6 or y > 6:
            return [0.13, 0.12, 0.11, 1.0]
        if normal[1] < 0.85:
            return [0.035, 0.1, 0.02, 1.0]
        return [0.05, 0.16, 0.025, 1.0] if random(index, 1) < 0.5 else [0.07, 0.19, 0.03, 1.0]

    return facets(triangles, paint)


def corner(x, z, along_x, along_z):
    return [x + along_x, height(x + along_x, z + along_z), z + along_z]


def square(size, y):
    low, high = -size / 2, size / 2
    white = [1.0, 1.0, 1.0, 1.0]
    return facets([[[low, y, low], [low, y, high], [high, y, low], white], [[high, y, low], [low, y, high], [high, y, high], white]])


# A pine: a five-sided trunk under two stacked cones.
def pine():
    trunk = [[*triangle, [0.12, 0.06, 0.03, 1.0]] for triangle in cone(0.18, 0.0, 1.3, 5, True)]
    crown = [[*triangle, [0.02, 0.09, 0.035, 1.0]] for triangle in cone(1.2, 1.0, 3.2, 7) + cone(0.85, 2.2, 4.3, 7)]
    return facets(trunk + crown)


# A tuft: blades fanned around one root, each bending outward in three steps that
# taper to a tip and lighten from the soil up. A blade has a face on each side,
# since the material draws front faces only, and its normals lean toward the sky,
# so it takes the light the way the ground does.
def tuft(blades, tall, colors):
    mesh = Mesh()
    for blade in range(blades):
        cos, sin = math.cos(blade * 2.4), math.sin(blade * 2.4)
        top = tall * (0.7 + random(blade, 20) * 0.6)
        lean = 0.12 + random(blade, 21) * 0.25
        # From the root up: how far out, how high, and half the width.
        rings = [[0.0, 0.0, 0.035], [lean * 0.25, top * 0.45, 0.028], [lean * 0.65, top * 0.8, 0.016], [lean, top, 0.0]]

        def corner(ring, side, cos=cos, sin=sin):
            out, y, half = ring
            return [cos * out + sin * half * side, y, -sin * out + cos * half * side]

        for step in range(3):
            low, high = rings[step], rings[step + 1]
            faces = [[corner(low, -1), corner(low, 1), corner(high, 1)]]
            if step < 2:
                faces.append([corner(low, -1), corner(high, 1), corner(high, -1)])
            for face in faces:
                for side in (face, face[::-1]):
                    x, y, z = normalize(cross(minus(side[1], side[0]), minus(side[2], side[0])))
                    up = normalize([x * 0.4, y * 0.4 + 1, z * 0.4])
                    mesh.add_vertices([Vertex.pbr(point).set(Vertex.NORMAL, up).set(Vertex.COLOR0, colors[step]) for point in side])
    return mesh


# A cone, or with prism set, a prism with the same sides.
def cone(radius, bottom, top, sides, prism=False):
    def rim(angle, y):
        return [math.cos(angle) * radius, y, -math.sin(angle) * radius]

    triangles = []
    for face in range(sides):
        start, end = face / sides * math.tau, (face + 1) / sides * math.tau
        if prism:
            triangles += [[rim(start, bottom), rim(end, bottom), rim(end, top)], [rim(start, bottom), rim(end, top), rim(start, top)]]
        else:
            triangles.append([rim(start, bottom), rim(end, bottom), [0.0, top, 0.0]])
        triangles.append([rim(end, bottom), rim(start, bottom), [0.0, bottom, 0.0]])
    return triangles


def column(color):
    return facets([[*triangle, color] for triangle in cone(0.7, 0.0, 1.0, 4, True)])


# An icosahedron: twenty faces, shaded flat.
def boulder():
    golden = 1.618
    corners = [
        normalize(point)
        for point in [
            [-1, golden, 0], [1, golden, 0], [-1, -golden, 0], [1, -golden, 0],
            [0, -1, golden], [0, 1, golden], [0, -1, -golden], [0, 1, -golden],
            [golden, 0, -1], [golden, 0, 1], [-golden, 0, -1], [-golden, 0, 1],
        ]
    ]
    faces = [
        [0, 11, 5], [0, 5, 1], [0, 1, 7], [0, 7, 10], [0, 10, 11],
        [1, 5, 9], [5, 11, 4], [11, 10, 2], [10, 7, 6], [7, 1, 8],
        [3, 9, 4], [3, 4, 2], [3, 2, 6], [3, 6, 8], [3, 8, 9],
        [4, 9, 5], [2, 4, 11], [6, 2, 10], [8, 6, 7], [9, 8, 1],
    ]

    def paint(triangle, normal, index):
        shade = 0.16 + random(index, 7) * 0.08
        return [shade, shade * 0.97, shade * 0.92, 1.0]

    return facets([[corners[point] for point in face] for face in faces], paint)


# A flat-shaded mesh: every triangle gets three vertices of its own, its face normal
# and a color, which paint picks or the triangle carries fourth.
def facets(triangles, paint=lambda triangle, normal, index: triangle[3]):
    mesh = Mesh()
    for index, triangle in enumerate(triangles):
        first, second, third = triangle[:3]
        normal = normalize(cross(minus(second, first), minus(third, first)))
        color = paint(triangle, normal, index)
        mesh.add_vertices([Vertex.pbr(point).set(Vertex.NORMAL, normal).set(Vertex.COLOR0, color) for point in (first, second, third)])
    return mesh


def place(mesh, material, position, turn, scale):
    model = Model(mesh, material)
    model.translate(position)
    model.rotate([0.0, 1.0, 0.0], turn)
    model.scale(scale)
    return model


# The same pseudo-random numbers in every language: a 32-bit integer hash.
def random(index, salt):
    value = ((index ^ ((salt * 0x9E3779B9) & 0xFFFFFFFF)) * 0x85EBCA6B) & 0xFFFFFFFF
    value = ((value ^ (value >> 13)) * 0xC2B2AE35) & 0xFFFFFFFF
    return (value ^ (value >> 16)) / 4294967296


def spot(index, radius):
    distance = math.sqrt(random(index, 8)) * radius
    angle = random(index, 9) * math.tau
    return math.cos(angle) * distance, math.sin(angle) * distance


def minus(first, second):
    return [first[0] - second[0], first[1] - second[1], first[2] - second[2]]


def cross(first, second):
    return [
        first[1] * second[2] - first[2] * second[1],
        first[2] * second[0] - first[0] * second[2],
        first[0] * second[1] - first[1] * second[0],
    ]


def normalize(vector):
    length = math.sqrt(sum(value * value for value in vector))
    return [value / length for value in vector]


def mix(start, end, amount):
    return [low + (high - low) * amount for low, high in zip(start, end)]


def smoothstep(low, high, value):
    amount = min(max((value - low) / (high - low), 0.0), 1.0)
    return amount * amount * (3 - 2 * amount)
`;
}

function writeSwift({ library, sky }) {
  const libraryLine = library.length ? wrap("", "let library = [", library.map(quote), "]", 100) : "let library: [String] = []";
  return `// ${NEEDS}

import Foundation
import FragmentColor

${libraryLine}

/// The valley is \`valleySize\` meters across, and a day lasts \`dayLength\` seconds. Hold T to hurry it.
let valleySize: Float = 72
let dayLength: Float = 180
let treeCount = 140
let rockCount: UInt32 = 36
let lampCount: UInt32 = 8
let grassSpots: UInt32 = 8000
let water: Float = -1.2

/// Meters per second (Shift runs), eye height in meters, and radians of turn per point dragged.
let speed: Float = 3.5
let eyeHeight: Float = 1.6
let turnRate: Float = 0.005
let fieldOfView: Float = 1.1
let tau = Float.pi * 2

/// Three corners and the color the triangle carries.
typealias Triangle = ([[Float]], [Float])

/// Blade colors from the soil to the tip, in three steps, for three kinds of tuft.
let lushColors: [[Float]] = [[0.04, 0.12, 0.02, 1], [0.07, 0.2, 0.03, 1], [0.13, 0.29, 0.05, 1]]
let dryColors: [[Float]] = [[0.05, 0.12, 0.02, 1], [0.13, 0.21, 0.04, 1], [0.26, 0.3, 0.09, 1]]
let shortColors: [[Float]] = [[0.035, 0.1, 0.02, 1], [0.055, 0.16, 0.025, 1], [0.09, 0.23, 0.04, 1]]

/// What the wheel, a drag and a pinch did since the last frame: zoom in doublings
/// (positive zooms in) and pan in view points.
struct Gesture {
    var zoom: Float = 0
    var pan: [Float] = [0, 0]
}

/// The keys held down, named the way \`KeyboardEvent.code\` names them
/// ("KeyW", "ArrowUp", "Space"), and a touch stick as [x, y] from -1 to 1, y up.
struct Keys {
    var down: Set<String> = []
    var stick: [Float] = [0, 0]

    /// 1 while any of \`codes\` is held down, 0 otherwise.
    func held(_ codes: String...) -> Float {
        codes.contains { down.contains($0) } ? 1 : 0
    }
}

struct Walker {
    var x: Float = 0
    var y: Float = height(0, 3) + eyeHeight
    var z: Float = 3
    var rise: Float = 0
    var yaw: Float = -0.7
    var pitch: Float = -0.05
    var walked: Float = 0
}

struct State {
    let scene: Scene
    let sky: Shader
    let sun: Light
    let moon: Light
    let lamps: [Light]
    let bulb: Material
    let camera: Camera
    let obstacles: [[Float]]
    var size: [UInt32] = [0, 0]
    var walker = Walker()
    var clock: Float = 0.28 * dayLength
    var last: Float?
}

// Call setup once, then frame from your MTKView or CADisplayLink callback.
func setup() throws -> State {
    let skyURL = Bundle.main.url(forResource: "sky", withExtension: "wgsl")!
    let sky = try Shader(library + [String(contentsOf: skyURL, encoding: .utf8)])
${setCalls("swift", "sky", sky, { before: "try " })}

    // The valley draws over the sky. A pass that draws models gets a depth buffer,
    // so the nearest surface wins at each pixel.
    let world = Pass("world")
    world.loadPrevious()

    let scene = Scene()
    scene.addPass(Pass.fromShader("sky", sky)).addPass(world)

    try scene.addTo("world", Model(ground(), Material.pbr().roughness(0.95)))
    let glass = try Material.pbr().baseColor([0.02, 0.08, 0.1, 0.8]).roughness(0.08).alphaMode(.blend)
    try scene.addTo("world", Model(square(valleySize, water), glass))

    // Trees and rocks share one mesh each, so each kind draws in one call.
    var obstacles: [[Float]] = []
    let tree = try pine()
    let bark = Material.pbr().roughness(0.8)
    for index in 0..<UInt32(treeCount * 8) where obstacles.count < treeCount {
        let (x, z) = spot(index, valleySize * 0.42)
        let floor = height(x, z)
        if floor < water + 0.5 || floor > 5 || hypot(x, z) < 6 {
            continue
        }
        let scale = 0.7 + random(index, 2) * 0.7
        let model = try place(tree, bark, [x, floor - 0.1, z], random(index, 3) * tau, [scale, scale, scale])
        try scene.addTo("world", model)
        obstacles.append([x, z, 0.35 * scale])
    }
    let rock = try boulder()
    let stone = Material.pbr().roughness(0.7)
    for index in 0..<rockCount {
        let (x, z) = spot(index + 5000, valleySize * 0.45)
        let scale = 0.4 + random(index, 4) * 0.9
        let stretch = [scale * (1 + random(index, 5)), scale * 0.7, scale]
        let position = [x, height(x, z) - 0.15 * scale, z]
        try scene.addTo("world", place(rock, stone, position, random(index, 6) * tau, stretch))
        obstacles.append([x, z, scale])
    }

    // Grass grows in meadows on the gentle slopes between the water and the rocks,
    // thicker where two slow waves meet. Its three kinds of tuft share one material.
    let kinds = try [tuft(9, 0.55, lushColors), tuft(6, 0.85, dryColors), tuft(12, 0.3, shortColors)]
    let grass = Material.pbr().roughness(0.85)
    for index in 0..<grassSpots {
        let (x, z) = spot(index + 9000, valleySize * 0.4)
        let floor = height(x, z)
        let slope = hypot(height(x + 0.5, z) - floor, height(x, z + 0.5) - floor)
        let waves = sin(x * 0.23 + 1.7) * cos(z * 0.19 - 0.6)
        let meadow = 0.55 + 0.35 * waves + 0.2 * sin((x + z) * 0.51)
        if floor < water + 0.4 || floor > 5 || slope > 0.25 || random(index, 10) > meadow {
            continue
        }
        let kind = random(index, 13) < 0.12 ? 1 : meadow > 0.7 ? 0 : 2
        let size = 0.7 + random(index, 11) * 0.6
        let stretch = [size, size * (0.8 + random(index, 14) * 0.5), size]
        let turn = random(index, 12) * tau
        try scene.addTo("world", place(kinds[kind], grass, [x, floor - 0.02, z], turn, stretch))
    }

    // Lamps ring the clearing where the walk starts. Each one carries a point light.
    let post = try column([0.02, 0.018, 0.016, 1])
    let metal = Material.pbr().metallic(0.6).roughness(0.4)
    let bulb = try Material.pbr().baseColor([1, 0.85, 0.6, 1])
    var lamps: [Light] = []
    for index in 0..<lampCount {
        let angle = Float(index) / Float(lampCount) * tau
        let (x, z) = (cos(angle) * 5, sin(angle) * 5)
        let y = height(x, z)
        try scene.addTo("world", place(post, metal, [x, y, z], 0, [0.12, 2.6, 0.12]))
        try scene.addTo("world", place(post, bulb, [x, y + 2.6, z], 0.785, [0.32, 0.32, 0.32]))
        let lamp = try Light.point([x, y + 2.75, z], [1, 0.72, 0.42]).setRange(14.0)
        try scene.addTo("world", lamp)
        lamps.append(lamp)
        obstacles.append([x, z, 0.3])
    }

    let sun = try Light.directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0])
    let moon = try Light.directional([0.3, -1, 0.5], [0.45, 0.55, 0.85])
    let camera = try Camera.perspective(fieldOfView, 1, 0.05, 200)
    try scene.addTo("world", sun).addTo("world", moon).addTo("world", camera)

    return State(
        scene: scene, sky: sky, sun: sun,
        moon: moon, lamps: lamps, bulb: bulb, camera: camera, obstacles: obstacles
    )
}

func frame(
    state: inout State,
    renderer: Renderer,
    target: WindowTarget,
    time: Float,
    size: [Float],
    pointer: [Float],
    gesture: Gesture = Gesture(),
    keys: Keys = Keys()
) throws {
    let delta = state.last.map { min(max(time - $0, 0), 0.1) } ?? 0
    state.last = time
    state.clock += delta * (keys.down.contains("KeyT") ? 20 : 1)

    // A new view size changes the camera's aspect.
    let pixels = [UInt32(max(size[0], 1)), UInt32(max(size[1], 1))]
    if pixels != state.size {
        try state.camera.setAspect(Float(pixels[0]) / Float(pixels[1]))
        state.size = pixels
    }
    try walk(&state, delta, gesture, keys)
    try light(state)
    try state.sky.set("sky.resolution", [Float(state.size[0]), Float(state.size[1])])
    try renderer.render(state.scene, target)
}

func walk(_ state: inout State, _ delta: Float, _ gesture: Gesture, _ keys: Keys) throws {
    var walker = state.walker
    let turn = keys.held("ArrowRight") - keys.held("ArrowLeft")
    walker.yaw -= gesture.pan[0] * turnRate + turn * 2 * delta
    walker.pitch = min(max(walker.pitch - gesture.pan[1] * turnRate, -1.4), 1.4)

    // Forward and right on the ground, from the keys, the stick and the wheel.
    var ahead = keys.held("KeyW", "ArrowUp") - keys.held("KeyS", "ArrowDown") + keys.stick[1]
    var side = keys.held("KeyD") - keys.held("KeyA") + keys.stick[0]
    let length = hypot(ahead, side)
    if length > 1 {
        ahead /= length
        side /= length
    }
    let pace = speed * (1 + keys.held("ShiftLeft", "ShiftRight"))
    let (sine, cosine) = (sin(walker.yaw), cos(walker.yaw))
    let step = ahead * pace * delta + gesture.zoom * 2
    var x = walker.x - sine * step + cosine * side * pace * delta
    var z = walker.z - cosine * step - sine * side * pace * delta

    // Trees, rocks and lamps push back, the water stops you, and the hills keep you in.
    for obstacle in state.obstacles {
        let (awayX, awayZ) = (x - obstacle[0], z - obstacle[1])
        let gap = hypot(awayX, awayZ)
        let reach = obstacle[2] + 0.3
        if gap > 0 && gap < reach {
            x = obstacle[0] + awayX / gap * reach
            z = obstacle[1] + awayZ / gap * reach
        }
    }
    let far = hypot(x, z) / (valleySize * 0.42)
    if far > 1 {
        x /= far
        z /= far
    }
    if height(x, z) > water + 0.1 {
        walker.walked += hypot(x - walker.x, z - walker.z)
        walker.x = x
        walker.z = z
    }

    // Space jumps. Feet land on the ground, and the head bobs while walking.
    let floor = height(walker.x, walker.z) + eyeHeight
    if walker.y <= floor + 0.01 && keys.down.contains("Space") {
        walker.rise = 4.5
    }
    walker.rise -= 12 * delta
    walker.y = max(floor, walker.y + walker.rise * delta)
    if walker.y <= floor {
        walker.rise = 0
    }
    let bob = sin(walker.walked * 2.4) * 0.05
    state.walker = walker

    let look = [-sine * cos(walker.pitch), sin(walker.pitch), -cosine * cos(walker.pitch)]
    let eye = [walker.x, walker.y + bob, walker.z]
    try state.camera.lookAt(eye, [eye[0] + look[0], eye[1] + look[1], eye[2] + look[2]], [0, 1, 0])
    let right = normalize(cross(look, [0, 1, 0]))
    try state.sky.set("sky.forward", look)
    try state.sky.set("sky.right", right)
    try state.sky.set("sky.up", cross(right, look))
    try state.sky.set("sky.lens", tan(fieldOfView / 2))
}

/// The sun turns once a day, above the horizon for two thirds of it. Lamps glow from dusk to dawn.
func light(_ state: State) throws {
    let angle = state.clock / dayLength * tau
    let sun = normalize([cos(angle), sin(angle) + 0.45, 0.35])
    let day = smoothstep(-0.08, 0.12, sun[1])
    let low = 1 - smoothstep(0, 0.35, abs(sun[1]))
    let warm: [Float] = [1, 0.95 - low * 0.4, 0.85 - low * 0.6]
    try state.sun.setDirection(sun.map { -$0 }).setColor(warm).setIntensity(3.2 * day)
    state.moon.setIntensity(0.35 * (1 - day))
    try state.scene.ambient(mix([0.025, 0.03, 0.06], [0.28, 0.32, 0.4], day))
    for lamp in state.lamps {
        lamp.setIntensity(5 * (1 - day))
    }
    try state.bulb.emissive(mix([0.3, 0.24, 0.14], [1.2, 0.75, 0.3], 1 - day))
    try state.sky.set("sky.sun", sun)
    try state.sky.set("sky.day", day)
}

/// The ground: a flat clearing in rolling hills, a pond, and a rim that rises toward the edge.
func height(_ x: Float, _ z: Float) -> Float {
    let middle = hypot(x, z)
    let angle = atan2(z, x)
    let hills = sin(x * 0.15) * cos(z * 0.12) * 1.2 + sin(x * 0.31 + z * 0.23) * 0.45
    let edge = 0.6 + sin(angle * 3) * 0.06 + sin(angle * 7 + 1) * 0.04
    let rim = max(0, middle / (valleySize / 2) - edge)
    let peaks = 1 + sin(angle * 5 + 2) * 0.35
    let pond = exp(-((x - 11) * (x - 11) + (z + 9) * (z + 9)) / 45)
    return hills * smoothstep(3, 12, middle) + rim * rim * 60 * peaks - pond * 3
}

func ground() throws -> Mesh {
    let cells = 72
    let cell = valleySize / Float(cells)
    var triangles: [Triangle] = []
    for row in 0..<cells {
        for slot in 0..<cells {
            let (x, z) = (-valleySize / 2 + Float(row) * cell, -valleySize / 2 + Float(slot) * cell)
            let corner = { (alongX: Float, alongZ: Float) in [x + alongX, height(x + alongX, z + alongZ), z + alongZ] }
            let (here, ahead) = (corner(0, 0), corner(0, cell))
            let (beside, across) = (corner(cell, 0), corner(cell, cell))
            for corners in [[here, ahead, beside], [beside, ahead, across]] {
                let y = (corners[0][1] + corners[1][1] + corners[2][1]) / 3
                let up = faceNormal(corners)[1]
                let color: [Float]
                if y < water + 0.3 {
                    color = [0.42, 0.33, 0.18, 1]
                } else if y > 10 {
                    color = [0.7, 0.72, 0.76, 1]
                } else if up < 0.6 || y > 6 {
                    color = [0.13, 0.12, 0.11, 1]
                } else if up < 0.85 {
                    color = [0.035, 0.1, 0.02, 1]
                } else {
                    color = random(UInt32(triangles.count), 1) < 0.5 ? [0.05, 0.16, 0.025, 1] : [0.07, 0.19, 0.03, 1]
                }
                triangles.append((corners, color))
            }
        }
    }
    return try facets(triangles)
}

func square(_ size: Float, _ y: Float) throws -> Mesh {
    let (low, high) = (-size / 2, size / 2)
    let white: [Float] = [1, 1, 1, 1]
    return try facets([
        ([[low, y, low], [low, y, high], [high, y, low]], white),
        ([[high, y, low], [low, y, high], [high, y, high]], white),
    ])
}

/// A pine: a five-sided trunk under two stacked cones.
func pine() throws -> Mesh {
    let trunk = cone(0.18, 0, 1.3, 5, prism: true).map { ($0, [0.12, 0.06, 0.03, 1] as [Float]) }
    let crown = (cone(1.2, 1, 3.2, 7) + cone(0.85, 2.2, 4.3, 7)).map { ($0, [0.02, 0.09, 0.035, 1] as [Float]) }
    return try facets(trunk + crown)
}

/// A tuft: blades fanned around one root, each bending outward in three steps that
/// taper to a tip and lighten from the soil up. A blade has a face on each side,
/// since the material draws front faces only, and its normals lean toward the sky,
/// so it takes the light the way the ground does.
func tuft(_ blades: UInt32, _ tall: Float, _ colors: [[Float]]) throws -> Mesh {
    let mesh = Mesh()
    for blade in 0..<blades {
        let angle = Float(blade) * 2.4
        let top = tall * (0.7 + random(blade, 20) * 0.6)
        let lean = 0.12 + random(blade, 21) * 0.25
        // From the root up: how far out, how high, and half the width.
        let rings: [[Float]] = [[0, 0, 0.035], [lean * 0.25, top * 0.45, 0.028], [lean * 0.65, top * 0.8, 0.016], [lean, top, 0]]
        let corner = { (ring: [Float], side: Float) -> [Float] in
            let (out, half) = (ring[0], ring[2] * side)
            return [cos(angle) * out + sin(angle) * half, ring[1], -sin(angle) * out + cos(angle) * half]
        }
        for step in 0..<3 {
            let (low, high) = (rings[step], rings[step + 1])
            var faces = [[corner(low, -1), corner(low, 1), corner(high, 1)]]
            if step < 2 {
                faces.append([corner(low, -1), corner(high, 1), corner(high, -1)])
            }
            for face in faces {
                for side in [face, Array(face.reversed())] {
                    let normal = faceNormal(side)
                    let up = normalize([normal[0] * 0.4, normal[1] * 0.4 + 1, normal[2] * 0.4])
                    mesh.addVertices(try side.map { try Vertex.pbr($0).set(Vertex.NORMAL, up).set(Vertex.COLOR0, colors[step]) })
                }
            }
        }
    }
    return mesh
}

/// A cone, or with \`prism\` set, a prism with the same sides.
func cone(_ radius: Float, _ bottom: Float, _ top: Float, _ sides: Int, prism: Bool = false) -> [[[Float]]] {
    let rim = { (angle: Float, y: Float) in [cos(angle) * radius, y, -sin(angle) * radius] }
    var triangles: [[[Float]]] = []
    for face in 0..<sides {
        let start = Float(face) / Float(sides) * tau
        let end = Float(face + 1) / Float(sides) * tau
        if prism {
            triangles.append([rim(start, bottom), rim(end, bottom), rim(end, top)])
            triangles.append([rim(start, bottom), rim(end, top), rim(start, top)])
        } else {
            triangles.append([rim(start, bottom), rim(end, bottom), [0, top, 0]])
        }
        triangles.append([rim(end, bottom), rim(start, bottom), [0, bottom, 0]])
    }
    return triangles
}

func column(_ color: [Float]) throws -> Mesh {
    try facets(cone(0.7, 0, 1, 4, prism: true).map { ($0, color) })
}

/// An icosahedron: twenty faces, shaded flat.
func boulder() throws -> Mesh {
    let golden: Float = 1.618
    let corners: [[Float]] = [
        [-1, golden, 0], [1, golden, 0], [-1, -golden, 0], [1, -golden, 0],
        [0, -1, golden], [0, 1, golden], [0, -1, -golden], [0, 1, -golden],
        [golden, 0, -1], [golden, 0, 1], [-golden, 0, -1], [-golden, 0, 1],
    ].map(normalize)
    let faces = [
        [0, 11, 5], [0, 5, 1], [0, 1, 7], [0, 7, 10], [0, 10, 11],
        [1, 5, 9], [5, 11, 4], [11, 10, 2], [10, 7, 6], [7, 1, 8],
        [3, 9, 4], [3, 4, 2], [3, 2, 6], [3, 6, 8], [3, 8, 9],
        [4, 9, 5], [2, 4, 11], [6, 2, 10], [8, 6, 7], [9, 8, 1],
    ]
    let triangles: [Triangle] = faces.enumerated().map { index, face in
        let shade = 0.16 + random(UInt32(index), 7) * 0.08
        return (face.map { corners[$0] }, [shade, shade * 0.97, shade * 0.92, 1])
    }
    return try facets(triangles)
}

/// A flat-shaded mesh: every triangle gets three vertices of its own, its face normal and its color.
func facets(_ triangles: [Triangle]) throws -> Mesh {
    let mesh = Mesh()
    for (corners, color) in triangles {
        let normal = faceNormal(corners)
        mesh.addVertices(try corners.map { try Vertex.pbr($0).set(Vertex.NORMAL, normal).set(Vertex.COLOR0, color) })
    }
    return mesh
}

func faceNormal(_ corners: [[Float]]) -> [Float] {
    normalize(cross(minus(corners[1], corners[0]), minus(corners[2], corners[0])))
}

func place(_ mesh: Mesh, _ material: Material, _ position: [Float], _ turn: Float, _ scale: [Float]) throws -> Model {
    let model = Model(mesh, material)
    try model.translate(position)
    try model.rotate([0, 1, 0], turn)
    try model.scale(scale)
    return model
}

/// The same pseudo-random numbers in every language: a 32-bit integer hash.
func random(_ index: UInt32, _ salt: UInt32) -> Float {
    var hash = (index ^ (salt &* 0x9e37_79b9)) &* 0x85eb_ca6b
    hash = (hash ^ (hash >> 13)) &* 0xc2b2_ae35
    return Float(Double(hash ^ (hash >> 16)) / 4_294_967_296)
}

func spot(_ index: UInt32, _ radius: Float) -> (Float, Float) {
    let distance = random(index, 8).squareRoot() * radius
    let angle = random(index, 9) * tau
    return (cos(angle) * distance, sin(angle) * distance)
}

func minus(_ first: [Float], _ second: [Float]) -> [Float] {
    [first[0] - second[0], first[1] - second[1], first[2] - second[2]]
}

func cross(_ first: [Float], _ second: [Float]) -> [Float] {
    [
        first[1] * second[2] - first[2] * second[1],
        first[2] * second[0] - first[0] * second[2],
        first[0] * second[1] - first[1] * second[0],
    ]
}

func normalize(_ vector: [Float]) -> [Float] {
    let length = vector.reduce(0) { $0 + $1 * $1 }.squareRoot()
    return vector.map { $0 / length }
}

func mix(_ start: [Float], _ end: [Float], _ amount: Float) -> [Float] {
    zip(start, end).map { $0 + ($1 - $0) * amount }
}

func smoothstep(_ low: Float, _ high: Float, _ value: Float) -> Float {
    let amount = min(max((value - low) / (high - low), 0), 1)
    return amount * amount * (3 - 2 * amount)
}
`;
}

function writeKotlin({ library, sky }) {
  const kotlinQuote = (text) => quote(text).replace(/\$/g, "\\$");
  const libraryLine = library.length
    ? wrap("", "private val LIBRARY = listOf(", library.map(kotlinQuote), ")", 100)
    : "private val LIBRARY = emptyList<String>()";
  return `// ${NEEDS}

import android.content.res.AssetManager
import kotlin.math.PI
import kotlin.math.abs
import kotlin.math.atan2
import kotlin.math.cos
import kotlin.math.exp
import kotlin.math.hypot
import kotlin.math.sin
import kotlin.math.sqrt
import kotlin.math.tan
import org.fragmentcolor.*

${libraryLine}

// The valley is SIZE meters across, and a day lasts DAY seconds. Hold T to hurry it.
private const val SIZE = 72f
private const val DAY = 180f
private const val TREES = 140
private const val ROCKS = 36u
private const val LAMPS = 8u
private const val GRASS = 8000u
private const val WATER = -1.2f

// Meters per second (Shift runs), eye height in meters, and radians of turn per pixel dragged.
private const val SPEED = 3.5f
private const val EYE = 1.6f
private const val TURN = 0.005f
private const val FOV = 1.1f
private const val TAU = (PI * 2).toFloat()

// Three corners and the color the triangle carries.
private typealias Triangle = Pair<List<List<Float>>, List<Float>>

// Blade colors from the soil to the tip, in three steps, for three kinds of tuft.
private val LUSH = listOf(listOf(0.04f, 0.12f, 0.02f, 1f), listOf(0.07f, 0.2f, 0.03f, 1f), listOf(0.13f, 0.29f, 0.05f, 1f))
private val DRY = listOf(listOf(0.05f, 0.12f, 0.02f, 1f), listOf(0.13f, 0.21f, 0.04f, 1f), listOf(0.26f, 0.3f, 0.09f, 1f))
private val SHORT = listOf(listOf(0.035f, 0.1f, 0.02f, 1f), listOf(0.055f, 0.16f, 0.025f, 1f), listOf(0.09f, 0.23f, 0.04f, 1f))

// What the wheel, a drag and a pinch did since the last frame: zoom in doublings
// (positive zooms in) and pan in view pixels.
data class Gesture(val zoom: Float = 0f, val pan: List<Float> = listOf(0f, 0f))

// The keys held down, named the way KeyboardEvent.code names them
// ("KeyW", "ArrowUp", "Space"), and a touch stick as [x, y] from -1 to 1, y up.
data class Keys(val down: Set<String> = emptySet(), val stick: List<Float> = listOf(0f, 0f)) {
    // 1 while any of codes is held down, 0 otherwise.
    fun held(vararg codes: String): Float = if (codes.any { it in down }) 1f else 0f
}

class Walker {
    var x = 0f
    var y = height(0f, 3f) + EYE
    var z = 3f
    var rise = 0f
    var yaw = -0.7f
    var pitch = -0.05f
    var walked = 0f
}

class State(
    val scene: Scene,
    val sky: Shader,
    val sun: Light,
    val moon: Light,
    val lamps: List<Light>,
    val bulb: Material,
    val camera: Camera,
    val obstacles: List<List<Float>>,
) {
    var size = listOf(0u, 0u)
    val walker = Walker()
    var clock = 0.28f * DAY
    var last: Float? = null
}

// Call setup once, then frame on each Choreographer callback.
fun setup(assets: AssetManager): State {
    val skySource = assets.open("sky.wgsl").bufferedReader().use { it.readText() }
    val sky = Shader(LIBRARY + listOf(skySource))
${setCalls("kotlin", "sky", sky)}

    // The valley draws over the sky. A pass that draws models gets a depth buffer,
    // so the nearest surface wins at each pixel.
    val world = Pass("world")
    world.loadPrevious()

    val scene = Scene()
    scene.addPass(Pass.fromShader("sky", sky)).addPass(world)

    scene.addTo("world", Model(ground(), Material.pbr().roughness(0.95f)))
    val water = Material.pbr().baseColor(listOf(0.02f, 0.08f, 0.1f, 0.8f)).roughness(0.08f).alphaMode(AlphaMode.BLEND)
    scene.addTo("world", Model(square(SIZE, WATER), water))

    // Trees and rocks share one mesh each, so each kind draws in one call.
    val obstacles = mutableListOf<List<Float>>()
    val tree = pine()
    val bark = Material.pbr().roughness(0.8f)
    for (index in 0u until (TREES * 8).toUInt()) {
        if (obstacles.size == TREES) break
        val (x, z) = spot(index, SIZE * 0.42f)
        val floor = height(x, z)
        if (floor < WATER + 0.5f || floor > 5f || hypot(x, z) < 6f) continue
        val scale = 0.7f + random(index, 2u) * 0.7f
        scene.addTo("world", place(tree, bark, listOf(x, floor - 0.1f, z), random(index, 3u) * TAU, listOf(scale, scale, scale)))
        obstacles.add(listOf(x, z, 0.35f * scale))
    }
    val rock = boulder()
    val stone = Material.pbr().roughness(0.7f)
    for (index in 0u until ROCKS) {
        val (x, z) = spot(index + 5000u, SIZE * 0.45f)
        val scale = 0.4f + random(index, 4u) * 0.9f
        val stretch = listOf(scale * (1f + random(index, 5u)), scale * 0.7f, scale)
        val position = listOf(x, height(x, z) - 0.15f * scale, z)
        scene.addTo("world", place(rock, stone, position, random(index, 6u) * TAU, stretch))
        obstacles.add(listOf(x, z, scale))
    }

    // Grass grows in meadows on the gentle slopes between the water and the rocks,
    // thicker where two slow waves meet. Its three kinds of tuft share one material.
    val kinds = listOf(tuft(9u, 0.55f, LUSH), tuft(6u, 0.85f, DRY), tuft(12u, 0.3f, SHORT))
    val grass = Material.pbr().roughness(0.85f)
    for (index in 0u until GRASS) {
        val (x, z) = spot(index + 9000u, SIZE * 0.4f)
        val floor = height(x, z)
        val slope = hypot(height(x + 0.5f, z) - floor, height(x, z + 0.5f) - floor)
        val waves = sin(x * 0.23f + 1.7f) * cos(z * 0.19f - 0.6f)
        val meadow = 0.55f + 0.35f * waves + 0.2f * sin((x + z) * 0.51f)
        if (floor < WATER + 0.4f || floor > 5f || slope > 0.25f || random(index, 10u) > meadow) continue
        val kind = if (random(index, 13u) < 0.12f) 1 else if (meadow > 0.7f) 0 else 2
        val size = 0.7f + random(index, 11u) * 0.6f
        val stretch = listOf(size, size * (0.8f + random(index, 14u) * 0.5f), size)
        val turn = random(index, 12u) * TAU
        scene.addTo("world", place(kinds[kind], grass, listOf(x, floor - 0.02f, z), turn, stretch))
    }

    // Lamps ring the clearing where the walk starts. Each one carries a point light.
    val post = column(listOf(0.02f, 0.018f, 0.016f, 1f))
    val metal = Material.pbr().metallic(0.6f).roughness(0.4f)
    val bulb = Material.pbr().baseColor(listOf(1f, 0.85f, 0.6f, 1f))
    val lamps = mutableListOf<Light>()
    for (index in 0u until LAMPS) {
        val angle = index.toFloat() / LAMPS.toFloat() * TAU
        val (x, z) = cos(angle) * 5f to sin(angle) * 5f
        val y = height(x, z)
        scene.addTo("world", place(post, metal, listOf(x, y, z), 0f, listOf(0.12f, 2.6f, 0.12f)))
        scene.addTo("world", place(post, bulb, listOf(x, y + 2.6f, z), 0.785f, listOf(0.32f, 0.32f, 0.32f)))
        val lamp = Light.point(listOf(x, y + 2.75f, z), listOf(1f, 0.72f, 0.42f)).setRange(14f)
        scene.addTo("world", lamp)
        lamps.add(lamp)
        obstacles.add(listOf(x, z, 0.3f))
    }

    val sun = Light.directional(listOf(0f, -1f, 0f), listOf(1f, 1f, 1f))
    val moon = Light.directional(listOf(0.3f, -1f, 0.5f), listOf(0.45f, 0.55f, 0.85f))
    val camera = Camera.perspective(FOV, 1f, 0.05f, 200f)
    scene.addTo("world", sun).addTo("world", moon).addTo("world", camera)

    return State(scene, sky, sun, moon, lamps, bulb, camera, obstacles)
}

fun frame(
    state: State,
    renderer: Renderer,
    target: WindowTarget,
    time: Float,
    size: List<Float>,
    pointer: List<Float>,
    gesture: Gesture = Gesture(),
    keys: Keys = Keys(),
) {
    val delta = state.last?.let { (time - it).coerceIn(0f, 0.1f) } ?: 0f
    state.last = time
    state.clock += delta * (if ("KeyT" in keys.down) 20f else 1f)

    // A new view size changes the camera's aspect.
    val pixels = listOf(size[0].coerceAtLeast(1f).toUInt(), size[1].coerceAtLeast(1f).toUInt())
    if (pixels != state.size) {
        state.camera.setAspect(pixels[0].toFloat() / pixels[1].toFloat())
        state.size = pixels
    }
    walk(state, delta, gesture, keys)
    light(state)
    state.sky.set("sky.resolution", listOf(state.size[0].toFloat(), state.size[1].toFloat()))
    renderer.render(state.scene, target)
}

private fun walk(state: State, delta: Float, gesture: Gesture, keys: Keys) {
    val walker = state.walker
    val turn = keys.held("ArrowRight") - keys.held("ArrowLeft")
    walker.yaw -= gesture.pan[0] * TURN + turn * 2f * delta
    walker.pitch = (walker.pitch - gesture.pan[1] * TURN).coerceIn(-1.4f, 1.4f)

    // Forward and right on the ground, from the keys, the stick and the wheel.
    var ahead = keys.held("KeyW", "ArrowUp") - keys.held("KeyS", "ArrowDown") + keys.stick[1]
    var side = keys.held("KeyD") - keys.held("KeyA") + keys.stick[0]
    val length = hypot(ahead, side)
    if (length > 1f) {
        ahead /= length
        side /= length
    }
    val speed = SPEED * (1f + keys.held("ShiftLeft", "ShiftRight"))
    val sine = sin(walker.yaw)
    val cosine = cos(walker.yaw)
    val step = ahead * speed * delta + gesture.zoom * 2f
    var x = walker.x - sine * step + cosine * side * speed * delta
    var z = walker.z - cosine * step - sine * side * speed * delta

    // Trees, rocks and lamps push back, the water stops you, and the hills keep you in.
    for ((stopX, stopZ, radius) in state.obstacles) {
        val awayX = x - stopX
        val awayZ = z - stopZ
        val gap = hypot(awayX, awayZ)
        val reach = radius + 0.3f
        if (gap > 0f && gap < reach) {
            x = stopX + awayX / gap * reach
            z = stopZ + awayZ / gap * reach
        }
    }
    val far = hypot(x, z) / (SIZE * 0.42f)
    if (far > 1f) {
        x /= far
        z /= far
    }
    if (height(x, z) > WATER + 0.1f) {
        walker.walked += hypot(x - walker.x, z - walker.z)
        walker.x = x
        walker.z = z
    }

    // Space jumps. Feet land on the ground, and the head bobs while walking.
    val floor = height(walker.x, walker.z) + EYE
    if (walker.y <= floor + 0.01f && "Space" in keys.down) walker.rise = 4.5f
    walker.rise -= 12f * delta
    walker.y = maxOf(floor, walker.y + walker.rise * delta)
    if (walker.y <= floor) walker.rise = 0f
    val bob = sin(walker.walked * 2.4f) * 0.05f

    val look = listOf(-sine * cos(walker.pitch), sin(walker.pitch), -cosine * cos(walker.pitch))
    val eye = listOf(walker.x, walker.y + bob, walker.z)
    state.camera.lookAt(eye, listOf(eye[0] + look[0], eye[1] + look[1], eye[2] + look[2]), listOf(0f, 1f, 0f))
    val right = normalize(cross(look, listOf(0f, 1f, 0f)))
    state.sky.set("sky.forward", look)
    state.sky.set("sky.right", right)
    state.sky.set("sky.up", cross(right, look))
    state.sky.set("sky.lens", tan(FOV / 2f))
}

// The sun turns once a day, above the horizon for two thirds of it. Lamps glow from dusk to dawn.
private fun light(state: State) {
    val angle = state.clock / DAY * TAU
    val sun = normalize(listOf(cos(angle), sin(angle) + 0.45f, 0.35f))
    val day = smoothstep(-0.08f, 0.12f, sun[1])
    val low = 1f - smoothstep(0f, 0.35f, abs(sun[1]))
    val warm = listOf(1f, 0.95f - low * 0.4f, 0.85f - low * 0.6f)
    state.sun.setDirection(sun.map { -it }).setColor(warm).setIntensity(3.2f * day)
    state.moon.setIntensity(0.35f * (1f - day))
    state.scene.ambient(mix(listOf(0.025f, 0.03f, 0.06f), listOf(0.28f, 0.32f, 0.4f), day))
    for (lamp in state.lamps) lamp.setIntensity(5f * (1f - day))
    state.bulb.emissive(mix(listOf(0.3f, 0.24f, 0.14f), listOf(1.2f, 0.75f, 0.3f), 1f - day))
    state.sky.set("sky.sun", sun)
    state.sky.set("sky.day", day)
}

// The ground: a flat clearing in rolling hills, a pond, and a rim that rises toward the edge.
private fun height(x: Float, z: Float): Float {
    val middle = hypot(x, z)
    val angle = atan2(z, x)
    val hills = sin(x * 0.15f) * cos(z * 0.12f) * 1.2f + sin(x * 0.31f + z * 0.23f) * 0.45f
    val edge = 0.6f + sin(angle * 3f) * 0.06f + sin(angle * 7f + 1f) * 0.04f
    val rim = maxOf(0f, middle / (SIZE / 2f) - edge)
    val peaks = 1f + sin(angle * 5f + 2f) * 0.35f
    val pond = exp(-((x - 11f) * (x - 11f) + (z + 9f) * (z + 9f)) / 45f)
    return hills * smoothstep(3f, 12f, middle) + rim * rim * 60f * peaks - pond * 3f
}

private fun ground(): Mesh {
    val cells = 72
    val cell = SIZE / cells
    val triangles = mutableListOf<Triangle>()
    for (row in 0 until cells) {
        for (slot in 0 until cells) {
            val x = -SIZE / 2f + row * cell
            val z = -SIZE / 2f + slot * cell
            fun corner(alongX: Float, alongZ: Float) = listOf(x + alongX, height(x + alongX, z + alongZ), z + alongZ)
            val here = corner(0f, 0f)
            val ahead = corner(0f, cell)
            val beside = corner(cell, 0f)
            val across = corner(cell, cell)
            for (corners in listOf(listOf(here, ahead, beside), listOf(beside, ahead, across))) {
                val y = (corners[0][1] + corners[1][1] + corners[2][1]) / 3f
                val up = faceNormal(corners)[1]
                val color = when {
                    y < WATER + 0.3f -> listOf(0.42f, 0.33f, 0.18f, 1f)
                    y > 10f -> listOf(0.7f, 0.72f, 0.76f, 1f)
                    up < 0.6f || y > 6f -> listOf(0.13f, 0.12f, 0.11f, 1f)
                    up < 0.85f -> listOf(0.035f, 0.1f, 0.02f, 1f)
                    random(triangles.size.toUInt(), 1u) < 0.5f -> listOf(0.05f, 0.16f, 0.025f, 1f)
                    else -> listOf(0.07f, 0.19f, 0.03f, 1f)
                }
                triangles.add(corners to color)
            }
        }
    }
    return facets(triangles)
}

private fun square(size: Float, y: Float): Mesh {
    val low = -size / 2f
    val high = size / 2f
    val white = listOf(1f, 1f, 1f, 1f)
    return facets(
        listOf(
            listOf(listOf(low, y, low), listOf(low, y, high), listOf(high, y, low)) to white,
            listOf(listOf(high, y, low), listOf(low, y, high), listOf(high, y, high)) to white,
        ),
    )
}

// A pine: a five-sided trunk under two stacked cones.
private fun pine(): Mesh {
    val trunk = cone(0.18f, 0f, 1.3f, 5, prism = true).map { it to listOf(0.12f, 0.06f, 0.03f, 1f) }
    val crown = (cone(1.2f, 1f, 3.2f, 7) + cone(0.85f, 2.2f, 4.3f, 7)).map { it to listOf(0.02f, 0.09f, 0.035f, 1f) }
    return facets(trunk + crown)
}

// A tuft: blades fanned around one root, each bending outward in three steps that
// taper to a tip and lighten from the soil up. A blade has a face on each side,
// since the material draws front faces only, and its normals lean toward the sky,
// so it takes the light the way the ground does.
private fun tuft(blades: UInt, tall: Float, colors: List<List<Float>>): Mesh {
    val mesh = Mesh()
    for (blade in 0u until blades) {
        val angle = blade.toFloat() * 2.4f
        val top = tall * (0.7f + random(blade, 20u) * 0.6f)
        val lean = 0.12f + random(blade, 21u) * 0.25f
        // From the root up: how far out, how high, and half the width.
        val rings = listOf(
            listOf(0f, 0f, 0.035f), listOf(lean * 0.25f, top * 0.45f, 0.028f),
            listOf(lean * 0.65f, top * 0.8f, 0.016f), listOf(lean, top, 0f),
        )
        fun corner(ring: List<Float>, side: Float): List<Float> {
            val (out, half) = ring[0] to ring[2] * side
            return listOf(cos(angle) * out + sin(angle) * half, ring[1], -sin(angle) * out + cos(angle) * half)
        }
        for (step in 0 until 3) {
            val (low, high) = rings[step] to rings[step + 1]
            val faces = mutableListOf(listOf(corner(low, -1f), corner(low, 1f), corner(high, 1f)))
            if (step < 2) faces.add(listOf(corner(low, -1f), corner(high, 1f), corner(high, -1f)))
            for (face in faces) {
                for (side in listOf(face, face.reversed())) {
                    val normal = faceNormal(side)
                    val up = normalize(listOf(normal[0] * 0.4f, normal[1] * 0.4f + 1f, normal[2] * 0.4f))
                    mesh.addVertices(side.map { Vertex.pbr(it).set(Vertex.NORMAL, up).set(Vertex.COLOR0, colors[step]) })
                }
            }
        }
    }
    return mesh
}

// A cone, or with prism set, a prism with the same sides.
private fun cone(radius: Float, bottom: Float, top: Float, sides: Int, prism: Boolean = false): List<List<List<Float>>> {
    fun rim(angle: Float, y: Float) = listOf(cos(angle) * radius, y, -sin(angle) * radius)
    val triangles = mutableListOf<List<List<Float>>>()
    for (face in 0 until sides) {
        val start = face.toFloat() / sides * TAU
        val end = (face + 1).toFloat() / sides * TAU
        if (prism) {
            triangles.add(listOf(rim(start, bottom), rim(end, bottom), rim(end, top)))
            triangles.add(listOf(rim(start, bottom), rim(end, top), rim(start, top)))
        } else {
            triangles.add(listOf(rim(start, bottom), rim(end, bottom), listOf(0f, top, 0f)))
        }
        triangles.add(listOf(rim(end, bottom), rim(start, bottom), listOf(0f, bottom, 0f)))
    }
    return triangles
}

private fun column(color: List<Float>): Mesh = facets(cone(0.7f, 0f, 1f, 4, prism = true).map { it to color })

// An icosahedron: twenty faces, shaded flat.
private fun boulder(): Mesh {
    val golden = 1.618f
    val corners = listOf(
        listOf(-1f, golden, 0f), listOf(1f, golden, 0f), listOf(-1f, -golden, 0f), listOf(1f, -golden, 0f),
        listOf(0f, -1f, golden), listOf(0f, 1f, golden), listOf(0f, -1f, -golden), listOf(0f, 1f, -golden),
        listOf(golden, 0f, -1f), listOf(golden, 0f, 1f), listOf(-golden, 0f, -1f), listOf(-golden, 0f, 1f),
    ).map(::normalize)
    val faces = listOf(
        listOf(0, 11, 5), listOf(0, 5, 1), listOf(0, 1, 7), listOf(0, 7, 10), listOf(0, 10, 11),
        listOf(1, 5, 9), listOf(5, 11, 4), listOf(11, 10, 2), listOf(10, 7, 6), listOf(7, 1, 8),
        listOf(3, 9, 4), listOf(3, 4, 2), listOf(3, 2, 6), listOf(3, 6, 8), listOf(3, 8, 9),
        listOf(4, 9, 5), listOf(2, 4, 11), listOf(6, 2, 10), listOf(8, 6, 7), listOf(9, 8, 1),
    )
    return facets(
        faces.mapIndexed { index, face ->
            val shade = 0.16f + random(index.toUInt(), 7u) * 0.08f
            face.map { corners[it] } to listOf(shade, shade * 0.97f, shade * 0.92f, 1f)
        },
    )
}

// A flat-shaded mesh: every triangle gets three vertices of its own, its face normal and its color.
private fun facets(triangles: List<Triangle>): Mesh {
    val mesh = Mesh()
    for ((corners, color) in triangles) {
        val normal = faceNormal(corners)
        mesh.addVertices(corners.map { Vertex.pbr(it).set(Vertex.NORMAL, normal).set(Vertex.COLOR0, color) })
    }
    return mesh
}

private fun faceNormal(corners: List<List<Float>>): List<Float> =
    normalize(cross(minus(corners[1], corners[0]), minus(corners[2], corners[0])))

private fun place(mesh: Mesh, material: Material, position: List<Float>, turn: Float, scale: List<Float>): Model {
    val model = Model(mesh, material)
    model.translate(position)
    model.rotate(listOf(0f, 1f, 0f), turn)
    model.scale(scale)
    return model
}

// The same pseudo-random numbers in every language: a 32-bit integer hash.
private fun random(index: UInt, salt: UInt): Float {
    var hash = (index xor (salt * 0x9e3779b9u)) * 0x85ebca6bu
    hash = (hash xor (hash shr 13)) * 0xc2b2ae35u
    return ((hash xor (hash shr 16)).toDouble() / 4294967296.0).toFloat()
}

private fun spot(index: UInt, radius: Float): Pair<Float, Float> {
    val distance = sqrt(random(index, 8u)) * radius
    val angle = random(index, 9u) * TAU
    return cos(angle) * distance to sin(angle) * distance
}

private fun minus(first: List<Float>, second: List<Float>) =
    listOf(first[0] - second[0], first[1] - second[1], first[2] - second[2])

private fun cross(first: List<Float>, second: List<Float>) = listOf(
    first[1] * second[2] - first[2] * second[1],
    first[2] * second[0] - first[0] * second[2],
    first[0] * second[1] - first[1] * second[0],
)

private fun normalize(vector: List<Float>): List<Float> {
    val length = sqrt(vector.sumOf { (it * it).toDouble() }).toFloat()
    return vector.map { it / length }
}

private fun mix(start: List<Float>, end: List<Float>, amount: Float) =
    start.zip(end) { low, high -> low + (high - low) * amount }

private fun smoothstep(low: Float, high: Float, value: Float): Float {
    val amount = ((value - low) / (high - low)).coerceIn(0f, 1f)
    return amount * amount * (3f - 2f * amount)
}
`;
}

export default {
  id: "walk",
  title: "Walkable world",
  summary:
    "A small valley built from Scene, Model, Material, Camera and Light, from afternoon into night. Click it, then walk with WASD or the arrow keys and drag to look. Space jumps. On a phone, the stick walks and a drag looks.",
  kind: "scene",
  needs: "0.12.7",
  gestures: true,
  keys: true,
  category: "3D",
  thumbnail: "/playground/thumbs/walk.webp",
  library: ["color/linear_to_srgb"],
  values: VALUES,
  files: { "sky.wgsl": SKY },
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
