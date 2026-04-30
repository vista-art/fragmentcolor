# Renderer::create_external_texture_mobile()

iOS / Android shim for `Renderer::create_external_texture`. The Web binding
takes an `HTMLVideoElement`; uniffi-marshaled languages take a raw `u64`
pointer to a `CVPixelBuffer` (iOS) or a `SurfaceTexture` (Android), since
uniffi cannot marshal those types directly. Currently returns
`FragmentColorError::Render("not implemented yet")` until the per-platform
plumbing to convert the native source into a `wgpu::ExternalTexture`
lands.

## Example

```rust
// hidden file; no public example (platform-specific)
```
