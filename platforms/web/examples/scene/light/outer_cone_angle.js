import { Light } from "fragmentcolor";

const torch = Light.spot([0.0, 1.0, 0.0], [0.0, -1.0, 0.0], [1.0, 1.0, 1.0]).setConeAngles(0.2, 0.5);
const outer = torch.outerConeAngle();