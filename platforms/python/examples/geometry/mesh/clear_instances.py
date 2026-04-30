from fragmentcolor import Mesh, Instance

m = Mesh()
red = [1.0, 0.0, 0.0, 1.0]
m.add_instance(Instance().set("tint", red))
m.clear_instances(); # back to a single uninstanced draw