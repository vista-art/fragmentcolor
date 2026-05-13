from fragmentcolor import Material, Renderer

renderer = Renderer()
normal_map = renderer.create_texture([
    128, 128, 255, 255,
    128,   128, 255, 255,
    128,   128, 255, 255,
    128,   128, 255, 255,
][..])
mat = Material.pbr(renderer).normal_texture(normal_map).normal_scale(1.2)