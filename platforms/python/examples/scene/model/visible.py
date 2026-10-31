from fragmentcolor import Material, Mesh, Model, Vertex

mesh = Mesh()
mesh.add_vertices([
    Vertex.pbr([0.0, 0.5, 0.0]),
    Vertex.pbr([-0.5, -0.5, 0.0]),
    Vertex.pbr([0.5, -0.5, 0.0]),
])
model = Model(mesh, Material.pbr())

# Models start visible; toggle with `set_visible`.
visible_now = model.visible()