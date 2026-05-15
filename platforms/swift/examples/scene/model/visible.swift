import FragmentColor

let mesh = Mesh()
try mesh.addVertex(Vertex.pbr([0.0, 0.5, 0.0]))
let model = Model(mesh, Material.pbr()?)
let _ = model.visible()