from fragmentcolor import Pass, Shader

cs = Shader("@compute @workgroup_size(8,8,1) fn cs_main() {}")
rpass = Pass("compute"); rpass.add_shader(cs)