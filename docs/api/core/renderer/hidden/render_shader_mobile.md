# Renderer::render_shader_mobile()

Uniffi variant of `render` targeting a `WindowTarget`. Since uniffi cannot
marshal the generic `&impl Renderable` / `&impl Target` signature, the
mobile bindings expose one concrete method per (renderable × target)
combination. Swift / Kotlin extensions recombine them into the single
idiomatic `render(renderable, target)` overload.

## Example

```rust
// hidden file; no public example
```
