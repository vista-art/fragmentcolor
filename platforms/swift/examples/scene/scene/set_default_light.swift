import FragmentColor

let mesh = Mesh()
try mesh.addVertex(
    try Vertex.pbr([0.0, 0.5, 0.0]).set("uv0", [0.5, 1.0])
)
let scene = Scene()
try scene.add(Model(mesh, Material.pbr()))

let key = try Light.directional([0.3, -1.0, -0.4], [1.0, 0.95, 0.9])
scene.setDefaultLight(key)