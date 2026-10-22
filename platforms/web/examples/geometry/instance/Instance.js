import { Instance, Mesh } from "fragmentcolor";

const mesh = new Mesh();
mesh.addVertex([0.0, 0.0]);

const instance = new Instance().set("offset", [0.5, 0.0]);
mesh.addInstance(instance);