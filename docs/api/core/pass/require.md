# Pass::require(deps)

Declare that this pass depends on one or more other passes. All required passes will render before this pass.

## Syntax

```rust
pub fn require<I, P>(&self, deps: I) -> Result<(), PassError>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Pass>,
```

## Parameters

- deps: One or more Pass values (e.g., `&pass`, `[&a, &b]`, `vec![&a, &b]`).

## Return value

- Ok(()) on success
- Err(PassError::SelfDependency) if a pass requires itself
- Err(PassError::DuplicateDependency(name)) if the dependency is already present
- Err(PassError::DependencyCycle { via }) if adding the dependency would create a cycle

## Description

`require` establishes a dependency: the given `deps` must render before `self`.
This allows you to build DAG render graphs directly from passes.
The graph is validated at build time and does not perform cycle checks at render time.

- Dependencies are stored in insertion order.
- Traversal is dependencies-first, then the current pass, with deduplication.
- Creation order of passes does not matter.

## Examples

```rust
use fragmentcolor:{Pass, Renderer}
# let renderer = Renderer::headless();
# let target = renderer.create_texture_target([100,100]);
let color = Pass::new("color");
let blurx = Pass::new("blur_x");
blurx.require(&color)?; // color before blur_x
let blury = Pass::new("blur_y");
blury.require(&blurx)?; // blur_x before blur_y
let compose = Pass::new("compose");
compose.require([&color, &blury])?; // fan-in; color and blur_y before compose
renderer.render(&compose, &target)?; // compose renders last
```
