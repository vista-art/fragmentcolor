from fragmentcolor import Mesh, Instance

m = Mesh()
m.add_instances([
    Instance().set("tint", [1.0, 0.0, 0.0, 1.0]),
    Instance().set("tint", [0.0, 1.0, 0.0, 1.0]),
    Instance().set("tint", [0.0, 0.0, 1.0, 1.0]),
])