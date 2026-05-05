# Shader::new(input: string | string[])

Creates a new [Shader](https://fragmentcolor.org/api/core/shader) from one of:

- a raw WGSL source string,
- a registry **slug** like `"sdf2d/circle"` (resolved against [Shader::set_registry](https://fragmentcolor.org/api/core/shader#shaderset_registry)),
- a `https://` URL pointing at a `.wgsl` file,
- a local file path ending in `.wgsl`, `.glsl`, `.frag`, or `.vert`,
- or **an array** mixing any of the above. Parts are resolved (fetched, read, looked up), deduplicated by source hash, and concatenated in order before validation.

If validation fails, the error message indicates the location of the error. If validation passes, the shader is guaranteed to work on the GPU. All uniforms are initialized to their default zero values.

GLSL is supported only as a single part (`.vert` / `.frag` / `.glsl` path). Mixing GLSL with other parts is rejected.

## Example - Single Shader Source

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::Shader;

let shader = Shader::new(r#"
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
"#)?;

# assert!(shader.list_keys().len() >= 1);
# Ok(())
# }
```

## Example - Shader Composition

The public registry at `https://fragmentcolor.org/shaders/` exposes pure helper
functions you can pull into your own shader. Pass them alongside your main
source as an array; they are concatenated in order and treated as a single
WGSL module.

```rust,ignore
use fragmentcolor::Shader;

let main = r#"
    @vertex fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
        let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
        return vec4<f32>(p[i], 0.0, 1.0);
    }

    @fragment fn fs(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
        let d = circle(pos.xy - vec2<f32>(400.0, 300.0), 100.0);
        let n = simplex2(pos.xy * 0.01);
        return vec4<f32>(vec3<f32>(step(0.0, d) + n * 0.1), 1.0);
    }
"#;

let shader = Shader::new([
    "sdf2d/circle",      // pure function: fn circle(p: vec2<f32>, r: f32) -> f32
    "noise/simplex2",    // pure function: fn simplex2(v: vec2<f32>) -> f32
    main,
])?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Platform-specific: Web

In WASM the constructor cannot perform network requests. Pass an array of
**raw source strings** to `new Shader([...])`, or use the async
[Shader::fetch](https://fragmentcolor.org/api/core/shader#shaderfetch) builder
to resolve URLs and slugs.
