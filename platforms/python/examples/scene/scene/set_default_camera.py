from fragmentcolor import Camera, Material, Mesh, Model, Scene, Vertex

mesh = Mesh()
mesh.add_vertex(
    Vertex.pbr([0.0, 0.5, 0.0]).set(Vertex.UV0, [0.5, 1.0]),
)
scene = Scene()
scene.add(Model(mesh, Material.pbr()))

camera = Camera.perspective(1.047, 16.0 / 9.0, 0.1, 100.0).look_at([0.0, 1.5, 4.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])
scene.set_default_camera(camera)