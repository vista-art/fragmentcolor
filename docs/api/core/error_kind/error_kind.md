# ErrorKind

Names the kind of GPU error a [DeviceError](https://fragmentcolor.org/api/core/deviceerror) carries, from the error scope that caught it.

- `Validation`: a call broke a rule of the GPU API, such as a bind group that doesn't match its layout.
- `Internal`: the GPU or its driver failed for a reason of its own, such as a shader the backend compiler refuses.
- `Memory`: the device ran out of memory.

Running out of memory or an internal failure leaves an object invalid, and each later use of that object fails validation. So when one error causes others, the error that returns is the first in the order memory, internal, validation.

Rust writes `ErrorKind::Memory`, JavaScript and Python `ErrorKind.Memory`, Swift `ErrorKind.memory` and Kotlin `ErrorKind.MEMORY`.

## Example

```rust
use fragmentcolor::ErrorKind;

let kind = ErrorKind::Memory;
# assert_ne!(kind, ErrorKind::Validation);
```
