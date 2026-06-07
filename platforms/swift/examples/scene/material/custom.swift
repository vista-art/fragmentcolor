import FragmentColor

let wireframe = try Shader("""
    struct MeshTransform { model: mat4x4<f32> }
    struct Camera { view_proj: mat4x4<f32>, position: vec3<f32> }
    @group(0) @binding(0) var<uniform> camera: Camera
    @group(1) @binding(0) var<uniform> mesh: MeshTransform

    @vertex
    fn vs_main(@location(0) p: vec3<f32>) -> @builtin(position) vec4<f32> {
        return camera.view_proj * mesh.model * vec4<f32>(p, 1.0)
    }
    @fragment fn fs_main() -> @location(0) vec4<f32> {
        return vec4<f32>(0.0, 1.0, 0.4, 1.0)
    }

""")

let wire_mat = Material.custom(wireframe)