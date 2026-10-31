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
GOOD: Camera::perspective(1.047, 16.0 / 9.0, 0.1, 100.0)    // 60° FOV
BAD : Camera::perspective(60.0_f32.to_radians(), 16.0 / 9.0, 0.1, 100.0)
BAD : Camera::perspective(60.0.to_radians(), 16.0 / 9.0, 0.1, 100.0)

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

Why: Rust's type inference picks `f32`, `u32` and the rest from the
function signature, so every FragmentColor API takes a bare literal.
A suffix is implementation detail that leaks into every language tab.
The Kotlin transpiler writes the `u` an unsigned parameter needs, as in
`64u`, from the parameter's type, so the source never carries it either.

Verification: `cargo test --doc` still passes, and the generated website
tabs show the same literal, with a `u` on Kotlin's unsigned arguments.

## Angles

Every FragmentColor function that takes an angle takes **radians**.
This is consistent across `Camera::perspective`, `Model::rotate`,
`Material::uv_transform`, `Light::set_cone_angles`, and any future
angle-bearing API. Examples must pass radian literals directly. Add
an inline comment with the degree equivalent for readability when the
intent helps.

```
GOOD: Camera::perspective(1.047, 16.0 / 9.0, 0.1, 100.0)    // 60° FOV
GOOD: Material::pbr().uv_transform([0.5, 0.0], [4.0, 4.0], 0.785)  // 45°
BAD : Camera::perspective(60.0.to_radians(), 16.0 / 9.0, 0.1, 100.0)
BAD : Camera::perspective(std::f32::consts::FRAC_PI_3, …, …, …)
```

Why: `.to_radians()` is a Rust `f32` / `f64` inherent method that has no
equivalent on Python's `float`, Swift's `Double`, or Kotlin's `Float`
without a per-language transpiler map. `std::f32::consts::FRAC_PI_4`
similarly has no clean cross-language translation. A bare numeric
literal works in every language unchanged.

Reference values worth memorizing:

| degrees | radians  | constant   |
| ------- | -------- | ---------- |
| 30°     | 0.524    | π / 6      |
| 45°     | 0.785    | π / 4      |
| 60°     | 1.047    | π / 3      |
| 90°     | 1.571    | π / 2      |

## Hidden lines

Lines prefixed with `# ` inside a Rust codefence are Rust's doctest
scaffolding (an `async fn run() { ... }` wrapper, an executor like
`pollster::block_on(run())`, error-case demonstrations the doctest is
expected to evaluate but the website should not show).

Every other language has hidden lines too, for the setup its
healthcheck file needs: a canvas, a file's bytes, a guard. They sit in
a hidden Rust block comment tagged with the language:

```
# let canvas = fragmentcolor::HeadlessWindow::new([800, 600]);
# /*[js]
# const canvas = document.createElement("canvas");
# */
# /*[python]
# from rendercanvas.auto import RenderCanvas
# canvas = RenderCanvas(size=(800, 600))
# */
```

The tags are `js`, `python`, `swift` and `kotlin`. Rustdoc compiles a
block as a comment, so the doctest still runs. Everything after the
`# ` of a line inside a block is copied as written, Python indentation
included, and no transpiler reads it.

Hidden lines, Rust's and every block, belong at the **start and end**
of the codefence only, never interleaved with the visible body. A
block before the first visible line lands right after the imports of
that language's example, and a block after the last visible line lands
at its end. The website's Rust tab leaves Rust's hidden lines out, as
rustdoc does, and every other tab shows its hidden lines collapsed.
The build fails on a block between visible lines, on an unknown tag,
on a block with no closing `# */`, and on a line inside a block that
holds `/*` or `*/`. Rust nests block comments,
so either one would move the end of the block and swallow the rest of
the doctest.

Platform setup lives in these blocks, never in the transpilers. When
the languages make a value differently (a window, a file's bytes),
make it under one name in every language's hidden lines and use that
name on the visible lines. The visible lines then hold only
FragmentColor's API, and the tabs pair line by line.

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
  every language's hidden block makes.

When a pattern fights the transpiler, **change the example's shape**
before trying to extend the transpiler. The Rust doc reads slightly
differently; every other language reads correctly.

## Links

Each object has one page at `/api/<category>/<object>`, with the
object's name in lower case, and each method's heading there takes the
method's name as its id. Link a method with that name as the anchor:

```
[`Scene::remove_pass`](https://fragmentcolor.org/api/scene/scene#remove_pass)
```

Other headings keep the id Starlight makes from their text. The build
fails on a link to a page or heading the site doesn't build, and on two
headings of one page that would get the same id.

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
- `scripts/example.rs` — splits a codefence into Rust's lines and each
  language's hidden blocks.
- `~/.claude/projects/.../memory/transpiler_v2_design.md` — design notes
  for the next-generation per-language harness scheme.
