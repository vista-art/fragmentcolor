import { Mesh, Instance } from "fragmentcolor";

const m = new Mesh();
const red = [1.0, 0.0, 0.0, 1.0];
const green = [0.0, 1.0, 0.0, 1.0];
const blue = [0.0, 0.0, 1.0, 1.0];
m.addInstances([
    Instance.new().set("tint", red),
    Instance.new().set("tint", green),
    Instance.new().set("tint", blue),
]);