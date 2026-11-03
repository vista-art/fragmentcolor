from fragmentcolor import Material, TextureSlot

# Every map spans the surface twice, and the normal map eight times.
tiles = Material.pbr().uv_transform([0.0, 0.0], [2.0, 2.0], 0.0).texture_transform(TextureSlot.Normal, [0.0, 0.0], [8.0, 8.0], 0.0)