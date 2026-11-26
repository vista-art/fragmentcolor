from fragmentcolor import Mesh, Model, PbrMaterial, Vertex

mesh = Mesh()
mesh.add_vertices([
    Vertex.pbr([0.0, 0.5, 0.0]),
    Vertex.pbr([-0.5, -0.5, 0.0]),
    Vertex.pbr([0.5, -0.5, 0.0]),
])
detail = Model(mesh, PbrMaterial())

# Far from the camera, hide the detail Model.
detail.set_visible(False)
# Closer again, show it.
detail.set_visible(True)