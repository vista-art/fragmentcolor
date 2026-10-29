import FragmentColor

let renderer = Renderer()

let mesh = Mesh()
try mesh.addVertices([
    try Vertex.pbr([0.0, 0.5, 0.0]).set(Vertex.UV0, [0.5, 1.0]),
    try Vertex.pbr([-0.5, -0.5, 0.0]).set(Vertex.UV0, [0.0, 0.0]),
    try Vertex.pbr([0.5, -0.5, 0.0]).set(Vertex.UV0, [1.0, 0.0]),
])
let model = Model(mesh, Material.pbr())

let camera = try Camera.perspective(1.047, 1.0, 0.1, 100.0).lookAt([0.0, 0.0, 2.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])
let sun = try Light.directional([0.3, -1.0, -0.4], [1.0, 0.95, 0.9])

let pass = Pass("scene")
try pass.add(model)
try pass.add(camera)
try pass.add(sun)

// Updating the camera later is enough. Every Model already on the pass
// picks up the new view_proj at the next render.
try camera.lookAt([3.0, 1.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])