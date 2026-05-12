import { Material } from "fragmentcolor";

const material = Material.pbr();
material.shader().set( "camera.viewProj", [ [1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0], ], );
material.shader().set("camera.position", [0.0, 0.0, 5.0]);