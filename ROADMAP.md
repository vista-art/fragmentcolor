# Roadmap

## Work in Progress

- [ ] V 0.10.6 Javascript support
  - [ ] Javascript Implementation
  - [ ] Publish to NPM

## Up Next

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

  - [ ] Git Hook: enforce conventional commits (fix:, feat: etc)
  - [ ] Create a distribution via GH release
  - [ ] Automatically update docs from Rust Doc Comments
  - [ ] Update cargo doc
  - [ ] Script to copy contents and publish to Website

## Backlog

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
