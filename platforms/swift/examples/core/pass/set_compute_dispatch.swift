import FragmentColor
let cs = try! Shader("@compute @workgroup_size(8,8,1) fn cs_main() {}")
let pass = Pass.fromShader("compute", cs)
pass.setComputeDispatch(64, 64, 1)