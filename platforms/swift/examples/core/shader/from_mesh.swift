import FragmentColor

let mesh = Mesh()
try mesh.addVertex([0.0, 0.0, 0.0])
let shader = try Shader.fromMesh(mesh)