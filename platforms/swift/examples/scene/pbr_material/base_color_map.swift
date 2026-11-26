import FragmentColor

// Raw pixels with their size upload when the scene loads.
let tint = [255, 128, 0, 255]
let material = PbrMaterial()
let map = try material.baseColorMap().texture(tint, [1, 1])