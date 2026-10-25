//! A particle system: a compute shader moves eight million particles and counts them in a grid.

#[path = "../host.rs"]
mod host;

use fragmentcolor::{FragmentColorError, Pass, Renderer, Shader, Target};

const COUNT: u32 = 8_000_000;
const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

const CLEAR: &str = r#"
// #region: clear-shader
@group(0) @binding(0) var<storage, read_write> counts: array<atomic<u32>, CELLS>;

@compute @workgroup_size(256)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x < CELLS) {
        atomicStore(&counts[id.x], 0u);
    }
}
// #endregion: clear-shader
"#;

const SIMULATE: &str = r#"
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

    // A new buffer holds zeros, so the first frame starts every particle
    // circling the center. After that, about one in 300 starts over on each frame.
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
    // The pull weakens with distance; 0.05 keeps it finite right at the attractor.
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
"#;

const DRAW: &str = r#"
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
"#;

/// What `frame` needs from `setup`.
struct State {
    simulate: Shader,
    draw: Shader,
    output: Pass,
}

// #region: setup
fn setup() -> Result<State, FragmentColorError> {
    // All three shaders take their sizes from these WGSL constants.
    let sizes = format!(
        "const COUNT: u32 = {COUNT}u;
         const GRID = vec2<u32>({WIDTH}u, {HEIGHT}u);
         const CELLS: u32 = {WIDTH}u * {HEIGHT}u;"
    );
    let clear = Shader::new([sizes.as_str(), CLEAR])?;
    let simulate = Shader::new([sizes.as_str(), SIMULATE])?;
    let draw = Shader::new([sizes.as_str(), DRAW])?;

    // One workgroup handles 256 items, as `@workgroup_size(256)` declares.
    let clear_pass = Pass::from_shader("clear", &clear);
    clear_pass.set_compute_dispatch((WIDTH * HEIGHT).div_ceil(256), 1, 1);

    let simulate_pass = Pass::from_shader("simulate", &simulate);
    simulate_pass.set_compute_dispatch(COUNT.div_ceil(256), 1, 1);
    simulate_pass.require(&clear_pass)?;

    let output = Pass::from_shader("draw", &draw);
    output.require(&simulate_pass)?;

    Ok(State {
        simulate,
        draw,
        output,
    })
}
// #endregion: setup

// #region: frame
fn frame(
    state: &State,
    renderer: &Renderer,
    target: &impl Target,
    time: f32,
) -> Result<(), FragmentColorError> {
    let size = target.size();
    let resolution = [size.width as f32, size.height as f32];
    state.simulate.set("resolution", resolution)?;
    state.simulate.set("time", time)?;
    state.draw.set("resolution", resolution)?;
    renderer.render(&state.output, target)?;
    Ok(())
}
// #endregion: frame

fn main() {
    host::run(async |_: &Renderer| setup(), frame);
}
