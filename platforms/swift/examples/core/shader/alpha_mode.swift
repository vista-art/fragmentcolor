import FragmentColor

let shader = try Shader("""
@vertex
fn vs_main(@location(0) position: vec3<f32>) -> @builtin(position) vec4<f32> {
    return vec4<f32>(position, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(0.6, 0.8, 1.0, 0.25);
}

""")

let glass = shader.alphaMode(AlphaMode.blend)