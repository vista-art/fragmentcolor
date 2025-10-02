from fragmentcolor import Shader, Pass

shader = Shader("""
@compute @workgroup_size(1)
fn cs_main() { }

""")
rpass = Pass("p"); rpass.add_shader(shader)

# Call the method
is_compute = rpass.is_compute()
