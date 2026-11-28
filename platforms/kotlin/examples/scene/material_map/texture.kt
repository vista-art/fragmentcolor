import org.fragmentcolor.*

val renderer = Renderer()
val albedo_bytes = listOf(255, 200, 120, 255, 255, 240, 180, 255, 230, 180, 100, 255, 255, 220, 150, 255)
val albedo = renderer.createTexture(albedo_bytes, listOf(2u, 2u))

// Every material whose map points at `albedo` reuses the same uploaded GPU
// texture; passing the same handle to N maps costs one upload and N
// shader-uniform references.
val material = PbrMaterial()
val map = material.baseColorMap().texture(albedo)

// Raw pixels with their size upload when the scene loads.
val tint = listOf(255, 128, 0, 255)
val tinted = PbrMaterial()
val tinted_map = tinted.baseColorMap().texture(tint, listOf(1u, 1u))

// A toon material samples the tinted texture too, which each Renderer
// uploads once for both maps.
val toon = ToonMaterial()
val toon_map = toon.baseColorMap().texture(tinted.baseColorMap())