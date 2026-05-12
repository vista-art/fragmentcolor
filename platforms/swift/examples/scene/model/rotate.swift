import FragmentColor

let mesh = Mesh()
try mesh.addVertex(
    try Vertex([0.0, 0.0, 0.0]).set(Vertex.nORMAL, [0.0, 1.0, 0.0]).set(Vertex.uV0, [0.0, 0.0]),
)

let model = Model(mesh, Material.pbr())
model.rotate([0.0, 1.0, 0.0], std.f32.consts.fRACPI2)