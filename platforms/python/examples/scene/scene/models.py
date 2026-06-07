from fragmentcolor import Material, Mesh, Model, Scene, Vertex

mesh = Mesh()
mesh.add_vertex(
    Vertex.pbr([0.0, 0.5, 0.0]).set(Vertex.UV0, [0.5, 1.0]),
)
model = Model(mesh, Material.pbr())

scene = Scene()
scene.add(model)

# LOD switch: hide every model the user just loaded, based on a
# camera-distance heuristic the caller computes elsewhere.
for m in scene.models():
    m.set_visible(False)