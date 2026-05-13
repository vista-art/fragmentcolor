from fragmentcolor import Material, Renderer

renderer = Renderer()
texture = renderer.create_texture([
    255, 200, 120, 255,
    255,  240, 180, 255,
    230,  180, 100, 255,
    255,  220, 150, 255,
][..])
mat = Material.pbr(renderer).base_color_texture(texture)