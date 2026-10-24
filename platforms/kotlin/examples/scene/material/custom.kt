import org.fragmentcolor.*

val wireframe = Shader("""
    struct Camera { view_proj: mat4x4<f32>, position: vec3<f32> }
    @group(0) @binding(0) var<uniform> camera: Camera;

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
        return vec4<f32>(0.0, 1.0, 0.4, 1.0);
    }

""")

val wire_mat = Material.custom(wireframe)