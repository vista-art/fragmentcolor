import FragmentColor

// Raw pixels with their size upload when the scene loads.
let pixels = [255, 128, 0, 255]
let material = UnlitMaterial()
let map = try material.baseColorMap().texture(pixels, [1, 1])