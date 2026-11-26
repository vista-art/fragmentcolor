from fragmentcolor import PbrMaterial

# Raw pixels with their size upload when the scene loads.
normal_map_bytes = [
    128, 128, 255, 255,
    128, 128, 255, 255,
    128, 128, 255, 255,
    128, 128, 255, 255,
]
material = PbrMaterial().normal_scale(1.2)
map = material.normal_map().texture((normal_map_bytes, [2, 2]))