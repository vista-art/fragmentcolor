import FragmentColor

let cs = try! Shader("@compute @workgroup_size(8,8,1) fn cs_main() {}")
let pass = Pass("compute"); pass.addShader(cs)