import { Light } from "fragmentcolor";

const lamp = Light.point([0.0, 1.0, 0.0], [1.0, 0.95, 0.8]).setIntensity(12.0);
const scale = lamp.intensity();