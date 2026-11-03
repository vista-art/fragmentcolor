from fragmentcolor import Material, Mesh, Model, Renderer, Scene, Vertex
image = open("examples/assets/logo.png", "rb").read()

renderer = Renderer()
target = renderer.create_texture_target([64, 64])

mesh = Mesh()
mesh.add_vertices([
    Vertex.pbr([0.0, 0.5, 0.0]).set(Vertex.UV0, [0.5, 1.0]),
    Vertex.pbr([-0.5, -0.5, 0.0]).set(Vertex.UV0, [0.0, 0.0]),
    Vertex.pbr([0.5, -0.5, 0.0]).set(Vertex.UV0, [1.0, 0.0]),
])

# The material keeps the encoded image as a map.
material = Material.pbr().base_color_texture(image)
model = Model(mesh, material)
scene = Scene()
scene.add(model)

# Decode and upload it now, so the first render only draws.
renderer.load(scene)
renderer.render(scene, target)

# A second Renderer draws the same map from a copy of its own.
other = Renderer()
frame = other.create_texture_target([64, 64])
other.load(scene)
other.render(scene, frame)