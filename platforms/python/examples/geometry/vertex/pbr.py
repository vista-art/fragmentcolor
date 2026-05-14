from fragmentcolor import Mesh, Vertex

mesh = Mesh()
for (pos, uv) in [
    ([0.0_, 0.5, 0.0], [0.5_, 1.0]),
    ([-0.5, -0.5, 0.0], [0.0, 0.0]),
    ([0.5, -0.5, 0.0], [1.0, 0.0]),
] {
# Override only what the mesh actually carries; NORMAL / COLOR0 / UV1 /
# TANGENT use their identity defaults.
    mesh.add_vertex(Vertex.pbr(pos).set(Vertex.UV0, uv))
}