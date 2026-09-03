# Tools

Build, test, and release helpers for the repository. Run them through the
dispatcher at the root, which lists everything when called without arguments:

```bash
./fc
./fc test
./fc healthcheck web
```

| Tool | What it does |
| --- | --- |
| `build_web` | Builds the npm package with wasm-pack into `platforms/web/pkg` (`--debug` for a debug build) |
| `build_py` | Builds Python wheels with maturin |
| `build_ios` | Builds the iOS xcframework and regenerates the Swift bindings |
| `build_android` | Builds the Android `.so` per ABI and regenerates the Kotlin bindings |
| `sync_js` | Syncs the JavaScript lockfiles to the published package floor |
| `check` | Pre-push validator: formatting, clippy, tests, and lockfile checks |
| `clippy` | Formats, then runs clippy across the workspace (`fix` to autofix) |
| `test` | Runs clippy, every workspace test, and the crate doctests |
| `coverage` | Runs `cargo llvm-cov` for the workspace |
| `healthcheck` | Runs the platform healthchecks (`web`, `py`, `ios`, `android`, or all) |
| `run_web` | Serves the web examples (`gallery`, `repl`, or `visual`) |
| `run_py` | Builds the wheel into a venv and runs a Python example |
| `run_docs` | Serves the documentation website (`preview` for the built site) |
| `example` | Interactive picker for the Rust examples |
| `bump_version` | Bumps the crate version across every manifest and badge |

`js/` holds the Node helpers behind `sync_js` and the docs dependency gate;
`swift/` holds the macOS stub compiled by `build_ios`. The Rust modules that
`build.rs` includes stay in `scripts/`.
