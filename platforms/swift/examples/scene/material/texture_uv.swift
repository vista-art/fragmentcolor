import FragmentColor

let mesh = Mesh()
try mesh.addVertex(
    try Vertex.pbr([0.0, 0.5, 0.0]).set(Vertex.UV0, [0.5, 1.0]).set(Vertex.UV1, [0.25, 0.75])
)

// The occlusion map follows the second UV layout.
let material = try Material.pbr().textureUv(TextureSlot.occlusion, 1)