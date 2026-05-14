import FragmentColor

let renderer = Renderer()

let mesh = Mesh()
try mesh.addVertex(
    try Vertex.pbr([0.0, 0.5, 0.0]).set(Vertex.uV0, [0.5, 1.0]),
)
let model = Model(mesh, Material.pbr()?)

let camera = Camera.perspective(60.0.toRadians(), 1.0, 0.1, 100.0).lookAt([0.0, 0.0, 2.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])
let sun = Light.directional([0.3, -1.0, -0.4], [1.0, 0.95, 0.9])

let pass = Pass("scene")
pass.add(model)?.add(camera)?.add(sun)

// Updating the camera later is enough — every Model already on the pass
// picks the view_proj up at the next render.
camera.lookAt([3.0, 1.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])