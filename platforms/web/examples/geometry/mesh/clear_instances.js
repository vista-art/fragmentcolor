import { Mesh, Instance } from "fragmentcolor";

const m = new Mesh();
const red = [1.0, 0.0, 0.0, 1.0];
m.addInstance(new Instance().set("tint", red));
m.clearInstances(); // back to a single uninstanced draw;