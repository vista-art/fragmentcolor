import FragmentColor

let mesh = Mesh()
try mesh.addVertex([0.0, 0.0])

let instance = try Instance().set("offset", [0.5, 0.0])
mesh.addInstance(instance)