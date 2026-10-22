# Scene::load_bytes_mobile()

Mobile form of `Scene::load` for in-memory files. It takes the bytes of a
`.glb` file, `Data` on Swift and a `ByteArray` on Kotlin, and loads them the
way `Scene::load(bytes)` does in Rust.

uniffi can't overload a name, so the path form stays `Scene.load(path)` and
this one is `Scene.loadBytes(bytes)`. The Swift and Kotlin extensions add a
`Scene.load(bytes)` overload on top, so both forms read like every other
platform.

## Example

```rust
// hidden file; no public example
```
