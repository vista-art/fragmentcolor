from fragmentcolor import Camera, Light, Material, Renderer

renderer = Renderer()
material = Material.pbr(renderer)

camera = Camera.perspective(60.0_.to_radians(), 16.0 / 9.0, 0.1, 100.0).look_at([0.0, 1.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])
sun = Light.directional([0.3, -1.0, -0.4], [1.0, 0.95, 0.9])

material.add(camera).add(sun)

# Updating the camera later is enough — the Material picks the new
# view_proj up at the next render without re-adding.
camera.look_at([3.0, 1.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])