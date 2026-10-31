# Renderer::set_log_level(level: LogLevel)

Sets how much FragmentColor logs, for the whole process. The `LogLevel` values run from quiet to verbose: `Off`, `Error`, `Warn`, `Info`, `Debug` and `Trace`.

The web build prints to the browser console and starts at `Info`. Raise it to `Debug` to see naga's notes on each shader. On other platforms FragmentColor logs through Rust's `log` crate, and the level limits what an installed logger prints.

## Example

```rust
use fragmentcolor::{LogLevel, Renderer};

Renderer::set_log_level(LogLevel::Debug);
```
