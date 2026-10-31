# Renderer::set_log_level(level: LogLevel)

Sets how much FragmentColor logs, for the whole process. The `LogLevel` values run from quiet to verbose: `Off`, `Error`, `Warn`, `Info`, `Debug` and `Trace`.

The web build prints to the browser console and starts at `Info`. Raise it to `Debug` to see naga's notes on each shader. On other platforms FragmentColor logs through Rust's `log` crate, and the level limits what an installed logger prints.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{LogLevel, Renderer, Shader};

// Log naga's notes while this shader builds, then go back to the usual level.
Renderer::set_log_level(LogLevel::Debug);
let shader = Shader::default();
Renderer::set_log_level(LogLevel::Info);
# let _ = shader;
# Ok(())
# }
```
