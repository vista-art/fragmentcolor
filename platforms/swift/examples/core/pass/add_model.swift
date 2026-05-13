import FragmentColor

let mesh = Mesh()
try mesh.addVertex(
    try Vertex([0.0, 0.5, 0.0]).set(Vertex.nORMAL, [0.0, 0.0, 1.0]).set(Vertex.uV0, [0.5, 1.0]),
)
try mesh.addVertex(
    try Vertex([-0.5, -0.5, 0.0]).set(Vertex.nORMAL, [0.0, 0.0, 1.0]).set(Vertex.uV0, [0.0, 0.0]),
)
try mesh.addVertex(
    try Vertex([0.5, -0.5, 0.0]).set(Vertex.nORMAL, [0.0, 0.0, 1.0]).set(Vertex.uV0, [1.0, 0.0]),
)

let template = Material.pbr()?.baseColor([0.85, 0.4, 0.2, 1.0])
let pass = Pass("scene")

let m1 = Model(mesh.clone(), template.clone())
m1.translate([-1.0, 0.0, 0.0])
pass.addModel(m1)

let m2 = Model(mesh, template)
m2.translate([1.0, 0.0, 0.0])
pass.addModel(m2)