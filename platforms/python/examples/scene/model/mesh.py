from fragmentcolor import Material, Mesh, Model, Renderer, Vertex

renderer = Renderer()
mesh = Mesh()
mesh.add_vertex(
    Vertex([0.0, 0.5, 0.0]).set(Vertex.NORMAL, [0.0, 0.0, 1.0]).set(Vertex.UV0, [0.5, 1.0]),
)

model = Model(mesh, Material.pbr())
model.mesh().add_vertex(
    Vertex([-0.5, -0.5, 0.0]).set(Vertex.NORMAL, [0.0, 0.0, 1.0]).set(Vertex.UV0, [0.0, 0.0]),
)