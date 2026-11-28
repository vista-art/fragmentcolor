import { AlphaMode, PbrMaterial } from "fragmentcolor";

const foliage = new PbrMaterial().alphaMode(AlphaMode.Mask).alphaCutoff(0.3);

const glass = new PbrMaterial().baseColor([0.9, 0.95, 1.0, 0.25]).alphaMode(AlphaMode.Blend);

const solid = new PbrMaterial().alphaMode(AlphaMode.Opaque);