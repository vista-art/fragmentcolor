import org.fragmentcolor.*

// Every map spans the surface twice, and the normal map eight times.
val tiles = PbrMaterial().uvTransform(listOf(0.0f, 0.0f), listOf(2.0f, 2.0f), 0.0f)
val normals = tiles.normalMap().transform(listOf(0.0f, 0.0f), listOf(8.0f, 8.0f), 0.0f)