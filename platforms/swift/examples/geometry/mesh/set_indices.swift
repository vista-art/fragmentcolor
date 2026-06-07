import FragmentColor

// A quad split into two triangles via explicit indexing. The four corners
// happen to carry distinct UVs (only positions repeat), so we keep them
// all and reference each by index.
let mesh = Mesh()
let uv00 = [0.0, 0.0]
let uv10 = [1.0, 0.0]
let uv11 = [1.0, 1.0]
let uv01 = [0.0, 1.0]
try mesh.addVertices([
    try Vertex([-0.5, -0.5]).set("uv", uv00),
    try Vertex([ 0.5, -0.5]).set("uv", uv10),
    try Vertex([ 0.5,  0.5]).set("uv", uv11),
    try Vertex([-0.5,  0.5]).set("uv", uv01),
])
try mesh.setIndices([0, 1, 2, 0, 2, 3])