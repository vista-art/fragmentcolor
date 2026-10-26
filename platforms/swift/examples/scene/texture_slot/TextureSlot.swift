import FragmentColor

let material = try Material.pbr().textureUv(TextureSlot.occlusion, 1).textureTransform(TextureSlot.normal, [0.0, 0.0], [4.0, 4.0], 0.0)