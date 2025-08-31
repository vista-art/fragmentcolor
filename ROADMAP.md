# Roadmap

## Work in Progress

### V 0.10.7 Wasm Fix and Documentation

- [x] fix: Shader went without .set() to NPM (WASM)
- [x] Automate Doc-string replication to all bindings
- [x] Create a distribution via GH release
    - [x] Python
    - [x] Javascript
- [x] Update documentation and examples
  - [x] Renderer
    - [x] constructor
    - [ ] create_target
    - [ ] render
    - [ ] render_image
  - [ ] Target
  - [ ] Shader
    - [ ] constructor
    - [ ] set
    - [ ] get
    - [ ] list_uniforms
    - [ ] list_keys
  - [ ] Pass
    - [ ] constructor
    - [ ] add_shader
  - [ ] Frame
    - [ ] constructor
    - [ ] add_pass
- [ ] Examples must use the published version
- [ ] Update website automatically from docs
- [ ] Refine build and publish processes
- [ ] EndToEnd tests to validate the public API
- [ ] Implement Texture API
  - [ ] Renderer.create_texture(&image) -> Texture
  - [ ] Renderer.create_target(Texture) -> Target
  - [ ] Pass.input(Texture, Op::LOAD|Op::STORE)
  - [ ] Pass.output(Texture)
  - [ ] Texture
  - [ ] StorageTexture
  - [ ] Sampler

### V 0.10.8 Automation and Build System to keep bindings in sync

- [ ] Incorporate the Website in the repository
- [ ] Adopt xtask
- [ ] Automate documentation with xtask
  - [ ] Move Doc Comments to separate MD files
  - [ ] Doc-comments MD files will be replicated on:
     - [ ] Doc-comments
     - [ ] Python Wrappers
     - [ ] JS Wrappers
     - [ ] Website
  - [ ] Copy all doc comments to Website
  - [ ] Reconfigure Vercel to use this repository instead
- [ ] Add more examples
- [ ] Update website content
- [ ] Refine build and publish processes

### V 0.10.9

- [ ] Swift Wrappers (future)

### V 0.10.10

- [ ] Kotlin Wrappers

## Up Next

- Revemp RenderPass API
  - It must give access to all wgpu::RenderPass customizations with sensible defaults, so we keep our API simple while still allowing for advanced use cases.

- [ ] Build System
  - [ ] Unit test all packages before building
  - [ ] Git hook: test builds for all platforms before push
  - [ ] Script to Test, Compile & Publish JS
  - [x] Script to Test, Compile & Publish Python
  - [ ] Script to Test, Compile & Publish Android
  - [ ] Script to Test, Compile & Publish iOS
  - [ ] Script to Test, Compile & Publish Rust + Winit
  - [ ] GHA wheel: Test build all packages for all OSses

- [ ] Release Management System
  - [ ] Automatically update docs from Rust Doc Comments
  - [ ] Update cargo doc
  - [ ] Script to copy contents and publish to Website

## Backlog

- [ ] Support 3D Textures
  - [ ] (check RenderPassColorAttachment.depth_slice) 

- [ ] Support other types of Window integrations in Python (decouple from RenderCanvas)
  - [ ] Qt
  - [ ] WxWidgets
  - [ ] GLTF
  - [ ] Jupyter
- [ ] Compute Pass support
- [ ] Frame setup Save & Load from JSON
- [ ] Ensure we expose all the ways to upload data to a GPU

  - [ ] VertexBuffer
  - [ ] IndexBuffer
  - [ ] StorageBuffer
  - [x] Uniform
  - [ ] Texture
  - [ ] StorageTexture
  - [ ] Sampler
  - [ ] PushConstant

- [ ] Custom blending

- [ ] Multisampling (resolve_target in RenderPassColorAttachments)

- [ ] Components library (prefabs)

- [ ] Improve shader debugging experience

  - [ ] User Interface (eGUI) for runtime debug messages
  - [ ] Utils (gizmo, camera)

- [ ] Consider provideing llms.txt and [MCP](https://modelcontextprotocol.io/introduction)

### Tutorials and Examples

#### Single-pass rendering

- [x] Hello Triangle
- [x] Hello Moving Circle (Uniforms)
- [x] Multiple objects
- [ ] Hello ShaderToy Clone
- [ ] Hello custom geometry (Vertex Input)
- [ ] Hello Instances (simple)
- [ ] Big Particle System (stress test)

#### Multi-pass rendering

- [ ] Simple shadows
- [ ] Hello ExternalTextures (video processing)
- [ ] Hello multiple screens
- [ ] Hello screen regions / viewport
- [ ] Let's build a simple image editor

#### Complex Scenes (v 2.0 ideas)

- [ ] Model loading
- [ ] Scene tree
- [ ] ECS
- [ ] Future: a Logo-like programming language

## Done
