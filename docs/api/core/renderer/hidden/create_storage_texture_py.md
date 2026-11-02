# Renderer::create_storage_texture (Python)

Python-only entry point for creating a storage texture with explicit size and format.

## Example

```python
from fragmentcolor import Renderer, Shader, TextureFormat

r = Renderer()
# Empty storage texture -- positional args: (size, format).
tex = r.create_storage_texture([64, 64], TextureFormat.Rgba)

# Pre-seeded with bytes -- same method, optional data kwarg.
pixels = [0] * (64 * 64 * 4)
tex2 = r.create_storage_texture([64, 64], TextureFormat.Rgba, data=pixels)

# One 32-bit counter per texel, for texture atomics.
counts = r.create_storage_texture([4, 4], TextureFormat.R32Uint)
shader = Shader("""
@group(0) @binding(0) var counts: texture_storage_2d<r32uint, atomic>;

@compute @workgroup_size(8, 8)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
  textureAtomicAdd(counts, id.xy / 16u, 1u);
}
""")
shader.set("counts", counts)
```
