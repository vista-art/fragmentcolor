from fragmentcolor import Material

# Raw pixels with their size upload when the scene loads.
glow_bytes = [
    255, 0, 0, 255,
    255, 0, 0, 255,
    255, 0, 0, 255,
    255, 0, 0, 255,
]
mat = Material.pbr().emissive([0.8, 0.0, 0.0]).emissive_texture(glow_bytes, size=[2, 2])