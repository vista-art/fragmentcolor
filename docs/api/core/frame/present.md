# Frame::present(pass)

Select which render pass presents to the final Target when rendering a Frame DAG.

- Must be a render pass and a DAG leaf.
- At most one present pass per Frame.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Frame, Pass};

let shadow = Pass::new("shadow");
let main = Pass::new("main");

let mut frame = Frame::new();
frame.add_pass(&shadow);
frame.add_pass(&main);
frame.connect(&shadow, &main)?;
frame.present(&main)?;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```