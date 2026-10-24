// Step 7: the GPU places a million triangles from their instance index alone.
// Call `setup` once, then call `frame` on every display refresh, such as from an `MTKView` delegate.

import FragmentColor

let copies: UInt32 = 1_000_000

let galaxy = """
// #region: shader
struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@group(0) @binding(0) var<uniform> resolution: vec2<f32>;
@group(0) @binding(1) var<uniform> time: f32;

// PCG hash: mixes the bits of a number so the result looks random.
fn hash(value: u32) -> u32 {
    let state = value * 747796405u + 2891336453u;
    let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

// The same seed always gives the same number between 0 and 1.
fn random(seed: u32) -> f32 {
    return f32(hash(seed)) / 4294967295.0;
}

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    @builtin(instance_index) index: u32,
) -> VOut {
    // Four random numbers per copy, drawn from its index.
    let radius = random(index * 4u);
    let arm = f32(hash(index * 4u + 1u) % 3u) * 2.094;
    let spread = (random(index * 4u + 2u) - 0.5) * (1.2 - radius);
    let brightness = 0.4 + 0.6 * random(index * 4u + 3u);

    // Three spiral arms, turning slowly as one disc.
    let angle = arm + radius * 5.0 + spread - time * 0.15;
    let center = vec2<f32>(cos(angle), sin(angle)) * radius * 0.95;

    // Keep the disc round on a canvas of any shape.
    let fit = min(resolution.x, resolution.y) / resolution;
    // Measured in pixels, each triangle is about four wide.
    let shape = position * 6.0 / resolution;

    var out: VOut;
    out.pos = vec4<f32>(center * fit + shape, 0.0, 1.0);
    let core = vec3<f32>(1.0, 0.8, 0.55);
    let rim = vec3<f32>(0.35, 0.55, 1.0);
    out.color = mix(core, rim, radius) * brightness;
    return out;
}

@fragment
fn fs_main(in: VOut) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
// #endregion: shader
"""

// #region: setup
func setup() throws -> Shader {
    let shader = try Shader(galaxy)

    let mesh = Mesh()
    try mesh.addVertices([[-0.7, -0.404], [0.7, -0.404], [0.0, 0.808]])
    mesh.setInstanceCount(copies)

    try shader.addMesh(mesh)
    return shader
}
// #endregion: setup

// #region: frame
func frame(_ shader: Shader, _ renderer: Renderer, _ target: WindowTarget, _ time: Float) throws {
    let size = target.size()
    try shader.set("resolution", [Float(size.width), Float(size.height)])
    try shader.set("time", time)
    try renderer.render(shader, target)
}
// #endregion: frame
