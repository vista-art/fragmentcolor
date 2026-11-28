import FragmentColor

let renderer = Renderer()
let albedo_bytes = [
    255, 200, 120, 255,
    255, 240, 180, 255,
    230, 180, 100, 255,
    255, 220, 150, 255,
]
let albedo = try await renderer.createTexture(albedo_bytes, [2, 2])

// Every material whose map points at `albedo` reuses the same uploaded GPU
// texture; passing the same handle to N maps costs one upload and N
// shader-uniform references.
let material = PbrMaterial()
let map = try material.baseColorMap().texture(albedo)

// Raw pixels with their size upload when the scene loads.
let tint = [255, 128, 0, 255]
let tinted = PbrMaterial()
let tinted_map = try tinted.baseColorMap().texture(tint, [1, 1])

// A toon material samples the tinted texture too, which each Renderer
// uploads once for both maps.
let toon = ToonMaterial()
let toon_map = try toon.baseColorMap().texture(tinted.baseColorMap())