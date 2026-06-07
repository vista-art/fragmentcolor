from fragmentcolor import Material, Mesh, Model, Pass, Renderer, Scene, Vertex

renderer = Renderer()

mesh = Mesh()
mesh.add_vertex(
    Vertex.pbr([0.0, 0.5, 0.0]).set(Vertex.UV0, [0.5, 1.0]),
)
model = Model(mesh, Material.pbr())

# A backdrop pass that clears to a soft blue before the scene's main draw.
backdrop = Pass("backdrop")
backdrop.set_clear_color([0.05, 0.08, 0.12, 1.0])

scene = Scene()
scene.add_pass(backdrop)
scene.add(model)
