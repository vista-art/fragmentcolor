import { Light } from "fragmentcolor";

const warm = Light.directional([0.0, -1.0, 0.0], [1.0, 0.85, 0.7]);
const color = warm.color();