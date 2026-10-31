import FragmentColor

let mesh = Mesh()
try mesh.addVertices([
    [-0.5, -0.5],
    [0.5, -0.5],
    [0.0, 0.5],
])

let instance = try Instance().set("offset", [0.5, 0.0])
mesh.addInstance(instance)