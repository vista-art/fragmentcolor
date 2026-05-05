
import FragmentColor

// Full registry URL.
let shader = try await Shader.fetch("https://fragmentcolor.org/shaders/sdf2d/circle.wgsl")

// Equivalent shorthand using the registry slug.
let shader2 = try await Shader.fetch("sdf2d/circle")