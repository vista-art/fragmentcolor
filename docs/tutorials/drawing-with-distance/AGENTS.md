# drawing-with-distance

Code for the seven steps of the "Drawing with distance" tutorial. The prose
lives in `docs/website/src/content/docs/tutorials/drawing-with-distance.mdx`.

Each step folder holds an `example.rs`, registered as a `cargo --example` target
in `examples/rust/Cargo.toml`, and an `example.js`, the port the page's `<Demo>`
runs. Both carry the same WGSL, character for character. A change to one shader
goes into the other in the same commit.

## Regions

Region markers follow `docs/tutorials/01-hello-triangle/AGENTS.md`, with one
addition: markers inside the WGSL string pick the parts of the shader the page
shows. The MDX reads them through its `region(step, name)` helper and renders
them as WGSL with Starlight's `<Code>`. At run time they are ordinary comments.

The host regions, `setup` and `frame`, hold FragmentColor calls and nothing else,
because `<Snippet>` transpiles them line by line into Python, Swift and Kotlin.
Window, event and clock code, the pointer included, stays outside them.

## Comments in the WGSL

The JavaScript files embed the WGSL in a template literal, so the WGSL never
holds a backtick or `${`.
