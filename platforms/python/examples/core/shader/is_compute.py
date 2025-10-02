from fragmentcolor import Shader

shader = Shader("""
@compute @workgroup_size(1)
fn cs_main() { }

""")

# Call the method
is_compute = shader.is_compute()
