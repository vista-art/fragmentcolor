# Pass::require(deps)

Declare that this pass depends on one or more other renderables (Pass, Shader, Frame, Mesh).
All dependencies will render before this Pass.

## Return value

- Ok(()) on success
- Err(PassError::SelfDependency) if a pass requires itself
- Err(PassError::DuplicateDependency(name)) if the dependency is already present
- Err(PassError::DependencyCycle { via }) if adding the dependency would create a cycle

## Description

`require` establishes a dependency: the given `dependencies` must render before `self`.

This allows you to build DAG render graphs directly from passes.
The graph is validated at build time and does not perform cycle checks at render time.

- Dependencies are stored in insertion order.
- Traversal is dependencies-first, then the current pass, with deduplication.
- Creation order of passes does not matter.

## Examples

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Pass, Renderer};
let renderer = Renderer::new();
let target = renderer.create_texture_target([100,100]).await?;
let color = Pass::new("color");
let blurx = Pass::new("blur_x");
blurx.require(&color)?; // color before blur_x
let blury = Pass::new("blur_y");
blury.require(&blurx)?; // blur_x before blur_y
let compose = Pass::new("compose");
compose.require(&color)?;
compose.require(&blury)?; // fan-in; color and blur_y before compose
renderer.render(&compose, &target)?; // compose renders last
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
