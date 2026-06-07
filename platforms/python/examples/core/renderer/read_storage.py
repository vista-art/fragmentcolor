from fragmentcolor import Pass, Renderer, Shader

renderer = Renderer()
target = renderer.create_texture_target([16, 16])

compute = Shader("""
    struct Out { values: array<f32, 4> };
    @group(0) @binding(0) var<storage, read_write> out: Out;
    @compute @workgroup_size(1) fn main() {
        out.values[0] = 1.0;
        out.values[1] = 2.0;
        out.values[2] = 3.0;
        out.values[3] = 4.0;
    }
    
""")

rpass = Pass.compute("seed")
rpass.set_compute_dispatch(1, 1, 1)
rpass.add_shader(compute)
renderer.render(rpass, target)

bytes = renderer.read_storage(compute, "out")