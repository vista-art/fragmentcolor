import FragmentColor

let shader = try Shader("""
@compute @workgroup_size(1)
fn cs_main() { }

""")
let pass = Pass.fromShader("p", shader)

// Call the method
let is_compute = pass.isCompute()