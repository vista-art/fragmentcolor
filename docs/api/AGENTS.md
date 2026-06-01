# docs/api — agent rules

This directory holds the canonical Rust documentation that drives every
binding (Python, JS, Swift, Kotlin) and the public website. The Rust
codefences here double as `cargo test --doc` doctests AND as transpiler
input. Patterns that read clean in idiomatic Rust often translate badly
into the other languages. The rules below keep the source authorable
without breaking transpilation.

## Numeric literals

**Never write type suffixes on numeric literals.** Bare numbers only.

```
GOOD: Camera::perspective(60.0.to_radians(), 16.0 / 9.0, 0.1, 100.0)
BAD : Camera::perspective(60.0_f32.to_radians(), 16.0 / 9.0, 0.1, 100.0)

GOOD: renderer.create_texture_target([16, 16]).await?
BAD : renderer.create_texture_target([16u32, 16u32]).await?

GOOD: mesh.set_indices([0, 1, 2, 0, 2, 3])
BAD : mesh.set_indices([0u32, 1u32, 2u32, 0u32, 2u32, 3u32])
```

Forbidden suffix patterns (both underscore-prefixed and bare):
`_f32`, `_f64`, `_u8`, `_u16`, `_u32`, `_u64`, `_usize`, `_i8`, `_i16`,
`_i32`, `_i64`, `_isize`, `_u128`, `_i128`, `f32`, `f64`, `u8`, `u16`,
`u32`, `u64`, `usize`, `i8`, `i16`, `i32`, `i64`, `isize`, `u128`,
`i128` when attached to a numeric literal.

Why: every FragmentColor API that takes a numeric type accepts bare
literals — Rust's type inference picks the right f32 / u32 / etc. from
the function signature. The suffix is implementation detail that leaks
into the website, JS, Python, Swift, and Kotlin tabs. Stripping it from
the source removes a class of transpiler bugs at the root.

Verification: `cargo test --doc` still passes; the language tabs in the
generated website show the same clean literal.

## Hidden lines

Lines prefixed with `# ` inside a Rust codefence are doctest scaffolding
(an `async fn run() { ... }` wrapper, an executor like
`pollster::block_on(run())`, error-case demonstrations the doctest is
expected to evaluate but the website should not show).

Hidden lines belong at the **start and end** of the codefence only,
never interleaved with the visible body. The website renderer trims them
before display, and the transpilers translate only the visible portion.

## Method and constructor shapes

The transpiler can map straightforward call shapes (`Type::new(args)`,
`obj.method(args)`, `Type::static_method(args)`, chained
`.foo(...).bar(...)`). It cannot reasonably translate:

- `if let Some(x) = expr { ... }` — use a `for x in expr` loop or hoist
  the bind into a `let x = expr.first()` form.
- Iterator chains like `.into_iter().next()` for the "take first" case;
  prefer `.first()` or just iterate.
- Range expressions `0..N` in `for` headers; rewrite as an iterator
  source the binding already exposes.
- Rust's `std::` paths (`std::f32::consts::FRAC_PI_4`,
  `std::fs::read`); hide them in `#` lines and pass a named local that
  the transpiler can stub per language.

When a pattern fights the transpiler, **change the example's shape**
before trying to extend the transpiler. The Rust doc reads slightly
differently; every other language reads correctly.

## Cross-cutting invariants

- Every public item must have a `## Example` section, an `#[lsp_doc(…)]`
  pointer from the Rust source, and a working transpiled example in
  every binding that exposes it. CI hard-fails on missing pieces.
- Drift tolerance is forbidden: when a transpiled example fails to
  compile, fix the source (or the transpiler), never the CI gate.
- Cross-platform parity is a hard guarantee. If a shape works in Rust
  but not in Kotlin / Swift / Python / JS, that is a bug to fix, not a
  caveat to document.

## See also

- Project-root `AGENTS.md` — the broader contributor guide.
- `scripts/convert.rs`, `scripts/kotlin.rs`, `scripts/swift.rs` — the
  transpilers that consume what gets written here.
- `~/.claude/projects/.../memory/transpiler_v2_design.md` — design notes
  for the next-generation per-language harness scheme.
