import FragmentColor

// Raw pixels with their size upload when the scene loads.
let ao_bytes = [
    220, 0, 0, 255,
    180, 0, 0, 255,
    200, 0, 0, 255,
    160, 0, 0, 255,
]
let mat = try Material.pbr().occlusionTexture(ao_bytes, [2, 2])