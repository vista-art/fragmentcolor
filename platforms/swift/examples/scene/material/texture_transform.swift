import FragmentColor

// Every map spans the surface twice, and the normal map eight times.
let tiles = try Material.pbr().uvTransform([0.0, 0.0], [2.0, 2.0], 0.0).textureTransform(TextureSlot.normal, [0.0, 0.0], [8.0, 8.0], 0.0)