# AGENTS.md — Frame rules (short)

DAG orchestration
- `Frame` is a DAG of passes. Nodes are created via `Frame::add_pass(&Pass)`; edges live in a private `dependencies: Vec<(parent, child)>` tracked by pass index.
- Execution order is a topological sort (Kahn's algorithm). Cycles or invalid pass references produce a clear `FrameError`; there is no silent fallback.
- A public edge-builder API (e.g. `Frame::connect`) is still on the roadmap — today only tests add edges via the `pub(crate) fn test_add_dependency` shim. Keep new public APIs small and typed (`FrameError`).

Presentation
- There is no explicit `Frame.present(pass)` method. The renderer walks the topologically-sorted pass list and presents whichever render pass is executed last (leaf).
- If all passes are compute, nothing is presented and the frame renders entirely offscreen.

Public API
- Public methods stay thin and return `FrameError` instead of panicking.
- `Frame` implements `Renderable` so it can be passed directly to `Renderer::render`.
