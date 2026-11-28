import FragmentColor

// Direct uniform access for a custom field that isn't covered by the
// PbrMaterial setters or by Camera / Light.
let material = PbrMaterial()
try material.shader().set("material.alpha_cutoff", 0.25)