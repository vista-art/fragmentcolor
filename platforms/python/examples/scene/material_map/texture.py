from fragmentcolor import PbrMaterial, Renderer

renderer = Renderer()
albedo_bytes = [
    255, 200, 120, 255,
    255, 240, 180, 255,
    230, 180, 100, 255,
    255, 220, 150, 255,
]
albedo = renderer.create_texture(albedo_bytes, size=[2, 2])

# Every material whose map points at `albedo` reuses the same uploaded GPU
# texture; passing the same handle to N maps costs one upload and N
# shader-uniform references.
material = PbrMaterial()
map = material.base_color_map().texture(albedo)

# Raw pixels with their size upload when the scene loads.
tint = [255, 128, 0, 255]
tinted = PbrMaterial()
tinted_map = tinted.base_color_map().texture((tint, [1, 1]))