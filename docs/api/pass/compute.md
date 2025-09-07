# compute(name: &str) -> Pass

Creates a new [Pass](https://fragmentcolor.org/api/pass) configured for compute workloads.

Only [Shader](https://fragmentcolor.org/api/shader) objects that compile to compute pipelines can be added.

## Example

```rust
use fragmentcolor::Pass;

let pass = Pass::compute("compute pass");
// Add compute shaders once available
```
