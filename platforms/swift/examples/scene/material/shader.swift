import FragmentColor

// Direct uniform access for a custom field that isn't covered by the
// Material setters or by Camera / Light.
let material = Material.pbr()
try material.shader().set("material.alphaCutoff", 0.25)