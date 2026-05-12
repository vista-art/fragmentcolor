from fragmentcolor import Material, Mesh, Model, Pass, Vertex

mesh = Mesh()
mesh.add_vertex(
    Vertex([0.0, 0.5, 0.0]).set(Vertex.NORMAL, [0.0, 0.0, 1.0]).set(Vertex.UV0, [0.5, 1.0]),
)
mesh.add_vertex(
    Vertex([-0.5, -0.5, 0.0]).set(Vertex.NORMAL, [0.0, 0.0, 1.0]).set(Vertex.UV0, [0.0, 0.0]),
)
mesh.add_vertex(
    Vertex([0.5, -0.5, 0.0]).set(Vertex.NORMAL, [0.0, 0.0, 1.0]).set(Vertex.UV0, [1.0, 0.0]),
)

template = Material.pbr().base_color([0.85, 0.4, 0.2, 1.0])
rpass = Pass("scene")

m1 = Model(mesh.clone(), template.clone())
m1.translate([-1.0, 0.0, 0.0])
rpass.add_model(m1)

m2 = Model(mesh, template)
m2.translate([1.0, 0.0, 0.0])
rpass.add_model(m2)
