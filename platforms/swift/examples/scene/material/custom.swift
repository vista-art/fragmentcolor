import FragmentColor

let flat = try Shader("""
    struct Camera { view_proj: mat4x4<f32>, position: vec3<f32> }
    struct Light { kind: u32, intensity: f32, range: f32, inner_cone_cos: f32,
        position: vec3<f32>, outer_cone_cos: f32, direction: vec3<f32>, color: vec3<f32> }
    struct LightGrid { view_proj: mat4x4<f32>, depth_plane: vec4<f32>, size: vec3<u32>,
        slicing: u32, slice_scale: f32, slice_bias: f32, reach: f32 }
    struct Lights { count: u32, global_count: u32, promoted_count: u32,
        ambient: vec3<f32>, grid: LightGrid, lights: array<Light> }
    @group(0) @binding(0) var<uniform> camera: Camera;
    @group(0) @binding(1) var<storage, read> lights: Lights;

    @vertex
    fn vs_main(
        @location(0) p: vec3<f32>,
        @location(3) model_0: vec4<f32>,
        @location(4) model_1: vec4<f32>,
        @location(5) model_2: vec4<f32>,
        @location(6) model_3: vec4<f32>,
    ) -> @builtin(position) vec4<f32> {
        let model = mat4x4<f32>(model_0, model_1, model_2, model_3);
        return camera.view_proj * model * vec4<f32>(p, 1.0);
    }
    @fragment fn fs_main() -> @location(0) vec4<f32> {
        var glow = lights.ambient;
        for (var i = 0u; i < lights.count; i++) {
            glow += lights.lights[i].color * lights.lights[i].intensity * 0.1;
        }
        return vec4<f32>(glow, 1.0);
    }

""")

let flat_mat = Material.custom(flat)