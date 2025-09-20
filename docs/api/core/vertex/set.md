# Vertex::set(key: string, value: any)

Planned API (draft)

Associate a property with the vertex and return a builder that can pin the property to a specific shader location using `.at(index)`.

Behavior
- If the key is new, the vertex assigns the next available `@location(N)` to this property automatically. Subsequent calls reuse the same location unless you override with `.at(index)`.
- At render time, shader vertex inputs (declared with `@location(N)`) are matched to Vertex/Instance properties by:
  1) explicit location (instance first, then vertex), then
  2) name (instance first, then vertex).

Notes
- This API complements `Vertex::with(key, value)`, which is sugar for auto‑assigning the next location without explicitly pinning it.
- For explicit control, call `.at(index)` on the returned builder.

## Example

```rust
use fragmentcolor::mesh::Vertex;

// Draft fluent API (subject to change)
let mut v = Vertex::new([0.0, 0.0]);

// Assign values and pin shader locations explicitly
v.set("uv", [0.5, 0.5]).at(1)
 .set("color", [1.0, 0.0, 0.0, 1.0]).at(2);

// Or mix with auto-indexing via `with`
let v2 = Vertex::new([0.0, 0.0])
    .with("weight", 1.0); // auto assigns next available location
```
