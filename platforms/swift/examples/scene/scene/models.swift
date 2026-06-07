import FragmentColor

let mesh = Mesh()
try mesh.addVertex(
    try Vertex.pbr([0.0, 0.5, 0.0]).set("uv0", [0.5, 1.0])
)
let model = Model(mesh, Material.pbr())

let scene = Scene()
try scene.add(model)

// LOD switch: hide every model the user just loaded, based on a
// camera-distance heuristic the caller computes elsewhere.
for m in scene.models() {
    m.setVisible(false)
}