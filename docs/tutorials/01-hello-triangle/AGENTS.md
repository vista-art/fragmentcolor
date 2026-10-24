# 01-hello-triangle

This directory holds the **code** for the eight steps of the tutorial. The
**prose** lives in `docs/website/src/content/docs/tutorials/01-hello-triangle.mdx`.

## Files

Each step folder holds the same program in five languages:

- `example.rs`, the canonical source, registered as a `cargo --example`
  target in `examples/rust/Cargo.toml`.
- `example.js`, the module the page's `<Demo>` imports and runs.
- `example.py`, `example.swift` and `example.kt`, which the page shows in
  their language tabs in place of a transpiled version.

Every port has the same two functions: `setup`, which runs once and
returns what `frame` needs, and `frame(state, renderer, target, time)`,
which runs on every refresh. Only those two, and the shader, appear on the
page.

`host.rs` and `host.py` open a window and call them. Run a step with
`cargo run -p fce --example tutorial_01_hello_triangle_three_lines` or
`python docs/tutorials/01-hello-triangle/host.py 01-three-lines`. Set
`FC_TUTORIAL_FRAMES=n` to stop after n frames. In Rust that renders them
into a 1280 by 720 texture with no window, prints the average frame time,
and fails if the last frame is one flat color; `FC_TUTORIAL_IMAGE=path` saves it.
The step counts (a million triangles in step 7, 8 million particles in
step 8) come from those timings. Swift and Kotlin have no
host: an app calls `setup` and `frame` from its render loop.

## Region markers

The page shows regions marked in each file:

```rust
// #region: setup
... visible in the snippet ...
// #endregion: setup
```

`scripts/tutorials.rs` parses them (Python uses `# #region: name`). A
region whose name contains `shader` or `wgsl` sits inside the WGSL string,
with the markers as WGSL comments, and the page shows it once as WGSL.
Every port must carry the same WGSL as `example.rs`; the build warns when
a shader region differs.
