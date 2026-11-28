import FragmentColor

let material = UnlitMaterial()
try material.shader().set("material.alpha_cutoff", 0.25)