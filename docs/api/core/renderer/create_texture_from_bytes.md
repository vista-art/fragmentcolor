# Renderer::create_texture

Create a [Texture](https://fragmentcolor.org/api/core/texture) from raw image bytes.

- Rust: returns a [Texture](https://fragmentcolor.org/api/core/texture) object wrapping a GPU texture
- JS/Python: parity pending

## Example (Rust)

```rust
let bytes = std::fs::read("./examples/assets/image.png").unwrap();
let renderer = Renderer::new();
let tex = futures::executor::block_on(renderer.create_texture(&bytes)).unwrap();
shader.set("tex", &tex).unwrap();
```
