import FragmentColor

let renderer = Renderer()
let target = try await renderer.createTextureTarget([64, 64])

let mesh = Mesh()
try mesh.addVertex(
    try Vertex([0.0, 0.5, 0.0]).set(Vertex.nORMAL, [0.0, 0.0, 1.0]).set(Vertex.uV0, [0.5, 1.0]),
)

let scene = Scene()
// Warm dusk ambient — applies to every Material added below.
scene.ambient([0.06, 0.04, 0.03])
scene.add(Model.new(mesh, Material.pbr()?))
scene.add(Light.directional([0.3, -1.0, -0.4], [1.0, 0.95, 0.9]))

try renderer.render(scene, target)