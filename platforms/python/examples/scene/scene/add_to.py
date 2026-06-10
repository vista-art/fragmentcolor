from fragmentcolor import Material, Mesh, Model, Pass, Scene, Vertex

mesh = Mesh()
mesh.add_vertex(
    Vertex.pbr([0.0, 0.5, 0.0]).set(Vertex.UV0, [0.5, 1.0]),
)
model = Model(mesh, Material.pbr())

scene = Scene()
scene.add_pass(Pass("geometry"))

# Target the pass by name (or pass its index: scene.add_to(0, model)).
scene.add_to("geometry", model)