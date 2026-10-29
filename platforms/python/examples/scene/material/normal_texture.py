from fragmentcolor import Material

# Raw pixels with their size upload when the scene loads.
normal_map_bytes = [
    128, 128, 255, 255,
    128, 128, 255, 255,
    128, 128, 255, 255,
    128, 128, 255, 255,
]
mat = Material.pbr().normal_texture(normal_map_bytes, size=[2, 2]).normal_scale(1.2)