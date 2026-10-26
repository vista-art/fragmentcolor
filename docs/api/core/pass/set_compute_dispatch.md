# Pass::set_compute_dispatch

Set the workgroup-grid size for a compute pass. The three numbers are the
number of workgroups to dispatch in each dimension; total invocations are
`x * y * z * workgroup_size_x * workgroup_size_y * workgroup_size_z`,
where the per-workgroup size comes from the WGSL `@workgroup_size`
attribute. A 0 in any dimension dispatches nothing, and the pass's storage keeps
what it holds. Has no effect on render passes.

Each count must fit the device, which takes at most 65535 workgroups per
dimension on most GPUs. A larger one makes `render` return
`PassError::DispatchLimit`, which names the counts and the limit, before
anything runs.

One shader can't read a texture it writes in the same dispatch. A shader that
samples a texture and stores into it through a storage binding returns
`PassError::TextureConflict`, which names both bindings. Read it in a later pass.

## Example

```rust
use fragmentcolor::{Pass, Shader};
let cs = Shader::new("@compute @workgroup_size(8,8,1) fn cs_main() {}").unwrap();
let pass = Pass::from_shader("compute", &cs);
pass.set_compute_dispatch(64, 64, 1);
```
