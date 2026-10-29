import FragmentColor

let mesh = Mesh()
try mesh.addVertices([
    try Vertex.pbr([0.0, 0.5, 0.0]).set(Vertex.UV0, [0.5, 1.0]),
    try Vertex.pbr([-0.5, -0.5, 0.0]).set(Vertex.UV0, [0.0, 0.0]),
    try Vertex.pbr([0.5, -0.5, 0.0]).set(Vertex.UV0, [1.0, 0.0]),
])

let model = Model(mesh, Material.pbr())
try model.material().shader().set("camera.position", [0.0, 0.0, 5.0])