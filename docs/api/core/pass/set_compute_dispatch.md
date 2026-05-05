# Pass::set_compute_dispatch

Set the workgroup-grid size for a compute pass. The three numbers are the
number of workgroups to dispatch in each dimension; total invocations are
`x * y * z * workgroup_size_x * workgroup_size_y * workgroup_size_z`,
where the per-workgroup size comes from the WGSL `@workgroup_size`
attribute. Has no effect on render passes.

## Example

```rust
use fragmentcolor::{Pass, Shader};
let cs = Shader::new("@compute @workgroup_size(8,8,1) fn cs_main() {}").unwrap();
let pass = Pass::from_shader("compute", &cs);
pass.set_compute_dispatch(64, 64, 1);
```
