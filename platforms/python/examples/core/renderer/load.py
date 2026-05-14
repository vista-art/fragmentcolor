from fragmentcolor import Material, Mesh, Model, Renderer, Scene, Vertex

renderer = Renderer()
target = renderer.create_texture_target([64, 64])

mesh = Mesh()
mesh.add_vertex(
    Vertex([0.0, 0.5, 0.0]).set(Vertex.NORMAL, [0.0, 0.0, 1.0]).set(Vertex.UV0, [0.5, 1.0]).set(Vertex.COLOR0, [1.0, 1.0, 1.0, 1.0]).set(Vertex.UV1, [0.0, 0.0]),
)
# Raw 2×2 RGBA pixel bytes — uploaded lazily by `Renderer.load` below.
# In practice the loader hands the setter encoded PNG/JPEG bytes (from a
# BIN chunk) or a file path (from a URI); the same `Into<TextureInput>`
# vocabulary covers all of them.
red_pixels = [
    255,   0,   0, 255,    0, 255,   0, 255,
      0,   0, 255, 255,  255, 255, 255, 255,
]
material = Material.pbr().base_color_texture((red_pixels, [2, 2]))
model = Model(mesh, material)
scene = Scene()
scene.add(model)

# Eager prewarm — uploads the pending texture(s) so the next render is
# GPU-only.
renderer.load(scene)
renderer.render(scene, target)