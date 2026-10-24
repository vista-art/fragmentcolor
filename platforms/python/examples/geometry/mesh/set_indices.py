from fragmentcolor import Mesh, Vertex

# A quad split into two triangles via explicit indexing. The four corners
# happen to carry distinct UVs (only positions repeat), so we keep them
# all and reference each by index.
mesh = Mesh()
mesh.add_vertices([
    Vertex([-0.5, -0.5]).set("uv", [0.0, 0.0]),
    Vertex([ 0.5, -0.5]).set("uv", [1.0, 0.0]),
    Vertex([ 0.5,  0.5]).set("uv", [1.0, 1.0]),
    Vertex([-0.5,  0.5]).set("uv", [0.0, 1.0]),
])
mesh.set_indices([0, 1, 2, 0, 2, 3])