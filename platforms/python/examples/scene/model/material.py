from fragmentcolor import Material, Mesh, Model, Vertex

mesh = Mesh()
mesh.add_vertex(
    Vertex([0.0, 0.0, 0.0]).set(Vertex.NORMAL, [0.0, 1.0, 0.0]).set(Vertex.UV0, [0.0, 0.0]),
)

model = Model(mesh, Material.pbr())
model.material().shader().set("camera.position", [0.0, 0.0, 5.0])