from fragmentcolor import Pass, Shader, Mesh, Vertex

mesh = Mesh()
mesh.add_vertex(Vertex([0.0, 0.0]))

shader = Shader.from_mesh(mesh)
rpass = Pass("rpass"); rpass.add_shader(shader)

rpass.add_mesh(mesh).expect("mesh is compatible")