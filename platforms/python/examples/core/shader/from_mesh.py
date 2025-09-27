from fragmentcolor import Mesh, Shader, Vertex

mesh = Mesh()
mesh.add_vertex(Vertex([0.0, 0.0, 0.0]))
shader = Shader.from_mesh(mesh)
