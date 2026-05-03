import FragmentColor
let m = Mesh()
try m.addVertices([
    [-0.01, -0.01],
    [ 0.01, -0.01],
    [ 0.00,  0.01],
])
// Draw one million instances, fetching per-particle data from a storage buffer.
m.setInstanceCount(1_000_000)