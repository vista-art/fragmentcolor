# Building an App in Rust (with FragmentColor's `App` helper)

FragmentColor ships a thin wrapper around [winit](https://docs.rs/winit/)
called [`App`](https://docs.rs/fragmentcolor/latest/fragmentcolor/struct.App.html)
that's the recommended way to drive a windowed renderer in Rust. It owns
the event loop, hides winit's lifecycle (pre-init → resumed → resize →
redraw), and exposes a small set of `on_*` callbacks so the app code
stays focused on what to render.

`App` is Rust-only — it depends on winit, which requires main-thread
ownership of the event loop. Python, JS, Swift, and Kotlin each have
their own native answers; see the other pages in this guide for those.

The cross-platform contract — `Renderer`, `Pass`, `Scene`, `Material`,
`Mesh`, `Camera`, `Light` — is identical across runtimes. The five
sections below show how to wire them into the Rust event loop.

## 1. Wire the Renderer to a window

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{App, Renderer};
use winit::window::WindowAttributes;

let renderer = Renderer::new();
let mut app = App::new(renderer);
app.add_window(WindowAttributes::default().with_title("My App"));
# Ok(())
# }
```

`add_window` queues a window blueprint; the actual surface comes up
inside winit's `resumed` lifecycle once the OS hands the app the right
to create one. `App` does that on your behalf and wires the resulting
`RenderTarget` to the window id.

## 2. Run async setup

Loading textures, building a `Scene`, decoding a `.glb` — all the work
that needs the renderer up but only runs once goes into `on_start`. It
fires after windows are created and gets the list of `Arc<Window>`
handles for any per-window setup.

```rust,no_run
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{App, Camera, Material, Renderer, Scene};
use std::time::Instant;

struct PreviewState {
    scene: Scene,
    camera: Camera,
    start_time: Instant,
}

let renderer = Renderer::new();
let mut app = App::new(renderer);

app.on_start(|app, _windows| {
    let scene = Scene::load("model.glb")?;
    let camera = Camera::perspective(60.0_f32.to_radians(), 1.0, 0.1, 100.0)
        .look_at([0.0, 0.0, 3.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    scene.add(&camera)?;

    // Stash anything the per-frame draw needs to read. App's typed
    // registry handles arbitrary `Send + Sync + 'static` values; for
    // interior-mutability writes wrap in `Arc<RwLock<T>>`.
    app.add_state("preview", PreviewState {
        scene,
        camera,
        start_time: Instant::now(),
    });
    Ok(())
});
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```

## 3. Per-frame draw

`on_draw` fires on every `WindowEvent::RedrawRequested`. Pull state out
of the registry, animate or update uniforms, then hand the renderer the
top-level `Renderable` (typically a `Scene`).

```rust,no_run
# struct PreviewState { scene: fragmentcolor::Scene, camera: fragmentcolor::Camera, start_time: std::time::Instant }
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let renderer = fragmentcolor::Renderer::new();
# let mut app = fragmentcolor::App::new(renderer);
app.on_draw(|app, window_id, _event| {
    let preview = match app.get_state::<PreviewState>("preview") {
        Some(p) => p,
        None => return, // setup hasn't run yet
    };

    // Orbit the camera around the origin once per second.
    let elapsed = preview.start_time.elapsed().as_secs_f32();
    let radius = 3.0_f32;
    preview.camera.look_at(
        [elapsed.cos() * radius, 1.0, elapsed.sin() * radius],
        [0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
    );

    let renderer = app.renderer();
    app.with_target(window_id, |target| {
        renderer.render(&preview.scene, target).expect("render");
    });
});
# Ok(())
# }
```

`Camera::look_at` mutates the same Arc-shared backing — the propagation
hook updates every shader and the Pass-level camera snapshot inline, so
the next frame renders the new view without re-adding the camera.

## 4. Event handlers (resize, input, …)

The window's aspect ratio changes whenever the user drags the corner.
Pair `on_resize` with `Camera::set_aspect` so the projection stays
square. Same Arc-shared backing — no Scene rebuild.

```rust,no_run
# struct PreviewState { scene: fragmentcolor::Scene, camera: fragmentcolor::Camera, start_time: std::time::Instant }
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let renderer = fragmentcolor::Renderer::new();
# let mut app = fragmentcolor::App::new(renderer);
app.on_resize(|app, size| {
    if let Some(preview) = app.get_state::<PreviewState>("preview") {
        preview.camera.set_aspect(size.width as f32 / size.height as f32);
    }
});
# Ok(())
# }
```

Keyboard, mouse, scroll, IME — every winit `WindowEvent` has a matching
`on_*` callback (see `examples/rust/examples/app_healthcheck.rs` for the
exhaustive list). They all take the same `(app, window_id, event_data)`
shape so state lookups stay uniform.

## 5. Drive the loop

```rust,no_run
# fn main() {
# let mut app = fragmentcolor::App::new(fragmentcolor::Renderer::new());
app.run();
# }
```

`App::run` calls into winit's `EventLoop::run`. It blocks the main
thread until the window closes, dispatching events through the
registered callbacks. There's no separate render thread — winit's
`RedrawRequested` is the frame tick, and you call `renderer.render`
directly from there.

## When to skip `App`

`App` is the recommended Rust pattern. Reach for raw winit + your own
`ApplicationHandler` impl when:

- You need a custom event loop driver (e.g. fixed-tick game loop with
  vsync gating in your own scheduler).
- You're embedding FragmentColor inside another engine that already
  owns the event loop.
- You need multi-window with significantly different render strategies
  per window.

In all three cases the cross-platform `Renderer` / `Pass` / `Scene`
APIs work the same — you just drive `renderer.render(...)` from
wherever your loop ticks. `App` is one shim; you can write your own.
