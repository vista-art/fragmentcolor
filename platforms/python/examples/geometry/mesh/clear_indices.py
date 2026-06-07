from fragmentcolor import Mesh, Vertex

mesh = Mesh()
mesh.add_vertices([
    Vertex([-0.5, -0.5]),
    Vertex([ 0.5, -0.5]),
    Vertex([ 0.0,  0.5]),
])
mesh.set_indices([0, 1, 2])
mesh.clear_indices(); # back to auto-derived dedup