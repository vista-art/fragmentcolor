use bytemuck
import org.fragmentcolor.*

val renderer = Renderer()
val target = renderer.createTextureTarget(16u, 16u)

val compute = Shader.new( r#"; struct Out { values: array<f32, 4> }; @group(0) @binding(0) var<storage, read_write> out: Out; @compute @workgroup_size(1) fn main() { out.values[0] = 1.0; out.values[1] = 2.0; out.values[2] = 3.0; out.values[3] = 4.0; }; "#, )

val pass = Pass.compute("seed")
pass.setComputeDispatch(1u,1u,1u)
pass.addShader(compute)
renderer.render(pass, target)

val bytes = renderer.readStorage(compute, "out")
val values = bytemuck.castSlice(bytes)