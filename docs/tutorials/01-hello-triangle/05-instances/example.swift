// Step 5: the GPU draws one mesh 15000 times, each copy with its own place, phase and tint.
// Call `setup` once, then call `frame` on every display refresh, such as from an `MTKView` delegate.

import FragmentColor

let copies = 15000

let particles = """
// #region: shader
struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@group(0) @binding(0) var<uniform> resolution: vec2<f32>;
@group(0) @binding(1) var<uniform> time: f32;

@vertex
fn vs_main(
    @location(0) position: vec3<f32>, // per vertex
    @location(1) color: vec3<f32>,    // per vertex
    @location(2) center: vec2<f32>,   // per instance
    @location(3) phase: f32,          // per instance
    @location(4) tint: vec3<f32>,     // per instance
) -> VOut {
    // Shrink the triangle and keep its shape, as in step 2.
    let aspect = resolution.x / resolution.y;
    var shape = position.xy * 0.03;
    if (aspect > 1.0) { shape.x = shape.x / aspect; }
    else              { shape.y = shape.y * aspect; }

    // Each copy loops around its center, offset by its own phase.
    let wobble = vec2<f32>(
        sin(time * 1.3 + phase),
        cos(time * 0.9 + phase * 1.4),
    ) * 0.03;

    // in_out_sine comes from easing/in_out_sine and flattens the wave near 0 and 1.
    let wave = 0.5 + 0.5 * sin(time * 2.0 + phase);
    let glow = 0.75 + 0.75 * in_out_sine(wave);

    var out: VOut;
    out.pos = vec4<f32>(center + wobble + shape, 0.0, 1.0);
    out.color = color * tint * glow;
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
    let shader = try Shader(["easing/in_out_sine", particles])

    let mesh = Mesh()
    mesh.addVertices([
        try Vertex([-0.7, -0.404, 0.0]).set("color", [0.95, 0.30, 0.42]),
        try Vertex([0.7, -0.404, 0.0]).set("color", [0.30, 0.85, 0.55]),
        try Vertex([0.0, 0.808, 0.0]).set("color", [0.30, 0.55, 0.95]),
    ])

    for _ in 0..<copies {
        let center = [Float.random(in: -1...1), Float.random(in: -1...1)]
        let phase = Float.random(in: 0..<(2 * .pi))
        let tint = (0..<3).map { _ in Float.random(in: 0.2...1.0) }
        try mesh.addInstance(
            Instance()
                .set("center", center)
                .set("phase", phase)
                .set("tint", tint)
        )
    }

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
