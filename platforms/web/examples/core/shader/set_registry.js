
import { Shader } from "fragmentcolor";

// Point at your own mirror of the registry
Shader.setRegistry("https://cdn.example.com/shaders/");

// Now the slug "sdf2d/circle" resolves to https://cdn.example.com/shaders/sdf2d/circle.wgsl
// (Skipping the actual fetch in this doctest)