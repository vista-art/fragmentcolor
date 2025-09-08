# Shader

The [Shader](https://fragmentcolor.org/api/core/shader) object is the main building block in [FragmentColor](https://fragmentcolor.org).

It takes a WGSL or GLSL shader source as input, parses it, validates it, and exposes the uniforms as keys.

To draw your shader, you must use your [Shader](https://fragmentcolor.org/api/core/shader) instance as input to a [Renderer](https://fragmentcolor.org/api/core/renderer).

You can compose [Shader](https://fragmentcolor.org/api/core/shader) instances into a [Pass](https://fragmentcolor.org/api/core/pass) object to create more complex rendering pipelines.

You can also create renderings with multiple Render Passes by using multiple [Pass](https://fragmentcolor.org/api/core/pass) instances to a [Frame](https://fragmentcolor.org/api/core/frame) object.

## Example

```rust
use fragmentcolor::Shader;

let _shader = Shader::default();
```
