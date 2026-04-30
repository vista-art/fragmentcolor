
import FragmentColor

// Point at your own mirror of the registry
Shader.setRegistry("https://cdn.example.com/shaders/")

// Now """sdf2d/circle""" resolves to https://cdn.example.com/shaders/sdf2d/circle.wgsl
// (Skipping the actual fetch in this doctest)