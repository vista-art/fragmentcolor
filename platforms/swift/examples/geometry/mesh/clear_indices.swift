import FragmentColor

let mesh = Mesh()
try mesh.addVertices([
    try Vertex([-0.5, -0.5]),
    try Vertex([ 0.5, -0.5]),
    try Vertex([ 0.0,  0.5]),
])
try mesh.setIndices([0, 1, 2])
mesh.clearIndices(); // back to auto-derived dedup