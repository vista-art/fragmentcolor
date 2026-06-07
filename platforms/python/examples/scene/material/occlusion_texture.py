from fragmentcolor import Material, Renderer

renderer = Renderer()
ao_bytes = [
    220, 0, 0, 255,
    180,   0, 0, 255,
    200,   0, 0, 255,
    160,   0, 0, 255,
]
ao = renderer.create_texture(ao_bytes, size=[2, 2])
mat = Material.pbr().occlusion_texture(ao)