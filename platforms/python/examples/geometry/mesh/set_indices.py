from fragmentcolor import Mesh, Vertex

# A quad split into two triangles via explicit indexing. The four corners
# happen to carry distinct UVs (only positions repeat), so we keep them
# all and reference each by index.
mesh = Mesh()
uv00 = [0.0, 0.0]
uv10 = [1.0, 0.0]
uv11 = [1.0, 1.0]
uv01 = [0.0, 1.0]
mesh.add_vertices([
    Vertex([-0.5, -0.5]).set("uv", uv00),
    Vertex([ 0.5, -0.5]).set("uv", uv10),
    Vertex([ 0.5,  0.5]).set("uv", uv11),
    Vertex([-0.5,  0.5]).set("uv", uv01),
])
mesh.set_indices([0, 1, 2, 0, 2, 3])