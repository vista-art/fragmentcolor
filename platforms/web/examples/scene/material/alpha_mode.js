import { AlphaMode, Material } from "fragmentcolor";

const foliage = Material.pbr().alphaMode(AlphaMode.Mask).alphaCutoff(0.3);

const glass = Material.pbr().baseColor([0.9, 0.95, 1.0, 0.25]).alphaMode(AlphaMode.Blend);

const solid = Material.pbr().alphaMode(AlphaMode.Opaque);