import FragmentColor

let torch = Light.spot([0.0, 1.8, 1.0], [0.0, -1.0, 0.0], [1.0, 1.0, 1.0])
torch.setConeAngles(0.15, 0.4)

// Non-spot lights error.
let lamp = Light.point([0.0, 0.0, 0.0], [1.0, 1.0, 1.0])
let unsupported = lamp.setConeAngles(0.15, 0.4)