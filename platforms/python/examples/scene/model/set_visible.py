from fragmentcolor import Material, Mesh, Model, Vertex

mesh = Mesh()
mesh.add_vertex(Vertex.pbr([0.0, 0.5, 0.0]))
blob = Model(mesh, Material.pbr())

# At a wide zoom level, skip the detail blobs.
blob.set_visible(False)
# Zoomed back in, turn them on again.
blob.set_visible(True)