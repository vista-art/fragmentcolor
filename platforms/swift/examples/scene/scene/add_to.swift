import FragmentColor

let mesh = Mesh()
try mesh.addVertex(
    try Vertex.pbr([0.0, 0.5, 0.0]).set("uv0", [0.5, 1.0])
)
let model = Model(mesh, Material.pbr())

let scene = Scene()
scene.addPass(Pass("geometry"))

// Target the pass by name (or pass its index: scene.addTo(0, model)).
try scene.addTo("geometry", model)