import FragmentColor

let renderer = Renderer()
let albedo_bytes = [
    255, 200, 120, 255,
    255, 240, 180, 255,
    230, 180, 100, 255,
    255, 220, 150, 255,
]
let albedo = try await renderer.createTexture(albedo_bytes, [2, 2])

// Every Material that points at `albedo` reuses the same uploaded GPU
// texture; passing the same handle into N Material instances costs one
// upload and N shader-uniform references.
let material = Material.pbr().baseColorTexture(albedo)

// Raw pixels with their size upload when the scene loads.
let tint = [255, 128, 0, 255]
let tinted = try Material.pbr().baseColorTexture(tint, [1, 1])