from fragmentcolor import Mesh, Instance

m = Mesh()
offset = [0.25, 0.10]
tint = [1.0, 0.0, 0.0, 1.0]
m.add_instance(Instance().set("offset", offset).set("tint", tint))