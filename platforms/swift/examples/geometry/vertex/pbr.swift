import FragmentColor

// Build a triangle and set only what the mesh carries. NORMAL, COLOR0,
// UV1 and TANGENT keep their identity defaults from Vertex.pbr.
let mesh = Mesh()
try mesh.addVertex(Vertex.pbr([ 0.0,  0.5, 0.0]).set(Vertex.UV0, [0.5, 1.0]))
try mesh.addVertex(Vertex.pbr([-0.5, -0.5, 0.0]).set(Vertex.UV0, [0.0, 0.0]))
try mesh.addVertex(Vertex.pbr([ 0.5, -0.5, 0.0]).set(Vertex.UV0, [1.0, 0.0]))