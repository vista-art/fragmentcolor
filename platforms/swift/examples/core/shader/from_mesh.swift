import FragmentColor

let mesh = Mesh()
try mesh.addVertices([
    [-0.5, -0.5, 0.0],
    [0.5, -0.5, 0.0],
    [0.0, 0.5, 0.0],
])
let shader = try Shader.fromMesh(mesh)