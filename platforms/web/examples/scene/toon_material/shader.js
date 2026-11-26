import { ToonMaterial } from "fragmentcolor";

const material = new ToonMaterial();
material.shader().set("material.alpha_cutoff", 0.25);