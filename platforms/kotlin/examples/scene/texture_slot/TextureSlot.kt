import org.fragmentcolor.*

val material = Material.pbr().textureUv(TextureSlot.OCCLUSION, 1u).textureTransform(TextureSlot.NORMAL, listOf(0.0f, 0.0f), listOf(4.0f, 4.0f), 0.0f)