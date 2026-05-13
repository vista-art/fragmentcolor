# Material::alpha_mode

Set how the renderer interprets the alpha channel of the material's output. Mirrors the
glTF 2.0 `alphaMode` semantics — pipeline state is baked into the WebGPU pipeline, so
different `AlphaMode` values for the same shader cache to distinct pipelines.

The three variants:

- `AlphaMode::Opaque` (default) — depth-test is on, blending is off. Alpha is written
  out but ignored by the framebuffer. Pick this for solid surfaces; it's the cheapest
  option and the only one that lets the depth buffer fully describe the geometry.
- `AlphaMode::Mask` — depth-test is on, blending is off, but the fragment shader
  `discard`s any pixel whose `material.base_color.a` falls below `material.alpha_cutoff`.
  The classic foliage / chain-link / decal trick: keeps the perf profile of opaque
  geometry (no order dependence, full depth writes) while letting silhouettes punch
  out hard-edged transparency. Tune the cut-off with `Material::alpha_cutoff`.
- `AlphaMode::Blend` — depth-test stays on but depth-write turns off, and the color
  target uses standard `SrcAlpha / OneMinusSrcAlpha` over-blend. Use for glass, smoke,
  fades, decals that need soft edges. Order-dependent: sort back-to-front yourself
  before submitting if you care about correctness across overlapping translucent layers.

This is a pipeline-state flag — changing it forces the renderer to rebuild the pipeline
for the affected `(shader, alpha_mode, double_sided)` key. Switching it every frame is
fine in practice (the second build hits the cache from then on) but avoid toggling per
draw call inside a frame.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{AlphaMode, Material, Renderer};

let renderer = Renderer::new();
let foliage = Material::pbr(&renderer)
    .await?
    .alpha_mode(AlphaMode::Mask)
    .alpha_cutoff(0.3);

let glass = Material::pbr(&renderer)
    .await?
    .base_color([0.9, 0.95, 1.0, 0.25])
    .alpha_mode(AlphaMode::Blend);

let solid = Material::pbr(&renderer).await?.alpha_mode(AlphaMode::Opaque);
# let _ = (foliage, glass, solid);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
