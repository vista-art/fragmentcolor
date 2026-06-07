import FragmentColor

let renderer = Renderer()

let mesh = Mesh()
try mesh.addVertex(
    try Vertex.pbr([0.0, 0.5, 0.0]).set("uv0", [0.5, 1.0])
)
let model = Model(mesh, Material.pbr())

// A backdrop pass that clears to a soft blue before the scene's main draw.
let backdrop = Pass("backdrop")
try backdrop.setClearColor([0.05, 0.08, 0.12, 1.0])

let scene = Scene()
scene.addPass(backdrop)
try scene.add(model)