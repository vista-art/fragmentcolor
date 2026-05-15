from fragmentcolor import Material, Renderer

renderer = Renderer()
albedo = renderer.create_texture([
    255, 200, 120, 255,
    255,  240, 180, 255,
    230,  180, 100, 255,
    255,  220, 150, 255,
][..])

# 279 blob Materials all sample the same uploaded `albedo` — one GPU
# texture, 279 shader references.
blob_materials = (0..279).map(|_| Material.pbr().base_color_texture(albedo)).collect()