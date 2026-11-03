from fragmentcolor import Mesh, Shader

mesh = Mesh()
mesh.add_vertices([
    [-0.5, -0.5, 0.0],
    [0.5, -0.5, 0.0],
    [0.0, 0.5, 0.0],
])
shader = Shader.from_mesh(mesh)
