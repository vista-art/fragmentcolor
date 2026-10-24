# Vertex::set_mobile(key, value)

Uniffi-friendly `set` method for Swift (iOS) and Kotlin (Android).

The raw method takes a list of 1 to 4 numbers, `[Double]` in Swift and
`List<Double>` in Kotlin, and throws for any other length. The Swift and
Kotlin extensions add `vertex.set(key, value)` overloads for each number
type and number list. The shader that reads the attribute decides its
type when the mesh attaches. Like every `set`, it returns a new `Vertex`.
Hidden from the public website.

## Example

```rust
// hidden file; no public example
```
