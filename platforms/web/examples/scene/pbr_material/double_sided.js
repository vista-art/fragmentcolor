import { AlphaMode, PbrMaterial, Renderer } from "fragmentcolor";

const renderer = new Renderer();
// Leaf cards: thin, single-quad geometry; needs both sides + alpha cut-out.
const leaf = new PbrMaterial().doubleSided(true).alphaMode(AlphaMode.Mask).alphaCutoff(0.5);

// The default is single-sided, with back-face culling on.
const solid_mesh = new PbrMaterial().doubleSided(false);