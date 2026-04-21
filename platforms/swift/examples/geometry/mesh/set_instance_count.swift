import FragmentColor
let m = Mesh()
m.addVertices([
    [-0.01, -0.01],
    [ 0.01, -0.01],
    [ 0.00,  0.01],
])
// draw one million instances
m.setInstanceCount(1_000_000)