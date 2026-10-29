import FragmentColor

// Raw pixels with their size upload when the scene loads.
let glow_bytes = [
    255, 0, 0, 255,
    255, 0, 0, 255,
    255, 0, 0, 255,
    255, 0, 0, 255,
]
let mat = try Material.pbr().emissive([0.8, 0.0, 0.0]).emissiveTexture(glow_bytes, [2, 2])