import { Light } from "fragmentcolor";

const torch = Light.spot([0.0, 1.8, 1.0], [0.0, -1.0, 0.0], [1.0, 1.0, 1.0]).setConeAngles(0.15, 0.4);
const sun = Light.directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0]);