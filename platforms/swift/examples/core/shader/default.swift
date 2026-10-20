
import FragmentColor

let shader = Shader.default()

// The default shader includes a "resolution" uniform
try shader.set("resolution", [800.0, 600.0])
let res = try shader.get("resolution")