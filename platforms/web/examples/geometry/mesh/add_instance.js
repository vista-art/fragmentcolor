import { Mesh, Instance } from "fragmentcolor";

const m = new Mesh();
const offset = [0.25, 0.10];
const tint = [1.0, 0.0, 0.0, 1.0];
m.addInstance(Instance.new().set("offset", offset).set("tint", tint));