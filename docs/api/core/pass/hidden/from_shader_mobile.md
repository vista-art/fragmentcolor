# Pass::fromShader (mobile)

Mobile constructor that creates a `Pass` configured for a specific `Shader`.

Uniffi always marshals strings by value (`String`, not `&str`), and wraps
Object parameters as `Arc<Shader>`. This shim adapts those differences so the
core `Pass::from_shader(&str, &Shader)` stays untouched for Rust callers.

## Example

```rust
// hidden mobile binding; no public example
```
