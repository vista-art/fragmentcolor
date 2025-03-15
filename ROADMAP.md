# Roadmap

## Work in Progress

- [ ] V 0.10.2 Python support
  - [x] Python Implementation
  - [ ] Publish to Pip

## Up Next

- [ ] V 0.10.2 Javascript support
- [ ] Javascript Implementation
- [ ] Publish to NPM

- [ ] Build System

  - [ ] Script to Test, Compile & Publish JS
  - [ ] Script to Test, Compile & Publish Python
  - [ ] Script to Test, Compile & Publish Android
  - [ ] Script to Test, Compile & Publish iOS
  - [ ] Script to Test, Compile & Publish Rust + Winit

- [ ] Improve debugging experience
  - [ ] User Interface (eGUI) for runtime debug messages
  - [ ] Utils (gizmo, camera)

## Backlog

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

- [ ] Multisampling (resolve_target in RenderPassColorAttachments)

- [ ] Components library

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
- [ ] Experiment with ECS
- [ ] Future: a Logo-like programming language

## Done

### V 0.10.1 Cleanup and Fix Bugs

- [x] Simplify Shader Internal representation
- [x] BufferPool implementation
- [x] Graceful runtime error handling (no panics)
- [x] Fix uniform getter and setter
- [x] Renderer render() method now has two arguments: Renderable and Target
- [x] Make the Renderer support Shader, Pass and Frame as input
  - [x] Shader
  - [x] Pass
  - [x] Frame
- [x] Improve public interface for Target - make the default easy (one target)
- [x] Set up cross-platform initializers (helper functions)
- [x] remove boilerplate from Rust demos

### V 0.10.0 Set up basic API and basic data structures

- [x] Renderer
- [x] Shader
  - [x] decide how to handle generic set() function
        [x] Pass
  - [x] RenderPass
  - [x] ComputePass
- [x] Frame
- [x] Renderer
- [x] Target

- [x] Design main public interface
- [x] Experimental GLSL Support

### Before V 0.9.0 (2023)

The initial versions of this library (up tp v0.9.0) were completely discarded.

About one year after not touching the code, in January 2025 I force-pushed and rewrote the **v0.10.0** branch from scratch.
