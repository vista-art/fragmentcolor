import org.fragmentcolor.*

val renderer = Renderer()
val albedo_bytes = listOf(255, 200, 120, 255, 255, 240, 180, 255, 230, 180, 100, 255, 255, 220, 150, 255)
val albedo = renderer.createTexture(albedo_bytes, listOf(2u, 2u))

// Every Material that points at `albedo` reuses the same uploaded GPU
// texture; passing the same handle into N Material instances costs one
// upload and N shader-uniform references.
val material = Material.pbr().baseColorTexture(albedo)