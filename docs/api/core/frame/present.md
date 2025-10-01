# Frame::present(pass)

Designates which render pass should present its output to the final target.

## Parameters

- `pass` - The render pass to designate for final presentation

## Return Value

- `Ok(())` - The pass was successfully set as the present pass
- `Err(FrameError::MissingPass)` - The pass is not present in this frame
- `Err(FrameError::NotRenderPass)` - The pass is a compute pass (only render passes can present)
- `Err(FrameError::NotALeaf)` - Other passes depend on this pass (must be a leaf node)
- `Err(FrameError::InvalidPresentPass)` - A different pass is already designated for presentation

## Description

The `present` method designates which render pass should output to the final render target (typically the screen or a final texture). This pass becomes the "output" of the entire frame.

### Requirements

The present pass must satisfy several requirements:

1. **Must be a render pass** - Only render passes can present output; compute passes cannot
2. **Must be a leaf node** - No other passes can depend on the present pass
3. **Only one per frame** - Each frame can have at most one present pass

## Examples

### Basic Usage

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Frame, Pass};

let shadow = Pass::new("shadow");
let main = Pass::new("main");

let mut frame = Frame::new();
frame.add_pass(&shadow);
frame.add_pass(&main);
main.require(&shadow)?;

// Main pass can present
frame.present(&main)?;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```

### Multi-pass Pipeline

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Frame, Pass};

let geometry = Pass::new("geometry");
let lighting = Pass::new("lighting");
let post_fx = Pass::new("post_effects");

let mut frame = Frame::new();
frame.add_pass(&geometry);
frame.add_pass(&lighting);
frame.add_pass(&post_fx);

// Build pipeline using Pass::require
lighting.require(&geometry)?;
post_fx.require(&lighting)?;

// Final post-effects pass presents to screen
frame.present(&post_fx)?;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```

## Error Conditions

### MissingPass

Occurs when the pass hasn't been added to the frame.

### NotRenderPass

Occurs when trying to set a compute pass as the present pass.

### NotALeaf

Occurs when other passes depend on the chosen pass.

### InvalidPresentPass

Occurs when trying to set a different pass as present when one is already set.

## Notes

- The present pass is automatically marked with a flag that the renderer uses
- Changing the present pass will clear the flag from the previous present pass
- If no present pass is set, the frame may not render visible output
