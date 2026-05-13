import { AlphaMode, Material, Renderer } from "fragmentcolor";

const renderer = new Renderer();
// Leaf cards: thin, single-quad geometry; needs both sides + alpha cut-out.
const leaf = await Material.pbr(renderer).doubleSided(true).alphaMode(AlphaMode.Mask).alphaCutoff(0.5);

// Default is single-sided — back-face culling on.
const solid_mesh = await Material.pbr(renderer).doubleSided(false);