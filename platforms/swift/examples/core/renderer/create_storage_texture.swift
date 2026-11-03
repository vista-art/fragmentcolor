import FragmentColor

let r = Renderer()

// Empty storage texture.
let tex = try await r.createStorageTexture([64, 64], TextureFormat.rgba)

// Pre-seeded with bytes.
let pixels = Array(repeating: 0, count: 64 * 64 * 4)
let tex2 = try await r.createStorageTexture([64, 64], TextureFormat.rgba, pixels)

// One 32-bit counter per texel, for texture atomics.
let counts = try await r.createStorageTexture([4, 4], TextureFormat.r32Uint)
let shader = try Shader("""
@group(0) @binding(0) var counts: texture_storage_2d<r32uint, atomic>;

@compute @workgroup_size(8, 8)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
  textureAtomicAdd(counts, id.xy / 16u, 1u);
}

""")
try shader.set("counts", counts)