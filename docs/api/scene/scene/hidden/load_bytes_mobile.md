# Scene::load_bytes_mobile()

Generated uniffi plumbing for `Scene::load` with in-memory files. Call
[`Scene.load(bytes)`](https://fragmentcolor.org/api/scene/scene#load)
instead: the Swift and Kotlin extensions add that overload on top of this
method, so the call reads like every other platform.

It takes the bytes of a `.glb` file, `Data` on Swift and a `ByteArray` on
Kotlin. uniffi can't overload a name, so the path form stays
`Scene.load(path)` and this raw form is `Scene.loadBytes(bytes)`.

## Example

```rust
// hidden file; no public example
```
