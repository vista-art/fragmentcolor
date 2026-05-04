
import { Shader } from "fragmentcolor";

// Single URL
const shader = await Shader.fetch("https://fragmentcolor.org/shaders/sdf2d/circle.wgsl");

// Registry slug
const shader2 = await Shader.fetch("sdf2d/circle");
