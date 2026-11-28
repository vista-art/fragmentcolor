from fragmentcolor import PbrMaterial

# Raw pixels with their size upload when the scene loads.
glow_bytes = [
    255, 0, 0, 255,
    255, 0, 0, 255,
    255, 0, 0, 255,
    255, 0, 0, 255,
]
material = PbrMaterial().emissive([0.8, 0.0, 0.0])
map = material.emissive_map().texture(glow_bytes, size=[2, 2])