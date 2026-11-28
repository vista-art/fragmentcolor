import FragmentColor

// Raw pixels with their size upload when the scene loads.
let normal_map_bytes = [
    128, 128, 255, 255,
    128, 128, 255, 255,
    128, 128, 255, 255,
    128, 128, 255, 255,
]
let material = try PbrMaterial().normalScale(1.2)
let map = try material.normalMap().texture(normal_map_bytes, [2, 2])