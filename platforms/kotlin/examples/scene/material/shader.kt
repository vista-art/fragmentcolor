import org.fragmentcolor.*

// Direct uniform access for a custom field that isn't covered by the
// Material setters or by Camera / Light.
val material = Material.pbr()
material.shader().set("material.alpha_cutoff", 0.25)