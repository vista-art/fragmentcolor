import FragmentColor

let cs = Shader("@compute @workgroup_size(8,8,1) fn cs_main() {}").unwrap()
let pass = Pass("compute"); pass.addShader(cs)