# Light::set_cone_angles

Update the inner and outer cone half-angles (radians). Defined for spot
lights only; returns `Ok(self)` for chaining. Calling this on a
directional or point light returns
`Err(LightError::FieldNotApplicable { kind, field: "set_cone_angles" })`.

The inner half-angle is the band where the light reaches full intensity;
between the inner and outer the contribution smoothly rolls off; beyond
the outer the fragment receives nothing from this light. Pass equal
values for a hard-edged spot, or pass `(0.0, π/4)` for the default
soft 45° cone.

The new values propagate live to every shader the Light has already
been wired into; no re-attach needed.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Light;

let torch = Light::spot([0.0, 1.8, 1.0], [0.0, -1.0, 0.0], [1.0, 1.0, 1.0]);
torch.set_cone_angles(0.15, 0.4)?;

// Non-spot lights error.
let lamp = Light::point([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
let unsupported = lamp.set_cone_angles(0.15, 0.4);
# assert!((torch.inner_cone_angle().unwrap() - 0.15).abs() < 1.0e-6);
# assert!((torch.outer_cone_angle().unwrap() - 0.4).abs() < 1.0e-6);
# assert!(unsupported.is_err());
# Ok(())
# }
```
