import FragmentColor

// Raw pixels with their size upload when the scene loads.
let glow_bytes = [
    255, 0, 0, 255,
    255, 0, 0, 255,
    255, 0, 0, 255,
    255, 0, 0, 255,
]
let material = try PbrMaterial().emissive([0.8, 0.0, 0.0])
let map = try material.emissiveMap().texture(glow_bytes, [2, 2])