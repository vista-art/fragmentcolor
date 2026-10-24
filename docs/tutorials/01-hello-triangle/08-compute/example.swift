// Step 8: a compute shader moves 8 million particles and counts them in a grid.
// Call `setup` once, then call `frame` on every display refresh, such as from an `MTKView` delegate.

import FragmentColor

let count: UInt32 = 8_000_000
let width: UInt32 = 1280
let height: UInt32 = 720

let clearSource = """
// #region: clear-shader
@group(0) @binding(0) var<storage, read_write> counts: array<atomic<u32>, CELLS>;

@compute @workgroup_size(256)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x < CELLS) {
        atomicStore(&counts[id.x], 0u);
    }
}
// #endregion: clear-shader
"""

let simulateSource = """
// #region: simulate-shader
struct Particle {
    position: vec2<f32>,
    velocity: vec2<f32>,
};

@group(0) @binding(0) var<storage, read_write> particles: array<Particle, COUNT>;
@group(0) @binding(1) var<storage, read_write> counts: array<atomic<u32>, CELLS>;
@group(0) @binding(2) var<uniform> resolution: vec2<f32>;
@group(0) @binding(3) var<uniform> time: f32;

fn hash(value: u32) -> u32 {
    let state = value * 747796405u + 2891336453u;
    let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

fn random(seed: u32) -> f32 {
    return f32(hash(seed)) / 4294967295.0;
}

@compute @workgroup_size(256)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let index = id.x;
    if (index >= COUNT) {
        return;
    }
    var particle = particles[index];

    // A new buffer holds zeros, so the first frame places every particle on a
    // circular orbit. After that, about one in 300 starts over on each frame.
    // Mixing in the bit pattern of time gives each frame its own dice.
    let seed = hash(index ^ bitcast<u32>(time));
    let fresh = all(particle.velocity == vec2<f32>(0.0));
    if (fresh || random(seed) < 0.003) {
        let radius = 0.2 + 0.8 * sqrt(random(seed + 1u));
        let angle = random(seed + 2u) * 6.283;
        let spot = vec2<f32>(cos(angle), sin(angle));
        particle.position = spot * radius;
        particle.velocity = vec2<f32>(-spot.y, spot.x) * 0.6;
    }

    // An attractor wanders through the middle and pulls every particle.
    // The pull weakens with distance; 0.05 keeps it finite at the center.
    let attractor = vec2<f32>(cos(time * 0.37), sin(time * 0.61)) * 0.4;
    let offset = attractor - particle.position;
    let pull = offset / (dot(offset, offset) + 0.05);

    // Each frame advances the simulation by 1/60 of a second, with a little drag.
    let step = 1.0 / 60.0;
    particle.velocity = (particle.velocity + pull * step) * 0.995;
    particle.position += particle.velocity * step;
    particles[index] = particle;

    // Count the particle in the grid cell under it.
    let fit = min(resolution.x, resolution.y) / resolution;
    let uv = particle.position * fit * 0.5 + 0.5;
    if (all(uv >= vec2<f32>(0.0)) && all(uv < vec2<f32>(1.0))) {
        let cell = vec2<u32>(uv * vec2<f32>(GRID));
        atomicAdd(&counts[cell.y * GRID.x + cell.x], 1u);
    }
}
// #endregion: simulate-shader
"""

let drawSource = """
// #region: draw-shader
@group(0) @binding(0) var<storage, read> counts: array<u32, CELLS>;
@group(0) @binding(1) var<uniform> resolution: vec2<f32>;

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    // One triangle large enough to cover the whole target.
    var corners = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0),
    );
    return vec4<f32>(corners[i], 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) pixel: vec4<f32>) -> @location(0) vec4<f32> {
    // Pixel rows count down from the top, and grid rows count up from the bottom.
    let uv = vec2<f32>(pixel.x, resolution.y - pixel.y) / resolution;
    let cell = min(vec2<u32>(uv * vec2<f32>(GRID)), GRID - 1u);
    let count = f32(counts[cell.y * GRID.x + cell.x]);

    // A logarithm keeps a lone particle visible next to a crowd of a thousand.
    let heat = clamp(log2(1.0 + count) / 10.0, 0.0, 1.0);
    let night = vec3<f32>(0.02, 0.02, 0.06);
    let ember = vec3<f32>(0.95, 0.45, 0.25);
    return vec4<f32>(mix(night, ember, heat) + heat * heat * 0.5, 1.0);
}
// #endregion: draw-shader
"""

/// What `frame` needs from `setup`.
struct State {
    let simulate: Shader
    let draw: Shader
    let output: Pass
}

// #region: setup
func setup() throws -> State {
    // All three shaders size their buffers with these WGSL constants.
    let sizes = """
        const COUNT: u32 = \(count)u;
        const GRID = vec2<u32>(\(width)u, \(height)u);
        const CELLS: u32 = \(width)u * \(height)u;
        """
    let clear = try Shader([sizes, clearSource])
    let simulate = try Shader([sizes, simulateSource])
    let draw = try Shader([sizes, drawSource])

    // One workgroup handles 256 items, as `@workgroup_size(256)` declares.
    let clearPass = Pass.fromShader("clear", clear)
    clearPass.setComputeDispatch((width * height + 255) / 256, 1, 1)

    let simulatePass = Pass.fromShader("simulate", simulate)
    simulatePass.setComputeDispatch((count + 255) / 256, 1, 1)
    try simulatePass.require(clearPass)

    let output = Pass.fromShader("draw", draw)
    try output.require(simulatePass)

    return State(simulate: simulate, draw: draw, output: output)
}
// #endregion: setup

// #region: frame
func frame(_ state: State, _ renderer: Renderer, _ target: WindowTarget, _ time: Float) throws {
    let size = target.size()
    let resolution = [Float(size.width), Float(size.height)]
    try state.simulate.set("resolution", resolution)
    try state.simulate.set("time", time)
    try state.draw.set("resolution", resolution)
    try renderer.render(state.output, target)
}
// #endregion: frame
