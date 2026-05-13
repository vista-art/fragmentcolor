import { AlphaMode, Material, Renderer } from "fragmentcolor";

const renderer = new Renderer();
// Leaf cards: thin, single-quad geometry; needs both sides + alpha cut-out.
const leaf = Material.pbr()?.doubleSided(true).alphaMode(AlphaMode.Mask).alphaCutoff(0.5);

// Default is single-sided — back-face culling on.
const solid_mesh = Material.pbr()?.doubleSided(false);