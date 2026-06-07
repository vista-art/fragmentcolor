import FragmentColor

let mesh = Mesh()
try mesh.addVertex(Vertex.pbr([0.0, 0.5, 0.0]))
let model = Model(mesh, Material.pbr())

// Models start visible; toggle with """set_visible""".
let visible_now = model.visible()