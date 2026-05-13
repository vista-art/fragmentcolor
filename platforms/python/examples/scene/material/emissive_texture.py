from fragmentcolor import Material, Renderer

renderer = Renderer()
glow = renderer.create_texture([
    255, 0, 0, 255,
    255,   0, 0, 255,
    255,   0, 0, 255,
    255,   0, 0, 255,
][..])
mat = Material.pbr(renderer).emissive([0.8, 0.0, 0.0]).emissive_texture(glow)