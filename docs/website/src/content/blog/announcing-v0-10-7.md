---
title: "Announcing FragmentColor v0.10.7"
description: "A personal tour of FragmentColor biggest release yet"
authors: ["rafaelbeckel"]
pubDate: "2025-10-05"
tags: ["release", "rust", "webgpu", "python", "javascript"]
---

Hey friends!

Today I'm shipping FragmentColor v0.10.7, our biggest release yet.

This version guarantees 100% feature parity across Javascript, Python, and Rust. All the documentstion in this website is auto-generated from the library's source code, and all examples are guaranteed to run.

It's funny to call this a "patch" when it's the biggest release I've done so far. I got carried away making the docs truthful, the API complete, and the pipeline calmer — and honestly, I'm very excited about what this unlocks for all of us.

If you just want to dive in:

- Docs: <https://fragmentcolor.org/welcome>
- API Reference: <https://fragmentcolor.org/api>
- Repo: <https://github.com/vista-art/fragmentcolor>

What's new (in plain English)

- Docs that don't lie
  - The docs in docs/api are the single source of truth. During the build, we validate that every public item is documented and has a runnable example. Then we generate the website from those same docs. If it's on the website, it actually runs.

- A friendlier, more complete API
  - You can pass uniforms, textures, samplers, storage buffers, and push constants with simple set/get methods. Meshes attach to shaders and passes in the obvious way. It feels much more "what I think is what I write".

- Rendering that stays out of your way
  - Better pipeline caching, MSAA support, safer surface handling, and a smoother frame path so you can think about images, not plumbing.

- A calmer release flow (for me, and for you)
  - Publishing now happens from a GitHub Release, and CI handles npm, PyPI, and crates.io. Post-publish, the website and examples update themselves. Less ceremony, more creating.

A tiny taste of what you can build now

- A quick single‑shader render for a window or canvas
- A multi‑pass frame where you fan out into multiple shaders, then combine
- Offscreen rendering with a TextureTarget (great for tests or server‑side)
- Texture uploads from bytes or URLs, and predictable uniform/struct updates

Quickstart

Rust

```rust
use fragmentcolor::{Renderer, Shader, Pass, Frame};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let renderer = Renderer::new();

// On desktop, create a target from a window you already have (winit works great)
// let window = ...;
// let target = renderer.create_target(&window)?;

// Or render offscreen:
let target = renderer.create_texture_target([800, 600]);

// Bring your WGSL (or GLSL when enabled)
let mut shader = Shader::new("./shaders/hello_triangle.wgsl")?;
shader.set("color", [0.9_f32, 0.2, 0.8])?;

// One shader is enough to render
renderer.render(&shader, &target)?;

// Or compose passes and frames
let mut pass = Pass::new("main");
pass.add_shader(&shader);

let mut frame = Frame::new();
frame.add_pass(pass);

renderer.render(&frame, &target)?;
# Ok(())
# }
```

JavaScript (Web)

```js
import { Renderer, Shader } from 'fragmentcolor';

// Create a renderer and a canvas target
const renderer = await Renderer.new();
const canvas = document.querySelector('canvas');
const target = await renderer.createTarget(canvas);

// Load a WGSL shader (string, file, or URL)
const shader = await Shader.new('/shaders/hello-triangle.wgsl');
shader.set('color', [0.2, 0.8, 0.9]);

await renderer.render(shader, target);
```

Python

```python
from fragmentcolor import Renderer, Shader

renderer = Renderer()
# Offscreen by default here; use a GUI backend (glfw/pyside) for windows
tex = renderer.create_texture_target([800, 600])

shader = Shader.new("./shaders/hello_triangle.wgsl")
shader.set("color", [0.9, 0.6, 0.2])

renderer.render(shader, tex)
```

What changed under the hood (still in human terms)

- Docs and site
  - docs/api is the source of truth; the build validates coverage and examples, then converts those files to MDX for the website. We also slice the runnable JS/Python snippets directly from the healthchecks.
  - Result: IDE hovers match the website, and the website matches reality.

- API & engine quality of life
  - Clear, typed uniform/storage updates (including arrays and nested structs). Texture and sampler handling that "just works". Multi‑mesh per pass without drama. MSAA and pipeline caching to keep your frames smooth.

- Release and distribution
  - GitHub Release → CI publishes to npm, PyPI, and crates.io → the website bumps its dependency and snapshots the API docs for that version. That's v0.10.7 in a sentence.

Thank you

If you've kicked the tires, filed a bug, or just read along, thank you.

I'm building FragmentColor to make cross‑platform rendering feel welcoming — whether you're on Rust, Web, or Python. If you create something colorful, please share it with me — I'd love to see it!

- Docs: <https://fragmentcolor.org/welcome>
- API: <https://fragmentcolor.org/api>
- GitHub: <https://github.com/vista-art/fragmentcolor>

Happy rendering,

— Rafael
