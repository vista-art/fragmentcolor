import org.fragmentcolor.*

val mesh = Mesh()
mesh.addVertex( Vertex.pbr(listOf(0.0f, 0.5f, 0.0f)).set(Vertex.UV0, listOf(0.5f, 1.0f)).set(Vertex.UV1, listOf(0.25f, 0.75f)), )

// The occlusion map follows the second UV layout.
val material = Material.pbr().textureUv(TextureSlot.OCCLUSION, 1u)