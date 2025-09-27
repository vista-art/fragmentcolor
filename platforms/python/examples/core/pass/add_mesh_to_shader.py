from fragmentcolor import Shader, Mesh, Vertex, Pass

mesh = Mesh()
mesh.add_vertex([0.0, 0.0])
shader = Shader.from_mesh(mesh)
rpass = Pass("rpass"); rpass.add_shader(shader)

rpass.add_mesh_to_shader(mesh, shader)
