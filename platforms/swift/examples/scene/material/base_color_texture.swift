import FragmentColor

let renderer = Renderer()
let albedo = renderer.createTexture([
    255, 200, 120, 255,
    255,  240, 180, 255,
    230,  180, 100, 255,
    255,  220, 150, 255,
try await ])

// 279 blob Materials all sample the same uploaded """albedo""" — one GPU
// texture, 279 shader references.
let blob_materials = Vec.withCapacity(279)
for _ in 0...279 {
    blob_materials.push(Material.pbr()?.baseColorTexture(albedo))
}