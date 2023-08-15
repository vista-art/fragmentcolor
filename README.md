# PL Video Processor (vip)

This is the future home for the **PL Video Processor** library, or `vip` for short.

Please check our [First RFC](./rfc/0001-pl-video-processor-proposal.md) for more information about this project.

Contributions are welcome! Please open a PR with changes in the RFC, or reach **@dan** or **@rgb** on Discord.

## Building this project

### Target: web browser / wasm

Make sure you have [wasm-pack](https://rustwasm.github.io/wasm-pack/installer) installed.

```bash
wasm-pack build --release --target web
```

The library will be available in `pkg/`.

Check the usage example in `index.html`.

### Target: desktop window

Running without building:

```bash
cargo run
```

Building:

```bash
cargo build --release
```

The dynamic library will be available in `target/release/`.

By default, the library will be built for the current platform. To build for a specific platform, use the `--target` flag:

```bash
# MacOS (Intel)
cargo build --release --target x86_64-apple-darwin

# MacOS (Apple Silicon)
cargo build --release --target aarch64-apple-darwin

# Linux
cargo build --release --target x86_64-unknown-linux-gnu

# Windows
cargo build --release --target x86_64-pc-windows-msvc
```

You can check the list of all available targets with:

```bash
rustup target list
```

Platform support is divided in Tiers, check the [Rust Platform Support](https://doc.rust-lang.org/nightly/rustc/platform-support.html) page for more information.

### Target: Python Library

Python support is planned for the near future. Please check the [First RFC](./rfc/0001-pl-video-processor-proposal.md) for more information.
