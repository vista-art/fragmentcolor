import FragmentColor

let mesh = Mesh()
try mesh.addVertex(
    try Vertex.pbr([0.0, 0.5, 0.0]).set("uv0", [0.5, 1.0])
)
let scene = Scene()
try scene.add(Model(mesh, Material.pbr()))

let camera = try Camera.perspective(1.047, 16.0 / 9.0, 0.1, 100.0).lookAt([0.0, 1.5, 4.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])
scene.setDefaultCamera(camera)