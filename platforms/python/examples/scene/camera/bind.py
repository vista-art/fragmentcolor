from fragmentcolor import Camera, Material

camera = Camera.perspective(60.0_.to_radians(), 16.0 / 9.0, 0.1, 100.0).look_at([0.0, 1.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])

material = Material.pbr()
camera.bind(material.shader())
