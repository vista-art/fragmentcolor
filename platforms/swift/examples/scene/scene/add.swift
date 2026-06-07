import FragmentColor

let renderer = Renderer()

let mesh = Mesh()
try mesh.addVertex(
    try Vertex.pbr([0.0, 0.5, 0.0]).set("uv0", [0.5, 1.0])
)
let model = Model(mesh, Material.pbr())

let camera = try Camera.perspective(1.047, 1.0, 0.1, 100.0).lookAt([0.0, 0.0, 3.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])
let sun = try Light.directional([0.3, -1.0, -0.4], [1.0, 0.95, 0.9])

let scene = Scene()
try scene.add(model)
try scene.add(camera)
try scene.add(sun)

// Updating the camera later is enough — every shader on the scene picks
// the view_proj up at the next render.
try camera.lookAt([3.0, 1.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])