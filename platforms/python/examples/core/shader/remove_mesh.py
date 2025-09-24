from fragmentcolor import Pass, Shader, Mesh, Vertex

shader = Shader.default()
rpass = Pass("p"); rpass.add_shader(shader)

mesh = Mesh()
mesh.add_vertex(Vertex([0.0, 0.0]))
shader.add_mesh(mesh)

# Detach the mesh
shader.remove_mesh(mesh)