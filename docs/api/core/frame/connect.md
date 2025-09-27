# Frame::connect(parent, child)

Connect a dependency edge from parent -> child inside a Frame DAG.

- Errors when either pass is missing, the edge already exists, or a cycle would be detected at execution time.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Frame, Pass};

let p1 = Pass::new("shadow");
let p2 = Pass::new("main");

let mut frame = Frame::new();
f.add_pass(&p1);
f.add_pass(&p2);

frame.connect(&p1, &p2)?;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```