# Shader::set_registry(base_url: string)

Override the base URL used to resolve shader **slugs** (e.g. `"sdf2d/circle"`).

When a slug is passed to [Shader::new](https://fragmentcolor.org/api/core/shader#shadernew), it is expanded to:

```text
<base_url>/<slug>.wgsl
```

The default base URL is `https://fragmentcolor.org/shaders/`, which serves the public shader registry. Override it to point at your own collection (a CDN, a local dev server, or a mirror) without changing the rest of your code.

The base may end with or without a trailing slash; both are normalised at lookup time.

This setting is **process-wide**. It applies to every subsequent call to `Shader::new(...)` until overridden again.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::Shader;

// Point at your own mirror of the registry
Shader::set_registry("https://cdn.example.com/shaders/");

// Now the slug "sdf2d/circle" resolves to https://cdn.example.com/shaders/sdf2d/circle.wgsl
// (Skipping the actual fetch in this doctest)
# let _ = Shader::default();
# Ok(())
# }
```
