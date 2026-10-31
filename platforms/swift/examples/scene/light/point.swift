import FragmentColor

let renderer = Renderer()
let target = try await renderer.createTextureTarget([64, 64])

let wall = Mesh()
wall.addVertices([
    try Vertex.pbr([-2.0, -2.0, 0.0]),
    try Vertex.pbr([2.0, -2.0, 0.0]),
    try Vertex.pbr([2.0, 2.0, 0.0]),
    try Vertex.pbr([-2.0, 2.0, 0.0]),
])
wall.setIndices([0, 1, 2, 0, 2, 3])

let camera = try Camera.perspective(1.047, 1.0, 0.1, 100.0).lookAt([0.0, 0.0, 4.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])
let bulb = try Light.point([0.0, 0.5, 1.0], [1.0, 0.95, 0.8]).setIntensity(15.0)
try bulb.setRange(3.0)

let pass = Pass("lamp")
try pass.add(Model(wall, Material.pbr()))
try pass.add(camera)
try pass.add(bulb)
try renderer.render(pass, target)