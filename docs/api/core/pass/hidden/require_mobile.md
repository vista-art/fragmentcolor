# Pass::require (mobile)

Mobile binding for `Pass::require`. Accepts a `Vec<RenderableHandle>` where
each element is a `Shader`, `Pass`, or `Mesh` variant.

Uniffi cannot marshal `&impl Renderable` across the FFI boundary, so the
mobile shim accepts the concrete `RenderableHandle` enum, the same enum used
by `Renderer::render_mobile`. Swift / Kotlin extension shims provide natural
typed overloads (`pass.require(shader)`, `pass.require(pass)`, etc.) that wrap
the value into the matching variant invisibly.

## Example

```rust
// hidden mobile binding; no public example
```
