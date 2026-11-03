from fragmentcolor import Material, TextureSlot

material = Material.pbr().texture_uv(TextureSlot.Occlusion, 1).texture_transform(TextureSlot.Normal, [0.0, 0.0], [4.0, 4.0], 0.0)