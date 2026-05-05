# Renderer.create_external_texture(source)

The Python binding does not currently expose `create_external_texture` —
Python lacks a portable native video-frame source equivalent to the
Web `HTMLVideoElement` / iOS `CVPixelBuffer` / Android `SurfaceTexture`.
Decode video frames host-side and upload via `Texture.write()` /
`Texture.write_region()` instead.

## Example

```python
# Python: decode video frames host-side and upload via Texture.write().
# A native external-texture source equivalent is not exposed on this binding.
```
