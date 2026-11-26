from fragmentcolor import Mesh, Model, PbrMaterial, Scene, Vertex

mesh = Mesh()
mesh.add_vertices([
    Vertex.pbr([0.0, 0.5, 0.0]).set(Vertex.UV0, [0.5, 1.0]),
    Vertex.pbr([-0.5, -0.5, 0.0]).set(Vertex.UV0, [0.0, 0.0]),
    Vertex.pbr([0.5, -0.5, 0.0]).set(Vertex.UV0, [1.0, 0.0]),
])
model = Model(mesh, PbrMaterial())

scene = Scene()
scene.add(model)

# LOD switch: hide every model the user just loaded, based on a
# camera-distance heuristic the caller computes elsewhere.
for m in scene.models():
    m.set_visible(False)