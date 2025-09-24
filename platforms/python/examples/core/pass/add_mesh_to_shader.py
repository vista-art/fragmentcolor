from fragmentcolor import Pass, Shader, Mesh, Vertex

shader = Shader.default()
rpass = Pass("p"); rpass.add_shader(shader)

mesh = Mesh()
mesh.add_vertex(Vertex([0.0, 0.0]))

rpass.add_mesh_to_shader(mesh, shader).expect("mesh is compatible")