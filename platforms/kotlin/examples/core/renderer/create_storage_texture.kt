import org.fragmentcolor.*

val r = Renderer()

// Empty storage texture.
val tex = r.createStorageTexture(listOf(64u, 64u), TextureFormat.RGBA)

// Pre-seeded with bytes.
val pixels = ByteArray(64 * 64 * 4)
val tex2 = r.createStorageTexture(listOf(64u, 64u), TextureFormat.RGBA, pixels)

// One 32-bit counter per texel, for texture atomics.
val counts = r.createStorageTexture(listOf(4u, 4u), TextureFormat.R32_UINT)
val shader = Shader("""
@group(0) @binding(0) var counts: texture_storage_2d<r32uint, atomic>;

@compute @workgroup_size(8, 8)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
  textureAtomicAdd(counts, id.xy / 16u, 1u);
}

""")
shader.set("counts", counts)