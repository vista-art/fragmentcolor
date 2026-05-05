# Shader::new (Python)

Create a shader from a WGSL string or a list of strings. URL slugs require network access.

## Example

```python
from fragmentcolor import Shader

# Simple WGSL fragment shader
shader = Shader("""
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4f(1., 0., 0., 1.); }
""")
```
