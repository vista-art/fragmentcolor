from fragmentcolor import Material, Renderer

renderer = Renderer()
albedo_bytes = [
    255, 200, 120, 255,
    255, 240, 180, 255,
    230, 180, 100, 255,
    255, 220, 150, 255,
]
albedo = renderer.create_texture(albedo_bytes, size=[2, 2])

# Every Material that points at `albedo` reuses the same uploaded GPU
# texture; passing the same handle into N Material instances costs one
# upload and N shader-uniform references.
material = Material.pbr().base_color_texture(albedo)

# Raw pixels with their size upload when the scene loads.
tint = [255, 128, 0, 255]
tinted = Material.pbr().base_color_texture(tint, size=[1, 1])