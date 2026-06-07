# 01-hello-triangle

This directory holds only the **code** for the six steps of the tutorial.
Each step has an `example.rs` (canonical Rust source, registered as a
`cargo --example` target) and an `example.js` (hand-written JS port the
docs-site `<Demo>` component imports).

The **prose** for this tutorial lives in
`docs/website/src/content/docs/tutorials/01-hello-triangle.mdx` so the
Starlight-rendered page is the single source of truth (no duplication
between a markdown file in `docs/tutorials/` and an MDX file under
`docs/website/`).

## Region markers

Every `example.{rs,js}` uses VSCode-native region markers that the build
script (`scripts/tutorials.rs`) parses to populate the `<Snippet>` tabs:

```rust
// #region: setup
... visible in the snippet ...
// #endregion: setup
```

Region names are arbitrary; the MDX references them by `region="..."`
prop. Snippets with the same region name in different files are
distinguished by the `file=` prop.
