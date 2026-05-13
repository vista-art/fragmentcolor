import FragmentColor

let renderer = Renderer()
let mesh = Mesh()
try mesh.addVertex(
    try Vertex([0.0, 0.0, 0.0]).set(Vertex.nORMAL, [0.0, 1.0, 0.0]).set(Vertex.uV0, [0.0, 0.0]),
)

let model = try await Model(mesh, Material.pbr(renderer))
try model.material().shader().set("camera.position", [0.0, 0.0, 5.0])