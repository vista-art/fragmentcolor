from fragmentcolor import Material, Mesh, Model, Scene, Vertex

mesh = Mesh()
mesh.add_vertex(
    Vertex.pbr([0.0, 0.5, 0.0]).set(Vertex.UV0, [0.5, 1.0]),
)
scene = Scene()
scene.add(Model(mesh, Material.pbr()))

# Compose, don't clear: keep whatever the previous pass drew.
for p in scene.list_passes():
    p.load_previous()