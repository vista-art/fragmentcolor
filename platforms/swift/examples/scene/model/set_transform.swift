import FragmentColor

let renderer = Renderer()
let mesh = Mesh()
try mesh.addVertex(
    try Vertex([0.0, 0.0, 0.0]).set("normal", [0.0, 1.0, 0.0]).set("uv0", [0.0, 0.0])
)

let model = Model(mesh, Material.pbr())
try model.setTransform([
    [2.0, 0.0, 0.0, 0.0],
    [0.0, 2.0, 0.0, 0.0],
    [0.0, 0.0, 2.0, 0.0],
    [3.0, 0.0, 0.0, 1.0],
])