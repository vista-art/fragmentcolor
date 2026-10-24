import org.fragmentcolor.*

val shader = Shader("""
@compute @workgroup_size(1)
fn cs_main() { }

""")
val pass = Pass.fromShader("p", shader)

// Call the method
val is_compute = pass.isCompute()