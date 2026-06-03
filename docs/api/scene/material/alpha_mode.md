# Material::alpha_mode

Set how the renderer interprets the alpha channel of the material's output. Mirrors the
glTF 2.0 `alphaMode` semantics. Pipeline state is baked into the WebGPU pipeline, so
different `AlphaMode` values for the same shader cache to distinct pipelines.

The three variants:

- `AlphaMode::Opaque` (default): depth-test is on, blending is off. Alpha is written
  out but ignored by the framebuffer. Pick this for solid surfaces; it's the cheapest
  option and the only one that lets the depth buffer fully describe the geometry.
- `AlphaMode::Mask`: depth-test is on, blending is off, but the fragment shader
  `discard`s any pixel whose `material.base_color.a` falls below `material.alpha_cutoff`.
  The classic foliage / chain-link / decal trick: keeps the perf profile of opaque
  geometry (no order dependence, full depth writes) while letting silhouettes punch
  out hard-edged transparency. Tune the cut-off with `Material::alpha_cutoff`.
- `AlphaMode::Blend`: depth-test stays on but depth-write turns off, and the color
  target uses standard `SrcAlpha / OneMinusSrcAlpha` over-blend. Use for glass, smoke,
  fades, decals that need soft edges. The renderer sorts blend Models on the Pass
  back-to-front by eye-space Z before drawing, using the Camera attached via
  `Pass::add(&camera)`. Translucent surfaces over-blend in the right order without the
  caller managing it. Limitation: sorts by per-Model AABB centroid (not per-fragment),
  so self-intersecting or interpenetrating translucent meshes can still show artifacts.
  Cross-Material interleaving works correctly: translucent draws across every shader
  in the Pass merge into one globally-sorted back-to-front pass.

This is a pipeline-state flag. Changing it forces the renderer to rebuild the pipeline
for the affected `(shader, alpha_mode, double_sided)` key. Switching it every frame is
fine in practice (the second build hits the cache from then on) but avoid toggling per
draw call inside a frame.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{AlphaMode, Material};

let foliage = Material::pbr()
    .alpha_mode(AlphaMode::Mask)
    .alpha_cutoff(0.3);

let glass = Material::pbr()
    .base_color([0.9, 0.95, 1.0, 0.25])
    .alpha_mode(AlphaMode::Blend);

let solid = Material::pbr().alpha_mode(AlphaMode::Opaque);
# let _ = (foliage, glass, solid);
# Ok(())
# }
```
