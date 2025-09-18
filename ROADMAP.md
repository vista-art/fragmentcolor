# Roadmap

This roadmap summarizes current focus and planned features.

## 0.10.7 API stabilization and documentation automation

- [ ] Rendering features
  - [x] Begin Texture & Sampler support in Shaders
  - [ ] Geometry/instancing groundwork

- [ ] Ensure we expose all the ways to upload data to a GPU

  - [ ] VertexBuffer
  - [ ] IndexBuffer
  - [x] StorageBuffer
  - [ ] StorageBuffer: Arrays
  - [x] Uniform
  - [ ] Uniform: Arrays
  - [x] Texture
  - [x] StorageTexture
  - [x] Sampler
  - [ ] PushConstant

## 0.10.8 Swift & Kotlin with Uniffi

- [ ] Swift Wrappers
- [ ] Kotlin Wrappers
- [ ] Mobile platform wiring
  - [ ] Contribute to Uniffi to suport renaming structs (so I can follow the same pattern as in Python/JS)
  - [ ] iOS: create a safe helper to wrap an existing CAMetalLayer into a wgpu::Surface, then delegate to a core helper that returns WindowTarget
  - [ ] Android: finalize uniffi method scaffolding and ensure JNI handle passing is safe; wire AndroidTarget and AndroidTextureTarget
  - [ ] Core: add a helper like create_target_from_surface(surface, size) to remove duplication across platforms (Web/Python/iOS/Android)
  - [ ] Add E2E tests for iOS/Android wrappers once bindings are generated
  - [ ] Script to Test, Compile & Publish Android
  - [ ] Script to Test, Compile & Publish iOS
- [ ] Revemp RenderPass API
  - It must give access to all wgpu::RenderPass customizations with sensible defaults, so we keep our API simple while still allowing for advanced use cases.
- [ ] Support 3D Textures
  - [ ] (check RenderPassColorAttachment.depth_slice)

## Up Next

- [ ] Compute Pass support
- [ ] Support other types of Window integrations in Python (decouple from RenderCanvas)
  - [ ] Qt
  - [ ] WxWidgets
  - [ ] GLTF
  - [ ] Jupyter

- [ ] Frame setup Save & Load from JSON
- [ ] Support Load and Save Shader states as JSON (uniform values, textures, etc.)
  - [ ] Define JSON schema to extract and set default Uniform values

- [ ] This was removed from Shader and maybe added again in the future:
  - [ ] This should be under a feature flag
    ```rust
    // removed from shader/input.rs
    if is_json {
        let json: serde_json::Value = serde_json::from_str(&body)?;
        let source = json["source"]
            .as_str()
            .ok_or_else(|| ShaderError::ParseError("JSON shader source not found".into()))?;
        return load_shader(source);
    }

    // removed from errors.rs
    #[error("JSON Deserialization Error: {0}")]
    JsonError(#[from] serde_json::Error),
    ```

### Buffer Pool

1. Device-lost recovery path (centralized)
  •  Why: Ruffle takes a robust stance on device/surface loss. We handle surface frame errors already, but not full device loss.
  •  Plan: Add a top-level “device lost” handler that:
  •  Re-requests the adapter/device.
  •  Rebuilds renderer context and reconfigures surfaces/targets.
  •  Clears or rebuilds caches/pools safely.
  •  Acceptance:
  •  Simulated device lost (where possible) recovers without crashing.
  •  Clear docs on which state is preserved vs reinitialized.

1. Staging-belt style uploads for uniforms
  •  Why: Ruffle patterns and wgpu::util::StagingBelt help amortize upload buffers.
  •  Plan: Either adopt StagingBelt or adapt BufferPool to batch/belt-like behavior (single per-frame belt flush), ensuring we still honor uniform alignment and backend constraints.
  •  Acceptance:
    •  No perf regression; fewer small writes.
    •  All uniform upload tests/examples behave identically.

1. Sampler and texture view caches (forward-looking)
  •  Why: When we start sampling textures, we'll want stable, keyed caches (descriptor-based).
  •  Plan: Add caches keyed by sampler/texture view descriptors with simple LRU caps. Ensure reuse across frames.
  •  Acceptance:
    •  Minimal API scaffolding with tests; no behavior change until used.

## Backlog (rough ideas)

- [ ] Custom blending

- [ ] Multisampling (resolve_target in RenderPassColorAttachments)

- [ ] Components library (prefabs)

- [ ] Improve shader debugging experience

  - [ ] User Interface (eGUI) for runtime debug messages
  - [ ] Utils (gizmo, camera)

- [ ] Consider provideing llms.txt and [MCP](https://modelcontextprotocol.io/introduction)

- [ ] Website & docs
  - [ ] Internationalization groundwork for docs

### Performance

- [ ] Async pipeline warming
  •  Why: Reduce first-frame stutter (precompile likely pipelines).
  •  Plan: Add a background warm step per (ShaderHash, format, sample_count) when shaders/passes are registered. Feature-gate if needed.
  •  Acceptance:
    •  Optional; no functional change when disabled. Documented and tested where possible.

- [ ] Frame acquire telemetry (optional)
  •  Why: Confirm centralized retry is effective.
  •  Plan: Add lightweight counters or log-once warnings for repeated Lost/Outdated, distinguish target-local and centralized retries.
  •  Acceptance:
    •  Visible, throttled logs in debug runs; no spam.

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
