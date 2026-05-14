import FragmentColor

let mesh = Mesh()
for (pos, uv) in [
    ([0.0, 0.5, 0.0], [0.5, 1.0]),
    ([-0.5, -0.5, 0.0], [0.0, 0.0]),
    ([0.5, -0.5, 0.0], [1.0, 0.0]),
] {
    // Override only what the mesh actually carries; NORMAL / COLOR0 / UV1 /
    // TANGENT use their identity defaults.
    try mesh.addVertex(Vertex.pbr(pos).set(Vertex.uV0, uv))
}