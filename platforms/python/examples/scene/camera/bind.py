from fragmentcolor import Camera, Material, Renderer

camera = Camera.perspective(60.0_.to_radians(), 16.0 / 9.0, 0.1, 100.0).look_at([0.0, 1.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])

renderer = Renderer()
material = Material.pbr(renderer)
camera.bind(material.shader())
