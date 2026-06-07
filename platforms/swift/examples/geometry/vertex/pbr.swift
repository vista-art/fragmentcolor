import FragmentColor

// Build a triangle; override only what the mesh actually carries — NORMAL
// / COLOR0 / UV1 / TANGENT use their identity defaults from Vertex.pbr.
let mesh = Mesh()
try mesh.addVertex(Vertex.pbr([ 0.0,  0.5, 0.0]).set("uv0", [0.5, 1.0]))
try mesh.addVertex(Vertex.pbr([-0.5, -0.5, 0.0]).set("uv0", [0.0, 0.0]))
try mesh.addVertex(Vertex.pbr([ 0.5, -0.5, 0.0]).set("uv0", [1.0, 0.0]))