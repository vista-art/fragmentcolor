import FragmentColor

let shader = Shader("""
@compute @workgroup_size(1)
fn cs_main() { }

""")

// Call the method
let is_compute = shader.isCompute()