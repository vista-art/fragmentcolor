import org.fragmentcolor.*

val flat = listOf(128, 128, 255, 255)
val gold = PbrMaterial().metallic(1.0f).roughness(0.3f)

// A normal map tiled four times over the second UV layout.
val map = gold.normalMap().texture(flat, listOf(1u, 1u)).uv(1u).transform(listOf(0.0f, 0.0f), listOf(4.0f, 4.0f), 0.0f)