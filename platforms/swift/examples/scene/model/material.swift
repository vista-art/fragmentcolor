import FragmentColor

let mesh = Mesh()
try mesh.addVertex(
    try Vertex([0.0, 0.0, 0.0]).set("normal", [0.0, 1.0, 0.0]).set("uv0", [0.0, 0.0])
)

let model = Model(mesh, Material.pbr())
try model.material().shader().set("camera.position", [0.0, 0.0, 5.0])