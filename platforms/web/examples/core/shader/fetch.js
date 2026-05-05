
import { Shader } from "fragmentcolor";

// Full registry URL.
const shader = await Shader.fetch("https://fragmentcolor.org/shaders/sdf2d/circle.wgsl");

// Equivalent shorthand using the registry slug.
const shader2 = await Shader.fetch("sdf2d/circle");
