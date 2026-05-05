# FragmentColor for JavaScript (WASM)

[FragmentColor](https://fragmentcolor.org) is a cross‑platform GPU programming library implemented in Rust and wgpu, compiled to WebAssembly for JavaScript.

This README is specific to the npm package. For Rust usage, see the repository README.md. For Python, see README_PY.md.

- Documentation: https://fragmentcolor.org/welcome
- API Reference: https://fragmentcolor.org/api

## Install

```bash
npm install fragmentcolor
# or
pnpm add fragmentcolor
# or
yarn add fragmentcolor
```

## Quick start

```js
import init, { Renderer, Shader, Pass } from "fragmentcolor";

async function start() {
  // Initializes the WASM module
  await init();

  // Initializes a renderer and a target compatible with the given canvas
  const canvas = document.getElementById("my-canvas");
  const renderer = new Renderer();
  const target = await renderer.createTarget(canvas);

  // You can pass the shader as a source string, a registry slug, or an
  // https URL pointing at a .wgsl file.
  const circle = await Shader.fetch("./path/to/circle.wgsl");
  const triangle = await Shader.fetch("https://fragmentcolor.org/shaders/sdf2d/circle.wgsl");
  const shader = new Shader(`
    struct VertexOutput {
        @builtin(position) coords: vec4<f32>,
    }

    struct MyStruct {
        my_field: vec3<f32>,
    }

    @group(0) @binding(0)
    var<uniform> my_struct: MyStruct;

    @group(0) @binding(1)
    var<uniform> my_vec2: vec2<f32>;

    @vertex
    fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
        const vertices = array(
            vec2( -1., -1.),
            vec2(  3., -1.),
            vec2( -1.,  3.)
        );
        return VertexOutput(vec4<f32>(vertices[in_vertex_index], 0.0, 1.0));
    }

    @fragment
    fn fs_main() -> @location(0) vec4<f32> {
        return vec4<f32>(my_struct.my_field, 1.0);
    }
  `);

  // The library binds and updates the uniforms automatically
  shader.set("my_struct.my_field", [0.1, 0.8, 0.9]);
  shader.set("my_vec2", [1.0, 1.0]);

  // One shader is all you need to render
  renderer.render(circle, target);

  // But you can also combine multiple shaders in a render Pass
  const rpass = new Pass("single pass");
  rpass.addShader(circle);
  rpass.addShader(triangle);
  rpass.addShader(shader);
  renderer.render(rpass, target);

  // Finally, you can render an array of passes in order. No extra type needed.
  const passes = [rpass, new Pass("GUI pass")];
  renderer.render(passes, target);

  // To animate, simply update the uniforms in a loop
  function animate() {
    circle.set("position", [0.0, 0.0]);
    renderer.render(passes, target);
    requestAnimationFrame(animate);
  }
  animate();
}

start();
```

## Web (WASM) development

```bash
# Build WASM package (wasm-pack target web) and sync into local JS examples
./build_web        # add --debug for a debug build

# Run JS demos (Vite dev server) and open browser
./run_web repl     # or: ./run_web multipass | ./run_web headless

# Manual alternative
pnpm --dir examples/javascript install
pnpm --dir examples/javascript dev
```

## Documentation & website

- Docs source of truth lives in docs/api and is referenced from code via `#[lsp_doc]`.
- Examples on method pages are sliced from the healthcheck scripts; no filesystem reads in docs.
- Doc examples follow async + pollster patterns on the Rust side and are converted to JavaScript automatically.

## Platform support

Platform support is aligned with upstream wgpu:

| API    | Windows      | Linux/Android   | macOS/iOS | Web (wasm)  |
| ------ | ------------ | --------------- | --------- | ----------- |
| Vulkan | ✅           | ✅              | 🌋        |             |
| Metal  |              |                 | ✅        |             |
| DX12   | ✅           |                 |           |             |
| OpenGL | 🆗 (GL 3.3+) | 🆗 (GL ES 3.0+) | 📐        | 🆗 (WebGL2) |
| WebGPU |              |                 |           | ✅          |

✅ = First Class Support  
🆗 = Downlevel/Best Effort Support  
📐 = Requires the ANGLE translation layer (GL ES 3.0 only)  
🌋 = Requires the MoltenVK translation layer

## Limitations (planned features)

- See the ROADMAP and CHANGELOG on the [repository](https://github.com/vista-art/fragmentcolor) for planned features and known limitations.

---

Note on generation: this README_JS.md is generated from this template (README_JS.tpl.md) and the repository README.md examples by the build script. Do not edit the generated README_JS.md directly.
