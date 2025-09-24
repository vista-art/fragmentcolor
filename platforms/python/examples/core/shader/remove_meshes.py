from fragmentcolor import Pass, Shader, Mesh, Vertex

shader = Shader.default()
rpass = Pass("p"); rpass.add_shader(shader)

m1 = Mesh()
m1.add_vertex(Vertex([0.0, 0.0]))
m2 = Mesh()
m2.add_vertex(Vertex([0.5, 0.0]))

shader.add_mesh(m1)
shader.add_mesh(m2)

shader.remove_meshes([m1, m2])