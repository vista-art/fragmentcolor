from fragmentcolor import Material, Renderer

renderer = Renderer()
glow_bytes = [
    255, 0, 0, 255,
    255,   0, 0, 255,
    255,   0, 0, 255,
    255,   0, 0, 255,
]
glow = renderer.create_texture(glow_bytes, size=[2, 2])
mat = Material.pbr().emissive([0.8, 0.0, 0.0]).emissive_texture(glow)