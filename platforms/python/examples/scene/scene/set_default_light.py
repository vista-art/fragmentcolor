from fragmentcolor import Light, Material, Mesh, Model, Scene, Vertex

mesh = Mesh()
mesh.add_vertex(
    Vertex.pbr([0.0, 0.5, 0.0]).set(Vertex.UV0, [0.5, 1.0]),
)
scene = Scene()
scene.add(Model(mesh, Material.pbr()))

key = Light.directional([0.3, -1.0, -0.4], [1.0, 0.95, 0.9])
scene.set_default_light(key)