import { Mesh, Instance } from "fragmentcolor";

const m = new Mesh();
m.addInstance(new Instance().set("tint", [1.0, 0.0, 0.0, 1.0]));
m.clearInstances(); // no instances left