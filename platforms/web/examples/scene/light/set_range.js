import { Light } from "fragmentcolor";

const lamp = Light.point([0.0, 2.0, 0.0], [1.0, 1.0, 1.0]);
lamp.setRange(8.0);
const negative = lamp.setRange(-1.0);

// Directional lights have no range — the call errors.
const sun = Light.directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0]);
const unsupported = sun.setRange(5.0);