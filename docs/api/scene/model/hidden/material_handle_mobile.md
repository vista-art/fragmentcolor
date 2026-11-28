# Model::materialHandle (mobile)

Mobile binding for `Model::material`.
Uniffi can't return one of several object types, so the raw method returns the
`MaterialHandle` enum, with one case per material type.
The Swift and Kotlin extensions expose it as `material()`, which returns the
`PbrMaterial`, `UnlitMaterial`, `ToonMaterial` or `Shader` itself, as JavaScript
and Python do.

## Example

```rust
// hidden mobile binding; no public example
```
