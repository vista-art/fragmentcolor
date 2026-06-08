import FragmentColor

let mesh = Mesh()
try mesh.addVertex(
    try Vertex.pbr([0.0, 0.5, 0.0]).set("uv0", [0.5, 1.0])
)
let scene = Scene()
try scene.add(Model(mesh, Material.pbr()))

// The host overrides every uniform, so suppress FC's stock camera + light.
scene.noDefaults()
for pass in scene.listPasses() {
    pass.loadPrevious()
}