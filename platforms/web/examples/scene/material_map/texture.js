import { PbrMaterial, Renderer, ToonMaterial } from "fragmentcolor";

const renderer = new Renderer();
const albedo_bytes = [ 255, 200, 120, 255, 255, 240, 180, 255, 230, 180, 100, 255, 255, 220, 150, 255, ];
const albedo = await renderer.createTexture(albedo_bytes, [2, 2]);

// Every material whose map points at `albedo` reuses the same uploaded GPU
// texture; passing the same handle to N maps costs one upload and N
// shader-uniform references.
const material = new PbrMaterial();
const map = material.baseColorMap().texture(albedo);

// Raw pixels with their size upload when the scene loads.
const tint = [255, 128, 0, 255];
const tinted = new PbrMaterial();
const tinted_map = tinted.baseColorMap().texture(tint, [1, 1]);

// A toon material samples the tinted texture too, which each Renderer
// uploads once for both maps.
const toon = new ToonMaterial();
const toon_map = toon.baseColorMap().texture(tinted.baseColorMap());