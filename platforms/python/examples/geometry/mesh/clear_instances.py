from fragmentcolor import Mesh, Instance

m = Mesh()
m.add_instance(Instance().set("tint", [1.0, 0.0, 0.0, 1.0]))
m.clear_instances() # back to a single uninstanced draw