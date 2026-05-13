import org.fragmentcolor.*

// Direct uniform access for a custom field that isn't covered by the
// Material setters or by Camera / Light.
val renderer = Renderer()
val material = Material.pbr(renderer)
material.shader().set("material.alphaCutoff", 0.25)