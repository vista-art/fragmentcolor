import { Light } from "fragmentcolor";

const lamp = Light.point([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
lamp.setPosition([3.0, 1.5, -2.0]);

// Directional lights have no position — the call errors.
const sun = Light.directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0]);
const result = sun.setPosition([0.0, 0.0, 0.0]);