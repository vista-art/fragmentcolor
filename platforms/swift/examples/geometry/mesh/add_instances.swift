import FragmentColor

let m = Mesh()
let red = [1.0, 0.0, 0.0, 1.0]
let green = [0.0, 1.0, 0.0, 1.0]
let blue = [0.0, 0.0, 1.0, 1.0]
m.addInstances([
    Instance.new().set("tint", red),
    Instance.new().set("tint", green),
    Instance.new().set("tint", blue),
])