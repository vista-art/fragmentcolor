from fragmentcolor import Pass, Shader
from fragmentcolor import {Mesh, Vertex}, Shader

shader = Shader.default()
rpass = Pass("p"); rpass.add_shader(shader)
mesh = Mesh()
mesh.add_vertex(Vertex.from([0.0, 0.0]))
rpass.add_mesh(mesh)