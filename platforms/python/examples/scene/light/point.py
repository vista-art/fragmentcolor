from fragmentcolor import Camera, Light, Material, Mesh, Model, Pass, Renderer, Vertex

renderer = Renderer()
target = renderer.create_texture_target([64, 64])

wall = Mesh()
wall.add_vertices([
    Vertex.pbr([-2.0, -2.0, 0.0]),
    Vertex.pbr([2.0, -2.0, 0.0]),
    Vertex.pbr([2.0, 2.0, 0.0]),
    Vertex.pbr([-2.0, 2.0, 0.0]),
])
wall.set_indices([0, 1, 2, 0, 2, 3])

camera = Camera.perspective(1.047, 1.0, 0.1, 100.0).look_at([0.0, 0.0, 4.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])
bulb = Light.point([0.0, 0.5, 1.0], [1.0, 0.95, 0.8]).set_intensity(15.0)
bulb.set_range(3.0)

rpass = Pass("lamp")
rpass.add(Model(wall, Material.pbr()))
rpass.add(camera)
rpass.add(bulb)
renderer.render(rpass, target)
