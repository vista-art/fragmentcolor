# FragmentColor for JavaScript (WASM)

[FragmentColor](https://fragmentcolor.org) is a universal GPU abstraction for Rust, JavaScript, Python, Swift, and Kotlin. This package is the JavaScript binding, compiled to WebAssembly.

This README is specific to the npm package. For Rust usage, see the repository README.md. For Python, see `platforms/python/README.md` in the repository.

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
  // Loads the WebAssembly module
  await init();

  // A renderer and a target that draws into the canvas
  const canvas = document.getElementById("my-canvas");
  const renderer = new Renderer();
  const target = await renderer.createTarget(canvas);

  // A shader from WGSL source. Its uniforms are ready to set by name.
  const background = new Shader(`
    struct VertexOutput {
        @builtin(position) coords: vec4<f32>,
    }

    struct MyStruct {
        my_field: vec3<f32>,
    }

    @group(0) @binding(0)
    var<uniform> my_struct: MyStruct;

    @vertex
    fn vs_main(@builtin(vertex_index) index: u32) -> VertexOutput {
        let vertices = array(vec2(-1., -1.), vec2(3., -1.), vec2(-1., 3.));
        return VertexOutput(vec4<f32>(vertices[index], 0.0, 1.0));
    }

    @fragment
    fn fs_main() -> @location(0) vec4<f32> {
        return vec4<f32>(my_struct.my_field, 1.0);
    }
  `);
  background.set("my_struct.my_field", [0.1, 0.8, 0.9]);

  // One shader is all you need to render
  renderer.render(background, target);

  // Library functions join your own source, which calls them.
  // On the web, Shader.fetch loads them from fragmentcolor.org.
  const circle = await Shader.fetch([
    "sdf2d/circle",
    `
    @group(0) @binding(0)
    var<uniform> center: vec2<f32>;

    @vertex
    fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
        let vertices = array(vec2(-1., -1.), vec2(3., -1.), vec2(-1., 3.));
        return vec4<f32>(vertices[index], 0.0, 1.0);
    }

    @fragment
    fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
        if circle(pos.xy - center, 100.0) > 0.0 {
            discard;
        }
        return vec4<f32>(1.0, 0.4, 0.2, 1.0);
    }
    `,
  ]);

  // Passes render in order, and the second one keeps what the first drew.
  const circlePass = Pass.fromShader("circle", circle);
  circlePass.loadPrevious();
  const passes = [Pass.fromShader("background", background), circlePass];

  // To animate, update the uniforms and render again on each frame
  function animate(time) {
    const size = target.size();
    const x = size.width / 2 + Math.cos(time / 1000) * 100;
    circle.set("center", [x, size.height / 2]);
    renderer.render(passes, target);
    requestAnimationFrame(animate);
  }
  requestAnimationFrame(animate);
}

start();
```

## Web (WASM) development

```bash
# Build WASM package (wasm-pack target web) and sync into local JS examples
./fc build_web        # add --debug for a debug build

# Run JS demos (Vite dev server) and open browser
./fc run_web repl     # or: ./fc run_web gallery | ./fc run_web visual

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

Note on generation: this README.md is generated from README.tpl.md (same directory) and the repository README.md examples by the build script. Do not edit the generated README.md directly.
