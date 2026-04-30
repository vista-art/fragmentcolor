from fragmentcolor import Mesh, Instance

m = Mesh()
red = [1.0, 0.0, 0.0, 1.0]
green = [0.0, 1.0, 0.0, 1.0]
blue = [0.0, 0.0, 1.0, 1.0]
m.add_instances([
    Instance().set("tint", red),
    Instance().set("tint", green),
    Instance().set("tint", blue),
])