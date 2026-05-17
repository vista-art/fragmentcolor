import { Light } from "fragmentcolor";

const sun = Light.directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0]);
sun.setDirection([0.3, -0.8, -0.5]);

// Point lights have no direction — the call errors.
const lamp = Light.point([0.0, 2.0, 0.0], [1.0, 1.0, 1.0]);
const result = lamp.setDirection([0.0, -1.0, 0.0]);