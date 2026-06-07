import FragmentColor

let mesh = Mesh()
try mesh.addVertex(
    try Vertex.pbr([0.0, 0.5, 0.0]).set("uv0", [0.5, 1.0])
)

let material = try Material.pbr().baseColor([0.85, 0.2, 0.2, 1.0]).metallic(0.0).roughness(0.4).emissive([0.0, 0.0, 0.05])

let model = Model(mesh, material)