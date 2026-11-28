import FragmentColor

// Deep blue shadows, a warm middle tone and full light, one texel each.
let ramp = [
    20, 30, 90, 255, 20, 30, 90, 255,
    200, 140, 110, 255, 255, 255, 255, 255,
]
let material = ToonMaterial()
let map = try material.rampMap().texture(ramp, [4, 1])