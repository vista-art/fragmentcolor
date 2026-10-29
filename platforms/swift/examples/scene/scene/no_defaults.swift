import FragmentColor

let mesh = Mesh()
try mesh.addVertices([
    try Vertex.pbr([0.0, 0.5, 0.0]).set(Vertex.UV0, [0.5, 1.0]),
    try Vertex.pbr([-0.5, -0.5, 0.0]).set(Vertex.UV0, [0.0, 0.0]),
    try Vertex.pbr([0.5, -0.5, 0.0]).set(Vertex.UV0, [1.0, 0.0]),
])
let scene = Scene()
try scene.add(Model(mesh, Material.pbr()))

// The host overrides every uniform, so suppress FC's stock camera + light.
scene.noDefaults()
for p in scene.listPasses() {
    p.loadPrevious()
}