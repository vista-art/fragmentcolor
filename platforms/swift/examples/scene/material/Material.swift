import FragmentColor

let renderer = Renderer()
let mesh = Mesh()
try mesh.addVertex(
    try Vertex([0.0, 0.5, 0.0]).set(Vertex.nORMAL, [0.0, 0.0, 1.0]).set(Vertex.uV0, [0.5, 1.0]),
)

let material = try await Material.pbr(renderer).baseColor([0.85, 0.2, 0.2, 1.0]).metallic(0.0).roughness(0.4).emissive([0.0, 0.0, 0.05])

let model = Model(mesh, material)