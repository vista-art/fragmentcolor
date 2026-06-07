# Building Apps with FragmentColor

FragmentColor's render API (`Renderer`, `Pass`, `Scene`, `Material`,
`Mesh`, `Camera`, `Light`) is identical across every runtime. What
differs is how the **render loop** ticks: each runtime has its own
ownership model for the event loop and its own conventions for window /
canvas / surface management.

This guide collects the canonical pattern for each supported runtime.
Every page follows the same five-section shape so callers can map
between them:

1. Wire the renderer to a window / canvas / surface
2. Run async setup (load textures, build the Scene, create targets)
3. Per-frame draw callback
4. Event handlers (resize, input, lifecycle)
5. Drive the render loop

The cross-platform contract is what's shared. The loop and event
plumbing is what's idiomatic per runtime.

## Per-platform tutorials

| Runtime | Loop driver | FragmentColor entry point |
| ------- | ----------- | ------------------------- |
| [Rust (with `App`)](./rust-with-app.md) | winit `EventLoop` | `App` (recommended) |
| Rust (raw winit) | winit `EventLoop` | `ApplicationHandler` impl |
| Python | `RenderCanvas.request_draw_callback` | `Renderer.render` |
| JavaScript | `requestAnimationFrame` | `Renderer.render` |
| Kotlin (Android) | `GLSurfaceView.onDrawFrame` | `Renderer.render` |
| Swift (iOS) | `MTKView.draw(in:)` | `Renderer.render` |

Rust is documented today. Python (RenderCanvas), JavaScript (RAF), Kotlin
(GLSurfaceView), and Swift (MTKView) follow the same five-section shape and
ship as they're written.

## Where the cross-platform parts live

The Renderer / Pass / Scene / Material / Mesh / Camera / Light surface
is documented in [`docs/api/`](../../api/). The platforms differ on:

- **Loop driver**: `EventLoop::run` (Rust), browser RAF (JS),
  `request_draw_callback` (Python), `onDrawFrame` (Android),
  `draw(in:)` (iOS).
- **Surface creation**: winit `Window` (Rust), `<canvas>` (JS),
  `RenderCanvas` (Python), `GLSurfaceView` (Android), `MTKView` (iOS).
- **State management**: Rust uses owned structs or App's typed registry;
  JS / Python idiomatically close over their state in callbacks; Android
  and iOS lean on the framework's view-controller / activity lifecycle.

Everything else (Scene construction, Material builders, vertex layout,
lighting, transparency, glTF loading) maps 1:1 across runtimes.
