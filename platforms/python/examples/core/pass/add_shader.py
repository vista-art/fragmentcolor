from fragmentcolor import Pass, Shader

shader = Shader.default()
rpass = Pass("p")
rpass.add_shader(shader)