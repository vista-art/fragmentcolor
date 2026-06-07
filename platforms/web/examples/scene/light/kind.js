import { Light, LightKind } from "fragmentcolor";

const sun = Light.directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0]);
const bulb = Light.point([0.0, 2.5, 0.0], [1.0, 1.0, 1.0]);
const torch = Light.spot([0.0, 1.8, 1.0], [0.0, -1.0, 0.0], [1.0, 1.0, 1.0]);