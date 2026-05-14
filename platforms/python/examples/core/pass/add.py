from fragmentcolor import Camera, Light, Material, Mesh, Model, Pass, Renderer, Vertex

renderer = Renderer()

mesh = Mesh()
mesh.add_vertex(
    Vertex.pbr([0.0, 0.5, 0.0]).set(Vertex.UV0, [0.5, 1.0]),
)
model = Model(mesh, Material.pbr())

camera = Camera.perspective(60.0_.to_radians(), 1.0, 0.1, 100.0).look_at([0.0, 0.0, 2.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])
sun = Light.directional([0.3, -1.0, -0.4], [1.0, 0.95, 0.9])

rpass = Pass("scene")
rpass.add(model).add(camera).add(sun)

# Updating the camera later is enough — every Model already on the rpass
# picks the new view_proj up at the next render.
camera.look_at([3.0, 1.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])