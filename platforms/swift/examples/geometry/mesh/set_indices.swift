import FragmentColor

// A quad split into two triangles via explicit indexing. The four corners
// happen to carry distinct UVs (only positions repeat), so we keep them
// all and reference each by index.
let mesh = Mesh()
try mesh.addVertices([
    try Vertex([-0.5, -0.5]).set("uv", [0.0, 0.0]),
    try Vertex([ 0.5, -0.5]).set("uv", [1.0, 0.0]),
    try Vertex([ 0.5,  0.5]).set("uv", [1.0, 1.0]),
    try Vertex([-0.5,  0.5]).set("uv", [0.0, 1.0]),
])
mesh.setIndices([0, 1, 2, 0, 2, 3])