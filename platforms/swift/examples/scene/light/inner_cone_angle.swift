import FragmentColor

let torch = Light.spot([0.0, 1.0, 0.0], [0.0, -1.0, 0.0], [1.0, 1.0, 1.0]).setConeAngles(0.2, 0.5)
let inner = torch.innerConeAngle()