from fragmentcolor import Material

# Raw pixels with their size upload when the scene loads.
mr_map_bytes = [
    0, 200, 50, 255,
    0, 240, 80, 255,
    0, 180, 30, 255,
    0, 220, 60, 255,
]
mat = Material.pbr().metallic_roughness_texture(mr_map_bytes, size=[2, 2])