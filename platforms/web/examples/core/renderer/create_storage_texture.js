import { Renderer, Shader, TextureFormat } from "fragmentcolor";

const r = new Renderer();

// Empty storage texture.
const tex = await r.createStorageTexture([64, 64], TextureFormat.Rgba);

// Pre-seeded with bytes.
const pixels = Array(64 * 64 * 4).fill(0);
const tex2 = await r.createStorageTexture([64, 64], TextureFormat.Rgba, pixels);

// One 32-bit counter per texel, for texture atomics.
const counts = await r.createStorageTexture([4, 4], TextureFormat.R32Uint);
const shader = new Shader(`

@group(0) @binding(0) var counts: texture_storage_2d<r32uint, atomic>;

@compute @workgroup_size(8, 8)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
  textureAtomicAdd(counts, id.xy / 16u, 1u);
}

`);
shader.set("counts", counts);