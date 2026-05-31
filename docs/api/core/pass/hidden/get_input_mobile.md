# Pass::getInput (mobile)

Mobile binding for `Pass::get_input`. Returns a `MobilePassInput` record
instead of `PassInput` because `PassInput` contains a `Color(u32)` tuple
newtype that uniffi's `Record` derive does not support.

`MobilePassInput` has the same semantics:
- `load: Bool`: `true` means "load previous frame contents"; `false` means "clear".
- `colorRgba: UInt32`: packed clear colour as `0xRRGGBBAA`.

## Example

```rust
// hidden mobile binding; no public example
```
