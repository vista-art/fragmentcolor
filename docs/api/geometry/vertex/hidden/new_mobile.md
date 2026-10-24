# Vertex::new_mobile(position)

Uniffi-friendly constructor for Swift (iOS) and Kotlin (Android).

Accepts a `Vec<f32>` of 1 to 4 components; any other length throws. Hidden
from the public website. uniffi makes a constructor named `new` the primary
constructor, so Swift and Kotlin users call `Vertex(position)`, and the
extensions add overloads for `[Double]`, `DoubleArray` and other number lists.

## Example

```rust
// hidden file; no public example
```
