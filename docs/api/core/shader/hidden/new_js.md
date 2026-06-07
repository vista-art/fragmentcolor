# Shader::new (JavaScript)

JavaScript override for `Shader::new`. The Rust example shows two flows
(single source string and registry-slug composition) in two `## Example`
blocks. The website builder concatenates both into a single JS file,
which produces a duplicate `import { Shader }` and a `Shader.new([...])`
call without `new`, so we override here with the JS-shaped variant.

In WASM the constructor cannot perform network requests, so registry
slugs / URLs go through `Shader.fetch(...)` instead. The override below
sticks to a single raw WGSL source.

## Example

```js
import { Shader } from "fragmentcolor";

const shader = new Shader(`
    @vertex
    fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
        var pos = array<vec2<f32>, 3>(
            vec2<f32>(-1.0, -1.0),
            vec2<f32>( 3.0, -1.0),
            vec2<f32>(-1.0,  3.0)
        );
        return vec4<f32>(pos[index], 0.0, 1.0);
    }

    @group(0) @binding(0)
    var<uniform> resolution: vec2<f32>;

    @fragment
    fn fs_main() -> @location(0) vec4<f32> {
        return vec4<f32>(1.0, 0.0, 0.0, 1.0); // Red
    }
`);
```
