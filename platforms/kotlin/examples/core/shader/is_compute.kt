import org.fragmentcolor.*

val shader = Shader("""
@compute @workgroup_size(1)
fn cs_main() { }

""")

// Call the method
val is_compute = shader.isCompute()