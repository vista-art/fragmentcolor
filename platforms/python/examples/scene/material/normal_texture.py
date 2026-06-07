from fragmentcolor import Material, Renderer

renderer = Renderer()
normal_map_bytes = [
    128, 128, 255, 255,
    128,   128, 255, 255,
    128,   128, 255, 255,
    128,   128, 255, 255,
]
normal_map = renderer.create_texture(normal_map_bytes, size=[2, 2])
mat = Material.pbr().normal_texture(normal_map).normal_scale(1.2)