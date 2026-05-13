import { AlphaMode, Material, Renderer } from "fragmentcolor";

const renderer = new Renderer();
const foliage = await Material.pbr(renderer).alphaMode(AlphaMode.Mask).alphaCutoff(0.3);

const glass = await Material.pbr(renderer).baseColor([0.9, 0.95, 1.0, 0.25]).alphaMode(AlphaMode.Blend);

const solid = await Material.pbr(renderer).alphaMode(AlphaMode.Opaque);