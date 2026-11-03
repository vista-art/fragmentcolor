import org.fragmentcolor.*

// Every map spans the surface twice, and the normal map eight times.
val tiles = Material.pbr().uvTransform(listOf(0.0f, 0.0f), listOf(2.0f, 2.0f), 0.0f).textureTransform(TextureSlot.NORMAL, listOf(0.0f, 0.0f), listOf(8.0f, 8.0f), 0.0f)