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
blob = Material.pbr().base_color_texture(albedo)