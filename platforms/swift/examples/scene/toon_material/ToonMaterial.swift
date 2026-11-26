import FragmentColor

let mesh = Mesh()
mesh.addVertices([
    try Vertex.pbr([0.0, 0.5, 0.0]),
    try Vertex.pbr([-0.5, -0.5, 0.0]),
    try Vertex.pbr([0.5, -0.5, 0.0]),
])

let hero = try ToonMaterial().baseColor([0.9, 0.3, 0.2, 1.0]).bands(2).highlight([1.0, 1.0, 1.0]).rim([0.3, 0.5, 1.0])
let model = Model(mesh, hero)