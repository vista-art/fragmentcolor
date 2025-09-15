# Renderer::create_texture_from_file

Create a [Texture](https://fragmentcolor.org/api/core/texture) from an image file path.

- Rust: returns a [Texture](https://fragmentcolor.org/api/core/texture) object wrapping a GPU texture
- JS/Python: parity pending

## Example (Rust)

```rust
let renderer = Renderer::new();
let tex = futures::executor::block_on(renderer.create_texture_from_file("./examples/assets/image.png")).unwrap();
shader.set("tex", &tex).unwrap();
```
