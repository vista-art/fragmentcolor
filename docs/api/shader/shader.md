# Shader

The [Shader](https://fragmentcolor.org/docs/api/shader) object is the main building block in FragmentColor.

It takes a WGSL or GLSL shader source as input, parses it, validates it, and exposes the uniforms as keys.

To draw your shader, you must use your [Shader](https://fragmentcolor.org/docs/api/shader) instance as input to a [Renderer](https://fragmentcolor.org/docs/api/renderer).

You can compose [Shader](https://fragmentcolor.org/docs/api/shader) instances into a [Pass](https://fragmentcolor.org/docs/api/pass) object to create more complex rendering pipelines.

You can also create renderings with multiple Render Passes by using multiple [Pass](https://fragmentcolor.org/docs/api/pass) instances to a [Frame](https://fragmentcolor.org/docs/api/frame) object.
