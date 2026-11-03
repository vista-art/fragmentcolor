from fragmentcolor import Camera, Light, Material, Mesh, Model, Renderer, Scene, Vertex

renderer = Renderer()

mesh = Mesh()
mesh.add_vertices([
    Vertex.pbr([0.0, 0.5, 0.0]).set(Vertex.UV0, [0.5, 1.0]),
    Vertex.pbr([-0.5, -0.5, 0.0]).set(Vertex.UV0, [0.0, 0.0]),
    Vertex.pbr([0.5, -0.5, 0.0]).set(Vertex.UV0, [1.0, 0.0]),
])
model = Model(mesh, Material.pbr())

camera = Camera.perspective(1.047, 1.0, 0.1, 100.0).look_at([0.0, 0.0, 3.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])
sun = Light.directional([0.3, -1.0, -0.4], [1.0, 0.95, 0.9])

scene = Scene()
scene.add(model)
scene.add(camera)
scene.add(sun)

# Updating the camera later is enough. Every shader on the scene picks
# up the new view_proj at the next render.
camera.look_at([3.0, 1.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])