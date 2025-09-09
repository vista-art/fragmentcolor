from fragmentcolor import Pass, Shader

shader = Shader.default()
rpass = Pass("single"); rpass.add_shader(shader)