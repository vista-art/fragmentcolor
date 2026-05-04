
import FragmentColor

// Single URL
let shader = try await Shader.fetch("https://fragmentcolor.org/shaders/sdf2d/circle.wgsl")

// Registry slug
let shader2 = try await Shader.fetch("sdf2d/circle")