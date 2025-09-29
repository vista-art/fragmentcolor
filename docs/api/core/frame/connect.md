# Frame::connect(parent, child)

Creates a dependency relationship between two passes in the frame's execution graph.

## Syntax

```rust
pub fn connect(
    &mut self,
    parent: &Pass,
    child: &Pass,
) -> Result<(), FrameError>
```

## Parameters

- `parent` - The pass that must execute before the child pass
- `child` - The pass that depends on the parent pass completing first

## Return Value

- `Ok(())` - The dependency was successfully established
- `Err(FrameError::MissingPass)` - One or both passes are not present in this frame
- `Err(FrameError::DuplicateEdge)` - This dependency relationship already exists

## Description

The `connect` method establishes an execution dependency between two passes. When the frame is rendered, the parent pass will always execute before the child pass. This allows you to build complex rendering pipelines where later passes depend on the results of earlier passes.

The frame maintains a directed acyclic graph (DAG) of pass dependencies. Each edge in this graph represents a "happens-before" relationship between passes.

## Examples

### Basic Usage

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Frame, Pass};

let depth_pass = Pass::new("depth_prepass");
let lighting_pass = Pass::new("lighting");

let mut frame = Frame::new();
frame.add_pass(&depth_pass);
frame.add_pass(&lighting_pass);

// Ensure depth prepass runs before lighting
frame.connect(&depth_pass, &lighting_pass)?;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```

### Complex Pipeline

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Frame, Pass};

let geometry_pass = Pass::new("geometry");
let shadow_pass = Pass::new("shadows");
let lighting_pass = Pass::new("lighting");
let post_process = Pass::new("post_processing");

let mut frame = Frame::new();

// Add all passes
frame.add_pass(&geometry_pass);
frame.add_pass(&shadow_pass);
frame.add_pass(&lighting_pass);
frame.add_pass(&post_process);

// Build dependency chain
frame.connect(&geometry_pass, &shadow_pass)?;
frame.connect(&shadow_pass, &lighting_pass)?;
frame.connect(&lighting_pass, &post_process)?;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```

## Error Conditions

### MissingPass

Occurs when trying to connect passes that haven't been added to the frame.

### DuplicateEdge

Occurs when trying to create the same dependency twice.

## Notes

- The dependency graph must remain acyclic (no circular dependencies)
- Cycles are detected during rendering and will cause fallback to insertion order
- Multiple parents can connect to the same child pass
- A pass can have multiple children
