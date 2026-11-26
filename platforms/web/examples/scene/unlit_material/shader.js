import { UnlitMaterial } from "fragmentcolor";

const material = new UnlitMaterial();
material.shader().set("material.alpha_cutoff", 0.25);