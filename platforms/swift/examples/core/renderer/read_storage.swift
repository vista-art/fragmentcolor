use bytemuck
import FragmentColor

let renderer = Renderer()
let target = try await renderer.createTextureTarget([16, 16])

let compute = try Shader.new(
    r#"
    struct Out { values: array<f32, 4> }
    @group(0) @binding(0) var<storage, read_write> out: Out
    @compute @workgroup_size(1) fn main() {
        out.values[0] = 1.0
        out.values[1] = 2.0
        out.values[2] = 3.0
        out.values[3] = 4.0
    }
    "#,
)

let pass = Pass.compute("seed")
pass.setComputeDispatch(1, 1, 1)
pass.addShader(compute)
try renderer.render(pass, target)

let bytes = try await renderer.readStorage(compute, "out")
let values = bytemuck.castSlice(bytes)