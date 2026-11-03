import FragmentColor

// Raw pixels with their size upload when the scene loads.
let normal_map_bytes = [
    128, 128, 255, 255,
    128, 128, 255, 255,
    128, 128, 255, 255,
    128, 128, 255, 255,
]
let mat = try Material.pbr().normalTexture(normal_map_bytes, [2, 2]).normalScale(1.2)