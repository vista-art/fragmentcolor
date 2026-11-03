from fragmentcolor import Instance, Mesh

mesh = Mesh()
mesh.add_vertices([
    [-0.5, -0.5],
    [0.5, -0.5],
    [0.0, 0.5],
])

instance = Instance().set("offset", [0.5, 0.0])
mesh.add_instance(instance)