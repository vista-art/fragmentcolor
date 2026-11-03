from fragmentcolor import Material

# Raw pixels with their size upload when the scene loads.
ao_bytes = [
    220, 0, 0, 255,
    180, 0, 0, 255,
    200, 0, 0, 255,
    160, 0, 0, 255,
]
mat = Material.pbr().occlusion_texture(ao_bytes, size=[2, 2])