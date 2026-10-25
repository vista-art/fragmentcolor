# eight-million-particles

This directory holds the **code** for the tutorial whose prose lives in
`docs/website/src/content/docs/tutorials/eight-million-particles.mdx`.

`01-splat` is one program in five languages (`example.rs`, `.js`, `.py`,
`.swift`, `.kt`), laid out and checked the same way as the steps in
`docs/tutorials/01-hello-triangle` (see its `AGENTS.md`): `setup` and
`frame` functions, region markers for the page, and identical WGSL in
every port. `host.rs` and `host.py` here are copies of that tutorial's
hosts. Run it with `cargo run -p fce --example tutorial_eight_million_particles_splat`
or `python docs/tutorials/eight-million-particles/host.py 01-splat`.

The demo needs WebGPU, so the page passes `webgpu` to `<Demo>`.
