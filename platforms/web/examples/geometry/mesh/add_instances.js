import { Mesh, Instance } from "fragmentcolor";

const m = new Mesh();
m.addInstances([ new Instance().set("tint", [1.0, 0.0, 0.0, 1.0]), new Instance().set("tint", [0.0, 1.0, 0.0, 1.0]), new Instance().set("tint", [0.0, 0.0, 1.0, 1.0]), ]);