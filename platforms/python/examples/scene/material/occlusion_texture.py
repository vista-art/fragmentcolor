from fragmentcolor import Material, Renderer

renderer = Renderer()
ao = renderer.create_texture([
    220, 0, 0, 255,
    180,   0, 0, 255,
    200,   0, 0, 255,
    160,   0, 0, 255,
][..])
mat = Material.pbr(renderer).occlusion_texture(ao)