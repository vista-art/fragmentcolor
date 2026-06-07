import { Mesh, Instance } from "fragmentcolor";

const m = new Mesh();
const red = [1.0, 0.0, 0.0, 1.0];
const green = [0.0, 1.0, 0.0, 1.0];
const blue = [0.0, 0.0, 1.0, 1.0];
m.addInstances([ new Instance().set("tint", red), new Instance().set("tint", green), new Instance().set("tint", blue), ]);