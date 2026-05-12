from fragmentcolor import Material, Mesh, Model, Vertex

mesh = Mesh()
mesh.add_vertex(
    Vertex([0.0, 0.0, 0.0]).set(Vertex.NORMAL, [0.0, 1.0, 0.0]).set(Vertex.UV0, [0.0, 0.0]),
)

model = Model(mesh, Material.pbr())
model.rotate([0.0, 1.0, 0.0], std.f32.consts.FRAC_PI_2)