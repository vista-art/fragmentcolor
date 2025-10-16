# Roadmap

This roadmap summarizes current focus and planned features.

use std::borrow::Borrow;

## 0.11.0 Swift & Kotlin with Uniffi

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
  - [ ] It must give access to all wgpu::RenderPass customizations with sensible defaults, so we keep our API simple while still allowing for advanced use cases.
- [ ] Create specialized Alias objects:
  - [ ] Compute: newtype for Shader, but only allows compute shaders (Shader continues to allow both)
  - [ ] RenderPass: newtype for Pass, but only allows Render passes (Pass continues to allow both)
  - [ ] ComputePass: newtype for Pass, but only allows Compute passes (Pass continues to allow both)
- [ ] Custom blending

## Up Next

### Automated Visual Testing

- Expand snapshot coverage beyond hello_triangle:
  - [ ] circle fragment shader (deterministic params)
  - [ ] compute_texture (deterministic inputs)
  - [ ] particles_splat (seed RNG; accept small tolerance or switch to invariant assertions)
  - [ ] storage texture pipelines (clear + splat sequence)
  - [ ] mesh rendering with vertex + instance attributes
  - [ ] MSAA rendering path and resolve
  - [ ] push-constant: native and uniform-fallback modes
- Test harness improvements:
  - [ ] Add helper to snapshot a Frame (multi-pass) directly
  - [ ] Document UPDATE_EXPECT flow in CONTRIBUTING
  - [ ] Add CI step (macOS) to run snapshots and upload tests/error artifacts on failures
  - [ ] Optional: headless flags for examples to produce snapshots as integration tests

- [ ] Support other types of Window integrations in Python (decouple from RenderCanvas)
  - [ ] Qt
  - [ ] WxWidgets
  - [ ] GLTF
  - [ ] Jupyter

- [ ] Mesh.load_* helpers and JSON inputs.

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
  •  Plan: Add a top-level "device lost" handler that:
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

  ```txt
    Roadmap Status Report

    1. Device-lost recovery path (centralized)
    Status: NOT IMPLEMENTED (Partial surface-level handling only)

    Rationale: The current implementation only handles surface frame errors (Lost/Outdated) with retry logic in src/target/window.rs:56-86 and src/renderer/mod.rs:486-501. There is no centralized device-lost recovery mechanism. The request_device function in src/renderer/platform/all.rs:47-49 only sets up error logging via device.on_uncaptured_error, but doesn't implement re-requesting the adapter/device or rebuilding the RenderContext. The surface-level retry is not the same as full device loss recovery.

    2. Staging-belt style uploads for uniforms  
    Status: NOT IMPLEMENTED

    Rationale: The current UniformBufferPool in src/renderer/buffer_pool/uniform.rs:104-172 uses individual queue.write_buffer calls for each upload. No wgpu::util::StagingBelt is present in the codebase (confirmed by grep search), and there's no belt-like batching mechanism that would flush uploads in a single operation per frame. Each uniform upload triggers an immediate write.

    3. Sampler and texture view caches (descriptor-keyed, forward-looking)
    Status: NOT IMPLEMENTED

    Rationale: While there is a TexturePool for transient MSAA textures (src/renderer/texture_pool.rs), there are no descriptor-keyed caches for samplers or texture views. Samplers are created on-demand in src/texture/sampler.rs:35 and texture views are created fresh each time in src/texture/mod.rs:464-467 using default descriptors. The existing texture pool serves a different purpose (temporary render attachments) and doesn't cache samplers/views for reuse across frames.

    Conclusion

    All three items remain unimplemented and should stay on the TODO list. The repository has good surface-level error handling and existing pooling infrastructure, but lacks the specific centralized device recovery, staging belt uploads, and descriptor-based caching that these roadmap items call for.
  ```

## Backlog (rough ideas)

- [ ] load ShaderToy URLs from Shader::new() and Shader::toy()

- [ ] Multisampling (resolve_target in RenderPassColorAttachments)

- [ ] Components library (prefabs)

- [ ] Improve shader debugging experience
  - [ ] User Interface (eGUI) for runtime debug messages
  - [ ] Utils (gizmo, camera)

- [ ] Consider provideing llms.txt and [MCP](https://modelcontextprotocol.io/introduction)

- [ ] Website & docs
  - [ ] Internationalization groundwork for docs

- [ ] You can remove half of the conversion traits by using Borrow:
    ```rust
    #[derive(Debug)]
    struct A {
        s: String,
    }

    struct B {
        i: usize,
    }

    impl <BB: Borrow<B>> From<BB> for A {
        fn from(b: BB) -> A {
            A {
                s: format!("a-{}", b.borrow().i),
            }
        }
    }

    fn main() {
        let b = B { i: 13 };
        let a1: A = (&b).into();
        let a2: A = b.into();
        println!("{a1:?}, {a2:?}");
    }
    ```

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

- [ ] Simple multi-pass projected shadows
- [ ] Hello ExternalTextures (video processing)
- [ ] Hello multiple screens
- [ ] Hello screen regions / viewport
- [ ] Let's build a simple image editor

#### Complex Scenes (v 2.0 ideas)

- [ ] Model loading
- [ ] Scene tree
- [ ] ECS
- [ ] Future: a Logo-like programming language
