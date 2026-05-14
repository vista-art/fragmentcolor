import FragmentColor

let renderer = Renderer()
let target = try await renderer.createTextureTarget([256, 256])

// Bind the offscreen target's contents as a uniform on a downstream
// post-processing shader.
let post = try Shader("""
    @group(0) @binding(0) var input_image : texture_2d<f32>
    @group(0) @binding(1) var input_sampler : sampler

    @vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
        let p = array<vec2<f32>, 3>(vec2f(-1.0,-1.0), vec2f(3.0,-1.0), vec2f(-1.0,3.0))
        return vec4<f32>(p[i], 0.0, 1.0)
    }
    @fragment fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
        return textureSample(input_image, input_sampler, vec2<f32>(0.5, 0.5))
    }

""")
try post.set("input_image", target.texture())