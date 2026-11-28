from fragmentcolor import PbrMaterial

# Raw pixels with their size upload when the scene loads.
ao_bytes = [
    220, 0, 0, 255,
    180, 0, 0, 255,
    200, 0, 0, 255,
    160, 0, 0, 255,
]
material = PbrMaterial()
map = material.occlusion_map().texture(ao_bytes, size=[2, 2])