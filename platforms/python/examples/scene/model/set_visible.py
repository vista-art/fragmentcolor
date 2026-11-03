from fragmentcolor import Material, Mesh, Model, Vertex

mesh = Mesh()
mesh.add_vertices([
    Vertex.pbr([0.0, 0.5, 0.0]),
    Vertex.pbr([-0.5, -0.5, 0.0]),
    Vertex.pbr([0.5, -0.5, 0.0]),
])
detail = Model(mesh, Material.pbr())

# Far from the camera, hide the detail Model.
detail.set_visible(False)
# Closer again, show it.
detail.set_visible(True)